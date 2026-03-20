# `agent:integration` Lane Review

## Judgment: **KEEP** — proceed to implementation family (Phase 1 slices 1, 2, 5)

The product does **not** need another upstream unblock. The `agent:experience` lane has been reviewed and is READY. The immediate implementation slices can proceed without any upstream dependency resolution. The remaining slices are blocked on `play:tui` binary, which is owned by a separate lane.

---

## Decision: Implementation Family Next

After reviewing the `agent:experience` lane artifacts (`spec.md` and `review.md`), the decision is:

**Proceed to implementation of Phase 1 slices that have no upstream blockers.**

| Slice | What | Upstream Blocker | Can Proceed |
|-------|------|-----------------|-------------|
| Slice 1: `agent_context.rs` | `AgentContext` load/save/default | `tui:shell` (trusted) | **YES** |
| Slice 2: `journal.rs` | Append-only markdown writer | `tui:shell` (trusted) | **YES** |
| Slice 5: `narration.rs` | `NarrationEngine::narrate(&GameState)` | `games:traits` (trusted) + `schema.rs` (trusted) | **YES** |
| Slice 3: `--context` wiring | CLI flag → `PipeMode` | `myosu-play` binary missing | NO |
| Slice 4: `reflect>` prompt | stdin block after hand | `myosu-play` binary missing | NO |
| Slice 6: `--narrate` wiring | CLI flag → `PipeMode` | Slice 3 dependency | NO |
| Slice 7: Lobby (Phase 0) | Hardcoded subnet list | `myosu-play` binary missing | NO |
| Slice 8: `SpectatorRelay` | Unix socket relay | `myosu-play` binary missing | NO |
| Slice 9: `SpectateScreen` | Fog-of-war TUI | `myosu-play` binary + Slice 8 | NO |

**The `play:tui` lane owns the `myosu-play` binary skeleton (Slice 1 of `play:tui`).** Until that binary exists, Slices 3–4 and 6–9 cannot be wired. This is a cross-lane dependency, not an upstream unblock within the product frontier.

---

## Rationale

### Why not another upstream unblock?

The remaining blockers (`myosu-play` binary, `chain:runtime`, `miner:service`) are **not owned by this lane**. They are owned by `play:tui`, `chain:runtime`, and `miner:service` respectively. This lane cannot unblock them — only the owning lanes can.

The correct response to a blocker you don't own is **not** to wait, but to work on the slices that don't depend on the blocker. Three slices (1, 2, 5) have no upstream unblock required.

### Why not wait for `play:tui`?

`play:tui` is a separate lane with its own milestones. This lane can make independent progress on `agent_context.rs`, `journal.rs`, and `narration.rs` — all of which are pure Rust library code in `crates/myosu-tui/src/` that does not require the `myosu-play` binary to exist to be implemented and tested.

### Why these three slices specifically?

- **Slice 1 (`agent_context.rs`)**: Pure `serde_json` + `AgentContext` struct. Can be developed against the `tui:shell` trusted interfaces. Tests can mock the `PipeMode` interaction.
- **Slice 2 (`journal.rs`)**: Pure file I/O + markdown formatting. Same situation — no binary required to develop or test.
- **Slice 5 (`narration.rs`)**: Pure `GameState → String` transformation. The `GameState` type is fully defined and trusted in `schema.rs`. No binary required.

---

## Cross-Lane Dependency Notes

| Blocker | Owner | Impact on This Lane | Recommended Action |
|---------|-------|---------------------|-------------------|
| `myosu-play` binary missing | `play:tui` | Slices 3–4, 6–9 cannot be wired | Monitor `play:tui` lane; begin these slices as soon as binary exists |
| `robopoker` git migration | `games:traits` | Affects `tui:shell` and `games:traits` which this lane depends on | Monitor; no action needed until integration testing phase |
| `chain:runtime` restart | `chain:runtime` | Phase 4 lobby and spectator WS blocked | Monitor; no action needed until Phase 4 |

---

## Lane Readiness Assessment

| Dimension | Status | Notes |
|-----------|--------|-------|
| Source spec (`agent:experience`) | **READY** | KEEP judgment; 9 slices defined; upstream trusted |
| Immediate implementation surfaces | **READY** | Slices 1, 2, 5 have no upstream blockers |
| Cross-lane dependency (`play:tui`) | **BLOCKED** | Slices 3–4, 6–9 blocked; owned externally |
| Phase 4 surfaces | **BLOCKED** | Blocked on `chain:runtime` and `miner:service` |
| Integration test path | **DEFINED** | Phase 1 can use `cargo test -p myosu-tui` with mocked binary |
| Proof gates | **DEFINED** | Each slice has a specific `cargo test` command in `agent:experience/spec.md` |

---

## Recommendation

Begin implementation of Slices 1, 2, and 5 immediately under this lane. These are self-contained, can be developed and tested against the trusted upstream surfaces, and do not require the `play:tui` binary to exist.

Track `play:tui` lane progress separately. As soon as the `myosu-play` binary skeleton exists, Slices 3–4 and 6–9 can proceed in rapid succession.

Do **not** treat the `play:tui` blocker as a reason to pause — the three unblocked slices represent real, valuable progress on the agent-facing surfaces.
