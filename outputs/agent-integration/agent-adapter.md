# `agent:integration` — Agent Adapter Capability Spec

Status: Draft
Date: 2026-03-20
Type: Capability Spec
Lane: `agent-integration`
Supersedes: none

---

## Purpose / User-Visible Outcome

This spec defines the **agent adapter layer** — the contract by which external agents
(LLMs, bots, scripts) connect to Myosu's game-solving infrastructure via the
`agent:experience` surface. It describes the integration points, wire format, trust
boundaries, and the minimal adapter types needed to act as an honest participant.

An agent that implements this adapter can:

- Play any Myosu game via the `--pipe` stdin/stdout protocol
- Persist identity and accumulate experience across sessions via the `--context` file
- Consume narrated game state via the `--narrate` flag
- Observe another agent's session via the spectator relay socket
- Query the game lobby to select which subnet to enter

---

## Whole-System Goal

Myosu's agent integration follows the same **permissionless participation** model as miner
and validator participation: any agent that obeys the wire protocol can connect and play,
accumulating a journal of experience over sessions. The adapter layer is deliberately thin
to minimize the integration burden on agents.

---

## Integration Surfaces

```
external agent
    │
    │  stdin/stdout (pipe protocol)
    ▼
┌─────────────────────────────────────────────────────────────┐
│  myosu-play --pipe [--context <path>] [--narrate]          │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ pipe.rs: PipeMode                                     │  │
│  │   • output_state() → plain-text state to stdout       │  │
│  │   • read_input() → agent action from stdin           │  │
│  │   • run_once() → output + read loop helper           │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ schema.rs: GameState JSON                            │  │
│  │   • machine-readable game state                       │  │
│  │   • exhaustive legal_actions array                   │  │
│  │   • game_type, phase, state_wrapper, meta            │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ agent_context.rs (MISSING — Slice 1 of agent:exp)   │  │
│  │   • AgentContext: identity, memory, journal          │  │
│  │   • load() / save() roundtrip across sessions       │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ narration.rs (MISSING — Slice 5 of agent:exp)        │  │
│  │   • NarrationEngine::narrate(&GameState) -> String  │  │
│  │   • board texture, session arc, opponent history     │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ SpectatorRelay (MISSING — Slice 8 of agent:exp)     │  │
│  │   • Unix socket: ~/.myosu/spectate/<session>.sock   │  │
│  │   • fog-of-war enforced at relay                    │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
    │
    │  (game logic via GameRenderer trait → games:traits)
    ▼
robopoker (external, absolute-path dep — git migration pending)
```

---

## Wire Protocol

### Pipe Mode (`--pipe`)

The pipe mode protocol is a **line-oriented plain-text stdin/stdout dialogue** between
an agent and `myosu-play`. No ANSI codes, no cursor manipulation, no color — only
human-readable text and line-delimited JSON where specified.

**Protocol flow:**

```
myosu-play --pipe [--context ./koan.json] [--narrate]
  │
  │  [on game state change, line printed to stdout]
  ▼
STATE preflop pot=4bb hero=AsKh stack=100bb actions=fold,call,raise
  │
  │  [agent reads line, decides, writes action to stdin]
  ▼
raise 12
  │
  │  [myosu-play processes action, advances game, outputs new state]
  ▼
STATE flop pot=18bb board=Ts7h2c hero=AsKh stack=88bb to_call=6bb actions=fold,call,raise
  │
  │  [... continues until hand complete ...]
  ▼
HAND COMPLETE
result: +14bb (showdown, AsKh vs QcJd)
session: +28bb over 47 hands

reflect>
  │
  │  [agent writes optional reflection, or empty line to skip]
  ▼
  [reflection appended to journal]
```

**Key design rules:**
- Output lines always end with `\n` and are flushed immediately
- No output line ever contains ANSI escape sequences (`\x1b[`)
- All state is encoded as `KEY=value` pairs in a fixed order
- Actions are space-separated: `fold`, `call`, `raise <amount>`, `shove`
- `HAND COMPLETE` is emitted after every hand regardless of outcome
- `reflect>` prompt blocks for input; empty line skips; non-empty line is appended to journal

### JSON Schema Mode

For structured agents that prefer JSON over text parsing, `schema.rs` defines a complete
`GameState` schema. The JSON representation is available via the same `GameRenderer` trait
— specifically, a `serde_json::to_string(&game_state)` call on the `GameState` produced
by the game engine.

The JSON schema covers 10 game types with exhaustive `legal_actions` arrays.

**Example JSON state:**

```json
{
  "game_type": "nlhe_hu",
  "hand_number": 47,
  "phase": "action",
  "state": {
    "board": ["Ts", "7h", "2c"],
    "your_hand": ["As", "Kh"],
    "your_stack": 88,
    "your_position": "BB",
    "opponents": [{"seat": "SB", "stack": 88, "hand": null}],
    "pot": 18,
    "to_act": "you",
    "last_action": {"player": "SB", "action": "raise", "amount": 6},
    "to_call": 6,
    "street": "flop"
  },
  "legal_actions": [
    {"action": "fold"},
    {"action": "call", "amount": 6},
    {"action": "raise", "min": 12, "max": 88},
    {"action": "shove", "amount": 88}
  ],
  "meta": {
    "solver_source": "miner-12",
    "solver_exploitability": 13.2,
    "subnet_id": 1
  }
}
```

**Key contract:** The `legal_actions` array is **exhaustive**. An agent never needs to
guess what is valid. Every legal action is enumerated with its parameters.

---

## Agent Adapter Types

The following types form the adapter surface. All are defined within
`crates/myosu-tui/src/` or `crates/myosu-play/src/`.

### Trusted Types (exist, tested)

| Type | File | Responsibility |
|------|------|---------------|
| `GameState` | `schema.rs` | Universal JSON game state; `game_type`, `phase`, `state_wrapper`, `legal_actions`, `meta` |
| `LegalAction` | `schema.rs` | Exhaustive action enum; `Fold`, `Call { amount }`, `Raise { min, max }`, `Shove { amount }`, etc. |
| `GamePhase` | `schema.rs` | `Waiting`, `Action`, `Betting`, `Showdown`, `Complete`, `Ended`, `Custom(String)` |
| `MetaInfo` | `schema.rs` | Solver metadata; `solver_source`, `solver_exploitability`, `subnet_id`, `miner_uid` |
| `AgentAction` | `schema.rs` | Agent-submitted action; variants mirror `LegalAction` |
| `ActionError` | `schema.rs` | Error response with `error`, `message`, `legal_actions` fields |
| `PipeMode` | `pipe.rs` | `new(renderer)`, `output_state()`, `read_input()`, `run_once()` |
| `is_plain_text()` | `pipe.rs` | Validates no ANSI codes in output |

### Missing Types (Slice 1–9 of `agent:experience`)

| Type | File | Status | Responsibility |
|------|------|--------|----------------|
| `AgentContext` | `agent_context.rs` | **MISSING** | Identity/memory/journal; `load()`, `save()`, `default()` |
| `Journal` | `journal.rs` | **MISSING** | Append-only markdown writer; `append_hand_entry()`, never truncates |
| `NarrationEngine` | `narration.rs` | **MISSING** | Prose rendering; `narrate(&GameState) -> String` |
| `SpectatorRelay` | `spectate.rs` | **MISSING** | Unix socket event relay; fog-of-war enforced at relay |

---

## Trust Boundaries

```
┌──────────────────────────────────────────────────────────────────┐
│  TRUSTED (myosu-tui crate boundary)                              │
│                                                                  │
│  AgentContext JSON file ──► serde validated ──► AgentContext     │
│  (agent-supplied)         (fails safe)        (never exposed)    │
│                                                                  │
│  Reflection text ──► Journal.append() ──► journal.md             │
│  (free-form string, no parsing, no evaluation)                   │
│                                                                  │
│  SpectatorRelay ──► fog-of-war filter ──► socket output         │
│  (hole cards redacted during play; showdown reveals)              │
└──────────────────────────────────────────────────────────────────┘
│  UNTRUSTED                                                       │
│                                                                  │
│  GameRenderer impl ──► provided by game engine (untrusted)       │
│  pipe_output() text ──► agent parses (agent's responsibility)    │
└──────────────────────────────────────────────────────────────────┘
```

**Invariant:** Agent context files are serde-validated before use. Invalid JSON produces
an error, never a panic, and never exposes unvalidated data to opponents.

**Invariant:** The spectator relay enforces fog-of-war at the relay — not in the renderer.
Hole cards are stripped from events before they reach the socket. This is a hard
enforcement boundary: the renderer has no knowledge of the relay's filtering.

---

## Context File Schema

The `--context <path>` flag loads a JSON file with the following schema:

```json
{
  "identity": {
    "name": "koan",
    "created": "2026-03-01T00:00:00Z",
    "games_played": 1847,
    "preferred_game": "nlhe-hu"
  },
  "memory": {
    "session_count": 23,
    "lifetime_result": "+342bb",
    "observations": [
      {
        "pattern": "overpair-on-dry-board",
        "hands": 12,
        "result": "+8bb",
        "notes": "..."
      }
    ]
  },
  "journal": [
    {
      "session": 23,
      "hand": 47,
      "board": "Ts7h2c",
      "held": "AsKh",
      "result": "+14bb",
      "reflection": "raised too large on the flop — fold equity was lower than I estimated"
    }
  ]
}
```

**Load/save contract:**
- `AgentContext::load(path)` — read JSON, validate, return `AgentContext`
- `AgentContext::save(path)` — serialize to JSON, write atomically (write-then-rename)
- Missing file → return `AgentContext::default()` (new identity)
- Invalid JSON → return error (does not create default)

---

## Journal Format

The journal is an append-only markdown file. It is never truncated or rewritten.

```markdown
# Agent Journal — koan

## Session 23 — 2026-03-20

### Hand 47 — NLHE Heads-Up
- Board: `Ts 7h 2c`
- Held: `As Kh`
- Stack: 88bb (from 100bb)
- Result: **+14bb** (showdown)

> reflection: raised too large on the flop — fold equity was lower than I estimated

### Hand 48 — NLHE Heads-Up
- Board: `Jh 8s 4c`
- Held: `Qc Qd`
- Stack: 102bb (from 88bb)
- Result: **-6bb** (fold)

```

**Format rules:**
- Each hand entry starts with `### Hand N`
- Each session starts with `## Session N`
- Each file starts with `# Agent Journal — {identity.name}`
- Reflection is prefixed with `> reflection:` on its own line
- Empty reflection → no `> reflection:` line

---

## Spectator Relay Protocol

**Socket path:** `~/.myosu/spectate/<session_id>.sock`

**Phase 0 (local):** Unix domain socket. Agents or humans connect as spectators to watch
an active session.

**Event format (JSON lines):**

```json
{"type": "hand_start", "hand": 47, "hero": "AsKh", "villain": "??", "pot": 4}
{"type": "board", "street": "flop", "cards": ["Ts", "7h", "2c"]}
{"type": "action", "player": "villain", "action": "raise", "amount": 6}
{"type": "action", "player": "hero", "action": "call"}
{"type": "board", "street": "turn", "cards": ["Ts", "7h", "2c", "Ks"]}
{"type": "showdown", "villain_hand": ["Qc", "Jd"], "pot": 18, "winner": "hero"}
{"type": "hand_end", "result": "+14bb", "session": "+28bb over 47 hands"}
```

**Fog-of-war enforcement:**
- `hand_start` never includes `villain_hand` during play
- `board` never includes hero's hole cards (only community cards)
- `showdown` event includes `villain_hand` — this is the only reveal point
- Hole cards are `null` or `??` in all pre-showdown events

---

## Next Slices

The `agent:experience` lane defines 9 slices that implement the missing adapter types.
The slices have a clean dependency chain:

```
Phase 1 (depends only on tui:shell — trusted):
  Slice 1: agent_context.rs    → AgentContext load/save/default
  Slice 2: journal.rs         → Journal append-only writer
  Slice 3: --context wiring   → PipeMode + context path
  Slice 4: reflect> prompt    → HAND COMPLETE + stdin read

Phase 2 (depends on Phase 1):
  Slice 5: narration.rs       → NarrationEngine prose
  Slice 6: --narrate wiring  → PipeMode + narration toggle
  Slice 7: lobby              → pipe mode without --subnet

Phase 3 (depends on play:tui binary + Phase 1):
  Slice 8: SpectatorRelay     → Unix socket relay
  Slice 9: SpectateScreen     → Spectator TUI screen

Phase 4 (depends on chain:runtime):
  Lobby queries miner axon; spectator upgrades to WebSocket
```

**This lane does not own implementation.** It owns the adapter spec and the review
judgment. Implementation belongs to `agent:experience`.

---

## Decision Log

- 2026-03-20: Drafted agent-adapter.md. Integration surfaces derived from
  `agent:experience` spec and `schema.rs` / `pipe.rs` inspection.
  Decision: adapter types grouped into trusted (existing) and missing (pending slices).
  Decision: SpectatorRelay fog-of-war enforced at relay, not renderer — hard boundary.
