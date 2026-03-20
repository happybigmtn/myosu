# `agent:experience` Integration Adapter

## Purpose

This document is the integration shim between the `agent:experience` lane specification and the concrete codebase. It maps each surface in the spec to actual file locations, records which surfaces are already implemented vs absent, and identifies the remaining cross-lane dependencies that must be resolved before the lane can complete.

This is not a spec — it is the bridge between the spec (`outputs/agent/experience/spec.md`) and the implementation surface of `crates/myosu-tui/` and `crates/myosu-play/`.

---

## Integration Surface Map

### Source of Truth

- Lane spec: `outputs/agent/experience/spec.md`
- Source spec: `specsarchive/031626-10-agent-experience.md` (AX-01..05)
- Lane review: `outputs/agent/experience/review.md` (KEEP judgment)

### Crate Ownership

`agent:experience` lives across two crates:

| Crate | Role | Location |
|-------|------|----------|
| `myosu-tui` | Pipe mode, narration, context, journal | `crates/myosu-tui/src/` |
| `myosu-play` | Spectator relay binary | `crates/myosu-play/src/` |

The `myosu-play` binary does not exist yet at `crates/myosu-play/`. The entire binary — including the `main.rs` that would wire `--pipe`, `--context`, `--narrate`, and `--spectate` flags — must be scaffolded by the `play:tui` lane before Slices 3, 6, 7, and 8 can be implemented.

---

## Surface Status by File

### `crates/myosu-tui/src/schema.rs` — **TRUSTED** ✓

Fully implemented. 939 lines, 16 tests passing. `GameState`, `LegalAction`, `GamePhase`, `GameStateBuilder` all present and serde-serializable. This is the JSON schema backbone consumed by structured agents and by the spectator relay.

### `crates/myosu-tui/src/pipe.rs` — **PARTIAL** ⚠

The `PipeMode` struct and `run_once()` loop exist. ANSI detection works. Six unit tests pass. However, the following are absent:

- `context_path: Option<PathBuf>` field on `PipeMode` — no `--context` awareness
- `narrate: bool` field on `PipeMode` — no `--narrate` awareness
- `reflect>` prompt after `HAND COMPLETE` output — not implemented
- Lobby rendering when no `--subnet` provided — not implemented
- `info <id>` command in lobby context — not implemented
- Game state update → journal append wiring — absent

### `crates/myosu-tui/src/agent_context.rs` — **MISSING** ✗

Not present. Must be created. Schema from AX-01:

```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 0, "preferred_game": null },
  "memory": { "session_count": 0, "lifetime_result": "0bb", "observations": [] },
  "journal": []
}
```

`AgentContext` must implement:
- `load(path: &Path) -> Result<AgentContext, Error>` — read and parse JSON; if file missing, return `default()`
- `save(&self, path: &Path) -> Result<(), Error>` — write JSON; must never truncate existing content
- `default() -> AgentContext` — fresh identity with auto-generated name and empty state
- `append_journal_entry(&mut self, entry: JournalEntry)` — append to journal, no overwrite

Journal entry shape:
```json
{ "session": 1, "hand": 47, "reflection": "optional string" }
```

### `crates/myosu-tui/src/journal.rs` — **MISSING** ✗

Not present. Must be created. The journal is an append-only markdown artifact separate from the JSON context file.

`Journal` must implement:
- `new(path: PathBuf) -> Journal` — open or create `{context-dir}/journal.md`
- `append_hand_entry(&self, entry: &JournalEntry, result: &str, cards: &str) -> Result<(), Error>` — write a markdown hand section without rewriting existing content
- `append_session_summary(&self, session: u32, hands: u32, result: &str) -> Result<(), Error>` — write session summary block
- `never_truncates(&self) -> bool` — invariant: `journal.path.metadata().len()` only increases

### `crates/myosu-tui/src/narration.rs` — **MISSING** ✗

Not present. Must be created. The `NarrationEngine` translates `GameState` into atmospheric prose.

`NarrationEngine::narrate(&GameState) -> String` must include:
- Board texture analysis: "dry" (no flush draws, no straight possibilities), "wet" (opportunities present), "connected" (sequential ranks)
- Session arc: stack trajectory, opponent history from context
- Pot odds and strategic framing
- Atmosphere per AX-03 example: prose that treats the game as a story, not a data feed

### `crates/myosu-play/src/spectate.rs` — **MISSING** ✗

Not present. `crates/myosu-play/` directory is empty (binary does not exist). `SpectatorRelay` must implement:

- Unix domain socket at `~/.myosu/spectate/<session_id>.sock`
- `emit(&GameEvent)` — serialize and send JSON line; handle disconnected listeners gracefully
- Fog-of-war: hole cards NEVER sent during active play; only after `showdown` event
- Phase 1 upgrade path: WebSocket via miner axon (blocked on `chain:runtime`)

### `crates/myosu-tui/src/screens/spectate.rs` — **MISSING** ✗

Not present. `Screen::Spectate` variant must be added to `screens.rs`. Keybindings: `n` (next session), `r` (reveal hole cards after showdown), `q` (quit to lobby).

---

## Cross-Lane Dependency Analysis

### Dependency on `tui:shell`

`tui:shell` is **trusted** (82 tests pass). The `GameRenderer` trait, `Shell`, `PipeMode`, `Events`, and `Theme` are all stable. `agent:experience` builds on these without modification.

### Dependency on `games:traits`

`games:traits` is **trusted** (14 tests pass). The robopoker git migration (Slice 1) is **already complete** — `rbp-core` and `rbp-mccfr` are now git dependencies, not absolute paths. The crate compiles and tests pass without a local robopoker clone.

### Dependency on `play:tui`

`play:tui` lane **has not completed its review yet**. The `myosu-play` binary skeleton does not exist. This blocks:

| Slice | Why blocked |
|-------|-------------|
| Slice 3 (`--context` wiring) | Requires modifying `myosu-play`'s `main.rs` CLI dispatch |
| Slice 6 (`--narrate` wiring) | Requires modifying `myosu-play`'s `main.rs` CLI dispatch |
| Slice 7 (lobby) | Depends on same binary dispatch infrastructure |
| Slice 8 (`SpectatorRelay`) | `crates/myosu-play/src/spectate.rs` is inside `myosu-play` crate |
| Slice 9 (`SpectateScreen`) | Depends on binary existing |

Slices 1, 2, 4, and 5 do **not** require the binary to exist — they can be implemented as standalone `myosu-tui` modules and tested with `cargo test -p myosu-tui`.

### Dependency on `chain:runtime`

`chain:runtime` is a **soft dependency** (Phase 2). The lobby (Slice 7) queries chain for subnet info, but can be stubbed with hardcoded data for Phase 0. The spectator WebSocket upgrade (Phase 1) requires miner axon, which requires `chain:runtime`.

---

## Honest Slice Classification

### Immediately Implementable (No Binary Required)

**Slice 1: `agent_context.rs`**
- File: `crates/myosu-tui/src/agent_context.rs`
- No `myosu-play` dependency
- No `robopoker` dependency (pure `serde` types)
- Test: `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip`

**Slice 2: `journal.rs`**
- File: `crates/myosu-tui/src/journal.rs`
- No `myosu-play` dependency
- No `robopoker` dependency (file I/O only)
- Test: `cargo test -p myosu-tui journal::tests::append_hand_entry`

**Slice 4: `reflect>` prompt**
- Location: `crates/myosu-tui/src/pipe.rs`
- After `HAND COMPLETE` output, block on stdin for reflection
- Empty line skips; non-empty appends to journal via `AgentContext`
- Test: `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand`

**Slice 5: `narration.rs`**
- File: `crates/myosu-tui/src/narration.rs`
- Uses `schema.rs` types only — no `robopoker` dependency
- Can be implemented and tested standalone
- Test: `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture`

### Requires `myosu-play` Binary (Must Wait for `play:tui`)

**Slice 3: `--context` flag wiring** — needs `main.rs` CLI modification
**Slice 6: `--narrate` flag wiring** — needs `main.rs` CLI modification
**Slice 7: Lobby + game selection** — needs binary infrastructure

### Phase 3 (Requires `play:tui` binary + spectator relay)

**Slice 8: `SpectatorRelay`** — `crates/myosu-play/src/spectate.rs` inside non-existent binary
**Slice 9: `SpectateScreen`** — depends on binary existing

---

## Implementation Adapter Notes

### Adding modules to `myosu-tui`

After creating `agent_context.rs`, add to `crates/myosu-tui/src/lib.rs`:
```rust
pub mod agent_context;
pub mod journal;
pub mod narration;
```

After creating `spectate.rs`, add screen variant to `crates/myosu-tui/src/screens.rs`.

### Adding `myosu-play` binary

The `crates/myosu-play/` directory currently does not exist. The `play:tui` lane owns the binary skeleton. Once `play:tui` Slice 1 lands, the binary must include:

```rust
// crates/myosu-play/src/main.rs
use myosu_tui::pipe::PipeMode;
// ... wire --pipe, --context <path>, --narrate, --spectate flags
```

The spectator relay (`spectate.rs`) lives inside this crate at `crates/myosu-play/src/spectate.rs`.

### Testing without the binary for Slices 1-2-4-5

`cargo test -p myosu-tui` tests all `myosu-tui` modules including the new ones. The binary is only needed for end-to-end CLI flag wiring and for the spectator relay socket.
