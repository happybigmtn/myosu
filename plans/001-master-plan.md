# Promote Myosu Solver Games into a Bitino-Compatible Canonical Policy Stack

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be maintained in accordance with it. This document supersedes the older single-issue planning batch archived under `archive/execplans_20260411/`. Those archived files remain useful historical context, but active planning now happens here so auto-corpus and future implementation runs have one master document to follow.

## Purpose / Big Picture

After this plan is complete, at least one Myosu solver-backed game can run inside the Bitino TUI through a truthful contract instead of ad hoc glue. “Truthful” here means four things at once. First, Myosu can produce a pinned policy bundle for a specific decision point, including public state, legal actions, a mixed strategy, and benchmark provenance. Second, Bitino can render that decision inside its existing `InteractivePresentation` shell without inventing a separate user interface. Third, the sampled house action is auditable even though it is non-deterministic: Myosu supplies the distribution, Bitino samples from that distribution using Bitino-controlled fairness entropy, and the replay surface can prove why the realized action was chosen. Fourth, each game is promoted only when it clears an explicit benchmark and provenance bar rather than because it merely routes through a smoke harness.

The first visible success is an offline Bitino table for `nlhe-heads-up` driven by a pinned Myosu artifact and rendered in the same Bitino TUI shell used for deterministic house games. The second is the same path for `liars-dice`. Only after those dedicated games work end to end should the portfolio-routed games start moving across the same seam.

## Progress

- [x] (2026-04-11 16:20Z) Reviewed the current Myosu solver, canonical, validator, and play surfaces plus the sibling Bitino canonical, wire, and TUI surfaces to frame the next phase around the code that actually exists.
- [x] (2026-04-11 16:35Z) Archived the previous `execplans/` batch under `archive/execplans_20260411/` and established `plans/` as the active planning root so future runs have one master entrypoint instead of four completed plans plus one active queue item.
- [x] (2026-04-11 16:45Z) Authored this master plan and repointed the top-level repo entrypoints to treat `plans/001-master-plan.md` as the active planning surface.
- [ ] Define the Myosu-side canonical policy bundle and verification contract in `crates/myosu-games-canonical`.
- [ ] Add a durable promotion manifest and benchmark gate surface for every research game in Myosu.
- [ ] Promote `nlhe-heads-up` to the first `promotable_local` game with pinned artifact provenance and a Bitino pilot bundle.
- [ ] Promote `liars-dice` to the same bar using its exact-exploitability surface.
- [ ] Choose the first portfolio game for serious promotion work, defaulting to `cribbage` unless new benchmark evidence makes another family materially easier.
- [ ] Add a Bitino-side canonical policy crate and local table adapter for solver-backed rounds in the sibling `../bitino/` repo.
- [ ] Prove the first same-TUI offline integration in Bitino before attempting any funded settlement or house-server integration.
- [ ] Add funded sampling, replay, and settlement proof only after the offline same-TUI path is stable and auditable.

## Surprises & Discoveries

- Observation: Myosu has already crossed the “can we represent and route these games?” threshold. The missing layer is solver credibility, not coverage.
  Evidence: `crates/myosu-games-portfolio/src/lib.rs` calls the portfolio engines “compact rule-aware reference engines”, while `archive/execplans_20260411/EXECPLAN_RESEARCH_SOLVER_STRENGTH_UPGRADE.md` ends with the open gap being deeper family engines plus the dedicated NLHE artifact path.

- Observation: Bitino’s canonical model is the wrong place to force solver-backed house policy without an intermediate abstraction.
  Evidence: `../bitino/crates/bitino-canonical/src/model.rs` defines `CanonicalWagerSpec`, `LiveContract`, and `CanonicalExposure`. Those types are about deterministic wager identity and settlement exposure, not mixed strategy provenance or sampled house action proofs.

- Observation: The two repos already meet cleanly at the presentation boundary.
  Evidence: `crates/myosu-play/src/blueprint.rs` already exposes game-specific renderer surfaces from pinned artifacts or demos, while `../bitino/crates/bitino-wire/src/interactive.rs` defines a stable `InteractivePresentation` envelope that the Bitino TUI renders.

- Observation: The validator happy path is still not a solver-strength proof.
  Evidence: `crates/myosu-validator/src/validation.rs` scores a response against the same checkpoint-loaded solver expectation. `IMPLEMENTATION_PLAN.md` still calls out the dedicated NLHE benchmark and the sampled postflop artifact gap as the real open convergence issue.

- Observation: The first same-TUI pilot does not need live Myosu miner discovery or funded Bitino settlement on day one.
  Evidence: `crates/myosu-play/src/cli.rs` still limits miner discovery to poker, and Bitino’s TUI already accepts local `InteractivePresentation` payloads in non-funded test paths. The clean lowest-risk path is a pinned local bundle, not immediate cross-system live orchestration.

## Decision Log

- Decision: `plans/` is the new active planning root, and the old `execplans/` batch is historical only.
  Rationale: The previous planning surface had completed canonical/core/harness plans plus one still-active solver-strength document. Future work is broader than that queue: it spans solver quality, canonical policy truth, artifact provenance, and Bitino integration. One master plan is easier for auto-corpus and future implementers to follow.
  Date/Author: 2026-04-11 / Codex

- Decision: The next execution stream is promotion-driven rather than corpus-driven.
  Rationale: Routing all 22 research games was the right bootstrap milestone, but it is no longer the main problem. The right next question is which games are strong enough to expose as solver-backed house policies, and what proof bar they must clear before that happens.
  Date/Author: 2026-04-11 / Codex

- Decision: The first Bitino pilot is offline and bundle-backed, not funded and not live-miner-backed.
  Rationale: This isolates the same-TUI and canonical-policy problems from chain liveness, wallet/channel settlement, and miner discovery. The first proof should show that Bitino can truthfully host a solver-backed table at all; funding and live sourcing come later.
  Date/Author: 2026-04-11 / Codex

- Decision: Myosu remains the source of strategy truth, but Bitino owns the final action sampling when the game is hosted inside Bitino.
  Rationale: Myosu should return a pinned mixed strategy plus provenance. Bitino already has the fairness and replay story for funded rounds. Reusing Bitino-controlled entropy prevents a silent split-brain where the external solver both defines and samples the house action.
  Date/Author: 2026-04-11 / Codex

- Decision: The Myosu-side policy types live in `crates/myosu-games-canonical` rather than in `myosu-play`.
  Rationale: `myosu-games-canonical` already owns canonical state snapshots, transition traces, and replay truth. Policy bundles are part of that same truth layer. `myosu-play` is only a consumer surface and should not become the authoritative home for solver provenance formats.
  Date/Author: 2026-04-11 / Codex

- Decision: Bitino gets a new sibling crate, `../bitino/crates/bitino-policy-canonical`, instead of forcing solver-backed round types into `bitino-canonical`.
  Rationale: `bitino-canonical` already has a sharp contract: canonical wager identity, exposure, and presentation for deterministic casino rounds. Solver-backed policy rounds are adjacent but different. A new crate keeps the boundary honest while still allowing the TUI to consume the same `InteractivePresentation` model.
  Date/Author: 2026-04-11 / Codex

- Decision: The first two promotion targets are `nlhe-heads-up` and `liars-dice`, and the first portfolio default target is `cribbage`.
  Rationale: The dedicated games already have the strongest benchmark surfaces in the repo. `cribbage` is the most practical first portfolio promotion because it is two-player, compact, heavily scored, and easier to audit than the large hidden-information families.
  Date/Author: 2026-04-11 / Codex

- Decision: Repo-owned artifacts are allowed to remain “proof bundles” as long as promoted games use pinned promotion dossiers that can point at stronger external artifacts.
  Rationale: Waiting for a fully checked-in trainable heads-up NLHE postflop abstraction would block useful progress. The real requirement is pinned artifact identity plus reproducible benchmark evidence, not that every strong artifact must live inside this repo.
  Date/Author: 2026-04-11 / Codex

## Outcomes & Retrospective

This planning pass did not change solver behavior. It changed the supervisory surface. The old `execplans/` batch is preserved in `archive/execplans_20260411/`, and the repo now has one active master plan that treats the next phase as promotion and integration work rather than another generic “add more games” cycle.

The key conclusion from the review is that Myosu is structurally ahead of its solver bar. The repo can represent, replay, and render far more games than it can currently promote into a truthful shared-house surface. That is the right place to focus next.

## Context and Orientation

The relevant Myosu files are these. `crates/myosu-games-portfolio/src/state.rs` derives typed portfolio challenges from bounded core state. `crates/myosu-games-portfolio/src/quality.rs` already carries a small quality-report surface for portfolio engines. `crates/myosu-games-poker/src/artifacts.rs` and `crates/myosu-games-poker/src/benchmark.rs` describe the current dedicated NLHE artifact and independent benchmark surfaces. `crates/myosu-validator/src/validation.rs` is the current validator scoring path and explains why the happy path is a same-checkpoint determinism proof, not a convergence proof. `crates/myosu-play/src/blueprint.rs` is the current local renderer/advice surface. `crates/myosu-games-canonical/src/playtrace.rs` is the canonical replay layer that already turns bounded state and local strategy choice into canonical traces.

The relevant Bitino files are in the sibling repo. `../bitino/crates/bitino-canonical/src/model.rs` defines the existing canonical wager truth model. `../bitino/crates/bitino-wire/src/interactive.rs` defines `InteractivePresentation`, the board/stream envelope already consumed by the Bitino TUI. `../bitino/crates/bitino-play/src/tui/mod.rs` renders those presentations. `../bitino/crates/bitino-play/src/agent/state.rs` is the agent-facing runtime state machine that advertises game catalog and session state. `../bitino/crates/bitino-engine/src/types.rs` defines `GameId`, which will need explicit new ids for solver-backed tables instead of overloading deterministic ones.

This plan uses three terms of art. A “policy bundle” is a self-contained artifact that describes one solver-backed decision point: public state, legal actions, probability distribution, solver provenance, and benchmark evidence. A “promotion bar” is the minimum benchmark, artifact, and replay evidence a game must have before it can be exposed inside Bitino. A “same-TUI pilot” is a Bitino table rendered in the existing Bitino TUI that consumes a Myosu policy bundle locally without yet involving funded settlement or live Myosu chain discovery.

## Plan of Work

The first milestone is to define a canonical policy contract on the Myosu side. Add `crates/myosu-games-canonical/src/policy.rs` and export it from `crates/myosu-games-canonical/src/lib.rs`. This module should not try to solve games. It should define the durable truth envelope for a promoted solver decision. The envelope must carry the game id, a decision id, the canonical public-state snapshot hash, the legal action ids, the returned mixed strategy, the recommended action, the artifact or checkpoint identity, the benchmark summary, and a deterministic bundle hash. It also needs a replayable sampling-proof type that records how a fairness draw is mapped onto the distribution so Bitino can later prove why a non-deterministic house action occurred.

The second milestone is to add promotion metadata and gates to Myosu. Create `ops/solver_promotion.yaml` as the durable promotion ledger. Each research game gets one entry that names its route (`dedicated` or `portfolio`), current tier (`routed`, `benchmarked`, `promotable_local`, or `promotable_funded`), benchmark surface, artifact requirement, and rollout target. Then add `crates/myosu-games-canonical/examples/promotion_manifest.rs` to print the current ledger plus observed bundle support from code, and add `tests/e2e/promotion_manifest.sh` to make that machine-checked. Do not let this manifest become a wish list. It must be derived from shipped surfaces and fail when a declared promotion tier is not actually supported by code and proofs.

The third milestone is to separate “proof bundles” from “promotion dossiers” for the dedicated games. In `crates/myosu-games-poker/src/artifacts.rs`, keep the current checked-in artifact shape as a small proof bundle, but add a parallel manifest or dossier reader that can point at stronger external artifact directories by hash. `crates/myosu-games-poker/src/benchmark.rs` should then benchmark that stronger artifact and write a benchmark summary that the promotion ledger can consume. Do the same for Liar’s Dice in `crates/myosu-games-liars-dice`. The important rule is that a promoted game must have pinned artifact identity plus benchmark evidence, even if the strong artifact itself lives outside the repo. The repo-owned examples remain useful, but they are not the promotion bar by themselves.

The fourth milestone is to promote `nlhe-heads-up` and `liars-dice` to `promotable_local`. Start with `nlhe-heads-up`. The Myosu side must be able to emit a policy bundle for a labeled heads-up decision point, verify the bundle, and deterministically sample the house action from supplied entropy. Then do the same for `liars-dice`, using exact exploitability and selected-checkpoint data as the benchmark provenance. At the end of this milestone, `ops/solver_promotion.yaml` should mark both games `promotable_local`, and the new promotion harness should fail if those bundle and benchmark surfaces regress.

The fifth milestone is to choose and deepen the first non-dedicated game. Default to `cribbage`. Add a real scenario pack and benchmark surface for `cribbage` inside `crates/myosu-games-portfolio`, with explicit labeled states rather than only bootstrap demos. The existing typed challenge, engine, and core adapter work in `crates/myosu-games-portfolio/src/core/cribbage.rs`, `src/engines/cribbage.rs`, and `src/state.rs` should become the base for a real promotion dossier, not just a smoke route. If later evidence makes another two-player family clearly easier, record that decision in this plan and switch once, explicitly.

The sixth milestone is the Bitino local adapter. In the sibling repo, add `../bitino/crates/bitino-policy-canonical/` and wire it into `../bitino/Cargo.toml`. This crate should deserialize the Myosu policy bundle, verify it, and convert it into the Bitino-side local model needed to render a solver-backed table. Extend `../bitino/crates/bitino-engine/src/types.rs` with new `GameId` values such as `SolverHoldemHeadsUp`, `SolverLiarsDice`, and later `SolverCribbage`; do not reuse `HoldemHeadsUp` because that would silently conflate deterministic and solver-backed semantics. Then add a local bundle-backed table adapter in `../bitino/crates/bitino-play/src/solver_tables.rs` and wire that into `../bitino/crates/bitino-play/src/agent/state.rs` and `../bitino/crates/bitino-play/src/tui/mod.rs`. The first Bitino proof should show one solver-backed heads-up hold’em table rendered through the normal Bitino TUI from a local pinned bundle, with the policy provenance visible in the session state or round metadata.

The seventh milestone is funded integration, and it should not start early. Only once the offline same-TUI pilot is stable should the work move into `../bitino/crates/bitino-house`. That slice should add policy-bundle loading, house-action sampling using Bitino fairness entropy, wire-level replay or provenance fields, and settlement integration. The realized action must be replayable from the saved fairness draw and the saved policy bundle hash. If a funded path cannot prove the sampled action after the fact, it is not ready.

## Concrete Steps

Work from the Myosu repository root for Myosu changes:

    cargo test -p myosu-games-canonical --quiet
    cargo test -p myosu-games-poker --quiet
    cargo test -p myosu-games-liars-dice --quiet
    bash tests/e2e/research_strength_harness.sh
    bash tests/e2e/research_games_harness.sh

After the policy bundle and promotion manifest surfaces exist, add and run:

    cargo run -p myosu-games-canonical --example promotion_manifest -- --format table
    bash tests/e2e/promotion_manifest.sh

The expected output should contain one row per research game and should make it obvious which games are only routed, which are benchmarked, and which are promoted. A successful heads-up NLHE row should eventually look like this in substance, even if the exact formatting changes:

    PROMOTION game=nlhe-heads-up route=dedicated tier=promotable_local artifact=external-pinned benchmark=reference-pack replay_bundle=true bitino_ready=true

When the first Bitino local adapter is ready, work from the sibling Bitino repo:

    cd ../bitino
    cargo test -p bitino-play --features tui --quiet
    cargo test -p bitino-policy-canonical --quiet

Add one targeted smoke command for the solver-backed local table path. The exact executable shape may differ, but the final proof must look like this in spirit:

    cargo run -q -p bitino-play -- --headless 1 --game solver_holdem_heads_up --policy-bundle ../myosu/outputs/solver-promotion/nlhe-heads-up/bundle.json

The result must show one solver-backed table round rendered through the existing Bitino shell, not a separate demo program.

## Validation and Acceptance

Acceptance for the Myosu-side policy layer is behavioral. A policy bundle built from a promoted game must verify locally, must refuse malformed distributions, and must sample the same action every time the same entropy bytes are supplied. Add tests in `crates/myosu-games-canonical` proving those invariants.

Acceptance for the promotion ledger is also behavioral. `ops/solver_promotion.yaml` plus `promotion_manifest.rs` must not allow a game to claim `promotable_local` unless all of the following are true: the game has an independent benchmark surface, the benchmark summary is present and passing, the artifact identity is pinned, and the policy bundle surface exists. The shell harness should fail if any declared promoted game is missing one of those proof surfaces.

Acceptance for the dedicated games means `nlhe-heads-up` and `liars-dice` can both emit verified policy bundles with non-demo provenance. For heads-up NLHE, that means a pinned stronger artifact and benchmark dossier, not just the repo’s sampled bootstrap encoder. For Liar’s Dice, that means a checkpoint with exact exploitability evidence and a selected-checkpoint summary.

Acceptance for the Bitino pilot is simple to observe. A solver-backed table must appear in the normal Bitino ready room and render through the same TUI framework already used by deterministic games. The session or round details must expose the bundle id, artifact hash, benchmark label, and sampled-action proof summary. The pilot is not complete if it only works in a one-off test harness outside the Bitino TUI.

Acceptance for funded integration comes last. A funded solver-backed round must log enough information for replay to recompute the sampled house action from the saved bundle hash and fairness draw. If the verify path cannot reproduce the action, the funded slice is incomplete.

## Idempotence and Recovery

This master plan should be updated in place. Do not create a second active master plan for the same promotion stream unless the work truly forks into unrelated programs. When a decision changes, update `Progress`, `Decision Log`, and the relevant milestone text here instead of opening a fresh planning branch.

The strong-artifact workflow must be safe to rerun. Promotion dossiers should point at pinned artifacts by hash and should copy any generated benchmark summaries into `outputs/solver-promotion/<game>/` so reruns can verify what was promoted without mutating the original artifact directory. The repo-owned “proof bundles” in code examples should remain small and reproducible so CI can still run without the large external artifacts.

When moving into the sibling Bitino repo, keep the first integration local and additive. Do not block on house-server or payment-channel changes during the offline pilot. If the pilot design proves wrong, it should be possible to drop the new `bitino-policy-canonical` crate and local table adapter without touching funded settlement code.

## Artifacts and Notes

The previous planning batch is archived in these files:

    archive/execplans_20260411/EXECPLAN_ACTIVE_QUEUE.md
    archive/execplans_20260411/EXECPLAN_CANONICAL_GAME_TRUTH_LAYER.md
    archive/execplans_20260411/EXECPLAN_CANONICAL_TEN_CORE_GAME_LOGIC.md
    archive/execplans_20260411/EXECPLAN_CANONICAL_TEN_PLAY_HARNESS.md
    archive/execplans_20260411/EXECPLAN_RESEARCH_SOLVER_STRENGTH_UPGRADE.md

The important proof outputs created by this plan should land under:

    outputs/solver-promotion/nlhe-heads-up/
    outputs/solver-promotion/liars-dice/
    outputs/solver-promotion/cribbage/

Each promoted game directory should eventually contain at least:

    bundle.json
    benchmark-summary.json
    artifact-manifest.json
    replay-proof-sample.json

Keep those outputs small and human-readable. The full solver checkpoint or abstraction artifact does not need to be copied into `outputs/`; only the pinned identifiers and proof summaries belong there.

## Interfaces and Dependencies

In `crates/myosu-games-canonical/src/policy.rs`, define the stable Myosu-side policy-truth types. Use these exact names unless a better local naming conflict is discovered and recorded in `Decision Log`:

    pub enum PolicyPromotionTier {
        Routed,
        Benchmarked,
        PromotableLocal,
        PromotableFunded,
    }

    pub struct CanonicalPolicyDistributionEntry {
        pub action_id: String,
        pub probability_ppm: u32,
    }

    pub struct CanonicalPolicyBenchmarkSummary {
        pub benchmark_id: String,
        pub metric_name: String,
        pub metric_value: f64,
        pub threshold: f64,
        pub passing: bool,
    }

    pub struct CanonicalPolicyProvenance {
        pub game_slug: String,
        pub solver_family: String,
        pub engine_tier: String,
        pub artifact_id: String,
        pub artifact_hash: String,
        pub benchmark: CanonicalPolicyBenchmarkSummary,
    }

    pub struct CanonicalPolicyBundle {
        pub game: ResearchGame,
        pub decision_id: String,
        pub public_state: CanonicalStateSnapshot,
        pub legal_action_ids: Vec<String>,
        pub distribution: Vec<CanonicalPolicyDistributionEntry>,
        pub recommended_action_id: String,
        pub provenance: CanonicalPolicyProvenance,
        pub bundle_hash: String,
    }

    pub struct CanonicalPolicySamplingProof {
        pub bundle_hash: String,
        pub entropy_source: String,
        pub entropy_hash: String,
        pub draw_u64: u64,
        pub sampled_action_id: String,
    }

    pub fn verify_policy_bundle(bundle: &CanonicalPolicyBundle) -> Result<(), CanonicalTruthError>;
    pub fn sample_policy_action(
        bundle: &CanonicalPolicyBundle,
        entropy_source: &str,
        entropy_bytes: &[u8],
    ) -> Result<CanonicalPolicySamplingProof, CanonicalTruthError>;

In `ops/solver_promotion.yaml`, store one entry per research game with these required fields: `route`, `tier`, `benchmark_surface`, `benchmark_threshold`, `artifact_requirement`, `bundle_support`, `bitino_target_phase`, and `notes`.

In `crates/myosu-games-canonical/examples/promotion_manifest.rs`, expose a machine-readable and table-readable manifest printer that joins the static ledger with live code-reported bundle support.

In the sibling Bitino repo, add `../bitino/crates/bitino-policy-canonical/src/model.rs` and define the bundle adapter types that deserialize `CanonicalPolicyBundle` and turn it into local table state. Then add `../bitino/crates/bitino-play/src/solver_tables.rs` with:

    pub struct SolverTableSession { ... }
    pub fn load_solver_table(bundle_path: &Path) -> Result<SolverTableSession, ...>;
    pub fn presentation_from_policy_bundle(
        session: &SolverTableSession,
    ) -> InteractivePresentation;

In `../bitino/crates/bitino-engine/src/types.rs`, add explicit new `GameId` values for solver-backed games rather than reusing deterministic ids. Start with `SolverHoldemHeadsUp`, `SolverLiarsDice`, and `SolverCribbage`.

At the end of the first local integration milestone, the dependency direction should still be simple: Myosu produces policy bundles and promotion outputs; Bitino consumes them. There should be no requirement for Bitino to link Myosu crates directly across repository boundaries.

Revision note: created on 2026-04-11 after the repo-level review concluded that Myosu had reached broad routing coverage but still lacked a promotion-driven master plan for solver quality, policy truth, and Bitino integration. This document replaces the earlier issue-specific `execplans/` queue as the active planning surface.
