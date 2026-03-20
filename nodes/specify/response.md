
`★ Insight ─────────────────────────────────────`
The `agent:integration` lane is fundamentally a **documentation-first integration specification** — it doesn't write code but specifies how the agent-facing surfaces (pipe protocol, JSON schema, agent context, journal, narration, spectator relay) fit together as a coherent system.

Key architectural decisions made in `agent-adapter.md`:
1. **Integration flows upward** — `PipeMode` is the driver that holds `AgentContext`, which holds `Journal`, and both use `NarrationEngine` for rendering
2. **Fog-of-war is enforced at the relay, not the renderer** — this is the correct separation: the game state renderer doesn't know about spectators; the relay filters before emitting
3. **Spectator socket path convention** (`~/.myosu/spectate/<session_id>.sock`) is noted as needing verification against `play:tui`'s data directory convention before Slice 6

The `review.md` decision was: **proceed to implementation family** — not another upstream unblock. The `agent:experience` spec is mature, the upstream blockers are owned elsewhere and tracked, and Slices 1-2 (AgentContext + Journal) can begin immediately without waiting for the robopoker migration or play:tui binary skeleton.
`─────────────────────────────────────────────────`

---

**Lane artifacts produced:**

| Artifact | Path |
|----------|------|
| Integration adapter spec | `outputs/agent-integration/agent-adapter.md` |
| Honest review | `outputs/agent-integration/review.md` |

**Lane judgment: KEEP — proceed to implementation family**

**Decision reached**: Product needs an **implementation family** next (specifically, `agent:experience` Slices 1-2: `AgentContext` + `Journal` + `--context` wiring). The remaining upstream blockers are owned elsewhere and tracked in their respective lane `review.md` files:
- robopoker git migration → `games:traits` lane
- `myosu-play` binary skeleton → `play:tui` lane
- Chain data for lobby → `chain:runtime` lane (stubbed for Phase 0, acceptable)