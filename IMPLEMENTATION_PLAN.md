# IMPLEMENTATION_PLAN

Generated: 2026-04-05
Codebase snapshot: trunk @ ba63a7d + local
Specs: gen-20260405-145446/specs/050426-*.md

---

## Priority Work

### Completed prerequisites

- `P-001` is already satisfied by commit `ba63a7d` (`myosu: auto loop checkpoint`), which landed the zero-dividend fallback, epoch consistency guard, validator scoring change, decode budget tightening, INV-004 CI gate, and Cargo.toml de-workspacing that the older queue still described as uncommitted.
- `P-002` is satisfied in the current local slice: `cargo test -p pallet-game-solver -- truncation` now sweeps 1 / 100 / 1_000 / 10_000 accrued blocks across representative emission rates and measures a worst-case stage-0 drift of 2 rao per accrued block (6 rao over the default tempo-2 epoch). The correction decision is intentionally deferred to `WORKLIST.md`.

### Checkpoint: Chain core confidence

After P-001 through P-007: emission is measured, E2E tests are wired, CI is hardened, dead code is audited. Pause and verify trunk CI is green, all E2E scripts pass, and no new regressions. Re-evaluate scope before proceeding to multi-node and operator work.

---

### Cluster 5: Validation and Scoring Hardening (dependency: P-001)

---

### Cluster 6: Miner HTTP Axon Gaps (dependency: P-001)

---

### Checkpoint: Scoring and serving confidence

After P-008 through P-010: wire codecs are fuzz-tested, determinism is verified across all games, and the Liar's Dice HTTP gap is resolved. Verify CI green before proceeding to multi-node work.

Ready-state note (2026-04-05): after live repo re-verification, no unchecked task
in this queue is implementation-ready. `F-003` still depends on external
multi-contributor review being recorded on ADR 008, and `F-007` still depends
on a truthful quality benchmark surface; the current validator path reuses the
miner checkpoint as its oracle, and the checked-in poker bootstrap artifacts are
still too sparse for positive-iteration training benchmarks.

---

## Follow-On Work

### Operator Tooling and Onboarding

### Token Economics Research Gate

- [ ] `F-003` Token economics decision document

  Spec: `specs/050426-token-economics.md`
  Why now: The token-economics spec is explicitly a research spec, not an implementation spec. It identifies 8+ design axes (single vs dual token, AMM type, fee model, registration cost, emission schedule) that must be decided before `NoOpSwap` can be replaced. No implementation work should begin until this decision document exists.
  Codebase evidence: `crates/myosu-chain/pallets/swap-interface/src/lib.rs` (SwapEngine trait), `crates/myosu-chain/pallets/swap/` (V3 AMM implementation exists but is not wired into stage-0 runtime), `runtime/src/lib.rs` (Stage0NoopSwap).
  Owns: Decision document evaluating all design axes with recommendations.
  Integration touchpoints: Swap interface trait, swap pallet, runtime swap config, emission pipeline.
  Scope boundary: Research and document only. Do NOT change swap implementation. Do NOT wire V3 AMM into runtime.
  Acceptance criteria: (1) Each design axis from the spec has a concrete recommendation with rationale. (2) Migration path from NoOpSwap to chosen model is sketched. (3) The review requirement matches `specs/050426-token-economics.md`: the document must be reviewed by at least two contributors with token-economics context before this task closes.
  Verification: Review-based.
  Required tests: None (research task).
  Dependencies: P-002 (emission understanding informs economic model).
  Blocker (2026-04-05, re-verified 2026-04-05): `docs/adr/008-future-token-economics-direction.md` now records the repo-local recommendation, but it still says `Status: Proposed` and `Deciders: pending maintainer review required by specs/050426-token-economics.md`. A repo search found no recorded multi-contributor signoff in `docs/adr/README.md`, `ops/decision_log.md`, or `.github/`. The spec requires review by at least two contributors with context before `F-003` can be removed from the queue, so this remains blocked on external review rather than code changes.
  Estimated scope: L
  Completion signal: Decision document exists, the multi-contributor review is recorded, and only then is `NoOpSwap` replacement work allowed to start.

### Miner Convergence Gate

- [ ] `F-007` Research: Minimum training iterations for meaningful strategy quality

  Spec: `specs/050426-mining-surface.md`
  Why now: The spec notes "no convergence gate exists" — a miner can train for 1 iteration and serve garbage. Validators score it low, but operators have no guidance on minimum training.
  Codebase evidence: `crates/myosu-miner/src/` training loop and `--train-iterations` flag accept any non-negative iteration count without a quality gate. `crates/myosu-validator/src/validation.rs` scores a miner response against `solver.answer(query)` from the checkpoint passed on the validator CLI, and both `tests/e2e/local_loop.sh` and `tests/e2e/validator_determinism.sh` currently pass the miner-produced checkpoint straight back into validator scoring.
  Owns: Research document or code comment with recommended minimums per game type.
  Integration touchpoints: Miner CLI documentation, operator guide.
  Scope boundary: Measure and document. Do not enforce in code (that's follow-on).
  Acceptance criteria: (1) A training-quality threshold that actually varies with solver quality is documented for Poker and Liar's Dice, using a truthful benchmark surface (exact exploitability or comparison against an independent reference checkpoint rather than the current self-check validator path). (2) Operator guide updated with recommendation.
  Verification: Run miner with varying iteration counts against the chosen quality benchmark. Do not use the current same-checkpoint validator exact-match path as convergence evidence.
  Required tests: None (research task).
  Dependencies: P-009 (determinism verified across games).
  Blocker (2026-04-05, re-verified 2026-04-05): The current stage-0 validator score is not a convergence metric. `score_response()` in `crates/myosu-validator/src/validation.rs` compares the observed miner response against `solver.answer(query)` from the checkpoint supplied on the validator CLI, and the repo-owned happy-path harnesses (`tests/e2e/local_loop.sh`, `tests/e2e/validator_determinism.sh`) pass the miner checkpoint straight into that validator path, so the truthful expected result is `exact_match=true` / `score=1.0` whenever the miner response came from the same checkpoint. Poker is additionally blocked because the only checked-in bootstrap artifact path (`crates/myosu-games-poker/examples/bootstrap_artifacts.rs`) still emits a single preflop lookup, and `cargo test -p myosu-miner --quiet run_training_batch_reports_sparse_encoder_failure_cleanly` plus `cargo test -p myosu-games-poker --quiet step_reports_sparse_encoder_failure_instead_of_panicking` confirm that any positive poker training iteration on those sparse artifacts fails upstream with `isomorphism not found`. Until the repo has either (a) richer poker encoder artifacts plus a quality benchmark such as exploitability, or (b) an independent reference-checkpoint validator path that does not self-score the miner checkpoint, this task cannot truthfully document minimum training iterations. This task subsumes the deferred Nemesis follow-up `NEM-008`; keep one canonical queue entry here.
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type.


## Completed / Already Satisfied

- [x] `C-001` NoOpSwap identity stub implements all 37 swap callsites
  Spec: `specs/050426-token-economics.md`
  Codebase evidence: `runtime/src/lib.rs` lines 89-150 define `Stage0NoopSwap` with 1:1 conversion, zero fees. `swap_stub.rs` documents `max_price()` returning `Balance::max_value()` (with new documentation from working tree).

- [x] `C-002` INV-004 solver-gameplay dependency boundary enforced in CI
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: Working tree adds `cargo tree` check to ci.yml verifying `myosu-play` does not depend on `myosu-miner` and vice versa. Confirmed via `cargo tree -p myosu-play` (no myosu-miner in tree).

- [x] `C-003` Multi-game architecture with zero-change extensibility
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: `GameRegistry::supported()` returns 4 games. Poker, Kuhn, Liar's Dice all implement `CfrGame`/`GameRenderer` traits. Adding Liar's Dice required zero changes to poker code.

- [x] `C-004` Workspace clippy lints enforced (arithmetic_side_effects, expect_used, indexing_slicing, unwrap_used)
  Spec: `specs/050426-ci-invariant-enforcement.md`
  Codebase evidence: Workspace `Cargo.toml` `[lints.clippy]` section. CI `chain-clippy` job runs with `-D warnings`.

- [x] `C-005` Validator scoring with hyperbolic formula and determinism test
  Spec: `specs/050426-validation-surface.md`
  Codebase evidence: Working tree changes `validation.rs` to `score = 1.0 / (1.0 + l1_distance)` with `score_from_l1_distance()` function. 14 unit tests. `validator_determinism.sh` in CI.

- [x] `C-006` Miner 7-step lifecycle (probe, register, serve, train, checkpoint, file-serve, HTTP-serve)
  Spec: `specs/050426-mining-surface.md`
  Codebase evidence: `crates/myosu-miner/src/lib.rs` (~2.2K lines) implements full lifecycle. `axon.rs` handles HTTP serving for poker. Training, checkpointing, and file-based serving work for all games.

- [x] `C-007` Gameplay surface with three modes (smoke-test, TUI, pipe)
  Spec: `specs/050426-gameplay-surface.md`
  Codebase evidence: `crates/myosu-play/src/entrypoint.rs` (~3.1K lines). `--smoke-test` flag, `train` subcommand (TUI), pipe protocol. CI runs smoke tests.

- [x] `C-008` Key management (`myosu-keys` crate with create, import, export, switch)
  Spec: `specs/050426-operator-tooling.md`
  Codebase evidence: `crates/myosu-keys/src/lib.rs` implements all documented commands. Network-namespaced storage.

- [x] `C-009` Two-node block sync proven
  Spec: `specs/050426-network-consensus.md`
  Codebase evidence: `tests/e2e/two_node_sync.sh` (8.7K) exists and is referenced in spec as proven.

- [x] `C-010` Aura + GRANDPA consensus configured with 4 chain spec variants
  Spec: `specs/050426-chain-runtime-pallet.md`
  Codebase evidence: Runtime `construct_runtime!` includes `pallet_aura` (3) and `pallet_grandpa` (4). Chain specs: `localnet`, `devnet`, `testnet`, `finney`.

- [x] `C-011` Epoch consistency guard (skip epoch on inconsistent state)
  Spec: `specs/050426-emission-epoch-mechanism.md`
  Codebase evidence: Working tree adds `is_epoch_input_state_consistent(netuid)` check at entry of both `epoch()` and `epoch_dense()` in `run_epoch.rs`. Two tests in `epoch.rs` verify the guard.

- [x] `C-012` Zero-dividend fallback distributes by stake weight
  Spec: `specs/050426-emission-epoch-mechanism.md`
  Codebase evidence: Working tree adds stake-weighted fallback in `run_coinbase.rs` `calculate_dividend_distribution` when `total_dividends == 0`. Test verifies 750/250 distribution for 3:1 stake ratio.

- [x] `C-013` Decode budget hardened to 1 MiB for poker wire codec
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: Working tree changes `MAX_DECODE_BYTES` from 256 MiB to 1 MiB in both `solver.rs` and `wire.rs`. Tests verify oversized payloads are rejected.

- [x] `C-014` Operator guide documentation exists
  Spec: `specs/050426-operator-tooling.md`
  Codebase evidence: `docs/operator-guide/quickstart.md`, `docs/operator-guide/upgrading.md`, `docs/operator-guide/troubleshooting.md`, `docs/operator-guide/architecture.md` all exist.

- [x] `C-015` Six environment variable contracts documented
  Spec: `specs/050426-operator-tooling.md`
  Codebase evidence: `MYOSU_KEY_PASSWORD`, `MYOSU_CONFIG_DIR`, `MYOSU_SUBNET`, `MYOSU_WORKDIR`, `MYOSU_CHAIN`, `MYOSU_OPERATOR_CHAIN` referenced in operator tooling spec and code.

- [x] `C-016` GameType on-chain encoding with proptest roundtrip
  Spec: `specs/050426-game-trait-interface.md`
  Codebase evidence: `GameType::from_bytes` / `to_bytes` in `myosu-games` crate with `#[non_exhaustive]` enum.

- [x] `C-017` Structured report types for miner, validator, and key management
  Spec: `specs/050426-validation-surface.md`
  Codebase evidence: 6 report types each in miner and validator (MINER, REGISTRATION, AXON, HTTP, TRAINING, STRATEGY / VALIDATOR, REGISTRATION, SUBTOKEN, PERMIT, VALIDATION, WEIGHTS).
