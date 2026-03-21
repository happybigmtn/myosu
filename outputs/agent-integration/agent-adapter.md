# `agent-integration` Lane Specification

## Purpose and User-Visible Outcome

`agent-integration` is the **client-side integration layer** for Myosu — the adapter code that connects programmatic agents (LLMs, bots, scripts) to the Myosu gameplay surface.

The lane delivers:

1. **`PipeClient`** — a typed Rust client for `myosu-play --pipe` mode, handling stdin/stdout serialization, context file lifecycle, and reflection prompt processing
2. **`AgentSession`** — a session manager that wraps `PipeClient`, exposes high-level `act()` and `query()` calls, and maintains the agent context + journal lifecycle
3. **`HttpAgentClient`** (Phase 2) — a future HTTP/WS client for direct miner axon communication, replacing the pipe adapter once `chain:runtime` is available
4. **`myosu-agent`** binary — a standalone agent runner that composes `AgentSession` with CLI argument parsing and optional autonomous loop control

**User-visible behavior**: An agent developer imports `myosu-agent-core`, constructs an `AgentSession::new(path_to_context)`, and calls `session.act(&game_state)` or `session.query_best_action()` to interact with the solver. The session handles all pipe protocol details, context persistence, and journal appending transparently.

---

## Lane Boundary

```
                            agent-integration (THIS LANE)
                            ┌──────────────────────────────────────────────────────┐
upstream                     │                                                      │
agent:experience ───────────► │  PipeClient          AgentSession                  │
  (pipe mode surfaces,        │  ├─ stdin/stdout      ├─ context lifecycle         │
   narration, schema)         │  ├─ reflection parse  ├─ journal lifecycle         │
                            │  ├─ context file I/O  ├─ high-level act()/query()   │
                            │  └─ lobby handling   └─ myosu-agent binary          │
                            │                                                      │
                            │  HttpAgentClient (Phase 2)                           │
                            │  ├─ HTTP POST to miner axon                         │
                            │  ├─ WebSocket event stream                          │
                            │  └─ replaces PipeClient after chain:runtime         │
                            │                                                      │
upstream                     │                                                      │
chain:runtime ──────────────► │  (Phase 2 only: miner axon endpoint discovery)    │
                            │                                                      │
downstream                   │                                                      │
agent:experience impl ──────► │  (consumes PipeClient and AgentSession)           │
  (slices 1-9)               │                                                      │
                            └──────────────────────────────────────────────────────┘
```

**Trusted upstream inputs:**
- `agent:experience` (reviewed spec + review) — defines `pipe_output()` contract, `--pipe` flag behavior, `--context` and `--narrate` semantics, `reflect>` prompt protocol, lobby format, and `game-state.json` schema
- `games:traits` (14 tests, trusted) — `CfrGame`, `StrategyQuery/Response` types
- `tui:shell` (82 tests, trusted) — `GameRenderer` trait and `PipeMode` behavior

**Trusted downstream outputs:**
- `agent:experience` implementation slices (consume `PipeClient` and `AgentSession`)
- `myosu-agent` binary (standalone agent runner)

---

## Current State

No code exists at these paths yet:

| Surface | Path | Status |
|---------|------|--------|
| `PipeClient` | `crates/myosu-agent-core/src/pipe_client.rs` | **MISSING** |
| `AgentSession` | `crates/myosu-agent-core/src/session.rs` | **MISSING** |
| `HttpAgentClient` | `crates/myosu-agent-core/src/http_client.rs` | **MISSING** |
| `myosu-agent` binary | `crates/myosu-agent/src/main.rs` | **MISSING** |
| `myosu-agent-core` crate | `crates/myosu-agent-core/` | **MISSING** |

**Preconditions from `agent:experience`:**
- `myosu-play --pipe` CLI exists and accepts `--context` and `--narrate` flags (Slice 3+ of `agent:experience`)
- `docs/api/game-state.json` schema is stable and trusted
- `GameRenderer::pipe_output()` produces the documented text format

---

## Code Boundaries and Deliverables

| File | Responsibility | Status |
|------|---------------|--------|
| `crates/myosu-agent-core/src/lib.rs` | Crate root; re-exports `PipeClient`, `AgentSession`, types | **MISSING** |
| `crates/myosu-agent-core/src/pipe_client.rs` | `PipeClient` struct; owns stdin/stdout handles; implements send/recv loop; handles `reflect>` prompt; parses lobby | **MISSING** |
| `crates/myosu-agent-core/src/session.rs` | `AgentSession` struct; wraps `PipeClient`; manages context load/save; exposes `act()`, `query_best_action()`, `start_session()` | **MISSING** |
| `crates/myosu-agent-core/src/http_client.rs` | `HttpAgentClient`; miner axon discovery; HTTP strategy queries; WS event subscription (Phase 2) | **MISSING** |
| `crates/myosu-agent-core/src/types.rs` | Shared types: `AgentContext`, `GameEvent`, `JournalEntry` (re-exported from `myosu-tui`) | **MISSING** |
| `crates/myosu-agent/src/main.rs` | `myosu-agent` binary; CLI arg parsing; autonomous loop mode; context path + session management | **MISSING** |
| `crates/myosu-agent-core/Cargo.toml` | Crate manifest; depends on `myosu-tui` (for types), `tokio`, `serde_json` | **MISSING** |

---

## Architecture

### `PipeClient`

```
PipeClient::new(context_path, narrate) -> Result<Self>
  ├─ spawns myosu-play --pipe --context <path> --narrate as child process
  ├─ owns Stdin/Stdout handles
  └─ drops process on drop

PipeClient::send_action(&self, action: &str) -> Result<GameOutput>
  ├─ writes action + "\n" to stdin
  ├─ reads lines from stdout until next prompt or HAND COMPLETE
  ├─ parses game state (terse or narrated depending on flag)
  └─ returns GameOutput

PipeClient::wait_reflection(&self) -> Result<Option<String>>
  ├─ reads until "reflect>" prompt
  ├─ user provides reflection text via PipeClient::provide_reflection()
  └─ returns Some(reflection) or None if empty line

PipeClient::request_lobby(&self) -> Result<Vec<SubnetInfo>>
  ├─ sends "lobby\n" command
  └─ parses MYOSU/LOBBY response

PipeClient::select_subnet(&self, id: u8) -> Result<GameOutput>
  ├─ sends "join <id>\n"
  └─ returns initial game state
```

### `AgentSession`

```
AgentSession::new(context_path: PathBuf) -> Result<Self>
  ├─ loads AgentContext from context_path (or creates default)
  ├─ spawns PipeClient with context path
  └─ initializes journal

AgentSession::act(&mut self, game_state: &GameState, action: &str) -> Result<ActResult>
  ├─ sends action via PipeClient::send_action()
  ├─ receives GameOutput
  ├─ if hand complete: wait for reflection, append to journal
  └─ return ActResult { game_output, reflection: Option<String> }

AgentSession::query_best_action(&mut self, game_state: &GameState) -> Result<String>
  ├─ sends "advice\n" command to pipe
  └─ parses advice response

AgentSession::journal_append(&mut self, entry: JournalEntry)
  ├─ appends to journal.md in context directory
  └─ never truncates

AgentSession::save_context(&self)
  ├─ serializes AgentContext to context_path
  └─ called on drop or explicitly

AgentSession::start_autonomous_loop<F>(&mut self, decide: F) -> Result<!>
  where F: Fn(&GameState) -> String
  ├─ enters loop: receive game state, call decide(), send action
  └─ never returns (runs until EOF or error)
```

### Relationship to `agent:experience` Surfaces

```
agent:experience defines          agent-integration implements
─────────────────────────────────────────────────────────────────
--pipe flag                      → PipeClient (child process management)
--context flag                   → AgentSession (context file lifecycle)
--narrate flag                   → PipeClient (passes flag to myosu-play)
reflect> prompt                  → PipeClient::wait_reflection()
lobby format                    → PipeClient::request_lobby() / ::select_subnet()
game-state.json schema          → types.rs (re-exported from myosu-tui)
journal.md format               → AgentSession::journal_append()
```

`PipeClient` does NOT implement narration or game state rendering — that is `myosu-play`'s responsibility. `PipeClient` only parses the text output that `myosu-play` produces.

---

## How the Integration Family Fits Together

```
agent:experience (spec lane) ──────────────────────────────
  └─ defines surfaces: pipe protocol, schema, UX behavior
                                                        │
                      agent-integration (impl lane) ─────┘
                        └─ implements PipeClient + AgentSession
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
   myosu-agent           myosu-agent-           agent:experience
   (binary)              core (library)         implementation slices
```

The separation is intentional: `myosu-agent-core` is a library that any agent implementation (Python, JavaScript, Rust) can depend on. The `myosu-agent` binary is one consumer of that library.

---

## Proof / Check Shape for the Lane

The lane is **proven** when all of the following pass:

```
# PipeClient roundtrips game state through pipe
cargo test -p myosu-agent-core pipe_client::tests::send_action_parses_output
cargo test -p myosu-agent-core pipe_client::tests::reflection_prompt_detected
cargo test -p myosu-agent-core pipe_client::tests::empty_reflection_returns_none
cargo test -p myosu-agent-core pipe_client::tests::lobby_parsed_correctly
cargo test -p myosu-agent-core pipe_client::tests::subnet_selection_starts_game

# AgentSession manages context and journal
cargo test -p myosu-agent-core session::tests::context_persists_across_sessions
cargo test -p myosu-agent-core session::tests::journal_appends_hand_entry
cargo test -p myosu-agent-core session::tests::drop_saves_context

# myosu-agent binary responds to --help
cargo test -p myyosu-agent myosu_agent::tests::help_flag_works
cargo test -p myyosu-agent myosu_agent::tests::autonomous_loop_runs

# myosu-agent-core compiles and types align with myosu-tui
cargo check -p myosu-agent-core
```

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1: `myosu-agent-core` Crate Skeleton + `PipeClient`
**Files**: `crates/myosu-agent-core/Cargo.toml`, `crates/myosu-agent-core/src/lib.rs`, `crates/myosu-agent-core/src/pipe_client.rs`
**What**: Create crate; define `PipeClient::new()` that spawns `myosu-play --pipe` as child; implement `send_action()` parsing; implement `wait_reflection()`; add roundtrip tests with a mock `myosu-play` subprocess.
**Proof gate**: `cargo test -p myosu-agent-core pipe_client::tests::send_action_parses_output`

### Slice 2: `AgentSession` + Context Lifecycle
**Files**: `crates/myosu-agent-core/src/session.rs`, `crates/myosu-agent-core/src/types.rs`
**What**: `AgentSession` wrapping `PipeClient`; context load/save; `act()` and `query_best_action()`; journal append; `Drop` saves context.
**Proof gate**: `cargo test -p myosu-agent-core session::tests::context_persists_across_sessions`

### Slice 3: Lobby and Subnet Selection in `PipeClient`
**Files**: `crates/myosu-agent-core/src/pipe_client.rs`
**What**: `request_lobby()` parsing; `select_subnet(id)` command; `SubnetInfo` type; lobby tests.
**Proof gate**: `cargo test -p myosu-agent-core pipe_client::tests::lobby_parsed_correctly`

### Slice 4: `myosu-agent` Binary
**Files**: `crates/myosu-agent/Cargo.toml`, `crates/myosu-agent/src/main.rs`
**What**: CLI with `--context`, `--narrate`, `--autonomous` flags; `main()` wires flags to `AgentSession`; autonomous loop mode; `--help` output.
**Proof gate**: `cargo test -p myyosu-agent myosu_agent::tests::help_flag_works`

### Slice 5: `HttpAgentClient` Skeleton (Phase 2 — blocked on `chain:runtime`)
**Files**: `crates/myosu-agent-core/src/http_client.rs`
**What**: `HttpAgentClient` struct; miner axon discovery stub; `query_strategy()` HTTP call; `subscribe_events()` WS stream; compilation-only for now.
**Proof gate**: `cargo check -p myosu-agent-core http_client`

---

## Dependency on Other Lanes

| Lane | Type | What Is Used |
|------|------|-------------|
| `agent:experience` | Hard upstream | Pipe protocol contract, `--pipe --context --narrate` flags, reflection prompt format, lobby format, schema |
| `play:tui` | Hard upstream | `myosu-play` binary must exist for `PipeClient` to spawn |
| `games:traits` | Hard upstream | `StrategyQuery/Response` types |
| `chain:runtime` | Soft upstream (Phase 2) | Miner axon endpoint discovery for `HttpAgentClient` |

**Critical dependency**: `play:tui` must produce the `myosu-play` binary (Slice 1 of that lane) before `PipeClient` can be integration-tested end-to-end. Slice 1 can be developed and unit-tested with a mock subprocess; integration testing requires `myosu-play`.

---

## Phase Ordering

```
Phase 1 (Pipe-Based — depends on play:tui binary):
  Slice 1 → Slice 2 → Slice 3 → Slice 4

Phase 2 (HTTP/WS-Based — depends on chain:runtime):
  Slice 5 (skeleton) → full HttpAgentClient implementation
```
