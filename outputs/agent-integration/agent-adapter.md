# Agent Integration Adapter

## Purpose

This document describes the integration contract between `agent:experience` and the rest of the Myosu system. It is the **authoritative adapter surface** — the set of integration points, wire contracts, and dependency constraints that any implementation of the agent-facing surfaces must satisfy.

This adapter is not the lane spec. The lane spec lives at `outputs/agent/experience/spec.md` and defines what the agent:experience lane does. This adapter describes how it connects to the surrounding system.

---

## Integration Topology

```
agent:experience integration surface
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  upstream: tui:shell (82 tests, trusted)
  ┌───────────────────────────────────────────────────────┐
  │  GameRenderer::pipe_output() ← pipe mode text output  │
  │  PipeMode driver ← stdin/stdout                       │
  │  Events, Theme ← terminal interface                   │
  └───────────────────────────────────────────────────────┘
                          │
  upstream: games:traits (14 tests, trusted)
  ┌───────────────────────────────────────────────────────┐
  │  CfrGame, Profile, GameConfig, GameType               │
  │  StrategyQuery / StrategyResponse                     │
  └───────────────────────────────────────────────────────┘
                          │
  ┌───────────────────────────────────────────────────────┐
  │  schema.rs + game-state.json (TRUSTED)               │
  │  GameStateBuilder, LegalAction, GamePhase            │
  └───────────────────────────────────────────────────────┘
                          │
  ┌───────────────────────────────────────────────────────┐
  │  NEW (9 slices):                                     │
  │  agent_context.rs  — AgentContext load/save/journal  │
  │  narration.rs      — NarrationEngine prose output   │
  │  journal.rs        — append-only markdown journal   │
  │  pipe.rs extensions — --context, --narrate, reflect> │
  │  spectate.rs       — SpectatorRelay Unix socket     │
  │  screens/spectate.rs — SpectateScreen fog-of-war   │
  └───────────────────────────────────────────────────────┘
                          │
  downstream: miner axon (Phase 2 — future)
  ┌───────────────────────────────────────────────────────┐
  │  Lobby queries miner HTTP endpoint (stubbed Phase 0)  │
  │  Spectator WS upgrade via miner axon (Phase 1)       │
  └───────────────────────────────────────────────────────┘
```

---

## Wire Contracts

### 1. Pipe Mode Protocol (`--pipe`)

**Transport**: stdin/stdout binary pipe. Agent drives `myosu-play --pipe [flags]`.

**Flags**:
- `--context <path>` — path to agent context JSON file; created with default identity if missing
- `--narrate` — use `NarrationEngine` prose instead of `pipe_output()` terse text
- `--subnet <id>` — directly enter subnet `<id>` (bypasses lobby); omit to show lobby first

**Session lifecycle**:
```
# Agent starts session
myosu-play --pipe --context ./koan.json --narrate

# Game state output (narrated or terse)
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
legal_actions: [fold, call, raise]
> call

# Per hand:
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
# Agent types reflection or empty line to skip

# Lobby (when --subnet omitted):
MYOSU/LOBBY
subnets:
  1 nlhe-hu    12 miners  13.2 mbb/h  ACTIVE
  2 nlhe-6max  18 miners  15.8 mbb/h  ACTIVE
>
info 1
  subnet: 1  game: nlhe-hu  miners: 12  avg_score: 13.2 mbb/h
> play 1
# Game begins
```

**Contract**: All output is UTF-8 text. All input is UTF-8 text terminated by newline. No binary protocol. Agent never needs to parse structured data from pipe mode unless using `--json` (future).

---

### 2. JSON Schema Surface (`GameState`)

**Schema**: `docs/api/game-state.json` (trusted) + `crates/myosu-tui/src/schema.rs` (trusted, 16 tests).

**Purpose**: Machine-readable game state for structured agents. Covers 10 game types with exhaustive `legal_actions`.

**Key types** (from `schema.rs`):
```rust
pub struct GameState {
    pub game_type: GameType,
    pub hand_number: u32,
    pub phase: GamePhase,
    pub state: GameSpecificState,
    pub legal_actions: Vec<LegalAction>,
    pub meta: MetaInfo,
}

pub enum GamePhase { Waiting, Action, Betting, Showdown, Complete, Ended }

pub enum LegalAction { Fold, Call, Raise { min: u32, max: u32 }, ... }
```

**Integration point**: `schema.rs` is consumed by both pipe mode (for structured output in future `--json` mode) and by the spectator relay (for event emission). It is the canonical game state representation.

---

### 3. Agent Context File (`--context <path>`)

**Format**: JSON. Serde-serialized `AgentContext`.

```json
{
  "identity": {
    "name": "koan",
    "created": "2026-03-20T10:00:00Z",
    "games_played": 1847,
    "preferred_game": "nlhe-hu"
  },
  "memory": {
    "session_count": 23,
    "lifetime_result": "+342bb",
    "observations": []
  },
  "journal": [
    {
      "session": 23,
      "hand": 47,
      "reflection": "I folded too tight on the river..."
    }
  ]
}
```

**Lifecycle**:
- Loaded on startup via `AgentContext::load(path)`
- Saved on shutdown via `AgentContext::save()`
- Missing file → creates new default identity
- Journal is append-only; never truncated

**Security contract**: Context file is never exposed to opponents. `identity` and `memory` fields are local-only.

---

### 4. Reflection Channel

**Trigger**: After each `HAND COMPLETE` block in pipe mode.

**Agent contract**:
- Output ends with `reflect>` prompt (blocks on stdin)
- Empty line → skip reflection, continue
- Non-empty line → append to `journal[]` entry for this hand

**Format**:
```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect> I overplayed top pair on a wet board.
```

---

### 5. Narration Engine (`--narrate`)

**Trigger**: `--narrate` flag on `myosu-play --pipe`.

**Engine**: `NarrationEngine::narrate(&GameState) -> String`

**Prose characteristics**:
- Board texture: "dry", "wet", "connected" based on suit/count analysis
- Stack trajectory: session arc from context file
- Opponent tendency: inferred from past hands in journal
- Atmospheric, not terse — designed for experienced readers

**Same-game-state guarantee**: `narrate(state)` and `pipe_output(state)` describe the identical game state. Only the rendering differs.

---

### 6. Spectator Relay (Phase 0)

**Socket**: Unix domain socket at `~/.myosu/spectate/<session_id>.sock`

**Protocol**: JSON lines (`\n`-delimited `GameEvent` objects)

**Fog-of-war contract**: Hole cards are **never** emitted during active play. They appear only after `showdown` event.

**Event types**:
```json
{"type": "hand_start", "hand": 47, "players": [...], "timestamp": "..."}
{"type": "action", "player": "hero", "action": "call", "amount": 200}
{"type": "showdown", "board": [...], "hole_cards": {"hero": "A♠ K♥", "villain": "Q♣ J♣"}, "result": "win"}
{"type": "hand_end", "result": "+14bb", "session": "+28bb"}
```

**Listener contract**: Relay handles 0..N disconnected listeners gracefully. Events are not buffered for late joiners.

---

## Dependency Constraints

| Dependency | Type | Integration Contract | Blocker If Missing |
|------------|------|---------------------|-------------------|
| `tui:shell` (82 tests) | Hard upstream | `GameRenderer`, `PipeMode`, `Events`, `Theme` | Nothing compiles |
| `games:traits` (14 tests) | Hard upstream | `CfrGame`, `Profile`, `GameType` | Nothing compiles |
| `play:tui` binary | Hard upstream | `myosu-play` main.rs CLI dispatch | `--pipe` flags have no home |
| `docs/api/game-state.json` | Trusted input | Schema contract | JSON mode unusable |
| `robopoker` (git dep) | Hard upstream | `Game`, `Recall`, `Action` | `games:traits` can't build |
| `chain:runtime` | Soft (Phase 2) | Miner axon HTTP for lobby | Lobby stubs with hardcoded data |

---

## Phase Ordering and Slice Contract

Slices 1–4 are Phase 1 (Agent Identity — only needs `tui:shell`):
1. `agent_context.rs` — `AgentContext` load/save/journal append
2. `journal.rs` — append-only markdown writer
3. `--context` wiring — `PipeMode` loads context on init, saves on drop
4. `reflect>` prompt — stdin read after `HAND COMPLETE`, append if non-empty

Slices 5–7 are Phase 2 (Narration + Pipe Mode — needs Phase 1):
5. `narration.rs` — `NarrationEngine` with board texture analysis
6. `--narrate` wiring — `PipeMode` uses narration engine when flag set
7. Lobby — pipe mode shows lobby when no `--subnet`; stub chain queries for Phase 0

Slices 8–9 are Phase 3 (Spectator — needs `play:tui` binary):
8. `SpectatorRelay` — Unix socket event emitter with fog-of-war
9. `SpectateScreen` — TUI screen with fog-of-war rendering

---

## Adapter Constraints

**Must preserve**:
- `GameRenderer::pipe_output()` trait contract — used by pipe mode
- `schema.rs` `GameState` structure — consumed by spectator relay
- Journal append-only invariant — never truncate, never rewrite
- Fog-of-war at relay — hole cards never emitted during active play
- `AgentContext` serde shape — agents depend on this file format

**Must not break**:
- `tui:shell` tests (82 tests, trusted upstream)
- `games:traits` tests (14 tests, trusted upstream)
- `schema.rs` tests (16 tests, trusted)

**Must never add**:
- Absolute path dependencies in `myosu-play/Cargo.toml` (would break CI)
- Blocking sleep in async event loop (would freeze TUI)
- Trait changes to `GameRenderer` without coordinated `tui:shell` migration
