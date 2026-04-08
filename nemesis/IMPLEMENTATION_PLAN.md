# IMPLEMENTATION_PLAN

Generated: 2026-04-08  
Audit findings: `nemesis/nemesis-audit.md`  
Codebase snapshot: trunk @ 4e0b37f

---

## Priority Work

### `- [ ] NEM-001A Document validator score self-reference limitation in E2E scripts`

Spec: INV-003 documentation
Why now: The `local_loop.sh` and `validator_determinism.sh` happy-path proofs report `score=1.0 exact_match=true` on zero-iteration checkpoints. Without explicit documentation, operators may misread these as quality proofs rather than self-consistency proofs.
Codebase evidence: `tests/e2e/local_loop.sh:267-290` feeds miner checkpoint back into validator. `crates/myosu-validator/src/validation.rs:217-260` computes `solver.answer(query)` from the same checkpoint the miner produced.
Owns: Add header comments to `tests/e2e/local_loop.sh` and `tests/e2e/validator_determinism.sh` explaining the self-referential limitation.
Integration touchpoints: E2E proof surfaces, AGENTS.md operator loop section.
Scope boundary: Shell script comments only. No code changes.
Required tests: `grep` for new comments in both shell files.
Dependencies: None.
Completion signal: Both scripts have header comments stating the score is a self-consistency proof and that independent quality benchmarking (e.g., `quality_benchmark` for Liar's Dice) is required for genuine quality signal.

---

### `- [ ] NEM-001B Add independent quality reference test for Poker`

Spec: INV-003 quality verification
Why now: The stage-0 validator scoring has no independent quality reference for Poker. The Liar's Dice `quality_benchmark` test serves this role for that game. Poker needs an equivalent so that minimum training iterations research has a truthful surface.
Codebase evidence: `crates/myosu-validator/src/validation.rs:583-607` has `quality_benchmark_liars_dice_exploitability_converges`. No equivalent exists for poker. `crates/myosu-games-poker/examples/bootstrap_artifacts.rs` emits sparse encoder that fails positive iteration training.
Owns: Add `quality_benchmark_poker_exploitability_converges` test to `crates/myosu-validator/src/validation.rs` that trains fresh solvers at 0/128/256/512 iterations and measures exploitability improvement.
Integration touchpoints: `crates/myosu-validator/src/validation.rs`, `myosu-games-poker` solver.
Scope boundary: Test-only. Does not change validator scoring formula. Does not require full PostgreSQL encoder.
Required tests: `cargo test -p myosu-validator -- quality_benchmark_poker` passes. Test documents poker exploitability ladder showing measurable quality improvement across iteration counts.
Dependencies: None (can use hand-crafted fixed-profile reference).
Completion signal: Test passes and documents exploitability drop from ~0.85 (0 iter) to ~0.70 (512 iter) for poker.

---

### `- [ ] NEM-002A Add compile-time guard for swap price limit bound`

Spec: INV-005 slippage protection
Why now: `Stage0NoopSwap::max_price()` returning `C::MAX` is a documented time-bomb. A future AMM substitution that forgets to bound the price limit would silently expose staking to slippage exploitation.
Codebase evidence: `crates/myosu-chain/runtime/src/lib.rs:94-97` acknowledges risk in prose. `crates/myosu-chain/pallets/game-solver/src/staking/add_stake.rs:78` uses `stage0_max_price()`. `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:141` uses it in emission swap.
Owns: Add `MAX_VALID_SWAP_PRICE_LIMIT: u64` associated constant to `Stage0SwapInterface` trait. Default to `u64::MAX` for stage-0. Add static assertion in runtime that any non-NoOpSwap impl must set bound below `u64::MAX / 2`.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/lib.rs` trait definition, `crates/myosu-chain/runtime/src/lib.rs` impl.
Scope boundary: Type-level enforcement. No runtime behavior changes.
Required tests: `cargo build -p myosu-chain-runtime` passes. Attempting to compile a real AMM with unbounded max_price fails at compile time.
Dependencies: None.
Completion signal: Compilation fails if any `SwapInterface` impl returns max_price above threshold without explicit `#[allow(unbounded_swap_price)]` attribute.

---

### `- [ ] NEM-003A Add epoch per-UID emission accumulation sweep test`

Spec: INV-005 emission accounting
Why now: The `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep` verifies coinbase split closes exactly. It does not verify that `sum(server_emission_per_uid) + sum(validator_emission_per_uid)` equals total epoch allocation after per-UID truncation.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:490-530` — per-UID `server_emission` and `validator_emission` computed independently with `saturating_to_num::<u64>()`. `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs:599-633` — existing sweep only covers coinbase split.
Owns: Add unit test `epoch_per_uid_emission_sum_equals_total` that generates synthetic epoch outputs for n=1..16 miners, n=0..4 validators, sums per-UID emissions, and asserts sum equals total epoch emission minus owner cut within epsilon of (n_miners + n_validators) rao.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`.
Scope boundary: Test-only. No emission math changes.
Required tests: New test passes covering representative UID counts and epoch emission values.
Dependencies: None.
Completion signal: `cargo test -p pallet-game-solver -- epoch_per_uid` passes with documented truncation gap bound.

---

### `- [ ] NEM-003B Document epoch per-UID truncation in run_epoch.rs`

Spec: INV-005 documentation
Why now: The epoch math uses `I96F32 → u64` truncation per UID. The gap is bounded by the number of UIDs but is not documented. A future maintainer might assume coinbase `close_integer_emission_split` coverage extends to epoch path.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs:490-530` — emission computation without truncation discussion.
Owns: Add doc comment at emission computation section noting: (1) per-UID truncation floors are bounded by n_validators + n_miners rao per epoch, (2) this path is separate from coinbase split verified by `close_integer_emission_split`, (3) NEM-003A test enforces the bound.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`.
Scope boundary: Documentation only.
Required tests: `cargo doc -p pallet-game-solver` passes with new comments.
Dependencies: NEM-003A.
Completion signal: Comments exist explaining truncation boundary and test coverage.

---

### `- [ ] NEM-004 Add HTTP axon integration test for validator scoring`

Spec: INV-005 end-to-end wiring
Why now: The miner HTTP axon + validator HTTP scoring path has no automated test. The file-based E2E harness bypasses this path entirely.
Codebase evidence: `crates/myosu-miner/src/axon.rs` — HTTP axon server. `crates/myosu-validator/src/chain.rs` — chain query for miner discovery. `tests/e2e/local_loop.sh` — file-based, not HTTP, for scoring.
Owns: Add integration test in `crates/myosu-validator/src/` that: (a) starts mock HTTP server simulating miner axon, (b) exercises validator's miner discovery and HTTP query path, (c) asserts HTTP request/response round-trips correctly. Use `wiremock` or `tiny_http` test server.
Integration touchpoints: `crates/myosu-validator/src/chain.rs`, `crates/myosu-validator/src/validation.rs`.
Scope boundary: Integration test within validator crate. Does not require chain binary.
Required tests: `cargo test -p myosu-validator -- http_axon` passes.
Dependencies: Add test-only dependency (wiremock or similar).
Completion signal: Test exercises full miner discovery → HTTP query → strategy decode path.

---

### `- [ ] NEM-005 Strengthen INV-004 check to catch transitive deps`

Spec: INV-004 enforcement
Why now: Current `cargo tree` check only prevents direct `myosu-play → myosu-miner` edges. Transitive import through shared deps would not be caught.
Codebase evidence: `crates/myosu-play/tests/invariants.rs:35-48` — only checks for presence of forbidden package name in output.
Owns: Add secondary check: (a) verify `myosu-chain-client` does not expose any type originating from `myosu-miner` in its public API, (b) add test that imports all `myosu-chain-client` public items and confirms none require `myosu-miner`.
Integration touchpoints: `crates/myosu-chain-client/Cargo.toml`, `crates/myosu-play/tests/invariants.rs`.
Scope boundary: Verification test addition. No crate boundary changes.
Required tests: New `inv_004_chain_client_does_not_re_export_miner_types` test passes. Existing test continues to pass.
Dependencies: None.
Completion signal: `cargo test -p myosu-play -- inv_004` passes with both subtests.

---

### `- [ ] NEM-006 Improve try_state emission diagnostics`

Spec: INV-005 diagnostic quality
Why now: `try_state` guard only asserts `diff <= 1` with generic message. If triggered, operators see "diff > 1 rao" without knowing magnitude or root cause.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs:20-30` — minimal diff check. `crates/myosu-chain/pallets/game-solver/src/lib.rs:75` — `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA = 1`.
Owns: Change assertion to: (a) compute and log actual diff magnitude when `diff > 0`, (b) include both `expected_total_issuance` and `live_total_issuance` in log, (c) use `log::error!` for visibility. Keep threshold at 1 rao.
Integration touchpoints: `crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs`.
Scope boundary: Logging/diagnostics improvement. No logic or threshold changes.
Required tests: `cargo test -p pallet-game-solver -- try_state` passes. Test with injected 2-rao diff verifies improved diagnostic.
Dependencies: None.
Completion signal: Log output for >0 rao diff includes actual diff magnitude, expected, and live values.

---

### `- [ ] NEM-007 Document INV-006 MCCFR review gate as blocking in CI`

Spec: INV-006 enforcement
Why now: INV-006 states MCCFR algorithm changes require review but enforcement is advisory (`continue-on-error: true`). A silent MCCFR correctness change could ship without review.
Codebase evidence: `docs/robopoker-fork-changelog.md` — tracking doc. `.github/workflows/ci.yml` — `robopoker-fork-coherence` job with `continue-on-error: true`. `INVARIANTS.md:INV-006` — policy statement.
Owns: Create ADR at `docs/adr/007-mccfr-review-gate.md` documenting: (a) current advisory enforcement, (b) proposal to make MCCFR divergence a blocking gate, (c) criteria for what constitutes MCCFR-relevant change (regret update, averaging formula, sampling method), (d) review checklist template.
Integration touchpoints: `docs/adr/`, `docs/robopoker-fork-changelog.md`, CI workflow.
Scope boundary: Documentation and proposed process hardening. Does not change CI behavior yet.
Required tests: ADR file exists and is linked from `INVARIANTS.md` INV-006 section.
Dependencies: None.
Completion signal: ADR exists at `docs/adr/007-mccfr-review-gate.md` with review checklist and proposed blocking gate criteria.

---

## Follow-On Work

### `- [ ] NEM-F01 Evaluate narrower Stage0SwapInterface trait`

Spec: INV-005 architecture
Why now: `Stage0SwapInterface` exposes full subtensor AMM contract. Stage-0 only needs identity conversion. A narrower trait would reduce surface area.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/lib.rs:88-168` — full trait. `crates/myosu-chain/runtime/src/lib.rs:108-180` — NoOpSwap impl.
Owns: Write ADR evaluating: (a) current full contract, (b) narrower stage-0 interface, (c) migration path. Decision: simplify now or defer.
Integration touchpoints: `lib.rs` trait, runtime impl.
Scope boundary: Research and ADR only. No implementation.
Required tests: None (research task).
Dependencies: RES-001 / F-003 (token economics decision).
Completion signal: ADR exists with decision recorded in `ops/decision_log.md`.

---

### `- [ ] NEM-F02 Audit BlocksSinceLastStep unconditional increment`

Spec: Performance / Architecture
Why now: `run_coinbase::drain_pending()` increments `BlocksSinceLastStep` for ALL subnets on every block, including subnets with no activity. At scale this is storage write amplification.
Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:296-305` — unconditional increment.
Owns: Evaluate whether unconditional increment can be replaced with conditional (only for subnets with epoch or registration activity). Write findings in ADR.
Integration touchpoints: `run_coinbase.rs`.
Scope boundary: Research and ADR only.
Required tests: None (research task).
Dependencies: None.
Completion signal: ADR documents current behavior, proposed optimization, and decision.

---

## Completed / Already Satisfied

- [x] **NEM-CO-01** Emission accounting coinbase split is verified closed  
  Spec: INV-005  
  Codebase evidence: `stage_0_coinbase_truncation_dust_is_closed_exactly_sweep` in `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs:599-633`. `close_integer_emission_split` in `crates/myosu-chain/pallets/game-solver/src/coinbase/run_coinbase.rs:47-65`.

- [x] **NEM-CO-02** Dense/sparse epoch parity test runs in default build  
  Spec: INV-003  
  Codebase evidence: `dense_sparse_epoch_paths_produce_identical_state` in `crates/myosu-chain/pallets/game-solver/src/tests/determinism.rs:217`. Test is NOT behind feature flag and runs in default builds.

- [x] **NEM-CO-03** INV-004 solver-gameplay dependency boundary enforced in CI  
  Spec: INV-004  
  Codebase evidence: `crates/myosu-play/tests/invariants.rs:inv_004_solver_and_gameplay_bins_do_not_depend_on_each_other`. CI job runs on every PR.

- [x] **NEM-CO-04** No unsafe code in pallet game-solver layer  
  Spec: INV-005  
  Codebase evidence: Grep for `unsafe` across all files in `crates/myosu-chain/pallets/game-solver/src/` returns zero matches.

- [x] **NEM-CO-05** Epoch inconsistency handled gracefully with skip event  
  Spec: INV-001, INV-003  
  Codebase evidence: `EpochSkipReason::InconsistentInputState` in `crates/myosu-chain/pallets/game-solver/src/lib.rs:77-82`. `legacy_epoch_skip_emits_event_when_state_is_inconsistent` test in `stage_0_flow.rs`.

- [x] **NEM-CO-06** Subnet dissolution clears all emission pending storage  
  Spec: INV-005  
  Codebase evidence: `tests/networks.rs:539-548` confirms `PendingServerEmission`, `PendingValidatorEmission`, `PendingRootAlphaDivs`, `PendingOwnerCut` cleared on `dissolve_network`.

- [x] **NEM-CO-07** Validator deterministic scoring formula is mathematically sound  
  Spec: INV-003  
  Codebase evidence: `score_from_l1_distance()` in `crates/myosu-validator/src/validation.rs:252-254`. 14 unit tests test the formula.

- [x] **NEM-CO-08** Stage0NoopSwap slippage risk is acknowledged  
  Spec: INV-005  
  Codebase evidence: `crates/myosu-chain/runtime/src/lib.rs:94-97` comment explicitly documents the unbounded limit caveat.

- [x] **NEM-CO-09** Robopoker fork is tracked with changelog  
  Spec: INV-006  
  Codebase evidence: `docs/robopoker-fork-changelog.md` — baseline, workspace pin, commit history, functional summary.
