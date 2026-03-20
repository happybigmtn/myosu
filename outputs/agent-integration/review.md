# `agent:integration` Lane Review

## Judgment: **SPLIT PROCEED / HOLD**

The `agent:experience` lane is not uniformly ready. It has two independent workstreams with different upstream dependency profiles:

| Workstream | Slices | Upstream blockers | Decision |
|------------|--------|------------------|----------|
| **A** | 1–2, 5 | None (`tui:shell`, `games:traits` trusted) | **PROCEED immediately** |
| **B** | 3–4, 6–9 | `play:tui` Slice 1 (binary skeleton absent) | **HOLD** until binary exists |

The `agent:experience` review at `outputs/agent/experience/review.md` says "KEEP — proceed to implementation-family workflow." That judgment is correct for Workstream A. It is optimistic for Workstream B without the `play:tui` binary skeleton first being delivered.

---

## Rationale

### Why Workstream A proceeds

1. **Upstream is trusted**: `tui:shell` (82 tests) and `games:traits` (14 tests) are both in the trusted state. Slices 1–2 (`agent_context.rs`, `journal.rs`) and Slice 5 (`narration.rs`) depend only on these trusted surfaces.

2. **Slice 1–2 are self-contained**: `agent_context.rs` is a pure data structure with `load()`, `save()`, `default()` methods and serde serialization. `journal.rs` is a markdown file writer. Neither requires the `myosu-play` binary or any CLI wiring.

3. **Slice 5 is self-contained**: `narration.rs` translates `GameState` to prose. It is a pure transformation with no I/O dependencies. It can be developed and tested entirely against the `GameState` type from `games:traits`.

4. **The implementation is additive and testable**: All three slices produce new files that do not modify existing code. They can be developed in isolation and tested with `cargo test -p myosu-tui`.

### Why Workstream B holds

1. **The `myosu-play` binary does not exist**: `crates/myosu-play/` has no code. The `play:tui` lane spec defines the binary structure, but its Slice 1 (binary bootstrap) has not been implemented. Slices 3–4, 6–9 all require CLI flag dispatch from `main.rs`.

2. **The `play:tui` review confirms this**: `outputs/play/tui/spec.md` marks the binary as **MISSING** with the note: "No crate at this path."

3. **Slices 3–4 are thin wrappers**: `--context` wiring and `reflect>` prompt are small additions to `pipe.rs`, but they require the CLI to pass the flags through to `PipeMode`. Without `main.rs`, these cannot be exercised end-to-end.

4. **Slices 6–9 have additional dependencies**: Lobby (Slice 7) requires chain query stubs; spectator relay (Slice 8) requires socket infrastructure; spectator screen (Slice 9) requires `tui:shell` screen extension. All of these are blocked on the binary existing first.

### The robopoker blocker is a testing concern, not an implementation blocker

The `games:traits` review identifies the robopoker absolute-path dependency as a **HIGH** blocker for CI. However, this does not prevent Workstream A implementation:

- Local development can proceed with the existing absolute-path setup
- The code written in Slices 1–2, 5 has no robopoker dependency directly — it depends on `tui:shell` and `games:traits`, which already have the paths configured
- The robopoker migration should be resolved in parallel by `games:traits`, but it is not a prerequisite for starting implementation

---

## Decision: What Happens Next

### Immediately (Workstream A — no blockers)

| Slice | Files | What | Proof gate |
|-------|-------|------|------------|
| Slice 1 | `crates/myosu-tui/src/agent_context.rs` | `AgentContext` with load/save/default; serde JSON; journal append | `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` |
| Slice 2 | `crates/myosu-tui/src/journal.rs` | `Journal` struct; append-only markdown; `append_hand_entry()`, `append_session_summary()` | `cargo test -p myosu-tui journal::tests::append_hand_entry` |
| Slice 5 | `crates/myosu-tui/src/narration.rs` | `NarrationEngine::narrate(&GameState)`; board texture analysis; session arc | `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture` |

### After `play:tui` Slice 1 (Workstream B — binary required)

| Slice | Files | What | Blocker |
|-------|-------|------|---------|
| Slice 3 | `crates/myosu-tui/src/pipe.rs` + `crates/myosu-play/src/main.rs` | `--context` flag wiring | `main.rs` CLI dispatch |
| Slice 4 | `crates/myosu-tui/src/pipe.rs` | `reflect>` prompt after hand | `main.rs` dispatch + `pipe.rs` |
| Slice 6 | `crates/myosu-tui/src/pipe.rs` | `--narrate` flag wiring | `main.rs` + `narration.rs` |
| Slice 7 | `crates/myosu-tui/src/pipe.rs` | Lobby + game selection | `main.rs` + chain stub |
| Slice 8 | `crates/myosu-play/src/spectate.rs` | `SpectatorRelay` Unix socket | `main.rs` + socket infra |
| Slice 9 | `crates/myosu-tui/src/screens/spectate.rs` | `SpectateScreen` | `tui:shell` screen extension |

---

## Lane Readiness Summary

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | AX-01..05 + SP-01..03 are sound; 9 slices defined |
| `tui:shell` upstream | **TRUSTED** | 82 tests pass; `GameRenderer`, `PipeMode` stable |
| `games:traits` upstream | **TRUSTED** | 14 tests pass; `CfrGame`, `GameType` stable |
| `play:tui` binary | **MISSING** | `crates/myosu-play/` has no code |
| robopoker git migration | **UNRESOLVED** | Blocks CI; does not block Workstream A implementation |
| Workstream A (1–2, 5) | **READY TO PROCEED** | No upstream blockers |
| Workstream B (3–4, 6–9) | **HOLD** | Blocked on `play:tui` Slice 1 |

---

## Recommendation

**Form an implementation family for Workstream A immediately.** The three slices (agent_context.rs, journal.rs, narration.rs) are independent of each other and can be developed in parallel by separate agents:

- **Agent 1**: `agent_context.rs` — `AgentContext` struct, load/save/default, serde, tests
- **Agent 2**: `journal.rs` — `Journal` struct, append-only markdown, hand entry formatting, tests
- **Agent 3**: `narration.rs` — `NarrationEngine`, board texture analysis, prose generation, tests

All three agents depend only on `tui:shell` (trusted) and `games:traits` (trusted). No `myosu-play` binary required.

**Do not form an implementation family for Workstream B yet.** Wait for `play:tui` Slice 1 to deliver the `myosu-play` binary skeleton. Once `main.rs` exists with `--pipe` flag dispatch, Workstream B can begin.

**Track the robopoker migration separately** as a `games:traits` owned blocker. It needs to be resolved before Workstream A slices can run in CI, but it does not prevent implementation or local testing.

---

## Update Log

- 2026-03-20: Initial review. Split judgment: Workstream A (Slices 1–2, 5) proceeds immediately; Workstream B (Slices 3–4, 6–9) holds on `play:tui` Slice 1.
