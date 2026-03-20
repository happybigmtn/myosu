# Foundations Lane Review

## Keep / Reopen / Reset Judgment

**Judgment: KEEP — foundations lane is necessary and correctly scoped**

The foundations lane is the first honest slice of the myosu control plane. Without
it, every other lane's completion claims rest on a false foundation. The lane
correctly identifies the `games:multi-game` false-submit as the primary evidence
of a broken pattern and prescribes either truthful failure or live implementation
as the only honest resolutions.

---

## Is the Lane Ready for Execution?

**Yes — and it must run before any other new lane is approved.**

The false-submit pattern is not an abstract concern. There is a concrete,
curated `review.md` artifact (`outputs/games/multi-game/review.md`) that declares
"Judgment: KEEP" and "Implementation lane unblocked: Yes" for a lane whose
implementing crate does not exist. This is a direct violation of:

- **INV-001 (Structured Closure Honesty)**: No dispatched turn may be treated as
  complete unless it ends in a trusted structured outcome. The `games:multi-game`
  lane was never dispatched (no Fabro run exists for it), yet its curated
  artifacts claim milestone progress.

- **INV-002 (Proof Honesty)**: Named proof commands must actually execute and
  must never be used as false-green placeholders. The `games:multi-game` review
  contains proof commands that cannot run because the code does not exist.

---

## Current Broken Surfaces

### Critical: `games:multi-game` Has Artifacts But No Code

**Location**: `outputs/games/multi-game/review.md` + `outputs/games/multi-game/spec.md`

The `games:multi-game` lane produced a full `review.md` that:
- Rates itself "KEEP" with "Implementation lane unblocked: Yes"
- Documents proof commands that require `cargo test -p myosu-games-liars-dice`
- But `crates/myosu-games-liars-dice/` does not exist anywhere in the workspace

**Impact**: Any consumer of `outputs/games/multi-game/review.md` who trusts its
"unblocked" judgment will attempt to build on a foundation that does not exist.
This is the definition of a false-green proof.

**What must happen**: Either (a) implement the `myosu-games-liars-dice` crate
before the `games:multi-game` lane can be considered unblocked, or (b) replace
the false "KEEP" judgment with a truthful "CLOSED-FAILED" judgment preserving the
evidence.

### Critical: `games:multi-game` Not in Bootstrap Program Manifest

**Location**: `fabro/programs/myosu-bootstrap.yaml`

The `games:multi-game` lane has curated artifacts but is not registered in the
active Raspberry program manifest. This means:
- Raspberry cannot track its milestones
- No `execute/status/watch` surface knows about it
- Any "progress" reported is invisible to the control plane

**What must happen**: Either add `games:multi-game` to `myosu-bootstrap.yaml`
with honest proof gates, or explicitly exclude it from the bootstrap program
with a closure artifact.

### High: `execute/status/watch` Truth Not Verified

**Location**: Unknown — the Fabro run state and Raspberry state have not been
audited against worktree ground truth

The bootstrap program has 4 active lanes. No one has verified that:
- `raspberry status` output matches what actually exists in the worktree
- Fabro run history accurately reflects what was executed
- No phantom milestones are reported

**What must happen**: Audit all 4 current lanes against worktree ground truth
before adding any new lanes.

---

## Honest Disposition Decision Required

The `games:multi-game` lane has two honest paths:

### Path A: Truthful Failure (close-failed)

**If this path is chosen:**
- Replace `outputs/games/multi-game/review.md` with a "CLOSED-FAILED" judgment
- Document exactly what was promised (7 slices, 22 test commands, zero-change
  architectural claim) vs. what exists (nothing)
- Preserve the spec and review as failure evidence
- Mark `games:multi-game` as blocked on: implementation resources
- The lane can be reopened when implementation resources are available

**Evidence preserved:**
```bash
# Ground truth:
ls crates/ | grep liars-dice
# Returns: (empty)

# But the review claims:
# "The implementation lane is unblocked. The lane has no critical design ambiguity."
# This is false.
```

### Path B: Live Implementation (honest success)

**If this path is chosen:**
- Add `games:multi-game` to `myosu-bootstrap.yaml` with honest proof gates:
  - Precondition: `crates/myosu-games-liars-dice/src/lib.rs` must exist
  - Proof gate 1: `cargo build -p myosu-games-liars-dice` exits 0
  - Proof gate 2: `cargo test -p myosu-games-liars-dice` exits 0
- Update `outputs/games/multi-game/review.md` to note the lane is now active
  with honest gates, not a false "KEEP"
- Run the lane through Slice 1 (crate skeleton) to prove the pattern works

**This path must not:**
- Claim "KEEP" or "unblocked" until the crate actually exists
- Run proof commands that cannot possibly pass

---

## Specific Defects to Fix in `execute/status/watch`

Before the foundations lane is complete, the following must be verified:

| Lane | What `raspberry status` should report | Ground truth check |
|------|--------------------------------------|-------------------|
| `games:traits` | `games:traits` lane: reviewed ✓ | `cargo test -p myosu-games` passes |
| `tui:shell` | `tui:shell` lane: reviewed ✓ | `crates/myosu-tui/src/shell.rs` exists |
| `chain:runtime` | `chain:runtime` lane: reviewed ✓ | `crates/myosu-chain/runtime/src/lib.rs` exists |
| `chain:pallet` | `chain:pallet` lane: reviewed ✓ | `crates/myosu-chain/pallets/game-solver/src/lib.rs` exists |
| `games:multi-game` | NOT IN MANIFEST or CLOSED-FAILED | `crates/myosu-games-liars-dice/` absent |

Any discrepancy between `raspberry status` and the ground truth check column
is a defect in the `execute/status/watch` surfaces that must be repaired.

---

## Risks the Foundations Lane Must Preserve

1. **INV-001 (Structured Closure Honesty)**: No future lane may have artifacts
   without corresponding code. The foundations lane must establish the pattern
   that proof gates run BEFORE artifacts are created.

2. **INV-002 (Proof Honesty)**: Proof commands must be runnable and must actually
   run. The foundations lane must not approve any lane with proof commands that
   cannot execute due to missing code.

3. **Doctrine hierarchy** (from `SPEC.md`): `outputs/` is the first durable
   control-plane artifact surface. Its integrity is paramount.

---

## Risks the Foundations Lane Should Reduce

1. **No verification of existing lanes**: The 4 bootstrap lanes have never been
   audited against worktree ground truth. This lane should perform that audit
   and repair any discrepancies.

2. **No `games:multi-game` in manifest**: The lane exists as artifacts but is
   invisible to Raspberry. This creates a zombie lane that confuses future
   contributors.

3. **No precedent for honest failure**: If this repo has a pattern of closing
   lanes truthfully (not hiding failures), the foundations lane should establish
   that precedent explicitly.

---

## Required Evidence Before Foundation Lane Is Complete

1. [ ] `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` output
      captured and compared against worktree ground truth
2. [ ] All 4 current lanes verified as truthfully reported (no phantom milestones)
3. [ ] `games:multi-game` disposition resolved (Path A or Path B above)
4. [ ] If Path A: `outputs/games/multi-game/CLOSED-FAILED.md` exists with evidence
5. [ ] If Path B: `fabro/programs/myosu-bootstrap.yaml` updated with honest gates
6. [ ] `outputs/foundations/review.md` (this file) has final judgment posted

---

## File Reference Index

| File | Role |
|------|------|
| `fabro/programs/myosu-bootstrap.yaml` | Raspberry program manifest — must be updated with honest gates |
| `outputs/games/multi-game/review.md` | False-submit artifact to resolve |
| `outputs/games/multi-game/spec.md` | Spec companion to false-submit artifact |
| `outputs/foundations/foundation-plan.md` | This lane's plan artifact |
| `outputs/foundations/review.md` | This file |
| `INVARIANTS.md` | INV-001 and INV-002 define the honest execution contract |
| `SPEC.md` | Doctrine for spec authoring |
| `PLANS.md` | Doctrine for plan authoring |
| `crates/` | Worktree ground truth for what code actually exists |
