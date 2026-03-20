`★ Insight ─────────────────────────────────────`
The `agent:integration` lane doesn't implement agent features — it decides **whether** and **when** the `agent:experience` implementation family can start. This is a critical control-plane function: aggregating the reviewed artifacts from multiple upstream lanes (`agent:experience`, `play:tui`) to make a judgment call. The key finding is that `agent:experience` Slices 1-2 are genuinely unblocked (they use only `serde` + `std`, no binary or robopoker dependency), so implementation can proceed immediately in parallel with `play:tui` Slice 1 completing.
`─────────────────────────────────────────────────`

---

## Lane Artifacts Produced

**`outputs/agent-integration/agent-adapter.md`** — Integration contract documenting:
- What `agent:experience` consumes from upstream lanes (`tui:shell`, `games:traits`, `play:tui` binary)
- What it produces across all 9 slices
- CLI wiring contract (`--pipe`, `--context`, `--narrate`, `--spectate`)
- Slice dependency map with clear UNBLOCKED / BLOCKED / STUBBED labels
- Session persistence contract and spectator relay fog-of-war enforcement point

**`outputs/agent-integration/review.md`** — Decision record:
- **Verdict: PROCEED** — implementation family for product frontier
- Confirms both `agent:experience` (KEEP) and `play:tui` (KEEP) are reviewed
- Documents 3 of 9 slices as immediately unblocked (Slices 1, 2, 5)
- Identifies remaining cross-lane blockers: `robopoker` git migration (`games:traits` owns), `myosu-play` binary skeleton (`play:tui` owns)
- Triggers: implementation slices 1–2 launch now; `play:tui` notified of Slice 3+ dependency

**Key Decision**: Launch `agent:experience` implementation family now rather than waiting for all blockers to clear, because Slices 1–2 have zero external dependencies beyond already-trusted `tui:shell`.