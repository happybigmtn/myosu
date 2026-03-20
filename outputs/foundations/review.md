# foundations — Lane Review

## Judgment Summary

| Surface | Status | Rationale |
|---------|--------|-----------|
| `games:traits` bootstrap artifacts | **KEEP** | Full bootstrap complete; all tests pass; path-dep risk accepted |
| `tui:shell` bootstrap artifacts | **KEEP (REOPEN modules)** | Bootstrap artifacts produced honestly; proof gaps in schema/events/shell are real and documented |
| `chain:runtime` bootstrap artifacts | **KEEP (RESTART)** | Bootstrap artifacts exist; runtime is non-existent as a crate — honest restart state |
| `chain:pallet` bootstrap artifacts | **KEEP (RESTART)** | Bootstrap artifacts exist; pallet is blocked upstream — honest restart state |
| `games:multi-game` submit | **RESET — FALSE SUBMIT** | Lane dispatched but no real implementation exists; must be repaired before treating as valid |
| `execute/status/watch` truth surface | **UNTRUSTWORTHY** | No evidence that consecutive runs agree on lane state; must be validated before downstream use |

---

## The `games:multi-game` False Submit — Root Cause Analysis

### What Happened

A Fabro run dispatched the `games:multi-game` lane. The workflow graph at
`fabro/workflows/bootstrap/multi-game.fabro` defines a 4-step pipeline:
`specify → review → polish → verify`. The `verify` step checked for the
existence of `outputs/games/multi-game/spec.md` and
`outputs/games/multi-game/review.md`, which had been pre-created as empty
`.gitkeep` siblings alongside real artifact files for other lanes.

The run produced an apparently successful exit, but:

1. No `crates/myosu-games-multi/` or equivalent multi-game implementation exists
2. No tests for multi-game logic exist in the workspace
3. The `polish` step ran against pre-seeded empty artifact files, not against
   a real implementation
4. The submit was **false** — it claimed completion without any real work done

### Why It Violates INV-001

INV-001 (Structured Closure Honesty) states:

> No dispatched turn may be treated as complete unless it ends in a trusted
> structured `RESULT:` or `BLOCKED:` outcome or fails closed.

The `games:multi-game` run did not fail closed. It reported success without
demonstrating that any real multi-game code was written, tested, or verified.
This is a false-green proof — exactly what INV-001 prohibits.

### Why It Violates INV-002

INV-002 (Proof Honesty) states:

> Named proof commands must actually execute and must never be used as
> false-green placeholders.

The `verify` step checked for file existence, but those files were pre-seeded
empty artifacts, not proof of real work. The proof command passed without
verifying that any implementation existed.

---

## What This Lane Must Preserve

### From `games:traits` (KEEP)

- All 10 unit tests + 4 doctests passing (`cargo test -p myosu-games`)
- The thin re-export surface in `src/traits.rs` (lines 8–9)
- The `reexports_compile` test confirming re-exports compile
- The path-dependency risk documented in `review.md` (accepted for now)

### From `tui:shell` (KEEP — with documented REOPEN modules)

- `screens.rs` (18 tests, all transitions, exhaustive) — KEEP without reservation
- `input.rs` (20+ tests, all key handlers) — KEEP without reservation
- `renderer.rs` (trait object-safe, mock-based tests) — KEEP without reservation
- `theme.rs` (8-token palette, 7 tests) — KEEP without reservation
- `pipe.rs` (ANSI detection, 5 tests) — KEEP with optional property test caveat
- `schema.rs` — REOPEN: only 3/20 games fully tested; proof gap is HIGH severity
- `events.rs` — REOPEN: 2 TTY-ignored tests, no headless alternative; proof gap
  is HIGH severity
- `shell.rs` — REOPEN: no integration test for screen transitions; proof gap is
  HIGH severity

The `tui:shell` bootstrap artifacts were produced **honestly** — the review
correctly identifies proof gaps and does not overclaim. The artifacts themselves
are trustworthy; what remains is the implementation work to close the proof gaps.

### From `chain:runtime` and `chain:pallet` (KEEP — RESTART state)

- Bootstrap artifacts exist and honestly characterize the restart state
- No Cargo.toml exists for `crates/myosu-chain/runtime/` — this is an honest
  "nothing to test yet" state, not a false claim of progress
- The restart lanes correctly show `chain:runtime` as a dependency for
  `chain:pallet`

---

## What This Lane Must Repair

### Priority 1: `games:multi-game` False Submit

**Must not preserve**: the false submit outcome.

**Must produce**: either (a) an honest failure if multi-game is not implemented,
or (b) a real implementation if multi-game should be built now.

**Verification after repair**:
```bash
# Option A (honest failure): verify the lane produces BLOCKED or honest failure
# Option B (real implementation): verify cargo test -p myosu-games-multi passes
```

### Priority 2: `execute/status/watch` Truth Reliability

**Must not assume**: that Fabro's `execute/status/watch` accurately reflects lane
state after a single run.

**Must verify**: that consecutive runs of the same lane produce agreeing outputs.

**Verification**:
```bash
# Run the same lane 3 times and compare execute/status/watch outputs
# If outputs disagree, the truth surface is not yet trustworthy
```

---

## Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| `games:multi-game` false submit propagates to downstream milestone decisions | **S0** | Must be repaired before any downstream lane depends on multi-game state |
| `execute/status/watch` disagrees across runs | **S1** | Do not use for milestone decisions until 3 consecutive agreeing runs |
| `tui:shell` proof gaps in schema/events/shell remain open | **S1** | Document in lane trust posture; do not claim "bootstrap complete" for these modules |
| `chain:runtime` has no Cargo.toml | **S2** | Honest restart state; not a risk, just a fact |
| `chain:pallet` blocked on runtime | **S2** | Honest dependency; not a risk, just a fact |

---

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| `games:traits` trust check | `cargo test -p myosu-games` | Exit 0, 10 unit + 4 doctest pass |
| `tui:shell` trust check | `cargo test -p myosu-tui` | Exit 0, all non-ignored tests pass |
| `multi-game` honest assessment | `ls crates/myosu-games-multi/` (or similar) | Returns "not found" or implementation files |
| `execute/status/watch` agreement | 3 consecutive Fabro runs of same lane | All 3 agree on lane state |

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/games/traits/spec.md` | Trusted leaf crate — `games:traits` lane contract |
| `outputs/games/traits/review.md` | Trusted leaf crate — `games:traits` trust assessment |
| `outputs/tui/shell/spec.md` | Shell lane contract with documented proof gaps |
| `outputs/tui/shell/review.md` | Shell trust assessment (KEEP + REOPEN on 3 modules) |
| `outputs/chain/runtime/spec.md` | Runtime restart lane — honest restart state |
| `outputs/chain/pallet/spec.md` | Pallet restart lane — honest restart state, blocked on runtime |
| `outputs/games/multi-game/review.md` | Multi-game false submit — must be repaired |
| `fabro/workflows/bootstrap/multi-game.fabro` | Multi-game workflow definition |
| `INVARIANTS.md` | INV-001 (structured closure honesty), INV-002 (proof honesty) definitions |
