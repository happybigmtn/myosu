# Agent Integration Adapter

## Purpose

This document is the **integration adapter** for the `agent:experience` lane. It maps the lane's integration surfaces, describes how agent-facing capabilities connect to the broader Myosu product, and serves as the integration contract between `agent:experience` and downstream consumers.

This is not a specification — `outputs/agent/experience/spec.md` is the specification. This is an **integration concern** document that answers: how does `agent:experience` wire into the product, what does it depend on, and what do downstream lanes need to know?

---

## Integration Context

```
                         ┌─────────────────────────────────────────────┐
                         │            Myosu Product                     │
                         │                                              │
  agent (LLM/bot) ─────► │  ┌─────────────────────────────────────┐  │
                         │  │        agent:experience lane          │  │
                         │  │                                       │  │
                         │  │  --pipe --context <path> --narrate   │  │
                         │  │  schema.rs (GameState JSON)          │  │
                         │  │  SpectatorRelay (Unix socket)        │  │
                         │  └───────────────┬─────────────────────┘  │
                         │                  │                         │
                         │  ┌───────────────▼─────────────────────┐  │
                         │  │              tui:shell               │  │
                         │  │  Shell, GameRenderer, PipeMode        │  │
                         │  └───────────────┬─────────────────────┘  │
                         │                  │                         │
                         │  ┌───────────────▼─────────────────────┐  │
                         │  │           games:traits                │  │
                         │  │  CfrGame, Profile, GameConfig         │  │
                         │  └─────────────────────────────────────┘  │
                         │                                              │
                         │  ┌─────────────────────────────────────┐  │
                         │  │           play:tui                   │  │
                         │  │  myosu-play binary skeleton          │  │
                         │  └─────────────────────────────────────┘  │
                         │                                              │
                         │  ┌─────────────────────────────────────┐  │
                         │  │          chain:runtime                │  │
                         │  │  miner axon, subnet registry (Phase 2)│  │
                         │  └─────────────────────────────────────┘  │
                         └─────────────────────────────────────────────┘
```

---

## Integration Surfaces

### Inbound: What `agent:experience` Consumes

| Upstream Lane | Surface | Integration Contract |
|---------------|---------|---------------------|
| `tui:shell` | `GameRenderer` trait | `pipe_output()` renders terse text; `render_state()` renders TUI. Both must produce consistent game state representations. |
| `tui:shell` | `PipeMode` driver | Agent session runs through `PipeMode::run()`. The driver manages stdin/stdout lifecycle. |
| `games:traits` | `CfrGame`, `Profile`, `GameConfig` | Game state types flow through schema.rs into `GameState` JSON. The schema must cover all game types that `games:traits` supports. |
| `play:tui` | `myosu-play` binary | Flags (`--pipe`, `--context`, `--narrate`) are wired into the binary's CLI. The binary is the agent's entry point. |

### Outbound: What `agent:experience` Produces

| Downstream Consumer | Surface | Contract |
|---------------------|---------|----------|
| Agent (LLM/bot/script) | `myosu-play --pipe` | Plain-text stdin/stdout protocol. Never ANSI-escaped in pipe mode. |
| Agent | JSON schema (`docs/api/game-state.json`) | Machine-readable game state consumed by structured agents. Schema is the authoritative contract. |
| Agent | Agent context file (`--context <path>`) | JSON file with identity, memory, journal. Serde-validated on load. |
| Agent | Journal (`journal.md`) | Append-only markdown. Never truncated. One entry per hand with optional reflection. |
| Spectator client | Unix socket (`~/.myosu/spectate/<id>.sock`) | JSON event stream. Fog-of-war enforced at relay (hole cards withheld during play). |

---

## Integration Points by Phase

### Phase 1: Agent Identity (Slices 1–4) — **Ready to implement**

These slices depend only on `tui:shell` and `games:traits`, both already trusted.

| Integration Point | How It Wires |
|-------------------|--------------|
| `agent_context.rs` → `PipeMode` | `AgentContext` loaded on `PipeMode::new()`; saved on `PipeMode::drop()`. Context path passed via `--context` flag. |
| `journal.rs` → `agent_context.rs` | `Journal` is owned by `AgentContext`. Hand entries appended after each hand in pipe mode. |
| `reflect>` prompt → `journal.rs` | After `HAND COMPLETE`, stdin blocks for reflection. Non-empty response appended to current journal entry. |

**Upstream requirements for Phase 1:**
- `tui:shell` trusted (82 tests) — ✅ satisfied
- `games:traits` trusted (14 tests) — ✅ satisfied
- `myosu-play` binary CLI dispatch — ⚠️ needed for Slice 3

### Phase 2: Narration + Pipe Mode (Slices 5–7) — **Blocked on Phase 1**

| Integration Point | How It Wires |
|-------------------|--------------|
| `narration.rs` → `GameState` | `NarrationEngine::narrate(&GameState) -> String` translates game state to prose. |
| `--narrate` flag → `PipeMode` | When `narrate: true`, `PipeMode` uses `NarrationEngine` instead of `pipe_output()`. |
| Lobby → chain stub | Lobby renders without `--subnet`. Chain queries stubbed with hardcoded data for Phase 0. |

**Upstream requirements for Phase 2:**
- Phase 1 Slices 1–4 complete — ⏳ pending
- `play:tui` binary skeleton — ⚠️ needed for Slice 6

### Phase 3: Spectator (Slices 8–9) — **Blocked on play:tui + Phase 1**

| Integration Point | How It Wires |
|-------------------|--------------|
| `SpectatorRelay` → `GameState` | Relay wraps game events from `PipeMode` session into JSON lines on Unix socket. |
| `SpectateScreen` → relay socket | `Screen::Spectate` variant connects to `~/.myosu/spectate/<id>.sock`; renders with fog-of-war. |
| Fog-of-war enforcement | Hole cards stripped at relay, not renderer. `showdown` event triggers card reveal. |

**Upstream requirements for Phase 3:**
- `play:tui` binary exists — ⚠️ owned by `play:tui` lane
- Phase 1 Slices 1–4 complete — ⏳ pending

### Phase 4: Chain-Connected — **Blocked on chain:runtime**

| Integration Point | How It Wires |
|-------------------|--------------|
| Lobby → miner axon | `info <id>` queries miner HTTP endpoint for active subnet data. |
| Spectator → miner axon WS | Relay upgrades from Unix socket to WebSocket via miner axon. |

**Upstream requirements for Phase 4:**
- `chain:runtime` lane complete — ⏳ pending

---

## Robopoker Dependency Note

`agent:experience` depends on `tui:shell` and `games:traits`, both of which depend on `robopoker` via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This is documented in `outputs/games/traits/review.md` as the highest-priority fix.

**Impact on this lane**: All slices ultimately call through `games:traits` into `robopoker`. Until the git migration resolves, `cargo build` and `cargo test` will fail on clean checkout or CI.

**Who owns resolution**: `games:traits` lane owns the robopoker migration. This lane should not proceed past Slice 4 without confirming the dependency is resolved.

---

## Integration Contract for Downstream Lanes

### For `play:tui` lane

The `myosu-play` binary must expose these CLI flags for `agent:experience`:

| Flag | Purpose | Slice |
|------|---------|-------|
| `--pipe` | Enable pipe mode | ✅ exists |
| `--context <path>` | Load agent context file | Slice 3 |
| `--narrate` | Enable narration mode | Slice 6 |
| `--spectate <session_id>` | Spectate a running session | Slice 8 |

The binary is the integration point. All flags route through `PipeMode`.

### For `chain:runtime` lane

Lobby queries (Slice 7, Phase 4) require:

| Query | Endpoint | Response |
|-------|----------|----------|
| List subnets | `GET /subnets` | `{ "subnets": [{ "id": 1, "game": "nlhe-hu", "miners": 12, "mbb_h": 13.2, "status": "ACTIVE" }] }` |
| Subnet detail | `GET /subnets/<id>` | `{ "id": 1, "game": "nlhe-hu", "miners": 12, "mbb_h": 13.2, "status": "ACTIVE", "exploitability": 32.1 }` |

These are Phase 4 requirements. Phase 0 lobby uses hardcoded stub data.

### For `miner` lane

Spectator WebSocket upgrade (Slice 9, Phase 4) requires the miner axon to expose:

| Endpoint | Purpose |
|----------|---------|
| `WS /spectate/<session_id>` | Stream game events to spectator clients |

---

## Summary

`agent:experience` is a **terminal integration lane** — it consumes upstream trusted surfaces and produces agent-facing capabilities. Its integration contract is:

1. **Inbound**: Relies on `tui:shell` (trusted), `games:traits` (trusted), and `play:tui` binary (in progress)
2. **Outbound**: Produces pipe protocol, JSON schema, agent context, journal, and spectator relay
3. **Phase 1 is unblocked**: Slices 1–2 (`agent_context.rs`, `journal.rs`) can begin immediately
4. **Phase 2–3 blocked by**: `play:tui` binary skeleton and Phase 1 completion
5. **Phase 4 blocked by**: `chain:runtime`
6. **Cross-lane risk**: Robopoker git migration (owned by `games:traits`) must resolve before Phase 1 testing