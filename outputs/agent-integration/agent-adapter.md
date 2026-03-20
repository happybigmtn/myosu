# `agent:experience` Integration Adapter

## Purpose

This document describes how the `agent:experience` lane integrates into the Myosu product frontier — specifically the contract between `agent:experience` and its upstream dependencies, the surfaces it exposes to agents, and the implications for the product-wide roadmap.

This is the **integration adapter** for the `agent` unit within the `product` frontier. It is distinct from `outputs/agent/experience/spec.md`, which is the lane's own specification. This document is the product-level integration view.

---

## Integration Contract

### What `agent:experience` Requires from Upstream

| Upstream Lane | What It Provides | Trust Level |
|--------------|-----------------|-------------|
| `tui:shell` | `GameRenderer` trait, `PipeMode`, `Shell`, `Events`, `Theme` | **TRUSTED** — 82 tests pass |
| `games:traits` | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` | **TRUSTED** — 14 tests pass |
| `play:tui` | `myosu-play` binary skeleton (CLI dispatch for `--pipe`, `--context`, `--narrate`, `--spectate`) | **BLOCKED** — binary does not exist yet |

### What `agent:experience` Provides to the Product

| Surface | Description | Consumer |
|---------|-------------|----------|
| `AgentContext` | Persistent identity, memory, journal — serialized JSON at `~/.myosu/agents/<name>/context.json` | Agents (LLMs, bots, scripts) |
| `Journal` | Append-only markdown autobiography at `~/.myosu/agents/<name>/journal.md` | Agents |
| `PipeMode` extension | `--pipe --context <path> --narrate` CLI surface | Agents via stdin/stdout |
| `SpectatorRelay` | Unix-domain socket event stream at `~/.myosu/spectate/<session_id>.sock` | Spectator clients |
| `game-state.json` schema | `docs/api/game-state.json` — canonical machine-readable game state | All agents and chain clients |

---

## Product-Level Dependency Graph

```
product frontier
│
├──► play:tui
│    │
│    └──► tui:shell (TRUSTED)
│    └──► games:traits (TRUSTED)
│
├──► agent (THIS LANE — depends on play:tui milestone)
│    │
│    ├──► tui:shell (TRUSTED)           ──► Slice 1–4 only
│    ├──► games:traits (TRUSTED)        ──► Slice 1–4 only
│    └──► play:tui (REVIEWED milestone) ──► Slice 3+ needs binary
│
└──► sdk/core
     └──► agent:experience surfaces (future consumer)

```

---

## Sequential Readiness of Implementation Slices

The 9 implementation slices of `agent:experience` have **three dependency tiers**:

### Tier 1 — Immediate (no binary required)

**Slices 1–4 depend only on `tui:shell` and `games:traits`, both already trusted.**

These slices can begin immediately without waiting for `play:tui`:

- **Slice 1**: `agent_context.rs` — `AgentContext` struct, load/save, default identity
- **Slice 2**: `journal.rs` — append-only markdown writer
- **Slice 3**: `--context` flag wiring in `PipeMode`
- **Slice 4**: `reflect>` prompt after hand completion

### Tier 2 — Requires `play:tui` binary skeleton (concurrent with `play:tui` Slice 1)

**Slice 3** (already started above) wires `--context` to the CLI. Once `play:tui` Slice 1 creates `crates/myosu-play/`, the wiring compiles.

- **Slice 5**: `narration.rs` — `NarrationEngine`, board texture, session arc
- **Slice 6**: `--narrate` flag wiring
- **Slice 7**: Lobby + game selection in pipe mode (stubbed chain data for Phase 0)

### Tier 3 — Requires `play:tui` binary + `chain:runtime` (future phases)

- **Slice 8**: `SpectatorRelay` (Unix socket) — depends on `play:tui` binary
- **Slice 9**: `SpectateScreen` — depends on `play:tui` binary

---

## Data Directory Convention

All agent-persisted data follows this convention:

| Artifact | Path |
|----------|------|
| Agent context JSON | `~/.myosu/agents/<agent-name>/context.json` |
| Agent journal markdown | `~/.myosu/agents/<agent-name>/journal.md` |
| Spectator socket | `~/.myosu/spectate/<session_id>.sock` |

This convention must stay aligned with `play:tui`'s data directory (`{data-dir}/hands/hand_{N}.json`). The base path `~/.myosu/` is shared; sub-paths are lane-scoped.

---

## Interface to Chain and Miner (Future Phases)

The `agent:experience` lane is designed to evolve from pure local mode to chain-connected mode:

| Phase | Lobby Source | Spectator Transport |
|-------|-------------|---------------------|
| Phase 0 (current design) | Stubbed hardcoded data | Unix domain socket |
| Phase 1 | Miner axon HTTP | Upgrade to WebSocket |
| Phase 2 | Chain on-chain registry | WebSocket relay |

The JSON schema (`docs/api/game-state.json`) is the common wire format across all phases. The schema is already complete and trusted (covers 10 game types, fully validated).

---

## Adapter Decisions

These are product-level decisions made during integration analysis:

**Decision 1 — Slice tiering separates immediate work from blocked work.**
Rationale: `tui:shell` and `games:traits` are trusted with passing tests. There is no reason to wait for `play:tui` before starting Slices 1–4. The lane's own review acknowledges this ("Slices 1–2 can begin immediately").

**Decision 2 — `play:tui` binary is the primary integration contract.**
Rationale: All agent-facing CLI flags (`--pipe`, `--context`, `--narrate`, `--spectate`) are dispatched through `myosu-play`'s `main.rs`. Until that binary exists, no agent surface can be end-to-end proven. This is a sequential dependency, not a parallel one.

**Decision 3 — Data directory convention is shared but lane-scoped.**
Rationale: Both `play:tui` and `agent:experience` write to `~/.myosu/`. Sub-paths (`agents/`, `spectate/`, `hands/`) must not collide. The convention above establishes that boundary.

**Decision 4 — robopoker git migration is a prerequisite for CI, not for Slice 1–4.**
Rationale: Slices 1–4 (`agent_context.rs`, `journal.rs`) do not call robopoker directly. They depend on `tui:shell`'s `PipeMode` and `games:traits`'s trait re-exports. The robopoker migration (owned by `games:traits` Slice 1) must complete before integration testing that exercises the full game loop, but does not block Slice 1–4 implementation or unit testing.
