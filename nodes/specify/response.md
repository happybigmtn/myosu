---

**Execution complete.** Two artifacts produced at `outputs/agent-integration/`:

| Artifact | Purpose |
|----------|---------|
| `agent-adapter.md` | Integration contracts: pipe protocol, JSON schema, context file, reflection channel, narration, spectator relay |
| `review.md` | **KEEP** — proceed to implementation-family workflow |

**Decision made**: Implementation family is the right next step. Not an upstream unblock.

`★ Insight ─────────────────────────────────────`
- **Parallelization is the key insight**: `agent:experience` Slices 1–2 (agent_context.rs, journal.rs) depend only on trusted `tui:shell` and can start immediately. `play:tui` Slice 1 (binary skeleton) runs concurrently and unblocks `agent:experience` Slice 3 onwards.
- **Robopoker is owned elsewhere**: The git migration in `games:traits` is tracked as in-progress risk there, not a gate here. `agent:experience` doesn't call robopoker directly in Phase 1.
- **Fog-of-war enforcement location**: Hole card privacy is enforced at `SpectatorRelay::emit()` (the relay process boundary), not at the screen renderer. This is the correct layer — the relay is the last place we can guarantee no data leaks before it leaves the process.
`─────────────────────────────────────────────────`