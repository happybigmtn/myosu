# Agent Integration: Adapter Surface

## Purpose

This document describes the interface contract between the `agent:experience` lane
and its upstream dependencies. It maps what the lane consumes, what it requires
from upstream to unblock each slice, and what signals constitute readiness.

This is not an implementation plan. It is the integration shim that allows the
control plane to reason about dependency satisfaction without reading every
upstream spec.

---

## Upstream Contract

`agent:experience` sits at the terminal edge of the product frontier. Its
upstream dependencies are:

| Upstream | Type | What `agent:experience` Consumes |
|----------|------|----------------------------------|
| `tui:shell` | **Hard, trusted** | `Shell`, `GameRenderer` trait, `PipeMode`, `Events`, `Theme` |
| `games:traits` | **Hard, trusted** | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` |
| `play:tui` | **Hard, gated** | `myosu-play` binary skeleton; CLI dispatch for `--pipe`, `--context`, `--narrate` |
| `robopoker` | **Hard, external** | `Game`, `Recall`, `Action` types; absolute path currently, needs git migration |
| `spectator-protocol` | **Spec source** | AX-01..05, SP-01..03 from `specsarchive/` |

---

## Slice-by-Slice Upstream Requirements

### Phase 1: Agent Identity (Slices 1–4)

**Slice 1: `agent_context.rs`**
- Requires: `tui:shell` (trusted, 82 tests) — `GameRenderer` trait, `PipeMode` driver
- Requires: `games:traits` (trusted, 14 tests) — `GameConfig`, `GameType`
- Status: **CAN START NOW** — all upstream trusted

**Slice 2: `journal.rs`**
- Requires: Same as Slice 1
- Status: **CAN START NOW** — all upstream trusted

**Slice 3: `--context` flag wiring**
- Requires: `myosu-play` binary skeleton from `play:tui` Slice 1
- Requires: `games:traits` for context file schema validation
- Status: **BLOCKED** — `play:tui` binary skeleton missing

**Slice 4: `reflect>` prompt**
- Requires: `PipeMode` driver in `tui:shell` (trusted)
- Requires: `journal.rs` from Slice 2 (in-phase dependency)
- Status: **CAN START WITH PARTIAL** — PipeMode trusted; journal.rs is in-phase

### Phase 2: Narration + Pipe Mode (Slices 5–7)

**Slice 5: `narration.rs`**
- Requires: `games:traits` — `GameState` type
- Requires: `tui:shell` — `GameRenderer` trait
- Status: **CAN START NOW** — upstream trusted

**Slice 6: `--narrate` flag wiring**
- Requires: `myosu-play` binary skeleton from `play:tui` Slice 1
- Requires: `narration.rs` from Slice 5 (in-phase dependency)
- Status: **BLOCKED** — `play:tui` binary skeleton missing

**Slice 7: Lobby + game selection**
- Requires: `myosu-play` binary skeleton
- Requires: Chain discovery (stubbed for Phase 0; real integration Phase 4)
- Status: **BLOCKED** — `play:tui` binary skeleton missing; chain stub is acceptable

### Phase 3: Spectator (Slices 8–9)

**Slice 8: `SpectatorRelay`**
- Requires: `play:tui` binary (for socket path convention alignment)
- Requires: `schema.rs` (trusted, 16 tests) — `GameEvent` JSON format
- Status: **BLOCKED** — `play:tui` binary skeleton missing

**Slice 9: `SpectateScreen`**
- Requires: Slice 8 complete (in-phase dependency)
- Requires: `tui:shell` — `ScreenManager`, screen rendering patterns
- Status: **BLOCKED** — depends on Slice 8 and `play:tui` binary

---

## Critical Path: robopoker Git Migration

The `robopoker` dependency is currently via **absolute filesystem paths**:

```
/home/r/coding/robopoker/crates/...
```

This is documented in both `outputs/play/tui/spec.md` and
`outputs/games/traits/review.md` as the highest-priority Slice 1 fix for
those lanes.

**Impact on `agent:experience`**: All 9 slices ultimately call into
`games:traits` or `tui:shell`, which call into `robopoker`. Until
`robopoker` is migrated to a proper git dependency (`git = "https://..."` in
`Cargo.toml`), `cargo build` and `cargo test` will fail on any clean
checkout or CI environment.

**Ownership**: `games:traits` lane owns the robopoker resolution.
`agent:experience` should not proceed past Slice 4 without confirming
robopoker is resolved, because Slices 5–9 require full integration testing.

---

## Interface Surface Summary

| Interface | Provided By | Consumed By | Type |
|-----------|-------------|-------------|------|
| `GameRenderer` trait | `tui:shell` | `agent:experience` | trait bound |
| `PipeMode` driver | `tui:shell` | `agent:experience` | struct |
| `CfrGame`, `Profile` | `games:traits` | `agent:experience` | trait + struct |
| `GameState` JSON schema | `schema.rs` (trusted) | `agent:experience` | type |
| `myosu-play` CLI | `play:tui` | agent/operator | binary |
| `GameEvent` JSON format | `schema.rs` | `SpectatorRelay` | type |

---

## Integration Signals

| Signal | Source | Meaning |
|--------|--------|---------|
| `cargo test -p myosu-tui` passes | CI / local | All upstream crates compile and tests pass |
| `robopoker` in `Cargo.toml` as git dep | `games:traits` | robopoker migration complete |
| `myosu-play --help` shows `--pipe` | `play:tui` | Binary skeleton ready for Slice 3+ |
| `outputs/games/traits/review.md` KEEP | review artifact | `games:traits` lane trusted |
| `outputs/tui/shell/review.md` KEEP | review artifact | `tui:shell` lane trusted |

---

## Adapter Responsibility

This adapter surface exists so that:

1. **Lane authors** can see at a glance what their lane needs from upstream
   without reading every upstream spec
2. **Control plane** can evaluate readiness without running code
3. **Reviewers** can confirm that slice dependencies are real (code-level)
   and not speculative

The adapter does NOT own resolution of upstream blockers. It only documents
the interface contract and identifies who owns each unblock.
