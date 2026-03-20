# `agent:experience` Integration — Agent Adapter Surface

## Purpose

This document maps every agent-facing surface that `agent:experience` owns — what each surface does, what inputs it consumes, what outputs it produces, and which upstream lane provides the underlying capability. It is the integration contract between the `agent:experience` lane and the rest of the product system.

---

## Agent-Facing Surfaces

### 1. `--pipe` Mode (stdin/stdout text protocol)

**Owning module**: `crates/myosu-tui/src/pipe.rs` (`PipeMode`)
**Upstream**: `tui:shell` — `GameRenderer::pipe_output()` trait method
**Upstream status**: TRUSTED (82 tests pass)

`--pipe` is the primary agent interface. Agents invoke `myosu-play --pipe` and communicate via plain-text stdin/stdout. The protocol is synchronous: agent sends an action, receives game state, sends next action.

**Extensions owned by `agent:experience`**:
- `--context <path>` — load/save agent persistent context (Slice 3)
- `--narrate` — switch from terse key-value to atmospheric prose (Slice 6)
- `reflect>` prompt — appears after `HAND COMPLETE`; agent responds or skips (Slice 4)
- Lobby when no `--subnet` — game selection before entering play (Slice 7)

### 2. JSON Schema (`docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs`)

**Owning module**: `crates/myosu-tui/src/schema.rs` (939 lines, 16 tests)
**Upstream**: `games:traits` — `GameState`, `LegalAction`, `GamePhase` types
**Upstream status**: TRUSTED (schema is fully implemented and stable)

The JSON schema is the machine-readable contract. Structured agents parse `GameState` JSON rather than relying on text output. The same schema powers both the `--pipe` mode and the spectator relay event format.

**Schema coverage**: 10 game types implemented (NLHE HU, NLHE 6max, PLO, Liar's Dice, etc.)
**Evolution path**: Schema is the foundation for Phase 2 HTTP/WS APIs — same types, different transport.

### 3. Agent Context File (`--context <path>`)

**Owning module**: `crates/myosu-tui/src/agent_context.rs` (MISSING — Slice 1)
**Upstream**: `tui:shell` — `PipeMode::new()` accepts arbitrary state
**Upstream status**: TRUSTED (pipe driver exists, context not yet wired)

JSON file with schema:
```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

Loaded at startup, saved on shutdown. Never exposed to opponents. Validated via serde before use.

### 4. Reflection Channel (`reflect>` prompt)

**Owning module**: `crates/myosu-tui/src/pipe.rs` (within `PipeMode`)
**Upstream**: `agent_context.rs` (journal append) + `tui:shell` (hand completion detection)
**Upstream status**: JOURNAL MISSING, `tui:shell` TRUSTED

After each hand completes, pipe mode outputs:
```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```

Empty line skips. Non-empty response is appended to the journal entry for that hand.

### 5. Agent Journal (`outputs/agent/journal.md`)

**Owning module**: `crates/myosu-tui/src/journal.rs` (MISSING — Slice 2)
**Upstream**: `agent_context.rs` (writes to journal array) + filesystem
**Upstream status**: Neither implemented yet

Append-only markdown artifact. Each hand produces a markdown entry:
```markdown
## Hand 47 — Session 23

board: T♠ 7♥ 2♣
held: A♠ K♥
result: +14bb (showdown vs Q♣ J♣)
session: +28bb over 47 hands

reflection: I overfolded the turn. His line was
consistent with a float. Note to re-examine
pot-odds on future rivers.
```

Never truncated. Grows monotonically.

### 6. Narration Engine (`--narrate` flag)

**Owning module**: `crates/myosu-tui/src/narration.rs` (MISSING — Slice 5)
**Upstream**: `games:traits` — `GameState` data model
**Upstream status**: TRUSTED at data model level; `narration.rs` itself is missing

Renders `GameState` as atmospheric prose instead of terse key-value. Example:
```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
```

Board texture analysis: "dry" / "wet" / "connected". Session arc woven from context file.

### 7. Spectator Relay (Phase 0 — Unix domain socket)

**Owning module**: `crates/myosu-play/src/spectate.rs` (MISSING — Slice 8)
**Upstream**: `schema.rs` (trusted GameEvent JSON format)
**Upstream status**: SCHEMA TRUSTED; relay implementation missing

Unix domain socket at `~/.myosu/spectate/<session_id>.sock`. Emits JSON event lines. Fog-of-war enforced at relay (hole cards never sent during play; shown after `showdown` event).

### 8. Spectate TUI Screen

**Owning module**: `crates/myosu-tui/src/screens/spectate.rs` (MISSING — Slice 9)
**Upstream**: `spectate.rs` (relay) + `tui:shell` (ScreenManager)
**Upstream status**: Both MISSING

`Screen::Spectate` variant in the TUI. Connects to relay socket, renders events with fog-of-war. Keybindings: `n` (next session), `r` (reveal hole cards after showdown), `q` (quit to lobby).

---

## Surface Dependency Map

```
agent (LLM/bot/script)
  │
  ├─► myosu-play --pipe
  │     │
  │     ├─► PipeMode (pipe.rs) — owned
  │     │     ├─► GameRenderer::pipe_output() — upstream: tui:shell (TRUSTED)
  │     │     ├─► AgentContext (agent_context.rs) — owned, MISSING
  │     │     ├─► Journal (journal.rs) — owned, MISSING
  │     │     ├─► NarrationEngine (narration.rs) — owned, MISSING
  │     │     └─► reflect> prompt — owned, MISSING
  │     │
  │     └─► myosu-play binary — upstream: play:tui (MISSING)
  │           └─► crates/myosu-play/src/spectate.rs — owned, MISSING
  │                 └─► schema.rs (GameEvent JSON) — upstream: tui:shell (TRUSTED)
  │
  └─► GameState JSON schema
        └─► docs/api/game-state.json + schema.rs — TRUSTED
              └─► games:traits (GameState, GameType) — upstream: games:traits (TRUSTED)
```

---

## Cross-Lane Dependency Table

| Lane | Type | Surfaces Used | Status |
|------|------|--------------|--------|
| `tui:shell` | Hard upstream | `Shell`, `GameRenderer`, `PipeMode`, `Events`, `Theme` | TRUSTED (82 tests pass) |
| `games:traits` | Hard upstream | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery` | TRUSTED (14 tests pass) |
| `play:tui` | Hard upstream (binary) | `myosu-play` binary dispatch for `--pipe`, `--spectate` | MISSING — Slice 1 of play:tui |
| `chain:runtime` | Soft upstream (Phase 2) | Miner axon for lobby queries; WebSocket upgrade for spectator | NOT STARTED |
| `robopoker` | Hard upstream (external) | All game computation — absolute path deps, git migration UNRESOLVED | BLOCKER |

---

## Lane Boundary Summary

`agent:experience` is a **terminal lane** — it has no trusted downstream outputs. It consumes trusted upstream surfaces from `tui:shell` and `games:traits`, and extends them with agent-facing affordances:

1. Persistence: `AgentContext` + `Journal` for cross-session memory
2. Richness: `NarrationEngine` for prose rendering
3. Protocol extensions: `reflect>` channel, lobby, game selection
4. Observation: `SpectatorRelay` + `SpectateScreen` for watching play

The lane does **not** own: chain connectivity, miner/validator coordination, or game logic. Those belong to their respective lanes.
