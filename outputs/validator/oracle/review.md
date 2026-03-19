# `validator:oracle` Lane Review

**Lane**: `validator:oracle`
**Date**: 2026-03-19
**Bootstrap status**: Complete — lane contract written

---

## 1. Keep / Reopen / Reset Judgment

**Judgment: KEEP (with upstream blockers documented)**

The lane contract is sound. The source spec (specsarchive/031626-04b-validator-oracle.md) AC-VO-01..07 is well-structured and the derived lane contract correctly maps each AC to an implementation slice. No reopen or reset is warranted.

**Rationale for KEEP**:
- AC-VO-01..07 form a coherent operational loop (register → discover → query → score → submit)
- INV-003 determinism requirement is correctly identified as the no-ship condition (Slice 8)
- The commit-reveal clarification (anti-gaming hides weights, not test positions) is preserved
- Slice ordering correctly gates on `games:poker-engine` (scoring depends on exploitability functions) and `chain:pallet` restart (storage queries and extrinsics)

**Rationale against RESET**: The source spec is stable, mature, and has no broken assumptions. A reset would discard 3 days of design work with no benefit.

**Conditions that WOULD trigger reopen**:
- `chain:pallet` restart changes the `set_weights` / `commit_weights` / `reveal_weights` extrinsic signature
- `games:poker-engine` changes the `remote_poker_exploitability` function signature
- The poker engine switches away from NLHE to a different primary game without preserving the exploitability interface

---

## 2. Current Lane State

### 2.1 What exists

| Artifact | Status |
|----------|--------|
| `specsarchive/031626-04b-validator-oracle.md` | Source spec — AC-VO-01..07, Draft, 2026-03-16 |
| `specsarchive/031626-18-operational-rpcs.md` | Related RPC spec — AC-OR-01..04 for operational screens |
| `outputs/validator/oracle/spec.md` | This lane's spec — just written |
| `crates/myosu-validator/` | **Does not exist** — zero code |
| `myosu-validator` binary | **Not built** — greenfield |

### 2.2 Upstream blockers

| Blocker | Severity | Est. state |
|---------|----------|------------|
| `games:poker-engine` crate not created | **Critical** | `myosu-games-poker/` missing; `poker_exploitability()` and `remote_poker_exploitability()` unavailable |
| `chain:pallet` restart required | **Critical** | `pallet-game-solver` has 50+ errors from subtensor fork dependencies; `Axons`, `set_weights`, `add_stake` not available |
| `miner:service` not created | **Blocking for integration** | Axon HTTP protocol (MN-03) not confirmed stable; `POST /strategy` / `GET /health` interface not verified |

**Bottom line**: `validator:oracle` cannot begin honest implementation until `games:poker-engine` is complete and `chain:pallet` restart lands. These are hard phase gates, not optional Nice-to-haves.

---

## 3. First Proof and Health Checks to Add Later

### 3.1 Proof commands (for bringup workflow)

After `myosu-validator` crate exists and compiles, verify in order:

```bash
# 1. Unit tests
cargo test -p myosu-validator

# 2. Skeleton compiles
cargo build -p myosu-validator

# 3. Chain client (requires running devnet)
cargo test -p myosu-validator chain::tests::connect_and_query -- --ignored

# 4. Position determinism
cargo test -p myosu-validator positions::tests::deterministic_same_seed
cargo test -p myosu-validator positions::tests::covers_all_streets

# 5. INV-003 determinism (no-ship condition)
cargo test -p myosu-validator determinism::tests::two_validators_agree -- --ignored --test-threads=1

# 6. Dry-run mode
cargo run -- -c ws://localhost:9944 --subnet 1 --key //Alice --dry-run
```

### 3.2 Health checks to add post-slice-7

These are not in the bootstrap contract but should be wired in the bringup workflow:

| Health check | What it verifies |
|--------------|------------------|
| Block subscription liveness | Missed >3 consecutive blocks → ERROR log + reconnect |
| Miner query coverage | `evaluated_count / discovered_count` < 0.8 per tempo → WARN |
| Weight submission confirmation | No on-chain confirmation within 2 blocks → WARN |
| Encoder hash consistency | Two validators' startup encoder hashes differ → ERROR |
| Score monotonicity | Validator's own score decreases significantly without explanation → WARN |

### 3.3 Observability surfaces for operator dashboards

The operational RPCs (AC-OR-03: `gameSolver_validatorDivergence`) are not owned by `validator:oracle`, but the validator binary should emit structured log lines that a downstream TUI or monitoring tool can parse:

```
# Structured log format (JSON):
{"level":"info","ts":1710800000,"service":"myosu-validator","subnet":1,"uid":3,"event":"epoch_complete","evaluated":8,"skipped":1,"weights_hash":"0xabc123"}
{"level":"warn","ts":1710800000,"service":"myosu-validator","subnet":1,"uid":3,"event":"miner_timeout","miner_uid":5}
```

---

## 4. Risks That Must Be Preserved or Reduced

| Risk | Mitigation | Who |
|------|-----------|-----|
| Non-deterministic scoring across validators (INV-003 violation) | Encoder pinning + two-validator integration test (Slice 8) | `validator:oracle` |
| Exploitability computation too slow for tempo window | Benchmark before shipping; partial weights fallback in Slice 6 | `validator:oracle` |
| Miner axon protocol diverges from MN-03 spec | Pin to MN-03 spec version; integration test with miner binary | `miner:service` + `validator:oracle` |
| `chain:pallet` restart changes extrinsic signatures | Do not hardcode pallet call indices; use `subxt` metadata | `validator:oracle` |
| Concurrent miner queries overwhelm validator host | Bounded concurrency (Slice 5: `FuturesUnordered`) | `validator:oracle` |

---

## 5. Is the Lane Unblocked for Implementation-Family Workflow Next?

**No. Not yet.**

The lane is **bootstrap complete** but **implementation blocked** by upstream dependencies:

```
games:poker-engine  ──X──►  validator:oracle (cannot start slices 3-8)
chain:pallet        ──X──►  validator:oracle (cannot start slices 2-8)
miner:service       ──X──►  validator:oracle (cannot verify Slice 5 integration)
```

**Required before validator:oracle implementation-family workflow**:
1. `games:poker-engine` must complete through Slice 5 (exploitability)
2. `chain:pallet` restart must complete through at least Phase 2 (storage + extrinsics available)
3. `miner:service` must complete through Slice 3 (MN-03 stable)

The lane contract is durable and ready to consume. When the upstream blockers clear, the implementation lane can begin at **Slice 1** without any re-derivation.

---

## 6. Notable Design Decisions From Source Spec (preserved)

| Decision | Rationale | Preserved in |
|---------|-----------|--------------|
| `weight = (1 - exploit/baseline) * 65535` | Simple, monotonic, maps to u16 | `scoring.rs` |
| Seed from `hash(subnet, epoch, hotkey)` | Deterministic per-validator, varies across validators for coverage | `positions.rs` |
| 6-second poll interval | Matches block time, minimal overhead | `main.rs` |
| Direct + commit-reveal modes | Direct for devnet simplicity; commit-reveal for production | `submitter.rs` |
| Commit-reveal hides weights, not positions | Bittensor anti-gaming standard; positions visible to miners in real-time | `submitter.rs` |
| 5s timeout per miner, score=0 on skip | Prevents one unresponsive miner from blocking evaluation | `evaluator.rs` |
| `tempo * 0.8` deadline for evaluation | Allows 20% window for submission on-chain | `submitter.rs` |
| Encoder hash logged on startup | INV-003 reproducibility checkpoint for operators | `main.rs` |
