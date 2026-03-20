`★ Insight ─────────────────────────────────────`
**Bootstrap pattern for new lanes**: When bootstrapping a new lane that synthesizes existing reviewed artifacts, the adapter document serves as the integration boundary map — it answers "what connects to what" without re-stating what each upstream lane already owns. This keeps the adapter focused on integration decisions, not duplicated spec content.

**Decision framing**: The keep/reopen/reset judgment was straightforward because `agent:experience` is already READY (KEEP). The more interesting question was "proceed or wait?" — and the answer was to proceed with the 3 unblocked slices rather than treating an external cross-lane dependency (`play:tui` binary) as a reason to pause this lane entirely.
`─────────────────────────────────────────────────`

## Summary

Bootstrapped the `agent:integration` lane — the last remaining ready product lane. Created:

**Required artifacts** (both done):
- `outputs/agent-integration/agent-adapter.md` — integration boundary map, Phase 1–4 state inventory, decision records
- `outputs/agent-integration/review.md` — judgment: **KEEP**, decision: **implementation family next**

**Decision**: The product does **not** need another upstream unblock. Slices 1 (`agent_context.rs`), 2 (`journal.rs`), and 5 (`narration.rs`) can proceed immediately — they depend only on already-trusted upstream surfaces (`tui:shell`, `games:traits`, `schema.rs`). The remaining slices are blocked on the `myosu-play` binary owned by `play:tui` lane.

**Also updated**:
- `fabro/programs/myosu-product.yaml` — added `agent:integration` lane under the `agent` unit
- `fabro/run-configs/product/agent-integration.toml` — run config for the lane
- `fabro/workflows/bootstrap/agent-integration.fabro` — workflow graph