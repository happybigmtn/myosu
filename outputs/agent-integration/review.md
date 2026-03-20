# `agent-integration` Lane Review

## Judgment: **IMPLEMENTATION FAMILY NEXT**

The `agent:experience` lane is the last remaining ready product lane. Its reviewed artifacts (`outputs/agent/experience/spec.md`, `outputs/agent/experience/review.md`) are sound. The integration adapter (`outputs/agent-integration/agent-adapter.md`) maps the lane's surfaces to the broader product. The decision is: **proceed to implementation-family workflow**.

---

## Decision Rationale

### Why Implementation Family Next

1. **`agent:experience` is already KEEP**: The lane's own `review.md` (at `outputs/agent/experience/review.md`) issued a **KEEP** judgment with the explicit recommendation: "Proceed to implementation-family workflow next." The spec quality is high, the upstream is trusted, and the slice dependency chain is clean.

2. **Phase 1 Slices 1–2 are unblocked**: `agent_context.rs` (Slice 1) and `journal.rs` (Slice 2) depend only on `tui:shell`, which is trusted with 82 passing tests. These slices can begin immediately without waiting for any other lane.

3. **Cross-lane robopoker risk is owned elsewhere**: The robopoker git migration is owned by the `games:traits` lane. This is a known risk that does not block the `agent:experience` implementation from starting — it only blocks full integration testing.

4. **`play:tui` binary is the only meaningful gate for early slices**: The `--context` flag wiring (Slice 3) requires the `myosu-play` binary CLI dispatch to exist. This is owned by `play:tui` lane Slice 1. Once that completes, Slices 3–4 can proceed in parallel with continued Slice 5 work.

5. **No upstream unblock is owned by this lane**: The blockers identified in `outputs/agent/experience/review.md` (robopoker migration, `myosu-play` binary, chain stub for lobby) are all owned by other lanes. There is no additional upstream work that `agent:experience` must complete before implementation.

---

## Integration Posture

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | AX-01..05 + SP-01..03 are sound; 9 slices defined |
| `agent:experience` review | **KEEP** | Proceed to implementation |
| Integration adapter | **COMPLETE** | `outputs/agent-integration/agent-adapter.md` maps all surfaces |
| Phase 1 (Slices 1–2) | **UNBLOCKED** | `tui:shell` trusted; can start immediately |
| Phase 1 (Slices 3–4) | **BLOCKED** | Waiting on `play:tui` Slice 1 (binary skeleton) |
| Phase 2 (Slices 5–7) | **BLOCKED** | Waiting on Phase 1 + `play:tui` |
| Phase 3 (Slices 8–9) | **BLOCKED** | Waiting on Phase 1 + `play:tui` |
| Phase 4 (chain-connected) | **BLOCKED** | Waiting on `chain:runtime` |
| robopoker git migration | **RISK** | Owned by `games:traits`; must resolve before Phase 1 integration testing |

---

## Downstream Blockers Owned by Other Lanes

These are not `agent-integration` problems to solve, but they are tracked here for visibility:

| Blocker | Owner | Impact on This Lane |
|---------|-------|---------------------|
| `robopoker` git migration | `games:traits` | Blocks `cargo test` on clean checkout; Phase 1 integration testing |
| `myosu-play` binary skeleton | `play:tui` | Blocks Slices 3–4 (`--context` wiring, `reflect>` prompt) |
| `chain:runtime` | `chain:runtime` | Blocks Phase 4 (lobby chain queries, spectator WS upgrade) |
| Lobby chain stub | `chain:runtime` | Slice 7 lobby uses hardcoded data for Phase 0 |

---

## Recommended Next Steps

### Immediate (this run)

1. **Create `agent:experience` implementation lane** in `fabro/programs/myosu-product.yaml` or a dedicated implementation program
2. **Begin Slice 1** (`agent_context.rs`) — unblocked, depends only on trusted `tui:shell`
3. **Begin Slice 2** (`journal.rs`) — unblocked, depends only on trusted `tui:shell`

### Near-term (after `play:tui` Slice 1)

4. **Begin Slice 3** (`--context` flag wiring) — requires `myosu-play` binary CLI dispatch
5. **Begin Slice 4** (`reflect>` prompt) — depends on Slice 3
6. **Begin Slice 5** (`narration.rs`) — can proceed independently once `GameState` schema is confirmed

### Medium-term

7. **Begin Slice 6** (`--narrate` flag wiring) — depends on Slice 5
8. **Begin Slice 7** (lobby + game selection) — depends on `play:tui` binary
9. **Begin Slices 8–9** (spectator relay + screen) — depends on Phase 1 + `play:tui`

### Coordination required

- Confirm `games:traits` robopoker migration timeline before committing to Phase 1 integration testing date
- Align `play:tui` binary skeleton delivery with `agent:experience` Slice 3 start
- Verify spectator socket path convention (`~/.myosu/spectate/<id>.sock`) against `play:tui` data directory convention before Slice 8

---

## Conclusion

`agent:experience` is ready. The integration adapter has been written. The lane's reviewed artifacts are sound. The decision is **implementation family next** — begin the implementation lane for `agent:experience` with Slices 1–2 immediately, and coordinate with `play:tui` for the binary skeleton that unlocks Slices 3–4.