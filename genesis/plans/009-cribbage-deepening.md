# Deepen Cribbage for Portfolio Promotion

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

After this plan is complete, `cribbage` is the first portfolio-routed game with a real scenario pack and benchmark surface suitable for promotion. The existing cribbage engine at `crates/myosu-games-portfolio/src/core/cribbage.rs` and `src/engines/cribbage.rs` provides rule-aware play, but it currently routes through the generic portfolio solver with no game-specific benchmark or labeled decision scenarios.

This plan adds explicit labeled states (e.g., "opening discard", "pegging play", "counting hand"), a scenario pack, and a benchmark dossier surface. It then promotes cribbage to `benchmarked` tier (not yet `promotable_local`, since the policy bundle builder needs to generalize beyond dedicated games first).

## Requirements Trace

- R1: Cribbage has at least 20 labeled scenario states covering opening discard, pegging, and counting
- R2: A cribbage benchmark dossier can be produced from the scenario pack
- R3: `ops/solver_promotion.yaml` cribbage entry updated to `tier: benchmarked`
- R4: `bash tests/e2e/promotion_manifest.sh` passes with cribbage at `benchmarked`
- R5: The scenario pack and benchmark surface use the existing `PortfolioSolver` and typed challenge protocol

## Scope Boundaries

This plan deepens cribbage only. It does not add MCCFR training for cribbage (the rule-aware engine remains the solver). It does not promote cribbage to `promotable_local` (that requires the policy bundle builder to generalize to portfolio games). It does not change any other game.

## Progress

- [ ] Add labeled cribbage scenario states to `crates/myosu-games-portfolio/src/core/cribbage.rs`
- [ ] Add cribbage scenario pack as an example binary
- [ ] Add cribbage benchmark dossier type and writer
- [ ] Add tests for scenario pack coverage and benchmark
- [ ] Update `ops/solver_promotion.yaml` cribbage entry to `tier: benchmarked`
- [ ] Verify `bash tests/e2e/promotion_manifest.sh` passes

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: Cribbage is the default first portfolio promotion target.
  Rationale: Master plan says "cribbage unless new benchmark evidence makes another family materially easier." Cribbage is two-player, compact, heavily scored, and easier to audit than large hidden-information families.
  Date/Author: 2026-04-11 / master plan

- Decision: Cribbage advances to `benchmarked` tier, not `promotable_local`.
  Rationale: The policy bundle builder (plan 001) is designed for dedicated games initially. Generalizing it to portfolio games requires extending the `CanonicalPolicyBundle` construction to work with the `PortfolioSolver` and typed challenge protocol. That generalization is follow-on work.
  Date/Author: 2026-04-11 / genesis corpus

## Outcomes & Retrospective

None yet.

## Context and Orientation

The relevant files are:

- `crates/myosu-games-portfolio/src/core/cribbage.rs`: Core cribbage model (hand representation, scoring logic, game phases)
- `crates/myosu-games-portfolio/src/engines/cribbage.rs`: Cribbage engine implementation (rule-aware strategy selection)
- `crates/myosu-games-portfolio/src/solver.rs`: `PortfolioSolver` that dispatches to per-game engines
- `crates/myosu-games-portfolio/src/protocol.rs`: `PortfolioAction`, `PortfolioInfo`, typed query/response

Cribbage game phases: deal → discard to crib → cut card → pegging → counting (hand, crib, nobs). The rule-aware engine selects actions based on hand evaluation heuristics.

A "labeled scenario" for cribbage means a specific game state with a descriptive label (e.g., "opening_discard_with_15_2_potential"), the expected legal actions, and the engine's recommended action with quality metrics.

## Plan of Work

1. In `crates/myosu-games-portfolio/src/core/cribbage.rs`, add a function `cribbage_scenario_pack()` that returns a vector of labeled scenario states covering the main decision points.

2. Add an example binary `crates/myosu-games-portfolio/examples/cribbage_benchmark.rs` that runs the scenario pack through the engine and produces a benchmark dossier.

3. Add a `CribbageBenchmarkDossier` type (or reuse a generic `PortfolioBenchmarkDossier`) that captures scenario count, coverage, mean quality, and engine tier.

4. Update `ops/solver_promotion.yaml` cribbage entry to `tier: benchmarked`.

## Implementation Units

### Unit 1: Scenario pack and benchmark

Goal: Add labeled scenarios and benchmark surface.
Requirements advanced: R1, R2, R5.
Dependencies: Existing cribbage engine.
Files to modify: `crates/myosu-games-portfolio/src/core/cribbage.rs`.
Files to create: `crates/myosu-games-portfolio/examples/cribbage_benchmark.rs`.
Tests to add: Scenario count, engine answers all scenarios, benchmark dossier serializes.
Approach: Hand-craft 20+ representative game states across phases.
Test scenarios: (a) All scenarios produce valid engine answers, (b) Dossier captures results.

### Unit 2: Promotion update

Goal: Update ledger.
Requirements advanced: R3, R4.
Dependencies: Unit 1, Plan 002.
Files to modify: `ops/solver_promotion.yaml`.
Tests to add: None (covered by harness).
Approach: Change tier, run harness.
Test scenarios: Manifest shows cribbage at `benchmarked`.

## Concrete Steps

Use the repo-root sequence below to prove both the scenario-pack surface and the ledger change.

## Verification

Use the commands below from the repo root to prove both the scenario-pack
surface and the ledger change.

From the repository root:

    cargo run -p myosu-games-portfolio --example cribbage_benchmark
    cargo test -p myosu-games-portfolio --quiet cribbage
    bash tests/e2e/promotion_manifest.sh

## Acceptance Criteria

- cribbage has a labeled scenario pack and benchmark dossier
- the ledger moves cribbage to `benchmarked`, not beyond
- existing portfolio tests stay green

## Validation and Acceptance

1. The cribbage benchmark example produces at least 20 scenario results.
2. `bash tests/e2e/promotion_manifest.sh` shows cribbage at `benchmarked`.
3. No regressions in existing portfolio tests.

## Idempotence and Recovery

Additive changes. Scenario pack and benchmark example can be rerun safely.

## Artifacts and Notes

Expected outputs:
    outputs/solver-promotion/cribbage/benchmark-dossier.json

## Interfaces and Dependencies

New function in `crates/myosu-games-portfolio/src/core/cribbage.rs`:

    pub fn cribbage_scenario_pack() -> Vec<LabeledScenario>;

Where `LabeledScenario` contains: label, game state, legal actions, expected engine answer.

Dependencies: Existing `PortfolioSolver`, `PortfolioAction`, cribbage engine.

Revision note (2026-04-11 / Codex review): added explicit verification and
acceptance headings for compatibility with the repo's plan checker.
