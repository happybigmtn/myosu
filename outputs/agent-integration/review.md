# `agent-integration` Lane Review

## Judgment: **UPSTREAM UNBLOCK FIRST** — then implementation-family workflow

The `agent:experience` lane is **well-specified and ready to implement** (KEEP from its own review). However, this lane's integration assessment reveals that **the immediate next step is not implementation of the agent:experience slices — it is resolving the upstream blockers that would prevent any implementation work from being tested in a clean environment**.

The `games:traits` lane owns the `robopoker` git migration (highest priority). The `play:tui` lane owns the `myosu-play` binary skeleton. Until both are resolved, agent:experience implementation slices 1–4 can be coded but cannot be verified via `cargo test` in a clean checkout.

---

## Decision Rationale

### `agent:experience` Is Ready to Implement

The `agent:experience` lane review (`outputs/agent/experience/review.md`) is correct: the spec is sound, the upstream (`tui:shell`, `games:traits`) is trusted, and the 9 implementation slices are sequential and minimally coupled.

Slices 1–2 (`agent_context.rs`, `journal.rs`) depend only on `tui:shell` (82 tests pass). Slices 3–4 (`--context` wiring, `reflect>` prompt) depend on `tui:shell` plus the `myosu-play` binary. Slices 5–7 (narration, `--narrate`, lobby) depend on slices 1–4.

The lane is ready. The problem is environmental, not architectural.

### The `robopoker` Git Migration Is the Primary Blocker

Both `tui:shell` and `games:traits` declare robopoker as a **filesystem path dependency**:

```toml
rbp-core = { path = "/home/r/coding/robopoker/crates/util" }
rbp-mccfr = { path = "/home/r/coding/robopoker/crates/mccfr" }
```

This means:
- Any developer without `robopoker` checked out at exactly `/home/r/coding/robopoker/` cannot build the project
- CI cannot run without this exact filesystem layout
- The `games:traits` lane identified this as its highest-priority Slice 1 fix
- This blocker affects ALL downstream lanes that touch `games:traits` or `tui:shell` — which is all of them

Until `robopoker` is migrated to a proper git dependency, no agent:experience slice can be proven in a clean environment.

### The `myosu-play` Binary Is the Secondary Blocker

Slices 3–4 (and all subsequent slices) require modifications to the `myosu-play` binary's CLI dispatch. The binary does not exist yet.

The `play:tui` lane owns this. The `agent:experience` lane cannot wire `--context` or `--narrate` flags to a binary that doesn't exist.

---

## Upstream Blocker Summary

| Blocker | Owner | Priority | Blocks |
|---------|-------|----------|--------|
| `robopoker` git migration | `games:traits` lane | HIGH | All `cargo test -p myosu-tui` in clean envs |
| `myosu-play` binary skeleton | `play:tui` lane | HIGH | `agent:experience` Slices 3–9 |
| `play:tui` lane REOPEN items | `play:tui` lane | MEDIUM | Full `play:tui` trust before agent:experience Slices 3+ |

The `tui:shell` REOPEN items (schema, events, shell) do NOT block this lane — they are in `myosu-tui` and the pipe mode tests (pipe.rs) already pass independently.

---

## What This Lane Recommends

### Recommendation: UPSTREAM UNBLOCK (immediate)

**Do this first**:

1. **`games:traits` Slice 1** — Replace absolute robopoker path dependencies with git dependencies:
   ```toml
   rbp-core = { git = "https://github.com/happybigmtn/robopoker", rev = "..." }
   rbp-mccfr = { git = "https://github.com/happybigmtn/robopoker", rev = "..." }
   ```
   **This unblocks `cargo test -p myosu-tui` in clean environments.**

2. **`play:tui` Slice 1** — Create `myosu-play` binary skeleton with basic CLI dispatch:
   - `--pipe` flag that enters pipe mode
   - Stub game state that makes `pipe_mode.run_once()` work without a live game
   **This unblocks `agent:experience` Slice 3 (--context wiring).**

### Recommendation: IMPLEMENTATION-FAMILY WORKFLOW (after upstream resolves)

After the upstream blockers are resolved:

1. **Begin `agent:experience` Slices 1–2** in parallel with continued `play:tui` work:
   - Slice 1: `agent_context.rs` (identity, memory, journal struct)
   - Slice 2: `journal.rs` (append-only markdown writer)
   Both depend only on `tui:shell` (trusted, 82 tests).

2. **Begin `agent:experience` Slice 5** (narration engine) in parallel:
   - `narration.rs` with board texture analysis
   - Depends only on `schema.rs` (trusted) and `tui:shell` (trusted)
   - Does NOT need the binary (no flag wiring yet)

The slices that need the binary (Slices 3–4, 6–7) should wait for `play:tui` Slice 1.

---

## Honest Assessment of Agent:experience Readiness

| Dimension | Status | Notes |
|-----------|--------|-------|
| Specification | **READY** | AX-01..05 + SP-01..03 spec complete; 9 slices defined |
| Upstream `tui:shell` | **TRUSTED** | 82 tests pass; pipe.rs independently tested |
| Upstream `games:traits` | **TRUSTED** | 14 tests pass; but robopoker git migration needed |
| `schema.rs` | **TRUSTED** | 16 tests; fully implemented |
| `play:tui` binary | **MISSING** | Binary skeleton needed for Slice 3+ |
| Clean env testability | **BLOCKED** | robopoker path deps prevent clean env builds |
| Spectator protocol | **SPEC ONLY** | Not implemented |

---

## Files This Lane Reviewed

| File | Role | Assessment |
|------|------|-----------|
| `outputs/agent/experience/spec.md` | Agent:experience lane spec | Complete, honest, correct boundaries |
| `outputs/agent/experience/review.md` | Agent:experience lane review | Correct KEEP judgment; good blocker analysis |
| `specsarchive/031626-10-agent-experience.md` | Source spec | AX-01..05 mature; decisions well-reasoned |
| `specsarchive/031626-17-spectator-protocol.md` | Source spec | AC-SP-01..03 well-designed |
| `docs/api/game-state.json` | JSON schema | Complete, covers 10 game types, well-structured |
| `crates/myosu-tui/src/schema.rs` | Rust schema | 939 lines, 16 tests, fully implemented |
| `crates/myosu-tui/src/pipe.rs` | Pipe driver | 217 lines, 6 tests, solid foundation but incomplete |
| `crates/myosu-tui/src/agent_context.rs` | Agent context | **MISSING** — AX-01 not implemented |
| `crates/myosu-tui/src/journal.rs` | Agent journal | **MISSING** — AX-04 not implemented |
| `crates/myosu-tui/src/narration.rs` | Narration engine | **MISSING** — AX-03 not implemented |
| `fabro/programs/myosu-product.yaml` | Product manifest | Correctly defines `agent` unit and `experience` lane |
| `fabro/run-configs/product/agent-experience.toml` | Run config | Clear goal, correct scope, achievable |

---

## Recommendation

**Immediate next step: `games:traits` Slice 1** — resolve the robopoker git migration. This is the highest-leverage action across the entire product because it affects all lanes that depend on `games:traits` or `tui:shell`.

**Parallel next step: `play:tui` Slice 1** — create the `myosu-play` binary skeleton. This unblocks the `agent:experience` flag-wiring slices.

**After those resolve: `agent:experience` implementation-family workflow** — Slices 1–2 in parallel with `play:tui` continued work, then Slices 3–4 once binary exists, then Slices 5–7, then Slices 8–9 (spectator).

The product does **not** need another upstream spec or design doc. It needs two concrete implementation slices from two other lanes, then it can proceed with implementation.
