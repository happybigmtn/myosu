# `agent-integration` Lane Review

## Judgment: **KEEP** — proceed to implementation-family workflow, run INT-1 and INT-2 immediately

The `agent-integration` lane is the honest first slice of the `agent:experience` implementation family. The reviewed spec (`outputs/agent/experience/spec.md`, judgment: KEEP) provides a clear 9-slice roadmap. The integration adapter (`agent-adapter.md`) maps those slices to concrete files, trait signatures, and CLI wiring. Slices 1–4 (INT-1 through INT-4) have no external dependencies beyond the already-trusted `tui:shell` and `games:traits` upstreams. The remaining slices (INT-5 through INT-9) depend on `play:tui` binary skeleton or chain stubs, but those are owned by other lanes and do not block INT-1/INT-2 from starting now.

---

## Rationale for KEEP

### 1. The spec is honest and the integration surface is well-bounded

`agent-adapter.md` maps all 9 `agent:experience` slices to specific files with concrete type signatures. The integration gaps identified (context directory convention, session ID generation, GameEvent vs GameState type separation) are all resolvable without further spec work. The adapter is ready to drive implementation.

### 2. INT-1 and INT-2 have no real upstream blockers

`agent_context.rs` (INT-1) depends only on `serde` and `serde_json` — no `tui:shell`, no `games:traits`, no `play:tui`. It is a pure data-structure + persistence module. `journal.rs` (INT-2) depends only on `std::io` and `chrono`. Both can be implemented, compiled, and tested in isolation today.

### 3. The `robopoker` git migration is a testing blocker, not an implementation blocker

`robopoker` absolute path deps prevent `cargo test -p myosu-tui` from running in CI or on a clean checkout. However, for INT-1 and INT-2 specifically:
- The new modules (`agent_context.rs`, `journal.rs`) have no `robopoker` dependencies
- They can be implemented against the existing `myosu-tui` crate surface
- Full integration tests require `robopoker` resolved, but unit tests for the new modules do not

### 4. `play:tui` binary skeleton is owned by another lane

The `myosu-play` binary skeleton (Slice 1 of `play:tui`) is the responsibility of the `play:tui` lane. INT-3 (--context flag wiring) depends on it, but INT-1 and INT-2 do not. This lane should not duplicate that work.

### 5. The decision from `agent:experience` review is clear

The reviewed judgment is: "Proceed to implementation-family workflow next." The `agent-integration` lane is that workflow. The question is not whether to proceed but how to sequence the slices. The honest answer: start with the slices that have no external blockers.

---

## What `agent-integration` Owns vs. What Other Lanes Own

### This lane owns:

| What | Where | Evidence |
|------|-------|----------|
| `agent_context.rs` | `crates/myosu-tui/src/` | New file; INT-1 |
| `journal.rs` | `crates/myosu-tui/src/` | New file; INT-2 |
| `--context` flag wiring | `crates/myosu-play/src/main.rs` + `pipe.rs` | INT-3 |
| `reflect>` prompt | `pipe.rs` | INT-4 |
| `narration.rs` | `crates/myosu-tui/src/` | INT-5 |
| `--narrate` flag wiring | `crates/myosu-play/src/main.rs` + `pipe.rs` | INT-6 |
| Spectator relay | `crates/myosu-play/src/spectate.rs` | INT-8 |
| Spectator screen | `crates/myosu-tui/src/screens/spectate.rs` | INT-9 |
| Integration adapter | `outputs/agent-integration/agent-adapter.md` | This document |

### Other lanes own (do not implement here):

| What | Owner lane | Blocker for |
|------|------------|-------------|
| `robopoker` git migration | `games:traits` | Full integration testing (all slices) |
| `myosu-play` binary skeleton | `play:tui` | INT-3, INT-6, INT-7, INT-8, INT-9 |
| `pipe.rs` existing `--pipe` logic | `tui:shell` | Existing contract |
| `schema.rs` | `tui:shell` | Already trusted; 16 tests pass |
| Lobby chain queries (stubbed for Phase 0) | `chain:runtime` (future) | INT-7 |
| Miner axon WebSocket upgrade | Future work | Spectator Phase 1 |

---

## Blocker Analysis

### 1. `robopoker` Git Migration — HIGH (testing impact, not implementation)

**Owned by:** `games:traits` lane, Slice 1

The absolute path dependencies in `crates/myosu-games/Cargo.toml` (`path = "/home/r/coding/robopoker/..."`) prevent `cargo test -p myosu-tui` from passing on a clean checkout or in CI.

**Impact on this lane:** New modules INT-1 and INT-2 have no `robopoker` dependencies, so they can be implemented and unit-tested now. Full integration tests (running the pipe mode end-to-end) will fail until `robopoker` is migrated.

**Resolution path:** `games:traits` lane is already working on this (Slice 1 of `myosu-games-traits-implementation.yaml`). Track there.

### 2. `myosu-play` Binary Skeleton — HIGH (blocks INT-3+)

**Owned by:** `play:tui` lane, Slice 1

`INT-3` (--context flag wiring), `INT-6` (--narrate flag), `INT-7` (lobby), `INT-8` (spectator relay), and `INT-9` (spectator screen) all require the `myosu-play` binary to exist and accept CLI flags.

**Impact on this lane:** INT-1 and INT-2 are unblocked. INT-3+ are blocked until the binary skeleton exists.

**Resolution path:** `play:tui` lane owns this. Until it completes, this lane focuses on INT-1 and INT-2.

### 3. Lobby Chain Discovery Stub — MEDIUM (blocks INT-7 only)

**Can be stubbed for Phase 0** without blocking other slices. The `agent-adapter.md` specifies the stub approach: hardcoded subnet data for Phase 0, real chain queries for Phase 4.

### 4. Spectator Socket Path Convention — LOW (confirm before INT-8)

The `agent:experience` review notes the spectator socket path (`~/.myosu/spectate/<session_id>.sock`) should be confirmed against `play:tui`'s data directory convention. The adapter already uses `~/.myosu/spectate/` which aligns with the stated `{data-dir}/hands/` convention from `play:tui`. No change expected; confirm before INT-8.

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Integration adapter | **READY** | `agent-adapter.md` maps all 9 slices with concrete signatures |
| INT-1 (`agent_context.rs`) | **READY TO START** | No upstream deps beyond serde/chrono |
| INT-2 (`journal.rs`) | **READY TO START** | No upstream deps |
| INT-3 (`--context` wiring) | **BLOCKED** | Waiting on `play:tui` binary skeleton |
| INT-4 (`reflect>`) | **BLOCKED** | Waiting on INT-3 |
| INT-5 (`narration.rs`) | **READY TO START** | Only depends on `games:traits` + `tui:shell` (trusted) |
| INT-6 (`--narrate` wiring) | **BLOCKED** | Waiting on INT-3 + INT-5 |
| INT-7 (lobby) | **BLOCKED** | Waiting on INT-3 + chain stub |
| INT-8 (spectator relay) | **READY TO START** | `schema.rs` is trusted; no binary dep |
| INT-9 (spectator screen) | **BLOCKED** | Waiting on INT-8 + binary skeleton |
| `robopoker` git migration | **BLOCKER** | Owned by `games:traits`; testing impact only for INT-1/2 |
| `myosu-play` binary skeleton | **BLOCKER** | Owned by `play:tui`; blocks INT-3+ |

---

## The Decision

### The `agent:experience` lane verdict was: **proceed to implementation-family workflow**.

The `agent-integration` lane is the honest first slice of that workflow. The honest assessment is:

1. **INT-1 and INT-2 can start immediately.** `agent_context.rs` and `journal.rs` are pure data-structure + persistence modules. They compile against the `myosu-tui` surface (which is trusted) but have no external dep on `robopoker`, `play:tui` binary, or chain. Start these now.

2. **INT-5 can also start immediately.** `narration.rs` depends on `games:traits` and `tui:shell` which are trusted, and on `schema.rs` which is trusted. It does not need the binary skeleton. Start this in parallel with INT-1/INT-2.

3. **INT-3+ must wait for `play:tui` binary skeleton.** This is owned by the `play:tui` lane. Do not duplicate that work here. When `play:tui` Slice 1 is done, INT-3+ can proceed without re-deciding anything.

4. **`robopoker` git migration is a tracking concern, not a gating concern for INT-1/INT-2.** It must be resolved before any slice can be fully verified with integration tests, but it does not prevent writing the code.

**Bottom line:** Begin the implementation-family workflow. Run INT-1, INT-2, and INT-5 in parallel now. Monitor `play:tui` binary skeleton completion. When ready, run INT-3 → INT-4 → INT-6 → INT-7 → INT-8 → INT-9 sequentially.

---

## Recommendation

**Proceed to implementation-family workflow.** The `agent-integration` lane has a clear adapter, unblocked early slices, and a honest assessment of what depends on other lanes. Track `play:tui` binary skeleton progress separately. Do not wait for `robopoker` git migration to start writing INT-1 and INT-2.

Update this review after each integration slice completes to track proof availability and remaining cross-lane blockers.
