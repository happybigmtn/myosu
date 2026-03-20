# `agent:experience` Integration Adapter

## Purpose

This document is the **integration contract** between the `agent:experience` lane and the rest of the Myosu product. It defines how `agent:experience` surfaces attach to upstream crates (`tui:shell`, `games:traits`, `play:tui`) and what must be true for the lane to produce working code.

---

## Integration Surfaces

### 1. JSON Schema (`schema.rs` → `agent:experience`)

**Direction**: `schema.rs` (trusted, 939 lines, 16 tests) is consumed by `agent:experience`.

`schema.rs` defines `GameState`, `LegalAction`, and `GamePhase` — the canonical machine-readable game state format. All pipe mode output and the eventual WebSocket event stream use this schema.

**Integration point**: `agent:experience` imports `myosu_tui::schema::{GameState, LegalAction, GamePhase}`. No adapter needed — the schema is already a standalone, well-typed surface.

**Constraint**: `GameState` uses `serde_json::Value` for game-specific state (the `state_wrapper` flattened field). Consumers must handle arbitrary JSON in the `state` field. The exhaustive `legal_actions` array is the stable contract.

---

### 2. `GameRenderer` Trait → `PipeMode`

**Direction**: `tui:shell` (`GameRenderer` trait, 82 tests) is the rendering contract consumed by `PipeMode`.

`GameRenderer` defines:
- `pipe_output() -> String` — terse key-value text for agent consumption
- `parse_input(&str) -> Option<String>` — agent input parsing
- `completions() -> Vec<String>` — available actions

**Integration point**: `PipeMode` holds `&dyn GameRenderer` and calls `pipe_output()` on state changes. The `GameRenderer` trait is object-safe via `dyn` (no `Sized` or `Copy` constraints in the trait signature itself).

**Constraint**: `PipeMode::run_once()` outputs state then reads stdin. The caller is responsible for game state advancement. This is intentional — `PipeMode` is a driver, not a game loop.

---

### 3. `games:traits` → `agent:experience`

**Direction**: `games:traits` (`CfrGame`, `Profile`, `GameConfig`, `GameType`, 14 tests) provides game logic consumed indirectly via the renderer.

The data flow:
```
robopoker (game logic)
    ↓ (implements CfrGame, Profile traits)
games:traits (trait layer)
    ↓ (concrete types implement GameRenderer)
tui:shell (GameRenderer trait + Shell)
    ↓ (PipeMode consumes dyn GameRenderer)
agent:experience (pipe/narrate/journal surfaces)
```

**Integration point**: `agent:experience` does not call `games:traits` directly. It receives rendered output from types that implement `GameRenderer`, which in turn delegate to `games:traits` types. The integration is indirect but the contract is stable.

**Constraint**: The `Profile::exploitability()` method is used by validators (not directly by `agent:experience`). Agent context uses `GameState` for memory — no direct `Profile` access needed.

---

### 4. `myosu-play` Binary → `agent:experience` Flags

**Direction**: `play:tui` owns the `myosu-play` binary; `agent:experience` adds flags to it.

Current `myosu-play` CLI (from `pipe.rs` + `shell.rs`):
```
myosu-play [--pipe] [--subnet <id>]
```

Required additions for `agent:experience`:
```
myosu-play --pipe [--context <path>] [--narrate] [--subnet <id>]
```

| Flag | Purpose | Owner |
|------|---------|-------|
| `--pipe` | Enable pipe mode (already exists) | `play:tui` |
| `--context <path>` | Load/save agent context JSON | `agent:experience` |
| `--narrate` | Atmospheric prose instead of terse KV | `agent:experience` |
| `--subnet <id>` | Select game subnet | `play:tui` (extend) |

**Integration point**: `agent:experience` Slice 3 wires `--context` flag to `PipeMode` initialization; Slice 6 wires `--narrate`. Both require `play:tui` Slice 1 (binary skeleton) to exist first.

**Dependency chain**:
```
play:tui Slice 1 (binary skeleton)
    ↓
agent:experience Slice 3 (--context wiring)
    ↓
agent:experience Slice 4 (reflect> prompt)
    ↓
agent:experience Slice 6 (--narrate wiring)
```

---

### 5. Spectator Relay Socket Convention

**Direction**: `SpectatorRelay` creates a Unix socket; spectator clients connect to it.

AC-SP-01 specifies: `~/.myosu/spectate/<session_id>.sock`

`play:tui` uses: `{data-dir}/hands/hand_{N}.json` for hand history.

**Assessment**: The spectator socket path and hand history path are different concerns (event stream vs. file storage). No conflict. The `~/.myosu/` base convention should be checked against `play:tui` data directory configuration before Slice 8 implementation, but no conflict is expected.

---

## Adapter Module Sketch

The integration does not require a new adapter crate. The surfaces are already typed and the integration points are at the CLI flag wiring level.

If a future refactor introduces an adapter layer, it would live at:
```
crates/myosu-tui/src/agent_adapter.rs
```

Its responsibilities:
1. Initialize `AgentContext` from `--context` path
2. Initialize `Journal` and wire to `AgentContext`
3. Initialize `NarrationEngine` when `--narrate` is set
4. Initialize `SpectatorRelay` for `--spectate` mode
5. Shutdown sequence: save context + journal

This module does not exist yet. It is an optional future refactor — the slices can proceed without it by wiring flags directly in `pipe.rs` and `main.rs`.

---

## Blockers That Must Be Resolved Before Testing

| Blocker | Owner | Impact | Status |
|---------|-------|--------|--------|
| `robopoker` git migration | `games:traits` | All slices call into `games:traits` which calls into `robopoker` via absolute paths. Builds fail on clean checkout. | Unresolved |
| `myosu-play` binary skeleton | `play:tui` | Slices 3, 6, 7 require CLI flag wiring. No binary to wire into without Slice 1. | Unresolved |

These are **upstream unblocks**, not integration issues. The `agent:experience` integration is clean. The lane can proceed to implementation once upstream delivers.

---

## What `agent-integration` Assesses vs. `agent:experience`

| Concern | `agent:experience` | `agent:integration` |
|---------|-------------------|---------------------|
| Lane contract | Specified (9 slices) | Assessed (clean) |
| Upstream dependencies | Identified | Confirmed via review artifacts |
| Integration surfaces | Noted | Detailed above |
| Blocker ownership | Identified | Assigned to upstream lanes |
| Go/no-go decision | KEEP | **Proceed** (integration is not the constraint) |

---

## Conclusion

The `agent:experience` integration is straightforward. The surfaces are typed, the contracts are stable, and the dependency chain is clean. The two blockers (`robopoker` git migration and `play:tui` binary skeleton) are **upstream owned** and must be resolved before integration testing, but they do not block specification or slice-level implementation.

**Recommendation**: Proceed to implementation-family workflow. The integration surfaces are sound. Upstream unblocks should be tracked as dependencies, not as reasons to pause.
