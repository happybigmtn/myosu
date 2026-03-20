# `agent:experience` Integration Adapter

## Purpose

This document is the **integration contract** for the `agent:experience` lane — the terminal presentation layer that owns all surfaces through which programmatic agents (LLMs, bots, scripts) perceive and act upon the Myosu game world.

It answers: what does `agent:experience` expose to the rest of Myosu, what does it consume, and where is everything located?

## Lane Position

```
                         upstream lanes                     this lane
                      ┌─────────────────┐           ┌──────────────────────────┐
games:traits ───────► │ CfrGame,        │           │  GameRenderer::pipe_output()│
(14 tests, trusted)   │ GameType,       │──────────►│  --pipe --context --narrate │
                      │ StrategyQuery   │           │  schema.rs (trusted)        │
                      └─────────────────┘           │  agent_context.rs (Slice 1) │
                                                   │  journal.rs (Slice 2)       │
tui:shell ──────────► │ Shell,          │           │  narration.rs (Slice 5)     │
(82 tests, trusted)   │ GameRenderer,   │──────────►│  SpectatorRelay (Slice 8)   │
                      │ PipeMode,       │           └──────────────────────────┘
                      │ Events, Theme   │
                      └─────────────────┘
                                                   downstream: NONE (terminal lane)
```

The lane is **terminal** — it has no trusted downstream consumers. All outputs are final user-facing or agent-facing surfaces.

---

## Consumed Surfaces (Upstream)

### From `tui:shell` (trusted, 82 tests)

| Surface | Type | Location | Used In |
|---------|------|----------|---------|
| `GameRenderer` trait | trait | `crates/myosu-tui/src/renderer.rs` | pipe output, narration |
| `PipeMode` | struct | `crates/myosu-tui/src/pipe.rs` | `--pipe` flag driver |
| `Shell` | struct | `crates/myosu-tui/src/shell.rs` | context loading |
| `Events` | enum | `crates/myosu-tui/src/events.rs` | event emission |
| `Theme` | struct | `crates/myosu-tui/src/theme.rs` | narration style |

### From `games:traits` (trusted, 14 tests)

| Surface | Type | Location | Used In |
|---------|------|----------|---------|
| `CfrGame` | trait | `crates/myosu-games-traits/src/lib.rs` | game state generation |
| `GameType` | enum | `crates/myosu-games-traits/src/lib.rs` | game classification |
| `Profile` | trait | `crates/myosu-games-traits/src/lib.rs` | strategy lookup |
| `GameConfig` | struct | `crates/myosu-games-traits/src/lib.rs` | game configuration |
| `StrategyQuery`, `StrategyResponse` | structs | `crates/myosu-games-traits/src/lib.rs` | solver queries |

### From `spectator-protocol` (spec only, not implemented)

| Surface | Type | Spec Location | Used In |
|---------|------|--------------|---------|
| `GameEvent` JSON schema | type | `specsarchive/031626-17-spectator-protocol.md` | SpectatorRelay |
| AC-SP-01 (Unix socket relay) | spec | `specsarchive/031626-17-spectator-protocol.md` | Slice 8 |
| AC-SP-02 (SpectateScreen) | spec | `specsarchive/031626-17-spectator-protocol.md` | Slice 9 |

---

## Exposed Surfaces (Downstream / Agent-Facing)

### Phase 0 (implementable now, upstream trusted)

| Surface | File | Slice | Status |
|---------|------|-------|--------|
| `AgentContext` struct + load/save | `crates/myosu-tui/src/agent_context.rs` | Slice 1 | **MISSING** |
| `Journal` struct + append | `crates/myosu-tui/src/journal.rs` | Slice 2 | **MISSING** |
| `--context` flag wiring | `crates/myosu-tui/src/pipe.rs` | Slice 3 | **MISSING** |
| `reflect>` prompt after hand | `crates/myosu-tui/src/pipe.rs` | Slice 4 | **MISSING** |
| `GameState` JSON schema | `crates/myosu-tui/src/schema.rs` | — | **TRUSTED** (16 tests) |
| `docs/api/game-state.json` | `docs/api/game-state.json` | — | **TRUSTED** |

### Phase 1 (depends on `--context` wiring, Slice 3)

| Surface | File | Slice | Status |
|---------|------|-------|--------|
| `NarrationEngine` + board texture | `crates/myosu-tui/src/narration.rs` | Slice 5 | **MISSING** |
| `--narrate` flag wiring | `crates/myosu-tui/src/pipe.rs` | Slice 6 | **MISSING** |
| Lobby + `info` command | `crates/myosu-tui/src/pipe.rs` | Slice 7 | **MISSING** |

### Phase 2 (depends on `play:tui` binary, Slice 1 of that lane)

| Surface | File | Slice | Status |
|---------|------|-------|--------|
| `SpectatorRelay` Unix socket | `crates/myosu-play/src/spectate.rs` | Slice 8 | **MISSING** (binary absent) |
| `SpectateScreen` | `crates/myosu-tui/src/screens/spectate.rs` | Slice 9 | **MISSING** (binary absent) |

---

## Code Location Map

```
crates/myosu-tui/src/
├── lib.rs                  # crate root, re-exports GameRenderer, Theme
├── renderer.rs             # GameRenderer trait (upstream: tui:shell)
├── pipe.rs                 # PipeMode (upstream: tui:shell) ─── EXTEND with --context, --narrate, reflect>, lobby
├── schema.rs               # GameState JSON (TRUSTED, 16 tests) ── DO NOT MODIFY
├── agent_context.rs        # NEW (Slice 1) ── AgentContext load/save/identity
├── journal.rs              # NEW (Slice 2) ── Journal append-only markdown
├── narration.rs           # NEW (Slice 5) ── NarrationEngine prose generation
├── shell.rs               # Shell (upstream: tui:shell)
├── events.rs              # Events (upstream: tui:shell)
├── screens.rs             # Screen variants (upstream: tui:shell) ── ADD Spectate variant
└── screens/
    └── spectate.rs        # NEW (Slice 9) ── SpectateScreen fog-of-war rendering

crates/myosu-play/src/
├── main.rs                # Binary entrypoint (MISSING) ── Slice 1 of play:tui
└── spectate.rs           # NEW (Slice 8) ── SpectatorRelay Unix socket

docs/api/
└── game-state.json        # JSON schema for all 20 game types (TRUSTED)
```

---

## Integration Points

### 1. Pipe Mode CLI Integration (`myosu-play` binary)

The `--pipe`, `--context`, `--narrate`, and `--spectate` flags are dispatched in the `myosu-play` binary. This binary is **not yet built** — it is owned by `play:tui` lane Slice 1.

```
myosu-play --pipe --context ./koan.json --narrate
                           └──────────────┬──────────────┘
                              these flags are Slice 3+6 work
```

**Integration risk**: The `play:tui` lane's binary skeleton must accept these flags before Slices 3, 6, 7, 8, 9 can be wired. Confirm `play:tui` Slice 1 timeline before scheduling Slices 3+.

### 2. Schema Trust Boundary

`schema.rs` (939 lines, 16 tests) is the **most trusted surface** in the lane. It is the event format for the spectator relay and the JSON contract for structured agents.

**Rule**: `schema.rs` must not be modified by later slices without a full test run. It is the stable contract.

### 3. Journal File Convention

The journal file path convention: `{context-dir}/journal.md`

This is set by `agent_context.rs` (Slice 1) and must be consistent with the data directory convention in `play:tui` (`{data-dir}/hands/hand_{N}.json`).

**Action**: Verify `play:tui` data directory convention before Slice 8 (spectator socket path depends on alignment).

### 4. Spectator Socket Path

AC-SP-01 specifies `~/.myosu/spectate/<session_id>.sock` — this must be verified against `play:tui`'s data directory convention before Slice 8.

### 5. Chain Query Stub for Lobby

Slice 7 (lobby) queries the chain or miner for active subnet information. For Phase 0, this must be stubbed with hardcoded data.

**Integration point**: The stub must use the same `SubnetInfo` type that `chain:runtime` will eventually provide. Do not use a ad-hoc type — use the type from `games:traits` `GameType` or define a stub in `agent_context.rs`.

---

## robopoker Dependency (Critical Blocker)

Both `tui:shell` and `games:traits` depend on `robopoker` via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`).

All slices ultimately call into these upstream crates. **Until `robopoker` is migrated to a proper git dependency**, no slice can be tested on a clean checkout or CI environment.

**Resolution owned by**: `games:traits` lane ( Slice 1 of that lane).

**Impact on `agent:experience`**: Slices 1–4 can proceed with local path deps in place. Slices 5–9 require full integration testing — they must not be merged without confirming the robopoker git migration is complete.

---

## Next Integration Checkpoint

After each slice completes, update this adapter to reflect:

1. Which surfaces moved from MISSING → IMPLEMENTED
2. Any new integration points discovered during implementation
3. Changes to file locations or type signatures

The adapter is the **source of truth for integration status** — not the lane spec, not the review. Both of those documents capture intent and judgment; this adapter captures the concrete integration map that downstream consumers (including `chain:runtime`, `miner:service`, and `validator:oracle`) need.
