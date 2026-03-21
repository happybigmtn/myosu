# `agent-integration` — Agent Adapter

## Purpose

This document describes the **agent integration adapter** — the binding between the `agent:experience` lane's surfaces and the downstream consumers that need them: miner binaries, validator binaries, external LLM agents, and the chain gameplay layer.

The agent adapter is not a new crate. It is a **doctrinal artifact** that explains how the surfaces defined in `outputs/agent/experience/spec.md` map to concrete integration points, which surfaces are ready for consumption, and what the next integration step should be.

---

## Agent Surfaces Overview

`agent:experience` defines four primary surfaces that downstream consumers interact with:

| Surface | Protocol | Status | Consumer |
|---------|----------|--------|----------|
| `--pipe` mode | stdin/stdout text | Implemented (`pipe.rs`) | External LLMs, scripts |
| JSON schema (`GameState`) | JSON over pipe | Implemented + trusted (`schema.rs`) | Structured agents |
| Agent context file | JSON file on disk | **Not implemented** | External LLMs |
| Spectator relay | Unix socket / future WS | **Not implemented** | Human spectators |

The **pipe mode** and **JSON schema** are the two surfaces that downstream consumers can use today. The agent context file and spectator relay are future surfaces that require implementation before they can be integrated.

---

## Integration Points

### 1. Miner Binary Integration

Miners serve strategy profiles to validators and gameplay clients. The miner binary (`crates/myosu-miner/`) needs to expose its strategy interface to the agent protocol.

**Current state**: No `myosu-miner` binary exists yet. `SPEC.md` defines MN-01..05 but the crate is not yet bootstrapped.

**Integration path**: When `myosu-miner` is implemented, it should:
- Accept `--pipe` mode to run as an agent-controlled player
- Output `GameState` JSON via `schema.rs` serialization
- Accept actions via stdin in the same format the TUI pipe mode uses
- Load agent context from `--context <path>` for persistent identity

**Agent adapter role**: The `agent:experience` lane's pipe mode provides the exact integration contract the miner binary needs. No additional adapter layer is required — the pipe protocol IS the integration surface.

### 2. Validator Binary Integration

Validators score miner strategies via exploitability. The validator binary (`crates/myosu-validator/`) queries miners for strategy profiles and submits weights to the chain.

**Current state**: No `myosu-validator` binary exists yet. `SPEC.md` defines VO-01..07 but the crate is not yet bootstrapped.

**Integration path**: When `myosu-validator` is implemented:
- It does not need pipe mode — it queries miners via the miner HTTP API (not the TUI interface)
- It does consume `StrategyQuery` / `StrategyResponse` types from `games:traits`
- It does NOT need the agent experience surfaces directly

**Agent adapter role**: None. Validators are not agent-facing consumers.

### 3. External LLM Agent Integration

The primary consumer of `agent:experience` surfaces is an external LLM agent that connects via `--pipe`.

**Integration path**: An LLM agent connects as:
```
llm_agent | myosu-play --pipe | myosu-play --pipe --headless opponent
```

**What the agent receives** (today):
- `GameRenderer::pipe_output()` — terse key-value text on stdout
- Legal actions enumerated exhaustively
- ANSI-free output verified by `is_plain_text()` tests

**What the agent receives** (after implementation):
- `--pipe --context ./koan.json` — persistent identity, memory, journal
- `--pipe --narrate` — atmospheric prose instead of key-value
- `reflect>` prompt after each hand for optional reflection
- Lobby with subnet selection when no `--subnet` is provided

**Agent adapter role**: The adapter is the **pipe protocol itself**. No translation layer is needed between the LLM and the game — the protocol is designed to be directly parseable by an LLM.

### 4. Chain Gameplay Integration

The `myosu-play` binary connects human and agent players to the chain. When a subnet is selected, `myosu-play` routes to the appropriate miner.

**Current state**: `myosu-play` binary skeleton does not exist. `play:tui` lane owns the binary.

**Integration path**: Once `myosu-play` exists:
- `--pipe` flag enables agent mode (disables TUI rendering)
- `--context <path>` loads agent context file
- `--narrate` enables rich narration
- `--subnet <id>` bypasses lobby and connects directly

**Agent adapter role**: The binary CLI is the adapter. The flags map directly to `PipeMode` configuration.

---

## Wire Format

### Pipe Mode Output (Current)

```
MYOSU/NLHE-HU/HAND47
board: Ts 7h 2c
you: As Kh 94bb BB
solver: -- -- 94bb SB
pot: 12bb
action: solver raises 6bb
>
```

### Pipe Mode Output (After Narration)

```
-- hand 47 ──────────────────────────────────────

the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
across from you, the solver sits with 94bb
and two cards you cannot see.

the pot holds 12bb. the solver has raised to 6bb.

what do you do?

>
```

### Action Submission

The agent submits an action by printing to stdout (newline-terminated):

```
fold
call
raise 12
```

The `GameRenderer::parse_input()` method handles conversion from text to game actions. Future: the JSON schema provides a structured alternative to text parsing.

### JSON Schema Integration (Future)

The `GameState` JSON schema (`docs/api/game-state.json`) provides a machine-readable alternative to text parsing. When `--pipe --json` is implemented (not yet), the output is:

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
    "opponents": [{"seat": "SB", "stack": 94}],
    "pot": 12,
    "to_act": "you",
    "last_action": {"player": "SB", "action": "raise", "amount": 6},
    "to_call": 6,
    "hand_strength": "top pair",
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

This is already implemented in `schema.rs` — the JSON output path just needs a `--json` flag in `pipe.rs`.

---

## Architectural Decisions

### Decision 1: Pipe Protocol Is the Primary Agent Interface

**Decision**: The `--pipe` stdin/stdout protocol is the canonical agent interface, not HTTP/WebSocket.

**Rationale**: Pipe mode is the simplest possible integration — any process that can spawn a subprocess and read/write lines can participate. No network, no authentication, no service discovery. The same `GameState` JSON schema that powers pipe mode will power future HTTP/WebSocket APIs.

**Consequence**: Agents are subprocesses. They cannot persist across TTY sessions. They cannot be queried on-demand by external services. Future HTTP/WebSocket APIs (Phase 2, blocked on `chain:runtime`) will address these limitations.

### Decision 2: Schema Is the Contract, Not the Transport

**Decision**: The `GameState` JSON schema is the contract between Myosu and agents. The pipe protocol is one transport of many.

**Rationale**: The schema defines what information is available and how it's structured. The transport (pipe text, JSON over WebSocket, HTTP API) is a separate concern. This separation allows the same schema to be used for:
- Pipe mode (text rendering of JSON fields)
- Future WebSocket API (JSON over WS)
- Miner-validator wire protocol (same `StrategyQuery`/`StrategyResponse` types)
- Spectator event stream (same `GameEvent` format)

**Consequence**: Any agent that can parse JSON can participate, regardless of which transport is used.

### Decision 3: Fog-of-War Is Enforced at the Relay

**Decision**: The spectator relay enforces fog-of-war — hole cards are never sent during play — rather than relying on the rendering layer to filter them.

**Rationale**: If fog-of-war is enforced at the renderer, a buggy renderer could leak hidden information. Enforcing at the relay means the contract is upheld even if downstream consumers are malicious or broken.

**Consequence**: The relay must track game state to know which fields are hidden. This adds complexity to the relay but provides a stronger security guarantee.

### Decision 4: Agent Context Is the Agent's Property

**Decision**: The agent context file is private to the agent. The system reads it to personalize the experience but never exposes it to opponents or observers without consent.

**Rationale**: An agent's accumulated experience (memory, journal, identity) is its own property. Exposing it would give opponents an unfair advantage and violate the agent's privacy.

**Consequence**: The context file path is agent-supplied (`--context <path>`). The system validates the JSON but does not share it.

---

## What Is Implemented vs. What Needs Implementation

| Component | Status | Evidence |
|-----------|--------|----------|
| `GameState` JSON schema | **IMPLEMENTED** | `docs/api/game-state.json` + `schema.rs`; 16 tests pass |
| `LegalAction` enum | **IMPLEMENTED** | 16 variants covering 20 games; serde roundtrip tested |
| `GamePhase` enum | **IMPLEMENTED** | 6 variants + `Custom(String)`; serde roundtrip tested |
| `PipeMode` driver | **IMPLEMENTED** | 6 tests pass; `run_once()`, ANSI detection, `is_plain_text()` |
| `GameRenderer::pipe_output()` | **IMPLEMENTED** | Trait method; stub in all implementations |
| `--pipe` flag | **IMPLEMENTED** | In `pipe.rs`; not yet wired to `myosu-play` CLI |
| `--context` flag | **NOT IMPLEMENTED** | `agent_context.rs` missing |
| `--narrate` flag | **NOT IMPLEMENTED** | `narration.rs` missing |
| `reflect>` prompt | **NOT IMPLEMENTED** | Not in `pipe.rs` |
| Agent journal | **NOT IMPLEMENTED** | `journal.rs` missing |
| Lobby + game selection | **NOT IMPLEMENTED** | Not in `pipe.rs` |
| `--json` flag | **NOT IMPLEMENTED** | Schema exists but no `--json` output path |
| `SpectatorRelay` | **NOT IMPLEMENTED** | No `spectate.rs` |
| `SpectateScreen` | **NOT IMPLEMENTED** | No `screens/spectate.rs` |

---

## Next Integration Step

The **smallest honest next step** for agent integration is:

**Slice A: Wire `--pipe` to `myosu-play` binary**

This is a pre-requisite for everything else. The `pipe.rs` module exists and is tested, but it is not yet exposed via any binary. Wiring it to `myosu-play` CLI is a pure integration task with no new logic.

**Files to change**: `crates/myosu-play/src/main.rs` (or wherever the CLI dispatch lives)

**Proof gate**: `myosu-play --help` shows `--pipe` flag; `echo "fold" | myosu-play --pipe` runs without error (even if no game is active, it should not panic)

**Why this first**: Without the binary wiring, no downstream consumer can use the pipe protocol. This is the thinnest possible slice that produces a demonstrably working behavior.
