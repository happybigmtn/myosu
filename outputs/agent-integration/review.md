# `agent-integration` Lane Review

## Judgment: **KEEP** — spec is sound, proceed to implementation-family workflow

The `agent-integration` spec is well-bounded, correctly identifies the gap between `agent:experience` (presentation) and the miner/validator infrastructure (execution), and defines a clean trait hierarchy (`AgentAdapter`, `ChainDiscovery`, `SessionManager`) that can be implemented incrementally. The dependency chain on `agent:experience` and `games:traits` is correct.

---

## Rationale for KEEP

1. **Spec correctly identifies the gap**: `agent:experience` specifies the presentation surfaces (pipe output, narration, journal, reflection, spectator relay) but says nothing about how an agent actually connects to a miner, discovers subnets, or manages sessions. `agent-adapter.md` fills exactly this gap without overlapping with either `agent:experience` or `games:traits`.

2. **Trait hierarchy is sound and minimal**: `AgentAdapter` as a `Send + Sync` trait object with four transport variants is the right abstraction — it allows transport swapping at runtime (important for testing vs production) without coupling the agent logic to any specific transport. The error taxonomy (`AdapterError`) is specific and actionable, not a generic catch-all.

3. **Phase ordering is honest**: Phase 1 (trait + two adapters) has no external dependencies beyond already-trusted upstream. Phase 3 (real chain discovery) correctly defers to `chain:runtime`. Phase 4 (spectator integration) correctly defers to `agent:experience` Slice 8 (SpectatorRelay) which does not exist yet.

4. **Session recovery design is correct**: The decision that recovery is local-only (the miner is NOT re-notified of missed hands) is the right tradeoff — it keeps the miner interface simple and correct while allowing agents to resume their own state.

5. **10 slices, sequentially ordered with clear gates**: Each slice has a specific file target and a specific proof gate. The dependency chain is linear within each phase and minimal cross-phase coupling.

---

## Decision: Implementation-Family Workflow

**The product needs an implementation-family workflow next**, not another upstream unblock.

Reasoning:
- `agent:experience` is already reviewed (KEEP, 2026-03-19). Its `review.md` explicitly says "proceed to implementation-family workflow next."
- `games:traits` is trusted (14 tests pass). No upstream unblock from that direction.
- `tui:shell` is trusted (82 tests pass).
- The only remaining upstream dependency for `agent:experience` implementation is the `myosu-play` binary skeleton (owned by `play:tui` lane, Slice 1). But `agent:experience` Slices 1-2 (agent_context.rs, journal.rs) can proceed immediately — they depend only on `tui:shell` which is trusted.
- `agent-integration` can proceed with Phase 1 (trait + PipeAdapter + HttpAdapter) immediately, as the trait definitions require only the types from `games:traits` which are already trusted.

**Conclusion**: The honest answer to "does product need an implementation family next or another upstream unblock" is: **implementation family next**. The upstream blockers are owned by other lanes and do not prevent the implementation team from starting Phase 1 work.

---

## Proof Expectations

To consider this lane **proven**, the following evidence must be available:

| Proof | How to Verify |
|-------|--------------|
| `AgentAdapter` trait compiles with all four transports | `cargo check -p myosu-tui --lib` exits 0 |
| `PipeAdapter` subprocess lifecycle works | Subprocess starts, communicates via stdin/stdout, cleans up on drop |
| `HttpAdapter` encodes/decodes `StrategyQuery`/`StrategyResponse` | `cargo test -p myosu-tui agent::adapter::tests::http_query_roundtrip` |
| `DevnetDiscovery` returns valid subnet info | `cargo test -p myosu-tui agent::discovery::tests::devnet_stub_returns_subnets` |
| `SessionManager` context roundtrips to JSON file | `cargo test -p myosu-tui agent::session::tests::context_roundtrip` |
| `myosu-agent` CLI binary wires all flags | `myosu-agent --help` shows `--pipe`, `--http`, `--ws`, `--subnet`, `--context` |
| Phase 3 (real chain discovery) against live devnet | Connects to Substrate RPC, returns actual subnet info |

---

## Remaining Blockers

### 1. `myosu-play` Binary Does Not Exist (HIGH — blocks Slice 2, 7)

`PipeAdapter` relies on `myosu-play --pipe` as a subprocess. The binary skeleton does not exist yet (owned by `play:tui` lane, Slice 1 of that lane).

**Impact on this lane**: `PipeAdapter` cannot be integration-tested until the binary exists. HTTP adapter can be tested with a mock server independently.

**Resolution**: `play:tui` lane Slice 1 (binary skeleton) must complete before `PipeAdapter` integration testing. Unit tests for `PipeAdapter` can proceed with a mock subprocess.

### 2. `SpectatorRelay` Does Not Exist (MEDIUM — blocks Slice 10)

`agent:experience` Slice 8 (SpectatorRelay, AC-SP-01) is not yet implemented. Slice 10 of this lane depends on it for spectator integration.

**Impact on this lane**: Slice 10 cannot proceed until the spectator relay exists. All other slices (1-9) are unblocked.

**Resolution**: `agent:experience` implementation must complete Slice 8 before `agent:integration` Slice 10. No action needed from this lane yet.

### 3. `robopoker` Git Migration Unresolved (LOW — indirect)

`games:traits` has absolute path dependencies on robopoker (`/home/r/coding/robopoker/...`). This is owned by `games:traits` lane. This lane depends on `games:traits` types (trusted via tests) but does not depend on robopoker directly.

**Impact on this lane**: Indirect. If `games:traits` cannot build, this lane cannot build. But this is a `games:traits` problem, not this lane's problem.

**Resolution**: `games:traits` lane owns this. This lane should not wait on it.

### 4. Real Chain Discovery Depends on `chain:runtime` (ACCEPTED DEFERRAL)

Phase 3 (`SubstrateDiscovery`) correctly defers to `chain:runtime`. This is not a blocker — it is an explicit Phase 3 deliverable.

**Resolution**: No action needed. Phase 1 and 2 (local integrations with devnet stubs) can proceed entirely without chain runtime.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification | **READY** | `agent-adapter.md` is sound; trait hierarchy is minimal and correct |
| Upstream (agent:experience) | **READY** | Reviewed KEEP; presentation surfaces are defined |
| Upstream (games:traits) | **TRUSTED** | 14 tests pass; `StrategyQuery`/`StrategyResponse` types are stable |
| Upstream (tui:shell) | **TRUSTED** | 82 tests pass |
| Implementation slices | **DEFINED** | 10 slices, sequential, minimal cross-slice coupling |
| Transport implementations | **SPEC ONLY** | `PipeAdapter`, `HttpAdapter`, `WsAdapter`, `ProcessAdapter` not yet coded |
| Chain discovery (real) | **SPEC ONLY** | Phase 3 deferral; devnet stub is Phase 1 |
| Session recovery | **SPEC ONLY** | SessionManager not yet coded |
| CLI binary | **MISSING** | `myosu-agent` binary does not exist |
| Play binary (for PipeAdapter) | **BLOCKER** | `myosu-play` binary owned by `play:tui` lane |
| SpectatorRelay (for Slice 10) | **BLOCKER** | Owned by `agent:experience` Slice 8 |

---

## Concrete Risks the Implementation Lane Must Preserve or Reduce

### Risk 1: Adapter Trait Object Safety
**Location**: `crates/myosu-tui/src/agent/adapter.rs`

`AgentAdapter` is a `dyn` trait object. All implementations must be `Send + Sync` to satisfy the trait bounds. Incorrectly implementing `Send` or `Sync` on an adapter will surface as a compile error (not a silent bug), which is the correct behavior.

**What must be preserved**:
- `Send + Sync` bounds on the trait
- `Box<dyn AgentAdapter>` as the runtime-polymorphic handle

**What must be reduced**:
- No blocking I/O inside `query_strategy` — all adapters must use async I/O internally so the session manager can compose them without blocking

### Risk 2: Miner Timeout Amplification
**Location**: `crates/myosu-tui/src/agent/adapter.rs` — `HttpAdapter`

A slow or unreachable miner causes `query_strategy` to block the session. If multiple agents query the same miner simultaneously, timeout cascades can freeze gameplay.

**What must be preserved**:
- Per-request timeout (5s default, configurable)
- `AdapterError::Timeout` variant for observability

**What must be reduced**:
- Retry logic should be explicit and bounded (1x retry, not infinite loop)
- Session manager should track which miners are unhealthy and avoid them

### Risk 3: StrategyResponse Validation
**Location**: `crates/myosu-tui/src/agent/adapter.rs` — response decoding

Miner axons return untrusted `StrategyResponse` JSON. The adapter must validate the response schema before constructing `ActionDistribution`. A malicious or buggy miner must not be able to panic the agent.

**What must be preserved**:
- Schema validation on all `StrategyResponse` inputs (serde-validated, not just type-checked)
- `AdapterError::InvalidResponse` for observability

**What must be reduced**:
- No `unwrap()` or `expect()` on miner response fields
- Use `serde_json::value::Map` access with explicit key extraction rather than deserializing directly into domain types

### Risk 4: Context File Security
**Location**: `crates/myosu-tui/src/agent/session.rs` — `save_context` / `load_context`

The context file contains session state including miner endpoints and hand history. It must not be exposed to other agents or stored in a world-readable location.

**What must be preserved**:
- Context file permissions: `0o600` (owner read/write only)
- No sensitive data (private keys) stored in context file

**What must be reduced**:
- Session IDs should be unguessable (UUID v4, not sequential)
- Context file path should be configurable, not hardcoded

---

## Proof Commands

| Context | Command | Expected |
|---------|---------|----------|
| Trait compiles (no implementations) | `cargo check -p myosu-tui --lib` | Exit 0, no errors about missing `query_strategy` |
| All transports compile | `cargo check -p myosu-tui --lib` | Exit 0 |
| Unit tests (no external deps) | `cargo test -p myosu-tui agent::` | Exit 0, all tests pass |
| Integration test (requires `myosu-play`) | `cargo test -p myosu-tui agent::adapter::tests::pipe_lifecycle` | Exit 0 (skipped if binary missing) |
| CLI binary | `myosu-agent --help` | Shows all flags |

---

## Recommendation

**Proceed to implementation-family workflow.** The `agent:integration` spec is sound and the upstream is trusted. The implementation lane can begin with Phase 1 (trait + `PipeAdapter` + `HttpAdapter`) immediately. `SessionManager` and `ChainDiscovery` devnet stub can proceed in parallel since they have no external dependencies.

The primary cross-lane dependency is `play:tui` Slice 1 (binary skeleton) for `PipeAdapter` integration testing. This should be tracked as a blocking item in the program manifest but should not prevent the implementation team from starting with Phase 1 unit tests.

Slice 10 (spectator integration) should be explicitly marked as blocked on `agent:experience` Slice 8 in the lane dependency graph.
