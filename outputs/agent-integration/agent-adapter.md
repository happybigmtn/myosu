# `agent:experience` Integration Adapter

## Purpose

This document describes how the surfaces delivered by `agent:experience` are wired into the Myosu product. It is the **integration contract** between the agent-facing surfaces and the consuming binary (`myosu-play`). It is a direct product of executing the `agent:experience` lane through the bootstrap spec+review workflow.

This document does not repeat the lane specification. It assumes the reader has `outputs/agent/experience/spec.md` and is focused on understanding how the surfaces compose into a running product.

---

## Integration Architecture

```
agent:experience surfaces (outputs/agent/experience/spec.md)
│
├── JSON schema (docs/api/game-state.json + schema.rs)
│   └── Consumed by: PipeMode driver, SpectatorRelay event encoding
│
├── pipe mode (GameRenderer::pipe_output() via PipeMode)
│   └── Driven by: myosu-play --pipe CLI flag
│       │
│       ├── --context <path>  →  AgentContext::load() on startup
│       │                          AgentContext::save() on shutdown
│       │
│       ├── --narrate          →  NarrationEngine (replaces pipe_output prose)
│       │
│       └── reflect> prompt    →  stdin blocked after HAND COMPLETE
│                                   → Journal::append_reflection()
│
├── AgentContext (agent_context.rs)
│   └── JSON file at path given by --context
│       Fields: identity, memory, journal
│       Lifecycle: load on PipeMode new(), save() on drop
│
├── Journal (journal.rs)
│   └── Append-only markdown at {context-dir}/journal.md
│       Entries: board, held cards, result, optional reflection
│       Never truncates; each hand produces one entry
│
├── Lobby (pipe mode, no --subnet flag)
│   └── myosu-play --pipe (no subnet)
│       → lobby output → info <id> → subnet detail → selection starts game
│       Stubbed chain query for Phase 0 (hardcoded subnet data)
│
└── SpectatorRelay (spectate.rs, AC-SP-01)
    └── Unix domain socket at ~/.myosu/spectate/<session_id>.sock
        Phase 0: local relay with fog-of-war
        Phase 1: miner axon WebSocket upgrade
```

**Data flow summary**: `robopoker Game` → `schema.rs GameState` → `GameRenderer::pipe_output()` / `NarrationEngine` / `SpectatorRelay` → agent (stdout / socket) / journal (file)

---

## Surface Integration Points

### 1. `schema.rs` — Shared Event and State Format

`crates/myosu-tui/src/schema.rs` (939 lines, 16 tests, **TRUSTED**) is the central data type for all agent-facing surfaces:

- **Pipe mode**: `PipeMode::render_state()` calls the `GameRenderer` which uses `schema.rs` types internally; `pipe_output()` emits the canonical text encoding of a `GameState`
- **Spectator relay**: `SpectatorRelay::emit()` serializes `GameEvent` (derived from `GameState`) as JSON lines to the Unix socket
- **Narration engine**: `NarrationEngine::narrate()` takes a `GameState` and produces atmospheric prose; both `--narrate` and non-narrate modes produce identical game outcomes

**Invariant**: The same `GameState` instance must produce identical outcomes in pipe, narration, and spectator modes. The narration engine is a rendering transform, not a game logic transform.

### 2. `AgentContext` — Persistent Identity Across Sessions

The agent context file is loaded at `PipeMode::new()` when `--context <path>` is provided and saved at `PipeMode::drop()`. The file format is JSON with three top-level fields:

- `identity`: name, created timestamp, games_played counter, preferred_game
- `memory`: session_count, lifetime_result (in big blinds), observations array
- `journal`: array of {session, hand, reflection} entries

**Load behavior**: If the file does not exist, `AgentContext::default()` creates a new identity with a generated name and zero counters. If the file exists but is malformed, `AgentContext::load()` returns an error — the binary must decide whether to fatal or create new.

**Save behavior**: `AgentContext::save()` writes the full context to the path atomically (rename over old file). The journal array is append-only; existing entries are never modified.

**Critical path**: If `--context` is not provided, `PipeMode` operates without persistent identity. The reflection prompt still appears, but `AgentContext` is not loaded or saved.

### 3. `Journal` — Append-Only Memory Artifact

The journal is a markdown file written by `Journal::append_hand_entry()` and `Journal::append_session_summary()`. It lives at `{context-dir}/journal.md` when `--context` is provided.

**Format per hand**:
```markdown
## Hand {N} — {game_type} — {timestamp}

board: T♠ 7♥ 2♣
held: A♠ K♥
result: +14bb (showdown)
session: +28bb over 47 hands

### Reflection
{agent-provided text, or "(skipped)"}
---
```

**Never truncates invariant**: The file is opened in append mode. `Journal::append_hand_entry()` and `Journal::append_session_summary()` write and fsync; they never open the file in write/truncate mode. The only way to reduce file size is manual editing.

### 4. `reflect>` Prompt — In-Hand Introspection Channel

After `HAND COMPLETE` output is written to stdout, `PipeMode` blocks on stdin for the reflection prompt. This is the **only stdin read that happens after game output** — all other stdin reads happen before action decisions.

```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```

**Interaction with journal**: If the agent provides non-empty input, the reflection is appended to the current hand's journal entry. Empty input (blank line or just whitespace) is stored as `"(skipped)"`.

**Interaction with context**: The reflection text is also appended to `AgentContext::journal` array for the current session. On save, the full journal (including new reflection) is written.

### 5. `NarrationEngine` — Atmospheric Prose Transform

`NarrationEngine::narrate(&GameState) -> String` produces prose when `--narrate` is set. It is a pure rendering transform — it receives the same `GameState` as `pipe_output()` and produces different text output.

**Board texture analysis** (from AC-AX-03):
- "dry" — low connectivity, few draws (e.g., rainbow board, broadway gaps)
- "wet" — many draws possible (flush draws, straight draws, paired board)
- "connected" — sequential ranks present (4-5-6, 8-9-T)

**Session arc**: `NarrationEngine` reads from `AgentContext::memory` to produce output that references the session trajectory: stack depth history, opponent tendencies observed, cumulative result.

**Key integration constraint**: `NarrationEngine::narrate()` must not alter game logic or state. The same `GameState` fed to narration and to pipe mode must produce identical game outcomes when acted upon.

### 6. Lobby — Game Selection Without Chain

When `myosu-play --pipe` is invoked without `--subnet`, `PipeMode` enters lobby mode. The lobby is a text listing of available subnets with their status.

**Phase 0 behavior**: Lobby data is hardcoded in `PipeMode::new()` (stubbed). The `info <id>` command shows hardcoded detail (miner count, exploitability, game status).

**Phase 1 behavior**: Lobby queries `chain:runtime` or a miner axon for live subnet data. This is blocked on `chain:runtime` being available.

**Key integration point**: Lobby rendering uses the same `pipe_output()` mechanism as game play — it is text to stdout, no TUI required.

### 7. `SpectatorRelay` — Fog-of-War Event Stream

The relay is a Unix domain socket at `~/.myosu/spectate/<session_id>.sock`. It emits JSON-encoded `GameEvent` lines for each game event (hand start, action, street change, showdown, hand end).

**Fog-of-war contract**: Hole cards are NEVER sent to the relay during active play. Only after the `showdown` event does the relay emit hole card data.

**Socket lifecycle**: The relay creates the socket file before the session starts. It cleans up on session end. If a listener disconnects, the relay continues operating (no blocking on write).

**Phase 1 upgrade**: The relay will accept WebSocket connections via miner axon. The event format (JSON GameEvent) remains identical. The fog-of-war contract remains identical.

---

## CLI Flag Integration

All agent-facing flags are exposed via `myosu-play`'s CLI dispatch. The integration point is in `crates/myosu-play/src/main.rs`.

| Flag | Consumer | Integration |
|------|----------|-------------|
| `--pipe` | `PipeMode` | Creates `PipeMode` instead of `Shell`; enables stdin/stdout protocol |
| `--context <path>` | `AgentContext` | Passed to `PipeMode::new()`; triggers load; saved on drop |
| `--narrate` | `NarrationEngine` | Passed to `PipeMode::new()`; switches pipe_output to narration |
| `--spectate <session_id>` | `SpectatorRelay` | Creates relay and connects to active session's socket |
| `--subnet <id>` | Lobby | Skips lobby; directly joins subnet <id> |

**Flag combinations**:
- `myosu-play --pipe` → lobby mode (no context, no narration)
- `myosu-play --pipe --context ./koan.json` → persistent agent session
- `myosu-play --pipe --context ./koan.json --narrate` → narrated persistent session
- `myosu-play --pipe --subnet 1` → direct subnet join, no lobby

---

## Integration Blockers

### HIGH — `myosu-play` Binary Absent

The `myosu-play` binary does not exist yet. All agent-facing surfaces require the binary to be scaffolded first (Slice 1 of `play:tui`). The `--pipe`, `--context`, `--narrate`, and `--spectate` flags cannot be wired until `main.rs` exists.

**Owner**: `play:tui` lane, Slice 1

### HIGH — `robopoker` Git Dependency Missing

All agent-facing surfaces ultimately call into `games:traits` or `tui:shell`, which call into `robopoker` via **absolute filesystem paths**. Until `robopoker` is a proper `git = "https://..."` dependency in `Cargo.toml`, `cargo build` fails on any clean checkout.

**Owner**: `games:traits` lane

### MEDIUM — Lobby Chain Query Stubbed for Phase 0

The lobby currently shows hardcoded subnet data. Real chain-connected lobby is Phase 4 (blocked on `chain:runtime`).

**Owner**: `agent:experience` lane, Slice 7

### LOW — Spectator Socket Path Not Aligned with `play:tui` Data Directory

Spectator relay uses `~/.myosu/spectate/<session_id>.sock`. `play:tui` uses `{data-dir}/hands/hand_{N}.json` for hand history. These are different base paths — this is acceptable but must be confirmed when `play:tui` Slice 1 lands.

**Owner**: `agent:experience` lane, Slice 8 (confirm before implementation)

---

## What This Document Does Not Cover

- The `spectate.rs` implementation (belongs to `agent:experience` Slice 8)
- The `SpectateScreen` TUI rendering (belongs to `agent:experience` Slice 9)
- The `NlheRenderer` concrete implementation (belongs to `play:tui` lane)
- The `chain:runtime` integration for live lobby data (Phase 4, blocked on `chain:runtime`)
- The miner axon WebSocket upgrade for spectator relay (Phase 1, blocked on miner serving)
