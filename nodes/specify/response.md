The `agent-integration` lane is bootstrapped. Here's what was produced:

---

**`outputs/agent-integration/agent-adapter.md`** — the integration map that translates the `agent:experience` spec into concrete integration points:

- All 9 agent:experience slices mapped to specific files with type signatures (`agent_context.rs`, `journal.rs`, `narration.rs`, `spectate.rs`, `screens/spectate.rs`)
- CLI flag wiring for `--context`, `--narrate`, `--spectate` in `myosu-play/src/main.rs`
- `PipeMode` extension showing how `context_path`, `narrate`, `AgentContext`, `Journal`, and `NarrationEngine` compose
- Fog-of-war enforcement contract in `SpectatorRelay::emit()` — hole cards must be `None` for all pre-showdown events
- 3 integration gaps documented (context directory convention, session ID generation, GameEvent vs GameState type separation) with explicit decisions recorded
- Integration slice order table (INT-1 through INT-9) with upstream dependencies per slice

**`outputs/agent-integration/review.md`** — the integration review and decision record:

- **Judgment: KEEP** — proceed to implementation-family workflow
- **Decision: run INT-1, INT-2, and INT-5 immediately** — they have no external blockers
- **INT-3+ blocked** on `play:tui` binary skeleton (owned by `play:tui` lane)
- **`robopoker` git migration** is a testing blocker only — it does not prevent writing INT-1/INT-2 code
- Clear table of what this lane owns vs. what other lanes own

`★ Insight ─────────────────────────────────────`
The critical finding: `agent_context.rs` and `journal.rs` are pure data-structure + persistence code — they have zero dependencies on `robopoker`, `play:tui` binary, or `chain:runtime`. The `robopoker` absolute-path blocker that affects integration *testing* does not prevent integration *implementation* for these early slices. This is why the review recommends starting immediately rather than waiting for upstream resolution.
`─────────────────────────────────────────────────`