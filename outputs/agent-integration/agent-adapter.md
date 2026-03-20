# `agent:experience` Integration Adapter

## Purpose

This document describes how the `agent:experience` lane fits into the product
surface — what it consumes from upstream lanes, what it provides to agents and
other consumers, and the critical path items that determine when its
implementation can proceed.

---

## Lane Dependency Graph

```
                           upstream lanes
                           ─────────────
  tui:shell ──────────────────────────────►
  (82 tests, trusted)                      │
  GameRenderer trait, PipeMode, Shell,     │
  Events, Theme                           │
                                           ▼
                              ┌────────────────────────┐
                              │  agent:experience       │
                              │  (THIS LANE)            │
                              │                         │
  games:traits ──────────────►│  pipe_output() contract │
  (14 tests, trusted)         │  --context file format  │
  CfrGame, Profile,           │  --narrate prose engine │
  GameConfig, GameType        │  reflect> prompt        │
                              │  journal.md append      │
                              │  SpectatorRelay socket  │
                              └──────────┬───────────────┘
                                         │
                           downstream    │
                           consumers     ▼
                           ─────────────────────────────
                           agents (LLM/bot/script)
                           myosu-play --pipe session
                           spectator --spectate client
```

---

## Upstream Contract

### From `tui:shell` (trusted — 82 tests pass)

| Surface | Path | What agent:experience uses |
|---------|------|---------------------------|
| `GameRenderer` trait | `crates/myosu-tui/src/renderer.rs` | `pipe_output()` renders game state as terse text; `agent:experience` wraps this with narration when `--narrate` is set |
| `PipeMode` driver | `crates/myosu-tui/src/pipe.rs` | Base `--pipe` stdin/stdout loop; `agent:experience` extends with `--context`, `--narrate`, `reflect>`, lobby |
| `Shell` | `crates/myosu-tui/src/shell.rs` | Not directly used by pipe mode; reference for screen-state concepts |
| `Events` | `crates/myosu-tui/src/events.rs` | Not used in pipe mode (no TTY) |
| `Theme` | `crates/myosu-tui/src/theme.rs` | Not used in pipe mode (no ratatui rendering) |
| `schema.rs` | `crates/myosu-tui/src/schema.rs` | `GameState`, `LegalAction`, `GamePhase` — trusted, fully tested (16 tests) |

**Critical**: `PipeMode` already has `--pipe` flag with ANSI detection and state
parsing. The agent:experience lane extends it, not replaces it.

### From `games:traits` (trusted — 14 tests pass)

| Surface | Path | What agent:experience uses |
|---------|------|---------------------------|
| `CfrGame` | `crates/myosu-games/src/traits.rs` | Game state abstraction — used by `GameRenderer` to traverse game tree |
| `Profile` | `crates/myosu-games/src/traits.rs` | Strategy profile — queried for action distributions in solver advisor |
| `GameConfig` | `crates/myosu-games/src/traits.rs` | Game parameters (N, stack, blinds) — serialized into `--context` JSON |
| `GameType` | `crates/myosu-games/src/traits.rs` | Game variant identifier (nlhe-hu, nlhe-6max) — used in lobby and journal |
| `StrategyQuery/Response` | `crates/myosu-games/src/traits.rs` | Miner-facing query protocol; not used by pipe mode directly |

**Critical**: `games:traits` has no `robopoker` absolute-path dependency after
Slice 1 (git rev pin at `04716310143094ab41ec7172e6cea5a2a66744ef`).

### From `play:tui` (binary — blocked on upstream)

| Surface | Path | What agent:experience needs |
|---------|------|---------------------------|
| `myosu-play` binary | `crates/myosu-play/src/main.rs` | CLI entry point that dispatches `--pipe`, `--context`, `--narrate`, `--spectate` flags |

**Blocker**: `play:tui` Slice 1 does not exist. The `myosu-play` binary is
completely absent. This blocks `agent:experience` Slices 3, 6, 7, 8, 9 from
proceeding.

---

## Downstream Contract (What agents consume)

### Primary: Pipe Mode Protocol

```
agent ──stdin──► myosu-play --pipe --context ./koan.json --narrate
                  │
                  ├──► GameRenderer::pipe_output() ──► terse key-value text
                  │         OR
                  └──► NarrationEngine::narrate() ──► atmospheric prose
                  │
                  ├──► HAND COMPLETE + reflect> prompt ──► stdin
                  └──► Journal append (never truncated)
```

**Terse output format** (when `--narrate` is absent):
```
street: flop
board: Ts 7h 2c
you: As Ks (hero)
solver: Kh Qh
pot: 12.5
stack: 94.0
legal_actions: fold call raise
>
```

**Narrated output format** (when `--narrate` is present):
```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♠ in the big blind. 94bb behind.
...
```

### Secondary: JSON Schema

`docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs` provide the
machine-readable game state consumed by structured agents. This is already
trusted (16 tests pass). The same `GameState` schema powers:
- `pipe_output()` rendering
- `narration.rs` analysis
- Spectator relay JSON events

### Tertiary: Agent Context File (`--context`)

JSON file providing persistent identity, memory, and journal across sessions:

```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [...] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

The context file is loaded at startup and saved at shutdown. It is never
transmitted to opponents.

### Quaternary: Journal (append-only markdown)

`{context-dir}/journal.md` — append-only markdown artifact recording every hand
and the agent's reflections. Never truncated. Each entry contains board, held
cards, result, and optional reflection.

### Quinary: Spectator Relay (Unix socket)

Phase 0: `~/.myosu/spectate/<session_id>.sock` — Unix domain socket emitting
JSON `GameEvent` lines. Fog-of-war enforced at relay (hole cards never emitted
during active play; only after `showdown`).

Phase 1 (future): WebSocket upgrade via miner axon.

---

## Interface Summary

| Interface | Direction | Protocol | Blocking? |
|-----------|-----------|----------|-----------|
| `--pipe` stdin/stdout | bidirectional | plain text | No — works offline |
| `--context` JSON file | file read/write | serde JSON | No — works offline |
| `--narrate` output | stdout | plain text prose | No — works offline |
| `reflect>` prompt | stdin | single-line text | No — works offline |
| Journal (`journal.md`) | file append | markdown | No — works offline |
| Spectator socket | Unix socket | JSON lines | No — local only |
| Miner `/strategy` | HTTP | JSON | **Yes — blocked on chain:runtime** |
| Lobby (subnet list) | chain query | JSON | **Yes — blocked on chain:runtime** |
| WS spectator upgrade | WebSocket | JSON | **Yes — blocked on chain:runtime** |

---

## Phase 0 vs Phase 1 Capabilities

**Phase 0** (achievable now, after `play:tui` Slice 1):
- `--pipe` with terse output
- `--context` with identity/memory/journal
- `--narrate` with board texture + session arc prose
- `reflect>` prompt after each hand
- Lobby with hardcoded stub data
- Spectator relay on Unix socket (local only)

**Phase 1** (requires `chain:runtime`):
- Lobby with live chain data (active miners, exploitability scores)
- Miner-connected play (`myosu-play --chain ws://...`)
- WebSocket spectator via miner axon

**Phase 2** (future, not specced):
- HTTP/WS API server for structured agents
- Remote agent connections over network

---

## Critical Path to Agent:experience Slice 1

Slices 1–2 of `agent:experience` (agent_context.rs + journal.rs) depend only
on `tui:shell` and `games:traits` — both already trusted. These slices can
begin immediately without waiting for `play:tui` binary or `chain:runtime`.

Slices 3–9 require the `myosu-play` binary from `play:tui` Slice 1.

---

## Key Files

| File | Role |
|------|------|
| `crates/myosu-tui/src/schema.rs` | Trusted `GameState` JSON schema (939 lines, 16 tests) |
| `crates/myosu-tui/src/pipe.rs` | `PipeMode` driver — extended by agent:experience |
| `crates/myosu-tui/src/agent_context.rs` | MISSING — Slice 1 of agent:experience |
| `crates/myosu-tui/src/journal.rs` | MISSING — Slice 2 of agent:experience |
| `crates/myosu-tui/src/narration.rs` | MISSING — Slice 5 of agent:experience |
| `crates/myosu-play/src/main.rs` | MISSING — `play:tui` Slice 1; required for agent:experience Slices 3+ |
| `crates/myosu-play/src/spectate.rs` | MISSING — `play:tui` Slice 8; Phase 0 spectator relay |
| `docs/api/game-state.json` | Trusted JSON schema (10 game types) |
