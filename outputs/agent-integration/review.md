# `agent-integration` Lane Review

## Decision: **PROCEED** — Implementation Family for Product Frontier

The `agent:experience` lane has been reviewed (KEEP). Both product lanes in the `myosu-product` frontier are now in reviewed state. The product frontier is ready for an implementation-family workflow.

**Decision: launch the `agent:experience` implementation family, beginning with Slices 1–2 immediately.**

---

## Evidence for Decision

### `agent:experience` Review Verdict (KEEP)

The `agent:experience` lane review (`outputs/agent/experience/review.md`) concludes:

> **Proceed to implementation-family workflow next.** The lane is well-specified and the upstream is trusted. The primary blocker (robopoker git migration) is owned by the `games:traits` lane.

The review identifies 9 sequential, minimally-coupled slices. Slices 1–2 (agent_context.rs, journal.rs) have no external dependencies beyond `tui:shell`, which is already trusted with 82 passing tests.

### `play:tui` Review Verdict (KEEP)

The `play:tui` lane review (`outputs/play/tui/review.md`) concludes:

> **Judgment: KEEP — Ready for Implementation-Family Workflow.** The `play:tui` lane is unblocked for an implementation-family workflow immediately.

Both product lanes are now reviewed. The product frontier has no remaining upstream unblocks for the first slices.

### Upstream Trust Status

| Lane | Status | Evidence |
|------|--------|----------|
| `tui:shell` | **TRUSTED** | 82 tests pass |
| `games:traits` | **TRUSTED** | 14 tests pass |
| `play:tui` | **REVIEWED KEEP** | Binary in progress (Slice 1) |

### Immediate Unblock Assessment

| `agent:experience` Slice | Blockers | Status |
|--------------------------|----------|--------|
| Slice 1: agent_context.rs | `tui:shell` only | **UNBLOCKED** |
| Slice 2: journal.rs | `tui:shell` only | **UNBLOCKED** |
| Slice 3: --context flag | `play:tui` binary | Blocked pending `play:tui` Slice 1 |
| Slice 4: reflect> prompt | `play:tui` binary | Blocked pending `play:tui` Slice 1 |
| Slice 5: narration.rs | `tui:shell` only | **UNBLOCKED** |
| Slice 6: --narrate flag | `play:tui` binary | Blocked pending `play:tui` Slice 1 |
| Slice 7: lobby | Chain discovery | Stub for Phase 0 |
| Slice 8: SpectatorRelay | `play:tui` binary | Blocked pending `play:tui` Slice 1 |
| Slice 9: SpectateScreen | `play:tui` binary | Blocked pending `play:tui` Slice 1 |

**Conclusion**: 3 of 9 slices are immediately unblocked. Launch implementation now.

---

## What This Decision Triggers

### In `myosu-product` Program

The `agent:experience` lane transitions from `reviewed` milestone to an implementation-family workflow:

- Create `fabro/run-configs/product/agent-experience-impl-{slice}.toml` for each implementation slice
- Add `implementation.md` and `verification.md` artifacts as slices complete
- Track proof availability in `outputs/agent/experience/review.md`

### In `myosu-product` Manifest

The `agent:experience` unit milestone advances from `reviewed` to a new `implementation` milestone:

```yaml
milestones:
  - id: reviewed
    requires: [spec, review]
  - id: implemented
    requires: [spec, review, implementation, verification]
```

### Cross-Program Notification

`games:traits` lane should be notified that `agent:experience` is proceeding to implementation. The robopoker git migration (owned by `games:traits` Slice 1) remains the highest-priority shared blocker. If it slips, `agent:experience` Phase 1 integration testing will fail.

`play:tui` lane should be notified that `agent:experience` Slice 3+ depends on the `myosu-play` binary skeleton. Parallel execution is acceptable: `agent:experience` Slices 1–2 run while `play:tui` Slice 1 completes.

---

## Blockers Still Owned Elsewhere

These blockers are not owned by `agent:integration`, but they affect the lane's later slices:

| Blocker | Severity | Owner | Impact |
|---------|----------|-------|--------|
| `robopoker` git migration | **HIGH** | `games:traits` Slice 1 | Blocks Phase 1 integration testing |
| `myosu-play` binary skeleton | **HIGH** | `play:tui` Slice 1 | Blocks Slices 3, 4, 6, 8, 9 |
| Chain discovery for lobby | **MEDIUM** | `chain:runtime` (future) | Slice 7 uses stub for Phase 0 |
| Spectator socket path convention | **LOW** | `play:tui` data-dir | Verify before Slice 8 |

---

## Rationale

**Why proceed now rather than wait for all blockers to clear?**

1. **Slices 1–2 are genuinely unblocked**: `agent_context.rs` and `journal.rs` use only `serde` + `std`. They compile and test without any binary, robopoker, or chain integration.

2. **Sequential coupling enables parallelism**: Slices 1–2 can run while `play:tui` Slice 1 completes. Waiting for `play:tui` to finish before starting `agent:experience` would add zero value.

3. **The review verdict is clear**: "proceed to implementation-family workflow next." The reviewed artifacts have already made this decision. This document records the confirmation and triggers the follow-on work.

4. **Early proof enables early feedback**: Getting `agent_context.rs` and `journal.rs` into the codebase early allows the team to validate the context file schema and journal format before the more complex pipe-mode wiring begins.

---

## Next Actions

1. Launch `agent:experience` implementation slices 1 and 2 immediately (unblocked)
2. Notify `play:tui` that `agent:experience` Slice 3+ depends on binary skeleton
3. Monitor `games:traits` robopoker migration progress (blocking Phase 1 integration)
4. Update `outputs/agent/experience/review.md` with proof availability after each slice
