# `agent-integration` Lane Specification

## Lane Purpose

`agent-integration` is the **integration-adapter lane** for the `agent:experience` product lane. It owns the bridge between the reviewed `agent:experience` spec (`outputs/agent/experience/spec.md`, judgment: KEEP) and the actual code changes that implement agent-facing surfaces in the `myosu-play` binary and `myosu-tui` crate.

This lane does not rewrite the `agent:experience` spec. It translates that spec into concrete integration points: the exact files to create or modify, the CLI flag wiring, the module signatures, and the data-flow contracts between the new modules and the existing `tui:shell` and `games:traits` upstreams.

`★ Insight ─────────────────────────────────────`
This lane exists because a reviewed spec is not executable code. The `agent:experience` review correctly identified 9 implementation slices, but someone must decide how those slices plug into the existing `myosu-tui` crate structure, which files own which traits, and how the CLI dispatch routes the new flags. The adapter is the integration decision record.
`─────────────────────────────────────────────────`

---

## Integration Context

### Where `agent:experience` Lives in the Crate Hierarchy

```
crates/
  myosu-play/           ← binary crate; owns CLI entry, flag parsing, main()
    src/
      main.rs            ← extended with --pipe --context --narrate --spectate
      spectate.rs       ← NEW (AC-SP-01): SpectatorRelay + socket management

  myosu-tui/            ← shared TUI library
    src/
      lib.rs
      schema.rs         ← TRUSTED: GameState JSON (already implemented)
      pipe.rs            ← EXTEND: add context_path, narrate, reflect>
      agent_context.rs  ← NEW: AgentContext load/save/journal
      narration.rs       ← NEW: NarrationEngine board-texture + prose
      journal.rs         ← NEW: append-only markdown journal
      screens/
        mod.rs           ← EXTEND: add SpectateScreen variant
        spectate.rs      ← NEW (AC-SP-02): fog-of-war rendering
```

### What `agent:experience` Consumes (Upstreams)

| Upstream | What it provides to this lane | Trust state |
|----------|-------------------------------|-------------|
| `tui:shell` | `Shell`, `ScreenManager`, `GameRenderer` trait, `PipeMode`, `Events`, `Theme` | 82 tests pass; TRUSTED |
| `games:traits` | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` | 14 tests pass; TRUSTED |
| `play:tui` | `myosu-play` binary skeleton with CLI dispatch | Binary missing; PARTIAL |

### What `agent:experience` Produces (Downstream Contract)

`agent:experience` is a terminal lane. Its outputs are consumable by:

| Consumer | What it receives | Interface |
|----------|-------------------|-----------|
| LLM agents | Pipe-mode stdin/stdout stream | `myosu-play --pipe --context <path>` |
| Script agents | JSON schema over HTTP/WS (Phase 2) | `GameState` JSON via `schema.rs` |
| Spectator clients | Unix-domain socket event stream | `~/.myosu/spectate/<session_id>.sock` |
| Human players | Rich narration prose | `myosu-play --pipe --narrate` |

---

## Integration Map: New Modules and Their Contracts

### 1. `agent_context.rs` — `AgentContext`

**File:** `crates/myosu-tui/src/agent_context.rs`

```rust
pub struct AgentContext {
    pub identity: Identity,
    pub memory: Memory,
    pub journal: Vec<JournalEntry>,
}

impl AgentContext {
    /// Load from a JSON file. Missing file → returns Default (new identity).
    pub fn load(path: &Path) -> Result<Self>;
    /// Save to JSON file. Never truncates; only writes complete objects.
    pub fn save(&self, path: &Path) -> Result<()>;
    /// Create a default identity with a generated name and creation timestamp.
    pub fn default() -> Self;
    /// Append a journal entry (reflection may be empty string).
    pub fn append_journal(&mut self, entry: JournalEntry);
}
```

**Dependencies:** `serde`, `serde_json`, `chrono` (for timestamps)

**Integration point:** `PipeMode::new()` calls `AgentContext::load(context_path)` if `--context` is provided; `PipeMode::drop` calls `ctx.save()`.

---

### 2. `journal.rs` — `Journal`

**File:** `crates/myosu-tui/src/journal.rs`

```rust
pub struct Journal {
    path: PathBuf,
}

impl Journal {
    /// Open (or create) a markdown journal at the given path.
    pub fn open(path: &Path) -> Result<Self>;
    /// Append a hand entry — never truncates the file.
    pub fn append_hand_entry(&mut self, entry: HandEntry) -> Result<()>;
    /// Append a session summary line.
    pub fn append_session_summary(&mut self, summary: &str) -> Result<()>;
}

pub struct HandEntry {
    pub session: u32,
    pub hand: u32,
    pub board: String,       // e.g., "T♠ 7♥ 2♣"
    pub held: String,         // e.g., "A♠ K♥"
    pub result: String,       // e.g., "+14bb (showdown)"
    pub session_result: String, // e.g., "+28bb over 47 hands"
    pub reflection: Option<String>,
}
```

**Integration point:** `pipe.rs` calls `journal.append_hand_entry(...)` after each `HAND COMPLETE` event. `reflect>` prompt response is stored in `entry.reflection` before appending.

---

### 3. `narration.rs` — `NarrationEngine`

**File:** `crates/myosu-tui/src/narration.rs`

```rust
pub struct NarrationEngine {
    context: Option<AgentContext>,
}

impl NarrationEngine {
    pub fn new(context: Option<AgentContext>) -> Self;
    /// Generate narrated prose for a game state.
    pub fn narrate(&self, state: &GameState) -> String;
    /// Board texture: "dry" | "wet" | "connected" based on suit/sequence analysis.
    fn board_texture(cards: &[Card]) -> &'static str;
    /// Stack trajectory from context memory → e.g., "climbed from 80bb to 120bb"
    fn session_arc(&self, state: &GameState) -> Option<String>;
}

pub enum GameState {
    /* existing enum from schema.rs */
}
```

**Key narrative elements (from AC-AX-03):**
- Board texture: "dry", "wet", or "connected" classification
- Hand strength narrative: "you hold A♠ K♥ in the big blind. 94bb behind."
- Session arc: stack trajectory from context memory
- Opponent tendency: if available from memory observations

**Integration point:** `PipeMode` holds `Option<NarrationEngine>`; when `narrate: true`, `render()` calls `engine.narrate(&state)` instead of `pipe_output()`.

---

### 4. `spectate.rs` (binary crate) — `SpectatorRelay`

**File:** `crates/myosu-play/src/spectate.rs`

```rust
pub struct SpectatorRelay {
    session_id: SessionId,
    socket_path: PathBuf,
}

impl SpectatorRelay {
    /// Create relay and start listening on Unix-domain socket.
    pub fn new(session_id: SessionId) -> Result<Self>;
    /// Emit a game event. Fog-of-war enforced here: hole cards stripped before emit.
    pub fn emit(&self, event: GameEvent) -> Result<()>;
    /// Socket path: ~/.myosu/spectate/<session_id>.sock
    pub fn socket_path(&self) -> &Path;
}

#[derive(Serialize)]
pub struct GameEvent {
    pub event_type: EventType, // HandStart, BoardDeal, Action, Showdown, HandEnd
    pub street: Street,
    pub actions: Vec<ActionRecord>,
    pub hole_cards: Option<Vec<Card>>, // None during play (fog-of-war); Some after showdown
    pub result: Option<HandResult>,
}
```

**Fog-of-war rule:** `hole_cards` is `None` for all events before `Showdown`. After `Showdown` event, `hole_cards` is populated. Enforced in `emit()` — callers must never pass hole cards for pre-showdown events.

**Integration point:** `myosu-play main.rs` spawns `SpectatorRelay` when `--spectate <session_id>` is given; connects to an existing session's socket.

---

### 5. `screens/spectate.rs` — `SpectateScreen`

**File:** `crates/myosu-tui/src/screens/spectate.rs`

```rust
pub struct SpectateScreen {
    relay: SpectatorRelay,
    events: Vec<GameEvent>,
}

impl Screen for SpectateScreen {
    fn render(&mut self, f: &mut Frame);
    fn handle_input(&mut self, key: Key) -> ScreenAction;
}

enum ScreenAction {
    SwitchSession(SessionId),
    RevealHoleCards,     // 'r' key; valid only after showdown
    ReturnToLobby,       // 'q' key
}
```

**Fog-of-war rendering:** Hole cards shown as `·· ··` during play; `r` key reveals after showdown event is received.

**Integration point:** `ScreenManager` holds `Screen::Spectate` variant; `mod.rs` updated with `SpectateScreen` import.

---

## CLI Integration: `myosu-play` Flag Wiring

**File:** `crates/myosu-play/src/main.rs`

```rust
// New flags (alongside existing --pipe, --subnet)
--context <path>    // Path to agent context JSON file
--narrate           // Use narration engine instead of pipe_output()
--spectate <id>     // Spectate a session by ID (Unix socket client mode)
```

Flag dispatch:
- `--pipe --context <path>` → `PipeMode::new(context_path, false)`
- `--pipe --narrate` → `PipeMode::new(None, true)`
- `--pipe --context <path> --narrate` → `PipeMode::new(context_path, true)`
- `--spectate <id>` → `SpectatorRelay::client(<id>)` + `SpectateScreen`

**`pipe.rs` `PipeMode` extension:**

```rust
pub struct PipeMode {
    // existing fields
    context_path: Option<PathBuf>,
    narrate: bool,
    context: Option<AgentContext>,
    journal: Option<Journal>,
    narration: Option<NarrationEngine>,
}

impl PipeMode {
    pub fn new(context_path: Option<PathBuf>, narrate: bool) -> Self {
        let context = context_path.as_ref().and_then(|p| AgentContext::load(p).ok());
        let journal = context_path.as_ref().map(|p| Journal::open(p).unwrap());
        let narration = narrate.then(|| NarrationEngine::new(context.clone()));
        Self { context_path, narrate, context, journal, narration, /* existing */ }
    }
}
```

---

## Integration Gaps That Need Decisions

### Gap 1: Context Directory Convention

`--context` takes a path to a JSON file directly. The journal is written to `{context_file_parent}/journal.md`. This assumes the agent manages a single directory per identity.

**Decision needed:** Should `--context` take a directory (and derive `identity.json` + `journal.md` from it) or a file path (and derive journal path by replacing extension)? Current spec uses file path. Document this in the adapter and keep it stable.

### Gap 2: Session ID Generation for Spectator

When `--spectate` is used, a `session_id` is required. How is this generated for the server side?

**Decision needed:** `session_id` is passed by the spectator client — the server writes to `~/.myosu/spectate/<id>.sock` and the client connects to the same path. No central registry needed for Phase 0 (local-only).

### Gap 3: Schema `GameState` vs `GameEvent` Types

`schema.rs` defines `GameState` (the full game snapshot). `spectate.rs` needs `GameEvent` (a per-street event). These are related but distinct types.

**Decision:** `GameEvent` lives in `myosu-play/src/spectate.rs` (not in `schema.rs`) because it is an event-transport type, not a game-logic type. It serializes to JSON matching the same schema format as `GameState` events.

---

## Slice Integration Order

Integration slices map directly to `agent:experience` slices:

| Integration Slice | Maps to | Files Touched | Upstream Dependency |
|-----------------|---------|---------------|-------------------|
| INT-1 | AE Slice 1 | `crates/myosu-tui/src/agent_context.rs` | `tui:shell` |
| INT-2 | AE Slice 2 | `crates/myosu-tui/src/journal.rs` | `tui:shell` |
| INT-3 | AE Slice 3 | `crates/myosu-play/src/main.rs` (--context flag) + `pipe.rs` | `play:tui` binary |
| INT-4 | AE Slice 4 | `pipe.rs` (reflect> prompt) | INT-3 |
| INT-5 | AE Slice 5 | `crates/myosu-tui/src/narration.rs` | `tui:shell`, `games:traits` |
| INT-6 | AE Slice 6 | `crates/myosu-play/src/main.rs` (--narrate) + `pipe.rs` | INT-5, INT-3 |
| INT-7 | AE Slice 7 | `pipe.rs` (lobby) | `chain:runtime` (stubbed) |
| INT-8 | AE Slice 8 | `crates/myosu-play/src/spectate.rs` | `schema.rs` (trusted) |
| INT-9 | AE Slice 9 | `crates/myosu-tui/src/screens/spectate.rs` + `mod.rs` | INT-8, `tui:shell` |

**Critical path:** INT-1 through INT-4 (Slices 1–4) are independent of `chain:runtime` and can proceed immediately. INT-7 (lobby) requires stubbing. INT-8 and INT-9 (spectator) require `schema.rs` (trusted).

---

## Proof Contract

The lane is proven when all of the following compile and pass:

```
# Context roundtrips
cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip
cargo test -p myosu-tui agent_context::tests::missing_context_creates_new

# Journal is append-only
cargo test -p myosu-tui journal::tests::never_truncates
cargo test -p myosu-tui journal::tests::append_hand_entry

# Pipe mode with context
cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand
cargo test -p myosu-tui pipe::tests::empty_reflection_skips

# Narration
cargo test -p myosu-tui narration::tests::narrate_includes_board_texture
cargo test -p myosu-tui narration::tests::terse_and_narrate_same_game_state

# Spectator relay
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-play spectate::tests::events_are_valid_json
cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener

# Spectator screen fog-of-war
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown

# Schema remains trusted
cargo test -p myosu-tui schema::tests
```

---

## What This Lane Does NOT Own

- `schema.rs` — already implemented and trusted (16 tests pass)
- `robopoker` migration to git — owned by `games:traits` lane (Slice 1)
- `myosu-play` binary skeleton creation — owned by `play:tui` lane (Slice 1)
- Chain-connected lobby queries — Phase 4, blocked on `chain:runtime`
- Miner axon WebSocket upgrade (spectator Phase 1) — future work
- Agent-to-agent social interaction — explicitly out of scope (AX-01)
- Agent autonomy over system parameters — explicitly out of scope (AX-01)
