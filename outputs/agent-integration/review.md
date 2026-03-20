# `agent:integration` Lane Review

## Judgment: **KEEP** — proceed to implementation-family workflow

This lane is the natural successor to `agent:experience`. Where `agent:experience` defined the *surfaces* agents interact with (pipe mode, context, journal, narration, lobby, spectator relay), `agent:integration` defines the *wiring* that connects those surfaces to Myosu's core systems without creating coupling between them.

The `AgentAdapter` trait is a thin porcelain layer over existing components — not a new abstraction but a convenient wiring point. The 4 slices are sequential, minimally coupled, and build on proven infrastructure.

---

## Rationale for KEEP

1. **Adapter is a thin layer, not a new architecture**: `AgentAdapter` doesn't introduce new concepts — it wires `PipeMode` to `games:traits`, `AgentContext` to filesystem, `SpectatorRelay` to Unix sockets, and lobby to (stubbed) chain queries. Each method maps to an existing component.

2. **Preserves existing abstractions**: `GameRenderer` trait, `CfrGame`, `Profile`, `StrategyQuery` all remain as defined in `games:traits`. The adapter doesn't replace them — it calls them. downstream consumers (pipe mode, spectator relay) don't need to know about the internal components.

3. **Stub adapter enables testing without full stack**: `StubAgentAdapter` provides a test double with hardcoded responses, enabling `cargo test -p myosu-tui agent_adapter::tests` to run without a live chain, miner, or validator.

4. **Slice coupling is clean**: Slices 1–2 define the trait and one implementation. Slice 3 refactors `PipeMode` to use it. Slice 4 refactors `AgentContext` to use it. Each slice modifies one file (plus tests), and each subsequent slice depends only on the previous slice's output.

5. **No new dependencies**: The lane doesn't introduce new external crates beyond `thiserror` (already used in the codebase). All types (`AgentContext`, `SubnetInfo`, `GameEvent`) are serializable with `serde` (already a workspace dependency).

---

## Proof Expectations

To consider this lane **proven**, the following evidence must be available:

| Proof | How to Verify |
|-------|--------------|
| `AgentAdapter` trait compiles with object safety | `cargo check -p myosu-tui` exits 0; `dyn AgentAdapter` in `PipeMode` compiles |
| `StubAgentAdapter` returns hardcoded context | `cargo test -p myosu-tui agent_adapter::tests::stub_returns_hardcoded_context` passes |
| `MyosuAgentAdapter::load_context()` returns error on missing file | `cargo test -p myosu-tui myosu_agent_adapter::tests::load_context_missing_file_returns_error` passes |
| `MyosuAgentAdapter::save_context()` uses atomic write | temp file + rename pattern; `cargo test -p myosu-tui myosu_agent_adapter::tests::save_context_atomic` passes |
| `PipeMode` holds `Arc<dyn AgentAdapter>` | `cargo test -p myosu-tui pipe::tests::pipe_mode_uses_adapter` passes |
| `AgentContext::load()` delegates to adapter | `cargo test -p myosu-tui agent_context::tests::load_uses_adapter` passes |
| `AgentContext::save()` delegates to adapter | `cargo test -p myosu-tui agent_context::tests::save_uses_adapter` passes |
| Full stack with stub adapter | `cargo test -p myosu-tui integration::tests::agent_plays_hand_with_stub_adapter` passes |
| All existing pipe tests still pass | `cargo test -p myosu-tui pipe::tests` exits 0 |

---

## Remaining Blockers

### 1. `myosu-play` Binary Skeleton (HIGH — blocks Slice 3)

`PipeMode` refactoring (Slice 3) requires the `myosu-play` binary to exist so the refactored `PipeMode` can be instantiated. The binary skeleton is owned by `play:tui` lane.

**Resolution**: `play:tui` lane Slice 1 (binary skeleton) must complete before or concurrently with `agent:integration` Slice 3.

### 2. `robopoker` Git Migration (HIGH — blocks Phase 2+)

Both `tui:shell` and `games:traits` depend on `robopoker` via **absolute filesystem paths**. This is documented in `outputs/play/tui/spec.md` and `outputs/games/traits/review.md` as the highest-priority fix.

**Impact on this lane**: `MyosuAgentAdapter` calls into `games:traits`, which calls into `robopoker`. Until `robopoker` is migrated to a proper git dependency, `cargo build` and `cargo test` will fail on any clean checkout or CI environment.

**Resolution**: `games:traits` lane owns this resolution. This lane can complete Slices 1–4 (trait + stub + refactors) without needing `robopoker` to be git-migrated, because `StubAgentAdapter` provides a test double that doesn't call into `games:traits`. Full integration testing requires the migration.

### 3. `agent_context.rs` Doesn't Exist Yet (MEDIUM — blocks Slice 4)

`agent:experience` Slice 1 is `agent_context.rs` (identity and memory). Slice 4 of this lane refactors `agent_context.rs` to use the adapter. Slice 4 cannot proceed until Slice 1 of `agent:experience` is complete.

**Resolution**: This is a soft ordering constraint, not a hard blocker. Slice 4 can proceed in parallel with `agent:experience` Slice 1 once `agent_context.rs` exists.

### 4. Spectator Relay Not Yet Implemented (LOW — blocks Phase 2)

`SpectatorRelay` (AC-SP-01) is `agent:experience` Slice 8. The `emit_spectator_event()` method in the adapter is a no-op stub until the relay exists.

**Resolution**: Implement `SpectatorRelay` in `agent:experience` Slice 8, then wire it to `MyosuAgentAdapter::emit_spectator_event()` in a follow-up slice.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification | **READY** | 4 slices defined with clear boundaries; `AgentAdapter` trait is thin and non-invasive |
| Upstream (tui:shell) | **READY** | 82 tests pass; `Shell`, `GameRenderer`, `PipeMode` trusted |
| Upstream (games:traits) | **READY** | 14 tests pass; `CfrGame`, `Profile`, `StrategyQuery` trusted |
| Upstream (agent:experience) | **READY** | spec.md + review.md complete; surfaces defined |
| Upstream (play:tui) | **PARTIAL** | Binary skeleton missing; needed for Slice 3 |
| Adapter trait | **DEFINED** | This lane defines it |
| Stub implementation | **DEFINED** | This lane defines it |
| Implementation slices | **DEFINED** | 4 slices, sequential, minimal cross-slice coupling |
| robopoker dependency | **BLOCKER** | Absolute path deps; must migrate to git before Phase 2 integration testing |

---

## Decision: Implementation Family Next

The `agent:experience` review concluded: "**Proceed to implementation-family workflow next.**"

This lane (`agent:integration`) IS that implementation-family workflow. It is the first honest bootstrap slice that moves from specification to code for the agent-facing integration layer.

**The decision is: implementation family.**

The next question is whether to begin with:
1. **This lane (`agent:integration`)** — the adapter layer, which is a prerequisite for `agent:experience` Slices 1–9 to work together
2. **`agent:experience` Slice 1 (`agent_context.rs`)** — which is independent of this lane and could proceed in parallel

**Recommendation**: Run `agent:integration` Slices 1–2 (trait + stub) in parallel with `agent:experience` Slice 1 (`agent_context.rs`). Both are independent at this point — the adapter doesn't need `agent_context.rs` to be defined (it uses `AgentContext` as a type, not an implementation). Once `agent_context.rs` exists, `agent:integration` Slice 4 can refactor it to use the adapter.

---

## Recommendation

**Proceed to implementation-family workflow next.** The lane is well-specified and the upstream is trusted. The primary blocker (`robopoker` git migration) is owned by the `games:traits` lane and does not prevent Slices 1–2 from completing.

Slices 1–2 (trait + stub) can begin immediately and produce tests that validate the adapter contract without needing `play:tui` or `robopoker` to be resolved.

The `review.md` should be updated after each slice completes to track proof availability and remaining blockers.
