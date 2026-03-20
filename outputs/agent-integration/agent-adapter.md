# `agent-integration` Lane Spec

## Purpose and User-Visible Outcome

`agent-integration` is the **integration wiring layer** between `agent:experience` (the presentation surface) and the rest of the myosu system (miners, validators, chain discovery, gameplay). It owns the adapter contracts that connect the agent-facing pipe/HTTP/WebSocket transports to the actual solver market.

The lane delivers:

1. **`AgentAdapter` trait** — a unified interface for sending game actions to a solver and receiving strategy responses, abstracting over pipe mode, HTTP, WebSocket, and in-process modes
2. **Transport implementations** — one implementation per transport: `PipeAdapter`, `HttpAdapter`, `WsAdapter`, `ProcessAdapter` (for the Rust `Strategy` trait)
3. **Chain discovery adapter** — resolves subnet IDs to miner axon endpoints so agents can select which game to play
4. **Strategy query pipeline** — encodes `GameState` into a `StrategyQuery`, sends it to a miner axon, decodes the `StrategyResponse` into an action distribution
5. **Session management** — manages the agent's active session across multiple hands, including reconnection and state resumption

**User-visible behavior**: An agent calls `MyosuAgent::new("nlhe-hu")`, receives action distributions for each decision point, sends back chosen actions, and the adapter handles all transport, retry, and discovery logic transparently.

---

## Lane Boundary

```
                            agent-integration (THIS LANE)
                            ┌──────────────────────────────────────────────────────────────┐
upstream                        │                                                              │
agent:experience ────────────► │  AgentAdapter (trait) ────► PipeAdapter                    │
  (spec reviewed, KEEP)        │                    ├──► HttpAdapter                       │
                               │                    ├──► WsAdapter                         │
                               │                    └──► ProcessAdapter (Strategy trait)    │
                               │                                                              │
                               │  ChainDiscovery ────► subnet → miner axon URL              │
                               │  StrategyQueryPipeline ───► encode → send → decode → act   │
                               │  SessionManager ────► multi-hand session, reconnect        │
                               │                                                              │
downstream                      │                                                              │
miner axon (HTTP) ──────────────► │  raw /strategy endpoint                                 │
validator scoring ─────────────► │  exploitability oracle adapter                             │
chain (runtime) ───────────────► │  subnet registry, registration                            │
                               └──────────────────────────────────────────────────────────────┘
```

**Trusted upstream inputs:**
- `agent:experience` (reviewed, KEEP) — `GameRenderer::pipe_output()`, `schema.rs`, the pipe mode driver
- `games:traits` (trusted) — `CfrGame`, `Profile`, `GameType`, `StrategyQuery`, `StrategyResponse`

**Trusted downstream outputs:**
- Miner HTTP axons (existing interface: `POST /strategy`, returns `StrategyResponse`)
- Chain RPC (subnet registry, neuron state)

**Untrusted inputs (validated at use site):**
- Miner HTTP responses (never trust; validate `StrategyResponse` schema before use)
- Chain state (always re-validate on read)

---

## What `agent:experience` Does NOT Cover

The `agent:experience` lane spec (`outputs/agent/experience/spec.md`) defines the **presentation** surfaces — pipe output format, narration, reflection prompts, journal, spectator relay. It does NOT define:

1. **How an agent connects to a miner** — the transport (HTTP, WS, pipe subprocess) is not specified in `agent:experience`
2. **How an agent discovers which miners exist** — chain discovery for subnet listing is not in `agent:experience`
3. **How an agent handles miner failures or retries** — retry logic, timeout, fallback to other miners
4. **How an agent encodes a query for the miner** — the `StrategyQuery` / `StrategyResponse` wire format is owned by `games:traits`
5. **How a session persists across multiple hands with the same miner** — session state management

`agent-integration` fills these gaps.

---

## `AgentAdapter` Trait

```rust
/// Unified interface for agent-to-solver communication.
///
/// All adapters translate agent actions into strategy queries
/// and return action distributions from the solver market.
pub trait AgentAdapter: Send + Sync {
    /// The transport kind for observability.
    fn transport(&self) -> TransportKind;

    /// Query the solver for action probabilities at the current game state.
    fn query_strategy(&self, state: &GameState) -> Result<ActionDistribution, AdapterError>;

    /// Submit the agent's chosen action for logging / potential future training.
    fn submit_action(&self, state: &GameState, action: &GameAction) -> Result<(), AdapterError>;

    /// Check if the adapter is still connected to a miner.
    fn is_connected(&self) -> bool;

    /// Attempt reconnection to the miner.
    fn reconnect(&mut self) -> Result<(), AdapterError>;
}

#[derive(Clone, Copy, Debug)]
pub enum TransportKind {
    Pipe,   // stdin/stdout subprocess
    Http,   // HTTP REST
    Ws,     // WebSocket
    Process, // in-process Rust Strategy trait
}
```

### `PipeAdapter`

Wraps a subprocess running `myosu-play --pipe`. Sends game state via stdin, reads action distribution from stdout. Used for simple bots, testing, and legacy integration.

**Contract**: Subprocess must speak the pipe protocol defined in `agent:experience`. The adapter does NOT implement the pipe protocol — it delegates to the subprocess.

### `HttpAdapter`

Connects to a miner axon's `POST /strategy` endpoint. Encodes `GameState` as JSON (using `schema.rs` types), decodes `StrategyResponse`. Timeout per request: 5s default, configurable.

**Contract**: Miner axon must implement the `/strategy` endpoint with the `StrategyQuery` / `StrategyResponse` schema from `games:traits`.

### `WsAdapter`

Persistent WebSocket connection to a miner axon. Sends game events as JSON lines, receives action distributions. Supports server-push (solver can send updates without a request). Used for real-time gameplay.

**Contract**: Same as `HttpAdapter` but with connection persistence and server-push.

### `ProcessAdapter`

Loads a Rust `Strategy` trait implementation from a dynamic library (`.so`). Zero-overhead in-process integration for high-frequency queries. Used when the agent and solver run in the same process.

**Contract**: Library must export a `strategize` function returning `ActionDistribution` for a given `GameState`.

---

## Chain Discovery

Agents need to discover which subnets and miners are available before playing. The `ChainDiscovery` trait abstracts chain queries:

```rust
pub trait ChainDiscovery: Send + Sync {
    /// List all active subnets with summary info.
    fn list_subnets(&self) -> Result<Vec<SubnetInfo>, AdapterError>;

    /// Get detailed info for a specific subnet including active miners.
    fn subnet_detail(&self, subnet_id: u16) -> Result<SubnetDetail, AdapterError>;

    /// Get the best miner endpoint for a subnet (by last score).
    fn best_miner(&self, subnet_id: u16) -> Result<MinerEndpoint, AdapterError>;
}

#[derive(Clone, Debug)]
pub struct SubnetInfo {
    pub id: u16,
    pub game_type: GameType,
    pub miner_count: u32,
    pub avg_exploitability: f64,
    pub status: SubnetStatus,
}

#[derive(Clone, Debug)]
pub struct MinerEndpoint {
    pub axon_url: Url,
    pub last_score: f64,
    pub version: String,
}
```

**Phase 0 stub**: `ChainDiscovery` returns hardcoded data for devnet. Real chain integration (querying the Substrate RPC) is Phase 4 (depends on `chain:runtime`).

---

## Strategy Query Pipeline

The query pipeline encodes the interaction between agent and miner:

```
GameState (from agent:experience)
  │
  ▼
encode_strategy_query(state) → StrategyQuery (serde JSON)
  │
  ▼
adapter.query_strategy(query) → StrategyResponse
  │
  ▼
decode_action_distribution(response) → ActionDistribution
  │
  ▼
agent selects action → GameAction
  │
  ▼
adapter.submit_action(state, action) → ()
```

**ActionDistribution** (from `games:traits`):
```rust
pub struct ActionDistribution {
    pub actions: Vec<(GameAction, Probability)>,
    pub best_action: GameAction,
    pub confidence: f64,        // 0..1, based on exploitability gap
    pub exploitability: f64,   // mbb/h, for display only
}
```

**Error taxonomy** (all adapter errors are typed, never stringly-typed):

```rust
#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("miner timeout after {0}s")]
    Timeout(u64),

    #[error("miner returned invalid response: {0}")]
    InvalidResponse(String),

    #[error("miner endpoint unreachable: {0}")]
    Unreachable(#[from] reqwest::Error),

    #[error("encoding error: {0}")]
    Encoding(#[from] serde_json::Error),

    #[error("not connected to miner")]
    Disconnected,

    #[error("chain query failed: {0}")]
    ChainError(String),
}
```

---

## Session Management

A **session** is an agent's continuous interaction with a subnet across multiple hands. The `SessionManager` owns:

```rust
pub struct SessionManager {
    adapter: Box<dyn AgentAdapter>,
    chain: Box<dyn ChainDiscovery>,
    subnet_id: u16,
    session_id: Uuid,
    hand_count: u64,
    cumulative_result: i64,  // in bb × 100 (milli-big-blinds)
}

impl SessionManager {
    /// Create a new session against a specific subnet.
    pub fn new(subnet_id: u16, adapter: Box<dyn AgentAdapter>, chain: Box<dyn ChainDiscovery>) -> Self;

    /// Play one hand: query strategy → get action → submit action.
    pub fn play_hand(&mut self, state: &GameState) -> Result<GameAction, AdapterError>;

    /// Get the session transcript (all states + actions for the journal).
    pub fn transcript(&self) -> &[HandTranscript];

    /// Save session state to a context file (for recovery after restart).
    pub fn save_context(&self, path: &Path) -> Result<(), AdapterError>;

    /// Load session state from a context file.
    pub fn load_context(path: &Path) -> Result<Self, AdapterError>;
}

pub struct HandTranscript {
    pub hand_number: u64,
    pub state: GameState,
    pub action: GameAction,
    pub distribution: ActionDistribution,
    pub result: Option<HandResult>,
}
```

**Recovery**: If an agent restarts with a context file, `SessionManager::load_context()` restores hand count, cumulative result, and adapter connection state. The miner is NOT re-notified of the missed hands — recovery is local only.

---

## File Deliverables

| File | Responsibility | Status |
|------|---------------|--------|
| `crates/myosu-tui/src/agent/adapter.rs` | `AgentAdapter` trait + all transport implementations | **MISSING** |
| `crates/myosu-tui/src/agent/discovery.rs` | `ChainDiscovery` trait + devnet stub | **MISSING** |
| `crates/myosu-tui/src/agent/session.rs` | `SessionManager` + `HandTranscript` | **MISSING** |
| `crates/myosu-tui/src/agent/mod.rs` | Module entry point | **MISSING** |
| `crates/myosu-tui/src/agent/error.rs` | `AdapterError` enum | **MISSING** |
| `crates/myosu-play/src/bin/myosu-agent.rs` | CLI binary: `myosu-agent --pipe --subnet 1` | **MISSING** |

---

## Proof / Check Shape for the Lane

The lane is **proven** when all of the following pass:

```
# Adapter trait object safety and Send+Sync bounds
cargo check -p myosu-tui --lib

# All transport adapters compile (no missing implementations)
cargo check -p myosu-tui --lib

# Session manager roundtrips context file
cargo test -p myosu-tui agent::session::tests::context_roundtrip

# HttpAdapter encodes/decodes StrategyQuery correctly
cargo test -p myosu-tui agent::adapter::tests::http_query_roundtrip

# ChainDiscovery devnet stub returns valid SubnetInfo
cargo test -p myosu-tui agent::discovery::tests::devnet_stub_returns_subnets

# PipeAdapter subprocess lifecycle (start, communicate, stop)
cargo test -p myosu-tui agent::adapter::tests::pipe_subprocess_lifecycle
```

---

## Dependency on Other Lanes

| Lane | Type | What Is Used |
|------|------|-------------|
| `agent:experience` | Hard upstream | `GameRenderer::pipe_output()`, `schema.rs` types, pipe mode driver |
| `games:traits` | Hard upstream | `StrategyQuery`, `StrategyResponse`, `GameState`, `GameAction`, `ActionDistribution` |
| `chain:runtime` | Soft upstream (Phase 4) | Real Substrate RPC for chain discovery |
| `play:tui` | Soft upstream | `myosu-play` binary for pipe subprocess mode |
| `miner:service` | Untrusted downstream | HTTP axon at `/strategy` endpoint |
| `validator:oracle` | Untrusted downstream | Exploitability scoring adapter (future) |

---

## Phase Ordering

```
Phase 1 (Local Integration — depends on agent:experience spec reviewed):
  Slice 1 → Slice 2 → Slice 3

Phase 2 (Transport Implementations — depends on Phase 1):
  Slice 4 → Slice 5 → Slice 6

Phase 3 (Chain Discovery — depends on chain:runtime):
  Slice 7 → Slice 8

Phase 4 (Session Recovery + Spectator Integration — depends on Phase 3):
  Slice 9 → Slice 10
```

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1: `AgentAdapter` Trait + Error Types
**Files**: `crates/myosu-tui/src/agent/error.rs`, `crates/myosu-tui/src/agent/adapter.rs`
**What**: Define `AdapterError`, `TransportKind`, `AgentAdapter` trait with all four transport variants. No implementations yet — just the trait and types.
**Proof gate**: `cargo check -p myosu-tui --lib` compiles with `use agent::{AdapterError, AgentAdapter};`

### Slice 2: `PipeAdapter` Implementation
**Files**: `crates/myosu-tui/src/agent/adapter.rs`
**What**: Implement `AgentAdapter` for `PipeAdapter` (subprocess management, stdin/stdout communication). Uses `myosu-play --pipe` as subprocess.
**Proof gate**: `cargo test -p myosu-tui agent::adapter::tests::pipe_subprocess_lifecycle`

### Slice 3: `HttpAdapter` Implementation
**Files**: `crates/myosu-tui/src/agent/adapter.rs`
**What**: Implement `AgentAdapter` for `HttpAdapter` using `reqwest`. Timeout, retry (1x), and response validation.
**Proof gate**: `cargo test -p myosu-tui agent::adapter::tests::http_query_roundtrip`

### Slice 4: `ChainDiscovery` + Devnet Stub
**Files**: `crates/myosu-tui/src/agent/discovery.rs`
**What**: Define `ChainDiscovery` trait and `DevnetDiscovery` stub that returns hardcoded `SubnetInfo` for 2 devnet subnets.
**Proof gate**: `cargo test -p myosu-tui agent::discovery::tests::devnet_stub_returns_subnets`

### Slice 5: `SessionManager`
**Files**: `crates/myosu-tui/src/agent/session.rs`
**What**: `SessionManager` with `play_hand()`, `transcript()`, `save_context()`, `load_context()`. Context file is JSON.
**Proof gate**: `cargo test -p myosu-tui agent::session::tests::context_roundtrip`

### Slice 6: `WsAdapter` + `ProcessAdapter`
**Files**: `crates/myosu-tui/src/agent/adapter.rs`
**What**: Complete the remaining two `AgentAdapter` implementations.
**Proof gate**: `cargo check -p myosu-tui --lib` compiles all four transports

### Slice 7: `myosu-agent` CLI Binary
**Files**: `crates/myosu-play/src/bin/myosu-agent.rs`
**What**: CLI binary that wires `--pipe`, `--http`, `--ws` flags to the appropriate `AgentAdapter` implementation. Connects to a subnet via `ChainDiscovery`.
**Proof gate**: `myosu-agent --help` shows all flags; `myosu-agent --pipe --subnet 1 --context ./koan.json` starts a session

### Slice 8: Real Chain Discovery (depends on `chain:runtime`)
**Files**: `crates/myosu-tui/src/agent/discovery.rs`
**What**: `SubstrateDiscovery` implementation that queries the actual Substrate RPC for subnet info and miner endpoints.
**Proof gate**: Against a live devnet chain: `myosu-agent --http --subnet 1` connects to the real subnet

### Slice 9: Reconnection Logic
**Files**: `crates/myosu-tui/src/agent/session.rs`
**What**: `SessionManager::reconnect()` that detects disconnects, tries all miners for a subnet, resumes session.
**Proof gate**: Kill a miner mid-session → agent reconnects to another miner → session continues with correct hand count

### Slice 10: Spectator Relay Integration
**Files**: `crates/myosu-tui/src/agent/session.rs`, `crates/myosu-tui/src/agent/adapter.rs`
**What**: `SessionManager` emits `HandTranscript` events to the `SpectatorRelay` (from `agent:experience` Slice 8) so out-of-process spectators can watch the agent play.
**Proof gate**: `myosu-agent --http --subnet 1 | SpectatorRelay` → spectator socket receives valid `GameEvent` JSON lines
