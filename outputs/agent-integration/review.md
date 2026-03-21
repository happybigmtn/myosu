# `agent-integration` Lane Review

## Judgment: **KEEP** — proceed to implementation-family workflow

This lane is the natural next step after `agent:experience`. The pipe protocol surfaces are well-specified and the `PipeClient` abstraction is the correct integration primitive. The separation between `myosu-agent-core` (library) and `myosu-agent` (binary) is sound — it allows Python and JavaScript agents to consume the same library without requiring a Rust runtime.

---

## Rationale for KEEP

1. **Clean abstraction boundary**: `PipeClient` is a thin, honest wrapper around `myosu-play --pipe`. It does not try to be clever about game state — it sends actions, receives output, and parses prompts. The complexity of game rendering stays in `myosu-play`; the complexity of agent session management stays in `AgentSession`. This is the right separation.

2. **`agent:experience` surfaces are ready**: The pipe protocol, reflection prompt, lobby format, and `--context`/`--narrate` flags are all defined in `agent:experience`'s reviewed spec. `PipeClient` is a direct translation of that contract into a client library — no ambiguity to resolve.

3. **Library-first design enables polyglot agents**: By keeping `PipeClient` and `AgentSession` in `myosu-agent-core` as a plain Rust library, any language with Rust FFI or process spawning can use it. The `myosu-agent` binary is one consumer. Python and JavaScript agents can use the same library or implement their own `PipeClient` following the same contract.

4. **Slice dependency chain is short**: Slice 1 (`PipeClient`) can be developed with a mock subprocess for unit testing. Integration testing with the real `myosu-play` binary requires `play:tui` Slice 1 (binary skeleton) — but this is a dependency the `play:tui` lane already owns and is working on.

5. **No speculative Phase 2 coupling**: `HttpAgentClient` is explicitly Phase 2 and blocked on `chain:runtime`. The spec defines it as a skeleton now so the `myosu-agent-core` crate layout is correct from the start, but Phase 1 is fully implementable without it.

---

## Proof Expectations

To consider this lane **proven**, the following evidence must be available:

| Proof | How to Verify |
|-------|--------------|
| `PipeClient` sends action and parses output | Spawn mock `cat`-like process; `send_action("fold\n")` returns parsed `GameOutput`; mock echoes correctly formatted text |
| `PipeClient` detects and handles `reflect>` prompt | PipeClient receives "HAND COMPLETE\nreflect>" → `wait_reflection()` returns `Ok(None)` for empty line, `Ok(Some(text))` for non-empty |
| `PipeClient` parses lobby correctly | `request_lobby()` on mock that echoes lobby format → returns `Vec<SubnetInfo>` with correct count and fields |
| `PipeClient` handles subnet selection | `select_subnet(1)` sends "join 1" → returns initial `GameOutput` |
| `AgentSession` persists context across sessions | `AgentSession::new()` → play 5 hands → drop → `AgentSession::new()` → context loaded with correct hand count |
| `AgentSession` appends journal without truncating | `journal_append()` called 100 times → file size grows monotonically; `save()` never truncates |
| `AgentSession::drop()` saves context | Create session; modify context; drop without explicit `save()`; create new session → changes preserved |
| `myosu-agent --help` works | `myosu-agent --help` exits 0 with non-empty stdout describing flags |
| `myosu-agent --autonomous` loop runs | Connect to test pipe; verify loop calls `decide()` callback and sends actions |

---

## Remaining Blockers

### 1. `myosu-play` Binary Does Not Exist (HIGH — blocks Slice 1 integration testing)

`PipeClient::new()` spawns `myosu-play --pipe --context <path> --narrate` as a child process. The `myosu-play` binary does not exist yet — it is owned by `play:tui` Slice 1.

**Impact on this lane**: Slice 1 unit tests can use a mock subprocess. Full integration testing requires the real binary.

**Resolution**: `play:tui` lane owns the binary skeleton. Begin Slice 1 development now; integration testing can proceed once `play:tui` Slice 1 is complete.

### 2. `agent:experience` Slices 3+ Not Yet Implemented (MEDIUM — blocks `--context` and `--narrate` wiring)

`PipeClient` passes `--context` and `--narrate` flags to `myosu-play`. These flags do not exist yet — they are `agent:experience` Slices 3 and 6 respectively.

**Impact on this lane**: `PipeClient::new(context_path, narrate)` will pass flags that `myosu-play` does not yet understand. The CLI will error on unknown flags.

**Resolution for Slice 1**: Use `PipeClient::new(context_path, false)` for initial development (narrate off). Once `agent:experience` Slice 3 lands, add narrate support. Alternatively, `play:tui` Slice 1 could add minimal `--context` support as a pre-slice if that is easier.

### 3. `AgentContext` and `Journal` Types Live in `myosu-tui` (LOW — refactor before Slice 2)

`AgentSession` needs `AgentContext` and `Journal` types. Currently these are defined in `myosu-tui` but are completely absent as implementation (see `agent:experience` spec). They will be implemented as part of `agent:experience` Slices 1 and 2.

**Impact on this lane**: `myosu-agent-core` cannot compile `session.rs` until `AgentContext` and `Journal` exist.

**Resolution**: Once `agent:experience` Slice 1 (`agent_context.rs`) lands, extract those types into a shared location or re-export through `myosu-tui`. `agent-integration` Slice 2 depends on `agent:experience` Slice 1 completing first — this is the correct ordering and is already captured in the `agent:experience` slice chain.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification | **READY** | `agent-adapter.md` defines clean abstraction, 5 slices, clear upstream deps |
| Upstream (`agent:experience`) | **READY** | Pipe protocol, reflection prompt, lobby format all reviewed and stable |
| Upstream (`play:tui`) | **PARTIAL** | Binary missing; needed for integration testing |
| Upstream (`games:traits`) | **READY** | 14 tests pass; `StrategyQuery/Response` types stable |
| Library design | **SOUND** | `myosu-agent-core` as library enables polyglot use; `myosu-agent` as binary provides CLI |
| Slice dependency chain | **CLEAN** | Slice 1 (mockable) → Slice 2 (needs agent:experience Slice 1) → Slice 3 (needs lobby) → Slice 4 (binary) → Slice 5 (Phase 2, blocked) |
| Polyglot agent support | **ENABLED** | Library-first design; no hard Rust-only assumptions |

---

## Recommendation

**Proceed with Slice 1 immediately.** `PipeClient` is the simplest honest first slice — it is a thin wrapper around the pipe protocol and can be developed and unit-tested without any downstream dependencies beyond a mock subprocess.

**Coordinate with `play:tui` on binary skeleton.** `play:tui` Slice 1 should add minimal `--pipe` flag support to `myosu-play` as early as possible, even before full `agent:experience` Slice 3. This unblocks `agent-integration` Slice 1 integration testing.

**Track `agent:experience` Slice 1 dependency for Slice 2.** `AgentSession` cannot be implemented without `AgentContext` from `agent:experience` Slice 1. Monitor that lane's progress and sequence accordingly.

**Do not implement Phase 2 (`HttpAgentClient`) until `chain:runtime` is available.** The skeleton in Slice 5 is fine as a compile-only placeholder. No real work should happen on HTTP/WS integration until miner axon endpoints are available.
