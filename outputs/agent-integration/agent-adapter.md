# Agent Adapter — `agent:experience` Integration Contract

## Purpose

This document records the integration contract between `agent:experience` and the rest of the product frontier. It captures what `agent:experience` provides, what it consumes, and how the lane wires into `myosu-play`'s CLI surface.

`agent:experience` is the **agent-facing presentation layer** for Myosu. It delivers all surfaces through which programmatic agents (LLMs, bots, scripts) perceive and act upon the game world.

---

## Integration Topology

```
myosu-product frontier
│
├── play:tui (reviewed KEEP)
│   └── produces: myosu-play binary
│       └── myosu-play --pipe --context <path> --narrate
│
└── agent:experience (reviewed KEEP)
    └── consumes: myosu-play --pipe surface
        │
        ├── upstream: tui:shell (82 tests, TRUSTED)
        ├── upstream: games:traits (14 tests, TRUSTED)
        ├── upstream: play:tui binary (in progress)
        └── provides: agent context + journal + narration + spectator relay
```

---

## What `agent:experience` Consumes

| Input | Source | Status |
|-------|--------|--------|
| `myosu-play` binary | `play:tui` lane | Binary skeleton missing; Slice 1 in progress |
| `GameRenderer` trait | `tui:shell` | TRUSTED (82 tests) |
| `PipeMode` driver | `tui:shell` | TRUSTED (6 tests pass) |
| `CfrGame`, `Profile`, `GameType` traits | `games:traits` | TRUSTED (14 tests) |
| `docs/api/game-state.json` schema | `agent:experience` | TRUSTED (complete) |
| `crates/myosu-tui/src/schema.rs` | `agent:experience` | TRUSTED (16 tests pass) |

---

## What `agent:experience` Produces

| Output | Path | Status |
|--------|------|--------|
| `AgentContext` struct + load/save | `crates/myosu-tui/src/agent_context.rs` | MISSING — Slice 1 |
| `Journal` append-only markdown | `crates/myosu-tui/src/journal.rs` | MISSING — Slice 2 |
| `--context` flag wiring | `crates/myosu-tui/src/pipe.rs` | MISSING — Slice 3 |
| `reflect>` prompt after hand | `crates/myosu-tui/src/pipe.rs` | MISSING — Slice 4 |
| `NarrationEngine` prose | `crates/myosu-tui/src/narration.rs` | MISSING — Slice 5 |
| `--narrate` flag wiring | `crates/myosu-tui/src/pipe.rs` | MISSING — Slice 6 |
| Lobby + game selection | `crates/myosu-tui/src/pipe.rs` | MISSING — Slice 7 |
| `SpectatorRelay` Unix socket | `crates/myosu-play/src/spectate.rs` | MISSING — Slice 8 |
| `SpectateScreen` | `crates/myosu-tui/src/screens/spectate.rs` | MISSING — Slice 9 |

---

## CLI Wiring Contract

`agent:experience` augments the `myosu-play` binary with these flags:

```
myosu-play --pipe [--context <path>] [--narrate]
myosu-play --spectate <session_id>
```

**Minimal viable flag set for Slice 1–4:**

| Flag | Slice | Owner |
|------|-------|-------|
| `--pipe` | existing | `tui:shell` |
| `--context <path>` | Slice 3 | `agent:experience` |
| `--narrate` | Slice 6 | `agent:experience` |

**Slice 7 lobby flag:**

| Flag | Slice | Owner |
|------|-------|-------|
| (no `--subnet` → lobby) | Slice 7 | `agent:experience` |

**Spectator flags:**

| Flag | Slice | Owner |
|------|-------|-------|
| `--spectate <session_id>` | Slice 8 | `agent:experience` |

---

## Upstream Integration Points

### `tui:shell` Integration (TRUSTED — 82 tests)

`agent:experience` builds on `PipeMode` in `crates/myosu-tui/src/pipe.rs`:

- `PipeMode` is constructed with a `Shell` reference and drives stdin/stdout for the pipe protocol
- `pipe_output()` renders `GameState` as terse key-value text for agent consumption
- `agent:experience` extends `PipeMode` with `--context` (agent context file) and `--narrate` (prose mode)
- The `reflect>` prompt is emitted by `PipeMode` after each `HAND COMPLETE` event

No changes to `tui:shell` internals are required. `agent:experience` is a pure extension of the existing pipe surface.

### `games:traits` Integration (TRUSTED — 14 tests)

`agent:experience` consumes `CfrGame`, `Profile`, `GameConfig`, and `GameType` from `myosu-games`:

- `GameState` JSON schema validates all game types
- `schema.rs` uses `GameType` enum to serialize game-specific state
- No trait impl changes required; `agent:experience` is a downstream consumer only

### `play:tui` Binary Integration (IN PROGRESS)

The `myosu-play` binary is the execution vehicle:

- `agent:experience` slices add CLI flags to `myosu-play`'s `main.rs` dispatch
- Slice 3 (`--context` wiring) requires `myosu-play` binary to exist
- Slice 8 (`SpectatorRelay`) requires `crates/myosu-play/src/spectate.rs`

**Integration risk**: `play:tui` Slice 1 (binary skeleton) must land before `agent:experience` Slice 3 can proceed.

### `robopoker` Dependency (BLOCKER — owned by `games:traits`)

Both `tui:shell` and `games:traits` depend on `robopoker` via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This blocks CI and clean checkouts.

- **Owner**: `games:traits` lane ( Slice 1 )
- **Impact on `agent:experience`**: All slices ultimately call into `games:traits` or `tui:shell`, which call into `robopoker`. Resolution required before Phase 1 integration testing.
- **Slices 1–2 are safe**: `agent_context.rs` and `journal.rs` use only `serde` + `std`; no robopoker calls.

---

## Slice Dependency Map

```
Phase 1 (Agent Identity — unblocked NOW):
  Slice 1: agent_context.rs     ──► (tui:shell only)     ──► UNBLOCKED
  Slice 2: journal.rs           ──► (tui:shell only)     ──► UNBLOCKED

Phase 2 (Narration + Pipe Mode):
  Slice 3: --context flag       ──► (play:tui binary)    ──► BLOCKED (play:tui Slice 1)
  Slice 4: reflect> prompt      ──► (play:tui binary)    ──► BLOCKED (play:tui Slice 1)
  Slice 5: narration.rs         ──► (tui:shell)          ──► UNBLOCKED
  Slice 6: --narrate flag       ──► (play:tui binary)     ──► BLOCKED (play:tui Slice 1)
  Slice 7: lobby               ──► (chain discovery)     ──► STUBBED FOR PHASE 0

Phase 3 (Spectator):
  Slice 8: SpectatorRelay      ──► (play:tui binary)    ──► BLOCKED (play:tui Slice 1)
  Slice 9: SpectateScreen       ──► (play:tui binary)    ──► BLOCKED (play:tui Slice 1)
```

---

## Session Persistence Contract

`AgentContext` persists across sessions via JSON file at `--context <path>`:

```json
{
  "identity": { "name": "...", "created": "...", "games_played": 0, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 0, "lifetime_result": "0bb", "observations": [] },
  "journal": []
}
```

`Journal` appends to `{context-dir}/journal.md` — append-only, never truncates.

The persistence boundary is clean: `agent:experience` does not share memory with `play:tui`. Agents and humans use separate context files.

---

## Spectator Relay Contract

Phase 0 (local Unix socket):

- Socket path: `~/.myosu/spectate/<session_id>.sock`
- Event format: valid JSON `GameEvent` lines (one per event)
- Fog-of-war: hole cards NEVER appear during active play; revealed only after `showdown` event
- Enforcement point: `SpectatorRelay` (not renderer)

Phase 1 (future): WebSocket upgrade via miner axon — blocked on `chain:runtime`.

---

## File Location Summary

| Artifact | Location |
|----------|----------|
| AgentContext | `crates/myosu-tui/src/agent_context.rs` |
| Journal | `crates/myosu-tui/src/journal.rs` |
| NarrationEngine | `crates/myosu-tui/src/narration.rs` |
| Pipe extensions | `crates/myosu-tui/src/pipe.rs` |
| SpectatorRelay | `crates/myosu-play/src/spectate.rs` |
| SpectateScreen | `crates/myosu-tui/src/screens/spectate.rs` |
| JSON schema | `docs/api/game-state.json` |
| Schema Rust impl | `crates/myosu-tui/src/schema.rs` |
