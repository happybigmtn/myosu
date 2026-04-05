# IMPLEMENTATION_PLAN

Generated: 2026-04-05
Codebase snapshot: trunk @ ba63a7d + local
Specs: gen-20260405-145446/specs/050426-*.md

---

## Priority Work

### Completed prerequisites

- `P-001` is already satisfied by commit `ba63a7d` (`myosu: auto loop checkpoint`), which landed the zero-dividend fallback, epoch consistency guard, validator scoring change, decode budget tightening, INV-004 CI gate, and Cargo.toml de-workspacing that the older queue still described as uncommitted.
- `P-002` is satisfied in the current local slice: `cargo test -p pallet-game-solver -- truncation` now sweeps 1 / 100 / 1_000 / 10_000 accrued blocks across representative emission rates and measures a worst-case stage-0 drift of 2 rao per accrued block (6 rao over the default tempo-2 epoch). The correction decision is intentionally deferred to `WORKLIST.md`.

---

### Cluster 2: Emission Conservation Proof (dependency: P-001)

The truncation gap is now quantified; the remaining emission-conservation work is wiring the live devnet proof into CI so the unit-level bound has an end-to-end guardrail.

- [ ] `P-003` Wire `emission_flow.sh` E2E test into CI

  Spec: `specs/050426-emission-epoch-mechanism.md`
  Why now: `emission_flow.sh` exists (14.2K lines) but is not in CI. It tests emission accounting end-to-end on a live devnet. Without it, the truncation measurement (P-002) is only a unit-level assertion — E2E validation of emission conservation is missing.
  Codebase evidence: `tests/e2e/emission_flow.sh` exists. `.github/workflows/ci.yml` wires `local_loop.sh` and `validator_determinism.sh` but not `emission_flow.sh`.
  Owns: `.github/workflows/ci.yml` (new job or step in `integration-e2e`).
  Integration touchpoints: `integration-e2e` CI job (depends on `chain-core`), `emission_flow.sh` script, devnet chain spec.
  Scope boundary: Wire the existing script. If the script needs fixes to pass, fix minimally. Do not rewrite.
  Acceptance criteria: (1) `emission_flow.sh` runs in CI under `integration-e2e`. (2) It passes on current trunk. (3) CI failure in this script blocks merge.
  Verification: Push branch, confirm `integration-e2e` job includes emission_flow step and passes.
  Required tests: The script itself is the test.
  Dependencies: P-001 (clean trunk), P-002 (drift is quantified, so the E2E has context).
  Estimated scope: S
  Completion signal: CI runs `emission_flow.sh` and passes.

---

### Checkpoint: Emission confidence

After P-002 and P-003, the emission path has: (a) unit-level truncation bounds, (b) E2E accounting validation, (c) zero-dividend fallback. If truncation drift is unexpectedly large, stop and create a correction task before proceeding.

---

### Cluster 3: CI Hardening (dependency: P-001)

- [ ] `P-004` SHA-pin all GitHub Actions

  Spec: `specs/050426-ci-invariant-enforcement.md`
  Why now: `actions/checkout@v6` appears 10 times without SHA pin. This is a supply-chain risk: a tag can be moved to point at malicious code. Low-effort fix with high security payoff.
  Codebase evidence: `ci.yml` lines 25, 37, 65, 163, 205, 235, 247, 282, 296, 323 all use `actions/checkout@v6`.
  Owns: `.github/workflows/ci.yml`
  Integration touchpoints: All CI jobs (every job checks out code).
  Scope boundary: Pin to current SHA for each action. Add version comment. Do not change job logic or sequencing.
  Acceptance criteria: (1) Every `uses:` line in ci.yml references a full SHA with a `# vX.Y.Z` comment. (2) CI passes with pinned SHAs.
  Verification: `actionlint .github/workflows/ci.yml && zizmor .github/workflows/ci.yml`
  Required tests: CI green on the pinned workflow.
  Dependencies: P-001 (clean trunk).
  Estimated scope: XS
  Completion signal: `zizmor` reports no unpinned action findings.

- [ ] `P-005` Wire `two_node_sync.sh` E2E test into CI

  Spec: `specs/050426-network-consensus.md`
  Why now: Two-node block sync is the only proven multi-node property (spec says the script exists and passes). Wiring it into CI prevents regressions before multi-node work begins.
  Codebase evidence: `tests/e2e/two_node_sync.sh` exists (8.7K). Not referenced in `ci.yml`.
  Owns: `.github/workflows/ci.yml` (new step in `integration-e2e` or new job).
  Integration touchpoints: `integration-e2e` CI job, devnet chain spec, node binary build.
  Scope boundary: Wire the existing script. Fix minimally if needed. Do not add new multi-node tests.
  Acceptance criteria: (1) `two_node_sync.sh` runs in CI. (2) It passes. (3) Failure blocks merge.
  Verification: Push branch, confirm CI job includes two_node_sync step and passes.
  Required tests: The script itself is the test.
  Dependencies: P-001 (clean trunk).
  Estimated scope: S
  Completion signal: CI runs `two_node_sync.sh` and passes.

---

### Cluster 4: Dead Code and Storage Reduction (dependency: P-001)

- [ ] `P-006` Research: Audit stage-0 extrinsic surface and storage items

  Spec: `specs/050426-chain-runtime-pallet.md`
  Why now: The spec documents 25 stage-0 extrinsics and 193 storage items, noting both may be reducible. The spec also flags that target storage is ~80 items (per Plan 005) but no formal audit exists. Before removing anything, the reduction candidates must be identified and their removal safety confirmed.
  Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/macros/dispatches.rs` (extrinsic definitions), `crates/myosu-chain/pallets/game-solver/src/macros/config.rs` (storage items), runtime `lib.rs` (`construct_runtime!` at line 1233).
  Owns: A decision document (can be a code comment block or markdown file in `gen-20260405-145446/`) listing each extrinsic and storage item with keep/remove/defer verdict.
  Integration touchpoints: Runtime `lib.rs`, game-solver pallet dispatch surface, any RPC that reads storage.
  Scope boundary: Research and document only. Do NOT remove any extrinsics or storage items in this task.
  Acceptance criteria: (1) Every stage-0 extrinsic has a keep/remove/defer verdict with rationale. (2) Storage items are categorized as active/dead/deferred with counts. (3) A concrete removal plan is proposed for items marked "remove."
  Verification: Review-based. The document is verifiable by checking each verdict against `cargo test` and `grep` for callsites.
  Required tests: None (research task).
  Dependencies: P-001 (clean trunk).
  Estimated scope: M
  Completion signal: Decision document exists with verdicts for all 25 extrinsics and a storage item census.

- [ ] `P-007` Remove dense epoch path or add parity test

  Spec: `specs/050426-chain-runtime-pallet.md`
  Why now: Dense epoch (`epoch_dense()`) is retained "for test parity" but no CI job verifies that dense and sparse produce identical results. This is dead weight that either needs a parity assertion or removal.
  Codebase evidence: `epoch/run_epoch.rs` contains both `epoch()` (sparse, production) and `epoch_dense()`. `tests/epoch.rs` and `tests/consensus.rs` reference dense epoch. No parity test exists.
  Owns: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`, `tests/epoch.rs`.
  Integration touchpoints: Epoch tests that call dense path. Coinbase flow (only calls sparse).
  Scope boundary: Either (a) add a parity test asserting `epoch() == epoch_dense()` for a representative scenario, OR (b) remove `epoch_dense()` and update tests to use sparse only. Decision should be made based on whether any test uniquely depends on dense semantics.
  Acceptance criteria: (1) If parity test: test passes and is in CI. (2) If removal: `epoch_dense` is gone, all tests pass, no dead code remains.
  Verification: `cargo test -p pallet-game-solver -- epoch`
  Required tests: Either a parity test or updated existing tests (depending on chosen path).
  Dependencies: P-001, P-002 (emission understanding informs whether dense path has value).
  Estimated scope: S
  Completion signal: Dense epoch path is either tested for parity or removed.

---

### Checkpoint: Chain core confidence

After P-001 through P-007: emission is measured, E2E tests are wired, CI is hardened, dead code is audited. Pause and verify trunk CI is green, all E2E scripts pass, and no new regressions. Re-evaluate scope before proceeding to multi-node and operator work.

---

### Cluster 5: Validation and Scoring Hardening (dependency: P-001)

- [ ] `P-008` Add wire codec fuzz tests for poker and Liar's Dice

  Spec: `specs/050426-ci-invariant-enforcement.md`
  Why now: The CI spec notes "no fuzzing or adversarial input testing exists for wire codecs." Wire codecs are the trust boundary between miner and validator — malformed payloads could crash validators or produce incorrect scores. The decode budget was tightened (P-001) but no fuzz coverage exists.
  Codebase evidence: `crates/myosu-games-poker/src/wire.rs` (poker codec with 1MB limit), `crates/myosu-games-liars-dice/src/` (Liar's Dice codec). No `fuzz/` directory exists.
  Owns: New fuzz targets (likely `fuzz/` directories in relevant crates) or proptest-based roundtrip tests.
  Integration touchpoints: `wire.rs` in poker, codec paths in Liar's Dice, `StrategyQuery`/`StrategyResponse` types from `myosu-games`.
  Scope boundary: Fuzz decode paths only (the trust boundary). Do not fuzz solver internals. Proptest roundtrip (encode→decode→re-encode == original) is sufficient if `cargo-fuzz` setup is too heavy.
  Acceptance criteria: (1) Roundtrip property tests exist for `StrategyQuery` and `StrategyResponse` for poker and Liar's Dice. (2) Decode of random bytes does not panic (returns error). (3) Tests run in CI under `active-crates` job.
  Verification: `cargo test -p myosu-games-poker -- fuzz && cargo test -p myosu-games-liars-dice -- fuzz`
  Required tests: Proptest roundtrip tests, random-bytes-decode-doesn't-panic tests.
  Dependencies: P-001 (clean trunk, decode limits landed).
  Estimated scope: S
  Completion signal: Fuzz/property tests pass in CI.

- [ ] `P-009` Validate INV-003 determinism across game types

  Spec: `specs/050426-ci-invariant-enforcement.md`
  Why now: INV-003 (validator determinism, epsilon < 1e-6) is enforced by `validator_determinism.sh` in CI, but the spec notes this may only cover poker. Liar's Dice and Kuhn validation paths should also be covered.
  Codebase evidence: `tests/e2e/validator_determinism.sh` (11.2K). `crates/myosu-validator/src/validation.rs` handles `GameSelection::Poker`, `GameSelection::LiarsDice`, `GameSelection::Kuhn`. Unclear if E2E script exercises all three.
  Owns: `tests/e2e/validator_determinism.sh` (extend or parametrize), or new unit tests in `validation.rs`.
  Integration touchpoints: Validator binary, game-specific solver paths, CI `integration-e2e` job.
  Scope boundary: Verify existing determinism test covers all implemented games. If not, extend it. Do not add new games.
  Acceptance criteria: (1) Determinism is verified for Poker, Liar's Dice, and Kuhn. (2) Epsilon < 1e-6 for all three. (3) CI enforces this.
  Verification: `bash tests/e2e/validator_determinism.sh` (with all game types)
  Required tests: Extended determinism script or new unit tests per game type.
  Dependencies: P-001 (clean trunk, scoring formula landed).
  Estimated scope: S
  Completion signal: Determinism assertion passes for all three game types in CI.

---

### Cluster 6: Miner HTTP Axon Gaps (dependency: P-001)

- [ ] `P-010` Decision: Liar's Dice HTTP axon — implement or formally defer

  Spec: `specs/050426-mining-surface.md`
  Why now: `axon.rs:51-52` explicitly errors with `UnsupportedGame` for Liar's Dice and Kuhn HTTP serving. The spec flags this as an open question. Validators querying Liar's Dice miners over HTTP will fail. This needs a deliberate decision before operator tooling work.
  Codebase evidence: `crates/myosu-miner/src/axon.rs` lines 51-52 (`UnsupportedGame` error), line 601+ (test confirming the gate). Liar's Dice training works (`LiarsDiceSolver<1024>`), file-based strategy serving works, but HTTP path is gated off.
  Owns: Decision document or code change in `axon.rs`.
  Integration touchpoints: Validator scoring (if validators query over HTTP), operator bundle scripts, miner CLI documentation.
  Scope boundary: Either (a) implement HTTP serving for Liar's Dice (reuse poker axon pattern with Liar's Dice wire codec), or (b) document the limitation in operator guide and ensure validators use file-based scoring for Liar's Dice. Decision task — implementation is follow-on if chosen.
  Acceptance criteria: (1) Decision is made and documented. (2) If implementing: HTTP axon serves Liar's Dice strategies, test added. (3) If deferring: operator guide documents the limitation, validator determinism test uses file-based path for Liar's Dice.
  Verification: If implementing: `cargo test -p myosu-miner -- liars_dice_http`. If deferring: grep operator guide for limitation note.
  Required tests: Depends on decision.
  Dependencies: P-001 (clean trunk).
  Estimated scope: XS (decision) or S (implementation)
  Completion signal: Decision documented. If implementing, HTTP axon test passes.

---

### Checkpoint: Scoring and serving confidence

After P-008 through P-010: wire codecs are fuzz-tested, determinism is verified across all games, and the Liar's Dice HTTP gap is resolved. Verify CI green before proceeding to multi-node work.

---

### Cluster 7: Multi-Node Devnet Foundation (dependency: P-003, P-005)

- [ ] `P-011` Three-node GRANDPA finality proof

  Spec: `specs/050426-network-consensus.md`
  Why now: Two-node sync is proven (and wired into CI via P-005). Three-node GRANDPA finality is the next consensus milestone. The spec explicitly lists this as "not proven (design-phase)" and gates Phase 1 completion on it.
  Codebase evidence: `tests/e2e/two_node_sync.sh` (2-node). Chain specs in `crates/myosu-chain/node/` define `devnet` with 3 authorities. No 3-node E2E test exists.
  Owns: New E2E script `tests/e2e/three_node_finality.sh` and CI wiring.
  Integration touchpoints: Node binary, devnet chain spec (3 authorities), GRANDPA finality gadget, CI `integration-e2e` job.
  Scope boundary: Prove finality with 3 nodes. Test one-node-down tolerance (2/3 still finalizes). Do not test network partitions or restart recovery (those are follow-on).
  Acceptance criteria: (1) 3 nodes start, produce blocks, and reach GRANDPA finality. (2) Stopping 1 node does not halt finality (2/3 quorum). (3) Script runs in CI.
  Verification: `bash tests/e2e/three_node_finality.sh`
  Required tests: The E2E script is the test. Should assert finalized block height increases after epoch transitions.
  Dependencies: P-005 (two_node_sync in CI — proves the infrastructure works).
  Estimated scope: M
  Completion signal: 3-node finality script passes in CI.

- [ ] `P-012` Cross-node emission agreement test

  Spec: `specs/050426-emission-epoch-mechanism.md`
  Why now: The spec explicitly states cross-node emission agreement is "tested single-node only." Fixed-point determinism is assumed but not proven across nodes. This is the highest-risk multi-node property — if nodes disagree on emission, the chain forks.
  Codebase evidence: All epoch/coinbase tests run in single-node mock runtime. `substrate_fixed` types (I32F32, I64F64, U96F32) are deterministic per the spec but unverified across separate process instances.
  Owns: New E2E test or extension of `three_node_finality.sh` that compares emission storage across nodes after epoch transitions.
  Integration touchpoints: RPC endpoints for reading storage, epoch mechanism, coinbase pipeline, node binary.
  Scope boundary: Compare emission-related storage values across 3 nodes after N epochs. Assert bit-identical. Do not test under adversarial conditions.
  Acceptance criteria: (1) After 3+ epoch transitions on a 3-node devnet, emission storage values (total issuance, per-subnet pending, stake maps) are identical across all nodes. (2) Test runs in CI.
  Verification: E2E script that queries storage via RPC on all 3 nodes and diffs.
  Required tests: The E2E script.
  Dependencies: P-011 (3-node devnet running), P-002 (truncation drift quantified).
  Estimated scope: M
  Completion signal: Cross-node emission agreement test passes in CI.

---

### Checkpoint: Multi-node confidence

After P-011 and P-012: 3-node finality is proven, cross-node emission agreement is verified. This satisfies the Phase 1 gate from the network-consensus spec. Re-evaluate whether Phase 2 (operator packaging) work should begin or whether restart resilience testing is needed first.

---

## Follow-On Work

### Operator Tooling and Onboarding

- [ ] `F-001` Fresh-machine operator bundle test

  Spec: `specs/050426-operator-tooling.md`
  Why now: The spec explicitly states "bundle has not been tested on a fresh machine outside CI." Operator onboarding cannot be trusted until this is verified.
  Codebase evidence: `.github/scripts/prepare_operator_network_bundle.sh`, `docs/operator-guide/quickstart.md`, CI `operator-network` job.
  Owns: Test procedure (Docker or VM) that runs the bundle from scratch and verifies miner+validator pair starts.
  Integration touchpoints: Bundle scripts, node binary, miner binary, validator binary, key management.
  Scope boundary: Test the existing bundle. Document failures. Do not rewrite the bundle.
  Acceptance criteria: (1) Bundle produces a running miner+validator pair on a clean Ubuntu 22.04 (or equivalent) with no pre-existing Rust toolchain (or documents exact prerequisites). (2) Failures are filed as concrete fix tasks.
  Verification: Run bundle on fresh Docker image, verify miner and validator produce expected report output.
  Required tests: The bundle test procedure itself.
  Dependencies: P-011 (multi-node devnet for realistic test).
  Estimated scope: M
  Completion signal: Bundle test passes or failures are documented as fix tasks.

- [ ] `F-002` Node restart resilience test

  Spec: `specs/050426-network-consensus.md`
  Why now: The spec lists "node restart resilience (catch-up without fork)" as unproven. Operators will restart nodes; this must work.
  Codebase evidence: No restart test exists in `tests/e2e/`.
  Owns: New E2E script testing node restart and catch-up.
  Integration touchpoints: Node binary, GRANDPA, block import.
  Scope boundary: Single node restart in a 3-node network. Verify it catches up to finalized head. Do not test simultaneous restart of all nodes.
  Acceptance criteria: (1) A restarted node catches up to the finalized head within a bounded time. (2) No fork occurs.
  Verification: E2E script.
  Required tests: The E2E script.
  Dependencies: P-011 (3-node devnet).
  Estimated scope: M
  Completion signal: Restart test passes in CI.

### Token Economics Research Gate

- [ ] `F-003` Token economics decision document

  Spec: `specs/050426-token-economics.md`
  Why now: The token-economics spec is explicitly a research spec, not an implementation spec. It identifies 8+ design axes (single vs dual token, AMM type, fee model, registration cost, emission schedule) that must be decided before `NoOpSwap` can be replaced. No implementation work should begin until this decision document exists.
  Codebase evidence: `crates/myosu-chain/pallets/swap-interface/src/lib.rs` (SwapEngine trait), `crates/myosu-chain/pallets/swap/` (V3 AMM implementation exists but is not wired into stage-0 runtime), `runtime/src/lib.rs` (Stage0NoopSwap).
  Owns: Decision document evaluating all design axes with recommendations.
  Integration touchpoints: Swap interface trait, swap pallet, runtime swap config, emission pipeline.
  Scope boundary: Research and document only. Do NOT change swap implementation. Do NOT wire V3 AMM into runtime.
  Acceptance criteria: (1) Each design axis from the spec has a concrete recommendation with rationale. (2) Migration path from NoOpSwap to chosen model is sketched. (3) Document is reviewed by at least one other contributor.
  Verification: Review-based.
  Required tests: None (research task).
  Dependencies: P-002 (emission understanding informs economic model).
  Estimated scope: L
  Completion signal: Decision document exists and is reviewed.

### Robopoker Fork Coherence

- [ ] `F-004` INV-006 automated gate for robopoker fork

  Spec: `specs/050426-game-trait-interface.md`
  Why now: INV-006 (robopoker fork coherence) has no automated CI gate. The game-trait-interface spec notes "process for upstreaming changes is undefined." Drift between the fork and upstream could introduce subtle solver bugs.
  Codebase evidence: `Cargo.toml` references `happybigmtn/robopoker` (or similar). No CI job compares fork to upstream.
  Owns: CI job or script that checks fork divergence.
  Integration touchpoints: Workspace `Cargo.toml`, `myosu-games-poker` dependency on robopoker.
  Scope boundary: Detect divergence, not resolve it. Alert on new upstream commits not in fork.
  Acceptance criteria: (1) CI job reports fork divergence count. (2) Does not block merge (advisory only).
  Verification: CI job runs and reports.
  Required tests: CI job itself.
  Dependencies: None (independent).
  Estimated scope: S
  Completion signal: CI reports fork status on each run.

### Runtime Migration Testing

- [ ] `F-005` Runtime upgrade and migration smoke test

  Spec: `specs/050426-chain-runtime-pallet.md`
  Why now: The CI spec notes "no runtime upgrade/migration tests exist." The game-solver pallet has 29 migration files. Any runtime upgrade in production could corrupt state if migrations are untested.
  Codebase evidence: 29 migration files in `crates/myosu-chain/pallets/game-solver/src/migrations/`. `tests/migration.rs` exists but only tests individual migrations, not the full upgrade path.
  Owns: E2E test or try-runtime test that applies all migrations to a snapshot.
  Integration touchpoints: Runtime `lib.rs`, migration sequence, FRAME migration hooks.
  Scope boundary: Test that `try-runtime` (or equivalent) applies all pending migrations without error on a devnet snapshot. Do not test production state.
  Acceptance criteria: (1) A migration smoke test exists. (2) It passes on a fresh devnet genesis snapshot. (3) It runs in CI.
  Verification: `try-runtime` or equivalent command.
  Required tests: The migration smoke test.
  Dependencies: P-001 (clean trunk).
  Estimated scope: M
  Completion signal: Migration smoke test passes in CI.

### Stage-0 Extrinsic Reduction (follows P-006)

- [ ] `F-006` Remove identified dead extrinsics from stage-0 surface

  Spec: `specs/050426-chain-runtime-pallet.md`
  Why now: Follows the audit in P-006. Each unnecessary extrinsic is attack surface.
  Codebase evidence: Determined by P-006 audit.
  Owns: Game-solver pallet dispatch surface, runtime integration.
  Integration touchpoints: RPC clients, operator tooling (if any tool calls removed extrinsics), CI tests.
  Scope boundary: Remove only extrinsics marked "remove" in P-006 audit. Feature-gate if removal is risky.
  Acceptance criteria: (1) Stage-0 extrinsic count decreases. (2) All tests pass. (3) No operator-facing tool breaks.
  Verification: `cargo test -p pallet-game-solver && cargo test -p myosu-chain-runtime`
  Required tests: Existing tests must still pass. Add negative tests for removed extrinsics (call returns error).
  Dependencies: P-006 (audit complete).
  Estimated scope: M
  Completion signal: Extrinsic count reduced per P-006 recommendations, CI green.

### Miner Convergence Gate

- [ ] `F-007` Research: Minimum training iterations for meaningful strategy quality

  Spec: `specs/050426-mining-surface.md`
  Why now: The spec notes "no convergence gate exists" — a miner can train for 1 iteration and serve garbage. Validators score it low, but operators have no guidance on minimum training.
  Codebase evidence: `crates/myosu-miner/src/` training loop, `--train-iterations` flag. No convergence check or quality gate.
  Owns: Research document or code comment with recommended minimums per game type.
  Integration touchpoints: Miner CLI documentation, operator guide.
  Scope boundary: Measure and document. Do not enforce in code (that's follow-on).
  Acceptance criteria: (1) Minimum iterations for score > 0.5 documented for Poker and Liar's Dice. (2) Operator guide updated with recommendation.
  Verification: Run miner with varying iteration counts, measure validator score.
  Required tests: None (research task).
  Dependencies: P-009 (determinism verified across games).
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type.

### Polkadot SDK Migration Research

- [ ] `F-008` Research: Upstream polkadot-sdk migration feasibility

  Spec: `specs/050426-chain-runtime-pallet.md`
  Why now: The workspace uses an opentensor fork of polkadot-sdk at a specific rev. The spec notes migration to upstream may be feasible if fork divergence is limited. The Cargo.toml de-workspacing (P-001) makes dependency tracking clearer.
  Codebase evidence: Runtime `Cargo.toml` references `opentensor/polkadot-sdk.git` at rev `71629fd`. 7 suppressed RUSTSECs from inherited Substrate stack.
  Owns: Research document assessing fork divergence and migration path.
  Integration touchpoints: All chain crates, FRAME pallets, consensus.
  Scope boundary: Research only. Count divergent commits, identify blocking changes, estimate effort.
  Acceptance criteria: (1) Fork divergence is quantified (commit count, file count). (2) Blocking changes are identified. (3) Go/no-go recommendation with rationale.
  Verification: Review-based.
  Required tests: None (research task).
  Dependencies: None (independent research).
  Estimated scope: M
  Completion signal: Research document exists with go/no-go recommendation.

---

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
