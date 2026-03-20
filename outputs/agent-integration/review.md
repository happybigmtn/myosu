# `agent:integration` Lane Review

## Judgment: **KEEP — Proceed with Partial Implementation Family**

The `agent:experience` lane has produced sound reviewed artifacts. The honest decision is: **start implementation for the 3 unblocked slices immediately, and unblock the remaining 6 slices through their upstream dependencies**.

---

## Rationale for Partial Go

The `agent:experience` review identified 9 implementation slices. Breaking them down by dependency:

### Immediately Runnable (Slices 1, 2, 5)

| Slice | File | Why Unblocked |
|-------|------|---------------|
| 1 | `agent_context.rs` | Depends only on `tui:shell` (82 tests, trusted) |
| 2 | `journal.rs` | Depends only on `tui:shell` (82 tests, trusted) |
| 5 | `narration.rs` | Depends only on `tui:shell` (82 tests, trusted) |

These three slices require no `play:tui` binary, no chain runtime, and no cross-slice dependencies. They can proceed immediately.

### Blocked on `play:tui` Binary (Slices 3, 6)

| Slice | File | Blocker |
|-------|------|---------|
| 3 | `--context` wiring in `PipeMode` | `myosu-play` binary must exist to wire flags |
| 6 | `--narrate` wiring in `PipeMode` | `myosu-play` binary must exist to wire flags |

The `play:tui` lane is in implementation but has not yet produced the binary skeleton.

### Blocked on Chain Runtime (Slice 7)

| Slice | File | Blocker |
|-------|------|---------|
| 7 | Lobby + game selection | Chain discovery stubbed for Phase 0; real integration needs `chain:runtime` |

### Blocked on Sequential Dependencies (Slices 4, 8, 9)

| Slice | File | Blocker |
|-------|------|---------|
| 4 | `reflect>` prompt | Depends on Slice 3 (context wiring) |
| 8 | `SpectatorRelay` | Depends on `play:tui` binary |
| 9 | `SpectateScreen` | Depends on Slice 8 |

---

## Decision: Implementation Family — Partial

**Product needs an implementation family next, but it is partially unblocked.**

The `agent:experience` spec defines 9 slices. Only 3 are ready to implement now. The other 6 require upstream work:

| Priority | Slices | Action |
|----------|--------|--------|
| **IMMEDIATE** | 1, 2, 5 | Start `agent_context.rs`, `journal.rs`, `narration.rs` |
| **NEXT** | 3, 6 | Wait for `play:tui` binary, then wire `--context` and `--narrate` |
| **LATER** | 4 | After Slice 3 complete |
| **LATER** | 7 | After `chain:runtime` available (stubbed Phase 0 OK) |
| **LATER** | 8, 9 | After `play:tui` binary + Slice 7 |

---

## Upstream Unblock Requirements

For the blocked slices to proceed, the following must happen:

| Lane | Slice | Required Action | Status |
|------|-------|-----------------|--------|
| `play:tui` | Slice 1 | `myosu-play` binary skeleton | **NOT STARTED** |
| `chain:runtime` | — | Minimal chain client for lobby stub | **NOT STARTED** |
| `games:traits` | Slice 1 | `robopoker` git migration | **IN PROGRESS** |

---

## Recommendation

**Proceed to implementation-family workflow for slices 1, 2, and 5 immediately.**

These three slices:
- Are fully specified in `agent:experience/spec.md`
- Depend only on trusted infrastructure (`tui:shell`, 82 tests)
- Produce foundational primitives (`AgentContext`, `Journal`, `NarrationEngine`) used by all subsequent slices
- Can be implemented in parallel by separate agents

The remaining 6 slices should be tracked as **blocked** until their upstream dependencies resolve. They do not need another spec pass — the spec is complete. They need implementation execution in the dependent lanes first.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | 9 slices defined with clear boundaries |
| `agent:experience` review | **READY** | KEEP judgment; proceed to implementation |
| Upstream (`tui:shell`) | **TRUSTED** | 82 tests pass |
| Upstream (`games:traits`) | **TRUSTED** | 14 tests pass |
| Upstream (`play:tui`) | **PARTIAL** | Binary missing; blocks Slices 3, 6, 8, 9 |
| Upstream (`chain:runtime`) | **MISSING** | Blocks Slice 7 |
| Implementation slices 1, 2, 5 | **READY TO START** | Can begin immediately |
| Implementation slices 3, 4, 6, 7, 8, 9 | **BLOCKED** | Awaiting upstream |

---

## What This Lane Owns

The `agent:integration` lane exists to:
1. **Decide** whether `agent:experience` artifacts are sufficient for implementation
2. **Map** slice dependencies to upstream lane readiness
3. **Route** unblocked slices to implementation and track blocked slices
4. **Update** the adapter contract as implementation reveals gaps

This lane does **not** own the implementation of the 9 slices. That work belongs to the implementation-family workflow that executes the slices.

---

## Next Steps

1. **Start implementation** for slices 1 (`agent_context.rs`), 2 (`journal.rs`), 5 (`narration.rs`) in parallel
2. **Track `play:tui`** binary progress; wire Slices 3 and 6 when ready
3. **Track `chain:runtime`** progress; implement Slice 7 when chain client available
4. **Update this review** when upstream dependencies resolve
