`★ Insight ─────────────────────────────────────`
The `agent-integration` lane bridges `agent:experience` outputs to the rest of the system. Key architectural decisions documented:
1. **Fog-of-war at the relay layer** — enforced at `SpectatorRelay`, not at `GameRenderer`, ensuring no hole card leakage regardless of renderer bugs
2. **Schema as canonical contract** — all agent-facing surfaces (pipe, narration, spectator events) must be representable as `GameState`
3. **Phase ordering isolation** — Phase 1–3 have no chain dependencies; Phase 4 integration is cleanly gated
`─────────────────────────────────────────────────`

## Summary

**Decision: KEEP — proceed to implementation-family workflow for product frontier**

The `agent:experience` lane has complete, reviewed artifacts. The two HIGH blockers identified in its review are owned by other lanes:
- **robopoker git migration** → `games:traits` lane (KEEP, implementation unblocked)
- **`myosu-play` binary missing** → `play:tui` lane (KEEP, ready for implementation)

Both blocking items are tracked and addressed in lanes that are themselves ready for implementation.

**Artifacts produced:**
- `outputs/agent-integration/agent-adapter.md` — Integration contracts for all 9 surfaces (pipe mode, JSON schema, context file, narration, journal, reflection, spectator relay, lobby)
- `outputs/agent-integration/review.md` — Judgment: KEEP, decision: implementation family next

**Immediate next steps:**
1. Begin `agent:experience` Slices 1–4 (no external dependencies beyond trusted `tui:shell`)
2. `play:tui` begins Slice 1 (binary skeleton) — unblocks `agent:experience` Slice 3
3. `games:traits` proceeds with Slice 1 (robopoker git migration) — unblocks CI