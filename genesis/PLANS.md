# Genesis Plan Index

Generated: 2026-04-11

This index points to the generated numbered ExecPlans under `genesis/plans/`.
It is intentionally only an index and sequencing guide. The active planning
surface is [plans/001-master-plan.md](/home/r/Coding/myosu/plans/001-master-plan.md),
while root `PLANS.md` still defines the ExecPlan structure. These generated
plans are subordinate decompositions of the active master plan, not a parallel
control plane.

Compatibility note:
the current repo still has a legacy CI shape check in
`.github/scripts/check_plan_quality.sh` that looks for `## Acceptance Criteria`
or `## Gate Criteria` plus `## Verification`. The revised genesis plans carry
those headings in addition to the newer ExecPlan sections so the corpus is
compatible with both the living standard and the live repo tooling.

## Plan Set

| # | File | Title | Status |
|---|------|-------|--------|
| 001 | `genesis/plans/001-master-plan.md` | Canonical policy bundle contract | Ready |
| 002 | `genesis/plans/002-promotion-ledger.md` | Promotion ledger and benchmark gate surface | Ready |
| 003 | `genesis/plans/003-checkpoint-policy-promotion.md` | Checkpoint: policy and promotion readiness | Blocked on 001-002 |
| 004 | `genesis/plans/004-nlhe-benchmark-unblock.md` | NLHE benchmark/dossier unblock | Ready |
| 005 | `genesis/plans/005-nlhe-promotion.md` | Promote NLHE heads-up to `promotable_local` | Blocked on 003-004 |
| 006 | `genesis/plans/006-liars-dice-promotion.md` | Promote Liar's Dice to `promotable_local` | Blocked on 003 |
| 007 | `genesis/plans/007-checkpoint-dedicated-promotion.md` | Checkpoint: dedicated-promotion readiness | Blocked on 005-006 |
| 008 | `genesis/plans/008-security-debt-triage.md` | Security advisory triage (`SEC-001`) | Parallel |
| 009 | `genesis/plans/009-cribbage-deepening.md` | Deepen cribbage to `benchmarked` | Blocked on 007 |
| 010 | `genesis/plans/010-bitino-local-adapter.md` | Bitino local adapter and same-TUI pilot | Blocked on 007 |
| 011 | `genesis/plans/011-checkpoint-bitino-pilot.md` | Checkpoint: same-TUI pilot readiness | Blocked on 009-010 |

## Sequencing Rationale

The sequence now follows three clusters:

1. **Foundation**: plans `001-003` define the policy bundle and promotion
   ledger, then stop at a gate before downstream plans depend on them.
2. **Dedicated-game proof**: plans `004-007` unblock NLHE evidence, promote the
   two strongest dedicated games, then stop again before expanding scope.
3. **Expansion**: plans `008-011` handle parallel security triage, deepen the
   first portfolio game, and then execute the sibling-repo Bitino local adapter
   and same-TUI pilot described by the active master plan.

## Why This Order

### Alternative A: Security first, then promotion

Rejected. The security work is important, but the next product-learning loop is
still promotion credibility, not dependency hygiene alone.

### Alternative B: Immediate Bitino implementation before grounding the sibling-repo surface

Rejected as the initial sequence. Ungrounded direct sibling-repo edits would
have been speculative. The corrected plan set now follows the active master
plan instead: policy/promotion first, then grounded `../bitino/` adapter work
against the inspected presentation and `GameId` surfaces.

### Alternative C: Deepen multiple portfolio games before any same-TUI pilot

Rejected for now. One portfolio proof is enough to test whether the promotion
pipeline generalizes; broadening the portfolio before the export contract is
stable would mostly increase uncertainty, not confidence.

## Dependency Graph

```text
001 -> 003
002 -^

004 -> 005 -> 007 -> 009 -> 011
006 -^    -^      \-> 010 -^

008 runs in parallel with the main sequence
```
