# Genesis Report

**Date**: 2026-03-30
**Author**: Genesis synthesis run, adjudicated in-repo by Codex
**Corpus**: adjudicated master plan plus supporting ExecPlans

## Executive Summary

The Genesis run was useful as a planning corpus, but its status language had
gone stale. The repo no longer needs a report about how to return to the
chain-first path. It needs a report that says where that adjudication landed.

The current adjudicated truth is:

- `021` is the promoted active next-step plan behind `001`
- completed: `002` through `020`
- the stage-0 release gate is now fully synced to live repo surfaces
- future synth governance now lives in `genesis/PLANS.md`
- no generated plan is implicitly active anymore; each one is now completed,
  active next-step, next queued, or historical/reference-only

## What Changed

The earlier report still described a repo that had to claw its way back from
doctrine work to chain execution. That was true during the first adjudication,
but it is no longer true now that the stage-0 local loop and the two-subnet
second-game proof are both closed locally.

The corpus is now organized around the current reality:

- `010` is no longer open; hosted run `23741634642` closed it with a full green
  GitHub Actions proof on the current draft PR branch.
- `014` through `019` are completed local doctrine cleanups, not deferred
  abstractions.
- `020` is completed locally, so the multi-game proof is no longer just an
  architectural seam; it is an owned two-subnet execution proof.
- the remaining release-governance doctrine drift under `011` is now closed, so
  the stage-0 completion claim is no longer blocked by stale invariant
  references.
- future `fabro synth genesis` runs now have a documented launch procedure,
  provider order, fallback posture, and adjudication-before-merge rule in
  `genesis/PLANS.md`.

## Current Plan Set

| Status | Plans | Notes |
|--------|-------|-------|
| Active next-step | `001`, `021` | `001` remains the control-plane view and `021` is now the promoted next lane for operator hardening and network packaging |
| Completed | `002`-`020` | These plans now have local proof, hosted proof, synced doctrine surfaces, or documented governance policy behind them |
| Historical / reference-only | archived Genesis runs, dropped records, `specsarchive/`, the legacy executor spec | Preserved for provenance, not for live prioritization |

## Remaining Gaps

The Genesis corpus is more truthful than it was, but it still records one real
open edge:

1. `021` now needs execution to prove the repo can be run by a second operator
   without relying on founder-local habits.
2. Future synth governance is now documented, but it has only been exercised on
   the current corpus so far; the next real synth pass will be the first proof
   that the written procedure is easy to follow in practice.

## Recommended Next Steps

1. Execute `021` before promoting any broader product or stage-1 lane.
2. Keep treating archived Genesis runs as inputs, not doctrine.
3. Use the new `genesis/PLANS.md` synth-governance procedure unchanged on the
   next real Genesis refresh unless the operator discovers a concrete gap.

## Proof of Re-adjudication

```bash
# The master plan and report agree on the current live/queued set
rg -n "010|018|019|020" genesis/plans/001-master-plan.md genesis/GENESIS-REPORT.md

# The Genesis adjudication and synth-governance plans are both closed locally
rg -n "Completed locally on 2026-03-30" \
  genesis/plans/018-*.md genesis/plans/019-*.md

# The assessment and report both acknowledge the closed hosted-CI proof and the
# reopened canonical-spec freshness pass
rg -n "23741634642|freshness pass|two-subnet" \
  genesis/ASSESSMENT.md genesis/GENESIS-REPORT.md
```
