# Foundations Lane — Foundation Plan

This ExecPlan is a living document. The sections `Progress`,
`Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must
be kept up to date as work proceeds.

`PLANS.md` is checked into the repository root and this document must be
maintained in accordance with it.

## Purpose / Big Picture

The `foundations` lane owns the execution machinery that all other lanes depend
on. After this lane lands, the Fabro-to-Raspberry dispatch and status truth
surfaces produce signals that can be trusted — not just signals that appear to
succeed. A contributor running `fabro run` followed by `raspberry status` gets
an honest account of what actually happened, not a false-positive completion
report.

The user-visible outcome is a trustworthy `execute/status/watch` loop for all
subsequent lane work: when a lane fails, it is reported as a failure; when it
succeeds, the success is real and verifiable.

## Progress

- [ ] (2026-03-20) Write `outputs/foundations/foundation-plan.md` (this document)
- [ ] (2026-03-20) Write `outputs/foundations/review.md`
- [ ] Establish honest baseline: document what `execute/status/watch` currently
  reports vs. what is actually true for each active lane
- [ ] Identify the Fabro detach path defect that produces false-submit signals
- [ ] Fix the defect or convert false-submit into a truthful failure
- [ ] Rerun affected frontier with repaired Fabro detach path
- [ ] Verify `execute/status/watch` truth matches observable reality

## Surprises & Discoveries

_Discovered during this slice._

## Decision Log

- Decision: foundations lane focuses on execution machinery truth, not new
  feature development.
  Rationale: the current execute/status/watch signals are untrustworthy, which
  undermines all subsequent lane work. Fixing truth is a prerequisite to any
  meaningful progress reporting.
  Date/Author: 2026-03-20 / foundations lane

- Decision: fix defects only when they are discovered by real Myosu execution,
  not speculatively.
  Rationale: the bootstrap lanes have already established which surfaces are
  trustworthy. The remaining defects must surface through actual execution before
  they are addressed, to avoid over-engineering.
  Date/Author: 2026-03-20 / foundations lane

## Outcomes & Retrospective

_To be written after the lane produces honest execution truth._

## Context and Orientation

### What This Lane Is

`foundations` is the meta-lane that owns the execution and control plane
machinery: Fabro run dispatch, Raspberry status rendering, and the detach
path between them. All other lanes (`games:traits`, `tui:shell`,
`chain:runtime`, `chain:pallet`, `games:multi-game`, etc.) depend on this
machinery being trustworthy.

### What This Lane Is Not

`foundations` is not an implementation lane. It does not own any product
crate (`myosu-games`, `myosu-tui`, `pallet-game-solver`, etc.). It does not
own the chain fork restart work — that belongs to `chain:runtime` and
`chain:pallet`. It does not own the Liar's Dice implementation — that belongs
to `games:multi-game`.

### Current State

The Fabro/Raspberry execution machinery has the following known problem:

The `games:multi-game` lane produced a **false-submit**: Raspberry dispatched a
lane run that Fabro reported as complete, but the lane did not actually execute
honestly. This means the Fabro detach path — the mechanism by which Raspberry
hands off to Fabro and receives completion signals — is not producing truthful
status.

The downstream impact is that `execute/status/watch` truth cannot be trusted
until the detach path is repaired.

### Key Files

| File | Role |
|------|------|
| `fabro/programs/myosu.yaml` | Top-level program manifest; `foundations` is a unit here |
| `fabro/programs/myosu-bootstrap.yaml` | Bootstrap program with `games:traits`, `tui:shell`, `chain:runtime`, `chain:pallet` lanes |
| `crates/myosu-games/Cargo.toml` | Git-pinned robopoker dep; source of `games:traits` trust |
| `crates/myosu-tui/src/shell.rs` | TUI shell source |
| `crates/myosu-chain/pallets/game-solver/src/lib.rs` | Pallet source; currently does not build |

### Active Lane Trust Inventory

| Lane | Status | Evidence |
|------|--------|----------|
| `games:traits` | **TRUSTED** | `cargo test -p myosu-games` passes; spec + review artifacts exist |
| `tui:shell` | **PARTIALLY TRUSTED** | Unit tests pass; schema, events, shell integration have high-severity proof gaps |
| `chain:runtime` | **RESTART REQUIRED** | No buildable runtime; workspace path commented out |
| `chain:pallet` | **RESTART REQUIRED** | `cargo check -p pallet-game-solver` fails with 50+ errors |
| `games:multi-game` | **UNTRUSTworthy** | False-submit from Raspberry dispatch; `games:multi-game` crate does not exist |
| `foundations` | **THIS LANE** | No artifacts yet |

## Plan of Work

### Step 1: Produce Honest `review.md`

Before attempting any fixes, document the current state of the execution
machinery with absolute honesty. This is the baseline against which all future
progress is measured.

### Step 2: Establish the Honest Baseline

Run `fabro run` for each active lane and compare what Fabro reports against
what actually happened. The delta is the false-positive rate. Record:

- Which lanes produce false-positive completion reports
- Which lanes produce false-negative failure reports
- Whether `raspberry status` reflects actual Fabro run state

### Step 3: Identify the Fabro Detach Path Defect

The `games:multi-game` false-submit is the primary evidence of a detach path
problem. Trace the handoff from Raspberry dispatch through Fabro completion
signal. Find the specific step where a false signal is produced.

### Step 4: Fix or Truthfully Report

If the defect is in the Fabro machinery: fix it directly.
If the defect is in the Raspberry rendering: fix it there.
If the lane itself is not ready (e.g., `games:multi-game` crate does not
exist): convert the false-submit into a truthful failure signal, not a silent
success.

### Step 5: Rerun the Affected Frontier

After the fix or truthful failure conversion, rerun the `games:multi-game`
lane with the repaired Fabro detach path. Verify that `execute/status/watch`
now reports truthfully.

## Concrete Steps

### Step 1: Verify Current `games:multi-game` State

```bash
# Check whether the myosu-games-liars-dice crate exists
ls crates/myosu-games-liars-dice/ 2>/dev/null && echo "EXISTS" || echo "MISSING"

# Check whether the lane was dispatched
raspberry status --manifest fabro/programs/myosu.yaml 2>&1 | grep -A5 "multi-game"

# Check Fabro run history for this lane
fabro inspect --recent 2>&1 | grep -i "multi-game"
```

### Step 2: Establish `execute/status/watch` Baseline

```bash
# For each bootstrap lane, compare Fabro report vs. actual state
for lane in games:traits tui:shell chain:runtime chain:pallet; do
  echo "=== $lane ==="
  raspberry status --manifest fabro/programs/myosu-bootstrap.yaml 2>&1 | grep "$lane"
  # Compare against what actually happened:
  # - Did tests pass?
  # - Did the build succeed?
  # - Are artifacts real files?
done
```

### Step 3: Fix the Detach Path

The fix depends on Step 2's findings. If the detach path is producing
false-positive completions:

1. Find the Raspberry code that renders Fabro completion signals
2. Find the Fabro code that emits those signals
3. Ensure the signal accurately reflects the underlying run state

If the lane is simply not ready:

1. Update the lane check to fail honestly when preconditions are not met
2. Do not allow a lane to report "complete" when its preconditions are absent

### Step 4: Rerun with Truthful Signals

```bash
# After the fix, rerun the affected lane
fabro run fabro/run-configs/bootstrap/game-traits.toml
raspberry status --manifest fabro/programs/myosu-bootstrap.yaml

# Verify: status should match actual run state
# - Success should mean success (tests passed, artifacts exist)
# - Failure should mean failure (tests failed or preconditions absent)
```

## Validation and Acceptance

The lane is complete when:

1. `outputs/foundations/foundation-plan.md` exists and is honest about current
   state and next steps
2. `outputs/foundations/review.md` exists and accurately characterizes the
   execution machinery's current truthfulness
3. The `games:multi-game` lane either:
   - Runs successfully (if the crate exists and the lane is truly ready), OR
   - Fails with an honest, specific error (if the crate is missing or
     preconditions are not met)
4. `execute/status/watch` truth matches observable reality for all bootstrap
   lanes
5. No lane produces a false-positive completion report

## Idempotence and Recovery

If a step fails, do not proceed to the next step until the failure is
understood. If the fix breaks another lane's status rendering, revert and
re-diagnose before attempting a second fix.

All changes to the execution machinery should be tested against at least two
lanes before being treated as trustworthy.
