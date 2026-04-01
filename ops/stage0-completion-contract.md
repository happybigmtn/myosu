# Stage-0 Completion Contract

Last updated: 2026-03-30
Status: Live completion-claim and fail-closed sync contract for stage-0.

## Purpose

This file defines what has to stay in sync before Myosu can honestly treat a
stage-0 completion claim as landed truth.

## Required Surfaces

These surfaces must agree in the same slice:

- `ops/release-gate-stage0.md` marks every invariant `PASS`
- `ops/no-ship-ledger.md` says no active no-ship condition is open
- `genesis/plans/001-master-plan.md` and `genesis/GENESIS-REPORT.md` agree on
  the active and completed plan set
- the hosted CI proof named in `genesis/plans/010-ci-proof-gates-expansion.md`
  remains the cited current hosted gate closure
- the robopoker pin and fork-delta record agree between workspace manifests and
  `docs/robopoker-fork-changelog.md`

## Completion-Claim Procedure

1. Re-run doctrine integrity and plan-quality gates.
2. Re-check the live invariant statuses in `ops/release-gate-stage0.md`.
3. Confirm the no-ship ledger still says no completion blocker is open.
4. Confirm the master plan and Genesis report still describe the same promoted
   plan state.
5. If any named proof or status diverges, do not treat the completion claim as
   landed truth.

## Fail-Closed Rule

If any required surface drifts after a completion claim is asserted:

- reopen the affected invariant or plan status
- restore the no-ship note
- record the divergence in the next doctrine-sync slice
- treat the completion claim as incomplete until the surfaces agree again

## Proof Commands

```bash
bash .github/scripts/check_doctrine_integrity.sh
bash .github/scripts/check_plan_quality.sh
rg -n "No promoted|Completed|011|010|002" \
  genesis/plans/001-master-plan.md \
  genesis/GENESIS-REPORT.md \
  ops/no-ship-ledger.md
rg -n "happybigmtn/robopoker|04716310143094ab41ec7172e6cea5a2a66744ef" \
  crates/myosu-games/Cargo.toml \
  crates/myosu-games-poker/Cargo.toml \
  Cargo.lock \
  docs/robopoker-fork-changelog.md
```
