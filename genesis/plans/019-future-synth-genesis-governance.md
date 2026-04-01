# Future Synth Genesis Governance

Status: Completed locally on 2026-03-30.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

`fabro synth genesis` is now powerful enough to generate a complete corpus, but it is still a draft generator, not a source of truth. This plan defines how future synth runs should be launched, reviewed, and merged so they clarify next steps instead of rewriting doctrine by accident.

After this plan, the repo will have a clear synthesis policy: preferred provider order, fallback expectations, when to use the live repo versus a worktree, how to archive prior runs, and how to adjudicate generated output before merging it.

## Progress

- [x] (2026-03-28) Verified that `fabro synth genesis` now archives prior runs automatically.
- [x] (2026-03-28) Verified that `claude-opus-4-6` is working again and that a fresh synth run can execute under Opus.
- [x] (2026-03-30) Documented the preferred run procedure for future synth
  passes directly in `genesis/PLANS.md`, including `tmux` launch, light-touch
  polling, and archive expectations.
- [x] (2026-03-30) Recorded explicit provider policy in `genesis/PLANS.md`:
  prefer `claude-opus-4-6`; if fallback is needed, record an intentional
  `gpt-5.4` `high`-reasoning run instead of allowing ambient drift.
- [x] (2026-03-30) Recorded merge discipline in `genesis/PLANS.md`: future
  synth output is advisory, must pass through plan `018` adjudication, and may
  not replace the Genesis corpus wholesale.

## Surprises & Discoveries

- Observation: The synth run was not broken; earlier "stalling" was partly a launch-policy misunderstanding and partly very deep repo exploration.
  Evidence: Once left running long enough in `tmux`, the run produced a full Genesis corpus.
- Observation: Provider choice materially changes trust in the output.
  Evidence: The operator explicitly wanted Opus first, and the previous silent fallback to Codex obscured that the run was no longer using the intended planner.

## Decision Log

- Decision: Future Genesis synth runs are advisory and require explicit adjudication before merge.
  Rationale: The current run produced valuable material but still needed strong post-run curation.
  Date/Author: 2026-03-28 / Codex

- Decision: Preferred planner is `claude-opus-4-6`; fallback, if allowed, should be explicit and high-effort rather than ambient.
  Rationale: Silent provider drift makes the resulting corpus harder to trust.
  Date/Author: 2026-03-28 / Codex

- Decision: Write synth governance into `genesis/PLANS.md` instead of creating
  a new standalone governance document.
  Rationale: Future synth policy is part of how Genesis ExecPlans are authored,
  revised, and merged, so the most durable home is the meta-plan rules file the
  next contributor already has to read.
  Date/Author: 2026-03-30 / Codex

## Failure Modes

| Codepath | Realistic failure | Handling |
|---|---|---|
| Run launch | Operator kills a valid long-running synth pass too early | Standardize `tmux` launch and light polling |
| Provider fallback | Synth silently uses a different model than intended | Record provider policy in-repo and verify process tree before trusting output |
| Merge process | Raw generated corpus replaces doctrine wholesale | Require explicit adjudication step before merge |

## Outcomes & Retrospective

This plan closed without adding another top-level governance artifact. The repo
already had the right place for this policy: `genesis/PLANS.md`, which every
future Genesis contributor must read anyway. The important result is that
future synth runs now have an explicit run procedure, provider order, fallback
posture, live-repo versus worktree rule, and adjudication-before-merge rule.
That means the next synth run can add value without silently destabilizing the
adjudicated corpus.

## Context and Orientation

Relevant surfaces:
- `fabro synth genesis`
- `archive/genesis_*`
- `genesis/`
- provider policy in the Fabro model configuration

## Milestones

### Milestone 1: Standardize the run procedure

Document the preferred run command, `tmux` usage, and light-touch polling procedure.

Proof command:

    test -d archive
    ls archive | tail -n 5

### Milestone 2: Record provider policy

Document the intended provider order for synth runs and the expected fallback posture.

Proof command:

    rg -n "Opus|gpt-5.4|fallback|high" genesis fabro README.md docs || true

### Milestone 3: Require adjudication before merge

Document that every future synth output must pass through the adjudication flow from plan 018 before it becomes live doctrine.

Proof command:

    rg -n "adjudicat" genesis/plans/018-*.md genesis/plans/019-*.md genesis/GENESIS-REPORT.md

## Plan of Work

1. Write the synth procedure.
2. Record provider/fallback expectations.
3. Tie future synth runs to plan 018's adjudication loop.

## Concrete Steps

From `/home/r/coding/myosu`:

    ls archive | tail -n 10
    pgrep -af "fabro synth genesis|claude -p|codex exec" || true

## Validation and Acceptance

Accepted when:
- Future synth runs have a documented launch procedure.
- Provider expectations are explicit.
- No one reading the repo would assume raw synth output should be merged untouched.

## Idempotence and Recovery

The governance procedure can be reused for every future run. If a synth output is poor, archive it and keep the prior adjudicated corpus intact.

## Interfaces and Dependencies

Depends on: 018.
Blocks: none.

```text
synth run
   |
   v
archive prior corpus
   |
   v
generate draft corpus
   |
   v
adjudicate (plan 018)
   |
   v
merge selectively
```

Revision note (2026-03-30): executed the plan by codifying future synth
governance in `genesis/PLANS.md` and syncing the status/control-plane docs to
reflect local completion.
