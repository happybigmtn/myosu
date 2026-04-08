# IMPLEMENTATION_PLAN

Generated: 2026-04-08
Codebase snapshot: trunk @ 4e0b37f
Specs: gen-20260408-013810/specs/080426-*.md

---

## Priority Work

### Phase 1: Reduce and Clean

### Phase 2: Harden and Measure

- [ ] `GATE-002` Checkpoint: Hardening verification

  Spec: `specs/070426-validator-subsystem.md`
  Why now: After emission dust is decided and test gaps are closed, verify the hardening work before proceeding to the miner quality benchmark (which builds on this foundation). This is a lightweight stop-and-verify, not a full gate.
  Codebase evidence: Verification against state produced by EMIT-001, TEST-001, TEST-002, TEST-003.
  Owns: Run test suite and confirm all new tests pass alongside existing suite. Verify WORKLIST.md items are updated.
  Integration touchpoints: All files modified in Phase 2 tasks.
  Scope boundary: Verification only. No code changes.
  Acceptance criteria: (1) `SKIP_WASM_BUILD=1 cargo test --workspace --quiet` passes. (2) `bash tests/e2e/emission_flow.sh` passes. (3) WORKLIST.md `EM-DUST-001` is resolved. (4) HTTP axon tests exist and pass.
  Verification:
  ```bash
  SKIP_WASM_BUILD=1 cargo test --workspace --quiet
  bash tests/e2e/emission_flow.sh
  bash tests/e2e/local_loop.sh
  ```
  Required tests: Full workspace test suite.
  Dependencies: EMIT-001, TEST-001, TEST-002, TEST-003.
  Estimated scope: XS
  Completion signal: Hardening verified, proceed to benchmark.

- [ ] `BENCH-001` Miner quality benchmark surface for Liar's Dice

  Spec: `specs/070426-miner-subsystem.md`
  Why now: The current validator self-scores the miner checkpoint — identical checkpoint in and out always produces `exact_match=true, score=1.0`. This is a determinism proof, not a quality benchmark. Liar's Dice has a complete native solver with no external encoder dependency, making it the viable first target for a truthful quality benchmark. Poker is blocked on sparse encoder artifacts (WORKLIST.md `MINER-QUAL-001`).
  Codebase evidence: `crates/myosu-validator/src/validation.rs` — `score_response()` compares against `solver.answer(query)` from the validator CLI checkpoint. `crates/myosu-games-liars-dice/` has a complete MCCFR implementation with no external dependency. WORKLIST.md `MINER-QUAL-001` documents the self-scoring limitation.
  Owns: (1) Add a benchmark test or command that trains a Liar's Dice strategy for varying iterations and measures exploitability independent of the self-scoring path. (2) Document minimum recommended training iterations for Liar's Dice based on measured exploitability convergence. (3) Update WORKLIST.md `MINER-QUAL-001` with the chosen approach.
  Integration touchpoints: `crates/myosu-validator/`, `crates/myosu-games-liars-dice/`, WORKLIST.md, operator guide.
  Scope boundary: Liar's Dice only. Do not attempt poker benchmark (blocked on encoder). Do not change validator scoring logic. Do not enforce minimum iterations in code (guidance only).
  Acceptance criteria: (1) A test or command exists that measures Liar's Dice strategy exploitability after N iterations. (2) Exploitability decreases with more iterations (convergence proof). (3) Minimum training iteration recommendation documented for Liar's Dice. (4) WORKLIST.md `MINER-QUAL-001` updated. (5) Poker benchmark explicitly documented as blocked on encoder artifacts.
  Verification:
  ```bash
  SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet -- quality_benchmark
  SKIP_WASM_BUILD=1 cargo test -p myosu-games-liars-dice --quiet
  ```
  Required tests: Quality benchmark test showing exploitability convergence for Liar's Dice.
  Dependencies: GATE-002, TEST-003.
  Estimated scope: M
  Completion signal: Liar's Dice has a truthful quality benchmark with documented minimum iterations.

- [ ] `GATE-003` Phase 2 decision gate

  Spec: `specs/070426-validator-subsystem.md`
  Why now: Verify that Phase 2 hardening is complete before proceeding to operator packaging. Emission dust decided, test gaps closed, Liar's Dice quality benchmark exists.
  Codebase evidence: Verification against all Phase 2 task outputs.
  Owns: Run full verification suite and produce a decision artifact.
  Integration touchpoints: All Phase 2 outputs.
  Scope boundary: Verification only.
  Acceptance criteria: (1) Emission dust policy ADR exists. (2) `try_state` threshold justified. (3) HTTP axon security tests exist. (4) Key corruption test exists. (5) Cross-game scoring documented. (6) Liar's Dice quality benchmark exists with minimum iterations documented. (7) All CI jobs green. (8) All E2E scripts pass. (9) WORKLIST.md `EM-DUST-001` and `MINER-QUAL-001` resolved or have concrete follow-on.
  Verification:
  ```bash
  SKIP_WASM_BUILD=1 cargo test --workspace --quiet
  bash tests/e2e/local_loop.sh
  bash tests/e2e/emission_flow.sh
  bash tests/e2e/validator_determinism.sh
  test -f docs/adr/011-emission-dust-policy.md
  ```
  Required tests: Full suite plus all E2E scripts.
  Dependencies: EMIT-001, TEST-001, TEST-002, TEST-003, BENCH-001.
  Estimated scope: XS
  Completion signal: Decision artifact produced. Green = proceed to Phase 3.

### Phase 3: Package and Document

- [ ] `OPS-001` README and onboarding overhaul

  Spec: `specs/070426-gameplay-advisor-surface.md`
  Why now: README is the first file a new developer reads. Currently it lacks prerequisites, references broken fabro commands, and doesn't document the fastest path to success. After Phase 1 cleanup, the README should describe the post-cleanup codebase.
  Codebase evidence: `README.md` (~143 lines). Missing: Rust edition 2024 prerequisite, WASM target, protoc. Contains `fabro run` commands that fail. Fastest meaningful test (`cargo test -p myosu-games-kuhn --quiet`, ~30s warm cache) is not documented.
  Owns: (1) Add "Prerequisites" section (Rust 2024 edition, WASM target, protoc). (2) Add "Quick Verify" section with fastest passing command. (3) Remove or replace all `fabro run` commands. (4) Separate "Developer" and "Operator" paths. (5) Point operator path to `docs/operator-guide/quickstart.md`.
  Integration touchpoints: README.md, `docs/operator-guide/quickstart.md` (link target).
  Scope boundary: README.md only. Do not rewrite operator guide. Do not add new documentation files.
  Acceptance criteria: (1) Every command in README succeeds on a fresh checkout with prerequisites installed. (2) `cargo test -p myosu-games-kuhn --quiet` is documented as the quick verify. (3) No `fabro run` commands remain. (4) Prerequisites are listed.
  Verification:
  ```bash
  ! grep -q "fabro run" README.md
  grep -q "Prerequisites\|prerequisites" README.md
  grep -q "myosu-games-kuhn" README.md
  cargo test -p myosu-games-kuhn --quiet
  ```
  Required tests: Quick verify command must pass.
  Dependencies: GATE-001 (README describes post-cleanup state).
  Estimated scope: S
  Completion signal: New developer can go from README to green test in under 5 minutes.

- [ ] `OPS-002` Planned Fabro ghost infrastructure resolution

  Spec: `specs/070426-operator-infrastructure.md`
  Why now: After DEBT-005 marks the planned Fabro references in docs, a decision must be made: create the minimum execution infrastructure or fully remove all references. DEBT-005 marks them as "planned" — this task resolves the decision.
  Codebase evidence: `fabro.toml` exists at root (24 lines, references MiniMax model). The planned `fabro/` directory does not exist yet. AGENTS.md "Bootstrap Lanes" and OS.md "Planned Control Plane" describe a planned supervision model built on Fabro/Raspberry.
  Owns: Decision and execution: (A) If Fabro is the intended substrate, create the planned `fabro/programs/myosu-bootstrap.yaml` and one working run config, OR (B) If not, remove all Fabro/Raspberry references from AGENTS.md, OS.md, and delete `fabro.toml`.
  Integration touchpoints: AGENTS.md, OS.md, `fabro.toml`, `.github/scripts/check_doctrine_integrity.sh`.
  Scope boundary: Resolve the ghost. If option A, create only the minimum viable entrypoint. Do not build a full execution framework.
  Acceptance criteria: Either (A) `fabro run <config>` executes something meaningful, OR (B) `grep -rq "fabro/" AGENTS.md OS.md README.md` returns zero results and `fabro.toml` is deleted.
  Verification:
  ```bash
  # Option B verification (more likely):
  ! grep -rq "fabro/" AGENTS.md OS.md README.md 2>/dev/null
  test ! -f fabro.toml
  bash .github/scripts/check_doctrine_integrity.sh
  ```
  Required tests: Doctrine integrity check passes.
  Dependencies: DEBT-005, GATE-001.
  Estimated scope: S
  Completion signal: No ghost infrastructure in documentation.

- [ ] `OPS-003` Container packaging research and implementation

  Spec: `specs/070426-operator-infrastructure.md`
  Why now: Operators currently must compile from source with Rust nightly, WASM target, and protoc. Cold compilation takes 10-30 minutes. Containerization is the single highest-impact operator experience improvement. However, WASM build inside Docker is untested and may require 8+ GB RAM.
  Codebase evidence: No Dockerfile, docker-compose, or container configs exist anywhere in the repo. `.github/scripts/check_operator_network_fresh_machine.sh` proves the full operator flow inside `ubuntu:22.04`, so compilation requirements are well-understood.
  Owns: (1) Research: verify WASM build works inside Docker with acceptable resource requirements. (2) If feasible: create multi-stage Dockerfile(s) for chain, miner, validator. (3) Create docker-compose.yml for single-authority devnet with miner and validator. (4) If WASM build exceeds 16GB RAM or 30 minutes: document the limitation and consider pre-built WASM approach.
  Integration touchpoints: New files at repo root: `Dockerfile` (or `Dockerfile.chain`, `Dockerfile.miner`), `docker-compose.yml`. Chain spec configuration for containerized devnet.
  Scope boundary: Devnet containers only. No production deployment. No Kubernetes. No CI Docker image building (follow-on).
  Acceptance criteria: (1) `docker build` succeeds for at least the chain binary. (2) `docker compose up` produces logs showing chain block production + miner registration + validator scoring. (3) Container images under 500MB each. (4) No secrets baked into images. (5) If WASM build is infeasible inside Docker, the decision is documented in an ADR.
  Verification:
  ```bash
  docker build -t myosu-chain -f Dockerfile.chain .
  docker compose up --abort-on-container-exit 2>&1 | tail -20
  docker images myosu-chain --format '{{.Size}}'
  ```
  Required tests: `docker compose up` produces the expected lifecycle logs.
  Dependencies: GATE-003 (binaries should be in final form before containerizing).
  Estimated scope: M
  Completion signal: Operator can `docker compose up` and see a working devnet.

---

## Follow-On Work

### Research Gates (externally blocked or long-horizon)

- [ ] `RES-001` Token economics decision document

  Spec: `specs/070426-emission-yuma-consensus.md`
  Why now: The token-economics spec is a research spec. It identifies 8 design axes that must be decided before `NoOpSwap` can be replaced. ADR 008 exists as `Proposed` but requires multi-contributor review. This is the correct next step but is externally blocked.
  Codebase evidence: `docs/adr/008-future-token-economics-direction.md` (14.5K, status: Proposed). `crates/myosu-chain/runtime/src/lib.rs:99-198` implements `Stage0NoopSwap`. WORKLIST.md references no resolution path.
  Owns: Facilitate the review of ADR 008 by at least two contributors with token-economics context. Record the review outcome. Change ADR status from `Proposed` to `Accepted` or `Rejected`.
  Integration touchpoints: `docs/adr/008-future-token-economics-direction.md`, `ops/decision_log.md`.
  Scope boundary: Research and decision only. Do NOT change swap implementation. Do NOT wire V3 AMM into runtime.
  Acceptance criteria: (1) ADR 008 status is no longer `Proposed`. (2) At least two named reviewers recorded. (3) Each of 8 design axes has a concrete recommendation. (4) Migration path from NoOpSwap to chosen model is sketched.
  Verification: Review-based (no code commands).
  Required tests: None (research task).
  Dependencies: None (runs independently of all phases).
  Blocker: External human review. Cannot be self-closed by automated work.
  Estimated scope: L
  Completion signal: Decision document accepted, NoOpSwap replacement work allowed to start.

- [ ] `RES-002` Polkadot SDK fork patch classification

  Spec: `specs/070426-runtime-architecture.md`
  Why now: The opentensor polkadot-sdk fork (rev `71629fd`) has 21 fork-only commits touching consensus-critical paths. ADR 009 documents the delta but individual patches are not classified. Classification must happen before any migration attempt.
  Codebase evidence: `docs/adr/009-polkadot-sdk-migration-feasibility.md` (8.8K). WORKLIST.md `CHAIN-SDK-001` recommends starting with patch classification. `Cargo.toml` pins to opentensor fork.
  Owns: Classify each of the 21 fork-only commits as: "Needed by myosu" (with rationale), "Subtensor-specific" (safe to drop), or "Uncertain" (needs investigation). Propose migration timeline. Update ADR 009.
  Integration touchpoints: `docs/adr/009-polkadot-sdk-migration-feasibility.md`, WORKLIST.md `CHAIN-SDK-001`.
  Scope boundary: Research and classification only. Do not change polkadot-sdk pin. Do not attempt migration.
  Acceptance criteria: (1) Each of 21 commits classified. (2) Migration timeline proposed. (3) ADR 009 updated with classification table. (4) WORKLIST.md `CHAIN-SDK-001` resolved.
  Verification: Research task — verify ADR 009 has classification table.
  Required tests: None.
  Dependencies: None (runs independently).
  Estimated scope: M
  Completion signal: Each fork commit is classified, migration path is clear.

- [ ] `RES-003` Poker quality benchmark (blocked on encoder artifacts)

  Spec: `specs/070426-validator-subsystem.md`
  Why now: Poker quality measurement requires either richer encoder artifacts (7-11 GB RAM for full encoder) or an independent reference checkpoint. The checked-in bootstrap artifacts are intentionally sparse — positive-iteration poker training fails with `isomorphism not found`. This blocks documenting minimum poker training iterations.
  Codebase evidence: `crates/myosu-games-poker/examples/bootstrap_artifacts.rs` emits a single preflop lookup. `cargo test -p myosu-games-poker -- step_reports_sparse_encoder_failure_instead_of_panicking` confirms sparse artifact limitation. WORKLIST.md `MINER-QUAL-001` documents this.
  Owns: Either (a) ship richer poker encoder artifacts as test fixtures and build an exploitability benchmark, or (b) document the hardware requirements for generating full encoder and provide a script for operators to self-generate.
  Integration touchpoints: `crates/myosu-games-poker/`, `crates/myosu-validator/`, robopoker fork.
  Scope boundary: Poker encoder artifacts and benchmark only. Do not change robopoker fork algorithm.
  Acceptance criteria: (1) A path exists to measure poker strategy quality independent of self-scoring. (2) Hardware requirements documented. (3) Minimum poker training iterations documented (or explicitly blocked with requirements).
  Verification: Research task — verify benchmark or documentation exists.
  Required tests: Poker quality benchmark test if encoder artifacts are shipped.
  Dependencies: BENCH-001 (Liar's Dice benchmark proves the pattern first).
  Blocker: Encoder artifact size and robopoker upstream limitations.
  Estimated scope: M
  Completion signal: Poker quality measurement path is documented and actionable.

---

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
  Blocker (2026-04-05, re-verified 2026-04-05): The current stage-0 validator score is not a convergence metric. `score_response()` in `crates/myosu-validator/src/validation.rs` compares the observed miner response against `solver.answer(query)` from the checkpoint supplied on the validator CLI, and the repo-owned happy-path harnesses (`tests/e2e/local_loop.sh`, `tests/e2e/validator_determinism.sh`) pass the miner checkpoint straight into that validator path, so the truthful expected result is `exact_match=true` / `score=1.0` whenever the miner response came from the same checkpoint. Poker is additionally blocked because the only checked-in bootstrap artifact path (`crates/myosu-games-poker/examples/bootstrap_artifacts.rs`) still emits a single preflop lookup, and direct re-verification against the generated `target/e2e/*/poker/encoder/manifest.json` outputs shows the same shape (`preflop.entries = 1`, no flop/turn/river files). `cargo test -p myosu-validator --quiet exact_match_scores_one`, `cargo test -p myosu-miner --quiet run_training_batch_reports_sparse_encoder_failure_cleanly`, and `cargo test -p myosu-games-poker --quiet step_reports_sparse_encoder_failure_instead_of_panicking` confirm that the current validator happy path is a self-check and that any positive poker training iteration on those sparse artifacts fails upstream with `isomorphism not found`. Until the repo has either (a) richer poker encoder artifacts plus a quality benchmark such as exploitability, or (b) an independent reference-checkpoint validator path that does not self-score the miner checkpoint, this task cannot truthfully document minimum training iterations. This task subsumes the deferred Nemesis follow-up `NEM-008`; keep one canonical queue entry here.
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type.


## Completed / Already Satisfied

- [x] `C-001` NoOpSwap identity stub implements all swap callsites
  Spec: `specs/070426-emission-yuma-consensus.md`
  Codebase evidence: `runtime/src/lib.rs` lines 99-198 define `Stage0NoopSwap` with 1:1 conversion, zero fees. Coinbase unit tests and `swap_stub.rs` tests pass.

- [x] `C-002` INV-004 solver-gameplay dependency boundary enforced in CI
  Spec: `specs/070426-game-engine-framework.md`
  Codebase evidence: CI job "INV-004 solver-gameplay dependency boundary" runs `cargo tree` check. Confirmed via `cargo tree -p myosu-play` (no myosu-miner in tree).

- [x] `C-003` Multi-game architecture with zero-change extensibility
  Spec: `specs/070426-game-engine-framework.md`
  Codebase evidence: `GameType` enum with `NlheHeadsUp`, `NlheSixMax`, `KuhnPoker`, `LiarsDice`, `Custom(String)`. Adding Liar's Dice required zero changes to poker code. Kuhn poker exists as third-game proof.

- [x] `C-004` Workspace clippy lints enforced
  Spec: `specs/070426-runtime-architecture.md`
  Codebase evidence: `Cargo.toml` workspace `[lints.clippy]`: `arithmetic-side-effects = "deny"`, `expect-used = "deny"`, `indexing-slicing = "deny"`, `unwrap-used = "deny"`.

- [x] `C-005` Validator deterministic scoring with hyperbolic formula
  Spec: `specs/070426-validator-subsystem.md`
  Codebase evidence: `validation.rs` implements `score = 1.0 / (1.0 + l1_distance)`. 14 unit tests. `validator_determinism.sh` covers poker + Liar's Dice. INV-003 has E2E coverage.

- [x] `C-006` Miner 7-step lifecycle proven
  Spec: `specs/070426-miner-subsystem.md`
  Codebase evidence: `crates/myosu-miner/src/` implements probe, register, serve, train, checkpoint, file-serve, HTTP-serve. E2E local loop proves full lifecycle.

- [x] `C-007` Gameplay surface with three modes
  Spec: `specs/070426-gameplay-advisor-surface.md`
  Codebase evidence: `myosu-play` implements smoke-test, TUI (ratatui 0.29), and pipe (newline-delimited JSON) modes. CI runs smoke tests for poker and Liar's Dice.

- [x] `C-008` Key management CLI with network-namespaced storage
  Spec: `specs/070426-operator-infrastructure.md`
  Codebase evidence: `myosu-keys` (1,956 lines) implements create, show-active, print-bootstrap, import (keyfile/mnemonic/raw-seed), list, export, switch, change-password. Network-namespaced via `--network devnet|testnet|finney`.

- [x] `C-009` Multi-node consensus proven
  Spec: `specs/070426-runtime-architecture.md`
  Codebase evidence: `two_node_sync.sh` proves peer-to-peer sync. `four_node_finality.sh` proves 4-authority GRANDPA finality (kill 1, 3 keep finalizing). `consensus_resilience.sh` proves restart catch-up.

- [x] `C-010` Aura + GRANDPA consensus with 4 chain spec variants
  Spec: `specs/070426-runtime-architecture.md`
  Codebase evidence: `construct_runtime!` includes `pallet_aura` (index 3) and `pallet_grandpa` (index 4). Chain specs: localnet, devnet, testnet (test_finney), finney.

- [x] `C-011` Stage-0 emission flow proven end-to-end
  Spec: `specs/070426-emission-yuma-consensus.md`
  Codebase evidence: `cargo test -p pallet-game-solver -- stage_0` (26 tests). `tests/e2e/emission_flow.sh`. `tests/e2e/cross_node_emission.sh`. Truncation sweep measures 2 rao/block worst-case drift.

- [x] `C-012` GameType on-chain encoding with proptest roundtrip
  Spec: `specs/070426-game-engine-framework.md`
  Codebase evidence: `GameType::from_bytes` / `to_bytes` with canonical byte encoding. `#[non_exhaustive]` enum. Proptest fuzzing and serde roundtrip tests pass.

- [x] `C-013` StrategyResponse wire codec with 1 MiB decode budget
  Spec: `specs/070426-game-engine-framework.md`
  Codebase evidence: `MAX_DECODE_BYTES` set to 1 MiB. Probabilities sum check within epsilon 0.001. Tests verify oversized payloads rejected.

- [x] `C-014` Robopoker fork tracked with changelog
  Spec: `specs/070426-game-engine-framework.md`
  Codebase evidence: `docs/robopoker-fork-changelog.md` (2.2K). Workspace pins to `happybigmtn/robopoker` rev `04716310`. CI `robopoker-fork-coherence` job (advisory, continue-on-error). INV-006 documented.

- [x] `C-015` Operator guide documentation
  Spec: `specs/070426-operator-infrastructure.md`
  Codebase evidence: `docs/operator-guide/quickstart.md` (10.0K), `architecture.md` (7.2K), `troubleshooting.md` (14.2K), `upgrading.md` (7.8K).

- [x] `C-016` CI pipeline with 9+ jobs
  Spec: `specs/070426-operator-infrastructure.md`
  Codebase evidence: `.github/workflows/ci.yml` (419 lines). Jobs: repo-shape, active-crates (check/test/clippy/fmt), chain-core, E2E scripts (7), doctrine-integrity, dependency-audit, plan-quality, operator-network, INV-004 boundary.

- [x] `C-017` E2E test suite covering all critical paths
  Spec: `specs/070426-emission-yuma-consensus.md`
  Codebase evidence: 7 E2E scripts: `local_loop.sh`, `validator_determinism.sh`, `four_node_finality.sh`, `consensus_resilience.sh`, `cross_node_emission.sh`, `emission_flow.sh`, `two_node_sync.sh`.

- [x] `C-018` Structured report types for miner and validator
  Spec: `specs/070426-validator-subsystem.md`
  Codebase evidence: 6 report types each in miner and validator (MINER, REGISTRATION, AXON, HTTP, TRAINING, STRATEGY / VALIDATOR, REGISTRATION, SUBTOKEN, PERMIT, VALIDATION, WEIGHTS).

- [x] `C-019` Chain client RPC wrapper
  Spec: `specs/070426-operator-infrastructure.md`
  Codebase evidence: `myosu-chain-client` (2,203 lines, 16 tests). Typed Substrate RPC methods for subnet queries, registration, weight submission.

- [x] `C-020` Operator bundle packaging and fresh-machine proof
  Spec: `specs/070426-operator-infrastructure.md`
  Codebase evidence: `.github/scripts/prepare_operator_network_bundle.sh` generates bundle with devnet-spec.json, test-finney-spec.json, bundle-manifest.toml. `.github/scripts/check_operator_network_fresh_machine.sh` proves the flow inside ubuntu:22.04.
