# `agent:integration` Lane Specification

## Purpose and User-Visible Outcome

`agent:integration` is the **adapter layer** that wires `agent:experience` surfaces to the rest of the Myosu product — specifically to chain discovery, miner serving, validator scoring, and gameplay binary delivery.

The lane delivers:

1. **JSON Schema as Universal Contract** — The `GameState` schema (`docs/api/game-state.json` + `crates/myosu-tui/src/schema.rs`) is the single vocabulary shared by pipe mode, narration engine, spectator relay, and future HTTP/WS APIs
2. **Pipe Mode Adapter** — `PipeMode` driver (`crates/myosu-tui/src/pipe.rs`) bridges `games:traits` game state to stdin/stdout text protocol
3. **Context File Adapter** — `AgentContext` (`crates/myosu-tui/src/agent_context.rs`) bridges JSON context files to the pipe mode session lifecycle
4. **Narration Adapter** — `NarrationEngine` (`crates/myosu-tui/src/narration.rs`) bridges `GameState` to atmospheric prose
5. **Journal Adapter** — `Journal` (`crates/myosu-tui/src/journal.rs`) bridges hand events to append-only markdown
6. **Spectator Relay Adapter** — `SpectatorRelay` (`crates/myosu-play/src/spectate.rs`) bridges game events to Unix socket event stream
7. **Binary Delivery** — `myosu-play` CLI (`crates/myosu-play/src/main.rs`) exposes `--pipe`, `--context`, `--narrate`, `--spectate` flags

**User-visible behavior**: An agent can connect via `myosu-play --pipe --context ./koan.json --narrate` and interact with the game, with all surfaces (pipe protocol, narration, context persistence, journal, spectator) composing cleanly through shared types.

---

## Lane Boundary

```
                            agent:integration (THIS LANE)
                            ┌──────────────────────────────────────────────────────────────┐
upstream                         │                                                              │
agent:experience ───────────────► │  JSON Schema (schema.rs) = universal contract             │
  (spec.md, review.md)            │  ┌────────────────────────────────────────────────────┐ │
                                 │  │ GameState + LegalAction + GamePhase + MetaInfo     │ │
                                 │  └────────────────────────────────────────────────────┘ │
                                 │                                                              │
upstream                         │  ┌────────────────────────────────────────────────────┐ │
games:traits ────────────────────► │  pipe.rs: PipeMode driver                             │ │
  (CfrGame, Profile, GameConfig)  │  │   ↕                                                           │ │
                                 │  │  stdin/stdout text protocol                           │ │
                                 │  └────────────────────────────────────────────────────┘ │
                                 │                                                              │
untrusted                         │  ┌────────────────────────────────────────────────────┐ │
robopoker ───────────────────────► │  games:traits wraps robopoker types                   │ │
  (absolute path deps)            │  │   ↕                                                           │ │
                                 │  │  GameRenderer trait normalizes to GameState            │ │
                                 │  └────────────────────────────────────────────────────┘ │
                                 │                                                              │
downstream                        │  ┌────────────────────────────────────────────────────┐ │
chain:runtime ──────────────────► │  Lobby queries miner axon (Phase 2)                    │ │
  (miner serving)                 │  │  Spectator upgrades to WS via miner axon (Phase 2)   │ │
                                 │  └────────────────────────────────────────────────────┘ │
                                 │                                                              │
downstream                        │  ┌────────────────────────────────────────────────────┐ │
myosu-play ─────────────────────► │  CLI binary exposes --pipe --context --narrate         │ │
  (binary delivery)              │  │  myosu-play --pipe --context ./koan.json --narrate  │ │
                                 │  └────────────────────────────────────────────────────┘ │
                                 └──────────────────────────────────────────────────────────────┘
```

**Trusted upstream inputs:**
- `agent:experience` (spec.md, review.md) — specification only; defines surfaces
- `games:traits` (14 tests pass) — `CfrGame`, `Profile`, `GameConfig`, `GameType`
- `tui:shell` (82 tests pass) — `Shell`, `GameRenderer` trait, `PipeMode`

**Untrusted inputs** (validated at use site):
- Agent-supplied context JSON file (serde-validated before use)
- Agent-supplied reflection text (free-form string; no parsing required)
- Spectator relay socket events (fog-of-war enforced at relay)

**Trusted downstream outputs:**
- `myosu-play` binary with `--pipe`, `--context`, `--narrate`, `--spectate` flags
- `SpectatorRelay` Unix socket event stream

---

## How the JSON Schema Acts as Universal Adapter

The `GameState` JSON schema is the contract that enables all agent-facing surfaces to share the same vocabulary:

```
                    GameState JSON Schema
                    (shared vocabulary)
                           │
           ┌───────────────┼───────────────┐
           │               │               │
           ▼               ▼               ▼
    pipe_output()    narration.rs    SpectatorRelay
    (text adapter)   (prose adapter)  (event adapter)
           │               │               │
           ▼               ▼               ▼
      stdin/stdout     prose output    Unix socket
```

**The schema enables:**
1. **Pipe mode** — `GameRenderer::pipe_output()` serializes `GameState` to terse key-value text
2. **Narration** — `NarrationEngine::narrate(&GameState)` translates the same `GameState` to atmospheric prose
3. **Spectator relay** — `GameEvent` reuses the same `GameState` types with fog-of-war filtering
4. **Future HTTP/WS APIs** — Phase 2 miner axon endpoints serve the same `GameState` JSON

**Key contract**: `legal_actions` is always exhaustive. An agent never needs to guess what's legal.

---

## Integration Points with Other Lanes

| Lane | Integration Point | Direction |
|------|------------------|-----------|
| `games:traits` | `GameRenderer::pipe_output()` → `GameState` | upstream |
| `tui:shell` | `PipeMode` driver + `GameRenderer` trait | upstream |
| `agent:experience` | Spec defines surfaces; this lane wires them | upstream spec |
| `play:tui` | `myosu-play` binary skeleton | downstream |
| `chain:runtime` | Miner axon for lobby queries + WS spectator | downstream (Phase 2) |
| `robopoker` | Absolute path deps; git migration needed before integration testing | upstream (blocked) |

---

## Broken or Missing Integration Points

### 1. `robopoker` Absolute Path Deps — Integration Testing Blocked

Both `games:traits` and `tui:shell` depend on `robopoker` via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This prevents `cargo build` and `cargo test` from running on clean checkout or CI.

**Impact**: Cannot verify integration between `agent:experience` surfaces and the actual game engine.
**Resolution**: `games:traits` lane owns robopoker git migration (RF-01..04). Integration testing depends on that resolution.

### 2. `myosu-play` Binary Skeleton Missing — CLI Delivery Blocked

The `myosu-play` binary (`crates/myosu-play/src/main.rs`) does not exist. All `--pipe`, `--context`, `--narrate`, and `--spectate` flags require modifications to this binary's CLI dispatch.

**Impact**: `agent:integration` surfaces cannot be delivered as a runnable binary.
**Resolution**: `play:tui` lane owns the binary skeleton (Slice 1 of that lane).

### 3. Chain Discovery Stubbed in Lobby — Phase 2 Only

The lobby (AC-AX-05) requires querying the chain or miner for active subnet information. This is stubbed for Phase 0.

**Impact**: Agents in pipe mode can only select from hardcoded subnet data.
**Resolution**: Real chain integration is Phase 2 (depends on `chain:runtime`).

### 4. Spectator Socket Path Convention Not Confirmed

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock` but `play:tui` uses `{data-dir}/hands/hand_{N}.json`. The spectator socket path convention should be confirmed against this.

**Impact**: Potential mismatch between spectator relay socket path and data directory convention.
**Resolution**: Verify `play:tui` data directory convention before Slice 8 implementation.

---

## Code Boundaries and Deliverables

| File | Responsibility | Status |
|------|---------------|--------|
| `crates/myosu-tui/src/schema.rs` | `GameState`, `LegalAction`, `GamePhase`, `MetaInfo` types | **TRUSTED** |
| `crates/myosu-tui/src/pipe.rs` | `PipeMode` driver; `--context`, `--narrate`, `reflect>` wiring | **PARTIAL** (no flags yet) |
| `crates/myosu-tui/src/agent_context.rs` | `AgentContext` load/save; journal append | **MISSING** |
| `crates/myosu-tui/src/narration.rs` | `NarrationEngine` prose generation | **MISSING** |
| `crates/myosu-tui/src/journal.rs` | `Journal` append-only markdown | **MISSING** |
| `crates/myosu-play/src/spectate.rs` | `SpectatorRelay` Unix socket | **MISSING** |
| `crates/myosu-play/src/main.rs` | CLI dispatch for `--pipe`, `--context`, `--narrate` | **MISSING** |

---

## Phase Ordering for Integration

```
Phase 0 (Standalone — no chain):
  1. robopoker git migration (games:traits owns this)
  2. play:tui binary skeleton (play:tui owns this)
  3. agent_context.rs + journal.rs + --context wiring (agent:experience Slices 1-3)
  4. reflect> prompt (agent:experience Slice 4)
  5. narration.rs + --narrate wiring (agent:experience Slices 5-6)
  6. Lobby + game selection (agent:experience Slice 7)

Phase 1 (Spectator — no chain):
  7. SpectatorRelay Unix socket (agent:experience Slice 8)
  8. SpectateScreen TUI (agent:experience Slice 9)

Phase 2 (Chain-connected):
  9. Lobby queries miner axon
  10. Spectator upgrades to miner axon WS
```

---

## Decision from `agent:experience` Review

The `agent:experience` review (outputs/agent/experience/review.md) judged **KEEP — proceed to implementation-family workflow**. This `agent:integration` lane is the integration layer that delivers that implementation family.

The review identified:
- `games:traits` (robopoker git migration) as the primary blocker
- `play:tui` (binary skeleton) as the secondary blocker
- Both are owned by other lanes and do not block this lane from beginning its work

**This lane should proceed with Phase 0 integration work while those blockers are resolved in parallel.**

---

## Next Implementation Slices

### Slice INT-1: Verify `games:traits` + `robopoker` Integration Contract
**What**: Confirm that `GameRenderer::pipe_output()` can consume `games:traits` types and produce valid `GameState` JSON
**Blocker**: robopoker git migration (games:traits lane owns)
**Proof gate**: `cargo test -p myosu-tui schema::tests` exits 0

### Slice INT-2: Deliver `myosu-play` Binary Skeleton with `--pipe` Flag
**What**: Create `crates/myosu-play/src/main.rs` with `--pipe` flag that instantiates `PipeMode`
**Blocker**: play:tui lane owns binary skeleton
**Proof gate**: `myosu-play --help` shows `--pipe` flag

### Slice INT-3: Wire `--context` Flag to `PipeMode`
**What**: Add `context_path: Option<PathBuf>` to `PipeMode`; load/save `AgentContext`
**Depends on**: Slice INT-2 + agent:experience Slice 1
**Proof gate**: Agent plays 10 hands, shuts down, restarts → memory preserved

### Slice INT-4: Wire `--narrate` Flag to `PipeMode`
**What**: Add `narrate: bool` to `PipeMode`; use `NarrationEngine` when true
**Depends on**: Slice INT-3 + agent:experience Slice 5
**Proof gate**: `--narrate` output contains board texture + session arc prose

### Slice INT-5: Integrate `SpectatorRelay` with `PipeMode`
**What**: `SpectatorRelay` emits events from active `PipeMode` session to Unix socket
**Depends on**: Slice INT-2 + agent:experience Slice 8
**Proof gate**: `cargo test -p myosu-play spectate::tests::relay_emits_events` exits 0
