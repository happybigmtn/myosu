# `agent-integration` Lane Review

## Judgment: **GO** — proceed to implementation; `games:traits` robopoker migration is the critical path

This lane has produced the honest first slice: an integration adapter that documents how the `agent:experience` surfaces compose into the product. The adapter surfaces are well-defined and self-consistent. The decision is **GO** for `agent:experience` implementation with one critical-path upstream unblock.

---

## Rationale for GO

1. **Integration surfaces are well-defined**: The `agent-adapter.md` produced by this lane documents a coherent integration architecture. The seven surfaces (schema, pipe mode, AgentContext, Journal, reflect prompt, NarrationEngine, lobby, SpectatorRelay) have clear data flow, clear CLI flag integration points, and clear ownership boundaries. No ambiguity about where surfaces fit.

2. **`agent:experience` review confirms implementation readiness**: The `agent:experience` lane at `outputs/agent/experience/review.md` gives a **KEEP** judgment with 9 sequential slices and clean internal dependencies. Slices 1–2 (agent_context.rs, journal.rs) depend only on `tui:shell` which is already trusted. The lane is not blocked by anything inside itself.

3. **Critical path is identified, not hidden**: The two blockers (robopoker git migration, myosu-play binary) are not surprises discovered during integration — they were already documented in the `agent:experience` review. This lane confirms them as the exact gate for when `agent:experience` slices 1–2 can start versus when slices 3+ can start.

4. **Integration adapter is honest about Phase 0 limitations**: The lobby stub (hardcoded subnet data), the spectator Phase 0 only (Unix socket, no WebSocket), and the robopoker path dependency are all explicitly called out. This prevents false confidence in early integration tests.

---

## Decision: Product Needs `games:traits` Robopoker Migration Next

The highest-value next step for the product is **not** another upstream unblock analysis. The highest-value next step is **executing the robopoker git migration** in the `games:traits` lane.

**Rationale**:
- `games:traits` is the first lane in the critical path that has a defined, small, achievable Slice 1 fix: replace `/home/r/coding/robopoker/crates/...` absolute path deps with `git = "https://github.com/krukah/robopoker"` in `Cargo.toml`.
- This single change unblocks `play:tui` Slice 1 (binary scaffold), which unblocks `agent:experience` Slice 3 (--context wiring).
- The `agent:experience` review explicitly says the lane can begin Slices 1–2 immediately (they only depend on `tui:shell`), but Slices 3+ require `play:tui` Slice 1 (binary skeleton).

**Decision chain**:
```
games:traits Slice 1 (robopoker git migration)
  → unblocks → games:traits Slice 2+ (continued traits work)
  → simultaneously → play:tui Slice 1 (binary scaffold, can start now or after robopoker)
  → then → agent:experience Slices 1-2 (agent_context, journal — can start any time tui:shell is trusted)
  → then → agent:experience Slices 3+ (needs binary scaffold AND robopoker resolved)
```

---

## Integration Assessment

### Surface Completeness: HIGH

The `agent-adapter.md` documents all seven integration surfaces with clear data flow and ownership. No surface is missing from the adapter that exists in the spec. The only surfaces not covered are those explicitly belonging to future phases (spectator WebSocket, chain-connected lobby).

### Integration Clarity: HIGH

The CLI flag integration table in the adapter is precise:
- `--pipe` creates `PipeMode`
- `--context` triggers `AgentContext` load/save lifecycle
- `--narrate` switches to `NarrationEngine`
- `--spectate` activates `SpectatorRelay`
- `--subnet` bypasses lobby

Each flag has a single integration point in `main.rs`. There is no ambiguous flag interaction.

### Data Flow Integrity: HIGH

The adapter correctly identifies the critical invariant: `NarrationEngine::narrate()` and `pipe_output()` must produce identical game outcomes from the same `GameState`. This prevents narration from becoming an implicit game logic change.

The fog-of-war contract for `SpectatorRelay` is also correctly specified: hole cards never appear during active play, only after the `showdown` event. This is enforced at the relay, not at the renderer — which is the right architectural choice because it means even a compromised renderer cannot leak information.

### Stub Reality: ACCEPTABLE FOR PHASE 0

The hardcoded lobby data for Phase 0 is explicitly documented as a stub. The adapter notes the integration point (chain query) and the Phase 1 upgrade path. This is honest and prevents premature integration testing against non-existent chain data.

---

## Remaining Blockers for Full Integration

| Blocker | Severity | Owner | Unblocks |
|---------|----------|-------|----------|
| robopoker git dependency | **HIGH** | `games:traits` | All slices; no clean-build possible without |
| `myosu-play` binary skeleton | **HIGH** | `play:tui` Slice 1 | Slices 3, 6, 7, 8, 9 |
| Spectator socket path not confirmed against `play:tui` data dir | LOW | `agent:experience` Slice 8 | Slice 8 |
| Chain lobby query stubbed | MEDIUM | Phase 4 | Slice 7 |

---

## What This Lane Does Not Decide

This lane does **not** decide:
- Whether `agent:experience` should use an implementation-family workflow (that judgment lives in `outputs/agent/experience/review.md`, which says KEEP/proceed)
- The exact shape of the `games:traits` implementation lane beyond the robopoker migration (that is owned by the `games:traits` lane)
- Whether the SpectatorRelay should use a thread or async task for socket management (that is an implementation detail for Slice 8)

---

## Recommendation

**Execute `games:traits` Slice 1 next.** The robopoker git migration is the single highest-leverage action for the entire product. Everything downstream — `play:tui`, `agent:experience`, the full implementation of all 9 slices — is blocked on that one change.

Once `games:traits` Slice 1 is complete:
1. `play:tui` can scaffold the `myosu-play` binary (Slice 1) without path-dep failures
2. `agent:experience` Slices 1–2 can begin immediately (only depend on `tui:shell`)
3. After binary scaffold lands, `agent:experience` Slices 3+ can proceed

This lane (`agent-integration`) should be considered **complete** for the bootstrap phase. The adapter is honest, the blockers are identified, and the critical path is clear. Future integration questions (spectator WebSocket, chain-connected lobby) belong to their respective phase specs.
