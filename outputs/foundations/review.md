# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP (foundations lane established)**

The foundations lane has been bootstrapped as the first honest slice for this frontier. This review assesses the current truth posture of the Myosu control plane and identifies the work required to make `execute/status/watch` truth trustworthy.

## Current Truth Posture: UNTRUSTWORTHY

The control plane currently has the following truth-integrity issues:

### Issue 1: `games:multi-game` False-Submit

**Location:** `outputs/games/multi-game/`

**Observation:** The `games:multi-game` lane directory contains only a `.gitkeep` file alongside `review.md` and `spec.md`. This pattern — artifacts claiming a reviewed milestone without corresponding execution output — is consistent with a false-submit: the lane recorded a submit without a successful live run.

**Evidence:**
```
outputs/games/multi-game/
  review.md    # claims review milestone
  spec.md      # claims spec milestone
  .gitkeep     # no actual output artifacts
```

A lane that has achieved a "reviewed" milestone should produce output artifacts (implementation records, verification evidence, etc.). The presence of only `.gitkeep` alongside milestone documents suggests the review was performed without corresponding live execution.

**Impact:** Undermines trust in all control-plane milestone decisions. If one lane can falsely claim reviewed status, the pattern may exist elsewhere.

**Required action:** Convert this false-submit to either:
- A **truthful failure** with documented root cause (preferred if execution cannot be recovered)
- A **successful live run** with verifiable execution evidence (if the lane can be re-run honestly)

### Issue 2: Absent `.raspberry/` State

**Location:** `.raspberry/` (does not exist in current worktree)

**Observation:** `fabro/programs/myosu.yaml` line 4 defines `state_path: ../../.raspberry/myosu-state.json` and `fabro/programs/myosu-bootstrap.yaml` line 4 defines `state_path: ../../.raspberry/myosu-bootstrap-state.json`. Neither the directory nor any state files exist in the current worktree.

**Evidence:**
```bash
$ ls -la .raspberry/
ls: cannot access '.raspberry/': No such file or directory
```

**Impact:** `raspberry execute` and `raspberry status` have no persistent state to read from. Any truth they report is ephemeral and cannot be audited across runs.

**Required action:** Establish `.raspberry/` as a real directory with proper state files. Verify that `raspberry execute` creates and updates state files atomically.

### Issue 3: Bootstrap Program Has No Foundations Unit

**Location:** `fabro/programs/myosu-bootstrap.yaml`

**Observation:** The bootstrap program defines three units (games, tui, chain) but no foundations unit. The lane responsible for truth-integrity has no structural home in the control plane.

**Evidence:** `fabro/programs/myosu-bootstrap.yaml` lines 1–122 define units `games`, `tui`, and `chain`. No `foundations` unit exists.

**Impact:** Foundations work has no program manifest entry, meaning it cannot be executed, tracked, or milestone-managed through the Fabro/Raspberry control plane.

**Required action:** Add a `foundations` unit to `myosu-bootstrap.yaml` once the foundations lane is ready for execution tracking.

## Honest Assessment: What Works

### Trusted Leaf Crates

The `games:traits` lane has been reviewed and is **trusted**. `myosu-games` compiles cleanly, all 14 tests pass, and the crate surface is small and well-bounded. This is a genuine keep.

### Bootstrap Structure

The Fabro/Raspberry bootstrap program structure is sound. The separation of units (games, tui, chain), lanes within units, and milestone tracking (specified → reviewed) is a correct pattern. The issue is not structural — it is that some lanes have recorded milestones without corresponding execution truth.

### Doctrine Is Clear

`SPEC.md`, `PLANS.md`, `specs/031626-00-master-index.md`, and `specs/031826-fabro-primary-executor-decision.md` provide clear, self-contained doctrine. A contributor starting from these documents and `README.md` can understand the system.

## Concrete Risks the Foundations Lane Must Address

### Risk 1: False-Submit Corrupts Milestone Trust
**Exact location:** `outputs/games/multi-game/review.md`

A reviewed milestone without live execution evidence. If not repaired, all milestone-based decisions are suspect.

**What must happen:** Honest failure record OR successful live re-run. No cosmetic fixes.

### Risk 2: Ephemeral Execute/Status Truth
**Exact location:** `.raspberry/` directory absence

Without persistent state, `raspberry execute` and `raspberry status` produce no auditable record. Each run starts from scratch.

**What must happen:** `.raspberry/` state directory created and populated by `raspberry execute`. State must be atomically written to survive crashes.

### Risk 3: No Structural Home for Truth-Integrity Work
**Exact location:** `fabro/programs/myosu-bootstrap.yaml`

Foundations lane has no unit in the bootstrap program, so it cannot be executed or milestone-tracked.

**What must happen:** Add `foundations` unit to `myosu-bootstrap.yaml` once Phase 1 is complete.

### Risk 4: Fabro Detach Path May Be Broken
**Location:** Fabro workflow cleanup mechanisms

If the Fabro detach path is broken, stale state from previous runs pollutes new executions, corrupting `execute/status/watch` truth.

**What must happen:** Verify detach cleanup. Check for stale lock files, orphaned processes, or incomplete state resets between runs.

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Foundations lane bootstrap | `ls outputs/foundations/` | `foundation-plan.md`, `review.md` |
| False-submit assessment | `ls outputs/games/multi-game/` | Honest artifact inventory |
| Raspberry state check | `ls .raspberry/` | State files after execute |
| Execute truth check | `raspberry status --manifest fabro/programs/myosu.yaml` | Matches actual execution |

## File Reference Index

| File | Role |
|------|------|
| `fabro/programs/myosu.yaml` | Repo-wide Raspberry program manifest |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program manifest (games/tui/chain units) |
| `outputs/foundations/foundation-plan.md` | Foundations lane execution plan |
| `outputs/foundations/review.md` | This file |
| `outputs/games/multi-game/review.md` | False-submit evidence source |
| `outputs/games/traits/review.md` | Reference pattern for honest reviewed lane |
| `SPEC.md` | Spec writing rules and doctrine |
| `PLANS.md` | Plan writing rules and doctrine |
| `specs/031826-fabro-primary-executor-decision.md` | Fabro-as-executor decision record |

## Next Steps

1. **Immediate:** Assess the `games:multi-game` false-submit in detail — determine if it is recoverable or must become a truthful failure
2. **Immediate:** Verify that `raspberry execute` and `raspberry status` can be run against the current bootstrap program without errors
3. **Short-term:** Add `foundations` unit to `myosu-bootstrap.yaml` once the lane structure is stable
4. **Medium-term:** Establish the Fabro detach path as a first-class concern — add a `detach` check to the bootstrap proof profile
5. **Long-term:** All lanes must produce verifiable execution evidence alongside their review artifacts
