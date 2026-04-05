# Specification: Nemesis Audit Findings and Hardening Requirements

**Audit Date:** 2026-04-05  
**Audit Scope:** myosu stage-0 codebase (pallet-game-solver, miner, validator, gameplay, chain-client)  
**Method:** Nemesis-style Feynman + State inconsistency audit, 3-pass convergence

---

## Finding Categories

- **S0** — Critical: Direct consensus/financial exploit path
- **S1** — High: Invariant violation or state corruption risk
- **S2** — Medium: Correctness gap or missing guard
- **S3** — Low: Hardening opportunity or documentation gap

---

## VERIFIED FINDINGS

### NEM-001: GRANDPA Finality Stalls on 3-Node Devnet After Authority Stop

**Severity:** S1  
**Affected Surfaces:** `crates/myosu-chain/node/`, `tests/e2e/`, consensus runtime configuration  
**Triggering Scenario:** Start 3-authority devnet. Stop authority-3. Remaining two authorities continue importing blocks but finalized height freezes at `#2`. Node logs show `Backing off claiming new slot for block authorship: finality is lagging`.  
**Invariant or Assumption That Breaks:** Bootstrap exit criteria requires multi-node GRANDPA finality. The system cannot reach production with this behavior.  
**Why This Matters Now:** Blocks `NET-FINALITY-001` in WORKLIST.md, which gates multi-node production readiness.  
**Discovery Path:** Feynman (WORKLIST.md documents stall; node service GRANDPA configuration review)

**Codebase Evidence:**
- `WORKLIST.md:NET-FINALITY-001`: "Investigate why GRANDPA finality stalls after one authority is stopped"
- `crates/myosu-chain/node/src/devnet.rs` defines 3-authority chain spec
- `crates/myosu-chain/node/src/service.rs` controls GRANDPA configuration
- Repro confirmed 2026-04-05: "all three authorities reached finalized block `#2`, then after terminating `authority-3` the surviving authorities kept importing past block `#10` while finalized height stayed frozen at `#2`"

---

### NEM-002: Cross-Node Emission Agreement Is Unverified

**Severity:** S1  
**Affected Surfaces:** `pallet-game-solver/src/coinbase/`, `pallet-game-solver/src/epoch/`, `tests/e2e/`  
**Triggering Scenario:** Two separate validator nodes produce different `TotalIssuance` values after N epoch transitions on the same 3-node devnet.  
**Invariant or Assumption That Breaks:** INV-005 requires emission accounting determinism across nodes. Cross-node fixed-point determinism of `substrate_fixed` types (U96F32, I64F64) is assumed, not verified.  
**Why This Matters Now:** If emission diverges across nodes, the chain forks or validators disagree on weights, breaking Yuma Consensus and INV-003.  
**Discovery Path:** State (All epoch/coinbase tests run single-node mock runtime)

**Codebase Evidence:**
- `pallet-game-solver/src/coinbase/run_coinbase.rs` uses U96F32 throughout
- `pallet-game-solver/src/utils/try_state.rs` only checks TotalIssuance locally
- No E2E test queries storage across multiple nodes and compares

---

### NEM-003: TotalIssuance Accounting Tolerance Masks Systematic Drift

**Severity:** S2  
**Affected Surfaces:** `pallet-game-solver/src/utils/try_state.rs`, `pallet-game-solver/src/coinbase/run_coinbase.rs`  
**Triggering Scenario:** During `on_idle` block execution, TotalIssuance diverges from expected by less than 1000 RAO. The delta check passes silently. Over many blocks, systematic under/over issuance accumulates.  
**Invariant or Assumption That Breaks:** The accounting invariant that `TotalIssuance == currency_issuance + TotalStake` (modulo rounding). The 1000 RAO tolerance is an intentional fudge factor without documented bounds.  
**Why This Matters Now:** `EM-DUST-001` documents 2 rao/block truncation loss. The tolerance makes it impossible to distinguish bounded drift from unbounded accumulation.  
**Discovery Path:** Feynman (examining `check_total_issuance` implementation)

**Codebase Evidence:**
```rust
// src/utils/try_state.rs:21
let delta = 1000;  // 1000 RAO tolerance
let total_issuance = TotalIssuance::<T>::get().to_u64();
// ... diff calculation ...
ensure!(diff <= delta, "TotalIssuance diff greater than allowable delta");
```
- `cargo test -p pallet-game-solver -- truncation` measures 2 rao/block worst case
- Stage-0 coinbase uses `tou64!` macro truncating U96F32 → u64

---

### NEM-004: Epoch Consistency Guard Emits No Event on Skip

**Severity:** S2  
**Affected Surfaces:** `pallet-game-solver/src/epoch/run_epoch.rs`, `pallet-game-solver/src/coinbase/run_coinbase.rs`  
**Triggering Scenario:** A subnet has duplicate hotkeys or key-index mismatch. When `is_epoch_input_state_consistent()` returns false, the epoch is skipped silently. No chain event is deposited for monitoring systems to observe.  
**Invariant or Assumption That Breaks:** Silent epoch skipping means operators cannot detect when a subnet stops receiving emission.  
**Why This Matters Now:** The consistency guard was added (C-011) to prevent corruption, but skipping without observability means validators and miners stop receiving emission without knowing why.  
**Discovery Path:** Feynman (examining `run_epoch.rs:66-68`)

**Codebase Evidence:**
```rust
// src/epoch/run_epoch.rs:66-68
if !Self::is_epoch_input_state_consistent(netuid) {
    log::error!("Skipping legacy epoch for inconsistent netuid {netuid}");
    return Vec::new();  // No deposit_event
}
```
- grep confirms no `EpochSkipped` or similar event variant exists

---

### NEM-005: Validator L1 Distance Has Implicit Zero-Weight Action Asymmetry

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-validator/src/validation.rs`  
**Triggering Scenario:** A miner returns an action set where expected and observed have different actions. The second pass only counts unexpected actions if `expected == 0.0`, creating asymmetric weighting for edge cases.  
**Invariant or Assumption That Breaks:** The spec claims "L1 distance covers both expected and observed action sets" but the implementation weights asymmetries differently.  
**Why This Matters Now:** A miner could potentially game the scoring by including zero-probability actions in their response.  
**Discovery Path:** Feynman (examining `validation.rs:345-367`)

**Codebase Evidence:**
```rust
// First pass: actions in expected
for (action, prob) in &expected.actions {
    let observed_prob = observed.probability_for(action);
    l1 += (prob - observed_prob).abs();
}
// Second pass: actions only in observed
for (action, prob) in &observed.actions {
    if expected.probability_for(action) == 0.0 {  // IMPLICIT CHECK
        l1 += prob;
    }
}
```

---

### NEM-006: SwapInterface NoOp Stub Returns Unbounded Price

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-chain/runtime/src/lib.rs`, `pallet-game-solver/src/lib.rs`  
**Triggering Scenario:** Stage-0 runtime uses `Stage0NoopSwap` which returns `C::MAX` for `max_price()`. Any swap logic that checks against max price will never trigger price-limit guards.  
**Invariant or Assumption That Breaks:** The swap interface contract expects finite price limits for safety guards.  
**Why This Matters Now:** Stage-0 intentionally has no real swap, but if any code path assumes max_price() returns a realistic bound, it will fail open.  
**Discovery Path:** Cross-feed (AGENTS.md notes + code examination)

**Codebase Evidence:**
```rust
// runtime/src/lib.rs
fn max_price<C: Currency>() -> C { C::MAX }
```
- `pallet-game-solver/src/lib.rs` calls `T::SwapInterface::stage0_max_price()` in emission
- AGENTS.md: "SwapInterface no-op stub (1:1 identity)" — noted as intentional

---

### NEM-007: Liar's Dice Decode Budget 256x Larger Than Poker

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-games-liars-dice/src/wire.rs`  
**Triggering Scenario:** A malicious miner sends a 100MB response for Liar's Dice. The decoder permits up to 256 MiB while poker is limited to 1 MiB.  
**Invariant or Assumption That Breaks:** Consistent decode budget across all game wire formats.  
**Why This Matters Now:** Inconsistent bounds create an attack surface disparity between games.  
**Discovery Path:** Cross-feed (comparing poker and liars-dice wire.rs)

**Codebase Evidence:**
```rust
// poker/src/wire.rs
const MAX_DECODE_BYTES: u64 = 1_048_576;  // 1 MiB

// liars-dice/src/wire.rs
const MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024;  // 256 MiB
```

---

### NEM-008: No Minimum Training Iteration Gate

**Severity:** S3  
**Affected Surfaces:** `crates/myosu-miner/src/training.rs`, `crates/myosu-miner/src/cli.rs`  
**Triggering Scenario:** A miner trains for 1 MCCFR iteration and serves garbage strategy. Validators score it near zero, but the system has no way to distinguish "untrained" from "poorly trained."  
**Invariant or Assumption That Breaks:** Mining quality requires meaningful strategy convergence.  
**Why This Matters Now:** No guidance exists for operators on minimum viable training.  
**Discovery Path:** Feynman (examining training.rs and CLI arguments)

**Codebase Evidence:**
- `crates/myosu-miner/src/training.rs` accepts `--train-iterations` without minimum
- `crates/myosu-miner/src/cli.rs` has `#[arg(long, default_value_t = 0)] train_iterations: usize`

---

### NEM-009: INV-004 Runtime Enforcement Missing

**Severity:** S3  
**Affected Surfaces:** `crates/myosu-miner/`, `crates/myosu-play/`, CI configuration  
**Triggering Scenario:** A refactor accidentally adds a dependency from myosu-play to myosu-miner (or vice versa). CI `cargo tree` check catches it at PR level, but runtime behavior is unguarded.  
**Invariant or Assumption That Breaks:** INV-004: "A gameplay bug must not corrupt training data."  
**Why This Matters Now:** CI enforcement is compile-time only. At runtime, the separation is a cargo-dependency convention, not a hard boundary.  
**Discovery Path:** State (INV-004 enforcement analysis)

**Codebase Evidence:**
- CI workflow runs `cargo tree` check for INV-004
- No runtime assertion or feature flag exists to enforce separation

---

### NEM-010: AGENTS.md RTK.md Reference Documented but Not Present

**Severity:** S3  
**Affected Surfaces:** `WORKLIST.md:DOC-OPS-001`  
**Triggering Scenario:** WORKLIST.md tracks a dangling `@RTK.md` reference but AGENTS.md no longer contains it. The tracking entry itself may be stale.  
**Invariant or Assumption That Breaks:** Documentation tracking must reflect current state.  
**Why This Matters Now:** Creates confusion about whether the issue is resolved or the reference moved.  
**Discovery Path:** Cross-feed (WORKLIST.md vs AGENTS.md top matter comparison)

**Codebase Evidence:**
- `WORKLIST.md:DOC-OPS-001`: "Resolve the dangling `@RTK.md` reference at the top of `AGENTS.md`"
- AGENTS.md top matter contains no `@RTK.md` reference (verified via grep)

---

## VERIFIED SATISFIED (No Action Required)

### NEM-C01: Checkpoint Magic Header Present

**Status:** SATISFIED  
**Evidence:** `crates/myosu-games-poker/src/solver.rs` defines:
```rust
const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOS";
const CHECKPOINT_VERSION: u32 = 1;
```
The `save()` and `load()` methods include magic header validation. Draft finding NEM-011 was incorrect.

---

## SUMMARY TABLE

| ID | Severity | Category | Invariant/Property | Discovery |
|----|----------|----------|-------------------|-----------|
| NEM-001 | S1 | Consensus | 3-node GRANDPA finality | Feynman |
| NEM-002 | S1 | Accounting | Cross-node emission agreement | State |
| NEM-003 | S2 | Accounting | TotalIssuance accounting tolerance | Feynman |
| NEM-004 | S2 | Emission | Epoch skip observability | Feynman |
| NEM-005 | S2 | Scoring | L1 distance asymmetry | Feynman |
| NEM-006 | S2 | Swap | NoOp stub max_price unbounded | Cross-feed |
| NEM-007 | S2 | Wire | Decoder budget inconsistency | Cross-feed |
| NEM-008 | S3 | Mining | No min training gate | Feynman |
| NEM-009 | S3 | Architecture | INV-004 runtime enforcement | State |
| NEM-010 | S3 | Docs | WORKLIST stale tracking | Cross-feed |

---

*This document represents the final Nemesis synthesis pass. All findings are verified against live codebase. Draft items that could not be verified have been removed.*
