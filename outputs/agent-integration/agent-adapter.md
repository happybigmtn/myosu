# Agent Integration Adapter

## Purpose

This adapter synthesizes the `agent:experience` lane output into a form consumable by the product frontier and downstream implementation lanes. It records the lane boundary decisions, the adapter contract between agent surfaces and the broader Myosu system, and the integration points that implementation slices will depend on.

## Lane Boundary Recap

```
                            agent:experience
                            ┌──────────────────────────────────────────────┐
upstream                    │                                              │
tui:shell ────────────────► │  GameRenderer ──► pipe_output()             │
  (82 tests, trusted)       │  pipe.rs (PipeMode driver)                   │
                            │  --context flag → agent_context.rs           │
upstream                    │  --narrate flag → narration.rs               │
games:traits ──────────────► │  journal.rs                                  │
  (14 tests, trusted)       │                                              │
                            │  docs/api/game-state.json + schema.rs         │
                            │  (GameState JSON schema — trusted)          │
                            │                                              │
untrusted                   │  SpectatorRelay (AC-SP-01)                   │
miner axon (future) ──────► │  Unix socket → future WS upgrade             │
                            │                                              │
                            │  myosu-play binary (extends play:tui binary) │
                            │  --pipe --context <path> --narrate          │
                            └──────────────────────────────────────────────┘
```

## Trusted Upstream Contracts

| Upstream | Trust Level | Contract Surface |
|----------|-------------|------------------|
| `tui:shell` | TRUSTED | `Shell`, `GameRenderer` trait, `PipeMode`, `Events`, `Theme` — 82 tests pass |
| `games:traits` | TRUSTED | `CfrGame`, `Profile`, `GameConfig`, `GameType` — 14 tests pass |
| `docs/api/game-state.json` | TRUSTED | Complete JSON schema with 10 game types, exhaustive `legal_actions` |
| `crates/myosu-tui/src/schema.rs` | TRUSTED | Full Rust implementation, 16 tests pass |

## Trusted Downstream Contracts

None — `agent:experience` is a terminal lane that feeds into the execution plane only.

## Implementation Slices (Smallest Honest First)

| Slice | File | What | Gate |
|-------|------|------|------|
| 1 | `crates/myosu-tui/src/agent_context.rs` | `AgentContext` with load/save/default | `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip` |
| 2 | `crates/myosu-tui/src/journal.rs` | `Journal` append-only markdown writer | `cargo test -p myosu-tui journal::tests::append_hand_entry` |
| 3 | `crates/myosu-tui/src/pipe.rs` | `--context` flag wiring | Agent plays 10 hands → restart → memory preserved |
| 4 | `crates/myosu-tui/src/pipe.rs` | `reflect>` prompt after hand | `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand` |
| 5 | `crates/myosu-tui/src/narration.rs` | `NarrationEngine` prose engine | `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture` |
| 6 | `crates/myosu-tui/src/pipe.rs` | `--narrate` flag wiring | Prose output vs terse output from same game state |
| 7 | `crates/myosu-tui/src/pipe.rs` | Lobby + game selection | `cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag` |
| 8 | `crates/myosu-play/src/spectate.rs` | `SpectatorRelay` Unix socket | `cargo test -p myosu-play spectate::tests::relay_emits_events` |
| 9 | `crates/myosu-tui/src/screens/spectate.rs` | `SpectateScreen` fog-of-war | `cargo test -p myosu-tui spectate::tests::renders_fog_of_war` |

## Phase Dependencies

```
Phase 1 (Agent Identity — tui:shell dependency):
  Slice 1 → Slice 2 → Slice 3 → Slice 4

Phase 2 (Narration + Pipe Mode — Phase 1 dependency):
  Slice 5 → Slice 6 → Slice 7

Phase 3 (Spectator — play:tui binary + Phase 1 dependency):
  Slice 8 → Slice 9

Phase 4 (Chain-Connected — chain:runtime dependency):
  Lobby queries miner axon; spectator upgrades to WebSocket
```

## Blockers Inherited from Upstream

| Blocker | Owner | Impact on This Lane |
|---------|-------|---------------------|
| `robopoker` git migration | `games:traits` | All slices call into `games:traits` or `tui:shell`; full integration testing blocked until resolved |
| `myosu-play` binary skeleton | `play:tui` | Slices 3, 6, 7, 8, 9 require CLI dispatch modifications |
| Chain discovery stub for lobby | `chain:runtime` | Slice 7 lobby queries chain; stubbed for Phase 0 |
| Spectator socket path convention | `play:tui` | Slice 8 uses `~/.myosu/spectate/<session_id>.sock`; needs confirmation against `play:tui` data dir |

## Key Integration Points

### 1. Pipe Mode Driver (`pipe.rs`)

`PipeMode` is the central adapter. It receives game state from `games:traits` and routes to:
- `pipe_output()` for terse text (machine agents)
- `NarrationEngine` for prose (human-in-the-loop agents)
- `Journal` for append-only hand record
- `SpectatorRelay` for event emission

The `--context` flag loads `AgentContext` on startup and saves on drop. The `--narrate` flag switches rendering mode.

### 2. Schema as Integration Contract

`schema.rs` defines `GameState` and `LegalAction` — the canonical game state representation. This is the same schema used by:
- Pipe mode text rendering
- Narration engine prose generation
- Spectator relay JSON events
- Future HTTP/WS API (Phase 2)

### 3. Agent Context File Format

```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [...] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

This format is consumed by narration (session arc, opponent history) and is the persistence model for agent memory across sessions.

### 4. Spectator Relay Contract (AC-SP-01)

Phase 0: Unix domain socket at `~/.myosu/spectate/<session_id>.sock`
- Emits `GameEvent` JSON lines
- Fog-of-war enforced at relay (hole cards never during play; only after `showdown`)
- Handles disconnected listeners gracefully

Phase 1: WebSocket upgrade via miner axon (blocked on `chain:runtime`)

## What This Lane Does NOT Own

- `games:traits` trait definitions (upstream)
- `tui:shell` rendering primitives (upstream)
- `myosu-play` binary skeleton (owned by `play:tui`)
- Chain discovery and subnet registration (owned by `chain:runtime`)
- Miner axon HTTP endpoints (future Phase 1)
- Agent-to-agent social interaction
- Agent autonomy over system parameters
- Emotion/affect modeling

## Adapter Quality Gates

Before this adapter can be considered integrated:

1. `robopoker` must be migrated from absolute filesystem paths to git dependencies
2. `myosu-play` binary skeleton must exist in `play:tui` lane
3. Slice 1 (`agent_context.rs`) must pass roundtrip test
4. Slice 2 (`journal.rs`) must pass append test
5. The `GameRenderer::pipe_output()` contract must remain stable (no breaking changes to the trait)

## Relationship to Other Product Lanes

| Lane | Relationship |
|------|-------------|
| `play:tui` | Hard dependency for binary skeleton (Slice 3+); shares `schema.rs` |
| `games:traits` | Hard upstream; all game state flows through this lane's types |
| `chain:runtime` | Soft upstream (Phase 4); lobby and spectator WS depend on it |
| `miner:service` | Future consumer of agent context; miner may expose agent-facing HTTP |
| `validator:oracle` | No direct relationship; validator scoring is orthogonal to agent experience |
