# Bootstrap Artifact Truthfulness

**Plan ID:** 002
**Carries forward:** bootstrap-oriented review work from the March 2026 cleanup slice
**Status:** Complete — bootstrap artifacts are current against the live repo

`genesis/PLANS.md` at `genesis/PLANS.md` governs this document.

---

## Purpose / Big Picture

After this plan lands, the bootstrap review artifacts under `outputs/` will be
truthful, current, and explicitly tied to the code they describe. Bootstrap no
longer means "a lane was run." It means the repo has up-to-date `spec.md` and
`review.md` artifacts for the four bootstrap surfaces, and later plans can rely
on them as current orientation.

---

## Progress

- [x] Historical bootstrap artifacts exist under `outputs/`
- [x] Re-verify `games:traits` bootstrap artifacts against current `myosu-games`
- [x] Re-verify `tui:shell` bootstrap artifacts against current `myosu-tui`
- [x] Re-verify `chain:runtime` bootstrap artifacts against current chain runtime state
- [x] Re-verify `chain:pallet` bootstrap artifacts against current pallet state
- [x] Record bootstrap completion in the active plan set and output tree

---

## Surprises & Discoveries

*(To be written during implementation)*

---

## Decision Log

- Decision: Bootstrap is defined by truthful reviewed artifacts, not by a
  specific orchestration tool.
  Rationale: The artifact is what later plans consume. The mechanism that
  produced it is secondary.
  Date/Author: 2026-03-21 / Interim CEO

---

## Outcomes & Retrospective

*(To be written at plan completion)*

---

## Milestones

### M1: Verify `games:traits` bootstrap artifacts are current
Confirm that `outputs/games/traits/spec.md` and
`outputs/games/traits/review.md` reflect the actual current state of
`crates/myosu-games/`.

Proof: `test -f outputs/games/traits/spec.md && test -f outputs/games/traits/review.md`
passes, and the artifact contents match the current crate state.

### M2: Verify `tui:shell` bootstrap artifacts are current
Confirm that `outputs/tui/shell/spec.md` and `outputs/tui/shell/review.md`
reflect the actual current state of `crates/myosu-tui/`.

Proof: `test -f outputs/tui/shell/spec.md && test -f outputs/tui/shell/review.md`
passes, and the artifact contents match the current crate state.

### M3: Verify `chain:runtime` and `chain:pallet` bootstrap artifacts are current
Confirm that `outputs/chain/runtime/{spec,review}.md` and
`outputs/chain/pallet/{spec,review}.md` reflect the actual current state of the
runtime and pallet code.

Proof: all four files exist, and the runtime/pallet reviews mention the current
compile blockers rather than stale assumptions.

### M4: Record bootstrap completion in the active plan set
Update the plan notes so later phases can treat bootstrap as complete without
referring to external control-plane state.

Proof: `genesis/plans/001-master-plan.md` and this plan both describe bootstrap
as artifact-based and current.

---

## Context and Orientation

Bootstrap establishes the reviewed baseline for the repo's four earliest
surfaces. The four bootstrap lanes are:

- `games:traits`
- `tui:shell`
- `chain:runtime`
- `chain:pallet`

Key files:
- `outputs/{frontier}/{lane}/spec.md` and `outputs/{frontier}/{lane}/review.md`
- `crates/myosu-games/`, `crates/myosu-tui/`, and `crates/myosu-chain/`

---

## Plan of Work

For each bootstrap surface:
1. Read the current `outputs/{frontier}/{lane}/spec.md` and `review.md`
2. Compare against current source state of the relevant crate
3. If the artifacts are stale, rewrite them directly from current code
4. Verify the updated artifact names concrete files, blockers, and behavior
5. Update plan status

---

## Concrete Steps

```bash
# Check current output artifact presence
find outputs/games/traits outputs/tui/shell outputs/chain/runtime outputs/chain/pallet \
  -maxdepth 1 -type f | sort

# Compare artifacts to current code
git diff -- outputs/games/traits outputs/tui/shell outputs/chain/runtime outputs/chain/pallet
cargo test -p myosu-games traits::tests::serialization_roundtrip_strategy_response --quiet
cargo test -p myosu-tui
cargo test --workspace --all-targets
```

---

## Validation

- `test -f outputs/games/traits/spec.md && test -f outputs/games/traits/review.md`
- `test -f outputs/tui/shell/spec.md && test -f outputs/tui/shell/review.md`
- `test -f outputs/chain/runtime/spec.md && test -f outputs/chain/runtime/review.md`
- `test -f outputs/chain/pallet/spec.md && test -f outputs/chain/pallet/review.md`
- This plan and the master plan describe bootstrap as complete only when those artifacts are current
