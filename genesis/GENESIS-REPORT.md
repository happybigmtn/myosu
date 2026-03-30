# Genesis Report

**Date**: 2026-03-30
**Author**: Genesis synthesis run, adjudicated in-repo by Codex
**Corpus**: adjudicated master plan plus supporting ExecPlans

## Executive Summary

The Genesis run was useful as a planning corpus, but its status language had
gone stale. The repo no longer needs a report about how to return to the
chain-first path. It needs a report that says where that adjudication landed.

The current adjudicated truth is:

- active next-step plan: `010`
- completed locally: `002` through `009`, `011` through `020`
- future synth governance now lives in `genesis/PLANS.md`
- no generated plan is implicitly active anymore; each one is now completed,
  active next-step, next queued, or historical/reference-only

## What Changed

The earlier report still described a repo that had to claw its way back from
doctrine work to chain execution. That was true during the first adjudication,
but it is no longer true now that the stage-0 local loop and the two-subnet
second-game proof are both closed locally.

The corpus is now organized around the current reality:

- `010` is the only active next-step plan, and its only honest open item is
  the hosted GitHub Actions closure proof.
- `014` through `019` are completed local doctrine cleanups, not deferred
  abstractions.
- `020` is completed locally, so the multi-game proof is no longer just an
  architectural seam; it is an owned two-subnet execution proof.
- future `fabro synth genesis` runs now have a documented launch procedure,
  provider order, fallback posture, and adjudication-before-merge rule in
  `genesis/PLANS.md`.

## Current Plan Set

| Status | Plans | Notes |
|--------|-------|-------|
| Active next-step | `001`, `010` | `001` remains the control-plane view; `010` is the only live execution item still open |
| Completed locally | `002`-`009`, `011`-`020` | These plans now have local proof, synced doctrine surfaces, or documented governance policy behind them |
| Historical / reference-only | archived Genesis runs, dropped records, `specsarchive/`, the legacy executor spec | Preserved for provenance, not for live prioritization |

## Remaining Gaps

The Genesis corpus is more truthful than it was, but it still records real
open edges:

1. `010` still needs hosted GitHub Actions timing evidence. Local warm-cache
   timing is good enough to proceed, but it is not the same thing as the
   hosted runner result. That proof surface now exists: draft PR `#1`
   published the workflow and produced hosted runs. The old remote-drift
   blocker is gone on the draft PR branch. The latest honest hosted result is
   run `23730306070`: `Stage-0 Repo Shape`, `Plan Quality`, and `Doctrine
   Integrity` all passed on the current repo surface, then the long lanes
   failed concretely. `Active Crates` exposed two `myosu-play` startup-state
   tests that depended on ambient local artifact discovery, while `Chain Core`
   and `Chain Clippy` both failed because the runner lacked `protoc`. Local
   fixes for those blockers are now the immediate path to the first real hosted
   green timing run.
2. The canonical spec namespace is still messy. Empty specs and duplicate spec
   mirrors remain a real planning-quality problem.
3. Future synth governance is now documented, but it has only been exercised on
   the current corpus so far; the next real synth pass will be the first proof
   that the written procedure is easy to follow in practice.

## Recommended Next Steps

1. Publish enough of the current stage-0 repo surface that one hosted GitHub
   Actions run is exercising the same workspace/spec/doctrine shape already
   proved locally, then close `010` honestly.
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

# The assessment and report both acknowledge the hosted-CI proof blocker and the
# locally completed two-subnet execution proof
rg -n "hosted GitHub Actions|remote repo drift|two-subnet" \
  genesis/ASSESSMENT.md genesis/GENESIS-REPORT.md
```
