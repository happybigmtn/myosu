# Agent Integration — Adapter

## Purpose

This document maps the `agent:experience` lane surfaces to the Myosu system boundary.
It describes what the lane owns, what it depends on, and how the integration points
work. It is the integration contract that implementors must respect when building
on or adjacent to the agent-facing surfaces.

---

## Lane Boundary

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           agent:experience                                    │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ upstream (trusted)                                                     │  │
│  │   tui:shell (82 tests) ──► GameRenderer trait, PipeMode, Shell       │  │
│  │   games:traits (14 tests) ──► CfrGame, Profile, GameType,            │  │
│  │                               StrategyQuery                            │  │
│  │   spectator-protocol (spec only) ──► AX-01..05, SP-01..03           │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ surfaces owned by agent:experience                                      │  │
│  │   pipe mode (stdin/stdout protocol)                                    │  │
│  │   JSON schema (docs/api/game-state.json + schema.rs, 16 tests)        │  │
│  │   agent context (--context <path>, AgentContext, JSON file)          │  │
│  │   reflection channel (reflect> prompt, append-only journal)           │  │
│  │   narration engine (--narrate flag, atmospheric prose)                │  │
│  │   agent journal (append-only markdown, lifetime memory)               │  │
│  │   lobby (game selection when --pipe with no --subnet)                 │  │
│  │   spectator relay (Unix socket, Phase 0; WS upgrade Phase 1)         │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ downstream integration (future)                                        │  │
│  │   miner axon HTTP endpoint ──► lobby queries, game info               │  │
│  │   miner axon WebSocket ──► spectator relay Phase 1 upgrade           │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Integration Points

### 1. `tui:shell` (trusted upstream — hard dependency)

**What is used**: `GameRenderer` trait, `PipeMode` driver, `Shell`, `Events`, `Theme`

**How it is consumed**: All pipe mode output flows through `GameRenderer::pipe_output()`.
`PipeMode` in `crates/myosu-tui/src/pipe.rs` is the driver that wraps the game
renderer with stdin/stdout orchestration. The `agent:experience` lane extends
`PipeMode` with new flags (`--context`, `--narrate`) and new behaviors
(`reflect>` prompt, lobby rendering).

**Integration constraint**: The `GameRenderer::pipe_output()` contract must not
change in a way that breaks the terse key-value format that agents parse. Any
additions must be backward-compatible.

### 2. `games:traits` (trusted upstream — hard dependency)

**What is used**: `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery`,
`StrategyResponse`

**How it is consumed**: The `schema.rs` `GameState` type wraps `GameType` and
`Profile` outputs. `StrategyQuery` is the query interface used by the pipe mode
to ask the solver for action distributions. `GameConfig` configures game
parameters for the lobby.

**Integration constraint**: `CfrGame` and `Profile` must implement the required
traits for the schema serialization to work. The `StrategyQuery` response format
must match what `schema.rs` expects.

### 3. `play:tui` binary (trusted upstream — hard dependency for Slices 3+)

**What is used**: `myosu-play` binary dispatch in `crates/myosu-play/src/main.rs`

**How it is consumed**: All new flags (`--pipe`, `--context`, `--narrate`,
`--spectate`) must be added to the CLI dispatch in `main.rs` and passed to the
appropriate `PipeMode` constructor or screen manager. Slice 3 (--context wiring),
Slice 6 (--narrate wiring), Slice 7 (lobby), and Slice 9 (SpectateScreen) all
require `main.rs` modifications.

**Current state**: The binary skeleton exists but the new flags are not yet
wired. Slice 1 of `play:tui` must complete the binary dispatch before Slices
3+ of `agent:experience` can proceed.

### 4. `robopoker` fork (external — hard dependency, HIGH risk)

**What is used**: `Game`, `Recall`, `Action`, CFR traits

**How it is consumed**: Indirectly through `games:traits`. The `robopoker` crate
is currently referenced via **absolute filesystem paths** in `Cargo.toml` (e.g.,
`/home/r/coding/robopoker/crates/...`). This works locally but fails on clean
checkouts and CI.

**Resolution required**: Migrate `robopoker` to a proper `git = "https://..."`
Cargo dependency. This is owned by `games:traits` lane Slice 1. Until this
lands, `cargo build` and `cargo test` will fail on any machine that does not
have `robopoker` checked out at the hardcoded path.

**Impact on agent:experience**: All 9 slices ultimately call into
`games:traits` or `tui:shell`, which call into `robopoker`. Phase 1 (Slices
1–4) can begin with stubbed robopoker, but integration testing of Slices 5–9
requires the git dependency to be resolved.

### 5. `chain:runtime` (soft upstream — Phase 2 only)

**What is used**: Miner axon HTTP endpoint (for lobby game info), miner axon
WebSocket (for spectator relay Phase 1 upgrade)

**How it is consumed**: The lobby (Slice 7) queries active subnet information
from the miner axon. The spectator relay Phase 1 upgrade tunnels game events
over WebSocket via the miner axon.

**Current state**: Both are Phase 2 features. For Phase 0, the lobby is
stubbed with hardcoded data. The spectator relay uses a local Unix socket only.

### 6. `chain:runtime` (hard upstream — for `poker-engine` lane)

**Connection**: The `agent:experience` lane does not directly depend on
`chain:runtime`, but the `poker-engine` lane (which produces the solver that
agents query) does. Without `poker-engine`, the `StrategyQuery` responses that
power pipe mode are empty.

**Dependency chain**: `agent:experience` → `games:traits` → `poker-engine` →
`chain:runtime`. The lobby cannot show real exploitability scores until
`poker-engine` and `chain:runtime` are complete.

---

## Key Integration Concerns

### A. `robopoker` Git Migration (HIGH — blocks Phase 1+ integration testing)

The absolute path dependency means no CI, no clean checkout, no collaborator
repro. The `games:traits` lane owns this fix. Do not begin Phase 1 integration
testing on `agent:experience` until the git dependency is confirmed resolved.

### B. `myosu-play` Binary Dispatch (HIGH — blocks Slice 3+)

Slices 3, 6, 7, and 9 all require modifications to `main.rs` CLI dispatch.
The binary skeleton exists but new flags are not wired. The `play:tui` lane
must complete its Slice 1 (binary dispatch) before `agent:experience` can
proceed past Slice 2.

### C. Schema Trust Boundary

`schema.rs` (939 lines, 16 tests) is the most trusted surface in the lane and
is the event format for the spectator protocol. It must not be modified to
accommodate specific agent:experience features. If a feature requires a schema
change, that change must be reviewed against all other consumers (poker-engine,
miner, validator, spectator).

### D. Journal Append-Only Invariant

The journal (`journal.rs`) must never truncate. The implementation must open the
file in append mode and never rewrite. This invariant is what allows agents to
trust that their history is preserved. If the journal is ever overwritten, agent
memory is silently corrupted.

### E. Fog-of-War at the Relay

Hole cards must never appear in the spectator relay output during active play.
This is enforced at the relay (not at the renderer). The `SpectatorRelay` must
inspect each event before emitting and redact `hole_cards` fields unless the
event type is `showdown`.

### F. Pipe Mode Terse Format Contract

`pipe_output()` must maintain backward compatibility for any agent already
parsing the key-value format. New fields may be added, but existing field keys
and value formats must not change. If a breaking change is required, the lane
must emit a migration guide and a versioned format suffix (e.g., `pipe_v2`).

---

## File Map

| File | Lane owner | Integration consumer |
|------|-----------|---------------------|
| `crates/myosu-tui/src/schema.rs` | agent:experience | all lanes |
| `crates/myosu-tui/src/pipe.rs` | agent:experience | play:tui (CLI dispatch) |
| `crates/myosu-tui/src/agent_context.rs` | agent:experience | pipe.rs |
| `crates/myosu-tui/src/narration.rs` | agent:experience | pipe.rs |
| `crates/myosu-tui/src/journal.rs` | agent:experience | pipe.rs, agent_context.rs |
| `crates/myosu-play/src/spectate.rs` | agent:experience | pipe.rs |
| `crates/myosu-tui/src/screens/spectate.rs` | agent:experience | Shell |
| `crates/myosu-tui/src/pipe.rs` | tui:shell → agent:experience | main.rs |
| `crates/myosu-games/src/lib.rs` | games:traits | agent:experience |
| `crates/myosu-play/src/main.rs` | play:tui | agent:experience (CLI flags) |

---

## Phase Timeline

```
Phase 0 (current):
  Slices 1–2 (agent_context.rs, journal.rs)
  Depends on: tui:shell (trusted)
  Blockers: none

Phase 1:
  Slices 3–4 (--context wiring, reflect> prompt)
  Depends on: play:tui binary dispatch (Slice 1)
  Blockers: robopoker git migration (for integration testing)

Phase 2:
  Slices 5–7 (narration, --narrate wiring, lobby)
  Depends on: Phase 1 + poker-engine (for real lobby data)
  Blockers: robopoker git migration (hard)

Phase 3:
  Slices 8–9 (spectator relay, SpectateScreen)
  Depends on: play:tui binary + Phase 1
  Blockers: none beyond code authorship

Phase 4 (future):
  Lobby queries miner axon; spectator upgrades to WebSocket
  Depends on: chain:runtime + miner service
  Blockers: chain:runtime (far upstream)
```
