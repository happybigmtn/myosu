# Checkpoint: Same-TUI Pilot Readiness

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds. This document must be maintained in
accordance with root `PLANS.md`.

## Purpose / Big Picture

This is the decision gate after cribbage deepening and the Bitino local adapter
pilot. It verifies that the first offline same-TUI solver-backed table really
works through the normal Bitino shell and that the Myosu-side promotion and
bundle surfaces remained stable while that happened.

## Requirements Trace

- R1: A solver-backed table renders through the existing Bitino TUI shell from a
  promoted local bundle
- R2: Session or round metadata exposes bundle identity and provenance
- R3: The promotion ledger and manifest still pass after the Bitino pilot work
- R4: A decision is recorded: proceed toward funded integration or remediate
  policy/promotion/adapter gaps first

## Scope Boundaries

This checkpoint spans Myosu and the sibling Bitino pilot. It still does not add
funded settlement or claim live-miner-backed integration.

## Progress

- [ ] Verify a solver-backed table renders in Bitino from a promoted bundle
- [ ] Verify bundle/provenance metadata is visible in the session
- [ ] Re-run the promotion manifest gate
- [ ] Re-run the bundle verification path on the promoted bundle
- [ ] Record the gate decision

## Surprises & Discoveries

None yet.

## Decision Log

- Decision: The same-TUI pilot is the gate before funded integration, not the
  end state.
  Rationale: The system should prove local bundle-backed rendering and
  provenance first, then only later move into funded settlement and replay.
  Date/Author: 2026-04-11 / active root master plan

## Outcomes & Retrospective

None yet.

## Context and Orientation

This checkpoint depends on:

- plan 009: cribbage deepening
- plan 010: Bitino local adapter and same-TUI pilot

The outputs to verify span both repos:

- `outputs/solver-promotion/`
- the Bitino local adapter and TUI smoke path
- the promotion manifest and associated gate

## Plan of Work

Run the local verification commands across Myosu and Bitino. Record one of two
decisions:

- `proceed toward funded integration`
- `remediate policy/promotion/adapter gaps first`

## Implementation Units

### Unit 1: Verification and decision

Goal: Confirm the local same-TUI pilot is stable enough for the next
integration step.
Requirements advanced: R1-R4.
Dependencies: Plans 009 and 010 completed.
Files to create or modify: None.
Tests to add or modify: None.
Approach: Run the Myosu promotion gates plus the Bitino local pilot proof, then
record the decision.
Specific test scenarios:
- solver-backed table renders from a promoted bundle
- manifest gate stays green after pilot work
- provenance remains visible and verifiable

## Concrete Steps

Use the repo-root gate sequence below before deciding whether funded work can
start.

## Verification

    bash tests/e2e/promotion_manifest.sh
    cargo run -p myosu-games-canonical --example verify_policy_bundle -- \
      outputs/solver-promotion/nlhe-heads-up/bundle.json
    cd ../bitino
    cargo run -q -p bitino-play -- --headless 1 \
      --game solver_holdem_heads_up \
      --policy-bundle ../myosu/outputs/solver-promotion/nlhe-heads-up/bundle.json

## Gate Criteria

- promoted bundles verify and the solver-backed table renders cleanly
- the manifest gate still passes
- session metadata exposes bundle/provenance details
- the decision log records whether funded work can begin

## Validation and Acceptance

The checkpoint passes only when the same-TUI pilot is stable enough that the
next work can move toward funded integration without reopening the
policy/promotion contract.

## Idempotence and Recovery

This checkpoint is re-runnable. If it fails, the remedy belongs in Myosu-side
policy/promotion work or the Bitino local adapter, not in funded settlement
code.

## Artifacts and Notes

No new artifacts beyond the decision-log entry.

## Interfaces and Dependencies

Depends on the promoted Myosu outputs and the sibling Bitino local adapter from
plan 010.

Revision note (2026-04-11 / reconciliation pass): restored the grounded
same-TUI pilot gate from the active root master plan after reconciling the
corpus against the inspected Bitino surfaces.
