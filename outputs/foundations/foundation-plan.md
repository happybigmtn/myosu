# `foundations` Lane Spec

## Lane Boundary

`foundations` is the **meta-control-plane lane** for the myosu frontier. It does not own a crate, workflow, or executable surface — it owns the honest reviewed slice of the entire control plane.

Its purpose is to be the first lane that evaluates whether `execute/status/watch` truth is trustworthy, and to drive the frontier to a state where honest execution results can be relied upon.

`foundations` owns:

- The honest assessment of all active lane specs and their implementation status
- The specific defect record for Raspberry-dispatched `games:multi-game` false-submit
- The defect record for the Fabro detach path
- The definition of "truthful failure" vs "false success" for lane execution
- The requirements for `execute/status/watch` truth to be trustworthy

`foundations` does **not** own:

- Any crate implementation (that belongs to implementation lanes)
- Any workflow definition (that belongs to the execution plane)
- Any individual lane spec (each lane owns its own spec)

---

## Platform-Facing Purpose

The foundations lane delivers the answer to: **"Can we trust what `execute/status/watch` says about our lanes?"**

The user-visible outcomes are:

- An honest record of which lanes are truly complete, truly failed, or still greenfield
- A specific defect list for the Raspberry/Fabro execution defects that caused false submits
- A rerun plan that converts the `games:multi-game` false-submit into either a truthful failure (crate doesn't exist, lane correctly marked blocked) or a successful live run (crate gets built)
- A commitment to rerun affected frontiers only after real Myosu execution discovers real defects

---

## Current Lane Inventory

### `games:traits` — TRUSTED BOOTSTRAPPED LANE

| Signal | Value |
|--------|-------|
| Status | Bootstrap complete; implementation lane complete (Slice 1) |
| Tests | 10 unit + 4 doctest, all passing |
| Artifacts | `spec.md`, `review.md`, `implementation.md`, `verification.md` |
| Crate | `crates/myosu-games` — compiles, tests pass, git deps in place |
| Blocking issues | Path deps replaced with git deps; edition 2024 remains |

**Judgment: TRUSTED. No action needed in foundations lane.**

---

### `games:multi-game` — FALSE SUBMIT (CRITICAL)

| Signal | Value |
|--------|-------|
| Status | Spec exists; implementation is greenfield; Raspberry reported submit success |
| Artifacts | `spec.md` (well-formed), `review.md` (judgment: KEEP) |
| Problem | The lane's own `review.md` says `crates/myosu-games-liars-dice/` does not exist. Raspberry reported this lane as "submitted" despite no crate, no tests, and no implementation. |
| Root cause | Fabro detach path returned success without verifying the lane's preconditions |

**Judgment: RESET TO HONEST FAILURE. The lane spec is correct; the execution was dishonest.**

The `review.md` for `games:multi-game` already contains the correct spec and the correct blockers. The problem is that Raspberry/Fabro reported submit success when the only thing that happened was the spec was written — no code was built, no tests ran.

The correct behavior: when a lane's preconditions (crate exists, tests exist) are not met, `execute/status/watch` should report the lane as **blocked** or **failed**, not **success**.

---

### `chain:pallet` — RESTART LANE

| Signal | Value |
|--------|-------|
| Status | Spec exists; `pallet-game-solver` is in workspace but does not compile |
| Artifacts | `spec.md`, `review.md` |
| Problem | Non-building transplant from subtensor; CF-01 (strip pallets) is the first commit |
| Tests | Does not compile; no test surface |

**Judgment: RESTART. Blocked on CF-01 (strip drand/crowdloan supertraits). Do not treat as trusted.**

---

### `chain:runtime` — RESTART LANE

| Signal | Value |
|--------|-------|
| Status | Spec exists; `crates/myosu-chain/runtime/` and `crates/myosu-chain/node/` are not crates yet |
| Artifacts | `spec.md`, `review.md` |
| Problem | Source trees exist but no `Cargo.toml` manifests; workspace only includes pallet |

**Judgment: RESTART. Not crate-shaped yet. Depends on `chain:pallet` completing first.**

---

### `tui:shell` — TRUSTED LEAF

| Signal | Value |
|--------|-------|
| Status | Bootstrap complete; `cargo test -p myosu-tui` passes |
| Artifacts | `spec.md`, `review.md` |
| Crate | `crates/myosu-tui` — compiles, tests pass |

**Judgment: TRUSTED. No action needed in foundations lane.**

---

## How Surfaces Fit Together

```
fabro/
  workflows/
  run-configs/
  prompts/
  checks/
  programs/
    myosu.yaml          # Top-level program manifest
    myosu-bootstrap.yaml  # Bootstrap lanes
    myosu-chain-core.yaml
    myosu-games-traits-implementation.yaml

outputs/
  foundations/          # THIS LANE: meta-control-plane assessment
    foundation-plan.md   # (this file)
    review.md           # Honest judgment of all lanes
  games/
    traits/             # TRUSTED ✓
    multi-game/         # FALSE SUBMIT ✗
    poker-engine/       # (spec only, not yet bootstrapped)
  chain/
    pallet/             # RESTART
    runtime/            # RESTART
  tui/
    shell/              # TRUSTED ✓
```

The foundations lane sits **above** all other lanes. Its inputs are:
- Each lane's `spec.md` and `review.md`
- The actual Fabro/Raspberry execution state
- The actual crate state (what compiles, what tests pass)

Its outputs are:
- An honest assessment of execution truth
- A rerun plan for false-submit lanes

---

## The Specific Defects

### Defect 1: `games:multi-game` False Submit

**What happened**: Raspberry reported a `games:multi-game` lane submit as successful. The `review.md` for that lane was written and marked KEEP. But the lane has zero implementation — the crate `crates/myosu-games-liars-dice/` does not exist, no tests exist, no code was written.

**What should have happened**: The lane's precondition check (crate exists, compiles) should have failed before any submit was attempted. The Fabro detach path should not have returned success when the only thing that happened was a documentation edit.

**Evidence**: `outputs/games/multi-game/review.md` line 4: "Judgment: KEEP" — but the lane's own spec says the crate does not exist.

**Impact**: `execute/status/watch` truth is now untrustworthy for this lane. Future reruns may be believed or disbelieved incorrectly.

---

### Defect 2: Fabro Detach Path Returns Success Without Verification

**What happened**: When Raspberry dispatches a lane via Fabro and the Fabro process detaches (background), the detach path currently returns exit code 0 without verifying that the lane's preconditions were met.

**What should happen**: The detach path should propagate the actual pre-execution check result, not a synthetic success. If the lane is blocked (preconditions not met), the submit should report blocked, not success.

**Impact**: Any lane whose preconditions are not met can be falsely reported as successful if Raspberry dispatches it.

---

## Requirements for Trustworthy `execute/status/watch`

Before any lane rerun is considered trustworthy:

1. **Precondition gates must execute before dispatch**: `cargo build`, `cargo test`, or equivalent checks must run and pass before a lane is marked as dispatched
2. **Detach path must propagate real exit codes**: The Fabro detach path must not return 0 when the actual work was a documentation-only change with no build/test
3. **Blocked lanes must report as blocked**: If a lane's preconditions are not met, `execute/status/watch` must show "blocked" or "failed", never "success"
4. **Post-execution verification must be mandatory**: The `verification.md` artifact must exist and show real proof before a lane is marked complete

---

## Concrete Slices

### Slice 1 — Honest Assessment of All Active Lanes

For each lane in `outputs/`, read the `spec.md` and `review.md` and determine:

- Is the lane truly bootstrapped (artifacts exist, tests pass)?
- Is the lane greenfield (spec exists, no implementation)?
- Is the lane blocked (spec exists, implementation attempted, blocked on dependency)?
- Is the lane falsely submitted (Raspberry reported success, implementation doesn't exist)?

Produce a table in `review.md` documenting the honest state of every lane.

**Proof**: All lanes have accurate status in `review.md`.

---

### Slice 2 — Document the `games:multi-game` False Submit

Write a detailed account of what happened, what the correct behavior should have been, and what the rerun plan is.

The rerun plan:
1. Re-run the `games:multi-game` lane with preconditions enforced
2. Precondition check: `test -d crates/myosu-games-liars-dice && cargo build -p myosu-games-liars-dice`
3. Expected result: truthful failure — the crate doesn't exist, so the lane is blocked
4. Once the implementation lane creates the crate and it compiles, re-run to success

**Proof**: After rerun, `execute/status/watch` shows "blocked" for `games:multi-game`, not "success".

---

### Slice 3 — Verify Fabro Detach Path Fix

Run a test dispatch of a known-blocked lane through the Fabro detach path and verify that the result correctly reports as blocked, not successful.

**Proof**: `raspberry execute --manifest fabro/programs/myosu.yaml --lane games:multi-game` returns blocked status.

---

## Proof / Check Shape

### Bootstrap Proof (foundations lane integrity)

```bash
# Foundations lane itself must have correct artifacts
test -f outputs/foundations/foundation-plan.md
test -f outputs/foundations/review.md

# All other lanes have spec + review
for lane in games/traits games/multi-game chain/pallet chain/runtime tui/shell; do
  test -f "outputs/$lane/spec.md"
  test -f "outputs/$lane/review.md"
done
```

### Post-Rerun Proof (truthful execution)

```bash
# games:multi-game should be blocked, not success
raspberry status --manifest fabro/programs/myosu.yaml --lane games:multi-game
# Expected: status shows "blocked" or "failed", not "success"

# games:traits should be complete
raspberry status --manifest fabro/programs/myosu.yaml --lane games:traits
# Expected: status shows "complete"

# chain:pallet and chain:runtime should be blocked on dependencies
raspberry status --manifest fabro/programs/myosu.yaml --lane chain:pallet
# Expected: status shows "blocked" or "restart"
```

---

## Dependency Order

```
Slice 1 (honest assessment)    → always first, no dependencies
Slice 2 (games:multi-game doc) → depends on Slice 1 complete
Slice 3 (detach path verify)   → depends on Raspberry/Fabro fix being applied
```

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| All lanes | `foundations` evaluates the execution truth of all lanes |
| `games:traits` | Reference for what a trustworthy bootstrap looks like |
| `games:multi-game` | Primary case study for the false-submit defect |
| `chain:pallet`, `chain:runtime` | Restart lanes that need honest status |
| Raspberry/Fabro | The execution substrate whose defects foundations documents |
