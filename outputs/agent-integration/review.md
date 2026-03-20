# `agent:experience` Lane Review — Integration Assessment

## Judgment: **KEEP — proceed to implementation family, phased by binary availability**

The `agent:experience` lane is sound. The `robopoker` git migration that was the highest-priority cross-lane blocker is already complete (from `games:traits` Slice 1). `tui:shell` and `games:traits` are both trusted. The lane is ready to move from specification into implementation, with a phased slice strategy that respects the `play:tui` binary dependency.

---

## What Changed Since the Bootstrap Review

The bootstrap review (`outputs/agent/experience/review.md`) listed `robopoker` git migration as a **HIGH** blocker affecting all slices. This is now **resolved**:

- `crates/myosu-games/Cargo.toml` uses pinned git dependencies:
  ```
  rbp-core = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }
  rbp-mccfr = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }
  ```
- `cargo fetch` succeeds without a local robopoker clone
- `cargo test -p myosu-games` exits 0

The `play:tui` binary gap remains unresolved — `crates/myosu-play/` is an empty directory. This is the only remaining hard dependency that gates CLI flag wiring (Slices 3, 6, 7, 8, 9).

---

## Honest Assessment of Implementability

The 9 slices fall into three groups:

### Group A — Implementable Now (no binary required)

| Slice | File | Upstream needed | Test command |
|-------|------|-----------------|--------------|
| Slice 1 | `crates/myosu-tui/src/agent_context.rs` | `tui:shell` (trusted) | `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` |
| Slice 2 | `crates/myosu-tui/src/journal.rs` | `tui:shell` (trusted) | `cargo test -p myosu-tui journal::tests::append_hand_entry` |
| Slice 4 | `crates/myosu-tui/src/pipe.rs` (extend) | `tui:shell` (trusted) | `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand` |
| Slice 5 | `crates/myosu-tui/src/narration.rs` | `schema.rs` (trusted) | `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture` |

These four slices can be implemented, tested, and verified without the `myosu-play` binary existing. All required types are in `crates/myosu-tui/src/` and the existing test infrastructure (`cargo test -p myosu-tui`) is sufficient.

### Group B — Blocked on `play:tui` binary (concurrent)

| Slice | Why blocked |
|-------|-------------|
| Slice 3 (`--context` wiring) | Requires `main.rs` CLI dispatch modification |
| Slice 6 (`--narrate` wiring) | Requires `main.rs` CLI dispatch modification |
| Slice 7 (lobby) | Uses same CLI infrastructure; lobby stub can be built before binary but not wired |

`play:tui` lane owns the binary skeleton. This is not an upstream unblock — it is a parallel lane. The `agent:experience` implementation lane can begin Slice 1 while `play:tui` produces the binary skeleton.

### Group C — Phase 3 (spectator relay)

| Slice | Why blocked |
|-------|-------------|
| Slice 8 (`SpectatorRelay`) | `crates/myosu-play/src/spectate.rs` inside non-existent binary |
| Slice 9 (`SpectateScreen`) | Depends on binary existing + `SpectatorRelay` existing |

---

## Decision: Implementation Family Next

**The right move is implementation, not another upstream unblock.**

Rationale:
1. The primary cross-lane blocker (`robopoker` git migration) is already resolved.
2. `tui:shell` (82 tests) and `games:traits` (14 tests) are trusted — the foundation is solid.
3. Four slices (1, 2, 4, 5) are implementable immediately without the binary.
4. The `play:tui` dependency is a parallel lane, not a blocker — both can run concurrently.
5. The spec is mature (AX-01..05 + SP-01..03), the slice boundaries are clean, and the test shapes are defined.

The `agent:integration` recommendation is to spawn an implementation lane that:
1. Starts with Slices 1, 2, 4, 5 immediately (no binary needed)
2. Tracks `play:tui` binary availability as a milestone gate for Slices 3, 6, 7
3. Treats Slices 8, 9 as Phase 3 after both the binary and spectator relay infrastructure exist

---

## Risks the Implementation Lane Must Manage

### Risk 1: `myosu-play` Binary Is Not Yet Created

**Status**: `crates/myosu-play/` is an empty directory. `play:tui` lane has not produced the binary skeleton.

**Impact on this lane**: Slices 3, 6, 7, 8, 9 cannot be wired to a CLI until the binary exists.

**What the implementation lane must do**: Do not attempt to create the binary from this lane. Wait for `play:tui` Slice 1 to complete, then wire Slices 3, 6, 7 against the existing `main.rs` scaffold. Do not hard-code a binary path assumption — use the `myosu-tui` library API from within `myosu-play`.

### Risk 2: `myosu-tui` Schema Types Must Not Leak `robopoker` Internals

**Status**: The narration engine (Slice 5) takes `GameState` from `schema.rs`, not raw `robopoker` types. This is correct.

**Impact on this lane**: If future slices attempt to pass `robopoker` types directly to agent-facing surfaces, the coupling reintroduces the `robopoker` path dependency problem.

**What the implementation lane must do**: Keep `agent_context.rs`, `journal.rs`, and `narration.rs` as pure `myosu-tui` code. If they need game state information, use `schema.rs` types (`GameState`, `GamePhase`, etc.) — never raw `robopoker` types.

### Risk 3: Journal Append Invariant Must Hold Under Concurrent Access

**Status**: The spec says the journal is append-only. The `Journal::append_hand_entry()` must never truncate.

**Impact on this lane**: If the pipe mode loop races with a concurrent reader (e.g., a spectator process reading the journal while the agent is writing), standard file append is not atomic at line boundaries.

**What the implementation lane must do**: Use `std::fs::OpenOptions::append(true)` consistently. Add a test that verifies sequential appends only grow the file. Do not use `BufWriter` with `autoflush` in a way that could cause torn writes. Document that the journal is not a inter-process communication channel.

### Risk 4: Narration Engine Must Not Fabricate Game Information

**Status**: The narration engine translates `GameState` into prose. The prose must be grounded in actual game state — no hallucinated odds, no invented opponent tendencies.

**Impact on this lane**: An LLM consuming narrated output might treat confident-sounding prose as ground truth even when it is a paraphrase of uncertain information.

**What the implementation lane must do**: Every factual claim in narrated prose (e.g., "you have a 34% chance to win") must be computed from the `GameState` fields, not hard-coded strings. Add a test `narrate_same_game_state_in_both_modes` that verifies the same `GameState` produces consistent action distributions in both pipe and narrate modes.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Spec quality | **READY** | AX-01..05 + SP-01..03 are mature; 9 slices defined with clear boundaries |
| Upstream (`tui:shell`) | **READY** | 82 tests pass; `GameRenderer`, `PipeMode` trusted |
| Upstream (`games:traits`) | **READY** | 14 tests pass; git deps in place; robopoker migration complete |
| Upstream (`play:tui`) | **PARTIAL** | Binary missing; blocks Slices 3, 6, 7, 8, 9 |
| Schema (`schema.rs`) | **TRUSTED** | 939 lines, 16 tests, 10 game types |
| `robopoker` dependency | **RESOLVED** | Git deps with pinned rev; `cargo fetch` works |
| Implementation slices | **4 immediately ready** | Slices 1, 2, 4, 5 can start now |
| Spectator relay | **SPEC ONLY** | Not yet implemented; fully specified but no code |

---

## Recommendation

**Spawn the `agent:experience` implementation lane now.** Begin with Slices 1, 2, 4, and 5 in parallel with `play:tui` completing its binary skeleton. Do not wait for the binary before starting — the Group A slices are architecturally isolated and can be implemented, tested, and verified entirely within `myosu-tui`.

The decision to move to implementation is based on:
- The primary cross-lane blocker is resolved
- The spec is mature and stable
- The test surface is well-defined
- The slice grouping respects the binary dependency without blocking progress on the immediately-implementable parts

The `review.md` should be updated after each slice completes to track proof availability and remaining blockers.
