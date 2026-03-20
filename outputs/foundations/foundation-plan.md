# foundations — Lane Plan

## Purpose

Establish honest execution truth as the baseline contract for all Myosu Fabro lanes.

The `foundations` lane does not own a specific crate or product surface. It owns the
**cross-lane execution integrity** surface: ensuring that every Fabro-dispatched turn
produces a trustworthy structured outcome, that false submits are repaired before they
propagate, and that `execute/status/watch` truth is reliable enough to drive downstream
milestone decisions.

## Lane Boundary

`foundations` is the **horizontal execution-integrity lane**. It owns:

- The repair of the `games:multi-game` false-submit produced by an earlier Fabro run
- The definition and enforcement of honest submit criteria across all active lanes
- The honest trust posture assessment of every bootstrap lane (`games:traits`,
  `tui:shell`, `chain:runtime`, `chain:pallet`) as input to downstream milestone
  decisions
- The invariant-gate discipline that defects are only repaired when discovered by
  **real Myosu execution**, not by inspection

`foundations` does **not** own:

- Writing code in domain crates (games, chain, tui, miner, validator, play)
- Defining the product roadmap or architecture (those are `strategy/planning` and
  the domain specs)
- Owning the Raspberry manifest (that lives in `fabro/programs/myosu.yaml`)

## Current State

### False Submit on `games:multi-game`

A prior Fabro run dispatched the `games:multi-game` lane and produced an apparently
successful outcome, but the `polish` step did not execute real multi-game code.
The lane has no corresponding crate surface in `crates/`, no passing tests, and no
implementation. The submit was **false** — it ended without a trusted `RESULT:` or
`BLOCKED:` outcome and without failing closed.

This is an **INV-001 violation**: structured closure honesty.

### Lane Trust Posture Summary

| Lane | Trust | Bootstrap Status | Blocked By |
|------|-------|-----------------|-----------|
| `games:traits` | **KEEP** | Bootstrap complete | — |
| `tui:shell` | **REOPEN** (3 modules) | Bootstrap artifacts exist; proof gaps remain | `schema` coverage, `events` TTY, `shell` integration |
| `chain:runtime` | **RESTART** | Bootstrap artifacts exist | No Cargo.toml for chain/runtime yet |
| `chain:pallet` | **RESTART** | Bootstrap artifacts exist | Blocked on runtime review |
| `games:multi-game` | **RESET — FALSE SUBMIT** | No real implementation | Must become truthful failure or live run |

### The Two Frontier Tasks

**Task 1 — Fix Raspberry/Fabro defects only when discovered by real Myosu
execution, then rerun the affected frontier until `execute/status/watch` truth is
trustworthy again.**

This means:
- Do not proactively fix defects discovered by reading code
- Only act on defects surfaced by a Fabro run's proof output
- After each fix, rerun the affected lane and verify the run produces a
  trustworthy structured outcome
- Treat `execute/status/watch` as untrustworthy until multiple consecutive runs
  agree on lane state

**Task 2 — Convert `games:multi-game` false-submit into a truthful failure or
successful live run, then rerun the lane with the repaired Fabro detach path.**

This means:
- The false submit cannot be left as-is; it must be closed with a real outcome
- Two valid paths: (a) the lane honestly fails because multi-game has no
  implementation, or (b) an implementation is created that makes the lane succeed
- After closing the false submit, rerun the lane and verify the new run produces
  a real structured outcome

## Honest Submit Criteria

A Fabro lane submit is **honest** when it ends in one of:

1. `RESULT:` — all proof gates passed, curated artifacts written, tests green
2. `BLOCKED:` — lane is blocked on an upstream dependency that is not yet satisfied
3. **Fails closed** — if proof cannot be demonstrated, the lane must report failure
   rather than claiming success

A submit is **false** when:

- The lane reports success but no real implementation exists
- The lane reports success but proof gates were not actually executed
- The `polish` or `verify` step produced no meaningful evidence of work done

## Next Slices

### Slice 1 — Repair the `games:multi-game` False Submit

**Action**: Evaluate whether `games:multi-game` should honestly fail (no multi-game
implementation exists) or be built as a real lane.

If honest failure:
1. Create a `fabro/programs/myosu-platform.yaml` lane that honestly declares
   `games:multi-game` as blocked or not-yet-implemented
2. Rerun the `games:multi-game` lane through Fabro and verify it produces
   `BLOCKED:` or an honest failure rather than false success
3. Update `outputs/games/multi-game/review.md` to reflect the honest state

If real implementation:
1. Create the multi-game implementation surface under `crates/myosu-games-multi/`
2. Add tests proving at least one multi-game type works
3. Rerun the lane and verify it produces `RESULT:`

**Proof gate**: Rerunning the lane produces a trustworthy structured outcome
matching the actual state of the multi-game surface.

### Slice 2 — Assess and Document Lane Trust Posture

**Action**: For each active bootstrap lane, write an honest one-paragraph trust
summary in `outputs/foundations/lane-trust.md`:

- `games:traits` — KEEP; all tests pass; path-dependency risk accepted for now
- `tui:shell` — 3 modules REOPEN; proof gaps documented; lane cannot be declared
  "bootstrapped" until gaps are closed
- `chain:runtime` — RESTART; no Cargo.toml; nothing to test yet
- `chain:pallet` — RESTART; blocked upstream on runtime; same non-state as runtime

**Proof gate**: `outputs/foundations/lane-trust.md` exists and honestly characterizes
each lane.

### Slice 3 — Harden `execute/status/watch` Truth Reliability

**Action**: Define what "trustworthy `execute/status/watch`" means for Myosu lanes:

- Run the same lane 3 times consecutively
- Record whether `execute/status/watch` reports the same state all 3 times
- If outputs disagree, the truth surface is not yet trustworthy

Until `execute/status/watch` is trustworthy, downstream milestone decisions must not
assume lane state is reliable.

**Proof gate**: A lane run 3 times produces 3 agreeing `execute/status/watch`
reports.

## Progress

- [ ] Slice 1: Repair `games:multi-game` false submit
- [ ] Slice 2: Document lane trust posture honestly
- [ ] Slice 3: Harden `execute/status/watch` truth reliability

## Decision Log

- Decision: `foundations` is a horizontal lane, not a crate-specific lane.
  Rationale: execution integrity spans all lanes and cannot belong to any single
  domain unit.
  Date/Author: 2026-03-20 / foundations lane bootstrap

- Decision: A false submit is an INV-001 violation regardless of whether the lane
  "would have succeeded if code existed."
  Rationale: structured closure honesty requires the outcome to match the actual
  code state, not the hypothetical state.
  Date/Author: 2026-03-20 / foundations lane bootstrap

- Decision: Only repair defects discovered by real Fabro execution, not by
  inspection alone.
  Rationale: the execution substrate is the ground truth; reading code cannot
  substitute for running code.
  Date/Author: 2026-03-20 / foundations lane bootstrap
