# `agent-integration` Lane Specification

## Purpose and User-Visible Outcome

`agent-integration` is the **synthesis and go/no-go decision lane** for the `agent:experience` product lane. It reviews the `agent:experience` specification and review artifacts, evaluates upstream readiness, and produces the honest judgment on whether the product can proceed to an implementation family or requires another upstream unblock first.

The lane delivers:

1. **`agent-adapter.md`** — this document. The integration surface map: what `agent:experience` requires from upstream, what it provides to downstream, and where the adapter code must bridge the two.
2. **`review.md`** — the honest judgment: proceed to implementation family, or blocked on upstream.

**User-visible behavior**: No direct user-visible behavior. This lane produces control-plane artifacts only.

---

## Integration Surface Map

```
                            agent:experience
                     (outputs/agent/experience/spec.md)
                            ┌─────────────────────────────────────────────┐
                            │                                              │
upstream                    │  SURFACES OWNED BY agent:experience         │
tui:shell ────────────────►│  GameRenderer::pipe_output() contract       │
  (82 tests, trusted)       │  pipe.rs — PipeMode driver                 │
                            │  agent_context.rs (MISSING)                  │
upstream                    │  narration.rs (MISSING)                      │
games:traits ─────────────►│  journal.rs (MISSING)                        │
  (14 tests, trusted)       │  schema.rs (TRUSTED)                        │
                            │  docs/api/game-state.json (TRUSTED)         │
upstream                    │                                              │
play:tui ─────────────────►│  myosu-play binary skeleton (MISSING)         │
  (binary absent)          │  CLI dispatch for --pipe, --context,         │
                            │  --narrate, --spectate (MISSING)            │
                            │                                              │
                            │  SpectatorRelay (MISSING)                   │
                            │  SpectateScreen (MISSING)                   │
                            └─────────────────────────────────────────────┘
                                        │
                                        │ adapter wires
                                        ▼
                            ┌─────────────────────────────────────────────┐
                            │           product (myosu)                    │
                            │                                              │
                            │  crates/myosu-play/src/main.rs              │
                            │  crates/myosu-tui/src/pipe.rs              │
                            │  crates/myosu-tui/src/agent_context.rs     │
                            │  crates/myosu-tui/src/narration.rs         │
                            │  crates/myosu-tui/src/journal.rs           │
                            │  crates/myosu-tui/src/screens/spectate.rs  │
                            │                                              │
                            │  crates/myosu-games-poker/src/lib.rs        │
                            │  (NlheRenderer implements GameRenderer)     │
                            └─────────────────────────────────────────────┘
```

---

## Upstream Readiness Assessment

### `tui:shell` — TRUSTED

82 tests pass. `GameRenderer` trait, `PipeMode`, `Events`, `Theme` all exist and are stable. `agent:experience` Slices 1–4 can depend on this without risk.

### `games:traits` — TRUSTED

14 tests pass. `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` are stable. No changes expected before `agent:experience` Slice 5.

### `play:tui` — NOT TRUSTED (binary absent)

The `myosu-play` binary skeleton does not exist. `crates/myosu-play/` has no code. The `play:tui` lane spec defines the binary structure but Slice 1 (binary bootstrap) has not been implemented.

**Impact on `agent:experience`:**
- Slices 1–2 (`agent_context.rs`, `journal.rs`): **NOT BLOCKED** — depend only on `tui:shell`
- Slice 3 (`--context` flag wiring): **BLOCKED** — requires CLI dispatch in `main.rs`
- Slice 4 (`reflect>` prompt): **BLOCKED** — requires `pipe.rs` modification, which requires `main.rs` wiring
- Slice 5 (`narration.rs`): **NOT BLOCKED** — pure game-state-to-prose, no CLI needed
- Slice 6 (`--narrate` wiring): **BLOCKED** — requires CLI dispatch
- Slice 7 (lobby): **BLOCKED** — requires CLI + chain query (stubbed for Phase 0)
- Slices 8–9 (spectator): **BLOCKED** — require `myosu-play` binary + `play:tui` screen infrastructure

### `robopoker` — NOT TRUSTED (git migration unresolved)

`games:traits` and `tui:shell` both depend on robopoker via **absolute filesystem paths** (`/home/r/coding/robopoker/...`). This is documented in `outputs/games/traits/spec.md` as Blocker 1 (HIGH).

**Impact on `agent:experience`**: All slices ultimately call into robopoker through `games:traits` or `tui:shell`. Until robopoker is a proper git dependency, `cargo build` and `cargo test` will fail on any clean checkout or CI environment.

---

## Adapter Code Required

The adapter code is the wiring that connects `agent:experience` surfaces to the product. It lives in existing files that `agent:experience` does not own directly.

### 1. `crates/myosu-play/src/main.rs` — CLI Dispatch Bootstrap

**What**: Create the `myosu-play` binary with `--pipe`, `--context`, `--narrate`, `--spectate` flag dispatch.
**Status**: File does not exist.
**Owner**: `play:tui` lane (not `agent:experience`).
**What `agent:experience` needs**: A `PipeMode::new(context_path, narrate)` constructor that can be called from `main.rs`. The binary creates the `PipeMode` and runs the game loop.

### 2. `crates/myosu-tui/src/pipe.rs` — PipeMode Extensions

**What**: Extend `PipeMode` with:
- `context_path: Option<PathBuf>` field
- `narrate: bool` field
- `reflect>` prompt after each hand
- Lobby rendering when no `--subnet` is provided
**Status**: File exists, partially implemented (6 tests pass).
**What `agent:experience` needs**: The extended `PipeMode` that wires `--context` and `--narrate` flags.

### 3. `crates/myosu-tui/src/agent_context.rs` — NEW FILE

**What**: `AgentContext` struct with `load()`, `save()`, `default()`; serde JSON serialization; journal append.
**Status**: File does not exist.
**Owner**: `agent:experience` lane.
**What it enables**: Persistent agent identity and memory across sessions.

### 4. `crates/myosu-tui/src/narration.rs` — NEW FILE

**What**: `NarrationEngine::narrate(&GameState) -> String`; board texture analysis; session arc weaving.
**Status**: File does not exist.
**Owner**: `agent:experience` lane.
**What it enables**: Rich prose output for `--narrate` mode.

### 5. `crates/myosu-tui/src/journal.rs` — NEW FILE

**What**: `Journal` struct; append-only markdown writer; `append_hand_entry()`, `append_session_summary()`.
**Status**: File does not exist.
**Owner**: `agent:experience` lane.
**What it enables**: Append-only agent journal artifact.

### 6. `crates/myosu-play/src/spectate.rs` — NEW FILE

**What**: `SpectatorRelay`; Unix domain socket at `~/.myosu/spectate/<session_id>.sock`; fog-of-war at relay.
**Status**: File does not exist.
**Owner**: `agent:experience` lane.
**What it enables**: Phase 0 spectator relay.

### 7. `crates/myosu-tui/src/screens/spectate.rs` — NEW FILE

**What**: `SpectateScreen`; fog-of-war rendering; `r` key to reveal hole cards after showdown.
**Status**: File does not exist.
**Owner**: `agent:experience` lane.
**What it enables**: Spectator TUI screen.

---

## Integration Decision: What Must Happen First

### Critical Path to `agent:experience` Full Delivery

```
SLICES 1-2 (agent_context.rs, journal.rs)
  Can start: immediately
  Dependency: tui:shell (trusted)
  Blocker: NONE

SLICE 5 (narration.rs)
  Can start: immediately
  Dependency: games:traits (trusted), GameState type
  Blocker: NONE

SLICES 3-4 (--context wiring, reflect> prompt)
  Can start: only after play:tui Slice 1 (binary skeleton)
  Dependency: myosu-play/main.rs CLI dispatch
  Blocker: HIGH — binary absent

SLICES 6-7 (--narrate wiring, lobby)
  Can start: only after play:tui Slice 1 + Slice 5
  Dependency: myosu-play/main.rs + narration.rs
  Blocker: HIGH — binary absent

SLICES 8-9 (spectator relay + screen)
  Can start: only after play:tui Slice 1 + tui:shell screen infrastructure
  Dependency: myosu-play binary + Screen trait extension
  Blocker: HIGH — binary absent + screen infrastructure
```

### The Honest Answer

`agent:experience` is **not uniformly ready**. It has two independent workstreams:

- **Workstream A** (Slices 1–2, Slice 5): Can proceed immediately. No external blockers. `tui:shell` and `games:traits` are trusted. These slices produce `agent_context.rs`, `journal.rs`, and `narration.rs`.

- **Workstream B** (Slices 3–4, 6–9): Blocked on `play:tui` Slice 1 (binary skeleton). Cannot wire `--context`, `--narrate`, lobby, or spectator without the `myosu-play` binary existing.

The **robopoker git migration** (owned by `games:traits`) is a CI/blocker for all workstream testing but does not block Slice 1–2, 5 implementation.

---

## What the Implementation Family Needs from This Lane

When the implementation family is formed, it should receive:

1. **Split workstream assignment**: Workstream A (Slices 1–2, 5) can begin immediately. Workstream B (Slices 3–4, 6–9) waits on `play:tui` Slice 1.

2. **Adapter contract**: The `myosu-play` binary must expose a `PipeMode::new(context_path: Option<PathBuf>, narrate: bool)` constructor. This is the integration contract between `play:tui` and `agent:experience`.

3. **Test dependency**: All slices require `cargo test -p myosu-tui` to pass. This requires robopoker git migration to resolve first for CI. Local development can proceed with absolute path deps.

4. **Spectator socket convention**: `~/.myosu/spectate/<session_id>.sock` should be confirmed against `play:tui` data directory convention before Slice 8.

---

## Evidence This Lane Is Complete

This lane is **complete** when:
- `outputs/agent-integration/agent-adapter.md` exists and documents the integration surface accurately
- `outputs/agent-integration/review.md` exists and contains the honest go/no-go judgment
- The decision is communicated: Workstream A can proceed; Workstream B waits on `play:tui` Slice 1
