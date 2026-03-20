# Foundations Lane — Execution Infrastructure Honest Slice

This document is the `foundations` lane's implementation plan. It is a living
document and must be updated as defects are discovered, fixed, and re-verified.

`PLANS.md` is checked into the repository root and this document is maintained
in accordance with it. This lane depends on the Fabro primary executor decision
recorded in `specs/031826-fabro-primary-executor-decision.md` and the
bootstrap surface in `plans/031826-bootstrap-fabro-primary-executor-surface.md`.

## Purpose / Big Picture

The `foundations` lane does not ship code. It ships trustworthy execution
infrastructure for every other lane. After this lane completes a cycle:

- `fabro run` commands produce run records with truthful status (not
  false-positives)
- `raspberry execute` / `status` / `watch` commands report real lane health
- Any Fabro-dispatched lane that fails does so honestly — failure is visible,
  not hidden behind a false success artifact

The immediate trigger is the `games:multi-game` false-submit: the lane's
bootstrap workflow produced `spec.md` and `review.md` artifacts while none of the
underlying code exists, none of the proof commands would pass, and the
`multi-game.fabro` workflow graph uses the wrong prompt at the `polish` node.

## Progress

- [x] (2026-03-20) Created `outputs/foundations/` directory and this plan
- [ ] Diagnose the `games:multi-game` false-submit root cause
- [ ] Identify all Fabro workflow graphs that use incorrect prompt assignments
- [ ] Fix the `multi-game.fabro` `polish` node prompt (replace `review.md`
      with correct polish prompt or remove the node)
- [ ] Re-run `games:multi-game` bootstrap lane with truthful prompts, producing
      honest artifacts or honest failure
- [ ] Audit `raspberry execute` / `status` / `watch` for truthfulness gaps
- [ ] Fix identified Fabro/Raspberry defects discovered during re-execution
- [ ] Re-verify the affected frontier until `execute/status/watch` truth is
      trustworthy
- [ ] Produce honest `review.md` confirming the foundations are trustworthy

## Surprises & Discoveries

- Observation: `fabro/workflows/bootstrap/multi-game.fabro` uses `review.md` (the
    review prompt) as the `polish` node prompt, which is not a standard
    bootstrap step and does not align with the bootstrap workflow family
    Evidence: `multi-game.fabro` line 17: `polish [label="Polish", prompt="@../../prompts/bootstrap/review.md", ...]`

- Observation: `fabro/run-configs/platform/multi-game.toml` is in `run-configs/platform/`
    not `run-configs/bootstrap/`, meaning it is a platform lane, not a
    bootstrap lane — yet the workflow graph uses the bootstrap prompt family
    Evidence: `run-configs/platform/multi-game.toml` line 2: `graph = "../../workflows/bootstrap/multi-game.fabro"`

- Observation: `outputs/games/multi-game/spec.md` and `outputs/games/multi-game/review.md`
    exist and claim the lane is complete with KEEP judgment, but the review
    itself lists zero proof commands that would actually pass today
    Evidence: `outputs/games/multi-game/review.md` lines 15–56 list proof commands
    that all require `myosu-games-liars-dice` to exist; the crate does not exist

- Observation: `fabro/workflows/bootstrap/game-traits.fabro` uses the correct prompt
    structure (`plan.md` → `implement.md` → `review.md` → verify), which is the
    reference pattern for a correct bootstrap workflow
    Evidence: `fabro/workflows/bootstrap/game-traits.fabro` (exists and passes)

## Decision Log

- Decision: the `games:multi-game` lane artifacts must be treated as invalid
    until re-run with a correct workflow that produces truthful artifacts.
    Rationale: the existing artifacts were produced by a misconfigured workflow
    that used `review.md` as a production prompt, not as a review prompt.
    Date/Author: 2026-03-20 / foundations lane

- Decision: the `foundations` lane operates on the execution substrate itself,
    not on any product crate.
    Rationale: the trigger is a Fabro workflow misconfiguration, not a broken
    myosu crate. The fix belongs in the workflow graph and prompt assignments,
    not in any `crates/` directory.
    Date/Author: 2026-03-20 / foundations lane

- Decision: do not delete the existing `games:multi-game` artifacts until the
    re-run produces new artifacts or honest failure.
    Rationale: the artifacts serve as evidence of the false-submit. Removing
    them before the re-run destroys the audit trail.
    Date/Author: 2026-03-20 / foundations lane

## Concrete Steps

### Step 1 — Diagnose the false-submit root cause

Inspect `fabro/workflows/bootstrap/multi-game.fabro`:

```bash
cat fabro/workflows/bootstrap/multi-game.fabro
```

Confirm that:
- The `polish` node uses `review.md` which is not a production prompt
- The `review` node uses `implement.md` which is wrong for a bootstrap lane
- The correct bootstrap prompt sequence is `plan.md` → `review.md` → verify
  (bootstrap lanes do not have a separate implement step)

### Step 2 — Audit all workflow graphs for prompt misassignment

```bash
for graph in fabro/workflows/bootstrap/*.fabro; do
  echo "=== $graph ==="
  grep -E 'prompt="' "$graph"
done
```

Compare each graph against its run-config category (`bootstrap/` vs `platform/`).
Bootstrap graphs should use `plan.md` and `review.md` only. Platform or
implement graphs should use `implement.md` and `review.md`.

### Step 3 — Fix the `multi-game.fabro` workflow

The `games:multi-game` lane is a platform lane (not a bootstrap lane), but
its workflow lives in `workflows/bootstrap/`. There are two honest options:

**Option A** (preferred): Move it to `workflows/platform/` and fix the
prompt chain to `plan.md` → `implement.md` → `review.md` → verify.

**Option B**: If it truly is a bootstrap lane, rewrite the graph to use
`plan.md` → `review.md` → verify, and remove the `polish` node entirely.

The re-run must produce either honest artifacts or honest failure. It must
not produce artifacts through a misconfigured workflow.

### Step 4 — Re-run the `games:multi-game` lane

```bash
fabro run fabro/run-configs/platform/multi-game.toml
```

Observe whether the lane produces truthful artifacts. If it fails, capture the
failure evidence and update `review.md` with the honest failure mode.

### Step 5 — Audit `raspberry execute / status / watch` truthfulness

Run a Raspberry status command and compare its output against what `fabro
inspect` would report for the same runs:

```bash
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml
fabro inspect --help   # check available inspection commands
```

Identify any gaps where Raspberry reports healthy but Fabro shows failure, or
vice versa.

### Step 6 — Fix defects discovered during re-execution

Any defect found during Step 4 or Step 5 that is in the Fabro/Raspberry
layer (not in a product crate) is a foundations-lane fix. Fix it in place
and document it in `review.md`.

### Step 7 — Produce honest `outputs/foundations/review.md`

Update the companion `review.md` with the honest assessment of what was fixed,
what remains broken, and whether the execution substrate is now trustworthy.

## Validation and Acceptance

Acceptance for the foundations lane is not a single command exit code — it is
a state of knowledge:

- The `games:multi-game` false-submit is understood and either fixed or honestly
  documented as a known defect
- All Fabro workflow graphs in `fabro/workflows/` use prompts appropriate to
  their lane category
- `raspberry status` and `fabro inspect` agree on lane health for at least one
  re-run cycle
- No bootstrap lane can produce artifacts through a misconfigured workflow
  without the defect being visible and documented
- The foundations lane produces honest artifacts: success is real, failure is
  visible

## Context and Orientation

The `foundations` lane is unique among Myosu lanes: it operates on the
execution substrate (Fabro workflows, Raspberry supervision) rather than on
product code (`crates/`). Its "customers" are all other lanes. If
`foundations` produces false positives, every downstream lane is corrupted.

The lane is triggered by the `games:multi-game` false-submit but its scope
is wider: any Fabro/Raspberry defect discovered during real Myosu execution
belongs to this lane until fixed.

## Artifacts Produced by This Lane

| Artifact | Path |
|----------|------|
| This plan | `outputs/foundations/foundation-plan.md` |
| Review | `outputs/foundations/review.md` |
| Fixed workflow graphs | `fabro/workflows/` |
| Fixed run configs | `fabro/run-configs/` |

## Interfaces and Dependencies

The foundations lane depends on:

- `fabro/` execution plane assets (workflows, run configs, prompts, checks)
- `fabro/programs/myosu.yaml` and related manifests
- `.raspberry/` state (Raspberry program state)
- `outputs/games/multi-game/` (the false-submit evidence)

The foundations lane does not own any `crates/` code. Any fix that belongs in
`crates/` is a product lane fix, not a foundations fix.
