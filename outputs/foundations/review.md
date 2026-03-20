# `foundations` Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP**

The foundations lane is correctly scoped as a meta-control-plane assessment. Its purpose is to be the honest broker of whether `execute/status/watch` truth is trustworthy. This is a legitimate lane need — the false submit of `games:multi-game` demonstrates that the control plane can report success without implementation.

The lane is unblocked. No spec work is needed — the assessment is a matter of reading existing artifacts and running existing commands.

---

## Implementation Lane Unblocked?

**Yes — unconditionally.**

The foundations lane only needs to read existing lane artifacts and run status commands. No implementation work is required.

The implementation actions it prescribes (rerun `games:multi-game`, verify detach path) are operational steps, not development steps.

---

## Honest State of All Active Lanes

| Lane | Spec? | Review Judgment | Implementation Status | Execute Truth |
|------|-------|-----------------|----------------------|--------------|
| `games:traits` | ✓ | KEEP | Bootstrap complete; Slice 1 (git deps) complete | **TRUSTWORTHY** |
| `games:multi-game` | ✓ | KEEP | **GREENFIELD** — crate doesn't exist | **UNTRUSTWORTHY** — false submit |
| `games:poker-engine` | ✓ | (not bootstrapped) | Not bootstrapped | **UNKNOWN** |
| `tui:shell` | ✓ | KEEP | Bootstrap complete | **TRUSTWORTHY** |
| `chain:pallet` | ✓ | RESTART | Does not compile | **UNTRUSTWORTHY** |
| `chain:runtime` | ✓ | RESTART | Not crate-shaped | **UNKNOWN** |

---

## The False Submit Record

### What Happened

On or around 2026-03-19, Raspberry dispatched the `games:multi-game` lane. The lane's `spec.md` and `review.md` artifacts were produced. Raspberry reported the dispatch as successful.

However, the `games:multi-game` lane has **zero implementation**:

- `crates/myosu-games-liars-dice/` does not exist
- No `CfrGame` implementation for Liar's Dice exists
- No tests for the lane exist
- The `review.md` itself documents that the crate is greenfield and lists blockers

The lane was marked with judgment KEEP in its own `review.md`, but KEEP refers to the **spec** (which is well-formed), not the **implementation** (which is empty).

### Root Cause

The Fabro detach path returned success without verifying that the lane's preconditions were met. Specifically:

1. The lane's `review.md` lists "Blocker 1: `myosu-games-liars-dice` Crate Is Entirely Greenfield (Critical)"
2. No build or test command was run for this lane before reporting success
3. The detach path returned exit code 0 because the Fabro process itself exited cleanly, not because the lane's work was done

### What Should Have Happened

Before any submit was reported:

```
# Precondition check (must pass before dispatch)
test -d crates/myosu-games-liars-dice && cargo build -p myosu-games-liars-dice
```

This would have failed (directory doesn't exist), and the submit should have reported **blocked** or **failed**, not success.

### Impact on `execute/status/watch` Truth

The `games:multi-game` false submit means that `execute/status/watch` for that lane currently shows a status that does not match reality. The lane is reported as having been acted upon, when in fact nothing was done.

This erodes trust in the control plane. Future reruns of this lane may be believed or disbelieved incorrectly.

### Required Remediation

1. **Immediate**: Mark `games:multi-game` as honestly blocked (not success)
2. **Before rerun**: Ensure precondition gates run before dispatch is attempted
3. **After fix**: Rerun the lane and verify `execute/status/watch` shows blocked

---

## The Fabro Detach Path Defect

### Description

When Raspberry dispatches a lane via Fabro in detached mode (background), the current Fabro detach path returns exit code 0 to Raspberry regardless of whether the lane's preconditions were met.

This is the mechanical root cause of the `games:multi-game` false submit.

### Required Fix

The detach path must propagate the actual pre-execution check result. If the lane is blocked:

```
# Pseudo-code for the required behavior
if lane_preconditions_met():
    fabro dispatch --detached lane
else:
    report_blocked(lane)
    exit(1)  # or some blocked indicator, but NOT 0
```

The current behavior treats any Fabro process that exits cleanly as a success, which is wrong when the Fabro process only wrote documentation without building or testing anything.

### Verification

After the fix is applied:

```bash
# Dispatch a known-blocked lane (games:multi-game has no crate)
raspberry execute --manifest fabro/programs/myosu.yaml --lane games:multi-game

# Expected: status shows "blocked" or "failed"
# NOT: "success"
raspberry status --manifest fabro/programs/myosu.yaml --lane games:multi-game
```

---

## Trust Assessment of `execute/status/watch`

### Currently Untrustworthy For

- `games:multi-game` — reports success where blocked is correct
- Any lane whose preconditions are not enforced before dispatch

### Currently Trustworthy For

- `games:traits` — real implementation, real tests, real artifacts
- `tui:shell` — real implementation, real tests, real artifacts

### Unknown

- `games:poker-engine` — not yet bootstrapped
- `chain:runtime` — not yet bootstrapped

---

## Concrete Risks the Implementation Lane Must Preserve or Reduce

### Risk 1: False Submit Residue in `games:multi-game`

**Exact location**: `outputs/games/multi-game/review.md`

The `review.md` says "Judgment: KEEP" but does not explicitly call out that the previous submit was a false success. A future reader might think the lane is further along than it is.

**What must be reduced**: Add a note to `outputs/games/multi-game/review.md` documenting the false submit and the required rerun. The `review.md` should explicitly state: "Previous Raspberry submit reported success; this was incorrect — the crate does not exist. Lane is blocked."

### Risk 2: Detach Path Returns 0 Without Verification

**Exact location**: Fabro's detach dispatch path in the Raspberry-Fabro integration

**What must be reduced**: The detach path must run preconditions before dispatch. If preconditions fail, report blocked and return non-zero.

### Risk 3: Future Lanes Could Have Same False Submit Problem

**What must be reduced**: Every lane must have a precondition check that runs before any dispatch. The check must be explicit and checked by the control plane.

---

## What Must Happen to Make `execute/status/watch` Trustworthy Again

In order of priority:

1. **Mark `games:multi-game` as blocked** — the lane spec is good, the implementation is not started. Report this honestly.

2. **Fix the Fabro detach path preconditions** — the detach path must check preconditions before reporting success. This is a Raspberry/Fabro integration fix, not a Myosu lane fix.

3. **Rerun `games:multi-game` with preconditions enforced** — after the fix, dispatch should correctly report blocked.

4. **Verify the rerun reports blocked, not success** — confirm `execute/status/watch` shows the correct status.

5. **Only then extend to other lanes** — once the mechanism is proven, apply the same pattern to all other lanes.

---

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Foundations artifacts exist | `test -f outputs/foundations/foundation-plan.md && test -f outputs/foundations/review.md` | Exit 0 |
| All lanes have spec + review | `for lane in games/traits games/multi-game chain/pallet chain/runtime tui/shell; do test -f "outputs/$lane/spec.md" && test -f "outputs/$lane/review.md"; done` | Exit 0 |
| games:traits is trustworthy | `cargo test -p myosu-games` | Exit 0, all tests pass |
| tui:shell is trustworthy | `cargo test -p myosu-tui` | Exit 0, all tests pass |
| games:multi-game is blocked | `test -d crates/myosu-games-liars-dice && cargo build -p myosu-games-liars-dice` | Exit non-0 (dir doesn't exist) |
| chain:pallet does not compile | `cargo check -p pallet-game-solver` | Exit non-0 (expected — restart lane) |
| Post-fix: games:multi-game status is blocked | `raspberry status --manifest fabro/programs/myosu.yaml --lane games:multi-game` | Shows "blocked" |

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/foundations/foundation-plan.md` | This lane's spec artifact |
| `outputs/foundations/review.md` | This file |
| `outputs/games/traits/spec.md` | Reference for trustworthy bootstrap |
| `outputs/games/traits/review.md` | Reference for trustworthy bootstrap |
| `outputs/games/multi-game/spec.md` | The falsely-submitted lane spec |
| `outputs/games/multi-game/review.md` | The falsely-submitted lane review (documents blockers but doesn't call out false submit) |
| `outputs/chain/pallet/spec.md` | Restart lane spec |
| `outputs/chain/pallet/review.md` | Restart lane review |
| `outputs/chain/runtime/spec.md` | Restart lane spec |
| `outputs/chain/runtime/review.md` | Restart lane review |
| `outputs/tui/shell/spec.md` | Trusted leaf spec |
| `outputs/tui/shell/review.md` | Trusted leaf review |
| `fabro/programs/myosu.yaml` | Top-level Raspberry program manifest |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap lanes manifest |
| `plans/031826-bootstrap-fabro-primary-executor-surface.md` | Historical context on bootstrap execution |
