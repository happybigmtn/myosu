`★ Insight ─────────────────────────────────────`
The key insight from this integration analysis: the **JSON Schema is the universal adapter** that unifies all agent-facing surfaces. Pipe mode, narration, spectator relay, and future HTTP/WS APIs all consume `GameState` — they differ only in how they render it. This means adding a new surface (e.g., HTTP API) doesn't require new game integration; it just requires a new renderer on top of the existing `GameState` contract.
`─────────────────────────────────────────────────`

## Summary

**`agent:experience` lane**: Already bootstrapped with `spec.md` and `review.md`. Review judgment: **KEEP — proceed to implementation-family workflow**.

**`agent:integration` lane**: Bootstrap complete. Produced:

| Artifact | Path |
|----------|------|
| `agent-adapter.md` | `outputs/agent-integration/agent-adapter.md` |
| `review.md` | `outputs/agent-integration/review.md` |

**Decision from reviewed artifacts**: **Implementation family next** — no further upstream unblocks needed. The lane proceeds with 5 integration slices (INT-1 through INT-5) while `games:traits` (robopoker git migration) and `play:tui` (binary skeleton) resolve their blockers in parallel.

**Lane dependency chain**:
```
games:traits (robopoker git) ──► play:tui (binary skeleton) ──► agent:integration
                                                              ├── INT-1: schema contract
                                                              ├── INT-3: --context wiring
                                                              ├── INT-4: --narrate wiring
                                                              └── INT-5: spectator relay
```