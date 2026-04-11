# Promote Liar's Dice to Promotable Local

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, `liars-dice` is promoted to `promotable_local` alongside NLHE. Liar's Dice has a different benchmark surface than poker: it uses exact exploitability from its MCCFR solver rather than a scenario-pack reference. The promotion pipeline for Liar's Dice demonstrates that the canonical policy bundle contract works across game families, not just for the NLHE flagship.

An operator can produce a verified Liar's Dice policy bundle with exact exploitability evidence as benchmark provenance.

## Requirements Trace

- R1: `liars-dice` entry in `ops/solver_promotion.yaml` updated to `tier: promotable_local`
- R2: A Liar's Dice policy bundle can be constructed from a solver checkpoint with exploitability evidence
- R3: The bundle's provenance references exact exploitability as the benchmark metric
- R4: `bash tests/e2e/promotion_manifest.sh` passes with Liar's Dice at `promotable_local`
- R5: An example or test constructs, verifies, and samples from a Liar's Dice policy bundle

## Scope Boundaries

This plan promotes only `liars-dice`. It does not require external artifacts (the Liar's Dice solver is self-contained with `1 << 10` game trees). It does not change the miner or validator.

## Progress

- [ ] Add `build_liars_dice_policy_bundle()` function in `crates/myosu-games-liars-dice/src/`
- [ ] Wire builder to produce bundle from solver checkpoint + exploitability measurement
- [ ] Add exploitability measurement to LiarsDiceSolver (if not already exposed)
- [ ] Add test: bundle from solver with known exploitability
- [ ] Add test: bundle construction → verification → sampling roundtrip
- [ ] Update `ops/solver_promotion.yaml` Liar's Dice entry to `tier: promotable_local`
- [ ] Verify `bash tests/e2e/promotion_manifest.sh` passes

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: Liar's Dice uses exact exploitability as its benchmark metric rather than a scenario pack.
  Rationale: The Liar's Dice game tree is small enough (1,024 trees at `1 << 10`) to compute exact exploitability. This is a stronger benchmark than a reference pack and demonstrates that the promotion pipeline supports diverse benchmark types.
  Date/Author: 2026-04-11 / genesis corpus

## Outcomes & Retrospective

None yet.

## Context and Orientation

The relevant files are:

- `crates/myosu-games-liars-dice/src/lib.rs`: Exports `LiarsDiceSolver`, `LiarsDiceGame`, wire codecs
- `crates/myosu-games-liars-dice/src/solver.rs`: Implements the MCCFR solver with `1 << 10` game trees
- `crates/myosu-games/src/traits.rs`: The `Profile` trait from robopoker provides exploitability computation

The `LiarsDiceSolver` wraps a robopoker `Solver` instance. The `Profile` trait has an `exploitability()` method that computes `(BR(P1) + BR(P2)) / 2`. This is already available through the robopoker dependency.

A "benchmark dossier" for Liar's Dice is simpler than for poker: it records the solver tree count, iteration count, and the exact exploitability value. No external artifacts are needed.

## Plan of Work

1. Add a `LiarsDiceBenchmarkDossier` struct in `crates/myosu-games-liars-dice/src/solver.rs` (or a new module). It records: tree_count, iteration_count, exploitability, solver_family, and a summary string.

2. Add `build_liars_dice_policy_bundle()` that takes a solver instance, selects a representative information set, queries for the strategy distribution, and assembles a `CanonicalPolicyBundle` with exploitability-based provenance.

3. Add unit tests covering bundle construction, verification, and sampling.

4. Update `ops/solver_promotion.yaml` entry for `liars-dice` to `tier: promotable_local`.

## Implementation Units

### Unit 1: Benchmark dossier and policy bundle builder

Goal: Add Liar's Dice benchmark dossier and policy bundle construction.
Requirements advanced: R2, R3, R5.
Dependencies: Plan 001 (policy types).
Files to create or modify: `crates/myosu-games-liars-dice/src/solver.rs` or new module.
Tests to add: Bundle construction, verification, sampling.
Approach: Use existing solver, query strategy, convert to policy types.
Test scenarios: (a) Bundle from default solver, (b) Exploitability in provenance, (c) Sampling determinism.

### Unit 2: Promotion ledger update

Goal: Update YAML and verify manifest.
Requirements advanced: R1, R4.
Dependencies: Unit 1, Plan 002.
Files to modify: `ops/solver_promotion.yaml`.
Tests to add: None (covered by manifest harness).
Approach: Change tier, run harness.
Test scenarios: (a) Manifest shows Liar's Dice at `promotable_local`.

## Concrete Steps

Use the repo-root sequence below to prove both bundle construction and manifest promotion.

## Verification

Use the commands below to prove bundle construction and ledger promotion.

From the repository root:

    cargo test -p myosu-games-liars-dice --quiet policy_bundle
    bash tests/e2e/promotion_manifest.sh

## Acceptance Criteria

- `liars-dice` can emit a verified, sampleable policy bundle.
- The bundle provenance carries exact-exploitability evidence.
- The promotion manifest recognizes `liars-dice` as `promotable_local`.

## Validation and Acceptance

1. `cargo test -p myosu-games-liars-dice --quiet policy_bundle` passes with at least 2 tests.
2. Manifest harness shows Liar's Dice at `promotable_local`.
3. The bundle provenance contains `metric_name: exploitability` with a numeric value.

## Idempotence and Recovery

All changes are additive. The solver is deterministic with fixed tree count, so repeated runs produce identical bundles.

## Artifacts and Notes

Expected outputs:
    outputs/solver-promotion/liars-dice/bundle.json (production location)

## Interfaces and Dependencies

New function in `crates/myosu-games-liars-dice/src/`:

    pub fn build_liars_dice_policy_bundle(
        solver: &LiarsDiceSolver,
        decision_label: &str,
    ) -> Result<CanonicalPolicyBundle, LiarsDiceSolverError>;

Dependencies: `myosu-games-canonical` (policy types), `myosu-games-liars-dice` (solver).

Revision note (2026-04-11 / Codex review): added explicit verification and
acceptance headings for plan-quality compatibility.
