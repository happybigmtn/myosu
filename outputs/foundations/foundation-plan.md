# Foundations Lane — Execution Truth Plan

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it.

## Purpose / Big Picture

After this slice lands, the Myosu control plane will have an honest accounting of
which Fabro execution paths are trustworthy and which are not. A contributor
will be able to look at `outputs/foundations/review.md` and know immediately
whether `raspberry execute/status/watch` is reliable for a given lane, and
whether a detached submit is likely to produce a real running worker.

The user-visible outcome is not a code feature. It is a trust signal: a lane
can now be described as "honest" or "unverified" based on whether its submit
path has been proven to produce a real worker that matches the reported status.

## Progress

- [ ] Fix Raspberry/Fabro defects only when they are discovered by real Myosu
  execution, then rerun the affected frontier until `execute/status/watch` truth
  is trustworthy again.
- [ ] Convert the current Raspberry-dispatched `games:multi-game` false-submit
  into a truthful failure or successful live run, then rerun the lane with the
  repaired Fabro detach path.

## Surprises & Discoveries

No observations recorded yet. This is the first honest slice for this lane.

## Decision Log

No decisions recorded yet. This is the first honest slice for this lane.

## Outcomes & Retrospective

No outcomes recorded yet. This is the first honest slice for this lane.

## Context and Orientation

### The Execution Truth Problem

The Myosu control plane has a **false-submit problem**: `raspberry execute`
can return a run ID for a lane without that lane's worker actually starting.
The run directory shows `status.json = submitted` and a `detach.log` parse
failure, but no `run.pid` and no manifest. This makes `execute/status/watch`
untrustworthy for any lane using the detach path.

The specific known case is:

- **Lane**: `games:multi-game`
- **Submitted via**: `raspberry execute --manifest fabro/programs/myosu-product.yaml`
- **Run ID**: `01KM2BS4ASVRXVT2ND1GVVMKJ0`
- **Observed failure**: Run directory never gained `run.pid` or manifest; only
  `status.json = submitted` and `detach.log` parse failure were present

The fallback that worked:

- Direct foreground Fabro runs (not through Raspberry's detach path) for
  `games:multi-game` (`01KM2CGPHAJ95J38TQ7SPN46NZ`) and `validator:oracle`
  (`01KM2CGPCCC86SHEEQ6QFTRFEM`) produced real manifests, states, and stage
  labels.

### Why This Is a Foundations Problem

This is not a `games:multi-game` lane defect. It is a **control plane
execution truth defect** — the Raspberry supervisory layer reports a submit as
success when the worker never started. Until this is fixed, no lane that
depends on `raspberry execute` can be trusted.

The foundations lane scope is:

1. **Diagnose** the Fabro detach failure path: why does `--detach` emit a run
   ID when the worker process never forks?
2. **Repair** the detach path in `/home/r/coding/fabro`, or route the
   `games:multi-game` lane through the foreground submit path if detach cannot
   be repaired quickly.
3. **Verify** the repair against the same `games:multi-game` lane that exposed
   the defect.
4. **Establish criteria** for when a lane submit path is "honest" vs
   "unverified."

### Key Files

| File | Role |
|------|------|
| `fabro/programs/myosu-product.yaml` | Contains the `games:multi-game` lane unit |
| `/home/r/coding/fabro/` | Sibling Fabro repo where detach path must be fixed |
| `.raspberry/myosu-state.json` | Raspberry program state tracking submit status |
| `outputs/games/multi-game/spec.md` | The `games:multi-game` lane's own spec (do not confuse with this plan) |
| `outputs/foundations/review.md` | The companion review artifact for this lane |

### Terms of Art

- **false-submit**: A `raspberry execute` that returns a run ID but the worker
  never starts. The run directory has `status.json = submitted` and no `run.pid`.
- **detach path**: The Fabro `--detach` flag that backgrounds a worker. The
  defect is that this can return a run ID without the worker process actually
  starting.
- **foreground submit**: Running `fabro run <config.toml>` directly in the
  foreground. This path has been shown to work reliably.
- **execution truth**: Whether `raspberry status` and `raspberry watch`
  accurately reflect what is actually running or has run.

## Plan of Work

The foundations lane has two parallel concerns:

**Track A — Diagnose and repair the detach path.**

The detach path is in `/home/r/coding/fabro`. The symptom is: `--detach`
returns a run ID but the worker never starts (no `run.pid`, no manifest). The
likely root cause is in how Fabro rebuilds the detached worker's argv or how it
detects that the child process has actually started.

Steps:

1. Inspect the failed `games:multi-game` run directory for run
   `01KM2BS4ASVRXVT2ND1GVVMKJ0` in `.raspberry/runs/`.
2. Read the Fabro detach implementation in `/home/r/coding/fabro` to find where
   the argv rebuild happens and where the child-start detection happens.
3. Fix the detach path so a `run.pid` and manifest appear before the run ID is
   returned.
4. Test the fix against a simple lane before rerunning `games:multi-game`.

**Track B — Convert the `games:multi-game` false-submit into a truthful result.**

While Track A is in progress, the `games:multi-game` lane must not be blocked.
Use direct foreground Fabro execution as the honest fallback path.

Steps:

1. Run `games:multi-game` through direct foreground Fabro (not through Raspberry
   detach).
2. Capture whether this produces a truthful failure (a real lane error) or a
   successful live run.
3. If it fails, document the failure honestly in `outputs/games/multi-game/`.
4. If it succeeds, document the successful run and mark the lane as
   "foreground-verified."
5. Once Track A fix is available, rerun through the repaired detach path and
   compare results.

## Concrete Steps

### Track A: Detach Path Repair

1. Find the failed run directory.

       find ~/.raspberry/runs -name "01KM2BS4ASVRXVT2ND1GVVMKJ0" -o -name "01KM2CGPHAJ95J38TQ7SPN46NZ" 2>/dev/null | head -5
       # Also check Fabro's own run directory
       find ~/.fabro/runs -name "01KM2BS4ASVRXVT2ND1GVVMKJ0" 2>/dev/null | head -5

2. Read the detach log and status files.

       cat <run_dir>/detach.log
       cat <run_dir>/status.json
       # Note: no run.pid should be present in the false-submit case

3. Read the Fabro detach implementation.

       # In /home/r/coding/fabro, find the detach logic
       rg -n "detach|spawn|child|run.pid" lib/crates/fabro-cli/src/ | head -40
       rg -n "detach|spawn|child|run.pid" lib/crates/fabro-engine/src/ | head -40

4. Fix the argv rebuilding or child-start detection.

       # The fix should ensure the child process is confirmed running
       # before the run ID is returned to Raspberry.

5. Verify the fix with a test lane.

       bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/game-traits.toml'
       # After this run, confirm run.pid exists in the run directory

### Track B: Honest games:multi-game Result

1. Run `games:multi-game` through foreground Fabro.

       cd /home/r/coding/myosu
       bash -ic '/home/r/.cache/cargo-target/debug/fabro run fabro/run-configs/bootstrap/game-traits.toml'
       # Wait for completion, then inspect

2. Inspect the result.

       /home/r/.cache/cargo-target/debug/fabro inspect <run_id>
       # If this succeeds and shows a real worker, the foreground path works.

3. Update the `games:multi-game` lane artifacts with the honest result.

       # If truthful failure: update outputs/games/multi-game/review.md with the real error
       # If success: update outputs/games/multi-game/review.md noting foreground-verified

4. Once Track A fix is available, rerun through repaired detach path.

       bash -ic 'cd /home/r/coding/myosu && /home/r/.cache/cargo-target/debug/fabro run --detach fabro/run-configs/bootstrap/game-traits.toml'
       # Confirm run.pid appears before run ID is returned

## Validation and Acceptance

For Track A (detach repair):

- A Fabro detach submit for any lane must produce a `run.pid` in the run
  directory before the run ID is returned.
- `raspberry status --manifest fabro/programs/myosu-product.yaml` for a
  detached submit must eventually show `running` or `completed`, not just
  `submitted` forever.
- `raspberry watch --manifest fabro/programs/myosu-product.yaml` for a
  detached submit must show real stage progression.

For Track B (honest games:multi-game):

- `outputs/games/multi-game/review.md` must contain a truthful description of
  what happened when the lane ran, not a false "submitted and running" narrative.
- If the lane failed, the failure mode must be a real lane error (code, config,
  or dependency issue), not a control plane false-submit.
- If the lane succeeded, the review must note it was verified via foreground
  Fabro and is pending detach-path confirmation.

The combined acceptance criterion:

- `raspberry execute --manifest fabro/programs/myosu-product.yaml` for any lane
  that completes successfully must produce a `raspberry status` that matches the
  actual Fabro run state.
- `outputs/foundations/review.md` must contain an explicit honest assessment of
  which submit paths are trustworthy and which remain unverified.

## Idempotence and Recovery

If the detach fix in `/home/r/coding/fabro` breaks other lanes:

1. Revert the Fabro change.
2. Continue using foreground Fabro as the fallback path for blocked lanes.
3. Document the revert in the `Decision Log` with the evidence of what broke.

If a `games:multi-game` foreground run produces a real lane failure (not a
control plane issue):

1. Treat that as an honest lane result, not a control plane defect.
2. Update `outputs/games/multi-game/review.md` with the real failure.
3. Do not paper over lane failures to make the control plane look clean.

If the foundations lane itself is submitted through a false-submit:

1. Treat that as evidence that the detach path is still broken.
2. Switch to foreground Fabro for the foundations lane itself.
3. Update `outputs/foundations/review.md` with this observation.

## Artifacts and Notes

Current anchor artifacts for execution truth:

    .raspberry/runs/01KM2BS4ASVRXVT2ND1GVVMKJ0/  # false-submit
    .raspberry/runs/01KM2CGPHAJ95J38TQ7SPN46NZ/  # foreground success (games:multi-game)
    .raspberry/runs/01KM2CGPCCC86SHEEQ6QFTRFEM/  # foreground success (validator:oracle)
    outputs/games/multi-game/spec.md
    outputs/games/multi-game/review.md
    outputs/foundations/review.md  # this lane's companion review

Current submit path inventory:

| Lane | Submit Path | Status | Notes |
|------|-------------|--------|-------|
| `games:traits` | Direct Fabro foreground | **TRUSTED** | Ran successfully; produced real artifacts |
| `games:multi-game` | Raspberry detach | **BROKEN** | False-submit; no worker started |
| `games:multi-game` | Direct Fabro foreground | **VERIFIED** | Ran successfully; real manifests and state |
| `validator:oracle` | Direct Fabro foreground | **VERIFIED** | Ran successfully; real manifests and state |
| `tui:shell` | Direct Fabro foreground | **TRUSTED** | Bootstrap complete; reviewed artifacts exist |
| `chain:runtime` | Direct Fabro foreground | **TRUSTED** | Bootstrap complete; reviewed artifacts exist |
| `chain:pallet` | Direct Fabro foreground | **TRUSTED** | Bootstrap complete; reviewed artifacts exist |

## Interfaces and Dependencies

This lane depends on:

- `/home/r/coding/fabro` — the sibling Fabro repo where detach path must be
  repaired. This is a cross-repo dependency; the fix lives in Fabro, not Myosu.
- `fabro/programs/myosu-product.yaml` — the program manifest containing the
  `games:multi-game` lane unit.
- `outputs/games/multi-game/spec.md` and `outputs/games/multi-game/review.md`
  — the `games:multi-game` lane's own artifacts, which must be updated
  honestly after this lane's work.

This lane produces:

- `outputs/foundations/foundation-plan.md` — this document (PLANS.md-style
  ExecPlan)
- `outputs/foundations/review.md` — the companion review artifact with honest
  submit-path trust assessments
