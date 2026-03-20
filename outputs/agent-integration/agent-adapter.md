# `agent:integration` Lane Specification

## Purpose and User-Visible Outcome

`agent:integration` is the **adapter layer** that wires agent-facing surfaces to Myosu's core systems. Where `agent:experience` defines *what* the agent sees (`--pipe`, `--context`, `--narrate`, journal, reflection, lobby, spectator relay), this lane defines *how* those surfaces connect to `games:traits`, `play:tui`, and the chain client.

The lane delivers:

1. **`AgentAdapter` trait** — a thin abstraction that translates between agent-facing protocols and Myosu's internal game state
2. **`MyosuAgentAdapter` implementation** — wires `PipeMode` to `games:traits` (`CfrGame`, `Profile`, `StrategyQuery`) and persists agent context
3. **Context file integration** — loads/saves `AgentContext` JSON through the adapter rather than directly in `pipe.rs`
4. **Chain-discovery stub** — lobby queries miner axon for subnet info (stubbed for Phase 0; real chain integration in Phase 4)
5. **Spectator relay integration** — `SpectatorRelay` emits `GameEvent` lines through the adapter to Unix sockets

**User-visible behavior**: An agent connects via `myosu-play --pipe --context ./koan.json` and receives a fully integrated experience: game state from `games:traits`, persistent memory from the context file, journal entries appended to disk, and a spectator relay streaming valid JSON events.

---

## Lane Boundary

```
                            agent:integration (THIS LANE)
                            ┌──────────────────────────────────────────────────────────────┐
upstream surfaces           │                                                              │
from agent:experience       │  ┌──────────────────────────────────────────────────────┐   │
                            │  │              AgentAdapter trait                         │   │
pipe.rs ──────────────────► │  │  translate_agent_action() → GameAction                │   │
                            │  │  get_game_state() → GameState                        │   │
                            │  │  load_context() → AgentContext                       │   │
                            │  │  save_context()                                       │   │
                            │  │  query_subnets() → Vec<SubnetInfo>                  │   │
                            │  │  emit_spectator_event(GameEvent)                     │   │
                            │  └──────────────────────────────────────────────────────┘   │
                            │                           │                                 │
                            │          ┌────────────────┴────────────────┐               │
                            │          ▼                                 ▼               │
                            │  ┌───────────────────┐        ┌───────────────────────┐    │
                            │  │ MyosuAgentAdapter │        │ StubAgentAdapter      │    │
                            │  │ (real impl)       │        │ (for testing)        │    │
                            │  └───────────────────┘        └───────────────────────┘    │
                            │          │                                                │
upstream                    │          ▼                                                │
games:traits ─────────────► │  calls: CfrGame, Profile, StrategyQuery, GameConfig    │
upstream                    │          │                                                │
play:tui ─────────────────► │  uses: Shell, GameRenderer, PipeMode                    │
upstream                    │          │                                                │
chain client (future) ─────► │  queries: SubnetInfo, miner axon (Phase 4)             │
                            │                                                              │
downstream                  │  ┌──────────────────────────────────────────────────────┐   │
spectator relay ───────────► │  │ SpectatorRelay: emits GameEvent to Unix socket       │   │
                            │  └──────────────────────────────────────────────────────┘   │
                            └──────────────────────────────────────────────────────────────┘
```

**Trusted upstream inputs:**
- `tui:shell` (82 tests pass) — `Shell`, `GameRenderer`, `PipeMode`
- `games:traits` (14 tests pass) — `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response`
- `agent:experience` — surfaces: `PipeMode`, `AgentContext`, `Journal`, `NarrationEngine`, `SpectatorRelay`
- `play:tui` — `myosu-play` binary skeleton

**Untrusted inputs** (validated at use site):
- Agent-supplied context JSON file (serde-validated before use)
- Agent-supplied reflection text (free-form string)
- Spectator relay socket connections (managed by relay)

**Trusted downstream outputs:**
- `SpectatorRelay` — event emitter
- Agent context file on disk
- Journal markdown file on disk

---

## Current Implementation Status

| Surface | Status | Evidence |
|---------|--------|----------|
| `AgentAdapter` trait | **MISSING** | No trait at this path |
| `MyosuAgentAdapter` impl | **MISSING** | No implementation |
| `StubAgentAdapter` for tests | **MISSING** | No test adapter |
| Context file integration via adapter | **MISSING** | Context loaded directly in pipe.rs |
| Chain-discovery stub | **MISSING** | Lobby has no subnet query |
| Spectator relay integration via adapter | **MISSING** | `SpectatorRelay` not yet implemented |

---

## The `AgentAdapter` Trait

The `AgentAdapter` is a **porcelain layer** over Myosu's core systems. It is not a new abstraction — it is a convenient wiring of existing surfaces into a single interface that `pipe.rs` and the spectator relay can use without knowing about `games:traits`, `play:tui`, or the chain client directly.

```rust
// crates/myosu-tui/src/agent_adapter.rs

use serde::{Deserialize, Serialize};

/// Agent-facing integration trait.
///
/// Abstracts the wiring between agent-facing surfaces (pipe mode, context files,
/// journal, spectator relay) and Myosu's core systems (games:traits, play:tui,
/// chain client).
///
/// Implementors:
/// - `MyosuAgentAdapter` — real implementation using games:traits + play:tui
/// - `StubAgentAdapter` — test double with hardcoded responses
pub trait AgentAdapter: Send + Sync {
    /// Translate a raw text action from the agent into a game-specific action.
    fn translate_agent_action(&self, raw: &str, game: &dyn GameRenderer) -> Option<GameAction>;

    /// Load the agent's persistent context from disk.
    fn load_context(&self, path: &Path) -> Result<AgentContext, AdapterError>;

    /// Save the agent's persistent context to disk.
    fn save_context(&self, path: &Path, ctx: &AgentContext) -> Result<(), AdapterError>;

    /// Query available subnets (stubbed in Phase 0).
    fn query_subnets(&self) -> Vec<SubnetInfo>;

    /// Emit a spectator event to the relay.
    fn emit_spectator_event(&self, event: GameEvent) -> Result<(), AdapterError>;
}

/// Error type for adapter operations.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("context file not found")]
    ContextNotFound,

    #[error("context file corrupted: {0}")]
    ContextCorrupted(String),

    #[error("save failed: {0}")]
    SaveFailed(String),

    #[error("chain query failed: {0}")]
    ChainQueryFailed(String),

    #[error("spectator relay error: {0}")]
    SpectatorRelayError(String),
}

/// Agent context structure (mirrors AC-AX-01).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub identity: AgentIdentity,
    pub memory: AgentMemory,
    pub journal: Vec<JournalEntry>,
}

/// Subnet information for the lobby.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetInfo {
    pub id: u16,
    pub game_type: String,
    pub miner_count: u32,
    pub avg_exploitability: Option<f64>,
    pub status: SubnetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubnetStatus {
    Active,
    Bootstrap,
    Paused,
}

/// Spectator event structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub event_type: EventType,
    pub session_id: String,
    pub hand: Option<u32>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    HandStart,
    Action,
    HandComplete,
    Showdown,
    SessionSummary,
}
```

---

## Integration with Existing Surfaces

### Pipe Mode (`pipe.rs`)

`PipeMode` currently directly calls `GameRenderer::pipe_output()`. After this lane:

1. `PipeMode` holds `Arc<dyn AgentAdapter>` instead of `&dyn GameRenderer`
2. Agent actions are translated through `AgentAdapter::translate_agent_action()`
3. Context loading/saving goes through the adapter
4. Spectator events are emitted through the adapter

**Before** (current):
```rust
pub struct PipeMode<'a> {
    renderer: &'a dyn GameRenderer,
}
```

**After** (this lane):
```rust
pub struct PipeMode {
    adapter: Arc<dyn AgentAdapter>,
    renderer: Arc<dyn GameRenderer>,
    context_path: Option<PathBuf>,
}
```

### Agent Context (`agent_context.rs`)

`AgentContext` structs (from `agent:experience`) are loaded/saved through the adapter:

- Adapter's `load_context()` handles file I/O + serde validation
- Adapter's `save_context()` handles atomic write (write-temp-then-rename)
- Missing file → returns `AdapterError::ContextNotFound` → caller creates fresh context

### Spectator Relay

`SpectatorRelay` (from `agent:experience` Slice 8) emits through the adapter:

- `adapter.emit_spectator_event(event)` → relay writes JSON to Unix socket
- Adapter owns socket management; relay just calls `emit()`
- Fog-of-war enforcement stays in relay (not in adapter)

### Lobby (Chain Discovery)

`query_subnets()` returns `Vec<SubnetInfo>`:

- Phase 0: returns stubbed data (`[SubnetInfo { id: 1, game_type: "nlhe-hu", miner_count: 12, avg_exploitability: Some(13.2), status: Active }]`)
- Phase 4: queries miner axon HTTP endpoint

---

## Code Boundaries and Deliverables

| File | Responsibility | Status |
|------|---------------|--------|
| `crates/myosu-tui/src/agent_adapter.rs` | `AgentAdapter` trait + `AdapterError` + types | **MISSING** |
| `crates/myosu-tui/src/myousu_agent_adapter.rs` | `MyosuAgentAdapter` implementation | **MISSING** |
| `crates/myosu-tui/src/stub_agent_adapter.rs` | `StubAgentAdapter` for tests | **MISSING** |
| `crates/myosu-tui/src/pipe.rs` | Refactor to use `AgentAdapter` | **PARTIAL** |
| `crates/myosu-tui/src/agent_context.rs` | Uses adapter for I/O (not file ops directly) | **MISSING** |

---

## First Honest Implementation Slice

### Slice 1: `AgentAdapter` Trait + Types + Stub Implementation

**Files**: `crates/myosu-tui/src/agent_adapter.rs`
**What**:
1. Define `AgentAdapter` trait with all methods
2. Define `AdapterError` enum with `thiserror`
3. Define all types: `AgentContext`, `SubnetInfo`, `GameEvent`, `EventType`, `SubnetStatus`
4. Implement `StubAgentAdapter` with hardcoded responses
5. Write integration tests using `StubAgentAdapter`

**Proof gate**: `cargo test -p myosu-tui agent_adapter::tests`

### Slice 2: `MyosuAgentAdapter` Skeleton

**Files**: `crates/myosu-tui/src/myousu_agent_adapter.rs`
**What**:
1. `MyosuAgentAdapter` struct holding `Arc<CfrGame>`, `Arc<Profile>`, `PathBuf` for context
2. `translate_agent_action()` — delegates to `GameRenderer::parse_input()`
3. `load_context()` — reads JSON file, returns `Result<AgentContext, AdapterError>`
4. `save_context()` — atomic write (temp file + rename)
5. `query_subnets()` — returns stubbed `SubnetInfo` vec
6. `emit_spectator_event()` — no-op stub for Phase 0

**Proof gate**: `cargo test -p myosu-tui myosu_agent_adapter::tests`

### Slice 3: Refactor `PipeMode` to Use `AgentAdapter`

**Files**: `crates/myosu-tui/src/pipe.rs`
**What**:
1. Change `PipeMode<'a>` to `PipeMode` (no lifetime)
2. Replace `renderer: &'a dyn GameRenderer` with `adapter: Arc<dyn AgentAdapter>`
3. Add `context_path: Option<PathBuf>` field
4. On `new()`: load context if path provided
5. On drop: save context if path provided
6. Refactor `output_state()` to use adapter

**Proof gate**: `cargo test -p myosu-tui pipe::tests` (existing tests still pass)

### Slice 4: Refactor `agent_context.rs` to Use `AgentAdapter`

**Files**: `crates/myosu-tui/src/agent_context.rs`
**What**:
1. `AgentContext::load()` takes `&dyn AgentAdapter` + `&Path`
2. `AgentContext::save()` takes `&dyn AgentAdapter` + `&Path`
3. Remove direct `std::fs` calls from `agent_context.rs`
4. All file I/O goes through the adapter

**Proof gate**: `cargo test -p myosu-tui agent_context::tests`

---

## Dependency on Other Lanes

| Lane | Type | What Is Used |
|------|------|-------------|
| `agent:experience` | Spec source | Surfaces: `PipeMode`, `AgentContext`, `Journal`, `NarrationEngine`, `SpectatorRelay` |
| `tui:shell` | Hard upstream | `Shell`, `GameRenderer` trait, `PipeMode`; 82 tests pass |
| `games:traits` | Hard upstream | `CfrGame`, `Profile`, `GameConfig`, `GameType`; 14 tests pass |
| `play:tui` | Hard upstream | `myosu-play` binary (Slice 1: binary skeleton needed) |
| `chain:runtime` | Soft upstream (Phase 4) | Miner axon HTTP endpoint for subnet queries |

---

## Phase Ordering

```
Phase 1 (Adapter Core — depends on tui:shell + games:traits):
  Slice 1 → Slice 2 → Slice 3 → Slice 4

Phase 2 (Spectator Integration — depends on Slice 1):
  Slice 8 from agent:experience (SpectatorRelay) uses adapter.emit_spectator_event()

Phase 3 (Chain-Connected — depends on chain:runtime):
  MyosuAgentAdapter::query_subnets() upgrades from stub to real chain query
```

---

## Proof / Check Shape for the Lane

The lane is **proven** when all of the following pass:

```
# Adapter trait and stub
cargo test -p myosu-tui agent_adapter::tests
cargo test -p myosu-tui agent_adapter::tests::stub_returns_hardcoded_context

# Myosu adapter skeleton
cargo test -p myosu-tui myosu_agent_adapter::tests
cargo test -p myosu-tui myosu_agent_adapter::tests::load_context_missing_file_returns_error
cargo test -p myosu-tui myosu_agent_adapter::tests::save_context_atomic

# Pipe mode refactored to use adapter
cargo test -p myosu-tui pipe::tests
cargo test -p myosu-tui pipe::tests::pipe_mode_uses_adapter

# Agent context uses adapter
cargo test -p myosu-tui agent_context::tests
cargo test -p myosu-tui agent_context::tests::load_uses_adapter
cargo test -p myosu-tui agent_context::tests::save_uses_adapter

# Integration: full stack with stub adapter
cargo test -p myosu-tui integration::tests::agent_plays_hand_with_stub_adapter
```
