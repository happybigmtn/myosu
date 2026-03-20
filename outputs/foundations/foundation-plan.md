# Foundations Lane Plan

**Lane**: `foundations`
**Date**: 2026-03-20
**Status**: Active

---

## Purpose / Big Picture

This lane produces the first honest reviewed slice of the Myosu frontier. Its job is to assess the current execution truth — what Fabro/Raspberry can actually trust — and define the minimal honest path forward for bootstrapping a functional Fabro-native execution surface.

The lane does not implement product code. It implements **execution truth**: the durable artifacts that tell future agents and contributors what is working, what is not, and what to do next.

---

## Progress

- [x] (2026-03-20) Created this plan and `review.md` after auditing all existing outputs
- [ ] Assess `execute/status/watch` truth trustworthiness across all active lanes
- [ ] Define the honest slice for `games:multi-game` false-submit repair
- [ ] Produce updated `review.md` with lane-by-lane trust classification
- [ ] Update this `foundation-plan.md` with findings

---

## Current Lane Trust Inventory

### Trusted Lanes (can execute without defect repair)

| Lane | Trust Signal | Evidence |
|------|-------------|----------|
| `games:traits` | Full bootstrap + implementation cycle complete | `outputs/games/traits/{spec,review,implementation,verification}.md` all exist |
| `tui:shell` | Bootstrap spec + review exist | `outputs/tui/shell/{spec,review}.md` exist |

### Restart-Required Lanes (not trustworthy until rebuilt)

| Lane | Restart Signal | Blocker |
|------|---------------|---------|
| `chain:runtime` | `runtime/src/lib.rs` imports 15+ crates that don't exist in workspace | No `Cargo.toml` manifests for chain crates; `subtensor_*` workspace keys undefined |
| `chain:pallet` | `cargo check -p pallet-game-solver` fails with 50+ error types | `subtensor_runtime_common`, `subtensor_macros`, `codec` crate alias all missing |

### Greenfield Lanes (no code yet)

| Lane | Blocker |
|------|---------|
| `games:multi-game` | `myosu-games-liars-dice` crate does not exist; `ExploitMetric` not in `myosu-games` |
| `games:poker-engine` | No outputs yet |
| `miner:service` | No outputs yet |
| `validator:oracle` | No outputs yet |
| `play:tui` | No outputs yet |

### Untrustworthy Lanes (false-positive state)

| Lane | Problem | Required Action |
|------|---------|----------------|
| `games:multi-game` | Has `review.md` but no implementation code exists; review was written against a non-existent crate | Must be reset to greenfield or have honest review written against reality |

---

## The `games:multi-game` False-Submit Problem

### What happened

The `games:multi-game` lane has a `review.md` that was written as if the implementation existed. The lane was marked KEEP in the review, but the crate `myosu-games-liars-dice` does not exist in `crates/`. This is a **false-submit**: Raspberry was told the lane had reviewed artifacts when the implementation code was entirely greenfield.

### Root cause

The Fabro lane ran and produced `review.md` based on spec analysis rather than implementation evidence. The spec was sound, but the execution produced an artifact without verifying the implementation actually existed.

### Repair path

Two honest options:

**Option A — Honest Greenfield Reset**: Rewrite `games:multi-game/review.md` to honestly state the lane is greenfield and the existing review was a false-positive. Mark the lane as awaiting implementation, not as reviewed.

**Option B — Live Run Conversion**: Actually implement the `myosu-games-liars-dice` crate so the review matches reality. This requires:
1. Create `crates/myosu-games-liars-dice/src/game.rs` implementing `CfrGame: Copy`
2. Add `ExploitMetric` to `crates/myosu-games/src/traits.rs`
3. Pass all Slice 2–7 tests from the existing review

### Decision Required

The foundations lane cannot resolve this alone — it requires a judgment call:
- If Option A: the `games:multi-game` lane gets an honest reset review and the `false-submit` is recorded as a Fabro execution defect
- If Option B: the `games:multi-game` lane becomes the next honest implementation target after `games:traits`

---

## Fabro/Raspberry Defect Inventory

The task states: "Fix Raspberry/Fabro defects only when they are discovered by real Myosu execution."

### Known Defects

1. **`games:multi-game` false-submit**: Lane produced `review.md` without verifying the implementation existed. Root cause is that the bootstrap workflow does not have a precondition check for implementation existence before writing review artifacts.

2. **`chain:runtime` / `chain:pallet` stub checks**: The `./fabro/checks/chain-runtime-reset.sh` and `./fabro/checks/chain-pallet-reset.sh` scripts exit 0 even when the chain does not build. They are no-op stubs that give a false-positive healthy signal.

3. **`execute/status/watch` truth gap**: Without a stable Fabro-to-Raspberry run-truth adapter, Raspberry infers lane health by scanning run directories. The `myosu.yaml` program references `state_path: ../../.raspberry/myosu-state.json` but the `.raspberry/` directory does not exist in the worktree.

---

## The Honest Slice

The foundations lane produces:

1. **Updated `review.md`** — A lane-by-lane trust classification that replaces the `games:multi-game` false-positive with honest state

2. **This plan** — Updated with concrete next steps based on the two-option decision above

3. **A `games:multi-game` honest reset** (if Option A is chosen) — New `review.md` that honestly says "greenfield"

---

## Concrete Steps

1. Run lane trust inventory across all outputs:

```bash
# Check which lanes have real code vs. just artifacts
for lane in games/traits tui/shell chain/runtime chain/pallet games/multi-game games/poker-engine; do
  echo "=== $lane ==="
  ls outputs/$lane/*.md 2>/dev/null || echo "(no artifacts)"
done

# Check which crates actually build
cargo check -p myosu-games 2>&1 | tail -3
cargo check -p pallet-game-solver 2>&1 | tail -3
```

2. Make the Option A / Option B decision on `games:multi-game`

3. If Option A: write honest `games/multi-game/review.md`

4. If Option B: this plan becomes the bootstrap for the `games:multi-game` implementation lane

---

## Validation and Acceptance

Acceptance for this lane is honest artifacts, not code:

- `outputs/foundations/review.md` accurately describes the trust state of every lane
- `outputs/foundations/foundation-plan.md` is updated with concrete next steps
- The `games:multi-game` false-submit is either honestly reset or honestly repaired
- All Fabro/Raspberry defects are recorded, not hidden

---

## Dependencies

- This lane depends on no other lanes
- Other lanes depend on this lane's honest output for their execution truth

---

## Revision Log

- (2026-03-20) Initial draft — produced alongside `review.md` after auditing all existing outputs
