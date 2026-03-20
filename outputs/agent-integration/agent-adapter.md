# Agent Adapter — Integration Surface Map

## Purpose

This document maps `agent:experience` lane specifications onto actual codebase surfaces. It answers: **what exists today, what must be built, and what must happen first**.

This is not an implementation plan. It is the honest adapter layer between the product spec and the engineering reality.

---

## Current Codebase Surfaces

### `crates/myosu-tui/src/pipe.rs` — Pipe Mode Driver

**Status**: PARTIAL — skeleton exists, no flags, no persistence

The `PipeMode` struct is the stdin/stdout driver for agent piping. It owns:
- `output_state()` → calls `renderer.pipe_output()` and writes to stdout
- `read_input()` → reads one line from stdin
- `run_once()` → output then input in one call

**What exists**:
- No `--context` flag wiring
- No `--narrate` flag wiring
- No `reflect>` prompt after hand
- No lobby rendering
- No `AgentContext` integration
- No `Journal` integration

**Integration point for agent:experience**:
- `PipeMode` must gain `context_path: Option<PathBuf>` and `narrate: bool` fields
- After each hand completes, it must emit `HAND COMPLETE` block then call `read_input()` for reflection
- When no `--subnet` is provided, it must render lobby before game loop

```rust
// What needs to be added to PipeMode
pub struct PipeMode<'a> {
    renderer: &'a dyn GameRenderer,
    output: io::Stdout,
    context_path: Option<PathBuf>,   // NEW
    narrate: bool,                    // NEW
    agent_ctx: Option<AgentContext>,  // NEW
}
```

### `crates/myosu-tui/src/schema.rs` — JSON Game State Schema

**Status**: TRUSTED — fully implemented, 16 tests passing

`GameState`, `LegalAction`, `GamePhase`, and `GameStateBuilder` are complete for all 10 game types. This is the machine-readable contract for structured agents.

**Integration point for agent:experience**:
- `schema.rs` is already the foundation for `SpectatorRelay` events (AC-SP-01)
- `SpectatorRelay::emit()` should emit `GameState` JSON lines to its Unix socket
- No changes needed to schema itself

### `crates/myosu-tui/src/renderer.rs` — `GameRenderer` Trait

**Status**: TRUSTED — object-safe trait, 6 tests passing

The `GameRenderer` trait is the **only integration contract** between the shell and any game. It is object-safe (`Box<dyn GameRenderer>`).

**Integration point for agent:experience**:
- `pipe_output()` already exists and returns `String`
- `completions()` already exists for tab-completion in pipe mode
- `parse_input()` already exists for action parsing
- No trait changes needed for agent:experience Phase 1

### `crates/myosu-tui/src/` — Missing Modules

| Module | Status | Notes |
|--------|--------|-------|
| `agent_context.rs` | **MISSING** | `AgentContext` struct with load/save/journal |
| `narration.rs` | **MISSING** | `NarrationEngine` for `--narrate` prose |
| `journal.rs` | **MISSING** | Append-only markdown journal |
| `spectate.rs` | **MISSING** | `SpectatorRelay` + `SpectateScreen` |

### `crates/myosu-play/` — Binary Does Not Exist

**Status**: MISSING — no crate at this path

`myosu-play` is commented out of `Cargo.toml` members. All pipe-mode flags (`--pipe`, `--context`, `--narrate`, `--spectate`) require this binary's `main.rs` CLI dispatch.

**Integration point for agent:experience**:
- Slice 1 of `play:tui` creates this binary
- Slices 3, 6, 7, 8, 9 of `agent:experience` all modify `main.rs`
- **Hard dependency**: `play:tui` Slice 1 must complete before `agent:experience` Slices 3+

### `crates/myosu-games/src/traits.rs` — Game Traits

**Status**: TRUSTED — 14 tests passing

Provides `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response`. Used by both `myosu-tui` and the future `myosu-play`.

**Integration point for agent:experience**:
- `agent_context.rs` uses `GameType` for `preferred_game` field
- `journal.rs` uses `GameType` for hand entry metadata
- No changes needed to traits themselves

---

## What `agent:experience` Must Build (Summary)

| Slice | File to Create | Upstream Needed |
|-------|---------------|----------------|
| 1 | `crates/myosu-tui/src/agent_context.rs` | `games:traits` (GameType) |
| 2 | `crates/myosu-tui/src/journal.rs` | Slice 1 |
| 3 | `crates/myosu-tui/src/pipe.rs` (extend) | `play:tui` Slice 1 (binary) |
| 4 | `crates/myosu-tui/src/pipe.rs` (extend) | Slice 3 |
| 5 | `crates/myosu-tui/src/narration.rs` | Slice 1 |
| 6 | `crates/myosu-tui/src/pipe.rs` (extend) | Slice 5 + `play:tui` Slice 1 |
| 7 | `crates/myosu-tui/src/pipe.rs` (extend) | Slice 3 |
| 8 | `crates/myosu-play/src/spectate.rs` | `play:tui` Slice 1 |
| 9 | `crates/myosu-tui/src/screens/spectate.rs` | Slice 8 + `play:tui` Slice 1 |

---

## Dependency Chain

```
games:traits (robopoker git migration)
    │
    ├──► play:tui Slice 1 (myosu-play binary) ──────────────────────┐
    │                                                             │
    │         agent:experience                                     │
    │         ├── Slices 1-2 (agent_context, journal) ────────────┤
    │         ├── Slice 3 (--context wiring) ─────────────────────┤
    │         ├── Slice 4 (reflect> prompt) ───────────────────────┤
    │         ├── Slice 5 (narration.rs) ─────────────────────────┤
    │         ├── Slice 6 (--narrate wiring) ─────────────────────┤
    │         ├── Slice 7 (lobby) ─────────────────────────────────┤
    │         ├── Slice 8 (SpectatorRelay) ────────────────────────┤
    │         └── Slice 9 (SpectateScreen) ─────────────────────────┘
    │
    └──► chain:runtime (future, for Slice 7 lobby + Slice 8 WS)
```

**Key constraint**: Slices 1, 2, 5 are **unblocked right now** — they depend only on `games:traits` and `tui:shell`, both of which are trusted. Slices 3, 4, 6, 7, 8, 9 all require `play:tui` Slice 1 (binary skeleton) first.

---

## Honest State of Integration Readiness

| Question | Answer | Evidence |
|----------|--------|----------|
| Does `PipeMode` exist? | YES | `crates/myosu-tui/src/pipe.rs` |
| Does `GameRenderer::pipe_output()` exist? | YES | `crates/myosu-tui/src/renderer.rs:45` |
| Does `GameState` JSON schema exist? | YES | `crates/myosu-tui/src/schema.rs` (16 tests) |
| Does `myosu-play` binary exist? | NO | Not in `Cargo.toml` members |
| Does `agent_context.rs` exist? | NO | No such file |
| Does `narration.rs` exist? | NO | No such file |
| Does `journal.rs` exist? | NO | No such file |
| Does `SpectatorRelay` exist? | NO | No such file |
| Is `robopoker` a git dependency? | NO | Absolute path deps remain |
| Is `games:traits` fully tested? | YES | 14 tests pass |

---

## Critical Path Items (Blockers)

### 1. `robopoker` Git Migration (HIGH — owned by `games:traits`)

Both `myosu-tui` and `myosu-games` depend on robopoker via absolute filesystem paths. Until this is resolved to a git dependency, `cargo build` fails on any clean checkout or CI environment.

**Impact on agent:experience**: Slices 1-2 don't call robopoker directly (they only use `GameType` from `games:traits`). Slices 3+ will fail to test end-to-end until the dependency is resolved.

### 2. `myosu-play` Binary Skeleton (HIGH — owned by `play:tui`)

`play:tui` Slice 1 creates `crates/myosu-play/`. This binary is the vehicle for all CLI flags (`--pipe`, `--context`, `--narrate`, `--spectate`).

**Impact on agent:experience**: Slices 3, 4, 6, 7, 8, 9 cannot be wired until `main.rs` exists.

### 3. `games:traits` robopoker migration is the shared critical blocker

`agent:experience` and `play:tui` share this blocker. It is owned by `games:traits`. The review at `outputs/games/traits/review.md` tracks its resolution.

---

## Decision Framing for `review.md`

`agent:experience` is **READY** to proceed with Slices 1, 2, 5 immediately. These slices have no external blockers beyond already-trusted upstream.

`agent:experience` is **BLOCKED** on `play:tui` Slice 1 for Slices 3, 4, 6, 7, 8, 9.

The `robopoker` git migration (owned by `games:traits`) must complete before any integration testing that involves `myosu-play` calling robopoker APIs.
