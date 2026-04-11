# Promote NLHE Heads-Up to Promotable Local

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds. This document must be maintained in
accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, `nlhe-heads-up` is the first research-game entry
that can honestly claim `promotable_local`. That claim means four things are
all true at once:

1. Myosu can emit a verified policy bundle for a labeled NLHE decision point.
2. The bundle provenance points at a pinned artifact dossier and a benchmark
   dossier that cleared the declared promotion bar.
3. The promotion ledger and manifest gate recognize the game as promoted
   because the proof surfaces exist, not because the YAML says so.
4. The checked-in sparse bootstrap artifacts remain available only as negative
   fixtures and zero-iteration proof bundles, not as promotion inputs.

## Requirements Trace

- R1: `ops/solver_promotion.yaml` marks `nlhe-heads-up` as
  `tier: promotable_local`
- R2: A verified NLHE policy bundle can be built from a solver checkpoint plus
  pinned artifact/benchmark dossiers
- R3: The promotion gate rejects sparse bootstrap artifacts as promotion inputs
- R4: `bash tests/e2e/promotion_manifest.sh` passes with NLHE at
  `promotable_local`
- R5: An example or test constructs, verifies, and deterministically samples an
  NLHE policy bundle

## Scope Boundaries

This plan promotes only `nlhe-heads-up`. It does not promote Liar's Dice, does
not deepen cribbage, and does not implement any downstream same-TUI consumer.
It does not change the current validator scoring path. Its job is to make NLHE
promotion evidence truthful despite the repo-owned sparse artifacts remaining
non-trainable.

## Progress

- [ ] Add an NLHE policy-bundle builder in `crates/myosu-games-poker/src/`
- [ ] Accept pinned artifact and benchmark dossiers as inputs to the builder
- [ ] Add a negative test proving sparse bootstrap artifacts fail the promotion gate
- [ ] Add a positive bundle build/verify/sample test using a promotion-eligible dossier fixture
- [ ] Add an example binary that writes a verified NLHE bundle JSON
- [ ] Update `ops/solver_promotion.yaml` to mark NLHE `promotable_local`
- [ ] Verify the manifest gate recognizes the promotion and still rejects sparse-only claims

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: The checked-in sparse bootstrap artifacts remain negative fixtures
  and smoke-proof inputs, not promotion inputs.
  Rationale: The repo itself enforces that those artifacts are not a truthful
  positive-iteration training surface. Promoting NLHE on top of them would make
  the ledger more optimistic than the code.
  Date/Author: 2026-04-11 / Codex review

- Decision: NLHE promotion depends on a pinned dossier path rather than on
  checking a large trainable artifact set into the repo.
  Rationale: The repo needs reproducible provenance more than it needs to vendor
  multi-gigabyte artifacts.
  Date/Author: 2026-04-11 / corpus direction

## Outcomes & Retrospective

None yet.

## Context and Orientation

This plan builds on:

- plan 001: canonical policy-bundle and sampling-proof types
- plan 002: promotion ledger and manifest gate
- plan 004: pinned artifact and benchmark dossier surfaces

Relevant repo files:

- `crates/myosu-games-poker/src/solver.rs`
- `crates/myosu-games-poker/src/artifacts.rs`
- `crates/myosu-games-poker/src/benchmark.rs`
- `crates/myosu-games-poker/examples/benchmark_scenario_pack.rs`

The key current-state fact is that sparse bootstrap artifacts are still useful
for proving local wiring, but they are not sufficient to justify
`promotable_local`.

## Plan of Work

1. Add an NLHE policy-bundle builder that takes a solver, artifact dossier,
   benchmark dossier, and decision label and returns a verified
   `CanonicalPolicyBundle`.
2. Encode the promotion gate in code so sparse/bootstrap-only dossiers fail the
   `promotable_local` bar.
3. Add an example binary that writes bundle JSON to
   `outputs/solver-promotion/nlhe-heads-up/`.
4. Update the ledger and manifest gate once the builder and dossier checks are
   real.

## Implementation Units

### Unit 1: NLHE policy-bundle builder

Goal: Construct a verified bundle from solver + pinned dossiers.
Requirements advanced: R2, R5.
Dependencies: Plans 001 and 004.
Files to create or modify: `crates/myosu-games-poker/src/`, example under
`crates/myosu-games-poker/examples/`.
Tests to add or modify: Bundle build/verify/sample tests.
Approach: Reuse the existing solver-query path, convert probabilities into the
policy-bundle representation, and attach dossier-backed provenance.
Specific test scenarios:
- promotion-eligible dossier builds a valid bundle
- identical entropy yields identical sampled action
- malformed or mismatched dossier input is rejected

### Unit 2: Promotion gate and ledger update

Goal: Promote NLHE only when the gate is genuinely satisfied.
Requirements advanced: R1, R3, R4.
Dependencies: Unit 1 and plan 002.
Files to create or modify: `ops/solver_promotion.yaml`,
`crates/myosu-games-canonical/examples/promotion_manifest.rs`.
Tests to add or modify: Manifest gate coverage for promoted NLHE plus negative
fixture coverage for sparse artifacts.
Approach: Encode the dossier requirements in the manifest gate so a sparse-only
claim cannot pass.
Specific test scenarios:
- promoted NLHE entry passes with a promotion-eligible dossier
- sparse/bootstrap-only NLHE entry fails the promotion gate

## Concrete Steps

Use the repo-root sequence below for both the positive promotion case and the
negative sparse-artifact case.

## Verification

Run the commands below from the repo root. The negative sparse-artifact case is
part of the proof, not an edge case to ignore.

    cargo test -p myosu-games-poker --quiet policy_bundle
    cargo run -p myosu-games-poker --example nlhe_policy_bundle -- \
      --checkpoint <checkpoint> \
      --artifact-dossier <artifact-dossier.json> \
      --benchmark-dossier <benchmark-dossier.json> \
      --output outputs/solver-promotion/nlhe-heads-up/bundle.json
    bash tests/e2e/promotion_manifest.sh

## Acceptance Criteria

- NLHE can emit and verify a policy bundle with dossier-backed provenance.
- The manifest gate recognizes NLHE as `promotable_local`.
- Sparse bootstrap artifacts are explicitly rejected as promotion inputs.
- Bundle verification and deterministic sampling are test-backed.

## Validation and Acceptance

1. `cargo test -p myosu-games-poker --quiet policy_bundle` passes.
2. The example binary writes a valid NLHE bundle JSON under
   `outputs/solver-promotion/nlhe-heads-up/`.
3. `bash tests/e2e/promotion_manifest.sh` passes with NLHE at
   `promotable_local`.
4. A negative test proves sparse bootstrap artifacts cannot satisfy the
   promotion bar.

## Idempotence and Recovery

All changes are additive. Bundle generation can be rerun safely. If the pinned
dossier is later replaced with a stronger one, that should be a dossier swap and
manifest update, not a contract rewrite.

## Artifacts and Notes

Expected outputs:

    outputs/solver-promotion/nlhe-heads-up/bundle.json
    outputs/solver-promotion/nlhe-heads-up/benchmark-summary.json
    outputs/solver-promotion/nlhe-heads-up/artifact-dossier.json

## Interfaces and Dependencies

Expected builder surface inside `crates/myosu-games-poker/src/`:

    pub fn build_nlhe_policy_bundle(
        solver: &PokerSolver,
        artifact_dossier: &NlheArtifactDossier,
        benchmark_dossier: &NlheBenchmarkDossier,
        decision_label: &str,
    ) -> Result<CanonicalPolicyBundle, PokerSolverError>;

Revision note (2026-04-11 / Codex review): replaced the earlier sparse-artifact
promotion assumption with a truthful pinned-dossier promotion gate and added
compatibility verification/acceptance headings.
