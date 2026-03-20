# `agent-integration` Artifact — `agent-adapter.md`

## Purpose

This document is the **concrete adapter specification** between the `agent:experience` lane's trusted surfaces and the rest of the myosu product. It captures the code-level integration points, the missing binary ownership, the flag-wiring contracts, and the path to making the pipe protocol real.

This is not a lane spec — it is the **integration glue** that the `agent:experience` lane review identified as the honest next step after the KEEP verdict.

---

## 1. What Exists and What Is Trusted

From `outputs/agent/experience/spec.md` and `outputs/agent/experience/review.md`:

| Surface | Status | Location |
|---------|--------|---------|
| `schema.rs` — `GameState`, `LegalAction`, `GamePhase`, builder | **TRUSTED** | `crates/myosu-tui/src/schema.rs` (939 lines, 16 tests) |
| `pipe.rs` — `PipeMode` driver skeleton | **TRUSTED** | `crates/myosu-tui/src/pipe.rs` (6 tests pass) |
| `GameRenderer::pipe_output()` contract | **TRUSTED** | `crates/myosu-tui/src/renderer.rs` |
| `docs/api/game-state.json` | **TRUSTED** | Complete JSON schema for 20 game types |
| `crates/myosu-tui/src/agent_context.rs` | **MISSING** | No file at this path |
| `crates/myosu-tui/src/narration.rs` | **MISSING** | No file at this path |
| `crates/myosu-tui/src/journal.rs` | **MISSING** | No file at this path |
| `crates/myosu-play/src/main.rs` | **MISSING** | No `myosu-play` crate at all |
| `SpectatorRelay` (AC-SP-01) | **MISSING** | No `spectate.rs` |
| `SpectateScreen` (AC-SP-02) | **MISSING** | No `screens/spectate.rs` |

**Summary**: The trusted surfaces are all abstract (trait + data schema). Every concrete surface that makes the lane real — the binary, the context file, the narration engine, the journal, the spectator relay — is completely absent.

---

## 2. The Missing `myosu-play` Binary — Root Integration Blocker

The `agent:experience` lane, `play:tui` lane, and `tui:shell` lane all converge on one missing artifact: **`crates/myosu-play/`**.

The workspace `Cargo.toml` confirms this explicitly:

```toml
# crates/myosu-play       # Stage 5: Gameplay CLI  ← commented out, not yet created
```

This is not a small gap. The `myosu-play` binary is the:
- **Vehicle for `--pipe`** — pipe mode has no CLI without it
- **Vehicle for `--context`** — context flag wiring requires `main.rs` dispatch
- **Vehicle for `--narrate`** — narration flag wiring requires `main.rs` dispatch
- **Vehicle for `--spectate`** — spectator relay client requires `main.rs` dispatch
- **Vehicle for lobby** — game selection in pipe mode requires `main.rs` dispatch

Every feature in the `agent:experience` lane spec except `schema.rs` requires `myosu-play` to exist first.

**The `myosu-play` binary is the single most critical integration dependency across all three lanes (`agent:experience`, `play:tui`, `tui:shell`).**

### 2.1 Minimum Viable `myosu-play` Skeleton for `agent:experience` Slice 1–4

Before Slices 1–4 of `agent:experience` can be integration-tested, the following binary skeleton is required:

```
crates/
  myosu-play/
    Cargo.toml        # new crate, workspace member
    src/
      main.rs         # CLI dispatch: --pipe, --train, --chain, --spectate
```

The `main.rs` does **not** need to be fully functional. It needs to:
1. Accept `--pipe` flag
2. Accept `--context <path>` flag
3. Accept `--narrate` flag
4. Wire these flags into a `PipeMode` struct that can be constructed and run

The actual game logic (loading `games:traits`, connecting to miners, etc.) can be stubbed. The **flag wiring is the critical path** for the `agent:experience` lane's integration testing.

### 2.2 Cargo Workspace Entry

Once `crates/myosu-play/` is created, add to workspace `Cargo.toml`:

```toml
members = [
    "crates/myosu-games",
    "crates/myosu-tui",
    "crates/myosu-chain/pallets/game-solver",
    "crates/myosu-play",   # uncomment / add
]
```

---

## 3. Adapter Contract: `PipeMode` → `AgentContext` → `Journal`

The `agent:experience` lane's core value is the **persistent agent session**. The adapter chain is:

```
myosu-play --pipe --context ./koan.json
       │
       ▼
PipeMode::new(renderer, context_path, narrate)
       │
       ├──► AgentContext::load(context_path)  ← Slice 1 + Slice 3
       │         │
       │         ├──► identity (name, created, games_played)
       │         ├──► memory (session_count, lifetime_result, observations)
       │         └──► journal entries (session, hand, reflection)
       │
       ├──► PipeMode::run_once() loop
       │         │
       │         ├──► GameRenderer::pipe_output() or NarrationEngine::narrate()  ← Slice 5 + Slice 6
       │         │
       │         ├──► read stdin → parse_input()
       │         │
       │         └──► HAND COMPLETE → reflect> prompt  ← Slice 4
       │                   │
       │                   └──► Journal::append_hand_entry()  ← Slice 2
       │
       └──► AgentContext::save() on drop
```

### 3.1 Concrete Flag Types (to be added to `myosu-play` CLI)

```rust
// In crates/myosu-play/src/main.rs (skeleton)

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Enable pipe mode (plain-text stdin/stdout protocol)
    #[arg(long)]
    pipe: bool,

    /// Path to agent context file (created if absent)
    #[arg(long, value_name = "PATH")]
    context: Option<std::path::PathBuf>,

    /// Enable narrated prose output instead of terse key-value
    #[arg(long)]
    narrate: bool,

    /// Subnet ID to connect to (omit for lobby)
    #[arg(long)]
    subnet: Option<u32>,

    /// Enable spectator relay client
    #[arg(long)]
    spectate: bool,
}
```

### 3.2 `PipeMode` Constructor Extension (required in `crates/myosu-tui/src/pipe.rs`)

Current signature:
```rust
pub fn new(renderer: &'a dyn GameRenderer) -> Self
```

Required extension for Slice 3:
```rust
pub fn with_context(
    renderer: &'a dyn GameRenderer,
    context_path: Option<&std::path::Path>,
    narrate: bool,
) -> Self
```

This constructor:
1. Loads `AgentContext` from `context_path` if provided and file exists
2. Creates default `AgentContext` if path is `None` or file is absent
3. Stores `narrate` flag for output mode decision
4. Saves context on `drop`

---

## 4. Blockers and Their Resolution Owners

### Blocker 1: `robopoker` Git Migration (HIGH — owned by `games:traits`)

**Problem**: Both `tui:shell` and `games:traits` depend on robopoker via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). Any clean checkout or CI environment fails to build.

**Resolution owner**: `games:traits` lane

**Impact on `agent:experience`**: All 9 slices ultimately call into `games:traits` → robopoker. Cannot proceed past Slice 4 without resolution.

**Resolution path**: Change `myosu-games/Cargo.toml` from:
```toml
[dependencies]
robopoker = { path = "/home/r/coding/robopoker/crates/robopoker" }
```
to:
```toml
[dependencies]
robopoker = { git = "https://github.com/happybigmtn/robopoker", branch = "main" }
```
This is tracked in `games:traits` lane as Slice 1.

### Blocker 2: `myosu-play` Binary Skeleton (HIGH — owned by `play:tui`)

**Problem**: The binary that hosts `--pipe`, `--context`, `--narrate`, and `--spectate` does not exist at all. No `crates/myosu-play/` crate exists in the workspace.

**Resolution owner**: `play:tui` lane (Slice 1)

**Impact on `agent:experience`**: Slices 3, 4, 6, 7, 8, 9 all require modifications to `myosu-play`'s `main.rs` CLI dispatch.

**Resolution path**:
1. Create `crates/myosu-play/Cargo.toml` with `myosu-tui` and `myosu-games` as dependencies
2. Create `crates/myosu-play/src/main.rs` with `clap` CLI and stub implementations
3. Add to workspace `Cargo.toml` members
4. Verify `cargo build -p myosu-play` compiles

---

## 5. The Two-Track Integration Decision

The `agent:experience/review.md` says "proceed to implementation-family workflow". There are two honest tracks:

### Track A: Implementation Family (recommended)

Treat the 9 `agent:experience` slices as the first implementation family, with `play:tui` Slice 1 (`myosu-play` binary) as a **parallel co-requisite**.

```
Parallel work:
  Thread 1: play:tui Slice 1 — myosu-play binary skeleton
  Thread 2: agent:experience Slice 1+2 — agent_context.rs + journal.rs

Sync point: myosu-play skeleton exists
  Thread 2: Slice 3 — --context wiring (needs myosu-play main.rs)

Post-sync:
  Slices 4, 5, 6, 7 → sequential
  Slices 8, 9 → sequential (spectator relay, spectator screen)
```

### Track B: Another Upstream Unblock First

Confirm `robopoker` git migration is done (owned by `games:traits`) **before** starting any implementation. This is the safer path but takes longer.

**Verdict from `agent:experience/review.md`**: Track A. The upstream `tui:shell` and `games:traits` are already trusted with passing tests. The `robopoker` path issue is a build-system concern, not a functional correctness concern — slices 1–4 can be developed and unit-tested using the existing absolute-path setup; only full integration testing needs the git migration.

---

## 6. Schema as the Stable Integration Contract

The most production-ready surface in the entire `agent:experience` lane is `schema.rs`. It is the **integration contract** between the game layer and the agent layer.

The `GameState` JSON schema (trusted, 16 tests passing) is what makes the following possible:
- Pipe mode text output (via `GameRenderer::pipe_output()`)
- Narration engine (wraps `GameState`, produces prose)
- Spectator relay (emits `GameEvent` JSON matching schema)
- Future HTTP/WS API (Phase 2, same schema)

The schema must **not** break. Any changes to `schema.rs` must be reviewed against the existing 16 tests.

---

## 7. Spectator Protocol Integration Point

AC-SP-01 specifies a Unix domain socket at `~/.myosu/spectate/<session_id>.sock`. This convention should be verified against `play:tui`'s data directory convention:

From `outputs/play/tui/spec.md`, the data directory convention uses `{data-dir}/hands/hand_{N}.json`. The spectator socket path (`~/.myosu/spectate/`) is a **different base path** — this needs to be reconciled before Slice 8.

**Resolution**: Confirm `play:tui` data directory base before Slice 8. Likely no change needed, but must be confirmed.

---

## 8. Concrete Next Steps for Integration

| Step | Owner | Action |
|------|-------|--------|
| 1 | `play:tui` | Create `crates/myosu-play/Cargo.toml` + `src/main.rs` skeleton with `--pipe --context --narrate` flags |
| 2 | `play:tui` | Add `crates/myosu-play` to workspace `Cargo.toml` members |
| 3 | `play:tui` | Verify `cargo build -p myosu-play` compiles with stub implementations |
| 4 | `agent:experience` | Create `crates/myosu-tui/src/agent_context.rs` (Slice 1) |
| 5 | `agent:experience` | Create `crates/myosu-tui/src/journal.rs` (Slice 2) |
| 6 | `agent:experience` | Wire `--context` flag into `PipeMode` (Slice 3) |
| 7 | `agent:experience` | Add `reflect>` prompt after `HAND COMPLETE` (Slice 4) |
| 8 | `games:traits` | Migrate robopoker from absolute path to git dependency (unblocks full integration testing) |
| 9 | `agent:experience` | Create `crates/myosu-tui/src/narration.rs` (Slice 5) |
| 10 | `agent:experience` | Wire `--narrate` flag into `PipeMode` (Slice 6) |
| 11 | `agent:experience` | Add lobby rendering in pipe mode (Slice 7) |
| 12 | `agent:experience` | Create `SpectatorRelay` in `crates/myosu-play/src/spectate.rs` (Slice 8) |
| 13 | `agent:experience` | Create `SpectateScreen` in `crates/myosu-tui/src/screens/spectate.rs` (Slice 9) |
