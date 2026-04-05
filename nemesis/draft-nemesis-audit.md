# Specification: Nemesis Audit Findings and Hardening Requirements

**Audit Date:** 2026-04-05  
**Audit Scope:** myosu stage-0 codebase (pallet-game-solver, miner, validator, gameplay, chain-client)  
**Method:** Nemesis-style Feynman + State inconsistency audit, Pass 1

---

## Finding Categories

- **S0** — Critical: Direct consensus/financial exploit path
- **S1** — High: Invariant violation or state corruption risk
- **S2** — Medium: Correctness gap or missing guard
- **S3** — Low: Hardening opportunity or documentation gap

---

## INVARIANT VIOLATIONS AND STATE INCONSISTENCY RISKS

### NEM-001: GRANDPA Finality Stalls on 3-Node Devnet After Authority Stop

**Severity:** S1  
**Affected Surfaces:** `crates/myosu-chain/node/`, `tests/e2e/three_node_finality.sh`, consensus runtime configuration  
**Triggering Scenario:** Start 3-authority devnet. Stop authority-3. Remaining two authorities continue importing blocks but finalized height freezes at `#1` or `#2`. Node logs show `Backing off claiming new slot for block authorship: finality is lagging`.  
**Invariant or Assumption That Breaks:** `bootstrap exit criteria` requires "3-node GRANDPA finality" for Phase 1 completion. The system cannot reach multi-node production with this behavior.  
**Why This Matters Now:** BLOCKS P-011, which is the gating milestone for Phase 1. Operators cannot run production multi-node networks.  
**Discovery Path:** Feynman (examining `three_node_finality.sh` repro script and node service configuration)

**Codebase Evidence:**
- `tests/e2e/three_node_finality.sh` (exists, reproduces stalling)
- `crates/myosu-chain/node/src/devnet.rs` defines 3-authority chain spec
- `crates/myosu-chain/node/src/service.rs` controls GRANDPA configuration
- `WORKLIST.md:NET-FINALITY-001` explicitly documents this as open issue

---

### NEM-002: Cross-Node Emission Disagreement Is Unverified

**Severity:** S1  
**Affected Surfaces:** `crates/myosu-chain/pallets/game-solver/src/coinbase/`, `crates/myosu-chain/pallets/game-solver/src/epoch/`, `tests/e2e/`  
**Triggering Scenario:** Two separate validator nodes produce different `TotalIssuance` values after N epoch transitions on the same 3-node devnet.  
**Invariant or Assumption That Breaks:** INV-005 (Plan And Land Coherence) — the emission accounting spec states `sum(distributions) == block_emission * epochs` but this is only proven single-node. Cross-node fixed-point determinism of `substrate_fixed` types (U96F32, I64F64) is assumed, not verified.  
**Why This Matters Now:** If emission diverges across nodes, the chain forks or validators disagree on weights, breaking Yuma Consensus and INV-003.  
**Discovery Path:** State (examining coinbase/truncation sweep tests and P-012 planning)

**Codebase Evidence:**
- `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs` uses U96F32 throughout
- `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs` only checks TotalIssuance locally
- `IMPLEMENTATION_PLAN.md:P-012` explicitly states "tested single-node only"
- Stage-0 runtime defaults exposed via `myosu-chain-client` have `SubtensorInitialNetworkRateLimit = 0`

---

### NEM-003: TotalIssuance Accounting Tolerance Masks Systematic Drift

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs`, `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`  
**Triggering Scenario:** During `on_idle` block execution, TotalIssuance diverges from `currency_issuance + TotalStake` by more than 0 but less than 1000 RAO. The delta check passes silently. Over many blocks, systematic under/over issuance accumulates.  
**Invariant or Assumption That Breaks:** The accounting invariant that `TotalIssuance == currency_issuance + TotalStake` (modulo rounding). The 1000 RAO tolerance is an intentional fudge factor.  
**Why This Matters Now:** `WORKLIST.md:EM-DUST-001` documents 2 rao/block truncation loss. At 100 blocks/epoch × 3600 epochs/day, this could be significant over time. The tolerance hides whether the drift is bounded.  
**Discovery Path:** Feynman (examining `check_total_issuance` implementation and coinbase dust sweep)

**Codebase Evidence:**
```rust
// src/utils/try_state.rs:21
let delta = 1000;  // 1000 RAO tolerance
let total_issuance = TotalIssuance::<T>::get().to_u64();
let diff = if total_issuance > expected_total_issuance {
    total_issuance.checked_sub(expected_total_issuance)
} else {
    expected_total_issuance.checked_sub(total_issuance)
}.expect("LHS > RHS");
ensure!(diff <= delta, "TotalIssuance diff greater than allowable delta");
```
- `cargo test -p pallet-game-solver -- truncation` measures 2 rao/block worst case
- Stage-0 coinbase uses `tou64!` macro to truncate U96F32 → u64, losing fractional RAO

---

### NEM-004: Epoch Consistency Guard Silently Skips Processing on Detected Inconsistency

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`, `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`  
**Triggering Scenario:** A subnet has duplicate hotkeys or key-index mismatch (e.g., after a failed registration or storage corruption). When `should_run_epoch()` returns true, `is_epoch_input_state_consistent()` returns false. The epoch is skipped, emissions are not distributed, but no error is emitted and no operator alert fires.  
**Invariant or Assumption That Breaks:** Per-subnet epoch processing should occur on every tempo boundary. Silent skipping breaks emission distribution.  
**Why This Matters Now:** The consistency guard was added (C-011) to prevent epoch corruption, but skipping silently means validators and miners stop receiving emission without knowing why.  
**Discovery Path:** Feynman (examining `run_epoch.rs:66-68` and `run_coinbase.rs:306-310`)

**Codebase Evidence:**
```rust
// src/epoch/run_epoch.rs:66-68
if !Self::is_epoch_input_state_consistent(netuid) {
    log::error!("Skipping legacy epoch for inconsistent netuid {netuid}");
    return Vec::new();  // SILENT SKIP - no event emitted
}

// src/coinbase/run_coinbase.rs:306-310
if Self::should_run_epoch(netuid, current_block)
    && Self::is_epoch_input_state_consistent(netuid)
{
    // ... drain pending emissions
} else {
    // Pending emissions stay in storage, not distributed
}
```

---

## CORRECTNESS GAPS

### NEM-005: Validator L1 Distance Has Implicit Zero-Weight Action Asymmetry

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-validator/src/validation.rs`, `specs/050426-validation-surface.md`  
**Triggering Scenario:** A miner returns an action set where `expected = {A: 1.0}` and `observed = {B: 1.0}` (completely different actions). The algorithm penalizes missing A (contribution 1.0) and adds B (contribution 1.0), giving l1 = 2.0. But if expected = {A: 0.5, B: 0.5} and observed = {A: 1.0} (missing B), l1 = |0.5 - 1.0| + |0.5 - 0| = 1.0. The action "B" present in expected but missing from observed is counted; the action "A" present in observed but missing from expected is counted.  
**Invariant or Assumption That Breaks:** The spec claims "L1 distance covers both expected and observed action sets" which is technically true but asymmetrically weighted in edge cases.  
**Why This Matters Now:** A miner could potentially game the scoring by including zero-probability actions in their response that the validator cannot penalize.  
**Discovery Path:** Feynman (examining `validation.rs:345-367` L1 distance implementation)

**Codebase Evidence:**
```rust
// src/validation.rs:345-367
// First pass: actions in expected (penalizes missing from observed)
for (action, prob) in &expected.actions {
    let observed_prob = observed.probability_for(action);
    l1 += (prob - observed_prob).abs();
}
// Second pass: actions only in observed (penalizes unexpected)
for (action, prob) in &observed.actions {
    if expected.probability_for(action) == 0.0 {  // IMPLICIT CHECK
        l1 += prob;  // Only counted if expected == 0.0
    }
}
```

---

### NEM-006: SwapInterface NoOp Stub Returns Unbounded Price for Max Price

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-chain/runtime/src/lib.rs`, `crates/myosu-chain/pallets/swap-interface/src/lib.rs`  
**Triggering Scenario:** Stage-0 runtime uses `Stage0NoopSwap` which returns `Balance::max_value()` for `stage0_max_price()`. Any swap logic that checks against max price will never trigger price-limit guards.  
**Invariant or Assumption That Breaks:** The swap interface contract expects finite price limits for safety guards.  
**Why This Matters Now:** Stage-0 intentionally has no real swap, but if any future code path assumes max_price() returns a realistic bound, it will fail open.  
**Discovery Path:** Cross-feed (AGENTS.md notes SwapInterface stub and code examination)

**Codebase Evidence:**
- `runtime/src/lib.rs:89-150` defines `Stage0NoopSwap` with 1:1 identity conversion
- AGENTS.md: "SwapInterface no-op stub (1:1 identity)" — noted as intentional
- `pallet-game-solver/src/lib.rs` calls `T::SwapInterface::stage0_max_price()` in emission calculation

---

## MISSING GUARDS AND BOUNDARY ISSUES

### NEM-007: Key Password Exposed in Process Arguments

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-keys/src/lib.rs`, `crates/myosu-miner/src/cli.rs`, `crates/myosu-validator/src/cli.rs`  
**Triggering Scenario:** Operator runs `myosu-miner --key "file:///path/to/encrypted.json"` with a password argument. The password appears in `ps aux` output and `/proc/$pid/cmdline`.  
**Invariant or Assumption That Breaks:** Key material should not appear in process visibility surfaces.  
**Why This Matters Now:** On shared hosting or multi-tenant systems, other users can read process arguments.  
**Discovery Path:** Feynman (examining CLI argument handling)

**Codebase Evidence:**
- CLI uses `--key <uri>` and `--key-config-dir` patterns
- `myosu-keys/src/lib.rs` documents `load_active_secret_uri_from_env` pattern
- Default `MYOSU_KEY_PASSWORD` env var is the right approach, but direct CLI secrets bypass it

---

### NEM-008: GRANDPA Finality Lag Not Recovered After Network Partition

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-chain/node/src/service.rs`, GRANDPA configuration  
**Triggering Scenario:** Two of three authorities are offline. The remaining authority continues authoring blocks but cannot finalize. When the two come back online, finality does not catch up.  
**Invariant or Assumption That Breaks:** Specs assume "2/3 quorum for finality" but recovery behavior is unproven.  
**Why This Matters Now:** Operators need to understand restart behavior for production.  
**Discovery Path:** State (NET-FINALITY-001 in WORKLIST.md)

---

### NEM-009: Decoder Budget Hardening Applied to Poker Only

**Severity:** S3  
**Affected Surfaces:** `crates/myosu-games-poker/src/wire.rs`, `crates/myosu-games-liars-dice/src/`  
**Triggering Scenario:** A malicious miner sends a 100MB response for Liar's Dice. The decoder does not have the same 1 MiB budget as poker's `MAX_DECODE_BYTES = 1_048_576`.  
**Invariant or Assumption That Breaks:** Consistent decode budget across all game wire formats.  
**Why This Matters Now:** `C-013` hardened the poker wire codec but Liar's Dice wire format may not have equivalent bounds.  
**Discovery Path:** Cross-feed (AGENTS.md notes decode budget, code examination)

**Codebase Evidence:**
- `crates/myosu-games-poker/src/wire.rs:8` defines `MAX_DECODE_BYTES: u64 = 1_048_576`
- `crates/myosu-games-liars-dice/` has `decode_strategy_query` / `decode_strategy_response` but bounds not verified

---

## DOCUMENTATION AND SPECIFICATION ISSUES

### NEM-010: Spec Inconsistency — INV-006 Robopoker Fork Tracking

**Severity:** S3  
**Affected Surfaces:** `INVARIANTS.md`, `docs/robopoker-fork-changelog.md`, `Cargo.toml`  
**Triggering Scenario:** A contributor assumes INV-006 requires tracking `v1.0.0` tag, but the fork uses a branch pin.  
**Invariant or Assumption That Breaks:** INV-006 says "must track v1.0.0 as baseline" but AGENTS.md notes "fork uses branch, not upstream tag".  
**Why This Matters Now:** Creates confusion for contributors auditing fork coherence.  
**Discovery Path:** Cross-feed (INVARIANTS.md vs AGENTS.md)

**Codebase Evidence:**
- `INVARIANTS.md:INV-006`: "must track v1.0.0 as baseline"
- `AGENTS.md`: "INV-006 says 'git tag v1.0.0' — fork uses branch, not upstream tag"
- `docs/robopoker-fork-changelog.md` documents fork changes but baseline unclear

---

### NEM-011: Missing Checkpoint Version Magic Header

**Severity:** S3  
**Affected Surfaces:** `crates/myosu-games-poker/src/solver.rs`, `crates/myosu-miner/src/training.rs`  
**Triggering Scenario:** A future format change to `PokerSolver::save()` / `load()` produces incompatible files. Old clients silently fail to load or corrupt memory.  
**Invariant or Assumption That Breaks:** AGENTS.md specifies "checkpoint versioning: 4-byte magic + version" but no such magic/version header exists in current checkpoint format.  
**Why This Matters Now:** Format evolution without magic bytes risks silent corruption on upgrades.  
**Discovery Path:** Feynman (AGENTS.md key decision spec vs actual checkpoint implementation)

**Codebase Evidence:**
- AGENTS.md: "checkpoint versioning: 4-byte magic + version" is a KEY DECISION
- `crates/myosu-games-poker/src/solver.rs` — `save()` / `load()` implementation does not include magic bytes
- `crates/myosu-miner/src/training.rs` — checkpoint serialization uses bincode without magic header

---

### NEM-012: AGENTS.md References Nonexistent RTK.md

**Severity:** S3  
**Affected Surfaces:** `AGENTS.md`, `WORKLIST.md:DOC-OPS-001`  
**Triggering Scenario:** A future operator loop reads `@RTK.md` reference at top of AGENTS.md and cannot find the file.  
**Invariant or Assumption That Breaks:** Documentation references must resolve.  
**Why This Matters Now:** `WORKLIST.md:DOC-OPS-001` already tracks this.  
**Discovery Path:** Cross-feed (WORKLIST.md explicitly tracks dangling reference)

**Codebase Evidence:**
- `WORKLIST.md:DOC-OPS-001`: "Resolve the dangling @RTK.md reference at the top of AGENTS.md"
- `AGENTS.md` top contains `@RTK.md` reference
- No `RTK.md` exists in repository

---

## ARCHITECTURE AND DESIGN CONCERNS

### NEM-013: No Minimum Training Iteration Gate

**Severity:** S3  
**Affected Surfaces:** `crates/myosu-miner/src/training.rs`, `specs/050426-mining-surface.md`  
**Triggering Scenario:** A miner trains for 1 MCCFR iteration and serves garbage strategy. Validators score it near zero, but the system has no way to distinguish "untrained" from "poorly trained."  
**Invariant or Assumption That Breaks:** Mining quality requires meaningful strategy convergence.  
**Why This Matters Now:** `F-007` in IMPLEMENTATION_PLAN.md is follow-on research, not yet done.  
**Discovery Path:** Feynman (examining miner training loop and `solve()` implementation)

**Codebase Evidence:**
- `crates/myosu-miner/src/training.rs` accepts `--train-iterations` without minimum
- `solve()` in robopoker can return immediately with 0 iterations
- No convergence metric is enforced as prerequisite for serving

---

### NEM-014: INV-004 Solver-Gameplay Separation Not Enforced at Runtime

**Severity:** S2  
**Affected Surfaces:** `crates/myosu-miner/`, `crates/myosu-play/`, CI configuration  
**Triggering Scenario:** A refactor accidentally adds a dependency from myosu-play to myosu-miner (or vice versa). CI `cargo tree` check catches it at PR level, but runtime behavior is unguarded.  
**Invariant or Assumption That Breaks:** INV-004: "A gameplay bug must not corrupt training data."  
**Why This Matters Now:** CI enforcement is compile-time only. At runtime, the separation is a cargo-dependency convention, not a hard boundary.  
**Discovery Path:** State (INV-004 enforcement analysis)

**Codebase Evidence:**
- CI workflow (`ci.yml:107-121`) runs `cargo tree` check for INV-004
- `crates/myosu-play/Cargo.toml` and `crates/myosu-miner/Cargo.toml` have separate dependencies
- AGENTS.md: "INV-004 in CI gate" — correct, but runtime enforcement missing

---

## SUMMARY TABLE

| ID | Severity | Category | Invariant/Property | Discovery |
|----|----------|----------|-------------------|-----------|
| NEM-001 | S1 | Consensus | 3-node GRANDPA finality | Feynman |
| NEM-002 | S1 | Accounting | Cross-node emission agreement | State |
| NEM-003 | S2 | Accounting | TotalIssuance accounting tolerance | Feynman |
| NEM-004 | S2 | Emission | Epoch skip on inconsistency | Feynman |
| NEM-005 | S2 | Scoring | L1 distance asymmetry | Feynman |
| NEM-006 | S2 | Swap | NoOp stub max_price unbounded | Cross-feed |
| NEM-007 | S2 | Security | Key password in process args | Feynman |
| NEM-008 | S2 | Consensus | GRANDPA recovery unproven | State |
| NEM-009 | S3 | Wire | Decoder budget Liar's Dice | Cross-feed |
| NEM-010 | S3 | Spec | INV-006 fork tracking confusion | Cross-feed |
| NEM-011 | S3 | Checkpoint | Missing magic header | Feynman |
| NEM-012 | S3 | Docs | RTK.md dangling reference | Cross-feed |
| NEM-013 | S3 | Mining | No min training gate | Feynman |
| NEM-014 | S2 | Architecture | INV-004 runtime enforcement | State |

---

## OPEN QUESTIONS FOR PASS 2

1. Does the `validate_score_distribution` in validation tests actually catch the L1 asymmetry in NEM-005?
2. Is there a scenario where `is_epoch_input_state_consistent` can return false without a storage corruption that should be alarmed?
3. Does the 3-node finality stall reproduce in CI or only locally? (P-011 blockers)
4. Is the `swap_tao_for_alpha` path in coinbase reachable with NoOpSwap, and if so, does it panic or silently no-op?
5. Are there other game types beyond Poker and Liar's Dice that would bypass decode budget hardening?

---

*This document is a draft artifact for Nemesis Pass 1 review. Findings are evidence-backed. Open questions require Pass 2 investigation.*
