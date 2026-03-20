# `agent:integration` Lane Review

## Judgment: **KEEP — proceed to implementation family**

The `agent:experience` lane has produced honest, reviewed artifacts. The integration
adapter is now specified. The primary blockers (robopoker git migration, play:tui binary
skeleton) are owned by other lanes and tracked there. The integration slices are
well-ordered with clean dependencies. Product is ready for an implementation family
workflow — specifically, the `agent:experience` implementation slices should be
executed in the order defined in `agent-adapter.md`.

---

## Honest Assessment of the Integration Landscape

### What Exists

| Surface | Status | Evidence |
|---------|--------|---------|
| Pipe protocol (`pipe.rs`) | **Functional** | 6 tests pass; `--pipe` flag works; no `--context`, `--narrate`, `reflect>`, or lobby |
| JSON schema (`schema.rs`) | **Trusted** | 939 lines, 16 tests pass, 10 game types |
| `docs/api/game-state.json` | **Trusted** | Matches `schema.rs`; authoritative for machine-readable agents |
| `GameRenderer` trait | **Trusted** | Object-safe; `pipe_output()` contract clear; 82 tests in `tui:shell` |
| `myosu-play` binary | **Skeleton only** | `play:tui` lane owns this; CLI dispatch incomplete for new flags |

### What Is Missing

| Surface | File Needed | Blocker? |
|---------|------------|---------|
| `AgentContext` | `crates/myosu-tui/src/agent_context.rs` | **No** — slices 1–2 are independent |
| `Journal` | `crates/myosu-tui/src/journal.rs` | **No** — slices 1–2 are independent |
| `NarrationEngine` | `crates/myosu-tui/src/narration.rs` | **No** — slice 3 is independent |
| `--context` flag wiring | `crates/myosu-tui/src/pipe.rs` | **No** — depends only on `AgentContext` (slice 1) |
| `--narrate` flag wiring | `crates/myosu-tui/src/pipe.rs` | **No** — depends only on `NarrationEngine` (slice 3) |
| `reflect>` prompt | `crates/myosu-tui/src/pipe.rs` | **No** — slice 4, after context flag wiring |
| Lobby (pipe mode) | `crates/myosu-tui/src/pipe.rs` | **No** — slice 5, chain data stubbed for Phase 0 |
| `SpectatorRelay` | `crates/myosu-play/src/spectate.rs` | **No** — slice 6, independent of all above |
| `SpectateScreen` | `crates/myosu-tui/src/screens/spectate.rs` | **No** — slice 7, after SpectatorRelay |

### Upstream Blockers

| Blocker | Owned By | Impact | Resolution |
|---------|---------|--------|------------|
| robopoker git migration | `games:traits` lane | Cannot `cargo build` on clean checkout or CI | Must complete before Phase 1 integration testing |
| `myosu-play` binary skeleton | `play:tui` lane | CLI dispatch incomplete for `--pipe` with new flags | Slice 1 of `play:tui` must complete before `agent:experience` slices 3+ |
| Chain data for lobby | `chain:runtime` lane | Lobby displays stubbed data until miner axon exists | Stubbed for Phase 0; acceptable |
| Miner axon for spectator WS | `chain:runtime` lane | Phase 1 of spectator protocol | Acceptable for Phase 0 |

---

## Decision: Implementation Family vs. Another Upstream Unblock

**Decision: Proceed to implementation family.**

Rationale:

1. **`agent:experience` spec is mature and reviewed.** The 9 slices are defined with clear
   boundaries and sequential dependencies. The schema, pipe protocol, and trait contracts
   are sound. No further spec work is blocking implementation.

2. **The remaining upstream blockers are owned elsewhere and tracked.** The robopoker
   migration is owned by `games:traits` and is being addressed there. The `myosu-play`
   binary skeleton is owned by `play:tui`. Both are tracked in their respective lane
   `review.md` files.

3. **Slices 1–2 can begin immediately.** `AgentContext` and `Journal` are independent
   of both the binary skeleton and the robopoker migration. They depend only on the
   `GameRenderer` trait which is trusted and stable.

4. **The implementation family should follow the slice order in `agent-adapter.md`.**
   The natural grouping is:
   - **Implementation Family A (Context + Journal)**: Slices 1–2 — `agent_context.rs` + `journal.rs` + `--context` wiring
   - **Implementation Family B (Narration)**: Slices 3–4 — `narration.rs` + `--narrate` + `reflect>` prompt
   - **Implementation Family C (Lobby)**: Slice 5 — lobby in pipe mode (stubbed)
   - **Implementation Family D (Spectator)**: Slices 6–7 — relay + screen

5. **The decision to use an implementation family (vs. another upstream unblock) is
   supported by the `agent:experience` review judgment.** That review explicitly stated:
   "Proceed to implementation-family workflow next. The lane is well-specified and
   the upstream is trusted."

---

## Lane Readiness

| Dimension | Status | Notes |
|---------|--------|-------|
| Integration specification | **READY** | `agent-adapter.md` specifies all surfaces and integration points |
| Upstream (tui:shell) | **TRUSTED** | `GameRenderer`, `PipeMode` stable; 82 tests pass |
| Upstream (games:traits) | **TRUSTED** | `CfrGame`, `GameType`, `StrategyQuery` stable; 14 tests pass |
| Upstream (schema.rs) | **TRUSTED** | 16 tests pass; 10 game types |
| Binary skeleton (`play:tui`) | **PARTIAL** | Needed for slices 3+; not blocking slices 1–2 |
| robopoker dependency | **BLOCKER** | Must migrate to git before Phase 1 integration testing |
| Chain data (lobby) | **STUBBED** | Acceptable for Phase 0; real data requires `chain:runtime` |

---

## Proof Expectations

| Slice | Surface | Test |
|-------|---------|------|
| 1 | `AgentContext` | `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` |
| 1 | `AgentContext` | `cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites` |
| 1 | `AgentContext` | `cargo test -p myosu-tui agent_context::tests::missing_context_creates_new` |
| 2 | `--context` wiring | `cargo test -p myosu-tui pipe::tests::context_preserved_across_sessions` |
| 3 | `NarrationEngine` | `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture` |
| 3 | `NarrationEngine` | `cargo test -p myosu-tui narration::tests::narrate_includes_session_context` |
| 3 | `NarrationEngine` | `cargo test -p myosu-tui narration::tests::terse_and_narrate_same_game_state` |
| 4 | `reflect>` | `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand` |
| 4 | `reflect>` | `cargo test -p myosu-tui pipe::tests::empty_reflection_skips` |
| 4 | `reflect>` | `cargo test -p myosu-tui pipe::tests::reflection_saved_to_journal` |
| 5 | Lobby | `cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag` |
| 5 | Lobby | `cargo test -p myosu-tui pipe::tests::info_command_in_lobby` |
| 5 | Lobby | `cargo test -p myosu-tui pipe::tests::selection_starts_game` |
| 6 | `SpectatorRelay` | `cargo test -p myosu-play spectate::tests::relay_emits_events` |
| 6 | `SpectatorRelay` | `cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener` |
| 6 | `SpectatorRelay` | `cargo test -p myosu-play spectate::tests::events_are_valid_json` |
| 7 | `SpectateScreen` | `cargo test -p myosu-tui spectate::tests::renders_fog_of_war` |
| 7 | `SpectateScreen` | `cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown` |
| 7 | `SpectateScreen` | `cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play` |

---

## Integration Risks

### Risk 1: Socket Path Convention Not Aligned
**Status**: LOW — needs verification before Slice 6

The `SpectatorRelay` uses `~/.myosu/spectate/<session_id>.sock`. The `play:tui`
lane uses `{data-dir}/hands/hand_{N}.json`. If these conventions diverge in the base
path, the spectator socket path must align.

**Action**: Verify `play:tui` data directory convention before Slice 6 implementation.

### Risk 2: robopoker Path Dependency Blocks Integration Testing
**Status**: HIGH — blocks Phase 1 testing

The absolute path dependencies in `games:traits` prevent clean checkout builds.
Until `cargo fetch` succeeds without local robopoker, no integration testing
can occur in CI or on fresh checkouts.

**Action**: `games:traits` lane must complete Slice 1 before Phase 1 testing.

### Risk 3: `--narrate` Dual Rendering Divergence
**Status**: LOW — testable in slice 3

The same `GameState` must render identically in both `--narrate` and default pipe
mode (same game state, different rendering layer). The `NarrationEngine` must not
introduce state changes or filtering.

**Action**: `terse_and_narrate_same_game_state` test in slice 3.

### Risk 4: Journal Append is Crash-Adjacent
**Status**: MEDIUM — depends on error handling design

The journal is opened in append mode and written after each hand. If the process
crashes mid-write, the journal file may be in an incomplete state. Currently no
journal fsync or write-ahead logging.

**Action**: Document the crash behavior as acceptable for Phase 0; consider
journal integrity for Phase 1.

---

## Recommendation

**Proceed to `agent:experience` implementation family.** The integration is well-specified,
the upstream is trusted, and the blockers are tracked and owned elsewhere.

The first implementation family should execute Slices 1–2 (`AgentContext` + `Journal` +
`--context` wiring) as a single atomic unit, since they share the same file (`agent_context.rs`)
and the journal appends from within the context save path.

After Slices 1–2 complete, update this `review.md` with the evidence of test passes
and assess whether the next family (Slice 3 `NarrationEngine`) can proceed in parallel
with other product lanes or should be sequenced after the robopoker migration is confirmed.
