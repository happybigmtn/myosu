# IMPLEMENTATION_PLAN

Generated: 2026-04-08
Audit findings: `nemesis/draft-nemesis-audit.md`
Codebase snapshot: trunk @ 4e0b37f

---

## Priority Work

### `- [ ] NEM-001A Record validator score self-reference limitation in proof surfaces`

Spec: INV-003, INV-005
Why now: The `local_loop.sh` and `validator_determinism.sh` happy-path proofs currently report `score=1.0 exact_match=true` on a zero-iteration checkpoint. This is a self-consistency proof, not a quality proof. Without documentation, an operator could misread these scores as evidence of high solver quality.
Codebase evidence: `tests/e2e/local_loop.sh` feeds miner-produced checkpoint back into validator. `crates/myosu-validator/src/validation.rs:score_response()` computes `solver.answer(query)` from the same checkpoint the miner produced. AGENTS.md "current stage-0 happy path" entry confirms this is intentional.
Owns: Update `tests/e2e/local_loop.sh` header comment and `tests/e2e/validator_determinism.sh` header comment to explicitly state the self-referential limitation.
Integration touchpoints: E2E proof surfaces, AGENTS.md operator loop section.
Scope boundary: Proof surface comments only. No code changes. No validator scoring logic changes.
Required tests: `grep` for the new comments in the shell files.
Dependencies: None.
Completion signal: Both shell scripts have updated header comments stating that the score is a self-consistency proof and that independent quality benchmarking (e.g., `quality_benchmark` for Liar's Dice) is required for genuine quality signal.

---

### `- [ ] NEM-001B Add independent quality reference to validator scoring test suite`

Spec: INV-003
Why now: The stage-0 validator scoring has no independent quality reference for Poker. The Liar's Dice `quality_benchmark` test (already present) serves this role for that game. Poker needs an equivalent so that `F-007` (minimum training iterations research) has a truthful surface to measure against.
Codebase evidence: `crates/myosu-validator/src/validation.rs:quality_benchmark_liars_dice_exploitability_converges` test independently trains solvers and measures `exact_exploitability()`. No equivalent exists for poker. `crates/myosu-games-poker/examples/bootstrap_artifacts.rs` emits sparse encoder that fails positive iteration training.
Owns: Add a `quality_benchmark_poker_exploitability_converges` test to `crates/myosu-validator/src/validation.rs` that uses a non-sparse encoder (or synthesizes a fixed-profile reference) and measures how poker solver quality improves with iterations.
Integration touchpoints: `crates/myosu-validator/src/validation.rs`, `F-007` task in `IMPLEMENTATION_PLAN.md`.
Scope boundary: Test-only. Do not change validator scoring formula. Do not require a full PostgreSQL encoder for this test.
Required tests: The new `quality_benchmark_poker_exploitability_converges` test passes.
Dependencies: None (can use a hand-crafted fixed-profile reference, does not need robopoker DB).
Completion signal: `cargo test -p myosu-validator -- quality_benchmark_poker` passes. Test documents a poker exploitability ladder showing measurable quality improvement across iteration counts.

---

### `- [ ] NEM-002A Document Stage0NoopSwap price-limit risk as a blocking migration guard`

Spec: INV-005
Why now: The `Stage0NoopSwap::max_price()` returning `u64::MAX` is a documented time-bomb. A future AMM substitution that forgets to bound the price limit would silently expose staking operations to unbounded slippage. There is no compile-time or migration-time enforcement.
Codebase evidence: `crates/myosu-chain/runtime/src/lib.rs:94-97` comment acknowledges the risk. `crates/myosu-chain/pallets/game-solver/src/staking/add_stake.rs:196` uses `stage0_max_price()` as price limit. `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:141` uses it in emission swap path.
Owns: Add a `MAX_VALID_SWAP_PRICE_LIMIT: u64` constant to the pallet config trait. Assert in `try_state` that `SwapInterface::max_price()` ≤ `MAX_VALID_SWAP_PRICE_LIMIT`. Document the assertion in `utils/try_state.rs`. This makes the unbounded limit a detectable invariant violation rather than silent runtime behavior.
Integration touchpoints: Pallet config trait, `try_state.rs`, `runtime/src/lib.rs`.
Scope boundary: Add the bound and the try-state check. Do not change `Stage0NoopSwap` behavior. Do not change any staking or swap logic.
Required tests: `cargo test -p pallet-game-solver -- try_state` — add a test that confirms the new assertion is satisfied by the current `Stage0NoopSwap`.
Dependencies: None.
Completion signal: `cargo test -p pallet-game-solver -- try_state` passes with the new assertion. The assertion is checked on every `on_initialize` via `try_state`.

---

### `- [ ] NEM-002B Add comment to `SwapInterface` trait requiring bounded `max_price` before AMM substitution`

Spec: INV-005
Why now: The swap interface trait (`Stage0SwapInterface` and the underlying `SwapHandler`) documents the behavior but not the precondition for safe substitution. A future implementer must know that `max_price()` must return a non-unbounded value before the AMM path is wired.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/lib.rs:88-168` — `Stage0SwapInterface` trait definition with no preconditions on `max_price`. `crates/myosu-chain/runtime/src/lib.rs:94-97` — runtime comment on `max_price()` only.
Owns: Add a doc comment to the `Stage0SwapInterface` trait and the `SwapHandler` trait requiring that implementers bound `max_price()` to a non-identity value. Add a `const MAX_PRICE_LIMIT_BOUND: u64` associated constant that any AMM implementation must set below.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/lib.rs`, `crates/myosu-chain/runtime/src/lib.rs`.
Scope boundary: Documentation and type-level enforcement only. Do not change any runtime behavior.
Required tests: `cargo doc -p pallet-game-solver` passes with the new trait documentation.
Dependencies: None.
Completion signal: The trait documentation explicitly states "any implementer must bound max_price below u64::MAX before wiring the swap into staking or emission paths."

---

### `- [ ] NEM-003A Add epoch per-UID emission accumulation sweep test`

Spec: INV-005 (emission accounting)
Why now: The `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep` test verifies the coinbase block-emission → owner/server/validator split closes exactly. It does not verify that `sum(server_emission_per_uid) + sum(validator_emission_per_uid)` across all UIDs equals the total epoch allocation after individual `I96F32 → u64` truncation per UID.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:490-530` — per-UID `server_emission` and `validator_emission` computed independently with `saturating_to_num::<u64>()`. `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs:599-633` — existing sweep only covers the coinbase split, not per-UID epoch accumulation.
Owns: Add a unit test (or extend the existing sweep) that generates synthetic epoch outputs for representative numbers of miners/validators, sums the per-UID `server_emission + validator_emission`, and asserts the sum equals the total epoch emission minus the owner cut within a small epsilon (0 or 1 rao per UID).
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`, `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`.
Scope boundary: Test-only. Do not change epoch math. Do not change emission accumulation logic.
Required tests: New `stage_0_epoch_per_uid_emission_sum_equals_total` test passes.
Dependencies: None.
Completion signal: `cargo test -p pallet-game-solver -- epoch_per_uid` passes. The test sweep covers n=1..16 miners, n=0..4 validators, and representative epoch emission values.

---

### `- [ ] NEM-003B Document epoch per-UID truncation gap in run_epoch.rs comment`

Spec: INV-005
Why now: The epoch math uses `I96F32 → u64` truncation per UID. The gap from this truncation is bounded by the number of UIDs but is not documented. A future maintainer might assume the coinbase `close_integer_emission_split` coverage extends to the epoch path.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs` — per-UID `server_emission` and `validator_emission` computed with `I96F32::saturating_from_num(*se).saturating_mul(float_rao_emission).saturating_to_num::<u64>()` without explicit discussion of truncation behavior.
Owns: Add a doc comment at the emission computation section of `epoch_dense_mechanism` and `epoch_mechanism` noting: (1) per-UID truncation floors are bounded by n_validators + n_miners rao per epoch, (2) the try_state check (NEM-002A) will detect violations, (3) this path is separate from the coinbase split verified by `close_integer_emission_split`.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`.
Scope boundary: Documentation only.
Required tests: `cargo doc -p pallet-game-solver` passes with the new comments.
Dependencies: None.
Completion signal: The epoch emission section of `run_epoch.rs` has an explicit comment noting the truncation floor boundary and its relationship to the try_state guard.

---

### `- [ ] NEM-004A Make `dense_sparse_epoch_paths_produce_identical_state` run in the default build`

Spec: INV-003, INV-004
Why now: The INV-003 parity proof for `epoch_mechanism` vs `epoch_dense_mechanism` is behind `legacy-subtensor-tests`. The default build does not run this proof. A future regression in the sparse/dense parity would not be caught by CI.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs` — the test exists but is conditionally compiled. AGENTS.md "dense_sparse_epoch_paths_produce_identical_state" entry confirms the proof exists.
Owns: Either (a) move the `dense_sparse_epoch_paths_produce_identical_state` test out of the `#[cfg(feature = "legacy-subtensor-tests")]` block into the default test path, or (b) add the test to the default `stage_0_flow.rs` that exercises the key parity assertion without requiring the full legacy feature.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`, CI workflow.
Scope boundary: Test-only. Do not change epoch math.
Required tests: `cargo test -p pallet-game-solver -- dense_sparse_epoch` passes in the default (no feature flag) build.
Dependencies: None.
Completion signal: `SKIP_WASM_BUILD=1 cargo test -p pallet-game-solver -- dense_sparse_epoch --quiet` passes without feature flags.

---

### `- [ ] NEM-004B Add documentation to run_epoch.rs clarifying the two retained epoch paths`

Spec: INV-003, INV-004
Why now: `epoch_mechanism` (sparse) and `epoch_dense_mechanism` (dense) are both retained. Their relationship is not obvious from the code. A future maintainer might incorrectly assume one is deprecated.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:130-230` — both functions have sparse wrapper comments. `epoch_mechanism` is called from `run_coinbase` via `epoch_with_mechanisms`. `epoch_dense_mechanism` is called directly from `epoch_dense`.
Owns: Add doc comments at the module level of `run_epoch.rs` explaining: (1) which path is the canonical production path for stage-0, (2) which path is the compatibility/parity path, (3) that they must remain in sync and the test NEM-004A enforces this.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`.
Scope boundary: Documentation only.
Required tests: `cargo doc -p pallet-game-solver` passes with the new comments.
Dependencies: NEM-004A.
Completion signal: The `run_epoch.rs` module doc explains the role of each function and the enforced parity requirement.

---

### `- [ ] NEM-005A Strengthen INV-004 cargo tree check to also verify shared-dependency isolation`

Spec: INV-004
Why now: The current `cargo tree` check only prevents direct `myosu-play → myosu-miner` or `myosu-miner → myosu-play` edges. A transitive import through a shared dependency (e.g., `myosu-play → myosu-chain-client → myosu-miner`) would not be caught. The AGENTS.md acknowledges this is the current enforcement mechanism.
Codebase evidence: `crates/myosu-play/tests/invariants.rs:inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other` — only checks for presence of the forbidden package name in `cargo tree` output. Does not check for re-exports of miner-specific types through chain-client.
Owns: Add an additional check: (a) verify that `myosu-chain-client` does not expose any type that originates from `myosu-miner`, (b) add a workspace-level `Cargo.toml` note documenting the boundary, (c) add a test that imports all `myosu-chain-client` public items and confirms none require `myosu-miner`.
Integration touchpoints: `crates/myosu-chain-client/Cargo.toml`, `crates/myosu-play/tests/invariants.rs`, `Cargo.toml` (workspace).
Scope boundary: Verification test addition. No changes to crate boundaries or code.
Required tests: New `inv_004_chain_client_does_not_re_export_miner_types` test passes. Existing `inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other` continues to pass.
Dependencies: None.
Completion signal: `cargo test -p myosu-play -- inv_004` passes with both the existing and new subtests.

---

### `- [ ] NEM-005B Document INV-004 enforcement mechanism and its limitations in INVARIANTS.md`

Spec: INV-004
Why now: INV-004 states "no direct imports" but does not document the enforcement mechanism. The prose-only definition leaves room for misreading: a transitive import through `myosu-chain-client` is technically not a "direct" import but violates the spirit of the invariant.
Codebase evidence: `INVARIANTS.md:INV-004` — the invariant text. `crates/myosu-play/tests/invariants.rs` — the enforcement test. AGENTS.md "INV-004 boundary" entry.
Owns: Add an "Enforcement" subsection to INV-004 in `INVARIANTS.md` explaining: (a) the `cargo tree` textual check, (b) the transitive-dependency gap, (c) the improvement from NEM-005A.
Integration touchpoints: `INVARIANTS.md`.
Scope boundary: Documentation only. No code changes.
Required tests: `grep -r "INV-004" INVARIANTS.md` finds the new enforcement subsection.
Dependencies: NEM-005A.
Completion signal: `INVARIANTS.md` has an explicit enforcement section for INV-004 explaining both the current mechanism and its known limitations.

---

### `- [ ] NEM-006A Add HTTP axon integration test to validator crate`

Spec: INV-005 (end-to-end wiring)
Why now: The miner HTTP axon + validator HTTP scoring path has no automated test. The file-based E2E harness bypasses this path entirely. A codec mismatch, HTTP header error, or path issue would not be caught.
Codebase evidence: `crates/myosu-miner/src/axon.rs` — HTTP axon server. `crates/myosu-validator/src/chain.rs` — chain query for miner discovery. `crates/myosu-chain/pallets/game-solver/src/serving.rs` — `serve_axon` extrinsic. `tests/e2e/local_loop.sh` — file-based, not HTTP.
Owns: Add a test in `crates/myosu-validator/src/` that: (a) starts a mock HTTP server simulating a miner axon, (b) exercises the validator's miner discovery and HTTP query path, (c) asserts the HTTP request/response round-trips correctly. This can use a simple in-process HTTP mock (e.g., `wiremock` or a `tiny_http` mock) or a mock TCP server.
Integration touchpoints: `crates/myosu-validator/src/chain.rs`, `crates/myosu-validator/src/validation.rs`.
Scope boundary: Integration test within the validator crate. Does not require the chain binary. Uses a mock miner HTTP endpoint.
Required tests: `cargo test -p myosu-validator -- http_axon` passes.
Dependencies: None (can use a mock HTTP server or spawn a temporary process).
Completion signal: `cargo test -p myosu-validator -- http_axon` passes. The test exercises the full miner discovery → HTTP query → strategy decode path.

---

### `- [ ] NEM-006B Document the file-based vs HTTP proof distinction in local_loop.sh`

Spec: INV-005
Why now: `tests/e2e/local_loop.sh` is the primary operator proof entry in AGENTS.md. Its use of file-based query/response (not HTTP) is not documented. An operator reading the script for guidance on validator quality testing would not realize the HTTP path is unexercised.
Codebase evidence: `tests/e2e/local_loop.sh` uses `--query-file` and `--response-file` flags, not `--chain --subnet` miner discovery. AGENTS.md "current operator loop" documents this as the primary proof.
Owns: Add a comment block in `tests/e2e/local_loop.sh` explaining: (a) this script exercises file-based query/response, (b) the HTTP axon path requires a live chain + miner binary + validator binary, (c) reference the docker-compose proof for the HTTP path coverage.
Integration touchpoints: `tests/e2e/local_loop.sh`, `docker-compose.yml`.
Scope boundary: Shell script comments only. No code changes.
Required tests: `grep` for the new documentation in the shell file.
Dependencies: None.
Completion signal: `tests/e2e/local_loop.sh` has a comment block clearly distinguishing file-based from HTTP paths and referencing the docker-compose proof for HTTP coverage.

---

### `- [ ] NEM-007A Add diagnostic quality to try_state emission failure`

Spec: INV-005
Why now: The `try_state` emission guard (`TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA = 1`) only asserts `diff <= 1` without revealing the magnitude. If the guard triggers, an operator or developer sees "diff > 1 rao" without knowing whether it is 2 rao or 2 million rao. The diagnostic quality is insufficient for root-cause analysis.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs:20-30` — the diff assertion with no magnitude disclosure. `crates/myosu-chain/pallets/game-solver/src/lib.rs:75` — `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA = 1`.
Owns: Change the `try_state` assertion to: (a) compute and log the actual diff magnitude when `diff > 1`, (b) include both `expected_total_issuance` and `live_total_issuance` in the log message, (c) add a log-level `error!` for diff > 1 rao so it appears in node logs. Do not change the guard threshold.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs`.
Scope boundary: Logging/diagnostics improvement. No changes to emission logic or thresholds.
Required tests: `cargo test -p pallet-game-solver -- try_state` passes. Add a test that injects a 2-rao diff and verifies the improved diagnostic message appears in logs.
Dependencies: None.
Completion signal: `cargo test -p pallet-game-solver -- try_state` passes with the improved diagnostic message. The log output for a >1-rao diff includes the actual diff magnitude, expected, and live issuance values.

---

### `- [ ] NEM-007B Add fuzzing harness for epoch emission accumulation`

Spec: INV-005
Why now: The coinbase sweep (`stage_0_coinbase_truncation_dust_is_closed_exactly_sweep`) covers the coinbase split. NEM-003A covers per-UID epoch accumulation. A property-based fuzzing harness would provide continuous regression detection for the emission accounting invariant.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs:599-633` — deterministic sweep. No fuzzing harness exists for the emission accounting paths.
Owns: Add a `#[test]` or `#[cfg(test)]` property test that generates random: (a) number of miners, (b) number of validators, (c) epoch emission values, (d) owner cut percentage, and asserts: `sum(server_emission_per_uid) + sum(validator_emission_per_uid) <= total_epoch_allocation` and `sum >= total_epoch_allocation - (n_miners + n_validators)`.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`.
Scope boundary: Test-only. No production code changes.
Required tests: `cargo test -p pallet-game-solver -- fuzz_emission_accumulation` passes with 1,000+ random iterations.
Dependencies: NEM-003A (once merged, reuse the epoch emission computation infrastructure).
Completion signal: `cargo test -p pallet-game-solver -- fuzz_emission_accumulation` passes with property-test iterations. The harness detects the NEM-003 condition if reintroduced.

---

### `- [ ] NEM-008A Document INV-006 review requirement as a blocking CI gate in docs/adr/`

Spec: INV-006
Why now: INV-006 states "Core MCCFR algorithm changes require review" but the enforcement is advisory (`continue-on-error`). A silent MCCFR correctness change could be committed without triggering a mandatory review. The AGENTS.md entry confirms this is advisory-only.
Codebase evidence: `INVARIANTS.md:INV-006` — policy statement. `check_robopoker_fork_status.sh` — CI script with `continue-on-error`. `docs/robopoker-fork-changelog.md` — change log, no review-gate field.
Owns: Add an ADR under `docs/adr/` that: (a) documents INV-006's MCCFR review requirement, (b) proposes making the review requirement a blocking CI gate (e.g., adding a `review_required` boolean field to `docs/robopoker-fork-changelog.md` that gates CI on the reviewer's sign-off), (c) includes a template for MCCFR review. This does not immediately enforce the gate — it proposes the hardening and records the decision.
Integration touchpoints: `docs/adr/`, `docs/robopoker-fork-changelog.md`, CI workflow.
Scope boundary: Documentation and proposed CI hardening. Do not change CI behavior yet.
Required tests: ADR file exists at `docs/adr/XXX-mccfr-review-gate.md`.
Dependencies: None.
Completion signal: ADR exists at `docs/adr/` proposing the MCCFR review gate. The ADR is linked from `INVARIANTS.md` INV-006 section.

---

### `- [ ] NEM-008B Add INV-006 MCCFR review checklist to robopoker-fork-changelog.md`

Spec: INV-006
Why now: INV-006 requires MCCFR algorithm changes to be reviewed but provides no checklist or review criteria. A developer submitting a MCCFR-relevant commit has no guidance on what "review" means.
Codebase evidence: `docs/robopoker-fork-changelog.md` — existing changelog format with no MCCFR-specific section. `INVARIANTS.md:INV-006` — "Core MCCFR algorithm changes require review."
Owns: Add a "MCCFR Review Checklist" section to `docs/robopoker-fork-changelog.md` with: (a) a definition of what constitutes a MCCFR-relevant change, (b) the required review criteria (correctness of regret update, averaging formula, sampling method), (c) a sign-off format. Update the changelog entry for any pending MCCFR-relevant commits (currently none, but the infrastructure exists).
Integration touchpoints: `docs/robopoker-fork-changelog.md`.
Scope boundary: Documentation only.
Required tests: `cargo doc` passes with the updated changelog.
Dependencies: NEM-008A.
Completion signal: `docs/robopoker-fork-changelog.md` has an explicit MCCFR review checklist. Any future MCCFR-relevant commit changelog entry references this checklist.

---

## Follow-On Work

### `- [ ] NEM-009 Evaluate whether `Stage0SwapInterface` should be replaced with a narrower trait for stage-0`

Spec: INV-005
Why now: The `Stage0SwapInterface` trait in `lib.rs:88-168` exposes the full subtensor AMM contract (swap, sim_swap, price, fees, max/min price). Stage-0 only needs identity conversion: `amount_in == amount_out`, zero fees. A narrower trait would make the surface smaller and the NoOpSwap stub trivially implementable without the full swap machinery.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/lib.rs:88-168` — `Stage0SwapInterface` trait. `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs` — NoOpSwap stub implementing the full AMM contract.
Owns: Write an ADR evaluating: (a) the current full `SwapHandler + SwapEngine` contract, (b) the narrower stage-0 interface needed, (c) the migration path for replacing `Stage0SwapInterface` with the real AMM. Decision: either simplify the trait now or document the deferred simplification.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/lib.rs`, `crates/myosu-chain/pallets/game-solver/src/swap_stub.rs`.
Scope boundary: Research and ADR only. No implementation changes.
Required tests: None (research task).
Dependencies: RES-001 / F-003 (token economics decision informs the AMM design).
Completion signal: ADR exists evaluating the trait narrowing. Decision is recorded in `ops/decision_log.md`.

---

### `- [ ] NEM-010 Audit `BlocksSinceLastStep` unconditional increment for scalability`

Spec: Performance / Architecture
Why now: `run_coinbase::drain_pending()` increments `BlocksSinceLastStep` for ALL subnets on every block, including subnets with no activity. At scale (32 subnets × many blocks), this is storage write amplification.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs` — the `drain_pending` loop unconditionally mutates `BlocksSinceLastStep` for every subnet.
Owns: Evaluate whether the unconditional increment can be replaced with a conditional increment (only for subnets that had an epoch or registration event since the last block). Write findings in an ADR.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs`.
Scope boundary: Research and ADR only. No implementation changes.
Required tests: None (research task).
Dependencies: None.
Completion signal: ADR documents the current behavior and the proposed optimization, with a decision on whether to implement.

---

## Completed / Already Satisfied

- [x] **NEM-CO-01** Emission accounting coinbase split is verified closed (EMIT-001)
  Spec: INV-005
  Codebase evidence: `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep` in `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs:599`. `close_integer_emission_split` in `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:47-65`. `run_coinbase` summary tracks all four distribution components. `stage_0_coinbase_emission_accounting_matches_accrued_epoch_budget` integration test passes.

- [x] **NEM-CO-02** INV-004 solver-gameplay dependency boundary is enforced in CI
  Spec: INV-004
  Codebase evidence: `crates/myosu-play/tests/invariants.rs:inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other`. `cargo tree -p myosu-play` confirms no `myosu-miner` in the dependency tree. CI job "INV-004 solver-gameplay dependency boundary" runs on every PR.

- [x] **NEM-CO-03** No unsafe code in the pallet game-solver layer
  Spec: INV-005
  Codebase evidence: Grep for `unsafe` across all files in `crates/myosu-chain/pallets/game-solver/src/` returns zero matches. Grep for `unsafe_cell` and `unsafe_code` returns zero matches.

- [x] **NEM-CO-04** Epoch inconsistency is handled gracefully with skip event
  Spec: INV-001, INV-003
  Codebase evidence: `crate::EpochSkipReason::InconsistentInputState` enum in `crates/myosu-chain/pallets/game-solver/src/lib.rs:77-82`. `legacy_epoch_skip_emits_event_when_state_is_inconsistent` test in `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`. Epoch returns `Vec::new()` on inconsistent state rather than panicking.

- [x] **NEM-CO-05** Subnet dissolution clears all emission pending storage
  Spec: INV-005
  Codebase evidence: `tests/networks.rs:539-548` confirms `PendingServerEmission`, `PendingValidatorEmission`, `PendingRootAlphaDivs`, `PendingOwnerCut`, `BlocksSinceLastStep`, `LastMechansimStepBlock`, `ServingRateLimit` are all absent after `dissolve_network`. Dissolution path is exercised in `test_network_dissolve_clears_state`.

- [x] **NEM-CO-06** GameType on-chain encoding has proptest roundtrip coverage
  Spec: INV-003
  Codebase evidence: `crates/myosu-games/src/traits.rs:280-296` — `game_type_from_bytes_known`, `game_type_from_bytes_custom`, `game_type_to_bytes_roundtrip` tests. `crates/myosu-games/src/traits.rs:475-479` — `serialization_roundtrip_game_type` proptest. `crates/myosu-games/src/traits.rs:325-331` — `game_type_unicode_custom_roundtrips` test.

- [x] **NEM-CO-07** Validator deterministic scoring formula is mathematically sound
  Spec: INV-003
  Codebase evidence: `crates/myosu-validator/src/validation.rs:score_from_l1_distance()` — `1.0 / (1.0 + l1_distance.max(0.0))`. 14 unit tests in `crates/myosu-validator/src/validation.rs` test the formula. `l1_distance_union` uses union-based normalization so explicit zero entries are not double-counted. `cross_game_one_hot_degradation_stays_in_same_score_band` documents the formula's cross-game behavior.

- [x] **NEM-CO-08** Stage0NoopSwap slippage risk is acknowledged in runtime comments
  Spec: INV-005
  Codebase evidence: `crates/myosu-chain/runtime/src/lib.rs:94-97` comment: "max_price() intentionally returns the currency max value instead of a realistic ceiling. ... Any future runtime that wires a real swap engine must replace this unbounded limit before relying on price-limit semantics." This is documented but not enforced.

- [x] **NEM-CO-09** `dense_sparse_epoch_paths_produce_identical_state` test exists (behind feature flag)
  Spec: INV-003
  Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/tests/epoch.rs` (behind `legacy-subtensor-tests`). AGENTS.md entry confirms this is the parity proof. The test is behind a feature flag — see NEM-004A.

- [x] **NEM-CO-10** Robopoker fork is tracked with changelog
  Spec: INV-006
  Codebase evidence: `docs/robopoker-fork-changelog.md` — baseline (v1.0.0), workspace pin, 3 post-baseline commits, functional summary. CI `robopoker-fork-coherence` job (advisory, continue-on-error) runs `check_robopoker_fork_status.sh`.
