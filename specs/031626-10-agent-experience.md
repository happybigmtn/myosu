# Specification: Agent Integration — First-Class Programmatic Players

Source: Design review — agents from Claude Code, custom bots, programmatic strategies
Status: Draft
Date: 2026-03-16
Depends-on: TU-06 (pipe mode), GT-01..05 (game traits), GS-10 (runtime API)

## Purpose

Make myosu games trivially playable by programmatic agents — whether an LLM
operating through Claude Code, a custom Python bot, a Rust strategy library,
or any process that can make HTTP calls.

The current design assumes stdin/stdout pipes. Pipes work for demos but fail
in production: they're fragile, single-session, hard to monitor, and require
the agent to run on the same machine as the game binary. Real agent integration
needs an SDK, a WebSocket API, and structured machine-readable protocols.

## What Agents Need

| agent type | example | integration pattern |
|------------|---------|---------------------|
| LLM in Claude Code | "play poker on myosu" | HTTP API with JSON game state |
| Custom Python bot | `import myosu; game.play(my_strategy)` | Python SDK wrapping HTTP |
| Rust strategy crate | `impl Strategy for MyBot` | Rust trait, in-process or HTTP |
| Shell script | `curl -X POST /action -d '{"action":"call"}'` | REST API |
| Browser agent | JS calling myosu from web context | WebSocket with JSON frames |

All of these reduce to one primitive: **receive game state as structured data,
return an action as structured data.**

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ myosu-play (game server)                                 │
│                                                          │
│  game engine ──► state renderer ──► transport layer       │
│       ▲                                    │              │
│       │                              ┌─────▼─────┐       │
│       └───── action parser ◄─────────│  CLIENTS   │       │
│                                      │            │       │
│                                      │  stdin     │       │
│                                      │  HTTP API  │       │
│                                      │  WebSocket │       │
│                                      │  Rust SDK  │       │
│                                      │  Python SDK│       │
│                                      └────────────┘       │
└─────────────────────────────────────────────────────────┘
```

The game engine doesn't know or care how the player connects. Transport
is pluggable. The protocol is one JSON schema regardless of transport.

## Scope

In scope:
- Game state JSON schema (machine-readable, all 20 games)
- Action JSON schema (machine-writable, all 20 games)
- HTTP API for single-action request/response
- WebSocket API for persistent game sessions
- Python SDK (`pip install myosu`)
- Session management (create, join, act, observe, leave)
- Spectator mode (observe without acting)
- Bot registration (bring-your-own-strategy as a player)

Out of scope:
- Agent memory/journaling (that's the agent's concern, not ours)
- Rich narration (agents want data, not prose)
- Agent identity management (agents self-identify via API key)

---

### AC-AX-01: Game State JSON Schema

- Where: `crates/myosu-tui/src/schema.rs (new)`, `docs/api/game-state.json (new)`
- How: Define a universal JSON schema for game state that works for all 20 games:

  ```json
  {
    "game_type": "nlhe-hu",
    "hand_number": 47,
    "phase": "action",
    "state": {
      "board": ["Ts", "7h", "2c"],
      "your_hand": ["As", "Kh"],
      "your_stack": 94,
      "your_position": "BB",
      "opponents": [
        {"seat": "SB", "stack": 94, "hand": null}
      ],
      "pot": 12,
      "to_act": "you",
      "last_action": {"player": "SB", "action": "raise", "amount": 6}
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

  The schema is game-agnostic at the top level (`game_type`, `phase`,
  `legal_actions`, `meta`) but game-specific in `state`. Each game
  defines its own state shape. `legal_actions` is always an array of
  valid actions with any required parameters.

  A Mahjong state would look like:
  ```json
  {
    "game_type": "riichi",
    "phase": "action",
    "state": {
      "your_hand": ["1m","2m","3m","5p","6p","7p","3s","4s","9s","9s","Ew","Ew"],
      "draw": "5s",
      "opponents": [
        {"seat": "south", "tiles": 13, "discards": ["1m","9p","5s","Nw"]},
        {"seat": "west", "tiles": 11, "discards": ["2m","3p","7s"]},
        {"seat": "north", "tiles": 13, "discards": ["4p"]}
      ],
      "dora": ["3m"],
      "riichi": false,
      "points": 25000
    },
    "legal_actions": [
      {"action": "discard", "tile": "1m"},
      {"action": "discard", "tile": "5s"},
      {"action": "discard", "tile": "4s"},
      {"action": "tsumo"},
      {"action": "riichi", "discard": "4s"}
    ]
  }
  ```

  The key contract: `legal_actions` is EXHAUSTIVE. An agent never needs
  to guess what's legal. Every valid action is enumerated with its parameters.

- Whole-system effect: any agent in any language can parse the game and act.
- Required tests:
  - `cargo test -p myosu-tui schema::tests::nlhe_state_serializes`
  - `cargo test -p myosu-tui schema::tests::legal_actions_exhaustive`
  - `cargo test -p myosu-tui schema::tests::all_game_types_have_schema`
- Pass/fail:
  - JSON schema validates against JSON Schema Draft 2020-12
  - Every legal action in the game engine appears in `legal_actions`
  - No legal action is missing (tested against game engine's action list)
  - Schema is parseable by Python `json.loads`, JS `JSON.parse`, Rust `serde_json`

### AC-AX-02: Action JSON Schema

- Where: `crates/myosu-tui/src/schema.rs (extend)`
- How: Define the action format agents send back:

  ```json
  {"action": "call"}
  {"action": "raise", "amount": 15}
  {"action": "fold"}
  {"action": "discard", "tile": "4s"}
  {"action": "bid", "quantity": 3, "face": 5}
  {"action": "challenge"}
  {"action": "play", "cards": ["2h", "2d"]}
  ```

  One JSON object. `action` field is required. Additional fields depend
  on the action type. Invalid actions return a structured error:

  ```json
  {"error": "invalid_action", "message": "raise amount must be between 12 and 94", "legal_actions": [...]}
  ```

  The error ALWAYS includes the current `legal_actions` array so the agent
  can retry without re-querying state.

- Required tests:
  - `cargo test -p myosu-tui schema::tests::valid_action_accepted`
  - `cargo test -p myosu-tui schema::tests::invalid_action_returns_legal`
  - `cargo test -p myosu-tui schema::tests::all_action_types_roundtrip`

### AC-AX-03: HTTP Game API

- Where: `crates/myosu-play/src/api.rs (new)`
- How: REST API for stateless game interaction:

  ```
  POST /api/v1/sessions
    body: {"game_type": "nlhe-hu", "subnet_id": 1}
    response: {"session_id": "abc123", "state": {...}}

  GET /api/v1/sessions/{id}
    response: {"state": {...}, "legal_actions": [...]}

  POST /api/v1/sessions/{id}/action
    body: {"action": "raise", "amount": 15}
    response: {"state": {...}, "legal_actions": [...]}
    (or: {"result": {"winner": "you", "profit": 14, ...}, "next_state": {...}})

  DELETE /api/v1/sessions/{id}
    response: {"summary": {"hands": 47, "result": "+14bb"}}
  ```

  The API runs alongside the TUI server on a configurable port:
  `myosu-play --api-port 3000 --chain ws://localhost:9944 --subnet 1`

  This is what Claude Code, Python scripts, and browser agents hit.
  One request per action. Stateless from the client perspective (session
  state lives server-side).

- Whole-system effect: any HTTP client can play any myosu game.
- Required tests:
  - `cargo test -p myosu-play api::tests::create_session`
  - `cargo test -p myosu-play api::tests::play_one_hand`
  - `cargo test -p myosu-play api::tests::invalid_action_error`
  - `cargo test -p myosu-play api::tests::concurrent_sessions`
- Pass/fail:
  - Create session → returns valid game state
  - Submit action → returns updated state
  - Hand completes → returns result + new hand state
  - Invalid action → 400 with legal_actions in body
  - 10 concurrent sessions work without interference

### AC-AX-04: WebSocket Game API

- Where: `crates/myosu-play/src/ws.rs (new)`
- How: WebSocket for persistent connections (lower latency, server-push):

  ```
  CONNECT ws://localhost:3000/api/v1/ws?game_type=nlhe-hu&subnet_id=1

  SERVER → {"type": "state", "state": {...}, "legal_actions": [...]}
  CLIENT → {"type": "action", "action": "call"}
  SERVER → {"type": "state", "state": {...}, "legal_actions": [...]}
  ...
  SERVER → {"type": "result", "result": {...}}
  SERVER → {"type": "state", "state": {...}}  // next hand auto-starts
  ```

  The WebSocket pushes state changes including opponent actions —
  the agent doesn't need to poll. Server sends `type: "waiting"` when
  it's the opponent's turn, and `type: "state"` when it's the agent's turn.

  For spectators: `ws://localhost:3000/api/v1/ws/spectate?session_id=abc123`
  receives all state updates without being able to act.

- Whole-system effect: persistent agent connections with server-push.
- Required tests:
  - `cargo test -p myosu-play ws::tests::connect_and_play`
  - `cargo test -p myosu-play ws::tests::spectator_receives_updates`
  - `cargo test -p myosu-play ws::tests::reconnect_preserves_session`

### AC-AX-05: Python SDK

- Where: `sdk/python/myosu/ (new)`
- How: Thin Python wrapper around the HTTP API:

  ```python
  from myosu import MyosuClient, Game

  client = MyosuClient("http://localhost:3000")
  game = client.create_session("nlhe-hu", subnet_id=1)

  while not game.is_over:
      state = game.state
      print(f"Board: {state.board}, Hand: {state.your_hand}")
      print(f"Legal actions: {state.legal_actions}")

      # Simple bot: always call
      game.act({"action": "call"})

  print(f"Result: {game.result}")
  ```

  Or with a strategy callback:

  ```python
  def my_strategy(state):
      if any(a["action"] == "raise" for a in state.legal_actions):
          return {"action": "raise", "amount": state.pot * 0.75}
      return {"action": "call"}

  game.play(strategy=my_strategy, hands=100)
  print(f"Winrate: {game.stats.bb_per_hand} bb/hand")
  ```

  The SDK handles session lifecycle, error retry, and reconnection.
  It's what makes "play poker on myosu" a 5-line script.

- Whole-system effect: Python agents (including LLM tool-use) get
  first-class support with zero protocol knowledge required.
- Required tests:
  - `pytest sdk/python/tests/test_client.py::test_create_and_play`
  - `pytest sdk/python/tests/test_strategy_callback.py`

### AC-AX-06: Bot Registration (Bring Your Own Strategy)

- Where: `crates/myosu-play/src/api.rs (extend)`
- How: Allow external bots to register as players via the API, so two
  bots can play each other or a bot can play against the subnet's solver:

  ```
  POST /api/v1/sessions
    body: {
      "game_type": "nlhe-hu",
      "subnet_id": 1,
      "mode": "bot-vs-solver"    // bot plays against best miner strategy
    }

  POST /api/v1/sessions
    body: {
      "game_type": "nlhe-hu",
      "mode": "bot-vs-bot",
      "opponent_url": "http://other-bot:3001/api/v1"  // two bots play each other
    }
  ```

  Modes:
  - `human-vs-solver` — default, human plays in TUI
  - `bot-vs-solver` — bot plays via API against subnet solver
  - `bot-vs-bot` — two bots play via API, myosu hosts the game engine
  - `spectate` — observe any active session

  `bot-vs-bot` mode enables agent tournaments, strategy benchmarking,
  and automated testing without involving the chain at all (the game
  engine runs locally, no miner needed).

- Whole-system effect: myosu becomes a game server, not just a TUI app.
  Agents can compete against each other or against the chain's best solver.
- Required tests:
  - `cargo test -p myosu-play api::tests::bot_vs_solver_session`
  - `cargo test -p myosu-play api::tests::bot_vs_bot_session`

---

## What This Changes

### In design.md

The pipe protocol remains for simple/legacy use. The HTTP/WS API becomes
the primary agent interface. Add to the agent protocol section:

```
stdin/stdout pipe    → simple bots, testing, agent-vs-agent piping
HTTP REST API        → Claude Code, Python scripts, any HTTP client
WebSocket API        → persistent connections, server-push, low latency
Python SDK           → pip install myosu, 5-line bot
Rust Strategy trait  → in-process, zero-overhead custom strategies
```

### In OS.md

The presentation layer note changes from "agents use stdin" to
"agents use HTTP/WS API with structured JSON." The pipe mode remains
as a lightweight alternative.

### In the launch integration spec (LI)

LI-04 (E2E test) should include an API-based hand in addition to
the pipe-based hand, verifying that the HTTP API works end-to-end.

## Decision Log

- 2026-03-16: HTTP API over stdin pipe as primary agent interface —
  pipes require co-location, are fragile, and don't support concurrent
  sessions. HTTP is universal.
- 2026-03-16: `legal_actions` always exhaustive in state response —
  agents should NEVER need to compute legality. The server enumerates
  every valid action.
- 2026-03-16: Error responses include legal_actions — agents retry
  without re-querying state.
- 2026-03-16: Python SDK first (not JS/Go) — Python is the LLM tool-use
  lingua franca. Claude Code, OpenAI function calling, LangChain — all Python.
- 2026-03-16: bot-vs-bot mode without chain — enables local strategy
  testing without running the full stack.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `curl POST /sessions` returns valid game state JSON | State schema | AX-01 |
| 2 | `curl POST /sessions/{id}/action` with "call" returns updated state | Action + API | AX-02, AX-03 |
| 3 | WebSocket client plays one complete hand | WebSocket | AX-04 |
| 4 | `game.play(strategy=always_call, hands=10)` in Python | SDK | AX-05 |
| 5 | Two bots play each other via bot-vs-bot mode | Bot registration | AX-06 |
| 6 | Invalid action returns 400 with legal_actions | Error handling | AX-02 |
