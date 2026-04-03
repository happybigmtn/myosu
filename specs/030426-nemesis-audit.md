# Specification: Nemesis Audit Findings and Hardening Requirements

**Audit scope**: Live stage-0 surfaces across all active crates. Inherited pallet, robopoker fork, and chain-fork code are in scope as they are now committed to the repo. The audit covers business-logic flaws, state-desync risks, broken invariants, ordering problems, missing guards, and dangerous assumptions.

**Method**: Nemesis-style Feynman + State inconsistency passes. Each finding is evidence-backed and classified by discovery path.

**Date**: 2026-04-03

---

## Summary of Severity Distribution

| ID | Title | Severity | Discovery Path |
|----|-------|----------|----------------|
| NEM-F01 | MAX_DECODE_BYTES = 256 MB allows OOM DoS | S1 | State |
| NEM-F02 | weighted_median silent zero on iteration limit | S1 | Feynman |
| NEM-F03 | Zero-dividend edge case silently distributes nothing | S2 | Feynman |
| NEM-F04 | No INV-004 cargo tree check in CI | S2 | State |
| NEM-F05 | L1 distance score uses wrong denominator (2.0 for 3+ action games) | S2 | Feynman |
| NEM-F06 | Legacy epoch entrypoints missing consistency guard | S2 | State |
| NEM-F07 | Missing pallet-level emission accounting unit test | S2 | State |
| NEM-F08 | Swap stub max_price = u64::MAX (documented stage-0 risk) | S3 | State |
| NEM-F09 | exploitability() panic message lossy | S3 | Feynman |

**Active S1 items**: 2 (F01, F02)  
**Active S2 items**: 5 (F03, F04, F05, F06, F07)  
**Active S3 items**: 2 (F08, F09)

---

## Findings

### NEM-F01: MAX_DECODE_BYTES = 256 MB allows OOM DoS against validators

- **Discovery path**: `State` (size analysis of decode boundary)
- **Severity**: S1
- **Affected surfaces**:
  - `crates/myosu-games-poker/src/wire.rs:14` (`MAX_DECODE_BYTES = 256 * 1024 * 1024`)
  - `crates/myosu-games-poker/src/solver.rs:44` (imported constant)
  - `crates/myosu-validator/src/validation.rs:167-190` (decode before is_valid check)
- **Triggering scenario**: A malicious miner serves a crafted wire response of exactly 256 MB. The validator calls `decode_strategy_response()` which calls `bincode::deserialize` with a 256 MB limit. The deserializer allocates data structures proportional to input. A 256 MB response could allocate GBs of memory, OOM-killing the validator process before `is_valid()` runs.
- **Invariant/assumption that breaks**: Validators must be able to score miners without crashing. The wire format assumes bounded, honest inputs. The decode boundary is not small enough to prevent resource exhaustion.
- **Why this matters now**: Validators are economically adversarial actors. A miner who can crash validators during the scoring window earns a free score boost (no penalty for non-participation). The `is_valid()` check happens AFTER decode, so an OOM during decode prevents any scoring.

---

### NEM-F02: weighted_median silent zero-return on iteration limit

- **Discovery path**: `Feynman` (loop analysis of weighted_median convergence)
- **Severity**: S1
- **Affected surfaces**:
  - `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs:975-1050` (`weighted_median`)
  - `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs:1053-1118` (`weighted_median_col`)
  - `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs:1121-1170` (`weighted_median_col_sparse`)
- **Triggering scenario**: The `weighted_median` binary-search loop uses `iteration_limit = partition_idx.len()`. If the search does not converge within this many iterations, it breaks and returns `zero` (line 1049). In practice, the function should converge in O(log n) iterations, but pathological stake distributions or score distributions could cause unbalanced splits.
- **Invariant/assumption that breaks**: Validator emissions distributed via `weighted_median` would silently be zero for all validators in the affected column. No error propagates. This is the core of Yuma Consensus — stake-weighted median score determines validator reward share. A silent zero-return would drain all validator emission to zero for one epoch with no operator-visible signal.
- **Why this matters now**: This is the consensus-critical math for emission distribution. A silent failure here means validators get zero rewards for an epoch, breaking the economic model. The current code returns `zero` at line 1049 with no log message.

---

### NEM-F03: Zero-dividend edge case silently distributes nothing

- **Discovery path**: `Feynman` (branch analysis of dividend distribution logic)
- **Severity**: S2
- **Affected surfaces**:
  - `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:431-453` (stage-0 `calculate_dividend_distribution`)
- **Triggering scenario**: If `dividends` BTreeMap is empty (which occurs when the epoch returns zero emission for all hotkeys), `total_dividends = 0`, and the loop body at line 447 is skipped. The function returns an empty BTreeMap, meaning zero emissions are distributed to validators despite `pending_alpha > 0`. This is a silent, log-free failure.
- **Invariant/assumption that breaks**: The emission accounting invariant (sum of distributions should match epoch emissions) is violated. No event is emitted; no error is returned.
- **Why this matters now**: In the current local loop with zero-iteration MCCFR (uninitialized strategy profiles), the epoch may produce zero or near-zero dividends. This edge case is untested in `stage_0_flow.rs`.

---

### NEM-F04: No INV-004 cargo tree dependency check in CI

- **Discovery path**: `State` (grep of CI workflow for dependency boundary check)
- **Severity**: S2
- **Affected surfaces**:
  - `.github/workflows/ci.yml` (entire workflow)
  - `INVARIANTS.md` (INV-004 enforcement clause)
- **Triggering scenario**: Any future commit that introduces a dependency from `myosu-miner` → `myosu-play` or vice versa would not be caught by CI. The dependency boundary is enforced only by convention, not automated check.
- **Invariant/assumption that breaks**: INV-004 says "no direct imports between miner and play" with `cargo tree` as the measurement. This measurement is not automated.
- **Why this matters now**: In the current bootstrap phase, the boundary is maintained manually. Without automated enforcement, the boundary will erode as the codebase grows. A direct dependency would violate the solver-gameplay separation invariant.

---

### NEM-F05: L1 distance score uses wrong denominator (2.0 for 3+ action games)

- **Discovery path**: `Feynman` (gap analysis between scoring formula and game action spaces)
- **Severity**: S2
- **Affected surfaces**:
  - `crates/myosu-validator/src/validation.rs:264` (`score = (1.0 - (l1_distance / 2.0)).clamp(0.0, 1.0)`)
  - `crates/myosu-validator/src/validation.rs:331` (same formula for Liar's Dice)
- **Triggering scenario**: The score formula assumes L1 distance is bounded by 2.0 (maximum distance between two probability distributions over 2 actions). However, NLHE has 3+ actions (fold, call, raise). For a 3-action game, the maximum L1 distance is 2.0 (when distributions are completely disjoint), but for partial overlaps the score computation produces values outside the intended [0, 1] range or compresses the score range.
  
  Example: Expected {A: 0.5, B: 0.5, C: 0.0}, Observed {A: 0.0, B: 0.0, C: 1.0}
  - L1 distance = |0.5-0| + |0.5-0| + |0-1| = 2.0
  - Score = 1.0 - (2.0/2.0) = 0.0 (correct)
  
  But for near-matches with 3+ actions, the normalization is off. The denominator should be `min(2.0, num_actions * max_probability_difference)` or properly bounded.
- **Invariant/assumption that breaks**: INV-003 requires identical scores within epsilon. The scoring formula assumes a 2-action game. NLHE can have 3+ actions (fold, call, raise at minimum), making score ranges non-comparable across different game states.
- **Why this matters now**: The L1 distance directly drives validator scoring and weight distribution. An incorrect normalization makes scores non-comparable across different game states and could skew Yuma Consensus.

---

### NEM-F06: Legacy epoch entrypoints missing consistency guard

- **Discovery path**: `State` (cross-reference of public API vs. call sites)
- **Severity**: S2
- **Affected surfaces**:
  - `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:52-76` (`epoch` — legacy entrypoint)
  - `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:78-81` (`epoch_dense` — legacy entrypoint)
  - `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:302-303` (correct call site with check)
- **Triggering scenario**: If `epoch()` or `epoch_dense()` is called directly (not through `run_coinbase`), it processes the network state without checking for duplicate hotkeys. Duplicate hotkeys cause the epoch's stake-weight normalization to be incorrect (one hotkey counted twice in the stake vector). This could lead to incorrect emission distribution.
- **Invariant/assumption that breaks**: `is_epoch_input_state_consistent` must be checked before epoch runs. The check exists in the coinbase path but not in the legacy `epoch`/`epoch_dense` wrappers.
- **Why this matters now**: The stage-0 flow test `stage_0_flow_registers_stakes_serves_and_emits` calls `epoch` directly. The test does not insert duplicate hotkeys, so it doesn't exercise this gap. A future caller using the legacy `epoch` API with corrupt state would not be protected.

---

### NEM-F07: Missing pallet-level emission accounting invariant test

- **Discovery path**: `State` (test coverage gap analysis)
- **Severity**: S2
- **Affected surfaces**:
  - `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`
  - `crates/myosu-chain/pallets/game-solver/src/tests/determinism.rs`
  - `tests/e2e/emission_flow.sh` (E2E has the invariant)
- **Triggering scenario**: The E2E test `emission_flow.sh` asserts the emission accounting invariant (`sum(distributions) == block_emission * epochs`). However, the pallet unit tests cover individual components (server emission, validator emission, owner cut) but do not have a single test asserting `server + validator + root + owner == block_emission * accrual_blocks` at the pallet level without the E2E infrastructure.
- **Invariant/assumption that breaks**: The no-ship gate requires "Emission accounting: sum(distributions) == block_emission * epochs". The E2E proof exists, but the unit test surface does not assert this invariant in isolation.
- **Why this matters now**: Without a pallet-level invariant test, future changes to the coinbase logic could silently break the emission accounting without any test failing (only the E2E would catch it, and that requires a running devnet).

---

### NEM-F08: Swap stub max_price = u64::MAX (documented stage-0 risk)

- **Discovery path**: `State` (grep + cross-reference of stub → call sites)
- **Severity**: S3
- **Affected surfaces**:
  - `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs:75` (`max_price` returns `u64::MAX`)
  - `crates/myosu-chain/pallets/game-solver/src/lib.rs:166-168` (`stage0_max_price` forwards to `max_price`)
  - `crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs:146` (burn cap check)
- **Triggering scenario**: A miner registration calls `swap_tao_for_alpha` with the burn amount. The swap stub's `swap` returns identity (burn == output). The burn amount is checked against `stage0_max_price() == u64::MAX`, which always passes. In the real AMM this would be a price ceiling protecting against excessive slippage; in stage-0 the identity stub means the cap is effectively infinite, so there's no burn cap enforcement.
- **Invariant/assumption that breaks**: The spec describes a burn-registration model with a price cap. The stub makes the cap meaningless.
- **Why this matters now**: Low risk in stage-0 (registration is free via `burned_register`), but the stub would need to be replaced before any real-token deployment. This is documented as expected but not enforced.

---

### NEM-F09: exploitability() panic message extraction is lossy

- **Discovery path**: `Feynman` (panic payload analysis)
- **Severity**: S3
- **Affected surfaces**:
  - `crates/myosu-games-poker/src/solver.rs:294-299` (`panic_message` function)
  - `crates/myosu-games-poker/src/solver.rs:160-166` (`step` wrapper)
  - `crates/myosu-games-poker/src/solver.rs:208-215` (`exploitability` wrapper)
- **Triggering scenario**: If the upstream robopoker solver panics with a non-string, non-String payload (e.g., a custom panic type), `panic_message()` returns `"non-string panic payload"`. The validator or chain operator sees this generic message with no actionable detail.
- **Invariant/assumption that breaks**: No invariant is directly broken, but this degrades debuggability of upstream robopoker panics (which are known to occur with sparse encoders).
- **Why this matters now**: The `exploitability()` and `step()` panic wrappers are the safety net for robopoker panics. Losing the panic message makes debugging upstream solver failures much harder. The function only handles `&'static str` and `String`, not other common panic types.

---

## Hardening Requirements

### NEM-H01: Add INV-004 cargo tree dependency check to CI

- **Requirement**: Add a CI job or preflight step that runs `cargo tree -p myosu-play -p myosu-miner --invert` and asserts no dependency path exists from `myosu-play` → `myosu-miner` or `myosu-miner` → `myosu-play`.
- **Evidence**: `INVARIANTS.md` defines the measurement as `cargo tree` dependency check. The CI workflow (`.github/workflows/ci.yml`) has no such check.
- **Discovery path**: `State`

### NEM-H02: Reduce MAX_DECODE_BYTES from 256 MB to 1 MB

- **Requirement**: `MAX_DECODE_BYTES` should be reduced to a limit that prevents OOM but accommodates valid responses. A typical NLHE strategy response (3-7 actions, ~50 bytes per action, ~500 bytes total) should be well within 64 KB. A conservative stage-0 limit of 1 MB would accommodate valid responses while making 256 MB OOM attacks impractical.
- **Evidence**: `crates/myosu-games-poker/src/wire.rs:14` = 256 MB. `crates/myosu-validator/src/validation.rs:167` decodes before `is_valid()` check.
- **Discovery path**: `State`

### NEM-H03: Add is_epoch_input_state_consistent guard to legacy epoch entrypoints

- **Requirement**: Both legacy `epoch()` and `epoch_dense()` public functions should call `is_epoch_input_state_consistent(netuid)` before processing and return an error or skip if the state is inconsistent.
- **Evidence**: `run_coinbase.rs:302-303` has the check; `run_epoch.rs:52-76` and `run_epoch.rs:78-81` do not.
- **Discovery path**: `State`

### NEM-H04: Add pallet-level emission accounting invariant test

- **Requirement**: A unit test in `tests/stage_0_flow.rs` or `tests/determinism.rs` that sets up a complete epoch + coinbase cycle and asserts `server_emission_sum + validator_emission_sum + owner_cut == block_emission * (tempo + 1)` within rounding tolerance.
- **Evidence**: E2E has this (`emission_flow.sh`), pallet unit tests do not.
- **Discovery path**: `State`

### NEM-H05: Log warning in weighted_median when iteration limit is hit

- **Requirement**: Before returning `zero` on iteration limit, log a `warn!` with the partition state (partition size, iteration count, stake vector length) so operators can detect this condition.
- **Evidence**: `epoch/math.rs:1045-1049` silently returns `zero` when `iteration_counter > iteration_limit`.
- **Discovery path**: `Feynman`

### NEM-H06: Fix L1 distance score normalization for multi-action games

- **Requirement**: The L1 distance score formula should normalize by the actual maximum possible L1 distance for the game's action space, or use a game-agnostic normalization like `score = 1.0 / (1.0 + l1_distance)` which bounds output to [0, 1] regardless of action count.
- **Evidence**: `validation.rs:264,331` computes `score = 1.0 - (l1_distance / 2.0)`. The `2.0` denominator assumes at most 2 actions. NLHE responses can have 3+ actions.
- **Discovery path**: `Feynman`

### NEM-H07: Add unit test for zero-dividend edge case

- **Requirement**: A test in `tests/stage_0_flow.rs` that passes empty dividends to `calculate_dividend_distribution` and verifies the behavior is documented (either returns empty or distributes proportionally to stake).
- **Evidence**: `run_coinbase.rs:447` skips the loop when `total_dividends == 0`.
- **Discovery path**: `Feynman`

### NEM-H08: Document SwapInterface stub behavior

- **Requirement**: Add explicit documentation to `swap_stub.rs` noting that `max_price = u64::MAX` is intentional for stage-0 but must be revisited before mainnet.
- **Evidence**: `swap_stub.rs:75` returns `u64::MAX`.
- **Discovery path**: `State`

---

## Dropped Findings (Not Supported by Evidence)

The following draft findings were reviewed and dropped:

1. **"L1-distance asymmetry"** - After deeper Feynman analysis, the `l1_distance` function in `validation.rs:261-293` IS symmetric. The first loop handles actions in expected (including those not in observed), and the second loop handles actions ONLY in observed. This correctly implements total variation distance.

2. **"snapshot_profile races with concurrent training"** - The `snapshot_profile()` function requires `&mut self`, which the Rust type system uses to prevent concurrent access. The ArcSwap pattern documented in AGENTS.md for MN-02 is the intended concurrency mechanism. No data race is possible within the type system.

3. **"wrapping_add in blocks_until_next_epoch"** - The `wrapping_add` at line 1007 of `run_coinbase.rs` is intentional for long-running chains. At 1 block/second, u64::MAX would take ~584 billion years. This is not a practical concern for stage-0.
