# Agent Integration Adapter — `agent:experience` Lane Integration Contract

## Purpose

This document defines the integration contract between external agents (LLMs, bots, scripts) and the Myosu game-solving protocol via the `agent:experience` lane surfaces. It serves as the authoritative reference for what agents can rely on and what obligations Myosu guarantees across sessions.

## Agent Integration Points

An external agent integrates with Myosu through **four primary surfaces**:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         External Agent (LLM/Bot/Script)                  │
│                                                                          │
│   stdin/stdout ────► myosu-play --pipe ─────► Game state (pipe text)    │
│   context file  ────► --context <path>     ──► Persistent memory       │
│   narration     ────► --narrate            ──► Prose game state        │
│   reflection   ◄───► reflect> prompt      ◄── Optional introspection   │
│   journal      ◄───► Append-only .md      ◄── Autobiography          │
│   spectate     ◄───► ~/.myosu/spectate/  ◄── Socket event stream      │
└─────────────────────────────────────────────────────────────────────────┘
```

## Surface 1: Pipe Mode (`--pipe`)

Pipe mode is the primary agent integration primitive. It provides a **synchronous stdin/stdout text protocol** that works with any LLM or scripting environment.

### Protocol Contract

```
# Session initialization
myosu-play --pipe [--context <path>] [--narrate]

# Each game state transition, Myosu outputs:
STATE <key>=<value> ...

# Agent responds with:
<action>
```

### Pipe Output Format

The `GameRenderer::pipe_output()` contract produces:

```
STATE street=<preflop|flop|turn|river|showdown> board=<cards> pot=<bb> hero=<hand> stack=<bb> to_call=<bb> actions=<comma,separated>
```

Example:
```
STATE flop Ts7h2c pot=12bb hero=AcKh stack=88bb to_call=4bb actions=fold,call,raise
```

**Requirements:**
- No ANSI escape codes in pipe output
- Flush stdout after every write (agents expect immediate updates)
- `writeln!` with newline terminator on every output

### Input Parsing

Agent inputs are parsed via `GameRenderer::parse_input()`:

| Shorthand | Full      | Action      |
|-----------|-----------|-------------|
| `f`       | `fold`    | Fold hand   |
| `c`       | `call`    | Call bet    |
| `r <n>`   | `raise <n>` | Raise to n |

Invalid input returns `None` — the renderer then calls `clarify()` for a prompt.

## Surface 2: Agent Context (`--context <path>`)

The agent context file provides **persistent identity and memory across sessions**.

### Context File Schema

```json
{
  "identity": {
    "name": "koan",
    "created": "2026-03-17T00:00:00Z",
    "games_played": 1847,
    "preferred_game": "nlhe-hu"
  },
  "memory": {
    "session_count": 23,
    "lifetime_result": "+342bb",
    "observations": [
      {"hand": 47, "note": "overvalue on flush draw"}
    ]
  },
  "journal": [
    {"session": 23, "hand": 47, "reflection": "..."}
  ]
}
```

### Context Lifecycle

| Event | Action |
|-------|--------|
| Context file missing | Create new default identity |
| Context file present | Load and validate JSON schema |
| Session start | Load context, begin game |
| Session end | Save context, append journal entry |
| Invalid JSON | Fatal error — do not start |

### Journal Format

The journal is **append-only markdown**:

```markdown
# Session 23 — 2026-03-17

## Hand 47
**Board:** Ts 7h 2c
**Hero:** Ac Kh
**Result:** +14bb (showdown)
**Opponent:** Qc Jd

## Reflection
Needed to fold the flush draw on the river.
```

## Surface 3: Narration Mode (`--narrate`)

When `--narrate` is set, game state is rendered as **atmospheric prose** instead of terse key-value pairs.

### Prose Contract

Narration must include:

1. **Board texture**: "dry", "wet", "connected", paired, etc.
2. **Stack trajectory**: How stacks have changed across the session
3. **Opponent tendency**: Inferred from context file memory
4. **Atmospheric description**: Cards as story elements, not data

Example:
```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
```

### Constraints

- **Same game state** in both `--narrate` and default mode (no information asymmetry)
- **No invented information** — narration derives only from `GameState` and context memory
- **Deterministic** — same state always produces same narration (no randomness)

## Surface 4: Spectator Relay (Phase 0)

A **Unix domain socket** emits JSON game events for watching live play.

### Socket Contract

```
Path: ~/.myosu/spectate/<session_id>.sock
Format: One JSON line per event
```

### Event Schema

```json
{
  "event_type": "hand_complete",
  "hand_number": 47,
  "board": ["Ts", "7h", "2c"],
  "hero_hand": ["Ac", "Kh"],
  "result": "+14bb",
  "showdown": true
}
```

### Fog-of-War Enforcement

**Critical**: Hole cards are **never emitted during active play**. The relay enforces fog-of-war:

| Phase | Hero Hand | Opponent Hand |
|-------|-----------|---------------|
| Action | `null` | `null` |
| Showdown | `["Ac", "Kh"]` | `["Qc", "Jd"]` |

The relay, not the renderer, enforces this invariant.

## Integration Checklist

For an agent to successfully integrate:

- [ ] Connect via `myosu-play --pipe`
- [ ] Provide `--context` for persistent memory
- [ ] Parse `STATE` output for game state
- [ ] Send actions via shorthand (`f/c/r`) or full words
- [ ] Read `HAND COMPLETE` and optional `reflect>` prompt
- [ ] Optionally enable `--narrate` for prose mode
- [ ] Optionally connect to spectator socket for observation

## Blockers for Full Agent Integration

| Surface | Status | Blocker |
|---------|--------|---------|
| `--pipe` | **IMPLEMENTED** | `PipeMode` exists with 6 tests |
| `--context` | **MISSING** | `agent_context.rs` not written |
| `--narrate` | **MISSING** | `narration.rs` not written |
| Journal | **MISSING** | `journal.rs` not written |
| `reflect>` prompt | **MISSING** | Not in `pipe.rs` |
| Lobby game selection | **MISSING** | Not in `pipe.rs` |
| Spectator relay | **MISSING** | `spectate.rs` not written |
| Spectator TUI | **MISSING** | `screens/spectate.rs` not written |

## Upstream Dependencies

| Dependency | Status | Impact |
|------------|--------|--------|
| `tui:shell` | **TRUSTED** (82 tests) | `GameRenderer`, `PipeMode`, `Events` |
| `games:traits` | **TRUSTED** (14 tests) | `CfrGame`, `Profile`, `GameType` |
| `play:tui` binary | **MISSING** | Blocks Slices 8–9 (spectator) |
| `robopoker` git dep | **BLOCKER** | Absolute path deps; must migrate to git |

## Decision from `agent-integration` Review

**Go/No-Go for agent implementation slices:**

| Slice | Files | Dependency | Decision |
|-------|-------|------------|----------|
| 1 | `agent_context.rs` | `tui:shell` | **GO** — can start immediately |
| 2 | `journal.rs` | `tui:shell` | **GO** — can start immediately |
| 3 | `--context` wiring | Slice 1 + `play:tui` binary | **WAIT** — binary not ready |
| 4 | `reflect>` prompt | Slice 3 | **WAIT** — depends on Slice 3 |
| 5 | `narration.rs` | `tui:shell` | **GO** — can start immediately |
| 6 | `--narrate` wiring | Slice 5 + `play:tui` binary | **WAIT** — binary not ready |
| 7 | Lobby game selection | `chain:runtime` (stubbed OK) | **WAIT** — depends on chain |
| 8 | SpectatorRelay | `play:tui` binary | **WAIT** — binary not ready |
| 9 | SpectateScreen | Slice 8 | **WAIT** — depends on Slice 8 |

**Recommendation**: Start implementation slices 1, 2, and 5 in parallel. These depend only on trusted `tui:shell` infrastructure. Slices 3, 4, 6, 7, 8, 9 must wait for `play:tui` binary or chain runtime.
