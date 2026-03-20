# Agent Integration Lane Adapter

**Lane**: `agent:integration`
**Date**: 2026-03-20
**Status**: Bootstrap

---

## Purpose

This lane owns the **integration surface** between the agent-facing surfaces defined in `agent:experience` and the underlying game engine, miner, and validator infrastructure. Where `agent:experience` defines what the agent sees, this lane ensures the integration paths from the agent-facing presentation layer down to `games:traits`, `tui:shell`, and eventually the miner/validator service layer are coherent and non-blocking.

This lane is the **last unstarted product lane** after `agent:experience` achieved reviewed status. It synthesizes the `agent:experience` spec and review with the actual upstream readiness of the surfaces it depends on.

---

## Lane Boundary

```
                    agent:integration (THIS LANE)
                    ┌─────────────────────────────────────────────────────────────┐
upstream             │                                                              │
agent:experience ──► │  Synthesizes: schema.rs + pipe_output() + narration       │
  (reviewed, KEEP)  │  + agent_context.rs + journal.rs + SpectatorRelay         │
                     │                                                              │
upstream             │  Integration paths:                                          │
tui:shell ─────────► │  pipe.rs → Shell → GameRenderer → PipeMode                │
  (82 tests, trusted)                                                              │
                     │  agent_context.rs → load/save JSON → disk                  │
                     │  journal.rs → append-only markdown → disk                 │
                     │  narration.rs → GameState → prose                         │
                     │  SpectatorRelay → Unix socket → spectator TUI              │
                     │                                                              │
upstream             │  Phase 2 integration paths:                                │
games:traits ──────► │  GameType, GameConfig, StrategyQuery/Response             │
  (14 tests, trusted)                                                              │
                     │                                                              │
upstream             │  Phase 3 integration paths (blocked on chain:runtime):     │
chain:runtime ─────► │  lobby → miner axon HTTP → subnet list                    │
                     │  spectate → miner axon WS → live game events             │
                     │                                                              │
upstream             │  Phase 4 integration paths (blocked on miner/validator):  │
miner service ─────► │  StrategyQuery → miner HTTP → StrategyResponse            │
                     │  exploitability oracle → validator → weight submission     │
                     └─────────────────────────────────────────────────────────────┘
```

**This lane does NOT own**:
- The `tui:shell` implementation (trusted upstream)
- The `games:traits` implementation (trusted upstream)
- The `chain:runtime` implementation (restart lane, upstream)
- The `play:tui` binary skeleton (owns `myosu-play` binary)
- The `miner:service` implementation (future lane)

**This lane DOES own**:
- The integration wiring between agent:experience surfaces and upstream crates
- The `--context` and `--narrate` flag wiring into `myosu-play` CLI (once binary exists)
- The lobby stub for Phase 0 (hardcoded subnet list)
- The SpectatorRelay integration with the relay socket
- The decision record on which implementation slices can proceed without upstream blockers

---

## Integration State Inventory

### Phase 1 — Agent Identity (depends only on tui:shell)

| Surface | Status | Integration Path |
|---------|--------|-----------------|
| `agent_context.rs` | MISSING | `crates/myosu-tui/src/agent_context.rs` — loads/saves JSON; wired to `PipeMode` |
| `journal.rs` | MISSING | `crates/myosu-tui/src/journal.rs` — append-only markdown; called after each hand in pipe mode |
| `--context` flag wiring | MISSING | `PipeMode` struct needs `context_path: Option<PathBuf>` field |
| `reflect>` prompt | MISSING | After `HAND COMPLETE` in pipe mode, block stdin; append to journal |
| `narration.rs` | MISSING | `crates/myosu-tui/src/narration.rs` — `NarrationEngine::narrate(&GameState) -> String` |
| `--narrate` flag wiring | MISSING | `PipeMode` struct needs `narrate: bool` field; use `NarrationEngine` when true |
| Schema (`schema.rs`) | **TRUSTED** | `crates/myosu-tui/src/schema.rs` — fully implemented, 16 tests pass |

### Phase 2 — Narration + Lobby (depends on Phase 1 + tui:shell)

| Surface | Status | Integration Path |
|---------|--------|-----------------|
| Lobby in pipe mode | MISSING | When no `--subnet`, render hardcoded lobby (Phase 0 stub) |
| Game selection | MISSING | `info <id>` command; subnet selection starts game |

### Phase 3 — Spectator (depends on play:tui binary)

| Surface | Status | Integration Path |
|---------|--------|-----------------|
| `SpectatorRelay` | MISSING | `crates/myosu-play/src/spectate.rs` — Unix socket at `~/.myosu/spectate/<session_id>.sock` |
| `SpectateScreen` | MISSING | `crates/myosu-tui/src/screens/spectate.rs` — fog-of-war rendering |

### Phase 4 — Chain-Connected (blocked on chain:runtime)

| Surface | Status | Integration Path |
|---------|--------|-----------------|
| Lobby chain query | BLOCKED | Stub with hardcoded data for Phase 0; real query blocked on `chain:runtime` |
| Spectator WS upgrade | BLOCKED | Blocked on miner axon WS endpoint |

---

## Integration Decision Record

### Decision: Phase 1 slices 1-4 can proceed immediately without upstream unblock

**Rationale**: Slices 1–4 (`agent_context.rs`, `journal.rs`, `--context` wiring, `reflect>` prompt) depend only on `tui:shell` which is already trusted with 82 passing tests. The `PipeMode` driver exists and compiles. These slices can begin implementation immediately.

**Evidence**: `agent:experience/review.md` explicitly states: "This lane can begin with Slices 1–2 (`agent_context.rs` and `journal.rs`) immediately, as they depend only on `tui:shell` which is already trusted."

### Decision: Slice 5 (`narration.rs`) can proceed immediately but is independent

**Rationale**: `narration.rs` depends on `games:traits` (trusted, 14 tests) and the `GameState` type from `schema.rs` (trusted). It does not require the `--context` flag wiring. It can be implemented in isolation.

**Evidence**: `NarrationEngine::narrate(&GameState)` takes a `GameState` reference — the schema is already trusted and stable.

### Decision: Slice 6 (`--narrate` wiring) is blocked on Slice 3 (`--context` wiring)

**Rationale**: Both `--narrate` and `--context` require modification to `PipeMode` struct initialization in `myosu-play`'s `main.rs`. Slice 6 depends on the same CLI wiring that Slice 3 introduces.

**Evidence**: `agent:experience/spec.md` Slice 6 description: "wire `--narrate` flag to CLI" — same vehicle as Slice 3.

### Decision: Slice 7 (lobby) is blocked on `play:tui` binary + Phase 0 stub

**Rationale**: The lobby rendering in pipe mode requires the `myosu-play` binary to exist. For Phase 0, the lobby can be stubbed with hardcoded subnet data. Real chain-connected lobby is Phase 4 (blocked on `chain:runtime`).

**Resolution for Slice 7**: Use hardcoded lobby data for Phase 0. No chain query needed yet.

### Decision: Slices 8–9 (spectator) are blocked on `play:tui` binary

**Rationale**: `SpectatorRelay` is a `crates/myosu-play/src/spectate.rs` file — it lives in the `myosu-play` binary crate, which does not exist yet.

**Evidence**: `agent:experience/review.md`: "The `myosu-play` binary is defined in `play:tui`'s spec and is the vehicle through which all `--pipe`, `--context`, `--narrate`, and `--spectate` flags are exposed."

---

## Summary of Immediate Work (No Upstream Unblock Required)

The following can proceed immediately under `agent:integration`:

1. **Slice 1**: `agent_context.rs` — `AgentContext` struct with load/save/default; serde JSON
2. **Slice 2**: `journal.rs` — append-only markdown writer; hand entry formatter
3. **Slice 5**: `narration.rs` — `NarrationEngine::narrate(&GameState)`; board texture analysis
4. **Slice 7 Phase 0**: Lobby with hardcoded subnet data

The following are **blocked** and should not be attempted until the blocker is resolved:

| Slice | Blocker | Blocker Owner |
|-------|---------|--------------|
| Slice 3 (`--context` wiring) | `myosu-play` binary missing | `play:tui` |
| Slice 4 (`reflect>` prompt) | `myosu-play` binary missing | `play:tui` |
| Slice 6 (`--narrate` wiring) | Slice 3 dependency | `play:tui` |
| Slice 8 (`SpectatorRelay`) | `myosu-play` binary missing | `play:tui` |
| Slice 9 (`SpectateScreen`) | `play:tui` binary + SpectatorRelay | `play:tui` |
| Phase 4 lobby (real chain query) | `chain:runtime` restart | `chain:runtime` |
| Phase 4 spectator WS | `chain:runtime` + miner service | `chain:runtime`, `miner:service` |

---

## Phase Ordering Recommendation

```
Phase 1 (no upstream unblock required):
  Slice 1 → Slice 2 → Slice 5

Phase 2 (blocked on play:tui binary):
  Slice 3 → Slice 4 → Slice 6

Phase 3 (blocked on play:tui binary):
  Slice 7 (lobby, Phase 0 stub)

Phase 4 (blocked on chain:runtime + miner:service):
  Slice 7 (lobby, chain-connected)
  Slice 8 → Slice 9 (spectator WS upgrade)
```
