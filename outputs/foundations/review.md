# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (with blocking defects to resolve)**

The `foundations` lane is correctly scoped and necessary. The core problem it addresses — false-submit artifacts implying execution that never happened — is real and must be resolved before any other lane can trust the execution truth surface.

However, the lane has **no formal program entry** in `fabro/programs/myosu.yaml`, and the Raspberry state directory (`.raspberry/`) has never been written by any Fabro run. The lane's own execution infrastructure is also unproven. This is not a design flaw — it is the expected state of a bootstrap lane. The judgment is **KEEP** because the foundations work is the right thing to do; it is **not** a reopening of a failed design.

---

## Implementation Lane Unblocked?

**No — the `foundations` lane itself is blocked on Slice 1 (Raspberry state directory creation) and Slice 3 (program manifest entry).**

The lane cannot produce honest execution truth for other lanes until its own execution infrastructure is defined and running.

---

## Current State of Execution Truth Surfaces

### What Has Honest Execution Proof

| Lane | Proof | Status |
|------|-------|--------|
| `games:traits` | `cargo test -p myosu-games` passes with 10 unit + 4 doctest | **TRUSTED** |

### What Has Artifacts But No Execution

| Lane | Artifact Location | Problem |
|------|------------------|---------|
| `games:multi-game` | `outputs/games/multi-game/` | `crates/myosu-games-liars-dice/` does not exist; lane never dispatched |
| `games:poker-engine` | `outputs/games/poker-engine/` | Not in program manifest; artifacts may be speculative |
| `tui:shell` | `outputs/tui/shell/` | Check script exists but has never been run with Raspberry tracking |
| `chain:runtime` | `outputs/chain/runtime/` | Check script exists but has never been run with Raspberry tracking |
| `chain:pallet` | `outputs/chain/pallet/` | Check script exists but has never been run with Raspberry tracking |

### What Has No Artifacts

| Lane | Problem |
|------|---------|
| `foundations` itself | No `outputs/foundations/` until this bootstrap run |

### What Has Artifacts That Need Audit

All `outputs/*/review.md` files should be reviewed for:
1. Does the lane have a corresponding entry in `fabro/programs/myosu-bootstrap.yaml`?
2. Has a Fabro run ever dispatched this lane?
3. Do the proof commands in the artifacts match the check scripts that exist?
4. If the proof commands pass, did the code actually exist at that time?

---

## Critical Defects the Foundations Lane Must Resolve

### Defect 1: `.raspberry/` State Directory Never Created

**Severity**: Critical (blocks all supervisory truth)

**Location**: `.raspberry/` does not exist.

**Manifest path**: `fabro/programs/myosu-bootstrap.yaml` line 4:
```yaml
state_path: ../../.raspberry/myosu-state.json
```

**What should happen**: Every Fabro run should write state to `../../.raspberry/myosu-state.json`. The directory has never been created.

**Impact**: Without Raspberry state, there is no cross-run execution truth. Each Fabro run is ephemeral with no surviving record of what happened.

**Resolution**: Slice 1 — create `.raspberry/` directory. Investigate why Fabro is not writing state.

---

### Defect 2: `games:multi-game` False-Submit Artifacts

**Severity**: High (produces misleading lane status)

**Location**: `outputs/games/multi-game/spec.md` and `outputs/games/multi-game/review.md`

**Problem**: These artifacts describe a lane that has never been dispatched. The `review.md` says "Judgment: KEEP" and lists blockers, implying the lane has been attempted. It has not.

**What should happen**: Either:
- The artifacts should not exist until the lane runs honestly (Option A: Reset)
- The artifacts should exist with a prominent "UNEXECUTED" banner (Option B: Annotate)

**Impact**: Contributors reading `outputs/games/multi-game/review.md` will reasonably but incorrectly conclude the lane has been attempted.

**Resolution**: Slice 2 — decision needed: Reset or Annotate?

---

### Defect 3: No `foundations` Unit in Program Manifest

**Severity**: High (lane has no formal execution entry)

**Location**: `fabro/programs/myosu.yaml` — no `foundations` unit defined.

**Problem**: The `foundations` lane being bootstrapped here has no corresponding program entry. It cannot be dispatched, tracked, or proven by the Fabro/Raspberry system.

**Impact**: The lane can produce artifacts but cannot be part of the coordinated program execution.

**Resolution**: Slice 3 — add `foundations` unit to `fabro/programs/myosu.yaml`.

---

### Defect 4: Unknown Execution Status of `tui:shell`, `chain:runtime`, `chain:pallet`

**Severity**: Medium (lane status is unknown, not known-bad)

**Location**: `fabro/run-configs/bootstrap/*.toml` and `fabro/checks/*.sh` exist, but:

- No Raspberry state exists for any bootstrap run
- No record of whether check scripts have been run
- The `outputs/tui/shell/`, `outputs/chain/runtime/`, `outputs/chain/pallet/` artifacts may reflect expected state, not actual execution state

**Impact**: These lanes appear "ready" in the program manifest but may be as unexecuted as `games:multi-game`.

**Resolution**: After Slice 1 (Raspberry state exists), run each lane and record honest results.

---

## Specific Frontend Tasks from the Goal

### Task 1: Fix Raspberry/Fabro Defects When Discovered by Real Myosu Execution

**Current known defect**: `.raspberry/` state directory never created.

**Trigger condition**: Run a real Fabro dispatch and observe that no state is written.

**Resolution**: Investigate why `state_path` is not being written. Is the Fabro run completing? Is the state write step missing from the workflow? Is the path incorrect?

### Task 2: Convert `games:multi-game` False-Submit into Truthful Failure or Live Run

**Current state**: `outputs/games/multi-game/` artifacts exist but the lane was never dispatched.

**Resolution path A (Failure)**:
1. Run `fabro run fabro/run-configs/bootstrap/multi-game.toml` (or equivalent)
2. Observe it fails because `crates/myosu-games-liars-dice/` doesn't exist
3. Record the truthful failure in `outputs/games/multi-game/review.md`
4. Delete the speculative "KEEP" judgment; replace with "PENDING — failed at first honest dispatch"

**Resolution path B (Live Run)**:
1. Implement `crates/myosu-games-liars-dice/` (greenfield work)
2. Run the lane honestly
3. Record the truthful success

**Current blocker**: There is no `fabro/run-configs/bootstrap/multi-game.toml` and no `games:multi-game` lane in the program manifest. The lane cannot even be dispatched.

---

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Raspberry state initialization | `ls -la .raspberry/` | Directory exists with `myosu-state.json` or equivalent |
| Program manifest validity | `fabro run --dry-run fabro/run-configs/bootstrap/game-traits.toml` | Validates without error |
| Lane dispatch | `fabro run fabro/run-configs/bootstrap/game-traits.toml` | Produces state in `.raspberry/` |
| Status truth check | `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` | Output reflects actual execution state, not speculative artifacts |

---

## File Reference Index

| File | Role |
|------|------|
| `fabro/programs/myosu.yaml` | Top-level Raspberry program manifest — missing `foundations` unit |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest with 4 units |
| `fabro/run-configs/bootstrap/*.toml` | Run configs for bootstrap lanes |
| `fabro/checks/*.sh` | Check scripts that produce proof signals |
| `outputs/games/traits/spec.md` | Reference pattern for lane spec structure |
| `outputs/games/traits/review.md` | Reference pattern for lane review structure |
| `outputs/games/multi-game/spec.md` | **FALSE-SUBMIT** — lane never dispatched |
| `outputs/games/multi-game/review.md` | **FALSE-SUBMIT** — claims KEEP judgment for undispatched lane |
| `outputs/foundations/spec.md` | This lane's spec artifact |
| `outputs/foundations/review.md` | This file |

---

## Risks the Implementation Lane Must Preserve

1. **Raspberry state file format**: The JSON state format must remain compatible with what `raspberry status` expects. Changing the format breaks the control plane.

2. **Check script exit codes**: Check scripts must exit 0 for success, non-zero for failure. Inverting this breaks all lane status.

3. **Program manifest schema**: The YAML schema for `fabro/programs/myosu.yaml` is contract. Adding lanes incorrectly can break the entire program.

---

## Risks the Implementation Lane Should Reduce

1. **False-submit contagion**: If `games:multi-game` is a false-submit, other lanes may be too. A systematic audit of all `outputs/*/review.md` files should check: (a) is the lane in the program manifest? (b) has it been dispatched? (c) do the proof commands match existing check scripts?

2. **Ephemeral execution**: Without `.raspberry/` state, all Fabro runs are ephemeral. This makes debuggingLane status retrospective impossible. The state directory is the minimum viable durable execution record.

---

## Is the Lane Ready for an Implementation-Family Workflow Next?

**No — the `foundations` lane must complete its own bootstrap first.**

The conditions for proceeding to implementation:

1. Slice 1 (`.raspberry/` state directory) must complete
2. Slice 3 (`foundations` unit in program manifest) must complete
3. At least one honest Fabro run must produce Raspberry state
4. `raspberry status` must show truthful lane status

Without these, any implementation lane that claims to have "run" is also a false-submit candidate.

---

## Decision Log

- **Decision**: The `foundations` lane must be formally defined in `fabro/programs/myosu.yaml` before it can be tracked by Raspberry.
  **Rationale**: Without a program entry, the lane cannot be dispatched, verified, or proven by the Fabro/Raspberry system. Artifacts alone are not execution truth.
  **Date**: 2026-03-20

- **Decision**: `games:multi-game` artifacts are a false-submit and must be resolved before the lane is considered complete.
  **Rationale**: Artifacts implying execution that never happened are worse than no artifacts — they create false confidence. The lane should either show a truthful failure or a truthful success, not a speculative "KEEP" judgment.
  **Date**: 2026-03-20

- **Decision**: `.raspberry/` state directory is the minimum viable durable execution record for Raspberry.
  **Rationale**: Without cross-run state, each Fabro run is ephemeral and undebuggable. The state directory is the simplest durable record that `raspberry status` can read.
  **Date**: 2026-03-20
