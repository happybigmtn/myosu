# `agent:experience` Adapter Surface

## Purpose

This document describes how the `agent:experience` lane's surfaces integrate with the broader Myosu system. It defines the integration contracts, adapter responsibilities, and data flow between the agent-facing presentation layer and the rest of the stack.

This is the **integration adapter** for the `agent` frontier — it bridges `agent:experience` lane outputs to the `play:tui` binary, the chain runtime, and downstream consumers (spectators, miners, validators).

---

## Lane Context

```
                          agent:experience outputs
                          ┌─────────────────────────────────────────────────────────┐
                          │                                                          │
upstream tui:shell        │  GameRenderer::pipe_output() ──────► pipe mode text    │
  (82 tests, trusted)      │                                                          │
                          │  schema.rs (GameState JSON) ──────► API contract       │
upstream games:traits      │                                                          │
  (14 tests, trusted)      │  --context flag ─────────────────► AgentContext      │
                          │  --narrate flag ──────────────────► NarrationEngine   │
                          │  --spectate flag ──────────────────► SpectatorRelay    │
                          │                                                          │
                          └────────────────────┬────────────────────────────────────┘
                                               │
                          downstream consumers  │
                          ┌────────────────────▼────────────────────────────────────┐
                          │                                                          │
myosu-play binary          │  CLI dispatch: --pipe, --context, --narrate, --spectate │
(play:tui lane, Slice 1)  │  Entry point: crates/myosu-play/src/main.rs           │
                          │                                                          │
chain:runtime             │  Lobby queries (Phase 4) ─────────────────────────►    │
(chain:runtime lane)      │  WebSocket upgrade (Phase 4) ────────────────────►     │
                          │                                                          │
miner axon                │  Spectator relay (Phase 1) ──────────────────────►     │
(miner lane, future)      │                                                          │
                          └──────────────────────────────────────────────────────────┘
```

---

## Integration Points

### 1. `myosu-play` Binary Integration

**Consumer**: `play:tui` lane, Slice 1 (binary skeleton)

The `myosu-play` binary is the delivery vehicle for all `agent:experience` surfaces. The binary dispatches on `--pipe`, `--context`, `--narrate`, and `--spectate` flags.

**CLI contract** (from `agent:experience/spec.md`):

```bash
# Pipe mode with agent context and narration
myosu-play --pipe --context ./koan.json --narrate

# Spectator mode
myosu-play --spectate --session <session_id>

# Lobby (no subnet flag)
myosu-play --pipe
```

**Adapter responsibility**: The `myosu-play` binary does not implement agent logic — it imports `PipeMode`, `AgentContext`, `NarrationEngine`, `Journal`, and `SpectatorRelay` from `myosu-tui` and wires them to CLI flags.

**File**: `crates/myosu-play/src/main.rs` (created in `play:tui` Slice 1)

---

### 2. `GameRenderer::pipe_output()` Contract

**Provider**: `tui:shell` lane (trusted upstream)

`pipe_output()` is the core rendering method. It is a trait method on `GameRenderer` that renders game state as terse key-value text for pipe mode consumption.

**Contract**:
```rust
pub trait GameRenderer {
    fn render_state(&self, state: &GameState, frame: &mut Frame);
    fn pipe_output(&self, state: &GameState) -> String;
}
```

**Adapter responsibility**: `agent:experience` uses `pipe_output()` directly. The method must produce stable, parseable output across all game types. Any changes to `pipe_output()` output format constitute a breaking change for agent consumers.

---

### 3. JSON Schema Contract

**Provider**: `schema.rs` (trusted, 16 tests)

The `GameState` JSON schema (`docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs`) is the machine-readable contract for structured agents.

**Schema scope** (from `agent:experience/spec.md`):
- 10 game types covered
- Exhaustive `legal_actions` enumeration
- `GameStateBuilder` for construction
- `LegalAction` enum for valid moves
- `GamePhase` for game progression

**Adapter responsibility**: The schema is the canonical representation of game state. All agent-facing surfaces (pipe mode, narration, spectator relay events) must be representable as `GameState` or a subset thereof.

---

### 4. Agent Context File Contract

**Provider**: `agent_context.rs` (MISSING — Slice 1 of `agent:experience`)

The context file provides persistent identity, memory, and journal across sessions.

**Schema** (from `specsarchive/031626-10-agent-experience.md` AC-AX-01):
```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [...] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

**Adapter responsibility**: The context file is loaded at startup and saved at shutdown. It is never exposed to opponents. Serde validation is applied on load.

---

### 5. Narration Engine Contract

**Provider**: `narration.rs` (MISSING — Slice 5 of `agent:experience`)

The `--narrate` flag activates atmospheric prose rendering instead of terse key-value text.

**Example output** (from `agent:experience/spec.md`):
```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
```

**Adapter responsibility**: `NarrationEngine::narrate(&GameState) -> String` must produce prose that is semantically equivalent to the `GameState` but rendered in natural language. The same `GameState` produces both terse and narrated output.

---

### 6. Journal Contract

**Provider**: `journal.rs` (MISSING — Slice 2 of `agent:experience`)

The journal is append-only markdown recording every hand and agent reflections.

**Format**:
```markdown
# Session 23

## Hand 47
board: T♠ 7♥ 2♣
held: A♠ K♥
result: +14bb (showdown)
opponent: Q♣ J♣

reflection: The river was a thin value bet. Given the dry texture,
I should have sized larger to extract from worse hands.
```

**Adapter responsibility**: Journal entries are never truncated. The journal file grows indefinitely. `Journal::append_hand_entry()` is called after each hand in pipe mode.

---

### 7. Reflection Channel Contract

**Provider**: `pipe.rs` (extended in Slice 4 of `agent:experience`)

After each hand in pipe mode, the `reflect>` prompt appears:
```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```

**Adapter responsibility**: Empty line skips the reflection. Non-empty input is appended to the journal entry for that hand.

---

### 8. Spectator Relay Contract

**Provider**: `spectate.rs` (MISSING — Slice 8 of `agent:experience`)

Phase 0: Unix domain socket at `~/.myosu/spectate/<session_id>.sock`

**Event format** (JSON lines):
```json
{"type": "hand_start", "session": "abc123", "board": "T♠ 7♥ 2♣", "pot": 50.0}
{"type": "action", "player": "hero", "action": "bet", "amount": 25.0}
{"type": "showdown", "hero_hole_cards": "A♠ K♥", "villain_hole_cards": "Q♣ J♣"}
```

**Fog-of-war enforcement**: Hole cards are NEVER sent during active play. Only `showdown` events include hole cards.

**Adapter responsibility**: The relay enforces fog-of-war at the relay layer, not at the renderer. This ensures no hole card leakage regardless of how many spectators are connected.

---

### 9. Lobby + Game Selection Contract

**Provider**: `pipe.rs` (extended in Slice 7 of `agent:experience`)

When `--pipe` is used without `--subnet`, the lobby is displayed:
```
MYOSU/LOBBY
subnets:
  1 nlhe-hu    12 miners  13.2 mbb/h  ACTIVE
  2 nlhe-6max  18 miners  15.8 mbb/h  ACTIVE
>
```

**Adapter responsibility**: Lobby data is hardcoded for Phase 0 (stubbed). Phase 4 will query the chain or miner axon for real data.

---

## Phase 4 Integration (Chain-Connected)

When `chain:runtime` is ready, the following integrations activate:

| Integration | Phase | Description |
|------------|-------|-------------|
| Lobby chain query | Phase 4 | Lobby queries miner axon HTTP endpoint for active subnet list |
| Spectator WS upgrade | Phase 4 | Unix socket spectator relay upgrades to WebSocket via miner axon |
| Strategy queries | Phase 4 | Pipe mode queries miner for strategy distribution via HTTP |

---

## File Manifest

| File | Owner | Status |
|------|-------|--------|
| `crates/myosu-play/src/main.rs` | `play:tui` | Created in Slice 1 |
| `crates/myosu-tui/src/pipe.rs` | `agent:experience` | Extended in Slices 3, 4, 6, 7 |
| `crates/myosu-tui/src/agent_context.rs` | `agent:experience` | Created in Slice 1 |
| `crates/myosu-tui/src/narration.rs` | `agent:experience` | Created in Slice 5 |
| `crates/myosu-tui/src/journal.rs` | `agent:experience` | Created in Slice 2 |
| `crates/myosu-play/src/spectate.rs` | `agent:experience` | Created in Slice 8 |
| `crates/myosu-tui/src/screens/spectate.rs` | `agent:experience` | Created in Slice 9 |
| `docs/api/game-state.json` | `agent:experience` | TRUSTED |
| `crates/myosu-tui/src/schema.rs` | `agent:experience` | TRUSTED |

---

## Integration Decision Log

### Decision: Fog-of-War Enforcement at Relay (not Renderer)

**Date**: 2026-03-20

**Choice**: Fog-of-war is enforced at the `SpectatorRelay` layer, not at the `GameRenderer`.

**Rationale**: If fog-of-war is enforced at the renderer, a bug in the renderer could leak hole cards to the spectator output. By enforcing fog-of-war at the relay, we guarantee that hole cards never leave the relay regardless of how many spectators connect or how the renderer evolves.

**Alternative considered**: Enforcing fog-of-war in `GameRenderer::pipe_output()` or a dedicated spectator render method. Rejected because it requires every render path to be fog-of-war aware, creating ongoing coupling risk.
