# IMPLEMENTATION_PLAN

Generated: 2026-04-11
Codebase snapshot: trunk @ 4e0b37fbaa + local
Specs: gen-20260411-205202/specs/110426-*.md

---

## Priority Work

- [ ] `DOSSIER-002` Liar's Dice checkpoint dossier with exact exploitability

  Spec: `specs/110426-game-solver-core.md`
  Why now: Promoting `liars-dice` to `promotable_local` requires a checkpoint dossier with exact exploitability evidence. Liar's Dice already has `exact_exploitability()` on the solver, so the benchmark surface exists. What's missing is the dossier wrapper that records provenance and benchmark results in a format the promotion ledger can consume.
  Codebase evidence: `crates/myosu-games-liars-dice/src/solver.rs` implements `exact_exploitability()` on `LiarsDiceSolver<N>`. `crates/myosu-miner/src/training.rs` uses `train_select_best()` which returns `LiarsDiceTrainingSummary` with `selected_exploitability`. Checkpoint format uses `MYOS` magic + v1 + bincode. `LIARS_DICE_SOLVER_TREES = 1 << 10` in training.rs.
  Owns: `LiarsDiceArtifactDossier` and `LiarsDiceBenchmarkDossier` types in `crates/myosu-games-liars-dice/src/` (new module or extension to solver.rs).
  Integration touchpoints: `crates/myosu-games-liars-dice/src/solver.rs` (exploit API), `crates/myosu-games-canonical/src/policy.rs` (CanonicalPolicyBenchmarkSummary).
  Scope boundary: Dossier types and builder that captures: checkpoint hash, tree count, training epochs, exact exploitability value, pass/fail against threshold. Do NOT change the solver or training pipeline. Do NOT change checkpoint format. Plan 006 says Liar's Dice uses exact exploitability (not scenario pack).
  Acceptance criteria: (1) Dossier type exists with checkpoint_hash, tree_count, epochs, exploitability, threshold, passing fields. (2) Building a dossier from a zero-iteration solver reports high exploitability and `passing: false`. (3) Building a dossier from a trained solver with low exploitability reports `passing: true`. (4) Dossier serializes to JSON for `outputs/solver-promotion/liars-dice/`.
  Verification: `SKIP_WASM_BUILD=1 cargo test -p myosu-games-liars-dice --quiet`; `SKIP_WASM_BUILD=1 cargo clippy -p myosu-games-liars-dice -- -D warnings`.
  Required tests: (a) Zero-iteration dossier fails threshold. (b) Trained solver dossier passes threshold. (c) Dossier JSON serialization roundtrip.
  Dependencies: POLICY-001 (CanonicalPolicyBenchmarkSummary type).
  Estimated scope: S
  Completion signal: Dossier types exist. Tests pass. Clippy clean.

### Checkpoint: Dossier infrastructure verified

After DOSSIER-001, DOSSIER-002: both dedicated games have dossier types and negative/positive test fixtures. Verify: `SKIP_WASM_BUILD=1 cargo test -p myosu-games-poker -p myosu-games-liars-dice --quiet` passes, `bash tests/e2e/research_strength_harness.sh` still passes (no regressions). Re-evaluate whether the promotion tasks should proceed or if the dossier contracts need adjustment.

---

- [ ] `PROMOTE-001` Promote nlhe-heads-up to promotable_local

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: This is milestone 4 of the master plan, but it is a decision-gated integration proof, not evidence that a strong NLHE artifact already exists. NLHE is the intended first game to cross the promotion bar only after the policy bundle contract, dossier infrastructure, promotion ledger, and a real pinned artifact all verify together.
  Codebase evidence: `crates/myosu-games-poker/src/artifacts.rs` provides the artifact bundle infrastructure. `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs` provides the independent benchmark surface. `ops/solver_promotion.yaml` (after PROMO-001) lists `nlhe-heads-up` at `benchmarked`. The sparse bootstrap artifacts remain negative fixtures; promotion requires a pinned stronger artifact referenced by hash.
  Owns: `outputs/solver-promotion/nlhe-heads-up/` directory with `bundle.json`, `benchmark-summary.json`, `artifact-manifest.json`. Update `ops/solver_promotion.yaml` to `tier: promotable_local` for `nlhe-heads-up`. Code path in `myosu-games-canonical` that can emit a verified policy bundle for a labeled heads-up decision point.
  Integration touchpoints: `crates/myosu-games-canonical/src/policy.rs` (CanonicalPolicyBundle construction), `crates/myosu-games-poker/src/artifacts.rs` (NlheArtifactDossier), `ops/solver_promotion.yaml` (tier update), `tests/e2e/promotion_manifest.sh` (must still pass after tier change).
  Scope boundary: Emit a policy bundle from a pinned NLHE artifact, verify it, produce promotion outputs. The strong artifact itself lives outside the repo (referenced by hash). Do NOT require training in this task. Do NOT modify the miner or validator. The task must fail closed if the artifact hash, manifest, or benchmark dossier is absent or below threshold; do not produce a placeholder promotable bundle.
  Acceptance criteria: (1) `outputs/solver-promotion/nlhe-heads-up/bundle.json` exists and is valid JSON. (2) `verify_policy_bundle()` succeeds on the emitted bundle. (3) `sample_policy_action()` succeeds with test entropy. (4) `ops/solver_promotion.yaml` shows `tier: promotable_local` for `nlhe-heads-up`. (5) `bash tests/e2e/promotion_manifest.sh` passes after the tier change (code support exists). (6) Sparse bootstrap artifacts are rejected as promotion inputs (negative test). (7) A missing or mismatched external artifact hash prevents bundle emission and leaves the ledger below `promotable_local`.
  Verification: `test -f outputs/solver-promotion/nlhe-heads-up/bundle.json && echo EXISTS`; `bash tests/e2e/promotion_manifest.sh`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-poker --quiet`.
  Required tests: (a) Policy bundle construction from pinned artifact succeeds. (b) Verify + sample roundtrip succeeds. (c) Sparse artifact rejection as promotion input. (d) Missing/mismatched external artifact hash rejects promotion. (e) Promotion manifest harness passes with updated YAML.
  Dependencies: DOSSIER-001, PROMO-001, PROMO-002.
  Estimated scope: M
  Completion signal: `nlhe-heads-up` at `promotable_local` in YAML with verified bundle under `outputs/`.

- [ ] `PROMOTE-002` Promote liars-dice to promotable_local

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: This is the second dedicated game promotion, validating that the promotion pipeline works for a different solver type (exact exploitability vs scenario pack). Completing both dedicated games before any portfolio game is the master plan's stated ordering.
  Codebase evidence: `crates/myosu-games-liars-dice/src/solver.rs` provides `exact_exploitability()`. `LIARS_DICE_SOLVER_TREES = 1 << 10`. Plan 006 says Liar's Dice promotion uses exact exploitability (not scenario pack), tree size `1 << 10`.
  Owns: `outputs/solver-promotion/liars-dice/` directory with `bundle.json`, `benchmark-summary.json`, `artifact-manifest.json`. Update `ops/solver_promotion.yaml` to `tier: promotable_local` for `liars-dice`. Code path to emit and verify a Liar's Dice policy bundle.
  Integration touchpoints: `crates/myosu-games-canonical/src/policy.rs`, `crates/myosu-games-liars-dice/`, `ops/solver_promotion.yaml`, `tests/e2e/promotion_manifest.sh`.
  Scope boundary: Emit, verify, and sample a Liar's Dice policy bundle using an in-repo-trainable checkpoint. Unlike NLHE, Liar's Dice can be fully trained within the repo (tree size 1024 is manageable). Do NOT change the solver or training pipeline.
  Acceptance criteria: (1) `outputs/solver-promotion/liars-dice/bundle.json` exists and is valid JSON. (2) `verify_policy_bundle()` succeeds. (3) `sample_policy_action()` succeeds with test entropy. (4) `ops/solver_promotion.yaml` shows `tier: promotable_local`. (5) `bash tests/e2e/promotion_manifest.sh` passes. (6) Benchmark dossier shows `passing: true` with trained checkpoint.
  Verification: `test -f outputs/solver-promotion/liars-dice/bundle.json && echo EXISTS`; `bash tests/e2e/promotion_manifest.sh`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-liars-dice --quiet`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-canonical --quiet`.
  Required tests: (a) Policy bundle from trained checkpoint succeeds. (b) Verify + sample roundtrip. (c) Promotion manifest harness passes. (d) Zero-iteration checkpoint rejected as promotion input.
  Dependencies: DOSSIER-002, PROMO-001, PROMO-002.
  Estimated scope: S
  Completion signal: `liars-dice` at `promotable_local` in YAML with verified bundle under `outputs/`.

### Checkpoint: Two dedicated games promoted

After PROMOTE-001, PROMOTE-002: both dedicated games are at `promotable_local` with verified bundles and benchmark dossiers. This is the master plan's milestone 4 exit gate. Verify: `bash tests/e2e/promotion_manifest.sh` passes, `bash tests/e2e/research_strength_harness.sh` passes, `bash tests/e2e/canonical_ten_play_harness.sh` passes, `SKIP_WASM_BUILD=1 cargo test --workspace --quiet` passes. Re-evaluate before proceeding to portfolio game work or Bitino integration.

---

- [ ] `SEC-001` Triage all 19 advisory suppressions

  Spec: `specs/110426-security-posture.md`
  Why now: The CI advisory allowlist has grown to 19 entries (up from 7 in the 2026-04-03 review). Plan 008 proposes triage. Each advisory should have an explicit classification (remediate, accept, defer) with documented rationale. This is independent of promotion work and can run in parallel.
  Codebase evidence: `.github/workflows/ci.yml:358-376` lists 19 `--ignore` entries. AGENTS.md SEC-001 section documents the allowlist policy. The spec categorizes advisories as: directly owned (bincode 1.3.3, RUSTSEC-2025-0141), inherited chain (from opentensor polkadot-sdk fork), and workspace (paste, lru).
  Owns: Classification document or inline comments in CI workflow. Each advisory gets: classification, rationale, remediation plan or acceptance justification.
  Integration touchpoints: `.github/workflows/ci.yml` (allowlist comments), `WORKLIST.md` (SEC-001 entry update).
  Scope boundary: Classify and document all 19 advisories. Reduce the allowlist where feasible (remove advisories that have been fixed upstream). Do NOT migrate bincode (that is SEC-002). Do NOT rebase the polkadot-sdk fork.
  Acceptance criteria: (1) Every advisory in the CI allowlist has a documented classification (remediate/accept/defer) with justification. (2) `cargo audit -D warnings` with the updated allowlist passes. (3) Any advisory whose upstream crate has been patched is removed from the allowlist. (4) Inherited chain advisories are documented as "no direct Myosu usage" with per-advisory rationale.
  Verification: Run `cargo audit -D warnings` with the updated allowlist and verify exit 0. Count remaining ignores vs current 19.
  Required tests: None (documentation and CI config task).
  Dependencies: None (parallel with promotion work).
  Estimated scope: S
  Completion signal: All 19 advisories classified. Allowlist reduced where feasible. CI green.

- [ ] `SEC-002` Bincode 1.3.3 migration decision

  Spec: `specs/110426-security-posture.md`
  Why now: RUSTSEC-2025-0141 is the only directly owned advisory in the allowlist. It affects wire paths in all three dedicated solver crates plus checkpoint/artifact paths in poker and Liar's Dice. The decision (migrate to bincode 2.x/postcard, or accept with documented rationale) must be made before any future payload-bearing checkpoint format changes.
  Codebase evidence: `crates/myosu-games-poker/src/solver.rs:7,20-21` uses bincode for payload-bearing poker checkpoints. `crates/myosu-games-liars-dice/src/solver.rs:6,21-22` uses the same `"MYOS"` + version + bincode checkpoint pattern. `crates/myosu-games-kuhn/src/solver.rs:10-12` uses a distinct `"MYOK"` + version exact-solver checkpoint without a bincode payload, while `crates/myosu-games-kuhn/src/wire.rs:1` uses bincode for wire serialization. Poker artifacts also use bincode in `crates/myosu-games-poker/src/artifacts.rs:5`.
  Owns: Decision document (ADR or inline in WORKLIST.md) evaluating: bincode 2.x migration, postcard migration, or acceptance with rationale. Must include: blast radius assessment, checkpoint compatibility strategy, robopoker fork impact.
  Integration touchpoints: `crates/myosu-games-poker/src/solver.rs`, `crates/myosu-games-poker/src/wire.rs`, `crates/myosu-games-poker/src/artifacts.rs`, `crates/myosu-games-kuhn/src/wire.rs`, `crates/myosu-games-liars-dice/src/solver.rs`, `crates/myosu-games-liars-dice/src/wire.rs`, robopoker fork dependency.
  Scope boundary: Research and decide only. Do NOT implement the migration. Do NOT change checkpoint format. Document the decision so a future worker can act on it.
  Acceptance criteria: (1) Decision is documented with explicit rationale. (2) Blast radius is assessed separately for wire formats, poker artifacts, poker checkpoints, Liar's Dice checkpoints, Kuhn exact-solver checkpoints, and robopoker fork impact. (3) If migration chosen: payload-bearing checkpoint versioning strategy is sketched (version 2 reader + version 1 fallback). (4) If acceptance chosen: risk mitigation documented (e.g., decode budget limits already in place at 1 MiB).
  Verification: Review-based. Document exists and addresses all criteria.
  Required tests: None (research task).
  Dependencies: None (parallel with promotion work).
  Estimated scope: S
  Completion signal: Decision documented with rationale and blast radius assessment.

- [ ] `DX-001` Consolidate critical operator caveats

  Spec: `specs/110426-developer-experience.md`
  Why now: The developer-experience spec documents that critical caveats (WASM cache requirement, sparse artifact limitations, devnet timing, Python dependency management, `SKIP_WASM_BUILD` requirement) are scattered across README.md, AGENTS.md, and OS.md instead of being concentrated. A new contributor must read three files to discover the fastest first-success path. Concentrating these reduces onboarding friction.
  Codebase evidence: `README.md:78-115` has operator commands. `AGENTS.md:303-346` has detailed caveats. `OS.md` has doctrine. The developer-experience spec identifies this scatter as a finding. `docs/operator-guide/quickstart.md` exists but focuses on the operator network path, not the developer contributor path.
  Owns: A consolidated quickstart section (either in `CONTRIBUTING.md` or an expanded `docs/developer-quickstart.md`) that covers: prerequisites, environment variables, fastest first-success commands, common pitfalls, and links to deeper docs.
  Integration touchpoints: `README.md` (add link to new doc), existing operator guide docs.
  Scope boundary: Consolidate existing scattered information. Do NOT create new documentation content beyond what already exists across the three source files. Do NOT change code behavior. Do NOT add JSON output mode (that is a separate task if needed).
  Acceptance criteria: (1) All critical caveats from AGENTS.md (WASM cache, sparse artifacts, devnet timing, SKIP_WASM_BUILD, wasm32v1-none target) appear in one place. (2) The document includes the 4-step fastest first-success path from the developer-experience spec. (3) README.md links to the consolidated document. (4) Environment variable inventory covers at least: SKIP_WASM_BUILD, MYOSU_KEY_PASSWORD, MYOSU_NODE_AUTHORITY_SURI.
  Verification: `test -f docs/developer-quickstart.md && echo EXISTS` (or CONTRIBUTING.md); verify links with `grep 'developer-quickstart\|CONTRIBUTING' README.md`.
  Required tests: None (documentation task).
  Dependencies: None (parallel with all other work).
  Estimated scope: S
  Completion signal: Consolidated document exists. README links to it.

---

## Follow-On Work

- [ ] `F-001` Research: Cribbage deepening to benchmarked tier

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: This is milestone 5 of the master plan. Cribbage is the default first portfolio game for promotion work. However, the canonical-truth-promotion spec notes as a hypothesis that "whether policy bundle generalization to portfolio games is feasible without dedicated solver crates is open." This task must validate that hypothesis before attempting promotion.
  Codebase evidence: `crates/myosu-games-portfolio/src/core/cribbage.rs` and `crates/myosu-games-portfolio/src/engines/cribbage.rs` exist. Portfolio engines are all `rule-aware` tier (not trained CFR). Plan 009 says cribbage deepening targets `benchmarked` (not `promotable_local`).
  Owns: Scenario pack and benchmark surface for cribbage. Update `ops/solver_promotion.yaml` tier from `routed` to `benchmarked`.
  Integration touchpoints: `crates/myosu-games-portfolio/`, `ops/solver_promotion.yaml`, `tests/e2e/promotion_manifest.sh`.
  Scope boundary: Deepen cribbage only. Do NOT attempt `promotable_local` for any portfolio game. Do NOT create a dedicated cribbage solver crate.
  Acceptance criteria: (1) Cribbage scenario pack with labeled states exists. (2) Benchmark surface reports engine quality metrics. (3) `ops/solver_promotion.yaml` shows `tier: benchmarked` for cribbage. (4) Promotion manifest harness passes.
  Verification: `bash tests/e2e/promotion_manifest.sh`; `SKIP_WASM_BUILD=1 cargo test -p myosu-games-portfolio --quiet`.
  Required tests: Cribbage scenario pack tests, benchmark metric assertions.
  Dependencies: PROMOTE-001, PROMOTE-002 (both dedicated games promoted first, per master plan ordering).
  Estimated scope: M
  Completion signal: Cribbage at `benchmarked` in YAML with scenario pack and benchmark evidence.

- [ ] `F-002` Token economics decision document

  Spec: `specs/110426-chain-runtime-pallet.md`
  Why now: Carries forward from prior plan F-003. The token-economics spec is a research spec identifying 8+ design axes that must be decided before `Stage0NoopSwap` can be replaced. ADR-008 exists (`docs/adr/008-future-token-economics-direction.md`) with `Status: Proposed` but still lacks the multi-contributor review required by the spec.
  Codebase evidence: `crates/myosu-chain/pallets/game-solver/src/swap/` contains the NoOpSwap implementation. `docs/adr/008-future-token-economics-direction.md` exists with `Status: Proposed`. No recorded multi-contributor signoff found in `docs/adr/README.md`, `ops/decision_log.md`, or `.github/`.
  Owns: Multi-contributor review completion and status update for ADR-008.
  Integration touchpoints: `docs/adr/008-future-token-economics-direction.md`, `ops/decision_log.md`.
  Scope boundary: Review and document only. Do NOT change swap implementation. Do NOT wire V3 AMM into runtime.
  Acceptance criteria: (1) ADR-008 reviewed by at least two contributors with token-economics context. (2) Review recorded in `docs/adr/README.md` or `ops/decision_log.md`. (3) ADR status updated from `Proposed` to `Accepted` or `Superseded`.
  Verification: Review-based. `grep 'Status:' docs/adr/008-future-token-economics-direction.md`.
  Required tests: None (research task).
  Dependencies: None, but blocked on external review (not a code task).
  Estimated scope: L
  Completion signal: Multi-contributor review recorded. ADR status updated.

- [ ] `F-003` Miner convergence gate research

  Spec: `specs/110426-operator-stack.md`
  Why now: Carries forward from prior plan F-007. No convergence gate exists — a miner can train for 1 iteration and serve garbage. Validators score it low, but operators have no guidance on minimum training. This task is still blocked on a truthful quality benchmark surface.
  Codebase evidence: Positive-iteration poker training rejects bootstrap artifacts where `postflop_complete = false` (`crates/myosu-games-poker/src/artifacts.rs`). The reference-pack benchmark exists at `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs` but is independent of the self-check validator path. `bash tests/e2e/research_strength_harness.sh` exercises it. Liar's Dice has `exact_exploitability()` which is a truthful quality metric.
  Owns: Research document with recommended minimum iterations per game type, using truthful benchmark surfaces (not same-checkpoint self-check).
  Integration touchpoints: Miner CLI documentation, operator guide.
  Scope boundary: Measure and document. Do NOT enforce in code.
  Acceptance criteria: (1) Minimum iteration guidance documented for Liar's Dice (using exact exploitability). (2) Poker convergence guidance documented or explicitly marked as blocked on richer encoder artifacts. (3) Quality thresholds documented per game type.
  Verification: Run miner with varying iteration counts against quality benchmarks. For Liar's Dice: `exact_exploitability()` at N iterations. For poker: benchmark_scenario_pack reference surface (if artifacts available).
  Required tests: None (research task).
  Dependencies: DOSSIER-001 (NLHE benchmark surface), DOSSIER-002 (Liar's Dice exploitability surface). Poker convergence additionally blocked on richer encoder artifacts.
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type with truthful quality evidence.

- [ ] `F-004` Bitino policy canonical crate (sibling repo)

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: Milestone 6 of the master plan. The Bitino-side adapter crate deserializes Myosu policy bundles and converts them to Bitino-local table state. This is the first cross-repo integration point.
  Codebase evidence: Master plan specifies `../bitino/crates/bitino-policy-canonical/` as the new crate. `../bitino/crates/bitino-wire/src/interactive.rs` defines `InteractivePresentation` (the rendering envelope). `../bitino/crates/bitino-engine/src/types.rs` defines `GameId`.
  Owns: `../bitino/crates/bitino-policy-canonical/` (sibling repo). New `GameId` values for solver-backed games.
  Integration touchpoints: Bitino Cargo.toml, bitino-engine types, bitino-play TUI, bitino-play agent state.
  Scope boundary: Local adapter only. Do NOT implement funded settlement. Do NOT require live Myosu chain connection. Bundle loading from local files only.
  Acceptance criteria: (1) `bitino-policy-canonical` crate compiles. (2) Can deserialize a Myosu `CanonicalPolicyBundle` from JSON. (3) Can verify the deserialized bundle. (4) New `GameId` values exist for solver-backed games. (5) `InteractivePresentation` can be constructed from a policy bundle.
  Verification: `cargo test -p bitino-policy-canonical --quiet` in sibling repo.
  Required tests: Deserialization, verification, presentation construction.
  Dependencies: PROMOTE-001 (needs at least one verified bundle to test against).
  Estimated scope: M
  Completion signal: Bitino can deserialize and render a Myosu policy bundle locally.

- [ ] `F-005` Bitino local table adapter and same-TUI pilot

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: Milestone 6 continued. The adapter connects the policy canonical crate to the Bitino TUI. The master plan says "the first Bitino proof should show one solver-backed heads-up hold'em table rendered through the normal Bitino TUI from a local pinned bundle."
  Codebase evidence: Master plan specifies `../bitino/crates/bitino-play/src/solver_tables.rs` with `SolverTableSession`, `load_solver_table()`, `presentation_from_policy_bundle()`.
  Owns: `solver_tables.rs` in bitino-play, TUI wiring, agent state catalog extension.
  Integration touchpoints: `bitino-play/src/tui/mod.rs`, `bitino-play/src/agent/state.rs`, `bitino-engine/src/types.rs`.
  Scope boundary: Offline local table only. No funded settlement. No live miner discovery. Bundle loaded from file path. Preserve the normal Bitino TUI information architecture: ready-room discovery, table rendering, session metadata, and clear invalid-bundle errors. Do not add a one-off solver UI that bypasses existing keyboard/headless flows.
  Acceptance criteria: (1) A solver-backed table appears in Bitino ready room. (2) Renders through normal Bitino TUI framework. (3) Session/round details expose bundle id, artifact hash, benchmark label. (4) Invalid, missing, or unverifiable policy bundles render actionable error states instead of panics or silent fallback. (5) Existing keyboard/headless flow still reaches the table. (6) Proof command: `cargo run -q -p bitino-play -- --headless 1 --game solver_holdem_heads_up --policy-bundle <path>`.
  Verification: Run the headless proof command in Bitino repo.
  Required tests: Headless table session, presentation rendering, session metadata, invalid-bundle error state.
  Dependencies: F-004 (policy canonical crate), PROMOTE-001 (bundle to test with).
  Estimated scope: L
  Completion signal: Solver-backed table visible in Bitino TUI from local bundle.

- [ ] `F-006` Funded integration (sibling repo)

  Spec: `specs/110426-canonical-truth-promotion.md`
  Why now: Milestone 7 of the master plan. Should NOT start early. Only once the offline same-TUI pilot (F-005) is stable. Adds policy-bundle loading to bitino-house, house-action sampling with Bitino fairness entropy, wire-level replay/provenance, and settlement integration.
  Codebase evidence: Master plan: "the realized action must be replayable from the saved fairness draw and the saved policy bundle hash."
  Owns: `../bitino/crates/bitino-house/` extensions for policy-bundle-backed rounds.
  Integration touchpoints: bitino-house, bitino-wire, bitino-settlement.
  Scope boundary: Funded rounds with replay proof. Requires stable offline pilot first.
  Acceptance criteria: (1) Funded solver-backed round logs bundle hash and fairness draw. (2) Replay can reproduce sampled action from saved data. (3) If verification fails, funded slice is incomplete.
  Verification: Replay proof in Bitino test suite.
  Required tests: Replay determinism, settlement accounting.
  Dependencies: F-005 (stable offline pilot).
  Estimated scope: L
  Completion signal: Funded solver-backed round with replayable action proof.

---

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
  Blocker (2026-04-05, re-verified 2026-04-12): The current stage-0 validator score is not a convergence metric. `score_response()` in `crates/myosu-validator/src/validation.rs` compares the observed miner response against `solver.answer(query)` from the checkpoint supplied on the validator CLI, and the repo-owned happy-path harnesses (`tests/e2e/local_loop.sh`, `tests/e2e/validator_determinism.sh`) pass the miner checkpoint straight into that validator path, so the truthful expected result is `exact_match=true` / `score=1.0` whenever the miner response came from the same checkpoint. Poker now has an independent reference-pack benchmark surface in `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs`, and `bash tests/e2e/research_strength_harness.sh` exercises it against the repo-owned 80-scenario checkpoint pack instead of self-scoring the candidate checkpoint. Positive-iteration poker training is still blocked, though, because the checked-in bootstrap artifact path (`crates/myosu-games-poker/examples/bootstrap_artifacts.rs`) remains postflop-sampled, and direct re-verification against the generated `target/e2e/*/poker/encoder/manifest.json` outputs shows the same shape (`preflop.entries = 169`, `flop.entries = 24`, `turn.entries = 24`, `river.entries = 24`, `complete_streets=preflop`, `sampled_streets=flop,turn,river`, `postflop_complete=false`). `cargo test -p myosu-validator --quiet exact_match_scores_one`, `cargo test -p myosu-miner --quiet run_poker_training_batch_rejects_incomplete_artifacts_before_training`, `cargo test -p myosu-games-poker --quiet artifacts::tests::bootstrap_encoder_streets_report_sampled_postflop_shape`, and `cargo test -p myosu-games-poker --quiet benchmark::tests::sparse_bootstrap_checkpoint_differs_from_reference_pack` confirm that the validator happy path is a self-check, the reference-pack benchmark is independent, and positive poker training iterations on the repo-owned sparse artifacts are still rejected cleanly before training begins. Until the repo has richer poker encoder artifacts or another trainable benchmark surface, this task still cannot truthfully document minimum training iterations. This task subsumes the deferred Nemesis follow-up `NEM-008`; keep one canonical queue entry here.
  Estimated scope: S
  Completion signal: Minimum iterations documented per game type.


## Completed / Already Satisfied

- [x] `C-001` NoOpSwap identity stub implements all 37 swap callsites
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `Stage0NoopSwap` with 1:1 conversion, zero fees in runtime. Verified by `cargo test -p pallet-game-solver coinbase --quiet`.

- [x] `C-002` INV-004 solver-gameplay dependency boundary enforced in CI
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: `cargo tree` check in `.github/workflows/ci.yml:145-158`.

- [x] `C-003` Multi-game architecture with zero-change extensibility
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `GameRegistry::supported()` returns 23 games. Poker, Kuhn, Liar's Dice, 20 portfolio games all implement solver traits independently. Adding Liar's Dice required zero poker changes.

- [x] `C-004` Workspace clippy lints enforced
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: Workspace `Cargo.toml` `[lints.clippy]` denies: `arithmetic-side-effects`, `expect-used`, `indexing-slicing`, `unwrap-used`. CI runs with `-D warnings`.

- [x] `C-005` Validator scoring with hyperbolic formula and determinism tests
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `score = 1.0 / (1.0 + l1_distance)` in `validation.rs`. 14+ unit tests. `validator_determinism.sh` in CI. INV-003 epsilon < 1e-6.

- [x] `C-006` Miner 7-step lifecycle
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `crates/myosu-miner/src/main.rs` implements: probe, register, serve_axon, train, strategy, http_axon. All steps have structured reports.

- [x] `C-007` Gameplay surface with three modes
  Spec: `specs/110426-gameplay-surface.md`
  Codebase evidence: `crates/myosu-play/src/main.rs` implements smoke-test, TUI (train), and pipe modes. CI runs both poker and kuhn smoke tests.

- [x] `C-008` Key management with create, import, export, switch
  Spec: `specs/110426-key-management.md`
  Codebase evidence: `crates/myosu-keys/src/lib.rs` and `src/storage.rs` implement: `generate_mnemonic()`, `mnemonic_to_pair()`, `save_pair()`, `load_active_pair()`, `import_keyfile()`, `export_active_keyfile()`, `set_active_account()`, `list_stored_accounts()`. XSalsa20-Poly1305 encryption with scrypt KDF.

- [x] `C-009` Two-node block sync proven
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/two_node_sync.sh` in CI. Proves named-devnet peer discovery with `MYOSU_NODE_AUTHORITY_SURI`.

- [x] `C-010` Aura + GRANDPA consensus with 4 chain spec variants
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: Runtime `construct_runtime!` includes `pallet_aura` and `pallet_grandpa`. Chain specs: `localnet`, `devnet`, `testnet`, `finney`.

- [x] `C-011` Epoch consistency guard
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `is_epoch_input_state_consistent(netuid)` check in `run_epoch.rs`. Two tests verify the guard.

- [x] `C-012` Zero-dividend fallback distributes by stake weight
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: Stake-weighted fallback in `run_coinbase.rs`. Test verifies 750/250 distribution for 3:1 stake.

- [x] `C-013` Decode budget hardened to 1 MiB
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `MAX_DECODE_BYTES` = 1 MiB in `solver.rs` and `wire.rs`. Tests verify oversized payloads rejected.

- [x] `C-014` Operator guide documentation exists
  Spec: `specs/110426-developer-experience.md`
  Codebase evidence: `docs/operator-guide/quickstart.md`, `upgrading.md`, `troubleshooting.md`, `architecture.md` all exist.

- [x] `C-015` Environment variable contracts documented
  Spec: `specs/110426-developer-experience.md`
  Codebase evidence: `MYOSU_KEY_PASSWORD`, `MYOSU_CONFIG_DIR`, `MYOSU_SUBNET`, `MYOSU_WORKDIR`, `MYOSU_CHAIN`, `MYOSU_OPERATOR_CHAIN` referenced in operator tooling and code.

- [x] `C-016` GameType on-chain encoding with proptest roundtrip
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `GameType::from_bytes` / `to_bytes` with `#[non_exhaustive]` enum. CI runs `serialization_roundtrip`.

- [x] `C-017` Structured report types for miner, validator, and key management
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: 6 report types each in miner and validator with machine-readable prefixes per bootstrap stage.

- [x] `C-018` 11 CI jobs covering workspace, chain, E2E, doctrine, audit, operator
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: `.github/workflows/ci.yml` defines: repo-shape, robopoker-fork-coherence, python-research-qa, active-crates, chain-core, integration-e2e, doctrine, dependency-audit, plan-quality, operator-network, chain-clippy.

- [x] `C-019` 7 E2E integration proofs pass in CI
  Spec: `specs/110426-ci-quality-gates.md`
  Codebase evidence: `tests/e2e/` contains: local_loop.sh, two_node_sync.sh, four_node_finality.sh, consensus_resilience.sh, cross_node_emission.sh, validator_determinism.sh, emission_flow.sh. All wired in `.github/workflows/ci.yml:296-315`.

- [x] `C-020` GitHub Actions pinned to full SHA with persist-credentials: false
  Spec: `specs/110426-security-posture.md`
  Codebase evidence: All 5 action references in `ci.yml` use full SHA hashes with version comments. All checkouts use `persist-credentials: false`. Permissions scoped to `contents: read`.

- [x] `C-021` Canonical manifest gate: 10 games, 10 snapshot=ok
  Spec: `specs/110426-canonical-truth-promotion.md`
  Codebase evidence: `CANONICAL_TEN` contains exactly 10 games. CI validates: `cargo run -p myosu-games-canonical --example canonical_manifest` produces 10 `CANONICAL_GAME` lines and 10 `snapshot=ok`. Playtrace tests pass.

- [x] `C-022` All 4 E2E play/research harnesses pass in CI
  Spec: `specs/110426-gameplay-surface.md`
  Codebase evidence: `canonical_ten_play_harness.sh`, `research_play_harness.sh`, `research_games_harness.sh`, `research_strength_harness.sh` all wired in `.github/workflows/ci.yml:136-143`.

- [x] `C-023` 23 game types in registry with Custom extensibility
  Spec: `specs/110426-game-solver-core.md`
  Codebase evidence: `GameType` enum has 23 named variants plus `Custom(String)` at `crates/myosu-games/src/traits.rs:66`. `GameRegistry::supported()` returns 23 descriptors at `crates/myosu-games/src/registry.rs:43`.

- [x] `C-024` Four-authority finality proof with 1-down resilience
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/four_node_finality.sh` starts 4 authorities, stops 1, asserts surviving 3 keep finalizing. Threshold math: 3/3 for 4-authority set.

- [x] `C-025` Emission accounting: sum(distributions) == block_emission * epochs
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/emission_flow.sh` proves emission accounting integrity. Dust within `TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA` (1000 rao).

- [x] `C-026` Consensus resilience: authority restart and catch-up
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/consensus_resilience.sh` stops authority-4, waits for 1-3 to finalize, restarts authority-4, requires all 4 agree on finalized head.

- [x] `C-027` Cross-node emission agreement
  Spec: `specs/110426-chain-runtime-pallet.md`
  Codebase evidence: `tests/e2e/cross_node_emission.sh` starts 4-authority devnet, drives registration/weights, snapshots emission maps at shared block-23 hash.

- [x] `C-028` KeyError enum with 15+ actionable variants
  Spec: `specs/110426-key-management.md`
  Codebase evidence: `crates/myosu-keys/src/lib.rs:22-91` defines: InvalidMnemonic, MissingHomeDir, MissingPasswordEnv, MissingKeySource, CreateDirectory, InvalidPath, ReadFile, WriteFile, InvalidConfig, SerializeKeyfile, DeserializeKeyfile, InvalidKeyfile, MissingKeyfile, InvalidKeyfileHex, KeyDerivation, EncryptSeed, DecryptSeed, InvalidSeedMaterial, SecretUriUnsupported.

- [x] `C-029` Miner rejects positive-iteration poker training on sparse artifacts
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `crates/myosu-games-poker/src/artifacts.rs` enforces `postflop_complete = false` rejection. `cargo test -p myosu-miner --quiet run_poker_training_batch_rejects_incomplete_artifacts_before_training` passes.

- [x] `C-030` Validator determinism across all three dedicated games
  Spec: `specs/110426-operator-stack.md`
  Codebase evidence: `tests/e2e/validator_determinism.sh` defaults to poker, liars-dice, and kuhn in one run. `cargo test -p myosu-validator --quiet inv_003_determinism` and `liars_dice_inv_003_determinism` pass.
