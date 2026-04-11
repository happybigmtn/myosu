# Checkpoint: Policy and Promotion Readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

This is a decision gate, not an implementation plan. Before any game can be promoted to `benchmarked` or `promotable_local` tier, the policy bundle contract and promotion ledger must be stable. This checkpoint verifies that plans 001 and 002 are complete and that the resulting surfaces are CI-green and usable by downstream promotion plans.

## Requirements Trace

- R1: `crates/myosu-games-canonical/src/policy.rs` exists and exports all types from plan 001
- R2: `cargo test -p myosu-games-canonical --quiet policy` passes with at least 6 tests
- R3: `ops/solver_promotion.yaml` exists with 22 game entries
- R4: `bash tests/e2e/promotion_manifest.sh` passes
- R5: `cargo clippy -p myosu-games-canonical -- -D warnings` passes
- R6: No existing test regresses (`cargo test -p myosu-games-canonical --quiet` all green)

## Scope Boundaries

This checkpoint does not add or change code. It verifies that plans 001 and 002 are complete and decides whether to proceed to promotion work. If any requirement is not met, the checkpoint blocks until the issue is resolved.

## Progress

- [ ] Verify plan 001 completion: policy types, verification, sampling
- [ ] Verify plan 002 completion: YAML ledger, manifest binary, E2E harness
- [ ] Run full canonical crate test suite
- [ ] Run promotion manifest harness
- [ ] Confirm no regressions in active-crates CI job
- [ ] Record gate decision: proceed to plans 004-006 or remediate

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: This checkpoint exists as a separate plan rather than an inline note in plan 004.
  Rationale: The master plan milestones 1-2 are foundational. If the policy types or ledger have design flaws, it is cheaper to discover them before building promotion dossiers on top.
  Date/Author: 2026-04-11 / genesis corpus

## Outcomes & Retrospective

None yet.

## Context and Orientation

This checkpoint runs after plans 001 (policy bundle contract) and 002 (promotion ledger) are complete. The relevant files are:

- `crates/myosu-games-canonical/src/policy.rs` -- created by plan 001
- `crates/myosu-games-canonical/src/lib.rs` -- modified by plan 001 to export policy types
- `ops/solver_promotion.yaml` -- created by plan 002
- `crates/myosu-games-canonical/examples/promotion_manifest.rs` -- created by plan 002
- `tests/e2e/promotion_manifest.sh` -- created by plan 002

## Plan of Work

Run the verification commands listed in Concrete Steps. If all pass, record the gate decision as "proceed" in the Decision Log and mark this plan complete. If any fail, record the failure and block downstream plans until the issue is fixed.

## Implementation Units

### Unit 1: Verification sweep

Goal: Confirm all checkpoint requirements are met.
Requirements advanced: R1-R6.
Dependencies: Plans 001 and 002 completed.
Files to create or modify: None (verification only).
Tests to add or modify: None.
Approach: Run commands, inspect output.
Test expectation: none -- this is a verification-only checkpoint.

## Concrete Steps

Use the repo-root gate sequence below before recording the decision.

## Verification

Run the commands below exactly and inspect the manifest output before recording
the gate decision.

From the repository root:

    cargo test -p myosu-games-canonical --quiet
    cargo test -p myosu-games-canonical --quiet policy
    cargo clippy -p myosu-games-canonical -- -D warnings
    cargo run -p myosu-games-canonical --example promotion_manifest -- --format table
    bash tests/e2e/promotion_manifest.sh
    test -f ops/solver_promotion.yaml

All commands must exit 0. The policy test suite must show at least 6 passing tests. The promotion manifest must list 22 games.

## Gate Criteria

- policy types, verification, and sampling all exist and stay green
- the 22-entry promotion ledger exists and is parseable
- the manifest and harness both pass
- the decision log explicitly records either `proceed` or the blocking defect

## Validation and Acceptance

The checkpoint is accepted when all six commands above exit 0 and the Decision Log records "proceed to promotion plans."

## Idempotence and Recovery

This checkpoint can be re-run at any time. It has no side effects.

## Artifacts and Notes

No new artifacts. This checkpoint produces a Decision Log entry only.

## Interfaces and Dependencies

This checkpoint depends on:
- Plan 001 output: `crates/myosu-games-canonical/src/policy.rs`
- Plan 002 output: `ops/solver_promotion.yaml`, `promotion_manifest.rs`, `promotion_manifest.sh`

Revision note (2026-04-11 / Codex review): added explicit verification and gate
headings for compatibility with the repo's current plan-quality check.
