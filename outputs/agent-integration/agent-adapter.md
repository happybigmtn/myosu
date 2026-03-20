# `agent:experience` Adapter Specification

## Purpose

This document specifies the **adapter layer** that implements the `agent:experience` lane specification. The adapter lives in `crates/myosu-tui/src/` as three new modules (`agent_context.rs`, `journal.rs`, `narration.rs`) plus extensions to `pipe.rs`, and a new `spectate.rs` in `crates/myosu-play/src/`.

The adapter is not a new abstraction — it is the composition of existing trusted surfaces (`GameRenderer`, `PipeMode`, `schema.rs`) with new modules that implement the agent-facing capabilities: persistent identity, append-only journaling, and atmospheric narration.

---

## Composition Architecture

```
agent:experience adapter
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ pipe.rs (extended)                                   │   │
│  │  - context_path: Option<PathBuf>                     │   │
│  │  - narrate: bool                                     │   │
│  │  - reflects: Vec<ReflectEntry>                      │   │
│  │  - lobby() → render lobby when no subnet            │   │
│  │  - reflect_prompt() → after each hand              │   │
│  └────────────────────────────┬─────────────────────────┘   │
│                               │                              │
│  ┌────────────────────────────┴─────────────────────────┐   │
│  │ AgentContext (agent_context.rs)                       │   │
│  │  - identity: Identity (name, created, games_played)  │   │
│  │  - memory: Memory (session_count, lifetime_result)   │   │
│  │  - journal: Journal (reflections appended)           │   │
│  │  - load(path) → Result<AgentContext>                │   │
│  │  - save(path) → Result<()>                          │   │
│  └────────────────────────────┬─────────────────────────┘   │
│                               │                              │
│  ┌────────────────────────────┴─────────────────────────┐   │
│  │ Journal (journal.rs)                                 │   │
│  │  - append_hand(state, result, reflection)            │   │
│  │  - append_session_summary(stats)                     │   │
│  │  - never truncates (append-only invariant)           │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ NarrationEngine (narration.rs)                       │   │
│  │  - narrate(state) → String (atmospheric prose)       │   │
│  │  - board_texture(board) → "dry"|"wet"|"connected"   │   │
│  │  - session_arc(ctx) → stack trajectory, history     │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ SpectatorRelay (spectate.rs, myosu-play crate)       │   │
│  │  - emit(event) → writes JSON to Unix socket          │   │
│  │  - fog_of_war: hole cards hidden during play         │   │
│  │  - socket: ~/.myosu/spectate/<session_id>.sock      │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ schema.rs (existing, trusted)                        │   │
│  │  - GameState, LegalAction, GamePhase types           │   │
│  │  - 16 tests pass; 10 game types covered              │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘

Trusted upstream surfaces (read-only, not modified):
  - GameRenderer trait (renderer.rs)
  - PipeMode (pipe.rs)
  - schema.rs (fully implemented)

CLI entry point:
  myosu-play --pipe [--context <path>] [--narrate] [--subnet <id>]
```

---

## Module Specifications

### `agent_context.rs`

**Location**: `crates/myosu-tui/src/agent_context.rs`

**Purpose**: Persistent agent identity and memory across sessions.

```rust
// Schema from AC-AX-01
pub struct Identity {
    pub name: String,
    pub created: DateTime<Utc>,
    pub games_played: u64,
    pub preferred_game: GameType,
}

pub struct Memory {
    pub session_count: u64,
    pub lifetime_result: String,  // e.g., "+342bb"
    pub observations: Vec<Observation>,
}

pub struct AgentContext {
    pub identity: Identity,
    pub memory: Memory,
    pub journal: Vec<JournalEntry>,
}

impl AgentContext {
    /// Load from JSON file. Missing file creates a default identity.
    pub fn load(path: &Path) -> io::Result<Self>;

    /// Save to JSON file.
    pub fn save(&self, path: &Path) -> io::Result<()>;

    /// Create a default identity with a generated name and creation timestamp.
    pub fn default() -> Self;
}
```

**Required evidence**:
- `cargo test -p myosu-tui agent_context::tests::load_and_save_roundtrip`
- `cargo test -p myosu-tui agent_context::tests::journal_appends_not_overwrites`
- `cargo test -p myosu-tui agent_context::tests::missing_context_creates_new`

**State**: Completely absent — no file at this path.

---

### `journal.rs`

**Location**: `crates/myosu-tui/src/journal.rs`

**Purpose**: Append-only markdown journal. Never truncates. Each hand produces a markdown entry with board, held cards, result, and optional reflection.

```rust
pub struct Journal {
    path: PathBuf,
    file: fs::OpenOptions,
}

impl Journal {
    /// Append a hand entry to the journal. Never rewrites existing content.
    pub fn append_hand(
        &mut self,
        state: &GameState,
        result: &str,
        reflection: Option<&str>,
    ) -> io::Result<()>;

    /// Append a session summary.
    pub fn append_session_summary(&mut self, stats: &SessionStats) -> io::Result<()>;

    /// Invariant: file size only grows; never shrinks or truncates.
    fn assert_append_only(&self) -> bool;
}
```

**Required evidence**:
- `cargo test -p myosu-tui journal::tests::append_hand_entry`
- `cargo test -p myosu-tui journal::tests::never_truncates`

**State**: Completely absent — no file at this path.

---

### `narration.rs`

**Location**: `crates/myosu-tui/src/narration.rs`

**Purpose**: Translate `GameState` into atmospheric prose. Distinct from `pipe_output()` — pipe mode is terse key-value for fast parsing; narration is a story for experienced consumption.

```rust
pub struct NarrationEngine<'a> {
    context: &'a AgentContext,
}

impl<'a> NarrationEngine<'a> {
    pub fn new(context: &'a AgentContext) -> Self;

    /// Main entry point. Returns atmospheric prose for the given game state.
    pub fn narrate(&self, state: &GameState) -> String;

    /// Board texture analysis: "dry", "wet", "connected", "monotone"
    fn board_texture(&self, board: &[Card]) -> &'static str;

    /// Session arc: stack trajectory, opponent history from context.
    fn session_arc(&self) -> String;
}
```

Example narrated output (from AC-AX-03):
```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
```

**Required evidence**:
- `cargo test -p myosu-tui narration::tests::narrate_includes_board_texture`
- `cargo test -p myosu-tui narration::tests::narrate_includes_session_context`
- `cargo test -p myosu-tui narration::tests::terse_and_narrate_same_game_state`

**State**: Completely absent — no file at this path.

---

### `pipe.rs` Extensions

**Location**: `crates/myosu-tui/src/pipe.rs`

**Changes required**:

1. **Add fields to `PipeMode`**:
```rust
pub struct PipeMode<'a> {
    renderer: &'a dyn GameRenderer,
    output: io::Stdout,
    context_path: Option<PathBuf>,   // NEW
    narrate: bool,                   // NEW
    agent_ctx: Option<AgentContext>, // NEW
    journal: Option<Journal>,         // NEW
    narration: Option<NarrationEngine<'a>>, // NEW
}
```

2. **Add `reflect>` prompt after hand** (AC-AX-02):
```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```
- Empty line skips reflection.
- Non-empty line is appended to the journal entry.

3. **Add lobby rendering when no `--subnet`** (AC-AX-05):
```
MYOSU/LOBBY
subnets:
  1 nlhe-hu    12 miners  13.2 mbb/h  ACTIVE
  2 nlhe-6max  18 miners  15.8 mbb/h  ACTIVE
>
```

4. **Wire `--context` and `--narrate` flags**: Pass through CLI to `PipeMode`.

**Required evidence**:
- `cargo test -p myosu-tui pipe::tests::reflection_prompt_after_hand`
- `cargo test -p myosu-tui pipe::tests::empty_reflection_skips`
- `cargo test -p myosu-tui pipe::tests::lobby_presented_without_subnet_flag`

**State**: `PipeMode` exists with 6 tests. `--context`, `--narrate`, `reflect>`, and lobby are absent.

---

### `spectate.rs`

**Location**: `crates/myosu-play/src/spectate.rs`

**Purpose**: Phase 0 Unix-domain socket relay. Emits JSON event lines from an active session. Fog-of-war enforced at relay (hole cards never sent during play).

```rust
pub struct SpectatorRelay {
    session_id: SessionId,
    socket_path: PathBuf,
    listeners: Vec<TcpStream>,
}

impl SpectatorRelay {
    /// Emit a game event to all connected listeners.
    pub fn emit(&mut self, event: &GameEvent) -> io::Result<()>;

    /// Enforce fog-of-war: hole cards hidden during active play.
    fn redact_hole_cards(event: &mut GameEvent);
}

/// GameEvent matches the JSON schema in docs/api/game-state.json
pub struct GameEvent {
    pub event_type: EventType,
    pub session_id: SessionId,
    pub street: Option<Street>,
    pub hole_cards: Option<[Card; 2]>,  // None during play; Some after showdown
    pub board: Vec<Card>,
    pub pot: u64,
    pub action: Option<Action>,
}
```

Socket path: `~/.myosu/spectate/<session_id>.sock`

**Required evidence**:
- `cargo test -p myosu-play spectate::tests::relay_emits_events`
- `cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener`
- `cargo test -p myosu-play spectate::tests::events_are_valid_json`

**State**: Completely absent — no `crates/myosu-play/` crate exists.

---

## What Already Exists

These surfaces are **already implemented and trusted** — do not rewrite:

| Surface | Location | Evidence |
|---------|---------|---------|
| `GameRenderer` trait | `crates/myosu-tui/src/renderer.rs` | 8 tests, object-safe |
| `PipeMode` | `crates/myosu-tui/src/pipe.rs` | 6 tests |
| `schema.rs` + `GameState` | `crates/myosu-tui/src/schema.rs` | 16 tests, 10 game types |
| `docs/api/game-state.json` | `docs/api/game-state.json` | JSON schema, all 20 game types |
| `GameRenderer::pipe_output()` | `renderer.rs:45` | Returns `String` for pipe protocol |

---

## Slice Dependency Chain

```
Slice 1: agent_context.rs
  → Creates AgentContext, load/save, default identity

Slice 2: journal.rs
  → Depends on: Slice 1 (AgentContext needed for session tracking)

Slice 3: pipe.rs extensions (--context wiring)
  → Depends on: Slice 1 + Slice 2

Slice 4: reflect> prompt in pipe.rs
  → Depends on: Slice 3 (context loaded in PipeMode::new)

Slice 5: narration.rs
  → Depends on: Slice 1 (needs AgentContext for session arc)

Slice 6: pipe.rs extensions (--narrate wiring)
  → Depends on: Slice 5 (needs NarrationEngine)

Slice 7: lobby rendering in pipe.rs
  → Depends on: Slice 3 (context path already wired)
  → Stub chain discovery for Phase 0 (hardcoded lobby data)

Slice 8: spectate.rs (SpectatorRelay)
  → Depends on: myosu-play crate existing (play:tui Slice 1)

Slice 9: SpectateScreen
  → Depends on: Slice 8 (relay must exist)
```

---

## Phase 0 Slices That Need No Upstream

Slices 1 (agent_context.rs) and 2 (journal.rs) depend only on `myosu-tui` types that already exist (`GameType`, `serde`, `std::path::PathBuf`). These can begin immediately:

- **No `myosu-play` binary required** — these are library modules in `myosu-tui`
- **No `robopoker` git migration required** — no game logic, just JSON serialization
- **No chain connection required** — pure local file I/O

Slice 3 (`--context` wiring) requires the `myosu-play` binary skeleton (play:tui Slice 1) to wire the CLI flag.

---

## robopoker Dependency Status

The `agent:experience` adapter modules (`agent_context.rs`, `journal.rs`, `narration.rs`) have **no robopoker dependency**. They work with:

- `AgentContext` → pure JSON (serde)
- `Journal` → pure markdown file I/O
- `NarrationEngine` → `GameState` from `schema.rs` (no robopoker types needed for Phase 0 prose generation)

Only when Slice 5 (narration) needs to analyze actual hand histories from `robopoker` would the robopoker git migration become a dependency. For Phase 0, `GameState` from `schema.rs` is sufficient input.

---

## CLI Contract

The `myosu-play` binary exposes agent mode through:

```
myosu-play --pipe [--context <path>] [--narrate] [--subnet <id>]
```

| Flag | Purpose | Slice |
|------|---------|-------|
| `--pipe` | Enable agent stdin/stdout protocol | existing |
| `--context <path>` | Load/save agent identity and journal | Slice 3 |
| `--narrate` | Use narration engine instead of pipe_output() | Slice 6 |
| `--subnet <id>` | Connect to specific subnet (omit for lobby) | Slice 7 |

After each hand in pipe mode:
```
HAND COMPLETE
result: <result>
session: <session_result>

reflect>
```

Agent responds with reflection (or empty line to skip). Journal is appended.
