# `agent-integration` Lane — Agent Adapter Specification

## Purpose

This lane bridges the **agent-facing surfaces** defined in `agent:experience` with the **implementation reality** of `play:tui` and the broader Myosu product. It produces the integration artifact (`agent-adapter.md`) and the decision record (`review.md`) that determine whether the product is ready for an implementation-family workflow or requires another upstream unblock.

The lane does not implement code. It synthesizes the reviewed artifacts from `agent:experience`, traces the integration dependencies to `play:tui` and upstream lanes, and renders an honest judgment about what the product should do next.

---

## What `agent:experience` Promises

From `outputs/agent/experience/spec.md`, the `agent:experience` lane delivers these surfaces:

| Surface | File | Status |
|---------|------|--------|
| `--pipe` mode | `pipe.rs` | PARTIAL — driver exists, flags missing |
| `--context <path>` flag | `agent_context.rs` (new) | MISSING |
| `--narrate` flag | `narration.rs` (new) | MISSING |
| `reflect>` prompt | `pipe.rs` (extend) | MISSING |
| Agent context file | `agent_context.rs` | MISSING |
| Agent journal | `journal.rs` | MISSING |
| Lobby + game selection | `pipe.rs` (extend) | MISSING |
| JSON schema | `docs/api/game-state.json` + `schema.rs` | **TRUSTED** |
| `SpectatorRelay` | `spectate.rs` (new) | MISSING |
| `SpectateScreen` | `screens/spectate.rs` (new) | MISSING |

The lane defines 9 implementation slices (Slices 1–9) across 4 phases.

---

## Integration Contract: `agent:experience` → `play:tui`

`agent:experience` is NOT self-contained. Every surface it defines requires the `myosu-play` binary as the dispatch vehicle. The binary is owned by `play:tui`.

### The Integration Points

```
agent:experience surface          play:tui owner              Integration contract
─────────────────────────────────────────────────────────────────────────────────
--pipe flag                      myosu-play main.rs          CLI dispatch to PipeMode
--context <path> flag           myosu-play main.rs          Passes context_path to PipeMode
--narrate flag                  myosu-play main.rs          Passes narrate bool to PipeMode
--spectate flag                 myosu-play main.rs          Dispatches to SpectateScreen
SpectatorRelay socket           myosu-play spectate.rs      Emit socket events from active session
myosu-play binary               crates/myosu-play/          MUST EXIST before Slice 3+ begins
```

### `myosu-play` Binary Dependency Map

```
myosu-play binary (play:tui Slice 1)
│
├───► --pipe dispatch ────────────────────► PipeMode (myosu-tui)
│         ├───► --context flag ────────────► agent_context.rs (agent:experience Slice 1)
│         ├───► --narrate flag ────────────► narration.rs (agent:experience Slice 5)
│         ├───► reflect> prompt ───────────► pipe.rs extension (agent:experience Slice 4)
│         └───► lobby ────────────────────► pipe.rs extension (agent:experience Slice 7)
│
└───► --spectate dispatch ─────────────────► SpectateScreen (agent:experience Slice 9)
          └───► SpectatorRelay ────────────► spectate.rs (agent:experience Slice 8)
```

**The binary must exist before any `agent:experience` slice beyond Slice 2 can be implemented or tested.**

---

## Integration Status by Upstream Lane

### `play:tui` Status

| Milestone | Status | Evidence |
|-----------|--------|----------|
| `play:tui` reviewed milestone | **NOT ACHIEVED** | `outputs/play/tui/review.md` exists but binary and NLHE renderer are MISSING |
| `myosu-play` binary | **MISSING** | No crate at `crates/myosu-play/` |
| `myosu-games-poker` crate | **MISSING** | No crate at `crates/myosu-games-poker/` |

From `outputs/play/tui/spec.md`:
- Slice 1 (binary skeleton) is the first required step
- Slice 2 (NLHE renderer) follows
- Phase B (TrainingTable, BlueprintBackend, SolverAdvisor) follows
- Phase C (chain discovery) is blocked on `chain:runtime`

### `tui:shell` Status

| Milestone | Status | Evidence |
|-----------|--------|----------|
| All tests pass | **TRUSTED** | 82 tests pass |
| `GameRenderer` trait | **TRUSTED** | Exists at `crates/myosu-tui/src/renderer.rs` |
| `PipeMode` driver | **TRUSTED** | 6 tests pass |

### `games:traits` Status

| Milestone | Status | Evidence |
|-----------|--------|----------|
| All tests pass | **TRUSTED** | 14 tests pass |
| Traits re-export | **TRUSTED** | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` |

### `robopoker` Dependency

**BLOCKER for CI**: Both `myosu-games` and `myosu-tui` reference robopoker via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). No git dependency exists yet.

From `outputs/games/traits/review.md`: This is the highest-priority Slice 1 fix for `games:traits`. From `outputs/play/tui/spec.md`: `play:tui` Slice 1 must verify the robopoker dependency is resolved before robopoker APIs are called.

**Impact on `agent:experience`**: All 9 slices ultimately call into `games:traits` or `tui:shell`, which call into robopoker. Until robopoker is migrated to a git dependency, `cargo build` and `cargo test` will fail on any clean checkout or CI environment.

### `chain:runtime` Status

**Soft blocker for Phase 2**: Lobby queries (Slice 7) and spectator WebSocket upgrade (Phase 1 of spectator relay) depend on `chain:runtime`. Phase 0 of `agent:experience` (Slices 1–4) does NOT require chain.

---

## Honest Integration Assessment

### What Is Actually Ready

1. **`schema.rs` + `game-state.json`** — Fully trusted. `agent:experience` can rely on this without qualification.
2. **`GameRenderer` trait + `PipeMode` driver** — Trusted. Slices 1–4 can be designed against these.
3. **`tui:shell` + `games:traits`** — Trusted upstreams with passing tests.
4. **Source specs (AX-01..05 + SP-01..03)** — Mature drafts. Decision logs are complete.

### What Is NOT Ready

1. **`myosu-play` binary** — Blocks Slices 3–9. No binary, no `--context`, no `--narrate`, no lobby.
2. **`SpectatorRelay`** — Requires `spectate.rs` in `myosu-play`. Blocks Slice 8.
3. **`SpectateScreen`** — Requires `screens/spectate.rs` in `myosu-tui`. Blocks Slice 9.
4. **robopoker git migration** — Blocks CI. Affects any slice that compiles against `games:traits` or `tui:shell`.
5. **`myosu-games-poker`** — The concrete `GameRenderer` impl. `agent:experience` uses `GameRenderer::pipe_output()` — if the NLHE impl is missing, the pipe output contract is unproven for poker.

### Phase 0 Readiness: Slices 1–2 Only

Slices 1 (`agent_context.rs`) and 2 (`journal.rs`) are **independently implementable**:
- They depend only on `tui:shell` (82 tests, trusted)
- They do not require `myosu-play` binary
- They do not require `SpectatorRelay`
- They can be written, tested, and verified before `play:tui` exists

Slices 3–9 require `myosu-play` binary and are effectively blocked until `play:tui` Slice 1 completes.

---

## Slice Mapping: Phase 0 vs Phase 1+

| Slice | Module | Phase | Blocking Dependency |
|-------|--------|-------|---------------------|
| Slice 1 | `agent_context.rs` | Phase 0 | `tui:shell` (trusted) |
| Slice 2 | `journal.rs` | Phase 0 | `tui:shell` (trusted) |
| Slice 3 | `--context` wiring | Phase 1+ | `myosu-play` binary (MISSING) |
| Slice 4 | `reflect>` prompt | Phase 1+ | `myosu-play` binary (MISSING) |
| Slice 5 | `narration.rs` | Phase 1+ | `myosu-play` binary (MISSING) |
| Slice 6 | `--narrate` wiring | Phase 1+ | `myosu-play` binary (MISSING) |
| Slice 7 | Lobby + selection | Phase 1+ | `myosu-play` binary (MISSING) + chain stub |
| Slice 8 | `SpectatorRelay` | Phase 1+ | `myosu-play` binary (MISSING) |
| Slice 9 | `SpectateScreen` | Phase 1+ | Slice 8 + binary (MISSING) |

---

## Honest Next Steps

### Option A: Proceed to Implementation-Family for Phase 0 Slices

`agent:experience` Slices 1–2 can begin immediately. They are independent of `myosu-play`. This is the **honest** path given current state.

### Option B: Wait for `play:tui` Slice 1 Before Wider Implementation

`play:tui` Slice 1 creates the `myosu-play` binary. Until it exists, Slices 3–9 cannot be implemented or tested. Waiting avoids a split where `agent_context.rs` and `journal.rs` are written in Phase 0 but `--context` flag wiring is stranded without the binary.

### Decision for This Lane

**Honest answer**: The product is ready for a **phased implementation approach**:

- **Now**: `agent:experience` Phase 0 (Slices 1–2: `agent_context.rs`, `journal.rs`) — no `myosu-play` binary required
- **After `play:tui` Slice 1**: `agent:experience` Phase 1+ (Slices 3–9) — binary required for flag wiring and integration testing

The `robopoker` git migration is the **single most critical shared blocker** — it affects CI across `games:traits`, `play:tui`, and `agent:experience` alike. Resolving it is a prerequisite for any of these lanes to run clean tests in a CI environment.

---

## Adapter Surface Summary

The `agent:experience` integration surface maps to the product as follows:

```
Product surface              Owned by              agent:experience integration
──────────────────────────────────────────────────────────────────────────────
myosu-play CLI              play:tui              Dispatches --pipe, --context, --narrate, --spectate
PipeMode driver             tui:shell             Receives flags, drives game loop
GameRenderer (trait)        tui:shell             pipe_output() is the agent-facing text contract
schema.rs                   agent:experience      Trusted; used by PipeMode and SpectatorRelay
agent_context.rs            agent:experience      New; loaded at startup, saved at shutdown
journal.rs                  agent:experience      New; append-only markdown artifact
narration.rs                agent:experience      New; prose rendering when --narrate set
SpectatorRelay              agent:experience      New; emits GameEvent JSON to Unix socket
SpectateScreen              agent:experience      New; renders spectator view with fog-of-war
```
