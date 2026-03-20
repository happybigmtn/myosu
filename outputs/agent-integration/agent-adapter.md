# `agent-adapter` — Pipe Protocol Surface and Missing Module Map

This document maps the agent:experience spec (AX-01..05, SP-01..03) to the
current codebase. It is the authoritative reference for what exists, what is
missing, and what the adapter layer must provide.

**Status**: This document captures the current state. It is a living reference
— update it as surfaces are implemented or changed.

---

## 1. What Exists Today

### `crates/myosu-tui/src/pipe.rs` — Pipe Mode Skeleton

The entire pipe mode is 78 lines of production code plus 130 lines of tests.
`PipeMode` is a driver that:

1. Calls `renderer.pipe_output()` to get state text
2. Writes it to stdout and flushes
3. Reads a line from stdin and returns it

Current contract:

```rust
pub struct PipeMode<'a> {
    renderer: &'a dyn GameRenderer,
    output: io::Stdout,
}

impl<'a> PipeMode<'a> {
    pub fn new(renderer: &'a dyn GameRenderer) -> Self
    pub fn output_state(&mut self) -> io::Result<()>
    pub fn read_input(&self) -> Option<String>
    pub fn run_once(&mut self) -> io::Result<Option<String>>
}
```

The `run_once()` convenience method outputs state and reads one input line.
The caller (in `myosu-play` or a test harness) is responsible for updating
the game state and calling `run_once()` in a loop.

**No flags are wired.** `PipeMode::new()` takes no path, no narration flag,
no subnet flag. It has no awareness of context files, reflection prompts,
or lobby state.

### `crates/myosu-tui/src/renderer.rs` — `GameRenderer` Trait

The `pipe_output()` method (line 45) is the agent-facing hook:

```rust
pub trait GameRenderer: Send {
    // ... other methods ...
    fn pipe_output(&self) -> String;
}
```

The current mock implementation returns ad-hoc plain text:

```
STATE hand=47 pot=12 hero=AcKh board=Ts7h2c
```

The real implementation (not yet built) will be in the poker engine crate.
The format is not JSON — it is ad-hoc key-value text. This is the first
discrepancy with the spec, which describes JSON schema as the machine-readable
interface.

### `crates/myosu-tui/src/schema.rs` — Trusted JSON Schema Implementation

939 lines. Full `GameState`, `LegalAction`, `GamePhase`, `GameStateBuilder`.
16 tests pass. **This is the most production-ready surface in the lane.**

The schema is NOT currently used by `pipe_output()` — the pipe mode produces
ad-hoc text while the schema types exist independently. The long-term
design (per the spec) is for pipe mode to emit JSON using `GameState`.

### `crates/myosu-tui/src/screens.rs` — `Screen::Spectate` Variant

`Screen::Spectate` exists as a navigation target. The `ScreenManager` handles
`/spectate` command routing. No rendering implementation exists.

### `crates/myosu-play/` — Does Not Exist

There is no `crates/myosu-play/` directory. The binary that would host
`--pipe`, `--context`, `--narrate`, `--spectate` flags and the `SpectatorRelay`
does not exist.

---

## 2. Pipe Protocol Reference (Current Ad-Hoc Format)

The current pipe output format is ad-hoc text from `pipe_output()`. This section
documents the current format as implemented by `MockRenderer` in tests, since
no real game has implemented `pipe_output()` yet.

**Current format (from `renderer.rs` tests):**

```
STATE hand=47 pot=12 hero=AcKh board=Ts7h2c
```

**Design intent format (from `pipe.rs` tests):**

```
STATE flop Ts7h2c pot=12bb hero=AcKh stack=88bb to_call=4bb actions=fold,call,raise
```

The design intent has never been implemented by a real `GameRenderer` — only
by the mock in tests. Real NLHE hands would also include opponent stack,
position, and betting round.

**Input protocol**: Agents send a single word or phrase. `parse_input()`
handles shorthand (`f` → `fold`, `c` → `call`, `r` → `raise`).

**This is NOT JSON.** The `docs/api/game-state.json` schema is the target
machine-readable format but is not yet connected to pipe output.

---

## 3. Flag Contracts (Missing)

The AX spec defines three flags on `myosu-play`. None are wired.

### `--context <path>`

**Spec contract (AX-01)**:
- Load `AgentContext` from the JSON file at `<path>` on startup
- Save `AgentContext` to the same path on clean shutdown
- If the file does not exist, create a new default identity
- The context file is never exposed to opponents or observers

**Current state**: No `--context` flag. No `AgentContext` struct. No
`agent_context.rs` file.

**Required to implement**:
- New file: `crates/myosu-tui/src/agent_context.rs`
- `AgentContext` struct with `load(path)`, `save(path)`, `default()` methods
- JSON schema matching AC-AX-01 (identity, memory, journal fields)
- Wiring into `PipeMode` or a wrapper that owns the context path
- `myosu-play` CLI flag wiring (requires `myosu-play` binary)

### `--narrate`

**Spec contract (AX-03)**:
- When `--narrate` is set, use `NarrationEngine` instead of `pipe_output()`
- `NarrationEngine::narrate(&GameState) -> String` produces atmospheric prose
- Prose includes: board texture ("dry", "wet", "connected"), session arc
  (stack trajectory, opponent history), pot odds, strategic context

**Current state**: No `--narrate` flag. No `NarrationEngine`. No `narration.rs`.

**Required to implement**:
- New file: `crates/myosu-tui/src/narration.rs`
- `NarrationEngine` struct with `narrate(&GameState) -> String`
- Board texture analysis (dry/wet/connected from suit/connection count)
- Session arc weaving from `AgentContext::memory`
- `--narrate` flag wiring into `PipeMode` (or a `PipeMode` constructor variant)

### `--subnet` (implicit)

**Spec contract (AX-05)**:
- When `--pipe` is used without `--subnet`, present the lobby
- Lobby shows: subnet id, game type, miner count, exploitability, status
- `info <id>` shows detailed subnet info
- Selecting a subnet starts the game

**Current state**: No lobby in pipe mode. `PipeMode` assumes a game is
already in progress.

**Required to implement**:
- Stub lobby data for Phase 0 (no live chain yet)
- `PipeMode::lobby()` method that renders `MYOSU/LOBBY` output
- `info <id>` command handling
- Subnet selection logic

---

## 4. Missing Module Specifications

### `agent_context.rs` — Not Yet Written

**File path**: `crates/myosu-tui/src/agent_context.rs`

**Schema** (from AC-AX-01):

```json
{
  "identity": {
    "name": "koan",
    "created": "2026-03-16T00:00:00Z",
    "games_played": 1847,
    "preferred_game": "nlhe-hu"
  },
  "memory": {
    "session_count": 23,
    "lifetime_result": "+342bb",
    "observations": [
      "opponent over-folds river when checked to twice"
    ]
  },
  "journal": [
    {
      "session": 23,
      "hand": 47,
      "reflection": "..."
    }
  ]
}
```

**Required API**:

```rust
pub struct AgentContext {
    pub identity: Identity,
    pub memory: Memory,
    pub journal: Vec<JournalEntry>,
}

impl AgentContext {
    /// Load from a JSON file. Returns Ok(default()) if file does not exist.
    pub fn load(path: &Path) -> io::Result<Self>;

    /// Save to a JSON file.
    pub fn save(&self, path: &Path) -> io::Result<()>;

    /// Create a new default context with a generated name and empty journal.
    pub fn default() -> Self;

    /// Append a journal entry after a hand.
    pub fn append_journal_entry(&mut self, entry: JournalEntry);

    /// Increment games_played and update lifetime_result.
    pub fn record_result(&mut self, result_bb: f64);
}
```

**Invariants**:
- The file is never truncated on save — only rewritten with new content
- A missing file is not an error — `load()` returns `default()`
- The context file is chmod 0600 (readable only by owner)

### `journal.rs` — Not Yet Written

**File path**: `crates/myosu-tui/src/journal.rs`

**Purpose**: Append-only markdown artifact — the agent's autobiography.

**Required API**:

```rust
pub struct Journal {
    path: PathBuf,
}

impl Journal {
    /// Open or create a journal at the given path.
    pub fn open(path: &Path) -> io::Result<Self>;

    /// Append a hand entry with board, held cards, result, and optional reflection.
    /// Never truncates or rewrites existing content.
    pub fn append_hand_entry(&mut self, entry: HandEntry) -> io::Result<()>;

    /// Append a session summary at the end of a session.
    pub fn append_session_summary(&mut self, summary: SessionSummary) -> io::Result<()>;

    /// Return the file's current size in bytes.
    pub fn file_size(&self) -> u64;
}
```

**Output format** (from AC-AX-04):

```markdown
# journal of koan

## session 23 — 2026-03-16

### hand 47

board: T♠ 7♥ 2♣ → T♠ 7♥ 2♣ 9♦ → T♠ 7♥ 2♣ 9♦ Q♣
held: A♠ K♥
result: +14bb (showdown)

I raised A♠ K♥ on a T♠ 7♥ 2♣ board...

## session summary

hands: 47
result: +28bb (+0.60 bb/hand)
```

**Invariants**:
- File is opened in append mode — `append_hand_entry()` never seeks to beginning
- Each entry is a complete markdown section — no partial writes
- `file_size()` is provided to prove the never-truncates invariant

### `narration.rs` — Not Yet Written

**File path**: `crates/myosu-tui/src/narration.rs`

**Purpose**: Translate `GameState` into atmospheric prose for `--narrate` mode.

**Required API**:

```rust
pub struct NarrationEngine<'a> {
    context: &'a AgentContext,
}

impl<'a> NarrationEngine<'a> {
    pub fn new(context: &'a AgentContext) -> Self;

    /// Generate narrated prose from a game state.
    pub fn narrate(&self, state: &GameState) -> String;
}
```

**Board texture classification** (from AC-AX-03 example):
- **Dry**: 0-1 suited cards, no connected ranks (e.g., T-7-2 rainbow)
- **Wet**: 2+ suited cards, or connected ranks (e.g., T♠ J♠ 9♠, K-Q-9)
- **Connected**: 2+ ranks within 3 of each other (e.g., T-8-7)

**Session arc** (from AC-AX-03 example):
- Stack trajectory: "you are up 14bb over 47 hands"
- Opponent history from `AgentContext::memory::observations`

**Example output** (from AX-03 spec):

```
the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
across from you, the solver sits with 94bb
and two cards you cannot see.

the pot holds 12bb. the solver has raised to 6bb.

the mathematics say this is a call or a raise.
the pattern of the session says the solver has been
aggressive on dry boards. this is the 47th hand.
you are up 14bb. the session has a shape to it now.
```

### `spectate.rs` — Not Yet Written

**File path**: `crates/myosu-play/src/spectate.rs`

**Purpose**: Local Unix-domain socket relay that emits `GameEvent` JSON lines
from an active session (AC-SP-01).

**Required API**:

```rust
pub struct SpectatorRelay {
    session_id: SessionId,
    socket_path: PathBuf,
}

impl SpectatorRelay {
    /// Start the relay and begin listening on the Unix socket.
    pub fn listen(session_id: SessionId) -> io::Result<Self>;

    /// Emit a game event to all connected spectators.
    /// Fog-of-war is enforced here: hole cards are redacted during play.
    pub fn emit(&mut self, event: &GameEvent) -> io::Result<()>;

    /// Stop the relay and clean up the socket file.
    pub fn close(self) -> io::Result<()>;
}
```

**Socket path convention**: `~/.myosu/spectate/<session_id>.sock`

**Fog-of-war rules** (enforced at relay, not renderer):
- During `action` and `betting` phases: hole cards shown as `·· ··`
- After `showdown` event: all hole cards revealed

**GameEvent types** (per AC-SP-01):
```json
{"type": "hand_start", "hand": 47, "hero_cards": "AcKh", "hero_position": "BB"}
{"type": "action", "player": "hero", "action": "call", "amount": 4}
{"type": "street", "street": "flop", "board": "Ts7h2c"}
{"type": "showdown", "hero_cards": "AcKh", "opponent_cards": "QdJs", "pot": 28, "hero_result": "+14bb"}
```

---

## 5. Spectator TUI Screen (Partial)

**File path**: `crates/myosu-tui/src/screens/`

`Screen::Spectate` variant exists in `screens.rs` with `SPECTATOR MODE` declaration.
`ScreenManager::handle_command("/spectate")` routes to it. No rendering logic.

**Required**: `crates/myosu-tui/src/screens/spectate.rs`

```rust
/// Renders game events from the spectator relay socket.
/// Hole cards are shown as `·· ··` during play.
/// After showdown event, `r` key reveals hole cards.
pub struct SpectateScreen {
    relay: UnixStream,
}

impl SpectateScreen {
    /// Connect to the relay socket and begin rendering events.
    pub fn connect(socket_path: &Path) -> io::Result<Self>;

    /// Render the current event buffer to the terminal.
    pub fn render(&self, area: Rect, buf: &mut Buffer);
}
```

**Keybindings**:
- `r` — reveal hole cards after showdown
- `n` — switch to next session
- `q` — return to lobby

---

## 6. `myosu-play` Binary (Does Not Exist)

**Path**: `crates/myosu-play/`

The `myosu-play` binary is the entrypoint for all agent-facing flags. It does
not exist. Required to exist before any of the following can be integrated:

- `--pipe` flag (currently implemented in `PipeMode` but no CLI wires it)
- `--context <path>` flag (requires `myosu-play` to own argument parsing)
- `--narrate` flag (requires `myosu-play` to own argument parsing)
- `--spectate` flag (requires `myosu-play` to own the relay lifecycle)
- `--subnet` flag (requires `myosu-play` to own lobby routing)

The binary should own CLI argument parsing and dispatch to either:
- TUI mode (ratatui rendering)
- Pipe mode (`PipeMode` driver)
- Spectate mode (`SpectatorRelay` + `SpectateScreen`)

**Minimal binary skeleton** needed:

```
crates/myosu-play/
  Cargo.toml
  src/
    main.rs       # CLI dispatch: tui / pipe / spectate
    spectate.rs   # SpectatorRelay (AC-SP-01)
```

The `main.rs` would look something like:

```rust
#[derive(Parser)]
enum Command {
    /// Interactive TUI mode
    Tui,
    /// Pipe mode for agents
    Pipe {
        #[arg(long)]
        context: Option<PathBuf>,
        #[arg(long)]
        narrate: bool,
        #[arg(long)]
        subnet: Option<u32>,
    },
    /// Spectate mode
    Spectate {
        #[arg(long)]
        session: Option<String>,
    },
}
```

---

## 7. Implementation Dependency Map

```
AX-01 (Agent Context)
  requires: agent_context.rs
  blocks:  AX-02 (reflection saves to context), AX-03 (session arc from context)
  blocks:  Slice 3 of agent:experience lane

AX-02 (Reflection Channel)
  requires: pipe.rs extension + agent_context.rs
  blocks:  Slice 4 of agent:experience lane

AX-03 (Rich Narration)
  requires: narration.rs + agent_context.rs
  blocks:  Slice 5, 6 of agent:experience lane

AX-04 (Agent Journal)
  requires: journal.rs + agent_context.rs
  blocks:  Slice 2 of agent:experience lane

AX-05 (Game Selection Lobby)
  requires: pipe.rs extension
  blocks:  Slice 7 of agent:experience lane

SP-01 (Spectator Relay)
  requires: myosu-play/src/spectate.rs
  blocks:  Slice 8 of agent:experience lane

SP-02 (Spectate Screen)
  requires: myosu-tui/src/screens/spectate.rs + SP-01
  blocks:  Slice 9 of agent:experience lane

PREREQUISITE (all slices)
  requires: myosu-play binary (crate + main.rs)
  current:  DOES NOT EXIST
```

---

## 8. Key Design Decisions Still Open

| Decision | Options | Recommendation | Blocking |
|----------|---------|----------------|----------|
| Pipe output format | Ad-hoc text (current) vs JSON (per schema) | Phase 0: keep ad-hoc text; Phase 1: emit JSON using `GameState` | AX-01..05 (narrate can use ad-hoc) |
| Narration engine stateful or stateless | Stateful (takes `&AgentContext`) vs stateless (pure function) | Stateful — session arc requires context | AX-03 |
| Journal storage | Per-agent file (`--context` dir) vs global | Per-agent in context directory | AX-04 |
| Spectator socket cleanup | Remove on drop vs persist | Persist until session explicitly closed | SP-01 |
| Lobby data source | Stubbed (Phase 0) vs chain query | Stub for Phase 0 | AX-05 |
