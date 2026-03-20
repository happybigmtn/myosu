# Foundations Lane Plan

## Lane Boundary

The `foundations` lane is the **execution-truth and reliability foundation** for the myosu Fabro-first bootstrap. It does not ship game code, chain code, or feature work. It ships a trustworthy `execute/status/watch` signal and a repaired Fabro detach path — so that every other lane can trust what it reads from the control plane.

This lane owns:

- Establishing ground-truth for `execute/status/watch` signals by correlating them against real Myosu execution outcomes
- Discovering Raspberry/Fabro defects only through real Myosu execution runs (not through code inspection alone)
- Converting the current `games:multi-game` false-submit into a truthful outcome
- Producing honest lane artifacts (`foundation-plan.md`, `review.md`) that reflect what the evidence actually shows

This lane does **not** own:

- Implementing new game-trait, TUI, chain, or miner functionality
- Resolving defects found in other lanes before the control-plane truth is trustworthy

## Purpose / User-Visible Outcome

After this lane produces honest reviewed artifacts, a contributor can:

1. Trust that `execute/status/watch` outputs reflect real Fabro run state, not stale or fabricated signals
2. Run the `games:multi-game` lane and get a truthful pass/fail result instead of a false-submit
3. Rerun any affected frontier knowing that the control-plane truth is the same as the execution truth

## Current State

### What exists

- `fabro/programs/myosu.yaml` — the top-level program manifest with 7 frontier units and dependency graph
- `fabro/programs/myosu-bootstrap.yaml` — the bootstrap program with 3 units: `games`, `tui`, `chain`
- `outputs/games/traits/spec.md` + `review.md` — first honest reviewed slice; lane is `KEEP (with implementation lane unblocked)`
- `outputs/games/multi-game/spec.md` + `review.md` — exists but marked with a **false-submit** signal
- `fabro/workflows/implement/game-traits.fabro` — implementation workflow for `games:traits`
- `fabro/run-configs/implement/game-traits.toml` — implementation run config

### What is broken

#### Defect 1: `execute/status/watch` truth is not trustworthy

The Fabro `execute/status/watch` command reads run state from Fabro's run directory metadata. However, the current Raspberry dispatch for `games:multi-game` submits a lane result without the lane actually completing successfully — a **false-submit**. This means:

- The `status` signal says the lane succeeded
- The `watch` output shows a completion marker
- But the actual execution was not successful

Until this false-submit is repaired, **no** `execute/status/watch` output can be trusted for any lane, because the same supervisory dispatch path is used for all lanes.

#### Defect 2: `games:multi-game` false-submit

The `games:multi-game` lane in `myosu.yaml` produced a `review.md` that acknowledges the false-submit. The lane was dispatched, a result was recorded, but the actual work was not completed successfully. This is the **canary** that reveals the control-plane truth problem.

### Evidence of false-submit

- `outputs/games/multi-game/review.md` exists with a "false-submit" label
- The `games:multi-game` lane does not appear in the bootstrap trust-review corpus as "KEEP" or "RESET" like `games:traits`
- The `myosu.yaml` manifest has no `outputs/foundations/` root defined, meaning this lane has not been formally bootstrapped

## Concrete Steps

### Step 1: Produce honest `outputs/foundations/foundation-plan.md` and `outputs/foundations/review.md`

This document and its companion review are the first artifact this lane produces. They are the proof that the foundations lane exists and has been honestly assessed.

### Step 2: Add `foundations` unit to `fabro/programs/myosu.yaml`

The top-level program manifest needs a `foundations` unit that owns the two repair tasks:

```yaml
units:
  - id: foundations
    title: Foundations Frontier
    output_root: ../../.raspberry/portfolio/foundations
    lanes:
      - id: program
        kind: orchestration
        title: Foundations Program
        run_config: ../programs/myosu-foundations.yaml
        program_manifest: ../programs/myosu-foundations.yaml
        managed_milestone: coordinated
        depends_on:
          - unit: bootstrap
            lane: program
            milestone: coordinated
```

### Step 3: Create `fabro/programs/myosu-foundations.yaml`

A minimal program manifest for the foundations lane with two lanes:

- `games-multi-game-repair` — fix the false-submit, produce a truthful pass/fail
- `execute-truth` — verify that `execute/status/watch` outputs are trustworthy after the repair

### Step 4: Fix the `games:multi-game` false-submit

The repair path depends on whether the lane can produce a truthful successful result or must report a truthful failure:

- **If the lane can succeed**: run the `games:multi-game` lane through Fabro to completion, producing real artifacts under `outputs/games/multi-game/`
- **If the lane cannot succeed**: record a truthful failure in `outputs/games/multi-game/review.md`, update the manifest milestone to reflect the failure, and ensure the control plane reflects the actual state

### Step 5: Verify `execute/status/watch` truth after repair

After the `games:multi-game` repair, run `execute/status/watch` for the affected lane and confirm the control-plane signal matches the actual execution outcome. This becomes the proof that the truth path is trustworthy.

### Step 6: Rerun affected frontiers with trustworthy truth

Once `execute/status/watch` truth is established, rerun any frontier that was affected by the false-submit with the repaired control-plane path.

## Progress

- [x] (2026-03-20) Produced initial honest `foundation-plan.md` and `review.md` artifacts
- [ ] Add `foundations` unit to `fabro/programs/myosu.yaml`
- [ ] Create `fabro/programs/myosu-foundations.yaml` manifest
- [ ] Fix `games:multi-game` false-submit via truthful run or truthful failure record
- [ ] Verify `execute/status/watch` truth path after repair
- [ ] Rerun affected frontiers with repaired control-plane truth

## Decision Log

- Decision: treat the false-submit as a blocking defect rather than a lane state to work around
  Rationale: a false-submit that records success without actual success corrupts the control-plane truth for all lanes using the same dispatch path
  Date/Author: 2026-03-20 / foundations lane

- Decision: do not attempt to fix other lane defects until `execute/status/watch` truth is established
  Rationale: fixing defects without trustworthy control-plane signals means the fix cannot be verified reliably
  Date/Author: 2026-03-20 / foundations lane

## Interfaces and Dependencies

This lane depends on:

- `fabro/programs/myosu.yaml` — top-level program manifest (must accept new `foundations` unit)
- `fabro/programs/myosu-bootstrap.yaml` — bootstrap program (provides dependency baseline)
- `outputs/games/multi-game/` — the lane with the false-submit that needs repair
- `fabro/` execution plane — the Fabro engine must be able to run the repair lanes

Other lanes depend on this lane producing trustworthy `execute/status/watch` signals before they can trust their own control-plane readouts.
