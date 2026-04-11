# Checkpoint: Dedicated Game Promotion Readiness

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. This document must be maintained in accordance with root `PLANS.md`.

## Purpose / Big Picture

This is a decision gate after the first two dedicated games are promoted.
Before proceeding to portfolio deepening or same-TUI pilot work, this
checkpoint verifies that the promotion pipeline works end to end for both NLHE
and Liar's Dice and that the resulting policy bundles are structurally sound.

This checkpoint decides whether the dedicated-game evidence is strong enough to
open the expansion plans (`009` cribbage deepening and `010` Bitino local
adapter) without reopening the core promotion contract.

## Requirements Trace

- R1: NLHE and Liar's Dice are both at `promotable_local` in `ops/solver_promotion.yaml`
- R2: Both games have verified policy bundles under `outputs/solver-promotion/`
- R3: `bash tests/e2e/promotion_manifest.sh` passes with both at `promotable_local`
- R4: No existing E2E test regresses
- R5: Decision recorded: open the expansion plans or return to promotion remediation

## Scope Boundaries

This checkpoint does not add code. It verifies plans 004-006 and makes a sequencing decision.

## Progress

- [ ] Verify NLHE policy bundle exists and verifies
- [ ] Verify Liar's Dice policy bundle exists and verifies
- [ ] Run `bash tests/e2e/promotion_manifest.sh` -- both at `promotable_local`
- [ ] Run `bash tests/e2e/research_games_harness.sh` -- no regressions
- [ ] Run `bash tests/e2e/research_strength_harness.sh` -- no regressions
- [ ] Record sequencing decision in Decision Log

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: This checkpoint opens expansion work only after dedicated-game
  promotion evidence is stable.
  Rationale: The next plans depend on the policy-bundle and promotion-gate
  contract being real rather than provisional.
  Date/Author: 2026-04-11 / Codex review

## Outcomes & Retrospective

None yet.

## Context and Orientation

This checkpoint runs after plans 004 (NLHE benchmark unblock), 005 (NLHE promotion), and 006 (Liar's Dice promotion). The outputs to verify are:

- `ops/solver_promotion.yaml` -- NLHE and Liar's Dice at `promotable_local`
- `outputs/solver-promotion/nlhe-heads-up/bundle.json` -- verified bundle
- `outputs/solver-promotion/liars-dice/bundle.json` -- verified bundle
- All E2E harnesses green

## Plan of Work

Run verification commands. Record whether the repo is ready to start the
cribbage and same-TUI pilot plans, or whether remediation is still required
inside the dedicated-game stream.

## Implementation Units

### Unit 1: Verification and decision

Goal: Verify promotion pipeline, record decision.
Requirements advanced: R1-R5.
Dependencies: Plans 004, 005, 006 completed.
Files to create or modify: None.
Tests to add or modify: None.
Approach: Run commands, inspect outputs, record decision.
Test expectation: none -- verification-only checkpoint.

## Concrete Steps

Use the repo-root gate sequence below before opening the expansion plans.

## Verification

Run the commands below from the repo root and inspect the promoted bundle
artifacts before recording the gate decision.

From the repository root:

    bash tests/e2e/promotion_manifest.sh
    bash tests/e2e/research_games_harness.sh
    bash tests/e2e/research_strength_harness.sh
    test -f outputs/solver-promotion/nlhe-heads-up/bundle.json
    test -f outputs/solver-promotion/liars-dice/bundle.json

All commands must exit 0.

## Gate Criteria

- both dedicated promotion entries exist in the ledger
- both promoted bundle artifacts exist
- the manifest and research harnesses stay green
- the decision log explicitly records either `open expansion plans` or the
  blocking remediation item

## Validation and Acceptance

The checkpoint passes when all verification commands exit 0 and the Decision Log records a sequencing decision.

## Idempotence and Recovery

Re-runnable. No side effects.

## Artifacts and Notes

No new artifacts. Decision Log entry only.

## Interfaces and Dependencies

Depends on outputs from plans 004, 005, 006. Consumed by plans 009 and 010.

Revision note (2026-04-11 / Codex review): changed this checkpoint from an
either/or Bitino-vs-cribbage redirect into an explicit expansion gate and added
compatibility verification/gate headings.
