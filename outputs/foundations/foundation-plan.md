# Foundations Lane — First Honest Slice

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it. This plan depends on
`specs/031626-00-master-index.md` and
`plans/031826-bootstrap-fabro-primary-executor-surface.md`.

## Purpose / Big Picture

After this slice lands, the Myosu control plane will have an honest
foundational layer where:

1. The `games:multi-game` lane is no longer a false submit — it is either
   honestly failed (crate does not exist) or honestly successful (crate
   builds and tests pass)
2. The `execute/status/watch` truth in Raspberry/Fabro is trustworthy,
   meaning lane status reflects actual code state, not cached or stale
   artifacts
3. The Fabro detach path for Raspberry-dispatched lane execution is repaired
   so future submissions are truthful by construction

A contributor can run `fabro run` on any bootstrap or implementation lane and
trust that the reported status matches the actual code state.

## Progress

- [ ] (2026-03-20) Diagnose the `games:multi-game` false-submit root cause
- [ ] (2026-03-20) Assess whether `games:multi-game` lane should be RESET
  (crate does not exist) or if the review.md judgment was premature
- [ ] (2026-03-20) Audit the Raspberry `execute/status/watch` pipeline for
  the specific defect that allowed the false submit
- [ ] (2026-03-20) Produce a repaired `games:multi-game` lane state (either
  truthful failure or live run) and update `outputs/games/multi-game/review.md`
- [ ] (2026-03-20) Validate that `execute/status/watch` now reports truthful
  state for all bootstrap lanes
- [ ] (2026-03-20) Produce reviewed `outputs/foundations/review.md`

## Surprises & Discoveries

_(empty — to be filled during investigation)_

## Decision Log

_(empty — to be filled during investigation)_

## Outcomes & Retrospective

_(empty — to be filled at milestone completion)_

## Context and Orientation

### What Is the False Submit

The `games:multi-game` lane has `outputs/games/multi-game/review.md` with
judgment **KEEP** and a set of proof commands that all reference
`myosu-games-liars-dice`. However, `crates/myosu-games-liars-dice/` does not
exist in the workspace — it is entirely greenfield.

The lane was reviewed and marked complete without the actual implementation
existing. This means the review artifact does not reflect the true state of
the code.

### Why This Matters

The Fabro/Raspberry control plane is supposed to be the source of truth for
lane status. If a lane can be marked KEEP when its primary crate does not
exist, the control plane is not trustworthy. The `execute/status/watch`
commands would report a lane as healthy when it is not.

### Relevant Files

| File | Role |
|------|------|
| `outputs/games/multi-game/review.md` | Contains KEEP judgment for a non-existent crate |
| `outputs/games/multi-game/spec.md` | Specifies the multi-game lane contract |
| `fabro/programs/myosu-product.yaml` | Product program manifest (no games:multi-game lane defined yet) |
| `crates/myosu-games-liars-dice/` | Does NOT exist — this is the core problem |
| `crates/myosu-games/` | Trusted leaf crate; `GameType::LiarsDice` variant exists |
| `fabro/run-configs/` | Run configs for bootstrap lanes |
| `fabro/workflows/` | Workflow graphs for bootstrap lanes |

### Fabro/Raspberry Execution Model

- `fabro run <run-config>` executes a workflow graph
- `raspberry status --manifest <program> --lane <lane>` reports lane health
- `raspberry execute --manifest <program> --lane <lane>` dispatches a Fabro run
- The detach path is how Raspberry hands off to Fabro for execution
- Run truth is stored in Fabro run directories under `~/.fabro/runs/`

### The Two-Problem Structure

The task identifies two distinct problems that this lane must solve:

**Problem 1: `games:multi-game` false submit**
The lane has a review artifact claiming the implementation is KEEP, but the
implementation crate does not exist. This must be resolved — either by
actually implementing the crate (honest success) or by resetting the review
judgment to reflect the true state (honest failure).

**Problem 2: `execute/status/watch` untrustworthy**
The Raspberry commands that report lane status may be returning cached,
stale, or incorrect information. This must be diagnosed and repaired so
future lane status reports are trustworthy by construction.

## Plan of Work

### Phase 1: Honest Diagnosis

First, establish the exact state of the `games:multi-game` lane:

1. Confirm `crates/myosu-games-liars-dice/` does not exist in the workspace
2. Confirm `Cargo.toml` workspace members do not include `myosu-games-liars-dice`
3. Confirm the proof commands in `review.md` all fail (no crate to test)
4. Check whether Fabro has any run history for `games:multi-game` that could
   have been the source of the false submit
5. Check whether Raspberry's `status` command returns cached state that could
   explain the discrepancy

### Phase 2: Assess the `games:multi-game` Review Judgment

The current `review.md` says KEEP. But the lane has:

- A spec (good — the spec is coherent)
- A review judgment (bad — the judgment claims the implementation is KEEP
  when the crate does not exist)
- Proof commands (honest — they would fail if run)

The judgment must be corrected. The lane is not in a KEEP state. The options
are:

**Option A: RESET** — Mark the lane as RESET, noting the crate does not exist.
The spec is preserved. The review is corrected. The lane can be re-opened
when the implementation is ready.

**Option B: IMPLEMENT** — Actually create `myosu-games-liars-dice` and make the
lane honestly successful.

The bias should be toward RESET for this first honest slice, because:
- The task says to fix defects "only when they are discovered by real Myosu
  execution" — the false submit was discovered, not the implementation
- Implementing the full Liar's Dice crate is multiple slices of work
- The foundations lane's job is to make the control plane honest, not to
  implement the multi-game lane

### Phase 3: Diagnose `execute/status/watch` Untrustworthiness

1. Run `raspberry status` for a known-good lane (e.g., `games:traits`) and
   compare with actual code state
2. Run `raspberry status` for the `games:multi-game` lane and observe what it
   reports
3. Check whether the status command reads from:
   - Fabro run directories directly (brittle)
   - Raspberry state file (may be stale)
   - Something else (cached artifact files?)
4. Identify the specific defect: does status return stale cached state, or
   does it return incorrect state from a prior run, or does it return nothing?

### Phase 4: Repair the Detach Path

The Fabro detach path is how Raspberry hands off to Fabro for lane execution.
If this path is broken, Raspberry may report success without Fabro actually
running the lane.

1. Check `fabro/programs/myosu-product.yaml` — is there a `games:multi-game`
   lane defined? If not, that explains why the false submit happened: there was
   no lane to execute.
2. If no lane exists in the manifest, the review artifact was produced by a
   different mechanism (manual review, not Fabro execution)
3. If a lane does exist, check whether the detach path correctly invokes Fabro

### Phase 5: Produce Honest Artifacts

After diagnosis, produce corrected artifacts:

- Update `outputs/games/multi-game/review.md` with the corrected judgment
- Document the specific defect found in `execute/status/watch`
- Document the repair action taken (or if no repair was needed, document
  why the system was already correct)

## Concrete Steps

### Step 1: Confirm the false submit

```bash
# Confirm the crate does not exist
ls crates/myosu-games-liars-dice/ 2>&1
# Expected: "No such file or directory"

# Confirm the workspace does not reference it
grep -n "myosu-games-liars-dice" Cargo.toml
# Expected: no output

# Confirm the review commands would fail
cargo build -p myosu-games-liars-dice 2>&1
# Expected: "package not found"

# Confirm the Fabro run history
ls ~/.fabro/runs/ | grep multi-game
# Expected: no output (or empty)

# Check Raspberry state for this lane
cat .raspberry/myosu-state.json | python -c "import json,sys; d=json.load(sys.stdin); print(d.get('lanes', {}).get('games:multi-game', 'not found'))"
```

### Step 2: Run status for known-good lane

```bash
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml --lane games:traits
# Compare with actual: cargo test -p myosu-games
```

### Step 3: Run status for multi-game lane

```bash
raspberry status --manifest fabro/programs/myosu-product.yaml --lane multi-game 2>&1
# Expected: either "not found" (no lane defined) or "blocked" (crate missing)
```

### Step 4: Check product manifest for multi-game lane

```bash
grep -n "multi-game\|liars-dice\|LiarsDice" fabro/programs/myosu-product.yaml
# Expected: no output if lane is not defined
```

### Step 5: Produce corrected review artifact

After diagnosis, update `outputs/games/multi-game/review.md` with:

- Corrected judgment (RESET, not KEEP)
- Root cause of the false submit
- Specific repair action

## Validation and Acceptance

Acceptance is complete when all of the following are true:

- `outputs/games/multi-game/review.md` has a judgment that matches the true
  code state (either honestly RESET because the crate does not exist, or
  honestly KEEP after the crate is implemented)
- `raspberry status` for any bootstrap lane reports a status consistent with
  the actual code state
- The Fabro detach path for any Raspberry-dispatched lane either:
  - Correctly invokes Fabro and reports its true status, OR
  - Honestly reports that it cannot execute because the lane is not defined
- `outputs/foundations/review.md` contains an honest assessment of the
  foundations lane's current state

## Idempotence and Recovery

This slice should be read-only in terms of code changes to `crates/`. The
only changes should be to `outputs/` artifacts. If the diagnosis reveals that
the false submit was caused by a missing lane definition in a manifest, that
is a repair to be done in a subsequent slice — not in this foundations slice.

If the diagnosis reveals a Fabro or Raspberry code defect, that defect should
be documented in `outputs/foundations/review.md` as a finding, and the repair
should be planned as a subsequent slice.

## Artifacts and Notes

Expected artifacts for this slice:

    outputs/foundations/foundation-plan.md   ← this file
    outputs/foundations/review.md           ← lane trust assessment

The `outputs/games/multi-game/review.md` should be updated with the
corrected judgment as part of this slice.

## Interfaces and Dependencies

The key interfaces for this slice are:

- `raspberry status --manifest <prog> --lane <lane>` — lane health reporter
- `raspberry execute --manifest <prog> --lane <lane>` — lane executor
- `fabro run <run-config>` — workflow runner
- `outputs/<lane>/review.md` — lane trust artifact
- `fabro/programs/<program>.yaml` — Raspberry program manifest
- `fabro/run-configs/<lane>.toml` — Fabro run configuration
