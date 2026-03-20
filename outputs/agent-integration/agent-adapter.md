# `agent:experience` Integration Adapter

## Purpose

This document describes how the `agent:experience` lane surface connects to the rest of the Myosu system — what it consumes from upstream lanes, what it provides to downstream consumers, and the integration contracts that must hold for the lane to function correctly.

`agent:experience` is the **agent-facing presentation layer** for Myosu. It owns every surface through which programmatic agents — LLMs, bots, scripts — perceive and act upon the game world.

---

## Integration Topology

```
upstream lanes
───────────────────────────────────────────────────────────────────

tui:shell (82 tests, TRUSTED)
  └─► GameRenderer trait (pipe_output, render_state, parse_input)
  └─► PipeMode driver (stdin/stdout loop)
  └─► Shell, ScreenManager, Events, Theme

games:traits (14 tests, TRUSTED)
  └─► CfrGame, Profile, GameConfig, GameType
  └─► StrategyQuery / StrategyResponse

robopoker (git dep UNRESOLVED)
  └─► Game, Recall, Action (absolute path dep)

───────────────────────────────────────────────────────────────────
                    agent:experience (THIS LANE)
                    ┌─────────────────────────────────────────────────────┐
                    │                                                     │
tui:shell ────────► │  GameRenderer ──► pipe_output()                  │
                    │  pipe.rs (PipeMode)                                │
                    │    ├─ context_path: Option<PathBuf>  [Slice 3]     │
                    │    ├─ narrate: bool              [Slice 6]        │
                    │    ├─ reflect> prompt             [Slice 4]        │
                    │    └─ lobby rendering             [Slice 7]        │
                    │                                                     │
                    │  schema.rs ──► GameState JSON (TRUSTED, 16 tests) │
                    │                                                     │
                    │  agent_context.rs  [Slice 1] (MISSING)              │
                    │    └─ AgentContext: identity, memory, journal       │
                    │                                                     │
                    │  narration.rs  [Slice 5] (MISSING)                 │
                    │    └─ NarrationEngine: board texture, session arc  │
                    │                                                     │
                    │  journal.rs  [Slice 2] (MISSING)                   │
                    │    └─ Journal: append-only markdown artifact        │
                    │                                                     │
                    │  spectate.rs  [Slice 8] (MISSING)                 │
                    │    └─ SpectatorRelay: Unix socket, fog-of-war      │
                    │                                                     │
                    │  screens/spectate.rs  [Slice 9] (MISSING)          │
                    │    └─ SpectateScreen: fog-of-war TUI               │
                    └─────────────────────────────────────────────────────┘
                              │
                              ▼
downstream consumers
───────────────────────────────────────────────────────────────────

myosu-play binary
  └─► exposes: --pipe --context <path> --narrate --spectate
  └─► requires: play:tui binary skeleton (play:tui lane, Slice 1)

agent pipelines (LLM agents, bots, scripts)
  └─► consumes: pipe mode stdout, JSON schema
  └─► produces: action lines on stdin

spectator clients (human observers)
  └─► consumes: Unix socket events at ~/.myosu/spectate/<session_id>.sock
  └─► fog-of-war enforced at relay (hole cards never during play)
```

---

## Upstream Contracts

### `tui:shell` — Hard Dependency (82 tests pass)

`agent:experience` builds on:

| Contract | Type | Status |
|----------|------|--------|
| `GameRenderer` trait | interface | **TRUSTED** |
| `GameRenderer::pipe_output() -> String` | method | **TRUSTED** |
| `GameRenderer::parse_input(&str) -> Option<String>` | method | **TRUSTED** |
| `PipeMode` struct | driver | **TRUSTED** (6 tests) |
| `PipeMode::output_state()` | method | **TRUSTED** |
| `PipeMode::read_input()` | method | **TRUSTED** |

These are stable. The lane does not need to renegotiate them.

### `games:traits` — Hard Dependency (14 tests pass)

`agent:experience` requires:

| Contract | Type | Status |
|----------|------|--------|
| `CfrGame` trait | interface | **TRUSTED** |
| `GameConfig` | struct | **TRUSTED** |
| `GameType` enum | type | **TRUSTED** |
| `StrategyQuery / StrategyResponse` | types | **TRUSTED** |

### `robopoker` — Hard Dependency (BLOCKED)

Both `tui:shell` and `games:traits` depend on `robopoker` via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This is documented as the highest-priority Slice 1 fix for `games:traits` and `play:tui`.

**Impact on this lane**: All 9 slices ultimately call into `games:traits` or `tui:shell`, which call into `robopoker`. Until `robopoker` is migrated to a proper git dependency, `cargo build` and `cargo test` fail on any clean checkout or CI environment.

**Resolution owner**: `games:traits` lane (owns the robopoker fork migration)

### `play:tui` — Hard Dependency for Slices 3+

The `myosu-play` binary is defined in `play:tui`'s spec and is the vehicle through which all `--pipe`, `--context`, `--narrate`, and `--spectate` flags are exposed. The binary skeleton does not exist yet.

| Slice | Dependency | Status |
|-------|------------|--------|
| Slice 1-2 | `tui:shell`, `games:traits` only | **READY** |
| Slice 3 (--context wiring) | `play:tui` binary skeleton | **MISSING** |
| Slice 4 (reflect prompt) | `play:tui` binary skeleton | **MISSING** |
| Slice 5-6 (narration) | `play:tui` binary skeleton | **MISSING** |
| Slice 7 (lobby) | `play:tui` binary skeleton | **MISSING** |
| Slice 8-9 (spectator) | `play:tui` binary skeleton | **MISSING** |

**Resolution owner**: `play:tui` lane (Slice 1: binary skeleton)

---

## Downstream Contracts

### `myosu-play` Binary Interface

After all slices land, the `myosu-play` binary exposes:

```
myosu-play --pipe [--context <path>] [--narrate]
myosu-play --spectate <session_id>
```

The pipe mode contract:
- **Input**: newline-delimited action lines on stdin (`fold`, `call`, `raise 20`, etc.)
- **Output**: `GameRenderer::pipe_output()` lines on stdout (terse key-value or narrated prose)
- **No ANSI codes** — verified by `is_plain_text()`
- **Flush after every write** — agents cannot block on buffered output

The JSON schema (`schema.rs`) provides the machine-readable alternative to pipe mode text. The same `GameState` type powers both.

### Agent Context File Contract

```
~/.myosu/agents/<agent-id>/context.json
```

Schema (from AC-AX-01):
```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [...] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

- Loaded on startup via `--context <path>`
- Saved on shutdown (drop)
- Missing file creates new default identity
- Never exposed to opponents — agent-private

### Journal Contract

```
~/.myosu/agents/<agent-id>/journal.md
```

- Append-only markdown — never truncates
- Each hand produces a markdown entry: board, held cards, result, optional reflection
- Empty `reflect>` response skips the reflection field

### Spectator Socket Contract

```
~/.myosu/spectate/<session_id>.sock
```

- Unix domain socket
- Emits valid JSON `GameEvent` lines (matching `schema.rs` types)
- **Fog-of-war enforced at relay**: hole cards never appear during active play
- Only revealed after `showdown` event
- Handles disconnected listeners gracefully

---

## Lane Slice Dependency Chain

```
Phase 1 (no external deps beyond tui:shell):
  Slice 1 (agent_context.rs)  ──────────────────────────────► Slice 2 (journal.rs)
        │                                                        │
        └──────────────── Slice 3 (--context wiring) ◄───────────┘
                                   │
                                   ▼
                         Slice 4 (reflect> prompt)
                                   │
                                   ▼
Phase 2 (requires Phase 1):
                         Slice 5 (narration.rs)
                                   │
                                   ▼
                         Slice 6 (--narrate wiring)
                                   │
                                   ▼
                         Slice 7 (lobby + game selection)

Phase 3 (requires play:tui binary):
                         Slice 8 (SpectatorRelay)
                                   │
                                   ▼
                         Slice 9 (SpectateScreen)
```

**Slice 1 and Slice 2 can begin immediately** — they depend only on `tui:shell` which is already trusted.

---

## Integration Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| robopoker absolute path deps block build | HIGH | `games:traits` lane owns resolution; must complete before Phase 1 integration testing |
| `play:tui` binary skeleton missing | HIGH | Blocks Slices 3-9; must complete before or concurrently with Slice 3 |
| Spectator socket path convention drift | LOW | `play:tui` uses `{data-dir}/hands/hand_{N}.json`; spectator uses `~/.myosu/spectate/` — different base paths, likely no conflict, but confirm before Slice 8 |
| Chain discovery stubbed for Phase 0 | MEDIUM | Lobby (Slice 7) will show hardcoded data until `chain:runtime` provides miner axon queries |

---

## What `agent:experience` Does NOT Own

The lane boundary explicitly excludes:

- **Agent-to-agent social interaction** — not in scope (AX-01)
- **Agent autonomy over system parameters** — not in scope (AX-01)
- **Emotion/affect modeling** — not in scope (AX-01)
- **HTTP/WS API endpoints** — Phase 2, blocked on `chain:runtime`
- **Miner axon WebSocket upgrade** — Phase 1 of spectator protocol, blocked on `chain:runtime`
- **Chain-connected lobby data** — Phase 0 uses hardcoded stubs; Phase 4 hooks to `chain:runtime`

---

## Schema as the Universal Contract

The most production-ready surface in this lane is `schema.rs` (939 lines, 16 tests passing, covers 10 game types). It is the **only artifact that is already trusted** and does not require additional slices to complete.

The `GameState` JSON schema is the integration point for:
- Pipe mode text rendering (via `GameRenderer::pipe_output()`)
- Future HTTP/WS API event payloads (Phase 2)
- Spectator relay event format (AC-SP-01)

Any new integration surface should consume `schema.rs` types first, not raw text.
