# Genesis Corpus Adjudication and Downstream Selection

Status: Completed locally on 2026-03-30.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

The fresh Genesis run created a complete corpus, but not every generated plan deserves active status. This plan converts the raw generated corpus into an intentional operator tool: it marks which plans are active now, which are retained for later, and which are only reference material.

After this plan, the Genesis directory will tell one coherent story. A future contributor will not need to infer whether a plan is current, parked, or obsolete.

## Progress

- [x] (2026-03-28) Reviewed the generated corpus against the repo's current doctrine and operator objectives.
- [x] (2026-03-28) Replaced the raw generated master plan with a doctrine-cutover-first master plan.
- [x] (2026-03-30) Synced the remaining Genesis corpus to the adjudicated
  master-plan structure by rewriting `genesis/GENESIS-REPORT.md` to the
  current completed/active/queued state.
- [x] (2026-03-30) Reworked status language so no generated plan is silently
  active anymore: completed plans are marked completed, the remote-only `010`
  proof stays active, and `019` became the final queued doctrine plan before
  being executed in the next slice.
- [x] (2026-03-30) Updated `genesis/ASSESSMENT.md` where later execution had
  materially invalidated older audit statements about miner/validator status,
  CI coverage, multi-game proof, and `myosu-play` decomposition.

## Surprises & Discoveries

- Observation: The generated technical plans are not uniformly bad; the problem is that they were all promoted at once.
  Evidence: 002, 008, and 013 fit the repo well, while several broader architecture plans are reasonable drafts but wrong as immediate priorities.

## Decision Log

- Decision: Adjudication happens in-repo, not in a private operator note.
  Rationale: Future synth runs and contributors need to see the status of each plan from the checked-in corpus itself.
  Date/Author: 2026-03-28 / Codex

- Decision: Use explicit `completed`, `active next-step`, `next queued`, and
  `historical/reference-only` language instead of relying on a generic
  "parked" bucket everywhere.
  Rationale: By 2026-03-30 most of the generated plan set had either been
  executed or intentionally queued, so a blanket parked label would have been
  less truthful than a narrower status taxonomy.
  Date/Author: 2026-03-30 / Codex

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Plan status | Generated plans still look active after their real status changed | Add explicit completed/active/queued language in the master plan and report; add per-plan notes if confusion remains |
| Report sync | `GENESIS-REPORT.md` still narrates the raw synth output | Rewrite the report after the master plan is settled |

## Outcomes & Retrospective

This plan closed once the Genesis corpus stopped narrating an older stage of
repo life. The important change was not adding more Genesis surface. It was
making the existing surfaces agree on the same status model: `010` is still the
only live next-step plan, `019` was clearly surfaced as the next governance
move, and the rest of the selected doctrine/execution work is either completed
locally or historical. That made the corpus much safer to use as an operator
tool and set up the final synth-governance closure cleanly.

## Context and Orientation

This plan is about the Genesis corpus itself:
- `genesis/ASSESSMENT.md`
- `genesis/SPEC.md`
- `genesis/PLANS.md`
- `genesis/GENESIS-REPORT.md`
- `genesis/plans/*.md`

## Milestones

### Milestone 1: Promote the right plans

Make the master plan and report explicitly promote only the doctrine-cutover plans plus the selected downstream technical plans.

Proof command:

    rg -n "010|014|015|016|017|018|019|020" genesis/plans/001-master-plan.md genesis/GENESIS-REPORT.md

### Milestone 2: Make status explicit on purpose

Ensure the generated technical plans are described as completed, queued, or
historical/reference-only instead of being implicitly active.

Proof command:

    rg -n "Completed locally|Next queued|Historical / reference-only" genesis/plans/001-master-plan.md genesis/GENESIS-REPORT.md

### Milestone 3: Keep the assessment useful

Adjust any assessment language that still reflects pre-adjudication repo state
when that conflicts with the current completed/active/queued picture.

Proof command:

    rg -n "hosted GitHub Actions timing proof|two-subnet|1,597" genesis/ASSESSMENT.md

## Plan of Work

1. Rewrite the report.
2. Add or refine status language where needed.
3. Keep the assessment aligned enough to support the adjudicated story.

## Concrete Steps

From `/home/r/coding/myosu`:

    sed -n '1,260p' genesis/plans/001-master-plan.md
    sed -n '1,240p' genesis/GENESIS-REPORT.md
    sed -n '220,245p' genesis/ASSESSMENT.md

## Validation and Acceptance

Accepted when:
- The Genesis master plan and report agree on the active next-step set.
- Generated plans are intentionally classified, not merely omitted.
- The assessment no longer misleads readers about Malinka or plan status.

## Idempotence and Recovery

Adjudication is a repeatable editorial pass. If a later operator revives a parked plan, update the status explicitly rather than silently promoting it.

## Interfaces and Dependencies

Depends on: 014, 015, 016, 017.
Blocks: 019, because future synth governance should assume the corpus knows how to adjudicate itself.

```text
raw synth corpus
      |
      v
master-plan rewrite
      |
      v
report + status sync
      |
      v
adjudicated Genesis corpus
```
