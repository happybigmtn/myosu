# `agent:experience` Lane Specification

## Purpose and User-Visible Outcome

`agent:experience` is the **agent-facing presentation layer** for Myosu. It owns every surface through which programmatic agents — LLMs, bots, scripts — perceive and act upon the game world.

The lane delivers:

1. **`--pipe` mode** — a plain-text stdin/stdout protocol (`agent | myosu-play --pipe | opponent`) built on `GameRenderer::pipe_output()`, with two new flags: `--context <path>` and `--narrate`
2. **JSON schema** — `docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs` (fully implemented, trusted) — the machine-readable game state consumed by structured agents
3. **Agent context file** — a JSON file (`--context <path>`) providing persistent identity, memory, and journal across sessions
4. **Reflection channel** — a `reflect>` prompt after each hand in pipe mode; skippable via empty line; appended to the journal
5. **Rich narration mode** — `--narrate` flag that renders game state as atmospheric prose instead of terse key-value pairs
6. **Agent journal** — append-only markdown artifact recording every hand and the agent's reflections
7. **Game selection** — lobby presented in pipe mode (no `--subnet` flag) so agents can choose which game to enter
8. **Spectator relay** — Phase 0 local Unix-domain socket event stream (`AC-SP-01`); Phase 1 WebSocket via miner axon

**User-visible behavior**: An agent connects via `myosu-play --pipe --context ./koan.json --narrate`. It receives narrated game state, plays hands, writes optional reflections, and builds a persistent journal over sessions. A separate `myosu-play --spectate` client watches the same session over a Unix socket.

---

## Lane Boundary

```
                            agent:experience (THIS LANE)
                            ┌──────────────────────────────────────────────────────┐
upstream                    │                                                      │
tui:shell ────────────────► │  GameRenderer (trait)  ──► pipe_output()           │
  (82 tests, trusted)       │  pipe.rs (PipeMode driver)                         │
                            │  --context flag → agent_context.rs (new)            │
upstream                    │  --narrate flag → narration.rs (new)               │
games:traits ─────────────► │  journal.rs (new)                                  │
  (14 tests, trusted)       │                                                      │
                            │  ┌──────────────────────────────────────────────┐  │
upstream                    │  │ docs/api/game-state.json + schema.rs          │  │
spectator-protocol ────────► │  │ (GameState JSON schema — fully implemented) │  │
  (AX-01..05 + SP-01)       │  └──────────────────────────────────────────────┘  │
                            │  ┌──────────────────────────────────────────────┐  │
untrusted                   │  │ SpectatorRelay (AC-SP-01)                     │  │
miner axon (future) ───────► │  │ Unix socket → future WS upgrade             │  │
                            │  └──────────────────────────────────────────────┘  │
                            │  ┌──────────────────────────────────────────────┐  │
                            │  │ myosu-play binary (extends play:tui binary)  │  │
                            │  │ --pipe --context <path> --narrate           │  │
                            │  └──────────────────────────────────────────────┘  │
                            └──────────────────────────────────────────────────────┘
```

**Trusted upstream inputs:**
- `tui:shell` (82 tests pass) — `Shell`, `ScreenManager`, `GameRenderer` trait, `PipeMode`, `Events`, `Theme`
- `games:traits` (14 tests pass) — `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response`
- `spectator-protocol` (AX-01..05, SP-01) — specification only; not yet implemented

**Untrusted inputs** (validated at use site):
- Agent-supplied context JSON file (serde-validated before use; never exposed to opponents)
- Agent-supplied reflection text (free-form string; no parsing required)
- Spectator relay socket events (fog-of-war enforced at relay, not renderer)

**Trusted downstream outputs:** None — `agent:experience` is a terminal lane

---

## Current Implementation Status

| Surface | Status | Evidence |
|---------|--------|---------|
| `docs/api/game-state.json` | **TRUSTED** | Complete JSON schema with 10 game types, exhaustive `legal_actions` |
| `crates/myosu-tui/src/schema.rs` | **TRUSTED** | Full Rust implementation with `GameStateBuilder`, `LegalAction` enum, 16 tests passing |
| `GameRenderer::pipe_output()` contract | **TRUSTED** | Trait method exists; `pipe.rs` driver skeleton exists; no narration yet |
| `pipe.rs` — `PipeMode` driver | **TRUSTED** | 6 tests pass; `--pipe` flag exists; no `--context`, `--narrate`, `reflect>`, lobby |
| `crates/myosu-tui/src/agent_context.rs` | **MISSING** | No file at this path |
| `crates/myosu-tui/src/narration.rs` | **MISSING** | No file at this path |
| `crates/myosu-tui/src/journal.rs` | **MISSING** | No file at this path |
| `--context` flag wiring | **MISSING** | Not in `pipe.rs` or `main.rs` |
| `--narrate` flag wiring | **MISSING** | Not in `pipe.rs` or `main.rs` |
| `reflect>` prompt after hand | **MISSING** | Not in `pipe.rs` |
| Lobby + game selection in pipe mode | **MISSING** | Not in `pipe.rs` |
| `SpectatorRelay` (AC-SP-01) | **MISSING** | No `spectate.rs` |
| `SpectateScreen` (AC-SP-02) | **MISSING** | No `screens/spectate.rs` |

---

## Broken or Missing Surfaces

### 1. `agent_context.rs` — Completely Absent

The agent context file is the foundation of agent persistence. Without it, agents are ephemeral functions — they have no memory of previous sessions, no journal, and no identity.

The context file schema (from `specsarchive/031626-10-agent-experience.md` AC-AX-01):
```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [...] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

**Impact**: Agents cannot accumulate experience across sessions.
**Required**: `agent_context.rs` with load/save, journal append, and default identity creation.

### 2. `narration.rs` — Completely Absent

The `--narrate` flag requires a `narration.rs` engine that translates `GameState` into atmospheric prose. This is distinct from `pipe_output()` — where pipe mode is terse key-value for fast parsing, narration is a story for experienced consumption.

Example narrated output (from AX-03):
```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
...
```

**Impact**: No `--narrate` mode available; agents only get terse output.
**Required**: `narration.rs` with board texture analysis, session arc, and opponent tendency weaving.

### 3. `journal.rs` — Completely Absent

The agent journal is append-only markdown — the agent's autobiography. It is not a log; it is a narrative artifact that grows with every session.

**Impact**: No persistent narrative of agent experience.
**Required**: `journal.rs` writing `{context-dir}/journal.md`; never truncates; each hand produces a markdown entry with board, held cards, result, and optional reflection.

### 4. `--context` and `--narrate` Flag Wiring — Absent

Neither flag is wired into `pipe.rs` or `myosu-play`'s CLI. The pipe mode driver (`PipeMode`) has no awareness of context files or narration.

**Impact**: The design for `--context` and `--narrate` exists in AC-AX-01 and AC-AX-03 but has no implementation.
**Required**: Add flags to `myosu-play` CLI; pass through to `PipeMode`; conditionally load context on init, save on shutdown.

### 5. `reflect>` Prompt After Hand — Absent

The reflection channel is AC-AX-02's core feature. After each hand completes, pipe mode should output:
```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```

An empty line skips the reflection. A non-empty response is appended to the journal entry.

**Impact**: Agents cannot reflect on their play; journal entries lack inner monologue.
**Required**: After hand-complete output, block on stdin for reflection; append to journal if non-empty.

### 6. Lobby + Game Selection in Pipe Mode — Absent

AC-AX-05: When `--pipe` is used without `--subnet`, the agent receives the lobby and can choose which game to enter. Currently, no such lobby exists in pipe mode.

```
MYOSU/LOBBY
subnets:
  1 nlhe-hu    12 miners  13.2 mbb/h  ACTIVE
  2 nlhe-6max  18 miners  15.8 mbb/h  ACTIVE
>
```

**Impact**: Agents cannot choose which game to play; must always be told which subnet.
**Required**: Lobby rendering in pipe mode; `info <id>` command; subnet selection starts game.

### 7. `SpectatorRelay` (AC-SP-01) — Absent

Phase 0 of the spectator protocol: a local Unix-domain socket relay that emits JSON event lines from an active session. Fog-of-war is enforced at the relay (hole cards never sent during play).

**Impact**: No spectator mode for watching agent vs agent play.
**Required**: `spectate.rs` with `SpectatorRelay::emit()`; socket at `~/.myosu/spectate/<session_id>.sock`.

### 8. `SpectateScreen` (AC-SP-02) — Absent

The spectator TUI screen: renders game events from the relay socket with fog-of-war (hole cards shown as `·· ··` during play). After showdown, `r` key reveals hole cards.

**Impact**: No spectator UI.
**Required**: `screens/spectate.rs`; `Screen::Spectate` variant; `n`, `r`, `q` keybindings.

---

## Code Boundaries and Deliverables

| File | Responsibility | Status |
|------|---------------|--------|
| `crates/myosu-tui/src/agent_context.rs` | `AgentContext` struct; load/save JSON; default identity; journal append | **MISSING** |
| `crates/myosu-tui/src/narration.rs` | `NarrationEngine`; board texture analysis; session arc; prose generation | **MISSING** |
| `crates/myosu-tui/src/journal.rs` | `Journal` struct; append-only markdown writer; hand entry formatter | **MISSING** |
| `crates/myosu-tui/src/pipe.rs` | Extend `PipeMode` with `--context`, `--narrate`; add `reflect>` prompt; add lobby | **PARTIAL** |
| `crates/myosu-play/src/spectate.rs` | `SpectatorRelay` (AC-SP-01); event emission; socket management | **MISSING** |
| `crates/myosu-tui/src/screens/spectate.rs` | `SpectateScreen` (AC-SP-02); fog-of-war rendering; keybindings | **MISSING** |
| `docs/api/game-state.json` | JSON Schema for all 20 game types | **TRUSTED** |
| `crates/myosu-tui/src/schema.rs` | Rust `GameState`, `LegalAction`, `GamePhase`, builder | **TRUSTED** |

---

## How Pipe Mode, JSON Schema, HTTP/WS APIs, and Narration Fit Together

```
                    ┌─────────────────────────────────────────────┐
                    │           agent:experience lane              │
                    │                                              │
  agent             │  ┌─────────────┐   ┌──────────────────────┐ │
  (LLM/bot/ ──────► │  │ PipeMode    │   │ Future HTTP/WS API  │ │
  script)           │  │ (stdin/out)  │   │ (Phase 2, blocked on │ │
                    │  │             │   │  chain:runtime)      │ │
                    │  └──────┬──────┘   └──────────┬───────────┘ │
                    │         │                      │              │
                    │  ┌──────┴──────────────────────┴──────────┐ │
                    │  │         GameRenderer (trait)            │ │
                    │  │                                          │ │
                    │  │  pipe_output() ──► terse key-value text  │ │
                    │  │  render_state() ──► ratatui buffer       │ │
                    │  └──────────────────┬─────────────────────┘ │
                    │                     │                       │
                    │         ┌───────────┴───────────┐           │
                    │         │                       │           │
                    │  ┌──────┴──────┐    ┌─────────┴─────────┐  │
                    │  │ narration.rs │    │   schema.rs        │  │
                    │  │ (prose, when│    │ (GameState JSON,   │  │
                    │  │  --narrate) │    │  trusted)          │  │
                    │  └─────────────┘    └────────────────────┘  │
                    │                                              │
                    │  ┌─────────────────────────────────────────┐│
                    │  │ agent_context.rs + journal.rs            ││
                    │  │ (persistent identity, memory, journal)   ││
                    │  └─────────────────────────────────────────┘│
                    │                                              │
                    │  ┌─────────────────────────────────────────┐│
                    │  │ SpectatorRelay (Unix socket, Phase 0)   ││
                    │  │ → miner axon WS (Phase 1)               ││
                    │  └─────────────────────────────────────────┘│
                    └─────────────────────────────────────────────┘
```

**Layer ordering** (data flow from game to agent):
1. **Game state** (from `games:traits` + `robopoker`) → **`schema.rs`** (`GameState`) validates and serializes
2. **`GameRenderer::pipe_output()`** renders the validated state as terse text for pipe mode
3. **`narration.rs`** wraps the `GameState` in atmospheric prose when `--narrate` is set
4. **`agent_context.rs`** loads the agent's persistent context at startup and saves it on shutdown
5. **`journal.rs`** appends hand entries + reflections to the agent's markdown journal
6. **`SpectatorRelay`** emits game events to a Unix socket; Phase 1 upgrades to miner axon WebSocket

**HTTP/WS APIs are Phase 2** — they are not implemented yet. They are mentioned here because they are the natural evolution of the pipe protocol: the same `GameState` JSON schema that powers `pipe_output()` will power the WebSocket event stream. The spectator protocol spec (AC-SP-01) explicitly calls this out as the Phase 1 extension path.

---

## Proof / Check Shape for the Lane

The lane is **proven** when all of the following pass:

```
# Agent context roundtrips across sessions
cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip
cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites
cargo test -p myosu-tui agent_context::tests::missing_context_creates_new

# Reflection channel works
cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand
cargo test -p myosu-tui pipe::tests::empty_reflection_skips
cargo test -p myosu-tui pipe::tests::reflection_saved_to_journal

# Narration produces board texture + session arc
cargo test -p myosu-tui narration::tests::narrate_includes_board_texture
cargo test -p myosu-tui narration::tests::narrate_includes_session_context
cargo test -p myosu-tui narration::tests::terse_and_narrate_same_game_state

# Journal is append-only and valid markdown
cargo test -p myyosu-tui journal::tests::append_hand_entry
cargo test -p myyosu-tui journal::tests::append_session_summary
cargo test -p myyosu-tui journal::tests::never_truncates

# Lobby game selection
cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag
cargo test -p myosu-tui pipe::tests::info_command_in_lobby
cargo test -p myosu-tui pipe::tests::selection_starts_game

# Spectator relay (Phase 0)
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener
cargo test -p myosu-play spectate::tests::events_are_valid_json

# Spectator TUI screen
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown
cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play

# Schema remains trusted
cargo test -p myosu-tui schema::tests
```

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1: `agent_context.rs` — Identity and Memory
**Files**: `crates/myosu-tui/src/agent_context.rs`
**What**: `AgentContext` struct with `load()`, `save()`, `default()`; identity/memory/journal fields; serde serialization; roundtrip test; missing file creates new default.
**Proof gate**: `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip`

### Slice 2: `journal.rs` — Append-Only Markdown Artifact
**Files**: `crates/myosu-tui/src/journal.rs`
**What**: `Journal` struct wrapping a markdown file; `append_hand_entry()` with board/held/result/reflection; `append_session_summary()`; `never_truncates()` invariant; called from pipe mode after each hand.
**Proof gate**: `cargo test -p myosu-tui journal::tests::append_hand_entry`

### Slice 3: `--context` Flag Wiring in `PipeMode`
**Files**: `crates/myosu-tui/src/pipe.rs`
**What**: Add `context_path: Option<PathBuf>` to `PipeMode`; load context on `new()`; save context on drop; wire `--context` flag to `myosu-play` CLI.
**Proof gate**: Agent plays 10 hands, shuts down, restarts → memory preserved

### Slice 4: `reflect>` Prompt After Hand
**Files**: `crates/myosu-tui/src/pipe.rs`
**What**: After `HAND COMPLETE` output in pipe mode, block on stdin for reflection; empty line = skip; non-empty = append to `journal.rs` entry; test reflection prompt appears, empty skips, non-empty is saved.
**Proof gate**: `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand`

### Slice 5: `narration.rs` — Rich Prose Engine
**Files**: `crates/myosu-tui/src/narration.rs`
**What**: `NarrationEngine::narrate(&GameState) -> String`; board texture analysis ("dry", "wet", "connected"); session arc (stack trajectory, opponent history from context); atmospheric prose format per AX-AX-03 example.
**Proof gate**: `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture`

### Slice 6: `--narrate` Flag Wiring
**Files**: `crates/myosu-tui/src/pipe.rs`
**What**: Add `narrate: bool` to `PipeMode`; when true, use `NarrationEngine` instead of `pipe_output()`; wire `--narrate` flag to CLI.
**Proof gate**: `--narrate` produces prose with board texture + session arc; underlying game state identical in both modes

### Slice 7: Lobby + Game Selection in Pipe Mode
**Files**: `crates/myosu-tui/src/pipe.rs`
**What**: When no `--subnet` provided, render lobby; `info <id>` command shows subnet details; subnet selection starts the game; wire to chain discovery (stubbed for Phase 0).
**Proof gate**: `cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag`

### Slice 8: `SpectatorRelay` (AC-SP-01)
**Files**: `crates/myosu-play/src/spectate.rs`
**What**: `SpectatorRelay` struct with `emit(&GameEvent)`; Unix domain socket at `~/.myosu/spectate/<session_id>.sock`; fog-of-war enforced (no hole cards during play); `GameEvent` type matching AX-01 JSON schema; handles disconnected listeners gracefully.
**Proof gate**: `cargo test -p myosu-play spectate::tests::relay_emits_events`

### Slice 9: `SpectateScreen` (AC-SP-02)
**Files**: `crates/myosu-tui/src/screens/spectate.rs`
**What**: `Screen::Spectate` variant; connects to relay socket; renders events with fog-of-war; `r` key reveals hole cards after showdown; `n` switches sessions; `q` returns to lobby.
**Proof gate**: `cargo test -p myosu-tui spectate::tests::renders_fog_of_war`

---

## Dependency on Other Lanes

| Lane | Type | What Is Used |
|------|------|-------------|
| `tui:shell` | Hard upstream | `Shell`, `GameRenderer` trait, `PipeMode`, `Events`, `Theme`; 82 tests pass |
| `games:traits` | Hard upstream | `CfrGame`, `Profile`, `GameConfig`, `GameType`; 14 tests pass |
| `play:tui` | Hard upstream | `myosu-play` binary skeleton (Slice 1: binary dispatch) |
| `spectator-protocol` | Spec source | AX-01..05, SP-01..03 from `specsarchive/031626-10-agent-experience.md` and `specsarchive/031626-17-spectator-protocol.md` |
| `chain:runtime` | Soft upstream (Phase 2) | Miner axon HTTP endpoint for lobby + game selection; WebSocket upgrade for spectator |
| `robopoker` | Hard upstream (external) | `Game`, `Recall`, `Action` — absolute path deps; git migration unresolved |

---

## Phase Ordering

```
Phase 1 (Agent Identity — depends on tui:shell):
  Slice 1 → Slice 2 → Slice 3 → Slice 4

Phase 2 (Narration + Pipe Mode — depends on Phase 1):
  Slice 5 → Slice 6 → Slice 7

Phase 3 (Spectator — depends on play:tui binary + Phase 1):
  Slice 8 → Slice 9

Phase 4 (Chain-Connected — depends on chain:runtime):
  Lobby queries miner axon; spectator upgrades to WebSocket
```
