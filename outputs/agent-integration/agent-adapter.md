# Agent Integration — Adapter Document

## Purpose

This document describes the **agent adapter surface** — the interfaces through which programmatic agents (LLMs, bots, scripts) connect to the Myosu game-solving system. It is derived from the reviewed `agent:experience` lane artifacts and documents how the pipe mode, JSON schema, and related interfaces integrate with the broader Myosu architecture.

## Agent Adapter Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         AGENT (LLM / Bot / Script)                      │
│                                                                          │
│   stdin/stdout pipe              JSON via HTTP/WS (future)                │
└──────────────────┬──────────────────────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                     myosu-play (binary)                                  │
│                                                                          │
│  --pipe           --context <path>      --narrate        --spectate      │
└──────────┬────────────────┬─────────────────────┬───────────────────────┘
           │                │                     │
           ▼                ▼                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    PipeMode (crate: myosu-tui)                           │
│                                                                          │
│  pipe.rs: stdin/stdout driver                                            │
│  - output_state()  → GameRenderer::pipe_output() → terse text            │
│  - read_input()    → parse_input() → game action                        │
│                                                                          │
│  agent_context.rs (MISSING — Slice 1)                                    │
│  - load/save persistent identity, memory, journal                         │
│                                                                          │
│  narration.rs (MISSING — Slice 5)                                        │
│  - NarrationEngine for --narrate prose output                            │
│                                                                          │
│  journal.rs (MISSING — Slice 2)                                          │
│  - append-only markdown journal                                           │
└──────────┬──────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                   GameRenderer (trait: object-safe)                        │
│                                                                          │
│  pipe_output() ──► String  (plain text for pipe mode)                    │
│  render_state() ──► ratatui buffer (TUI rendering)                       │
│  parse_input()   ──► Option<String> (action parsing)                    │
│  completions()   ──► Vec<String> (tab completion)                       │
│                                                                          │
│  Implementations:                                                        │
│  - NlheRenderer (MISSING — poker engine lane)                           │
│  - LiarsDiceRenderer (MISSING — multi-game lane)                         │
│  - etc.                                                                  │
└──────────┬──────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      myosu-games (crate: traits)                         │
│                                                                          │
│  games:traits lane (TRUSTED — 14 tests pass)                            │
│  - CfrGame, Profile, GameConfig, GameType                                │
│  - StrategyQuery / StrategyResponse                                      │
│                                                                          │
│  games:poker-engine lane (blocked on games:traits)                      │
│  - NlheSolver, exploitability scoring                                    │
└─────────────────────────────────────────────────────────────────────────┘
```

## Interface Specification

### Interface 1: Pipe Mode (`--pipe`)

**Type**: stdin/stdout text protocol
**Status**: Partial implementation (6 tests pass, trusted)
**File**: `crates/myosu-tui/src/pipe.rs`

Pipe mode provides a plain-text interface for agent piping:

```
agent_a | myosu-play --pipe | agent_b
```

**Output format** (from `GameRenderer::pipe_output()`):
```
STATE hand=47 pot=12 street=flop board=Ts7h2c hero=AcKh to_call=4bb actions=fold,call,raise,shove
```

**Input format**: single keyword or shorthand
```
fold, f    → fold
call, c    → call
raise, r   → raise (with amount via clarify prompt)
check, k   → check (when no bet)
```

**Key behaviors**:
- No ANSI escape codes (verified by `is_plain_text()`)
- Flush stdout after every write
- No cursor manipulation or box-drawing characters
- EOF on stdin terminates the session

**Current contract** (from `pipe.rs:144-170`):
- `PipeMode::output_state()` writes `GameRenderer::pipe_output()` to stdout
- `PipeMode::read_input()` returns `Option<String>` — `None` if stdin closes or line is empty
- `PipeMode::has_ansi_codes()` detects escape sequences to prevent TUI pollution

### Interface 2: JSON Schema (`GameState`)

**Type**: JSON serialization
**Status**: Fully implemented and trusted (16 tests pass)
**File**: `crates/myosu-tui/src/schema.rs`

The `GameState` struct is the canonical machine-readable game state:

```rust
pub struct GameState {
    pub game_type: String,           // e.g., "nlhe_hu", "liars_dice"
    pub hand_number: Option<u32>,
    pub phase: GamePhase,            // Waiting, Action, Betting, Showdown, Complete, Ended
    pub state_wrapper: GameStateWrapper,  // game-specific JSON payload
    pub legal_actions: Vec<LegalAction>,  // EXHAUSTIVE — agent never guesses
    pub meta: Option<MetaInfo>,      // solver_source, exploitability, subnet_id
}
```

**Key design decision**: `legal_actions` is exhaustive. An agent never needs to guess what actions are legal — every valid action is enumerated with its parameters.

**Example NLHE state**:
```json
{
  "game_type": "nlhe_hu",
  "hand_number": 47,
  "phase": "action",
  "state": {
    "board": ["Ts", "7h", "2c"],
    "your_hand": ["As", "Kh"],
    "your_stack": 94,
    "your_position": "BB",
    "opponents": [{"seat": "SB", "stack": 94, "hand": null}],
    "pot": 12,
    "to_act": "you",
    "last_action": {"player": "SB", "action": "raise", "amount": 6},
    "street": "flop"
  },
  "legal_actions": [
    {"action": "fold"},
    {"action": "call", "amount": 6},
    {"action": "raise", "min": 12, "max": 94},
    {"action": "shove", "amount": 94}
  ],
  "meta": {
    "solver_source": "miner-12",
    "solver_exploitability": 13.2,
    "subnet_id": 1
  }
}
```

### Interface 3: Agent Context File (`--context <path>`)

**Type**: JSON file, load/save persistence
**Status**: Missing — Slice 1 of `agent:experience` implementation
**File**: `crates/myosu-tui/src/agent_context.rs` (to be created)

Schema (from `specsarchive/031626-10-agent-experience.md` AC-AX-01):
```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [...] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

**Key behaviors**:
- Load on startup, save on shutdown
- Missing file → create new default identity
- Journal is append-only (never truncates)
- Context file never exposed to opponents (privacy boundary)

### Interface 4: Reflection Channel

**Type**: stdin/stdout text interaction
**Status**: Missing — Slice 4 of `agent:experience` implementation

After each hand completes, pipe mode outputs:
```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```

**Behavior**:
- Empty line → skip reflection (journal entry has no reflection field)
- Non-empty line → append to journal entry

### Interface 5: Narration Mode (`--narrate`)

**Type**: stdout prose rendering
**Status**: Missing — Slice 5 of `agent:experience` implementation
**File**: `crates/myosu-tui/src/narration.rs` (to be created)

When `--narrate` is set, `GameState` is rendered as atmospheric prose:

```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
...
```

**Key behaviors**:
- Board texture analysis ("dry", "wet", "connected")
- Session arc (stack trajectory, opponent history)
- Uses same `GameState` as pipe mode — same underlying game logic

### Interface 6: Lobby + Game Selection

**Type**: stdin/stdout text interaction
**Status**: Missing — Slice 7 of `agent:experience` implementation

When `--pipe` is used without `--subnet`:
```
MYOSU/LOBBY
subnets:
  1 nlhe-hu    12 miners  13.2 mbb/h  ACTIVE
  2 nlhe-6max  18 miners  15.8 mbb/h  ACTIVE
>
```

Commands:
- `info <id>` → detailed subnet info
- `<id>` → select subnet and start game

### Interface 7: Spectator Relay

**Type**: Unix domain socket / WebSocket
**Status**: Missing — Slice 8-9 of `agent:experience` implementation

**Phase 0 (local socket)**:
- Socket path: `~/.myosu/spectate/<session_id>.sock`
- Emits JSON `GameEvent` lines
- Fog-of-war enforced at relay (hole cards hidden during play)

**Phase 1 (future)**: WebSocket upgrade via miner axon

## Integration Points

### With `play:tui`

The `myosu-play` binary (defined in `play:tui` lane) is the vehicle for all `--pipe`, `--context`, `--narrate`, and `--spectate` flags. The binary skeleton is Slice 1 of `play:tui` and is a prerequisite for `agent:experience` Slice 3+.

Current status: **binary skeleton missing** — HIGH blocker.

### With `games:traits`

The `games:traits` lane (14 tests, trusted) provides:
- `CfrGame` — game-agnostic MCCFR interface
- `Profile` — strategy profile with `exploitability()` method
- `GameConfig` — game configuration
- `GameType` — game type enumeration
- `StrategyQuery/Response` — miner query interface

The `GameRenderer` trait uses types from `games:traits` for game logic.

### With `chain:runtime`

The lobby (Slice 7) queries the chain for active subnet information:
- Miner count per subnet
- Average exploitability
- Subnet status (ACTIVE, etc.)

Current status: **stubbed for Phase 0** — real chain integration is Phase 4.

## Adapter Surface Summary

| Interface | Type | Status | File |
|----------|------|--------|------|
| Pipe mode | stdin/stdout text | Trusted (6 tests) | `pipe.rs` |
| JSON schema | JSON | Trusted (16 tests) | `schema.rs` |
| Agent context | JSON file | Missing | `agent_context.rs` (Slice 1) |
| Reflection | stdin/stdout | Missing | `pipe.rs` extension (Slice 4) |
| Narration | stdout prose | Missing | `narration.rs` (Slice 5) |
| Lobby | stdin/stdout | Missing | `pipe.rs` extension (Slice 7) |
| Spectator relay | Unix socket | Missing | `spectate.rs` (Slice 8) |

## Key Design Decisions

1. **Object-safe GameRenderer** — `Box<dyn GameRenderer>` allows shell composition without modifying core logic. Games implement the trait; no changes to shell/event loop/input handling.

2. **Exhaustive legal_actions** — Agent never guesses. Every valid action is enumerated with parameters. Reduces agent error rate on action validity.

3. **Fog-of-war at relay, not renderer** — Hole cards are suppressed at the `SpectatorRelay` level before emission, not in the game renderer. Simpler renderer implementation; relay is the enforcement boundary.

4. **Append-only journal** — Journal never truncates. Each hand produces a markdown entry. Reflection is optional (empty line skips). Journal is the agent's autobiography, not a log.

5. **Narration shares GameState with pipe mode** — Same game logic, two renderers. `--narrate` is purely a display choice; game state is identical.

6. **Stubbed chain for Phase 0** — Lobby displays hardcoded data initially. Real chain integration (Phase 4) depends on `chain:runtime` lane completing first.
