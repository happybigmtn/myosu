# `agent:integration` Lane Specification

## Purpose and User-Visible Outcome

`agent:integration` is the **integration adapter lane** that bridges the `agent:experience` product specification to implementation. It owns:

1. The integration surface between `agent:experience` surfaces (`--pipe`, `--context`, `--narrate`, journal, reflection, lobby, spectator relay) and the underlying `myosu-tui` crate
2. The wiring decisions for how the agent-facing flags connect to existing `PipeMode`, `Shell`, and `GameRenderer` infrastructure
3. The honest assessment of which implementation slices can proceed now versus which are blocked
4. The adapter types that must be introduced to connect product concepts to implementation modules

**User-visible behavior after full implementation**: An agent runs `myosu-play --pipe --context ./koan.json --narrate` and receives narrated game state, can write reflections after each hand, accumulates a persistent journal, and watches play via the spectator relay. The system behaves identically for agents and humans.

---

## Lane Boundary

```
                         agent:integration (THIS LANE)
                         ┌──────────────────────────────────────────────────────────────┐
                         │                                                               │
upstream                 │  Integration Surface                                       │
agent:experience ───────►│  ┌─────────────────────────────────────────────────────┐  │
(spec.md, review.md)    │  │ adapter.rs (NEW)                                      │  │
                         │  │   - AgentContextHandle (thin wrapper around context)   │  │
                         │  │   - JournalHandle (journal writer interface)          │  │
                         │  │   - NarrationHandle (engine dispatcher)              │  │
                         │  │   - SpectatorHandle (relay connector)                │  │
                         │  └─────────────────────────────────────────────────────┘  │
                         │  ┌─────────────────────────────────────────────────────┐  │
                         │  │ pipe_integration.rs (EXTEND)                        │  │
                         │  │   - --context flag wiring                           │  │
                         │  │   - --narrate flag wiring                           │  │
                         │  │   - reflect> prompt integration                     │  │
                         │  │   - lobby mode integration                          │  │
                         │  └─────────────────────────────────────────────────────┘  │
                         │  ┌─────────────────────────────────────────────────────┐  │
                         │  │ main.rs integration (EXTEND)                        │  │
                         │  │   - CLI flag registration                           │  │
                         │  │   - PipeMode initialization with adapter handles     │  │
                         │  └─────────────────────────────────────────────────────┘  │
                         │                                                               │
downstream               │  Trusted downstream                                         │
myosu-tui ──────────────│  Shell, GameRenderer, PipeMode, Events (all trusted)       │
(trusted)                │                                                               │
                         │  Untrusted downstream                                       │
spectator-protocol ──────│  SpectatorRelay (UNIX socket, future WS upgrade)          │
(spec only)              │                                                               │
                         └──────────────────────────────────────────────────────────────┘
```

**What this lane does NOT own:**
- The `agent:experience` product specification (that lives in `outputs/agent/experience/spec.md`)
- The `agent_context.rs`, `journal.rs`, `narration.rs` implementations (those are `agent:experience` implementation slices)
- The `myosu-play` binary skeleton (owned by `play:tui` lane)
- The `robopoker` git migration (owned by `games:traits` lane)
- The `SpectatorRelay` socket implementation (belongs to `agent:experience` Slice 8)

---

## Current Integration Surface Status

| Integration Point | Status | Location | Blocker |
|-------------------|--------|----------|---------|
| `PipeMode` with `--pipe` | **TRUSTED** | `crates/myosu-tui/src/pipe.rs` | None — works as-is |
| `GameRenderer::pipe_output()` | **TRUSTED** | `crates/myosu-tui/src/renderer.rs` | None |
| `Schema` (GameState JSON) | **TRUSTED** | `crates/myosu-tui/src/schema.rs` | None — 16 tests pass |
| `--context` flag wiring | **MISSING** | Not in `pipe.rs` or `main.rs` | Needs `myosu-play` binary |
| `--narrate` flag wiring | **MISSING** | Not in `pipe.rs` or `main.rs` | Needs `myosu-play` binary |
| `reflect>` prompt after hand | **MISSING** | Not in `pipe.rs` | Needs `myosu-play` binary |
| Lobby mode in pipe | **MISSING** | Not in `pipe.rs` | Needs `myosu-play` binary |
| `SpectatorRelay` UNIX socket | **MISSING** | No `spectate.rs` | Phase 0 not started |
| `AgentContext` struct | **MISSING** | No `agent_context.rs` | Slice 1 of 9 |
| `Journal` struct | **MISSING** | No `journal.rs` | Slice 2 of 9 |
| `NarrationEngine` | **MISSING** | No `narration.rs` | Slice 5 of 9 |

---

## Integration Architecture

### 1. Adapter Handle Pattern

Each product concept (context, journal, narration, spectator) gets a **handle** — a thin wrapper that provides a stable interface to the product layer while delegating to the implementation:

```rust
// crates/myosu-tui/src/adapter.rs (NEW)

/// Thin wrapper providing AgentContext access to PipeMode.
/// Does not own serialization — delegates to agent_context.rs (Slice 1).
pub struct AgentContextHandle {
    path: Option<PathBuf>,
    inner: Option<AgentContext>,
}

impl AgentContextHandle {
    /// Load context from path, or create default if None.
    pub fn load(path: Option<PathBuf>) -> Result<Self> { ... }

    /// Save context to disk. Called on drop or explicit flush.
    pub fn save(&self) -> Result<()> { ... }

    /// Access the inner context for reads.
    pub fn get(&self) -> Option<&AgentContext> { ... }

    /// Access the inner context for writes.
    pub fn get_mut(&mut self) -> Option<&mut AgentContext> { ... }
}

/// Thin wrapper providing journal write access.
/// Appends markdown entries; never truncates.
pub struct JournalHandle {
    path: PathBuf,
    file: RefCell<OpenOptions>,
}

impl JournalHandle {
    /// Open or create journal at path.
    pub fn open(path: PathBuf) -> Result<Self> { ... }

    /// Append a hand entry with board, held, result, optional reflection.
    pub fn append_hand_entry(&self, entry: &HandEntry) -> Result<()> { ... }

    /// Append a session summary.
    pub fn append_session_summary(&self, summary: &SessionSummary) -> Result<()> { ... }
}

/// Thin wrapper dispatching to NarrationEngine when --narrate is set.
pub struct NarrationHandle {
    engine: NarrationEngine,
    enabled: bool,
}

impl NarrationHandle {
    pub fn new(enabled: bool) -> Self { ... }

    /// Narrate game state as prose. Returns pipe_output() if disabled.
    pub fn narrate(&self, state: &GameState) -> String { ... }
}

/// Thin wrapper connecting to SpectatorRelay UNIX socket.
pub struct SpectatorHandle {
    session_id: Uuid,
    socket_path: PathBuf,
}

impl SpectatorHandle {
    /// Connect to relay socket for session.
    pub fn connect(session_id: Uuid) -> Result<Self> { ... }

    /// Emit a game event to relay.
    pub fn emit(&self, event: &GameEvent) -> Result<()> { ... }
}
```

### 2. PipeMode Integration Points

The `PipeMode` in `pipe.rs` must be extended with adapter handle fields:

```rust
// crates/myosu-tui/src/pipe.rs (EXTEND)

pub struct PipeMode {
    // ... existing fields ...
    context_handle: Option<AgentContextHandle>,   // NEW
    journal_handle: Option<JournalHandle>,         // NEW
    narration_handle: NarrationHandle,             // NEW
    spectator_handle: Option<SpectatorHandle>,     // NEW
    reflect_enabled: bool,                          // NEW
}

impl PipeMode {
    /// Initialize with --context and --narrate flags.
    pub fn new(
        context_path: Option<PathBuf>,
        narrate: bool,
        reflect: bool,
        spectate_session: Option<Uuid>,
    ) -> Result<Self> { ... }

    /// Handle reflect> prompt after hand complete.
    fn prompt_reflection(&mut self) -> Result<Option<String>> { ... }

    /// Render state — delegates to narration if enabled.
    fn render_state(&self, state: &GameState) -> String {
        if self.narration_handle.enabled {
            self.narration_handle.narrate(state)
        } else {
            self.pipe_output(state)
        }
    }
}
```

### 3. CLI Flag Wiring

The `myosu-play` `main.rs` must register and pass through the agent flags:

```rust
// crates/myosu-play/src/main.rs (EXTEND)

#[derive(Parser)]
enum PlayMode {
    /// Local training mode with blueprint bot
    Train(TrainArgs),
    /// Chain-connected mode with miner axon
    Chain(ChainArgs),
    /// Pipe mode for agent integration
    Pipe(PipeArgs),
}

#[derive(Parser)]
struct PipeArgs {
    /// Agent context file for persistent identity
    #[arg(long, value_name = "PATH")]
    context: Option<PathBuf>,

    /// Enable rich narration mode
    #[arg(long)]
    narrate: bool,

    /// Enable reflection prompt after each hand
    #[arg(long, default_value = "true")]
    reflect: bool,

    /// Spectator session ID for watching play
    #[arg(long, value_name = "UUID")]
    spectate: Option<Uuid>,

    /// Subnet to play on (omit for lobby)
    #[arg(long)]
    subnet: Option<u16>,
}
```

---

## Critical Integration Decisions

### Decision 1: Adapter Handles Own Serialization Boundaries

The adapter handles (`AgentContextHandle`, `JournalHandle`) **do not** implement serialization themselves. They delegate to the domain types:
- `AgentContextHandle` delegates to `agent_context.rs::AgentContext` (Slice 1)
- `JournalHandle` delegates to `journal.rs::Journal` (Slice 2)

**Rationale**: Keeps the adapter layer thin. Serialization logic belongs in domain types, not integration plumbing.

### Decision 2: NarrationEngine Is Instantiated but Not Wired in Slice 1

The `NarrationHandle` wraps `NarrationEngine`, which is implemented in Slice 5. For Slices 1–4, `NarrationHandle::narrate()` returns `pipe_output()` unconditionally.

**Rationale**: Allows `--narrate` flag to be wired in Slice 3 without requiring `NarrationEngine` to exist. The flag is a no-op until Slice 5 lands.

### Decision 3: SpectatorHandle Is Fire-and-Forget

`SpectatorHandle::emit()` returns `Result<()>` but does not block on listener connection. If no spectator is connected, events are dropped silently.

**Rationale**: Spectator is optional monitoring, not a required part of the agent experience. The agent's primary job (play) must not fail due to spectator unavailability.

### Decision 4: Context File Is Agent-Private

The context file loaded via `--context` is never exposed to opponents or the chain. It is loaded into memory at startup and saved at shutdown.

**Rationale**: Agent memory and identity are competitive information. The system treats them as confidential.

---

## Integration Slice Sequencing

The integration work follows the same 9-slice structure as `agent:experience`, but focuses on **wiring** rather than **implementation**:

| Slice | Integration Work | Blocks |
|-------|----------------|--------|
| 1 | Adapter handles scaffold (`adapter.rs`) + `AgentContextHandle::load()` wiring | None — can start immediately |
| 2 | `JournalHandle::append_hand_entry()` wiring in `PipeMode::handle_action()` | Slice 1 |
| 3 | `--context` CLI flag → `PipeMode::new()` + `AgentContextHandle` lifetime | `play:tui` binary skeleton |
| 4 | `reflect>` prompt integration (`prompt_reflection()`) | Slice 3 |
| 5 | `NarrationEngine` integration via `NarrationHandle` | `narration.rs` exists (Slice 5 of 9) |
| 6 | `--narrate` CLI flag wiring | Slice 5 |
| 7 | Lobby mode integration in `PipeMode` | `play:tui` binary + chain stub |
| 8 | `SpectatorHandle::emit()` wiring in game event loop | Slice 8 of 9 |
| 9 | `SpectateScreen` integration via `SpectatorHandle` | Slice 8 + `play:tui` screen system |

---

## Honest Blocker Assessment

### Blocker 1: `myosu-play` Binary Does Not Exist
**Severity: HIGH**
**Blocks: Slices 3, 4, 6, 7**

The `myosu-play` binary is the vehicle through which all CLI flags (`--context`, `--narrate`, `--reflect`, `--spectate`) are exposed. Until `play:tui` Slice 1 (binary skeleton) completes, no agent-facing flag can be wired.

**Owned by**: `play:tui` lane

### Blocker 2: `robopoker` Git Migration Unresolved
**Severity: HIGH (for CI/testing)**
**Blocks: Full integration testing**

All agent-facing code ultimately calls into `games:traits` or `tui:shell`, which call into `robopoker` via absolute filesystem paths. `cargo build` and `cargo test` will fail on any clean checkout or CI environment.

**Owned by**: `games:traits` lane (Slice 1)

### Blocker 3: `chain:runtime` Not Available
**Severity: MEDIUM**
**Blocks: Slice 7 (Lobby)**

The lobby (Slice 7) requires querying active subnet information. AC-AX-05 shows displaying miner count, exploitability, and game status — all requiring live chain data.

**Workaround for Slice 7**: Stub lobby with hardcoded data for Phase 0. Real chain integration is Phase 4.

### Blocker 4: `SpectatorRelay` UNIX Socket Path
**Severity: LOW**
**Blocks: Slice 8**

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock`. This convention should be confirmed against `play:tui`'s data directory convention.

**Verification needed**: Check `play:tui` data directory before Slice 8 implementation.

---

## Honest Slice-by-Slice Feasibility

| Slice | Can Start Now? | Why |
|-------|----------------|-----|
| Slice 1 (adapter scaffold) | **YES** | Depends only on `tui:shell` (trusted) |
| Slice 2 (journal wiring) | **YES** (after Slice 1) | Depends on Slice 1 |
| Slice 3 (--context wiring) | **NO** | Needs `myosu-play` binary from `play:tui` |
| Slice 4 (reflect> prompt) | **NO** | Needs Slice 3 |
| Slice 5 (narration engine) | **YES** (devenv) | Engine implementation independent of wiring |
| Slice 6 (--narrate wiring) | **NO** | Needs Slice 5 + `myosu-play` |
| Slice 7 (lobby) | **NO** | Needs `myosu-play` + chain stub |
| Slice 8 (spectator emit) | **NO** | Needs `SpectatorRelay` from Slice 8 of 9 |
| Slice 9 (spectate screen) | **NO** | Needs Slice 8 |

**Summary**: Slices 1 and 2 can start immediately. Slices 3–9 require `play:tui` binary skeleton (owned by `play:tui` lane). Slices 5 can be developed in isolation as a standalone engine.

---

## Proof / Check Shape for the Lane

The lane is **proven** when adapter handles compile and integrate correctly:

```bash
# Adapter scaffold compiles
cargo check -p myosu-tui

# Agent context roundtrips ( Slice 1 )
cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip

# Journal appends (Slice 2)
cargo test -p myosu-tui journal::tests::append_hand_entry

# Narration engine compiles (Slice 5)
cargo check -p myosu-tui --features narration

# Full pipe mode with context + narrate compiles (after play:tui binary)
cargo check -p myosu-play --features pipe
```

---

## Decision Log

- **Decision**: Adapter handles are thin wrappers, not new domain types.
  **Rationale**: Keeps integration layer minimal. Domain logic belongs in `agent_context.rs`, `journal.rs`, `narration.rs`.
  **Date**: 2026-03-20

- **Decision**: NarrationHandle is a no-op until NarrationEngine exists.
  **Rationale**: Allows `--narrate` flag wiring without blocking on Slice 5.
  **Date**: 2026-03-20

- **Decision**: SpectatorHandle emits fire-and-forget.
  **Rationale**: Agent's primary job (play) must not fail due to optional monitoring.
  **Date**: 2026-03-20

- **Decision**: Context file is agent-private, never exposed to opponents.
  **Rationale**: Agent memory is competitive information requiring confidentiality.
  **Date**: 2026-03-20

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/agent/experience/spec.md` | Product specification for agent experience |
| `outputs/agent/experience/review.md` | Product review; recommends implementation-family workflow |
| `crates/myosu-tui/src/pipe.rs` | PipeMode driver (trusted upstream; extend with adapter handles) |
| `crates/myosu-tui/src/renderer.rs` | GameRenderer trait (trusted) |
| `crates/myosu-tui/src/schema.rs` | GameState JSON schema (trusted; 16 tests pass) |
| `crates/myosu-tui/src/shell.rs` | Shell entry point (trusted; 82 tests pass) |
| `crates/myosu-play/src/main.rs` | CLI binary (owned by `play:tui`; adds Pipe args) |
| `specsarchive/031626-10-agent-experience.md` | AX-01..05 source specification |
| `specsarchive/031626-17-spectator-protocol.md` | SP-01..03 spectator protocol |
