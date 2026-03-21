# `agent:integration` Lane Review

## Judgment: **PROCEED** — implementation-family with staged unblock

The `agent:integration` lane is the honest first reviewed slice for the agent-facing integration frontier. The `agent-adapter.md` correctly identifies that Slices 1–2 (adapter scaffold + journal wiring) can start immediately on trusted upstream (`tui:shell`), while Slices 3–9 require the `myosu-play` binary skeleton from `play:tui`.

The decision is: **proceed with implementation-family workflow**, but stage the work so that slices that can start now do so, while slices blocked on `play:tui` wait for that lane to complete its binary skeleton.

---

## Rationale for PROCEED

1. **Slices 1–2 are unblocked**: The adapter scaffold (`adapter.rs` with `AgentContextHandle`, `JournalHandle`, `NarrationHandle`, `SpectatorHandle`) depends only on the `tui:shell` trusted surface. No `myosu-play` binary is required for these slices. `cargo check -p myosu-tui` already works.

2. **`play:tui` binary skeleton is imminent**: The `play:tui` lane review (at `outputs/play/tui/review.md`) gives it a KEEP judgment and marks it "Ready for Implementation-Family Workflow." Slice 1 of `play:tui` creates the `myosu-play` binary skeleton. Once that exists, Slices 3–9 of `agent:integration` can proceed.

3. **NarrationEngine (Slice 5) can be developed independently**: The `NarrationEngine` is a standalone text-generation engine that does not require any binary wiring. It can be implemented and tested in isolation while `play:tui` Slice 1 is being completed.

4. **Adapter pattern is sound**: The decision to use thin wrapper handles (`AgentContextHandle`, `JournalHandle`, etc.) that delegate to domain types keeps the integration layer minimal. This matches the pattern used in other lanes (`games:traits` thin re-export, `tui:shell` trait-based).

5. **No design ambiguity remains**: The `agent-adapter.md` specifies exact types (`adapter.rs`, `pipe_integration.rs`, CLI flag shapes), integration points (where `PipeMode` is extended), and slice sequencing. An implementer can start Slice 1 without further clarification.

---

## Decision: Implementation Family Next

Based on the `agent:experience` review recommendation ("Proceed to implementation-family workflow next") and this lane's honest blocker assessment, the product should **proceed with the implementation-family workflow** for the agent surfaces.

The staged approach:
- **Immediately**: Start `agent:integration` Slice 1 (adapter scaffold) + Slice 2 (journal wiring)
- **After `play:tui` Slice 1**: Start `agent:integration` Slice 3 (--context wiring), Slice 4 (reflect>), Slice 6 (--narrate), Slice 7 (lobby), Slice 8 (spectator emit), Slice 9 (spectate screen)
- **In parallel**: Develop `NarrationEngine` (Slice 5) as a standalone crate/module

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Adapter pattern defined | **READY** | Thin handles, delegating to domain types |
| Slice 1 (adapter scaffold) | **READY TO START** | Depends only on `tui:shell` (trusted) |
| Slice 2 (journal wiring) | **READY TO START** | Depends on Slice 1 |
| Slice 3 (--context wiring) | **BLOCKED** | Waiting on `play:tui` binary skeleton |
| Slice 4 (reflect> prompt) | **BLOCKED** | Waiting on Slice 3 |
| Slice 5 (NarrationEngine) | **READY TO START** | Standalone; no wiring dependency |
| Slice 6 (--narrate wiring) | **BLOCKED** | Waiting on Slice 5 + `play:tui` |
| Slice 7 (lobby) | **BLOCKED** | Waiting on `play:tui` + chain stub |
| Slice 8 (spectator emit) | **BLOCKED** | Waiting on `SpectatorRelay` (Slice 8 of 9) |
| Slice 9 (spectate screen) | **BLOCKED** | Waiting on Slice 8 |
| Upstream (`tui:shell`) | **TRUSTED** | 82 tests pass |
| Upstream (`games:traits`) | **TRUSTED** | 14 tests pass |
| Upstream (`play:tui`) | **READY SOON** | Binary skeleton imminent |

---

## Concrete Risks to Preserve

### Risk 1: Adapter Handles Must Not Reinvent Serialization

**Location**: `crates/myosu-tui/src/adapter.rs` (will be created in Slice 1)

The adapter handles should be **thin wrappers** that delegate to domain types (`AgentContext`, `Journal`, `NarrationEngine`, `SpectatorRelay`). They must not reimplement serialization, game logic, or text generation.

**Preserve**: Handles only manage lifetime, initialization, and dispatch. All domain logic stays in the domain types.

**Verify**: `cargo check -p myosu-tui` after Slice 1; domain types remain the serialization boundary.

### Risk 2: `--narrate` Flag Is No-Op Until Slice 5

**Location**: `crates/myosu-tui/src/pipe.rs` (after Slice 3)

The `--narrate` flag can be wired in Slice 3, but `NarrationHandle::narrate()` returns `pipe_output()` until Slice 5 implements `NarrationEngine`. This is intentional.

**Preserve**: Do not require `NarrationEngine` to exist before wiring the flag. The flag is a valid no-op.

**Verify**: `myosu-play --pipe --narrate` compiles and runs without panic; output is standard pipe output (not narration).

### Risk 3: SpectatorHandle Is Fire-and-Forget

**Location**: `crates/myosu-tui/src/adapter.rs::SpectatorHandle`

`spectate.rs::emit()` must not block or error if no spectator is connected. Agent play must succeed even if spectator monitoring is unavailable.

**Preserve**: Drop events silently when no listener is connected. Never fail agent play due to spectator unavailability.

**Verify**: Run `myosu-play --pipe` with no spectator connected; play succeeds; no error logged for missing spectator.

### Risk 4: Context File Never Exposed to Opponents

**Location**: `crates/myosu-tui/src/agent_context.rs` (Slice 1)

The context file loaded via `--context` must never appear in opponent-visible outputs (pipe output, logs, chain submissions). It is loaded at startup and saved at shutdown only.

**Preserve**: `AgentContext` is held in memory; never serialized to output streams or opponent-visible state.

**Verify**: Run with adversarial context file (extremely large journal, malicious content); opponent output is unaffected; context is loaded and saved correctly.

---

## Proof Expectations

### Required Proof Commands (in order)

```bash
# Proof 1: Adapter scaffold compiles
cargo check -p myosu-tui

# Proof 2: Agent context roundtrips (Slice 1)
cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip
cargo test -p myosu-tui agent_context::tests::missing_context_creates_new
cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites

# Proof 3: Journal wiring (Slice 2)
cargo test -p myosu-tui journal::tests::append_hand_entry
cargo test -p myosu-tui journal::tests::never_truncates

# Proof 4: Narration engine compiles (Slice 5)
cargo check -p myosu-tui --features narration

# Proof 5: --pipe with --context compiles (after play:tui binary, Slice 3)
cargo check -p myosu-play --features pipe

# Proof 6: --pipe with --narrate compiles (after Slice 5+6)
cargo check -p myosu-play --features narration
```

### Integration Proof (after all slices)

```bash
# Agent plays 10 hands, shuts down, restarts → memory preserved
myosu-play --pipe --context ./test-context.json
# ... play 10 hands ...
# Ctrl+D
# Restart
myosu-play --pipe --context ./test-context.json
# Agent identity + journal preserved

# Narration mode produces prose
myosu-play --pipe --context ./test-context.json --narrate
# Output contains narrative prose, not key-value pairs

# Reflection prompt appears after hand
myosu-play --pipe --context ./test-context.json
# After each hand: HAND COMPLETE + reflect>
# Empty line skips; non-empty is appended to journal

# Lobby presented without --subnet
myosu-play --pipe --context ./test-context.json
# MYOSU/LOBBY output appears
```

---

## Remaining Blockers

| Blocker | Severity | Owner | Resolution |
|---------|----------|-------|------------|
| `myosu-play` binary skeleton missing | **HIGH** | `play:tui` lane | Slice 1 of `play:tui` |
| `robopoker` git migration unresolved | **HIGH** (CI) | `games:traits` lane | Slice 1 of `games:traits` |
| Chain discovery for lobby | **MEDIUM** | `chain:runtime` lane | Stub lobby with hardcoded data for Phase 0 |
| Spectator socket path convention | **LOW** | Verify with `play:tui` | Confirm before Slice 8 |

---

## Recommendation

**Proceed immediately with `agent:integration` Slice 1 (adapter scaffold).** This slice is unblocked and depends only on the already-trusted `tui:shell` surface.

Simultaneously, `play:tui` can proceed with its Slice 1 (binary skeleton). Once that exists, `agent:integration` Slices 3–9 can proceed in the sequence defined in `agent-adapter.md`.

The `NarrationEngine` (Slice 5) can be developed in parallel as a standalone module, as it has no wiring dependencies.

The `review.md` should be updated as each slice completes to track proof availability and remaining blockers.

---

## File Reference Index

| File | Role |
|------|------|
| `outputs/agent-integration/agent-adapter.md` | Integration adapter specification |
| `outputs/agent/experience/spec.md` | Product specification for agent experience |
| `outputs/agent/experience/review.md` | Product review |
| `outputs/play/tui/review.md` | `play:tui` lane review (binary skeleton owner) |
| `outputs/games/traits/review.md` | `games:traits` lane review (robopoker migration owner) |
| `crates/myosu-tui/src/pipe.rs` | PipeMode driver (extend with adapter handles) |
| `crates/myosu-tui/src/shell.rs` | Shell (trusted upstream) |
| `crates/myosu-tui/src/renderer.rs` | GameRenderer trait (trusted) |
| `crates/myosu-tui/src/schema.rs` | GameState schema (trusted) |
| `fabro/run-configs/` | Run configs for this lane (to be created) |
| `fabro/workflows/` | Workflow graphs for this lane (to be created) |
| `fabro/programs/myosu-product.yaml` | Raspberry program manifest (to add `agent:integration`) |
