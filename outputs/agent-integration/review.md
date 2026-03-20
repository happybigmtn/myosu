# `agent:integration` Lane Review

## Judgment: **PROCEED with agent:experience implementation — slices 1-2 only.
Full lane integration waits on `myosu-play` binary.**

---

## Source Artifacts Assessed

| Artifact | Source | Judgment |
|----------|--------|----------|
| `outputs/agent/experience/spec.md` | `agent:experience` lane | **KEEP** — 9 slices defined, upstream trusted, spec is sound |
| `outputs/agent/experience/review.md` | `agent:experience` lane | **KEEP** — 4 blockers identified, lane ready for implementation |
| `outputs/agent-integration/agent-adapter.md` | This lane | **KEEP** — honest map of current state to implementation target |

---

## Decision Basis

The `agent:experience` lane has completed its specification and review milestones.
The spec (AX-01..05 + SP-01..03) is mature. The review identified 4 blockers.
This lane's job is to assess whether those blockers are unblocking upstream work or
whether the lane can proceed into an implementation family.

---

## The Four Blockers from `agent:experience` Review

### Blocker 1: `robopoker` Git Migration — HIGH

**Owned by**: `games:traits` lane
**Status**: Acknowledged. The absolute-path `path = "/home/r/coding/robopoker/..."`
dependencies in `crates/myosu-games/Cargo.toml` must be replaced with a git
dependency before `agent:experience` slices can be integration-tested.

**Impact on this lane**: All slices ultimately call into `games:traits` which
calls into `robopoker`. Without the git migration, `cargo build` fails on any
clean checkout or CI environment.

**Honest assessment**: This is a legitimate blocker for any integration test
that exercises the full call chain. However, slices 1-2 (`agent_context.rs`,
`journal.rs`) are pure data-structure work in `myosu-tui` that does not call
robopoker at the type level. They can be implemented and unit-tested without
the robopoker dependency being resolved.

**Decision**: Slices 1-2 can proceed. Slices 5+ (which call into `games:traits`
via `CfrGame`, `Profile`, etc.) must wait.

### Blocker 2: `myosu-play` Binary Does Not Exist — HIGH

**Owned by**: `play:tui` lane
**Status**: The `crates/myosu-play/` directory does not exist at all. No binary
skeleton, no CLI, no `main.rs`.

**Impact on this lane**: All AX slices require modifications to `myosu-play`'s
CLI dispatch. Without the binary:
- `--pipe` flag cannot be wired (PipeMode exists but nothing invokes it)
- `--context` flag has no argument parser
- `--narrate` flag has no argument parser
- `SpectatorRelay` has no process to own its lifecycle

**Honest assessment**: This is the hardest blocker. Unlike robopoker (which is
an external dependency issue), `myosu-play` is an entirely absent crate. No
amount of incremental work on `myosu-tui` can substitute for a binary that owns
CLI dispatch.

The `play:tui` lane review notes that binary skeleton is "PARTIAL" (Slice 1
missing). The `myosu-product.yaml` program structure shows `agent` unit
`depends_on` `play` unit milestone `reviewed`.

**Decision**: This lane cannot proceed to full integration until `play:tui`
produces the binary skeleton. Slices 1-2 (agent_context, journal) are
implementable without the binary, but they cannot be end-to-end tested without
it.

### Blocker 3: Chain Discovery Stubbed in Lobby — MEDIUM

**Owned by**: This lane (Phase 0 stub is acceptable)
**Status**: The lobby requires querying the chain for active subnet information.
Phase 0 accepts a hardcoded stub.

**Honest assessment**: Acceptable for Phase 0. The lobby can render stub data.
Real chain integration is Phase 4 (depends on `chain:runtime`).

**Decision**: Not a blocker for this lane. Implement the lobby with stub data.

### Blocker 4: Spectator Socket Path Convention — LOW

**Owned by**: `play:tui` lane (must confirm data directory convention)
**Status**: `agent-adapter.md` documents `~/.myosu/spectate/<session_id>.sock`
but `play:tui`'s data directory convention uses `{data-dir}/hands/hand_{N}.json`.
These must be reconciled.

**Honest assessment**: Low severity. The socket path convention is documented in
`agent-adapter.md` and should be confirmed against `play:tui`'s convention
before Slice 8 (spectator relay) implementation.

**Decision**: Not a blocker for this lane. Note the convention for Slice 8.

---

## Slice-by-Slice Assessment

| Slice | What | Blocker | Decision |
|-------|------|---------|----------|
| Slice 1 | `agent_context.rs` | None | **PROCEED NOW** |
| Slice 2 | `journal.rs` | None | **PROCEED NOW** |
| Slice 3 | `--context` wiring in `PipeMode` | `myosu-play` binary | Wait on `play:tui` Slice 1 |
| Slice 4 | `reflect>` prompt | `myosu-play` binary | Wait on `play:tui` Slice 1 |
| Slice 5 | `narration.rs` | None (stateless pure function on GameState) | **PROCEED NOW** (testable without binary) |
| Slice 6 | `--narrate` wiring | `myosu-play` binary | Wait on `play:tui` Slice 1 |
| Slice 7 | Lobby + game selection | `myosu-play` binary | Wait on `play:tui` Slice 1 |
| Slice 8 | `SpectatorRelay` | `myosu-play` binary + `play:tui` convention | Wait on `play:tui` Slice 1 |
| Slice 9 | `SpectateScreen` | Slice 8 + `myosu-play` binary | Wait on `play:tui` Slice 1 |

---

## What "Proceed Now" Means

Slices 1, 2, and 5 are implementable today as isolated `myosu-tui` modules:

- **Slice 1** (`agent_context.rs`): Pure data structures + serde. No external
  calls. Unit-testable with `cargo test -p myosu-tui`. Can use the existing
  `AgentContext::load()` / `save()` pattern without any binary.

- **Slice 2** (`journal.rs`): Pure file I/O + markdown formatting. Unit-testable
  by writing to a temp file and verifying append-only behavior.

- **Slice 5** (`narration.rs`): Pure function `NarrationEngine::narrate(&GameState)`.
  The `GameState` type from `schema.rs` is already complete. No binary needed.

The `games:traits` dependency (robopoker git migration) does not affect these
slices because they don't call into `games:traits` at the type level. `GameState`
from `schema.rs` is self-contained.

---

## Honest Assessment: Is the `agent:experience` Lane Blocked on Upstream?

**Yes, but partially.** The lane is blocked on:
1. `myosu-play` binary — owned by `play:tui` lane (hard blocker for slices 3-9)
2. `robopoker` git migration — owned by `games:traits` lane (hard blocker for
   integration tests, soft blocker for slices 1-2)

The `agent-adapter.md` maps the precise interface contracts needed, giving
`play:tui` Slice 1 a clear target to implement.

---

## Recommendation

**Do not create a new implementation family yet.** The agent:experience lane
is not ready for a full implementation family because the critical path runs
through `myosu-play`, which is owned by `play:tui`.

**Do the following in parallel:**

1. **`play:tui` lane**: Implement Slice 1 (binary skeleton) as the critical path.
   The `agent-adapter.md` gives `play:tui` a precise list of what flags and
   dispatch logic the binary must support.

2. **This lane**: Implement slices 1, 2, and 5 in `myosu-tui` while `play:tui`
   builds the binary skeleton. These slices are unit-testable without the
   binary and produce the foundational modules (`agent_context.rs`,
   `journal.rs`, `narration.rs`) that the binary will later wire.

3. **`games:traits` lane**: Resolve the robopoker git migration. This unblocks
   integration testing for slices 5+.

**The next honest decision point** is after `play:tui` Slice 1 lands. At that
point:
- `myosu-play` binary exists with CLI dispatch
- `agent_context.rs`, `journal.rs`, `narration.rs` are implemented and unit-tested
- The full `agent:experience` implementation family can begin with all
  prerequisites in place

---

## Proof Availability

No proof commands can run yet — `myosu-play` binary does not exist. The
following will be the proof gates once the binary lands:

```
cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip
cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites
cargo test -p myosu-tui agent_context::tests::missing_context_creates_new
cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand
cargo test -p myosu-tui pipe::tests::empty_reflection_skips
cargo test -p myosu-tui narration::tests::narrate_includes_board_texture
cargo test -p myosu-tui narration::tests::narrate_includes_session_context
cargo test -p myosu-tui schema::tests   # already passes
cargo test -p myosu-play spectate::tests::relay_emits_events
```

---

## Lane Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| `agent:experience` spec | **READY** | AX-01..05 + SP-01..03 are sound |
| `agent:experience` review | **READY** | KEEP; 4 blockers documented |
| `myosu-play` binary | **MISSING** | Blocks slices 3-9 |
| `myosu-tui` (pipe.rs) | **PARTIAL** | Skeleton exists; no flags wired |
| `myosu-tui` (schema.rs) | **TRUSTED** | 16 tests pass; fully implemented |
| `myosu-tui` (screens.rs) | **PARTIAL** | `Screen::Spectate` variant exists; no rendering |
| `robopoker` git migration | **BLOCKER** | Owned by `games:traits` lane |
| Implementation slices 1, 2, 5 | **CAN PROCEED** | Unit-testable without binary |
| Implementation slices 3-9 | **BLOCKED** | Waiting on `myosu-play` binary |
