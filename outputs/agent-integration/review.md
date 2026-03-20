# `agent:integration` Lane Review

## Judgment: **PROCEED** — implementation-family workflow next, parallel upstream unblocks

The `agent:experience` lane has passed honest review with a **KEEP** judgment.
The lane spec is sound, the upstream (`tui:shell`, `games:traits`) is trusted,
and the 9 implementation slices are sequential with minimal coupling.

The product should proceed to an implementation-family workflow for
`agent:experience` Slices 1–2 immediately. Parallel upstream unblocks
(`robopoker` git migration, `play:tui` binary skeleton) should proceed
concurrently as they are owned by other lanes.

---

## Decision Rationale

### Why implementation-family next (not more upstream work)

1. **`agent:experience` is specification-complete**: The lane has a full
   `spec.md` (368 lines) covering 9 slices with clear boundaries, proof gates,
   and dependency chains. The `schema.rs` (939 lines, 16 tests) is already
   trusted. The lane is not blocked on further specification work.

2. **Upstream is trusted**: `tui:shell` (82 tests) and `games:traits`
   (14 tests) are both in KEEP state. Slices 1–2 of `agent:experience`
   depend only on these trusted surfaces. They can begin immediately.

3. **Sequential slices reduce risk**: The 9 slices have a clean dependency
   chain (Slices 1→2→3→4→5→6→7→8→9). Starting with Slices 1–2 exercises
   the pattern (load/save, append-only journal) without triggering the
   `play:tui` dependency.

4. **Upstream unblocks are owned elsewhere**: The two HIGH blockers
   (`robopoker` git migration, `myosu-play` binary skeleton) are owned by
   `games:traits` and `play:tui` lanes respectively. They should proceed
   in parallel, not as prerequisites to starting implementation work.

### Why not wait for upstream to clear first

- **Slices 1–2 are independent**: They use only `tui:shell` and `games:traits`
  types that already exist and pass tests. No `play:tui` binary needed.
- **Feedback velocity**: Starting implementation surfaces integration issues
  faster than waiting. If the `AgentContext` design has problems, better to
  find out now than after `play:tui` is complete.
- **Parallel tracks are healthy**: The `games:traits` and `play:tui` lanes
  already have their own momentum. `agent:experience` should not create a
  serial bottleneck.

---

## Upstream Unblock Tracking

These are **not** prerequisites to starting Slices 1–2, but they must be
resolved before later slices:

| Blocker | Severity | Owner | Must Clear By |
|---------|----------|-------|---------------|
| `robopoker` git migration | HIGH | `games:traits` | Slice 5 (narration integration) |
| `myosu-play` binary skeleton | HIGH | `play:tui` | Slice 3 (--context wiring) |
| Chain discovery stub for lobby | MEDIUM | `agent:experience` | Slice 7 (self-stub is acceptable) |
| Spectator socket path convention | LOW | `play:tui` | Slice 8 |

The control plane should track these as parallel work items, not as gate
prerequisites for the `agent:experience` lane starting.

---

## Lane Readiness Summary

| Dimension | Status |
|-----------|--------|
| `agent:experience` spec | **READY** |
| `agent:experience` review | **KEEP** |
| `tui:shell` upstream | **TRUSTED** |
| `games:traits` upstream | **TRUSTED** |
| `play:tui` upstream | **PARTIAL** (binary skeleton missing) |
| `robopoker` dependency | **BLOCKER** (owned by `games:traits`) |
| Slices 1–2 readiness | **CAN START NOW** |
| Slices 3–9 readiness | **BLOCKED** (on `play:tui` + `robopoker`) |

---

## What This Means for the Control Plane

1. **Add `agent:experience` to the implementation-family program**: The lane
   is ready for Slice 1 (`agent_context.rs`). The Fabro run config should
   target `agent-experience.toml` and execute Slices 1–2 first.

2. **Do not gate `agent:experience` on `play:tui` completing**: The
   `depends_on` in `myosu-product.yaml` creates a milestone gate, but
   implementation can proceed in parallel. The gate should track milestone
   readiness (`reviewed`), not implementation readiness.

3. **Track `robopoker` migration separately**: This is on the critical path
   for Slice 5+. The `games:traits` lane should be queried for its robopoker
   resolution status before Slice 5 begins.

4. **Expect `agent:experience` slices to produce `implementation.md` and
   `verification.md` artifacts**: Per the outputs convention, each completed
   slice should leave behind evidence of what changed and what was proven.

---

## Recommendation

**Start `agent:experience` Slice 1 immediately.** The lane is reviewed, the
upstream is trusted for early slices, and the implementation-family workflow
is the correct next step. The upstream blockers are real but are owned by
other lanes with their own momentum.

Once `play:tui` Slice 1 (binary skeleton) and `robopoker` git migration
are complete, Slices 3–9 can proceed in sequence without additional review.
