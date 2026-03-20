# Foundations Lane Review

**Lane**: `foundations`
**Date**: 2026-03-20
**Status**: Bootstrap — no prior artifacts

---

## Judgment Summary

The `foundations` lane has not previously existed as a reviewed artifact surface.
This review establishes the honest baseline for the lane's scope and the current
state of the execution machinery it owns.

| Surface | Status | Rationale |
|---------|--------|-----------|
| Fabro dispatch + detach path | **UNTRUSTWORTHY** | `games:multi-game` produced a false-submit; signals do not match reality |
| `raspberry status` rendering | **UNCERTAIN** | Depends on Fabro signals; may render false positives as completions |
| `execute/status/watch` truth | **UNVERIFIED** | No systematic comparison of reported vs. actual state exists |
| Bootstrap lane artifact contracts | **TRUSTED (partial)** | `games:traits` has honest spec+review; `chain:runtime` and `chain:pallet` have honest restart reviews; `tui:shell` has honest review with reopen items |

---

## Detailed Assessments

### Fabro Dispatch and Detach Path — UNTRUSTWORTHY

**Evidence of untrustworthiness**: The `games:multi-game` lane produced a
false-submit — Raspberry dispatched a run that Fabro reported as complete, but
the lane did not actually execute honestly. The root cause is that the Fabro
detach path (the mechanism by which Raspberry hands off to Fabro and receives
completion signals) is not producing truthful status.

**Specific problem**: The `games:multi-game` lane cannot produce a truthful
completion because `crates/myosu-games-liars-dice/` does not exist. A lane
whose primary deliverable is an entire non-existent crate cannot succeed. Yet
the dispatch mechanism reported completion rather than failure.

**What must be preserved**: The Fabro run infrastructure itself (`fabro run`,
`fabro inspect`, `fabro/workflow` engine) is sound. The problem is in the
Raspberry-to-Fabro handoff and status rendering layer.

**What must be fixed**: The detach path must not emit a completion signal when
the dispatched lane has unmet preconditions. Preconditions for `games:multi-game`
include:
- `crates/myosu-games-liars-dice/` must exist and compile
- All upstream dependencies must be satisfied

### `raspberry status` Rendering — UNCERTAIN

**Current behavior**: `raspberry status` renders Fabro run state. It
depends entirely on Fabro's signals being truthful. If Fabro emits a
completion signal for a lane that did not actually complete, `raspberry status`
will render a false positive.

**What must be preserved**: The Raspberry status rendering itself (table
format, milestone display, artifact path resolution) is correct. The problem is
input-side: it receives false signals from Fabro.

**What must be fixed**: Either fix Fabro to not emit false completion signals,
or add a Raspberry-side validation gate that checks observable reality before
accepting a completion signal.

### Bootstrap Lane Trust Inventory — HONEST PARTIAL Picture

| Lane | Artifact State | Observable State | Match? |
|------|---------------|-----------------|--------|
| `games:traits` | `spec.md` ✓, `review.md` ✓, `implementation.md` ✓, `verification.md` ✓ | `cargo test -p myosu-games` passes | **YES** |
| `tui:shell` | `spec.md` ✓, `review.md` ✓ | Unit tests pass; schema/events/shell have high-severity reopen items | **PARTIAL** |
| `chain:runtime` | `spec.md` ✓, `review.md` ✓ (both say RESTART) | `cargo check -p myosu-runtime` fails | **YES** (both say broken) |
| `chain:pallet` | `spec.md` ✓, `review.md` ✓ (both say RESTART) | `cargo check -p pallet-game-solver` fails | **YES** (both say broken) |
| `games:multi-game` | `spec.md` ✓, `review.md` ✓ (says KEEP with blockers) | `crates/myosu-games-liars-dice/` does not exist | **FALSE-POSITIVE** (artifacts exist but lane has not run) |

**Key finding**: `games:multi-game` has honest `spec.md` and `review.md`
artifacts that correctly identify the blockers. However, those artifacts were
written without a real lane execution having been attempted. The false-submit
indicates that a dispatch was recorded without the lane actually running.

### `games:multi-game` False-Submit — Specific Analysis

**What happened**: A Raspberry dispatch for `games:multi-game` produced a
completion signal even though `crates/myosu-games-liars-dice/` does not exist.

**Root cause hypothesis**: The dispatch reported success based on a precondition
check that passed (e.g., the run config existed and was syntactically valid)
rather than on the actual lane outcome (e.g., the crate building and tests
passing).

**What the review.md for `games:multi-game` says**: "The lane is correctly
scoped." with blockers: the crate does not exist, `CfrGame: Copy` constraint
may be a problem, `ExploitMetric` not in `myosu-games`, and `SpectatorRelay`
missing. The review is correct about the blockers but does not address the
false-submit issue directly.

**Required fix**: The lane must either:
1. Fail honestly when preconditions are unmet (crate missing = fail, not success), OR
2. Be updated so preconditions are actually satisfiable before dispatch is attempted

---

## The Two Foundational Tasks

### Task 1: Fix `execute/status/watch` Truth

**Goal**: `raspberry status` and `raspberry execute` must report what actually
happened, not what the dispatch mechanism wishes had happened.

**Current gap**: No systematic verification that reported state matches
observable state. The false-submit on `games:multi-game` is the most visible
symptom, but the gap exists for any lane whose status is rendered from
dispatch rather than from observable proof.

**Fix approach**:
1. Add a Raspberry-side post-run verification step: after Fabro reports
   completion, verify the observable outcomes (artifacts exist, tests passed,
   build succeeded) before accepting the completion signal
2. If verification fails, render the lane as failed with a specific reason,
   not as succeeded

**Idempotence**: The fix must not break lanes that already report honestly.
Test against at least two lanes (`games:traits` and `chain:pallet`) before
treating the fix as trustworthy.

### Task 2: Convert `games:multi-game` False-Submit to Truthful Outcome

**Goal**: The `games:multi-game` lane either runs successfully or fails with a
specific, honest error message.

**Current state**: The lane was dispatched. A completion signal was emitted.
The crate does not exist. The lane did not actually run.

**Required actions**:
1. Do not re-dispatch `games:multi-game` until `crates/myosu-games-liars-dice/`
   exists and is buildable — dispatching now produces only another false signal
2. The `games:multi-game` lane is blocked on:
   - `myosu-games-liars-dice` crate creation (Slice 1 of the multi-game plan)
   - `ExploitMetric` addition to `myosu-games` (Slice 4)
   - `SpectatorRelay` creation in `myosu-play` (Slice 5)
3. Once those preconditions are met, re-dispatch and verify honest completion

---

## What the `foundations` Lane Must Own

The `foundations` lane is responsible for the execution machinery contract:

1. **Dispatch honesty**: A dispatched lane must not produce a false-positive
   completion signal when preconditions are unmet.

2. **Status rendering**: `raspberry status` must reflect observable reality,
   not dispatched intent.

3. **Watch truth**: `raspberry watch` must stream accurate real-time signals,
   not stale or fabricated progress.

4. **Execute fidelity**: `raspberry execute` must produce an outcome that
   matches what actually happened, with specific failure reasons when applicable.

---

## What the `foundations` Lane Must Not Own

The `foundations` lane must not own:

- Any product crate implementation (`myosu-games`, `myosu-tui`,
  `pallet-game-solver`, etc.)
- The chain fork restart work (`chain:runtime`, `chain:pallet`)
- The Liar's Dice implementation (`games:multi-game` slices)
- The TUI reopen items (schema, events, shell integration)

Those are owned by their respective lanes.

---

## Recommended Path Forward

### Phase 1: Establish Truth Baseline

1. Run `fabro run` for each active lane and record what Fabro reports vs.
   what actually happened
2. Produce a truth comparison table documenting which lanes produce
   false-positive or false-negative signals
3. Fix the Raspberry-side validation gate to check observable outcomes before
   accepting completion signals

### Phase 2: Fix the Detach Path

1. Identify the specific step in the Raspberry-to-Fabro handoff where false
   completion signals are emitted
2. Fix or add a guard that prevents false signals from propagating
3. Verify the fix against at least two lanes

### Phase 3: Re-run `games:multi-game` with Truth

1. Only after Phase 2 is verified, re-dispatch `games:multi-game`
2. Verify that the lane either succeeds (if preconditions are met) or fails
   with a specific honest error (if preconditions are still unmet)
3. Do not allow a dispatch that produces a false signal

---

## Verdict

**Restart from honest baseline.** The `foundations` lane has never produced
reviewed artifacts. The current execution machinery cannot be trusted to produce
truthful signals. The `games:multi-game` false-submit is the most visible
symptom, but the underlying problem is systemic: the detach path does not
validate observable outcomes before emitting completion signals.

The `foundations` lane is unblocked. The path forward is honest diagnosis,
then honest repair, then honest re-run.
