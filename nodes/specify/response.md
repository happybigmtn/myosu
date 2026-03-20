`★ Insight ─────────────────────────────────────`
The `agent-integration/` directory is distinct from the lane's own `outputs/agent/experience/` directory:
- **`outputs/agent/experience/`** — the lane's own spec and self-assessment (what the lane *is*)
- **`outputs/agent-integration/`** — the integration contract and forward-looking decision (how the lane *fits into the product*)

This separation is intentional. The lane spec captures what the lane will build. The integration adapter captures how the built surfaces connect to the rest of Myosu. The integration review answers "what should product do next" — a question the lane's own review doesn't address because it stays within lane boundaries.
`─────────────────────────────────────────────────`

---

## Execution Complete

**`agent:experience` lane** — last remaining ready product lane — has been honestly assessed and its artifacts are now produced.

### Required Durables

| Artifact | Path | Lines |
|----------|------|-------|
| Integration contract | `outputs/agent-integration/agent-adapter.md` | 180 |
| Forward-looking review | `outputs/agent-integration/review.md` | 117 |

### Decision

**IMPLEMENTATION — start Slices 1–2 now; coordinate upstream unblocks for Slices 5–9**

The honest read from the lane's own reviewed artifacts:

- **Slices 1–2 are ready now**: `agent_context.rs` and `journal.rs` depend only on the trusted `tui:shell` (82 tests). No robopoker, no binary, no chain.
- **Slices 3–9 are blocked** by other lanes: `play:tui` owns the binary skeleton; `games:traits` owns the robopoker git migration. These are not `agent:experience`'s blockers to resolve.
- **The `implement/` workflow family** is the right fit — bounded slices with clear proof commands, same family `games:traits` uses.

### Coordination Actions

| Action | Owner |
|--------|-------|
| Confirm robopoker git migration timeline | `games:traits` lane |
| Confirm `myosu-play` binary skeleton timeline | `play:tui` lane |
| Slice 1: `agent_context.rs` | `agent:experience` |
| Slice 2: `journal.rs` | `agent:experience` |

The product does **not** need another upstream unblock before starting — it needs to start the two implementable slices and coordinate with the two blocking lanes in parallel.