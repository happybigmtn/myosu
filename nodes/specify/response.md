`★ Insight ─────────────────────────────────────`
The key insight here is that "ready" lanes are not always uniformly ready. `agent:experience` has a **dependency split**: Slices 1–2 (`agent_context.rs`, `journal.rs`) and Slice 5 (`narration.rs`) have zero upstream blockers and can proceed immediately. But Slices 3–9 (all CLI-wired features) are genuinely blocked on `play:tui` Slice 1 delivering the `myosu-play` binary skeleton.

The second insight is that **CI blockers are not implementation blockers**. The robopoker git migration is a HIGH priority for testing reproducibility, but it doesn't prevent writing the code — local development can proceed with the existing absolute-path setup in the meantime.
`─────────────────────────────────────────────────`

Done. Created both required artifacts:

**`outputs/agent-integration/agent-adapter.md`** — Integration surface map documenting:
- The upstream dependency picture (`tui:shell` and `games:traits` trusted; `play:tui` binary absent)
- The adapter code required to wire `agent:experience` surfaces to the product
- The split workstream analysis (Workstream A: Slices 1–2, 5 — no blockers; Workstream B: Slices 3–9 — blocked on `play:tui`)

**`outputs/agent-integration/review.md`** — Honest judgment:
- **SPLIT PROCEED / HOLD**: Workstream A (Slices 1–2, 5) proceeds immediately; Workstream B (Slices 3–4, 6–9) holds until `play:tui` Slice 1 delivers the binary skeleton
- Recommendation: Form an implementation family for Workstream A now (3 agents, no upstream dependencies), hold Workstream B until binary exists, track robopoker migration separately as a CI concern not an implementation blocker