# `agent:experience` Integration Adapter

## Purpose

This document is the **integration contract** between the `agent:experience` product lane and its upstream providers. It maps what `agent:experience` needs, what each upstream provides, and the current integration status.

`agent:experience` is the agent-facing presentation layer for Myosu. It owns the surfaces through which programmatic agents perceive and act upon the game world: pipe mode, JSON schema, agent context, reflection channel, rich narration, and spectator relay.

---

## Upstream Provider Map

| Provider Lane | What `agent:experience` Consumes | Current Status |
|---------------|----------------------------------|----------------|
| `tui:shell` | `Shell`, `GameRenderer` trait, `PipeMode`, `Events`, `Theme` | **TRUSTED** — 82 tests pass |
| `games:traits` | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` | **TRUSTED** — 14 tests pass |
| `play:tui` | `myosu-play` binary skeleton; CLI dispatch for `--pipe`, `--context`, `--narrate`, `--spectate` flags | **MISSING** — binary scaffold not yet created |
| `spectator-protocol` | AX-01..05, SP-01..03 specs (specification only, not implemented) | **SPEC ONLY** |
| `robopoker` (external) | `Game`, `Recall`, `Action` — absolute path deps in `myosu-games` | **BLOCKED** — git migration unresolved in `games:traits` Slice 1 |

---

## What `agent:experience` Needs from Each Upstream

### From `tui:shell` (TRUSTED)

The entire pipe mode driver (`PipeMode`) and `GameRenderer` trait are consumed directly. `agent:experience` extends `PipeMode` with:
- `--context` flag: loads `AgentContext` from a JSON file
- `--narrate` flag: uses `NarrationEngine` instead of `pipe_output()`
- `reflect>` prompt: appends to `Journal` after each hand
- Lobby mode: renders game selection when no `--subnet` is provided

**Integration point**: `crates/myosu-tui/src/pipe.rs` — `agent:experience` modifies this file to add the above features.

**Contract**: `GameRenderer::pipe_output()` and `GameRenderer::render_state()` are frozen. Any trait changes require coordinated migration.

### From `games:traits` (TRUSTED)

`agent:experience` uses `GameType`, `GameConfig`, and the cfr trait re-exports for game state validation in pipe mode.

**Integration point**: `crates/myosu-games/src/traits.rs` — thin re-export surface consumed transitively through `myosu-tui`.

**Contract**: The re-export surface is thin. `agent:experience` does not call robopoker directly; it goes through `games:traits`.

### From `play:tui` (MISSING — BLOCKING)

`myosu-play` is the binary that exposes all CLI flags. Without it:
- Slices 3, 6, 7, 8, 9 cannot be wired to CLI
- The `spectate` subcommand cannot be registered
- The `--context` and `--narrate` flags have no dispatch target

**Owned by**: `play:tui` lane, Slice 1 (binary skeleton)

**Impact**: Slices 3–9 of `agent:experience` are blocked until `myosu-play` exists.

### From `robopoker` (BLOCKED)

Both `tui:shell` and `games:traits` depend on robopoker via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This blocks any `cargo build` or `cargo test` on clean checkout or CI.

**Owned by**: `games:traits` lane, Slice 1 (robopoker git migration)

**Impact**: All 9 slices ultimately call into `games:traits` or `tui:shell`, which call into robopoker. Until robopoker is migrated to a proper git dependency, full integration testing is impossible.

---

## Slice Dependency Analysis

| Slice | Module | Hard Upstream Dependency | Can Proceed? |
|-------|--------|-------------------------|--------------|
| 1 | `agent_context.rs` | `tui:shell` (trusted) | **YES** |
| 2 | `journal.rs` | `tui:shell` (trusted) | **YES** |
| 3 | `--context` flag wiring | `myosu-play` binary (missing) | NO |
| 4 | `reflect>` prompt | `pipe.rs` modification | NO (blocked by Slice 3) |
| 5 | `narration.rs` | `games:traits` (trusted) | **YES** (independent) |
| 6 | `--narrate` flag wiring | `myosu-play` binary (missing) | NO |
| 7 | Lobby + game selection | `myosu-play` binary (missing) + chain (stubbable) | NO |
| 8 | `SpectatorRelay` | `myosu-play` binary (missing) | NO |
| 9 | `SpectateScreen` | `tui:shell` screens (trusted) | NO (blocked by Slice 8) |

**Conclusion**: Slices 1, 2, and 5 can proceed immediately. Slices 3–4, 6–9 are blocked on `play:tui` Slice 1.

---

## Integration Risks

### Risk 1: Robopoker Absolute Path Coupling (CRITICAL — owned by `games:traits`)

**Location**: `crates/myosu-games/Cargo.toml` lines 16–17

All `agent:experience` implementation slices ultimately touch `games:traits` or `tui:shell`, both of which depend on robopoker via absolute paths. Until `games:traits` Slice 1 completes the git migration, no `cargo build` or `cargo test` will succeed on CI or clean checkout.

**Mitigation**: Slices 1, 2, 5 can proceed with local robopoker path. Full integration testing requires `games:traits` Slice 1.

### Risk 2: `myosu-play` Binary Missing (HIGH — owned by `play:tui`)

**Location**: `crates/myosu-play/` does not exist

The binary skeleton is the dispatch target for all CLI flags. Without it, Slices 3–9 cannot be wired.

**Mitigation**: Slices 1, 2, 5 can be implemented and tested in isolation from the binary. The implementation should be written as library modules in `myosu-tui` that the binary will call later.

### Risk 3: `GameRenderer` Trait Frozen (MEDIUM)

**Location**: `crates/myosu-tui/src/renderer.rs`

`agent:experience` extends `PipeMode` by calling `GameRenderer` trait methods. If the trait changes (e.g., a new required method), `PipeMode` and all renderers must be updated simultaneously.

**Mitigation**: Treat `GameRenderer` as frozen for Phase 1. Any trait changes require coordinated `tui:shell` + `agent:experience` migration.

### Risk 4: Spectator Socket Path Convention Drift (LOW)

**Location**: Not yet agreed

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock` — but `play:tui`'s data directory convention uses `{data-dir}/hands/hand_{N}.json`. These must align before Slice 8.

**Mitigation**: Verify `play:tui` data directory convention before Slice 8. Likely no change needed, but must be confirmed.

---

## Phase 0 Honest Slice (Can Proceed Now)

Given upstream status:
- `tui:shell`: **TRUSTED** (82 tests)
- `games:traits`: **TRUSTED** (14 tests)
- `play:tui`: **MISSING** binary

**Slices that can honestly proceed without any upstream unblock**:

1. **Slice 1**: `agent_context.rs` — depends only on `tui:shell` (trusted)
2. **Slice 2**: `journal.rs` — depends only on `tui:shell` (trusted)
3. **Slice 5**: `narration.rs` — depends only on `games:traits` (trusted)

These three slices produce:
- `crates/myosu-tui/src/agent_context.rs` — `AgentContext` struct with load/save/default
- `crates/myosu-tui/src/journal.rs` — append-only markdown journal writer
- `crates/myosu-tui/src/narration.rs` — `NarrationEngine` for atmospheric prose

**Proof gate** (all runnable without `myosu-play` binary):
```bash
cargo test -p myosu-tui agent_context::tests
cargo test -p myosu-tui journal::tests
cargo test -p myosu-tui narration::tests
```

---

## What Requires Upstream Unblock

| Slice | Blocker | Owner | Resolution |
|-------|---------|-------|------------|
| 3 | `myosu-play` binary | `play:tui` Slice 1 | Create binary skeleton first |
| 4 | `pipe.rs` extend | Slice 3 | Wire `--context` first |
| 6 | `myosu-play` binary | `play:tui` Slice 1 | Wire `--narrate` after binary |
| 7 | `myosu-play` binary + chain | `play:tui` Slice 1 + `chain:runtime` (future) | Stub lobby data for Phase 0 |
| 8 | `myosu-play` binary | `play:tui` Slice 1 | `SpectatorRelay` in binary crate |
| 9 | `SpectateScreen` | Slice 8 | Screen in `myosu-tui` |

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/agent/experience/spec.md` | `agent:experience` lane spec (9 slices defined) |
| `outputs/agent/experience/review.md` | `agent:experience` lane review (judgment: KEEP → implementation family) |
| `outputs/tui/shell/spec.md` | `tui:shell` lane spec (upstream, trusted) |
| `outputs/tui/shell/review.md` | `tui:shell` lane review (upstream, trusted) |
| `outputs/games/traits/spec.md` | `games:traits` lane spec (upstream, trusted) |
| `outputs/games/traits/review.md` | `games:traits` lane review (robopoker migration Risk 1, Slice 1) |
| `outputs/play/tui/spec.md` | `play:tui` lane spec (binary scaffold missing) |
| `outputs/play/tui/review.md` | `play:tui` lane review (judgment: KEEP → implementation family) |
| `crates/myosu-tui/src/pipe.rs` | Pipe mode driver — `agent:experience` extends this |
| `crates/myosu-tui/src/renderer.rs` | `GameRenderer` trait — frozen contract |
| `crates/myosu-games/src/traits.rs` | `games:traits` re-exports — trusted upstream |
| `crates/myosu-games/Cargo.toml` | Robopoker absolute path deps — `games:traits` Slice 1 must fix |
