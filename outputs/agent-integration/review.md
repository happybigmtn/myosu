# `agent-integration` Lane Review

## Judgment: **KEEP** — Phased implementation recommended

**Recommendation**: Proceed to implementation-family workflow for `agent:experience` Phase 0 (Slices 1–2) immediately. Defer Phase 1+ (Slices 3–9) until `play:tui` Slice 1 (`myosu-play` binary skeleton) is complete. Treat the `robopoker` git migration as a critical shared blocker that should be resolved before any lane attempts CI-based test runs.

---

## Rationale for KEEP

1. **`agent:experience` spec is sound**: The 9 slices are correctly ordered, dependencies are accurately traced, and the trusted/missing boundary is honest. The schema (`schema.rs`, `game-state.json`) is the strongest surface and is already trusted.

2. **Phase 0 is genuinely unblocked**: Slices 1 (`agent_context.rs`) and 2 (`journal.rs`) depend only on `tui:shell` (82 tests, trusted). They do not require the `myosu-play` binary, the `SpectatorRelay`, or any chain-connected surface. These two slices can begin implementation immediately.

3. **The binary dependency is real, not theoretical**: Every flag surface (`--context`, `--narrate`, `--spectate`) and every Phase 1+ slice requires `myosu-play` to exist. Implementing Slices 3–9 without the binary would produce code that cannot be exercised or tested. This is not a theoretical concern — it is the current state of the integration surface.

4. **`robopoker` git migration is a shared critical blocker**: Both `games:traits` and `tui:shell` (hence `agent:experience`) depend on robopoker via absolute filesystem paths. This blocks CI for any lane that compiles against these crates. Resolving it is a prerequisite for clean test runs across all product lanes.

5. **`play:tui` is the immediate critical path**: The `myosu-play` binary is the vehicle for every agent-facing surface. Until it exists, `agent:experience` Phase 1+ cannot be implemented, wired, or tested. `play:tui` is already specified and ready to build — its Slice 1 is the next gate.

---

## Proof Expectations

### For `agent:experience` Phase 0 (Slices 1–2)

These are achievable now without `myosu-play`:

```
# Slice 1: agent_context.rs
cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip
cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites
cargo test -p myosu-tui agent_context::tests::missing_context_creates_new

# Slice 2: journal.rs
cargo test -p myosu-tui journal::tests::append_hand_entry
cargo test -p myosu-tui journal::tests::append_session_summary
cargo test -p myosu-tui journal::tests::never_truncates
```

### For `agent:experience` Phase 1+ (Slices 3–9)

These require `myosu-play` binary to exist first:

```
# All slices require:
cargo build -p myosu-play

# Slice 3+: flag wiring
cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand
cargo test -p myosu-tui pipe::tests::empty_reflection_skips

# Slices 5–6: narration
cargo test -p myosu-tui narration::tests::narrate_includes_board_texture
cargo test -p myosu-tui narration::tests::narrate_includes_session_context

# Slices 8–9: spectator
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
```

---

## Remaining Blockers

### 1. `myosu-play` Binary Does Not Exist

**Severity**: CRITICAL — blocks Slices 3–9

`play:tui` Slice 1 (binary skeleton) is the gate for `agent:experience` Phase 1+. The `myosu-play` binary is the dispatch vehicle for `--pipe`, `--context`, `--narrate`, and `--spectate`. Without it, flag wiring cannot be implemented or tested.

**Resolution**: `play:tui` Slice 1 must complete first. This is the **critical path**.

**Evidence**: `outputs/play/tui/spec.md` confirms binary is MISSING. `outputs/agent/experience/spec.md` confirms Slices 3–9 require binary dispatch.

### 2. `robopoker` Git Migration Not Complete

**Severity**: CRITICAL — blocks CI for all lanes

Absolute filesystem paths (`/home/r/coding/robopoker/crates/...`) in `myosu-games` and `myosu-tui` prevent clean checkout builds and CI runs.

**Resolution**: `games:traits` lane owns this. Must be resolved before any lane attempts `cargo test` in CI.

**Evidence**: `outputs/games/traits/review.md` documents this as highest-priority Slice 1 fix.

### 3. `myosu-games-poker` Crate Does Not Exist

**Severity**: HIGH — blocks integration testing of pipe output for NLHE

`agent:experience` relies on `GameRenderer::pipe_output()` — the contract is defined, but the NLHE concrete implementation is missing. Without `myosu-games-poker`, the pipe output contract is unproven for actual poker.

**Resolution**: `play:tui` Slice 2 must complete to create the NLHE renderer.

**Evidence**: `outputs/play/tui/spec.md` confirms `myosu-games-poker/` is MISSING.

### 4. `SpectatorRelay` Socket Path Convention Not Verified

**Severity**: LOW — blocks Slice 8

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock`. This convention should be verified against `play:tui`'s data directory convention before Slice 8 implementation.

**Resolution**: Confirm `play:tui` data directory convention in Slice 1 or 2 of `play:tui`.

---

## Lane Readiness Assessment

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | 9 slices defined; AX-01..05 + SP-01..03 are sound |
| Phase 0 (Slices 1–2) | **READY TO PROCEED** | No external dependencies beyond trusted `tui:shell` |
| Phase 1+ (Slices 3–9) | **BLOCKED** | Requires `myosu-play` binary (owned by `play:tui` Slice 1) |
| `robopoker` git migration | **BLOCKER** | Shared critical blocker; owned by `games:traits` |
| `myosu-games-poker` | **BLOCKED** | Required for pipe output integration testing; owned by `play:tui` Slice 2 |
| `chain:runtime` | **NOT YET NEEDED** | Soft blocker for Phase 2 only (lobby + spectator WS); Phase 0 does not require it |

---

## Decision: Implementation-Family Workflow, Phased

### Immediate (Phase 0)

**Proceed** with `agent:experience` Slices 1–2:
- `agent_context.rs` — `AgentContext` with load/save/default, journal append
- `journal.rs` — append-only markdown writer with hand entry formatting

These can be implemented and tested against the trusted `tui:shell` (82 tests). No `myosu-play` binary required.

### After `play:tui` Slice 1 (Binary Skeleton)

**Proceed** with `agent:experience` Slices 3–7:
- Slice 3: `--context` flag wiring in `PipeMode`
- Slice 4: `reflect>` prompt after hand
- Slice 5: `narration.rs` — rich prose engine
- Slice 6: `--narrate` flag wiring
- Slice 7: Lobby + game selection in pipe mode

### After `play:tui` Slice 2 (NLHE Renderer)

**Proceed** with `SpectatorRelay` (Slice 8) and `SpectateScreen` (Slice 9).

### Parallel Track: `robopoker` Git Migration

This should proceed **in parallel** with all of the above. It does not block Phase 0 implementation, but it blocks CI. The `games:traits` lane owns this resolution.

---

## What This Lane Concludes

The `agent:experience` lane is **specification-ready and implementation-ready in two phases**:

- Phase 0: Start now. No blockers.
- Phase 1+: Wait for `play:tui` Slice 1. The binary must exist before flag wiring can be tested.

The single most important shared action is the `robopoker` git migration — it unblocks CI for every product lane simultaneously.

**The product does not need another upstream spec or planning lane. It needs:**
1. **`games:traits`** to resolve the `robopoker` git migration (unblocks CI)
2. **`play:tui`** to complete Slice 1 (unblocks `agent:experience` Phase 1+)
3. **`agent:experience`** to proceed with Slices 1–2 now, Slices 3–9 after `play:tui` Slice 1

The `agent-integration` lane's job is done. The decision is made: phased implementation, not another upstream unblock.
