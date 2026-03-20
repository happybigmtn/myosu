Done. Here's what was produced:

`★ Insight ─────────────────────────────────────`
**Integration synthesis vs. lane specification**: The `agent:experience` spec/review already exist — they define *what* the lane builds. The `agent-integration` artifacts answer *when and how* to proceed by mapping slice dependencies against current upstream state. This distinction matters: a lane spec tells you the design, an integration adapter tells you the scheduling decision.

**The unblocked slice finding**: Three slices (1, 2, 5) have no external blockers — they only depend on `tui:shell` which is already trusted. This is the "honest first slice" the bootstrap doctrine calls for. The remaining slices (3–4, 6–9) correctly remain blocked behind `play:tui`'s binary skeleton, which hasn't been built yet.
`─────────────────────────────────────────────────`

## Results

**Required artifacts delivered:**

| Artifact | Path |
|----------|------|
| Agent adapter | `outputs/agent-integration/agent-adapter.md` |
| Integration review | `outputs/agent-integration/review.md` |

**Decision rendered: GO — implementation-family slices 1–2 are unblocked.**

- **Slices 1–2** (`agent_context.rs`, `journal.rs`) depend only on `tui:shell` (82 tests, trusted) — no external blockers
- **Slice 5** (`narration.rs`) is also unblocked and can run parallel to Slices 3–4
- **Critical path**: `robopoker` git migration is owned by `games:traits` — must resolve before Slices 5–9 integration testing
- **Slices 3–4, 6–9** remain blocked on `play:tui` binary skeleton (not yet executed)

**Source artifacts synthesized:**
- `outputs/agent/experience/spec.md` — lane specification (9 slices, clear boundaries)
- `outputs/agent/experience/review.md` — KEEP judgment, updated with integration note
- `fabro/programs/myosu-product.yaml` — program manifest dependency structure