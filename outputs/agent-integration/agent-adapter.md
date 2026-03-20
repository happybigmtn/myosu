# `agent-integration` — Agent Adapter Integration Specification

## Purpose

This document is the **integration adapter** for the `agent:experience` lane. It synthesizes the reviewed `agent:experience` specification and maps every surface to its implementation dependencies, data flow, and sequencing constraints. It is the bridge between the product specification (`outputs/agent/experience/spec.md`) and the implementation family that will execute the nine slices.

This is not a product re-specification. It assumes the reader has already read `outputs/agent/experience/spec.md` and `outputs/agent/experience/review.md`. It focuses on **integration**, not product intent.

---

## Integration Landscape

```
                        upstream dependencies
                        ════════════════════

    tui:shell (82 tests) ──────────────────────►┐
    games:traits (14 tests) ──────────────────►  │
    play:tui binary (MISSING) ─────────────────► │
    robopoker (path dep, needs git migration) ──►┘

                        agent:experience surfaces
                        ═════════════════════════

    ┌─────────────────────────────────────────────────────────────┐
    │  PipeMode (stdin/stdout)                                   │
    │                                                             │
    │  --pipe [--context <path>] [--narrate]                     │
    │       │           │          │                             │
    │       │           │          └── narration.rs (missing)     │
    │       │           │                                       │
    │       │           └── agent_context.rs (missing)            │
    │       │                   │                               │
    │       │                   └── journal.rs (missing)          │
    │       │                           │                        │
    │       │                           └── reflect> prompt       │
    │       │                                                   │
    │       └── lobby (pipe mode, no --subnet)                   │
    │               └── chain query stubbed for Phase 0           │
    │                                                             │
    │  schema.rs (TRUSTED, 16 tests)                            │
    │  GameState JSON ──► pipe_output() OR narration.rs          │
    │                                                             │
    │  SpectatorRelay (Phase 0: Unix socket)                    │
    │  ~/.myosu/spectate/<session_id>.sock                      │
    └─────────────────────────────────────────────────────────────┘

                        downstream consumers
                        ═══════════════════

    External agents (LLM/bot/script) ──► stdin/stdout pipe ──► Myosu
    Spectator clients ────────────────► Unix socket ─────────► GameEvents
```

---

## Surface-by-Surface Integration Map

### 1. `PipeMode` Driver — Integration Root

**File**: `crates/myosu-tui/src/pipe.rs` (extend)

The `PipeMode` struct is the integration hub. All agent-facing surfaces attach through it.

```
PipeMode::new(
    game: Box<dyn GameRenderer>,   // from tui:shell (upstream)
    context_path: Option<PathBuf>, // --context flag → AgentContext
    narrate: bool,                 // --narrate flag → NarrationEngine
    reflect: bool,                 // always true in pipe mode
)
```

**Key integration points:**

| Flag / Field | Target Module | Integration Contract |
|---|---|---|
| `--context <path>` | `agent_context.rs` | Load on `PipeMode::new()`, save on `PipeMode::drop()` |
| `--narrate` | `narration.rs` | Conditionally route `render()` through `NarrationEngine::narrate()` |
| `reflect>` prompt | `journal.rs` | After each `HAND COMPLETE`, read stdin; append to journal if non-empty |
| No `--subnet` | internal | Render lobby; delegate game selection to stdin command |
| Spectator relay | `spectate.rs` | `SpectatorRelay::emit()` called on every game event |

**Existing wiring**: `PipeMode` already exists with 6 passing tests and `--pipe` flag. The missing wiring is: `context_path`, `narrate`, `reflect>`, and lobby.

---

### 2. `AgentContext` — Persistent Identity

**File**: `crates/myosu-tui/src/agent_context.rs` (new)

```rust
pub struct AgentContext {
    pub identity: Identity,
    pub memory: Memory,
    pub journal: Vec<JournalEntry>,
}

impl AgentContext {
    pub fn load(path: &Path) -> Result<Self, ContextError>;
    pub fn save(&self, path: &Path) -> Result<(), ContextError>;
    pub fn default() -> Self;
    pub fn append_journal_entry(&mut self, entry: JournalEntry);
}
```

**Schema** (from AX-AX-01):
```json
{
  "identity": { "name": "koan", "created": "...", "games_played": 1847, "preferred_game": "nlhe-hu" },
  "memory": { "session_count": 23, "lifetime_result": "+342bb", "observations": [...] },
  "journal": [ { "session": 23, "hand": 47, "reflection": "..." } ]
}
```

**Integration contract**: `AgentContext` is a pure data structure with serde serialization. It has no upstream dependencies beyond the filesystem. It is loaded once at `PipeMode::new()` and saved on `PipeMode::drop()`. The journal entries are appended by `journal.rs` after each hand.

**Dependence order**: Slice 1 in the `agent:experience` spec. No upstream dependencies beyond `tui:shell` (which is already trusted).

---

### 3. `Journal` — Append-Only Markdown Artifact

**File**: `crates/myosu-tui/src/journal.rs` (new)

```rust
pub struct Journal {
    path: PathBuf,
}

impl Journal {
    pub fn open(context_dir: &Path) -> Self;
    pub fn append_hand_entry(&mut self, entry: HandEntry) -> Result<(), JournalError>;
    pub fn append_session_summary(&mut self, summary: SessionSummary) -> Result<(), JournalError>;
}
```

**Output format**: append-only markdown at `{context-dir}/journal.md`:
```markdown
# journal of koan

## session 23 — 2026-03-16

### hand 47
board: T♠ 7♥ 2♣ → T♠ 7♥ 2♣ 9♦ → T♠ 7♥ 2♣ 9♦ Q♣
held: A♠ K♥
result: +14bb (showdown)

[reflection text if provided]

## session summary
hands: 47, result: +28bb (+0.60 bb/hand)
```

**Integration contract**: `Journal` is instantiated by `PipeMode` using the same `context_path`. After each hand, `PipeMode` calls `journal.append_hand_entry()`. The `reflect>` prompt result is passed as part of the entry. The file is opened in append mode and is never truncated or rewritten.

**Dependence order**: Slice 2. Depends on `AgentContext` (Slice 1) for the context directory convention, but `Journal` itself is independent.

---

### 4. `NarrationEngine` — Prose Rendering

**File**: `crates/myosu-tui/src/narration.rs` (new)

```rust
pub struct NarrationEngine {
    context: AgentContext,
}

impl NarrationEngine {
    pub fn narrate(&self, state: &GameState) -> String;
}
```

**Output example** (from AX-AX-03):
```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
...
```

**Board texture classification** (required capability):
- **dry**: 3+ suits, no connectors, no pairs → "the kind of board that rewards the player who arrived with the stronger range"
- **wet**: monotone or connected → "a wet board, where flush draws and straight possibilities reward aggression"
- **paired**: one pair → "a paired board, where trips and two-pair become dangerous"
- **connected**: 4+ straight possibilities → "a connected board where the Nuts shift turn to river"

**Integration contract**: `NarrationEngine` wraps an `AgentContext` to access session history (for session arc: "you are up 14bb over 47 hands") and opponent observations. When `PipeMode.narrate == true`, `render()` delegates to `narration_engine.narrate(&game_state)` instead of `pipe_output()`.

**Dependence order**: Slice 5. Requires `AgentContext` (Slice 1) for session arc data.

---

### 5. Lobby + Game Selection — Pipe Mode Extension

**File**: `crates/myosu-tui/src/pipe.rs` (extend)

When `--pipe` is used without `--subnet`:
```
MYOSU/LOBBY
subnets:
  1 nlhe-hu    12 miners  13.2 mbb/h  ACTIVE
  2 nlhe-6max  18 miners  15.8 mbb/h  ACTIVE
>
```

Agent types `info 1` for details, then `1` to select. Chain query is stubbed with hardcoded data for Phase 0 (real chain integration is Phase 4, blocked on `chain:runtime`).

**Integration contract**: Lobby rendering is part of `PipeMode::run()`. Subnet selection changes the active game type and reinitializes the game loop. The lobby uses the same `GameRenderer::pipe_output()` mechanism as normal play, just with a different state machine.

**Dependence order**: Slice 7. Requires `myosu-play` binary (from `play:tui` Slice 1). Blocked on `play:tui` lane completing its binary skeleton.

---

### 6. `SpectatorRelay` — Unix Socket Event Stream

**File**: `crates/myosu-play/src/spectate.rs` (new)

```rust
pub struct SpectatorRelay {
    session_id: Uuid,
    socket_path: PathBuf,
}

impl SpectatorRelay {
    pub fn new(session_id: Uuid) -> Self;
    pub fn emit(&mut self, event: GameEvent) -> Result<(), RelayError>;
    pub fn socket_path(&self) -> &Path;
}
```

**Socket path**: `~/.myosu/spectate/<session_id>.sock`

**Fog-of-war enforcement**: Hole cards are stripped at the relay, not at the renderer. The relay receives full `GameEvent` (with hole cards) and redacts them before sending to the socket. Only `showdown` events transmit revealed hole cards.

**Event format** (JSON, one object per line):
```json
{"type":"hand_start","session":"uuid","hand":47,"players":[{"seat":0,"stack":94},{"seat":1,"stack":94}]}
{"type":"board","board":"Ts 7h 2c","pot":12,"street":"flop"}
{"type":"action","player":0,"action":"raise","amount":6}
{"type":"showdown","board":"Ts 7h 2c 9d Qc","hole_cards":{"player0":"As Kh","player1":"Qh Jh"},"pot":28,"result":{"player0":14,"player1":-14}}
```

**Integration contract**: `SpectatorRelay` is instantiated by `PipeMode` at session start. `emit()` is called after every game event. The relay manages the Unix socket listener. Phase 1 (WebSocket via miner axon) is blocked on `chain:runtime`.

**Dependence order**: Slice 8. Requires `myosu-play` binary (from `play:tui`). Part of Phase 3.

---

## Upstream Dependency Status

| Dependency | Status | Impact on agent:experience |
|---|---|---|
| `tui:shell` | **TRUSTED** (82 tests) | `GameRenderer`, `PipeMode`, `Shell` all available and tested |
| `games:traits` | **TRUSTED** (14 tests) | `CfrGame`, `GameType`, `Profile` available |
| `play:tui` binary | **MISSING** | Slices 3, 6, 7, 8, 9 all need `myosu-play` main.rs CLI dispatch |
| `robopoker` git migration | **BLOCKER** | Absolute path deps in `games:traits` and `tui:shell`; `cargo build` fails on clean checkout |
| `chain:runtime` | **MISSING** (stubbed) | Slice 7 lobby uses hardcoded data; Slice 8 WebSocket upgrade blocked |

**Critical path for robopoker migration**: The `games:traits` lane owns the robopoker git migration resolution. All of `agent:experience` ultimately calls through `games:traits` or `tui:shell`, which call into robopoker. Slices 1–4 can begin immediately (no robopoker API calls), but Slices 5–9 require full integration testing which is blocked until the dependency is resolved.

---

## Data Flow Summary

```
Agent connects
    │
    ▼
myosu-play --pipe --context ./koan.json --narrate
    │
    ▼
PipeMode::new()
    │  Loads AgentContext from ./koan.json
    │  Opens Journal at {context-dir}/journal.md
    │  Creates NarrationEngine with AgentContext
    │  Creates SpectatorRelay (Unix socket)
    │
    ├──────────────────────────────┐
    ▼                              ▼
Lobby presented              Game loop starts
(no --subnet)               (--subnet 1 or default)
    │                              │
    │Subnet selection ─────────────┤
    │                              │
    ▼                              ▼
For each hand:
    GameState ──► GameRenderer::render()
                     │
                     ├── (narrate=false) ──► pipe_output() ──► stdout
                     │
                     └── (narrate=true) ──► NarrationEngine::narrate() ──► stdout
                                             │
                              ┌──────────────┘
                              ▼
                         SpectatorRelay::emit(game_event)
                              │
                              ▼
                         ~/.myosu/spectate/<id>.sock (JSON lines)
                              │
                              ▼
    HAND COMPLETE ──► stdout
    reflect> ──► stdin ──► Journal::append_hand_entry()
    │
    ▼
PipeMode::drop()
    │  Saves AgentContext to ./koan.json
    │  Saves final journal entries
    │  Closes SpectatorRelay socket
    │
    ▼
Agent disconnected
```

---

## Slice Execution Order

Based on the dependency analysis, the nine slices should be executed in this order:

```
Phase 1 (no external deps beyond tui:shell):
  Slice 1: agent_context.rs          ──► outputs/agent-integration/
  Slice 2: journal.rs               ──► depends on Slice 1
  Slice 3: --context wiring          ──► depends on Slice 1
  Slice 4: reflect> prompt           ──► depends on Slices 1, 2

Phase 2 (depends on Phase 1 + myosu-play binary from play:tui):
  Slice 5: narration.rs             ──► depends on Slice 1 (AgentContext)
  Slice 6: --narrate wiring         ──► depends on Slice 5
  Slice 7: lobby + game selection    ──► depends on play:tui Slice 1 (binary)

Phase 3 (depends on Phase 2 + play:tui binary complete):
  Slice 8: SpectatorRelay            ──► depends on play:tui Slice 1
  Slice 9: SpectateScreen            ──► depends on Slice 8

Phase 4 (depends on chain:runtime):
  Lobby real chain query             ──► lobby stub replaced
  Spectator WebSocket upgrade         ──► Unix socket replaced
```

**Robopoker dependency gate**: Slices 1–4 can begin immediately. Slices 5–9 require confirming the `robopoker` git migration is complete (owned by `games:traits` lane). Without it, `cargo build` fails on any clean environment.

**play:tui binary gate**: Slices 3, 6, 7, 8, 9 all require modifications to `myosu-play`'s `main.rs` CLI dispatch. These should not proceed past Slice 4 without confirming `play:tui` Slice 1 is complete or concurrent.

---

## Adapter Boundary Contract

The `agent:experience` lane is a **terminal lane** — it has no trusted downstream outputs. All surfaces produced by this lane are consumed by external agents or spectator clients.

The integration adapter boundary is defined by what crosses the `PipeMode` interface:

**In (agent → Myosu)**:
- `Action` (fold/call/raise/bet) via stdout text parse
- `Reflection` text via stdin after hand
- `Subnet selection` via stdin command in lobby
- `Context file path` via `--context` flag at startup

**Out (Myosu → agent)**:
- `GameState` via `pipe_output()` or `narration.rs` prose
- `HAND COMPLETE` + result via stdout
- `reflect>` prompt via stdout
- `Lobby` content via stdout (no --subnet mode)
- `SpectatorRelay` events via Unix socket JSON

**Not crossed** (internal only):
- `AgentContext` internal structure — only serialized to/from JSON file
- `Journal` file format — append-only, never exposed to opponent
- `SpectatorRelay` fog-of-war — enforced at relay, not at renderer

---

## Decision Record

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-20 | Phase 1 (Slices 1–4) can begin immediately | Only depends on `tui:shell` which is already trusted |
| 2026-03-20 | Phase 2 (Slices 5–7) waits for robopoker git migration and play:tui binary | Full integration testing blocked without resolved external deps |
| 2026-03-20 | Phase 3 (Slices 8–9) waits for play:tui binary | Spectator relay requires `myosu-play` binary socket management |
| 2026-03-20 | Phase 4 is future work blocked on chain:runtime | Lobby chain query and WebSocket upgrade are not Phase 0 scope |
