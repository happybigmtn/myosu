# `agent:integration` — Agent Integration Adapter

## Purpose

This document specifies the **integration adapter** that wires the agent-facing surfaces
from `agent:experience` into a coherent interface for programmatic agents. It is the
binding layer between the pipe protocol, JSON schema, agent context, journal, narration
engine, and spectator relay.

This is the first honest slice: an integration layer that exists as documentation
and as the structural blueprint for the actual Rust code that must be written.

## Agent Interface Contract

An agent interacts with Myosu through a single entrypoint:

```
myosu-play --pipe [--context <path>] [--narrate] [--subnet <id>]
```

### The Five Surfaces

**Surface 1 — Pipe Protocol (stdin/stdout)**
The primary interface. Plain text, no ANSI codes, no cursor manipulation.
All output is flushed immediately. All input is line-buffered.

**Surface 2 — JSON Schema (docs/api/game-state.json + schema.rs)**
Machine-readable game state. Used by structured agents that parse structured output
rather than text. The `GameState` type in `schema.rs` is the authoritative schema.

**Surface 3 — Agent Context File (--context <path>)**
A JSON file providing persistent identity, memory, and journal across sessions.
Loaded at startup, saved at shutdown.

**Surface 4 — Journal Artifact (append-only markdown)**
The agent's autobiography. Written after each hand. Never truncated. The agent's
own account of its experience.

**Surface 5 — Spectator Relay (Unix socket)**
A local event stream for observers. Fog-of-war enforced at the relay (hole cards
never sent during play).

---

## Integration Architecture

```
                          myosu-play --pipe [--context <path>] [--narrate] [--subnet <id>]
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        AgentIntegration                                  │
│                                                                         │
│  ┌─────────────┐   ┌──────────────┐   ┌──────────────────────────┐   │
│  │ PipeMode    │◄──│ AgentContext  │◄──│ Journal                  │   │
│  │ (stdin/out) │   │ (load/save)   │   │ (append-only markdown)  │   │
│  └──────┬──────┘   └──────────────┘   └──────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                   GameRenderer (trait)                         │   │
│  │                                                               │   │
│  │  pipe_output() ──► terse text (pipe mode)                    │   │
│  │  render_state() ──► ratatui buffer (TUI mode)                │   │
│  │                                                               │   │
│  │  When --narrate: NarrationEngine instead of pipe_output()     │   │
│  └──────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  SchemaBridge                                                 │   │
│  │  Converts internal game state → GameState JSON                │   │
│  │  (Used by spectator relay; available to pipe mode agents)     │   │
│  └──────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  SpectatorRelay                                               │   │
│  │  Unix socket: ~/.myosu/spectate/<session_id>.sock             │   │
│  │  Emits GameEvent JSON lines                                   │   │
│  │  Fog-of-war enforced at relay (not renderer)                  │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Current Implementation Status

| Surface | File | Status |
|---------|------|--------|
| Pipe protocol | `crates/myosu-tui/src/pipe.rs` | **EXISTS** — basic stdin/stdout, 6 tests pass |
| JSON schema | `crates/myosu-tui/src/schema.rs` | **EXISTS** — 939 lines, 16 tests pass |
| `--context` flag | Not wired | **MISSING** |
| `--narrate` flag | Not wired | **MISSING** |
| AgentContext | `crates/myosu-tui/src/agent_context.rs` | **MISSING** — no file |
| Journal | `crates/myosu-tui/src/journal.rs` | **MISSING** — no file |
| NarrationEngine | `crates/myosu-tui/src/narration.rs` | **MISSING** — no file |
| `reflect>` prompt | Not in pipe.rs | **MISSING** |
| Lobby (pipe mode) | Not in pipe.rs | **MISSING** |
| SpectatorRelay | `crates/myosu-play/src/spectate.rs` | **MISSING** — no file |
| SpectateScreen | `crates/myosu-tui/src/screens/spectate.rs` | **MISSING** — no file |

---

## Pipe Protocol Reference

### Current (exists)

```
# Start pipe mode
myosu-play --pipe

# Current output format (from GameRenderer::pipe_output())
STATE hand=N pot=Nbb hero=AhKs board=Ts7h2c to_call=Nbb actions=fold,call,raise

# Current input format
fold | f
call | c
raise | r
```

### Planned Extensions

```
# With persistent context
myosu-play --pipe --context ./koan.json

# With rich narration
myosu-play --pipe --narrate

# Both together
myosu-play --pipe --context ./koan.json --narrate

# With explicit subnet
myosu-play --pipe --subnet 1
```

---

## AgentContext Integration

### File Location
`crates/myosu-tui/src/agent_context.rs`

### Schema (from AX-AX-01)

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

### Integration Contract

```
PipeMode::new(renderer, context_path: Option<PathBuf>)
  → loads AgentContext from path, or creates default
  → passes context to NarrationEngine for session arc weaving
  → passes context to Journal for entry creation

PipeMode::shutdown()
  → saves AgentContext to path
  → appends to Journal
```

### Required Traits

```rust
pub struct AgentContext {
    pub identity: Identity,
    pub memory: Memory,
    pub journal: Vec<JournalEntry>,
}

impl AgentContext {
    pub fn load(path: &Path) -> Result<Self>;
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn default() -> Self;
    pub fn append_journal_entry(&mut self, entry: JournalEntry);
}
```

---

## Journal Integration

### File Location
`crates/myosu-tui/src/journal.rs`

### Integration Contract

```
Journal::append_hand_entry(path: &Path, entry: HandEntry) -> Result<()>
  → opens file in append mode
  → writes markdown-formatted entry
  → flushes and closes
  → NEVER truncates

Journal::append_session_summary(path: &Path, summary: SessionSummary) -> Result<()>
  → appends summary block after session end
```

### Markdown Format

```markdown
# journal of {identity.name}

## session {N} — {date}

### hand {M}

board: {board}
held: {hero_cards}
result: {result}

{reflection_text}
```

---

## NarrationEngine Integration

### File Location
`crates/myosu-tui/src/narration.rs`

### Integration Contract

```rust
pub struct NarrationEngine<'a> {
    context: &'a AgentContext,
}

impl<'a> NarrationEngine<'a> {
    pub fn narrative(&self, state: &GameState) -> String;
    fn board_texture(&self, board: &[Card]) -> Texture;  // dry | wet | connected
    fn session_arc(&self) -> String;  // from context.memory
    fn opponent_summary(&self) -> String;  // from context.memory.observations
}
```

### Board Texture Analysis

The narration engine analyzes board texture using card properties:

| Texture | Suits | Connections | Narrative Keyword |
|---------|-------|-------------|------------------|
| Dry | 3+ | none/1-gap | "dry", "static", "simple" |
| Wet | 2 | pairs/connected | "wet", "dynamic", "dangerous" |
| Connected | 3+ | runs/paired | "connected", "complex", "drawing" |

### Example Narrated Output

```
── hand 47 ──────────────────────────────────────

the board reads T♠ 7♥ 2♣. three suits, no connections.
a dry texture. the kind of board that rewards the player
who arrived with the stronger range.

you hold A♠ K♥ in the big blind. 94bb behind.
across from you, the solver — the distilled equilibrium
of ten thousand hours of self-play — sits with 94bb
and two cards you cannot see.

the pot holds 12bb. the solver has raised to 6bb.

the mathematics say this is a call or a raise.
the pattern of the session says the solver has been
aggressive on dry boards. this is the 47th hand.
you are up 14bb. the session has a shape to it now.

what do you do?

>
```

---

## SpectatorRelay Integration

### File Location
`crates/myosu-play/src/spectate.rs`

### Socket Path Convention
`~/.myosu/spectate/<session_id>.sock`

Note: This convention should be verified against `play:tui`'s data directory
convention (`{data-dir}/hands/hand_{N}.json`). If different, align before
Slice 8 implementation.

### Integration Contract

```rust
pub struct SpectatorRelay {
    session_id: Uuid,
    socket_path: PathBuf,
    fog_of_war: FogOfWar,
}

impl SpectatorRelay {
    pub fn new(session_id: Uuid) -> Self;
    pub fn emit(&mut self, event: &GameEvent) -> Result<()>;
}

pub enum GameEvent {
    HandStart { hand_id: u64, hole_cards: [Card; 2] },  // only for spectator's own cards
    BoardUpdate { street: Street, cards: Vec<Card> },
    Action { player: &str, action: &str, amount: u64 },
    Showdown { board: Vec<Card>, winners: Vec<&str> },
    HandEnd { hand_id: u64, result: &str },
}

pub struct FogOfWar;
impl FogOfWar {
    pub fn filter(&self, event: &GameEvent, spectator_cards: Option<[Card; 2]>) -> GameEvent;
}
```

### Fog-of-War Rules

- Hole cards of other players: NEVER sent to relay during play
- Spectator's own hole cards: sent at `HandStart` and remain visible
- After `Showdown` event: ALL hole cards revealed
- `r` key in SpectateScreen: manual reveal after showdown (fog lifted)

---

## Reflect Prompt Integration

### Location
`crates/myosu-tui/src/pipe.rs` — extend `PipeMode`

### Flow

```
1. Hand completes
2. PipeMode outputs HAND COMPLETE block
3. PipeMode outputs "reflect> " prompt to stdout
4. PipeMode reads line from stdin
5. If line is empty → skip, continue
6. If line is non-empty → append to JournalEntry.reflection
7. Continue to next hand
```

### Output Format

```
HAND COMPLETE
result: +14bb (showdown, A♠ K♥ vs Q♣ J♣)
session: +28bb over 47 hands

reflect>
```

---

## Lobby Integration (Pipe Mode Without --subnet)

### Location
`crates/myosu-tui/src/pipe.rs` — extend `PipeMode`

### Flow

```
1. myosu-play --pipe (no --subnet)
2. PipeMode outputs MYOSU/LOBBY
3. Agent types "info <id>" or "<id>"
4. If "info <id>" → output subnet detail
5. If "<id>" → start game on subnet
```

### Lobby Format

```
MYOSU/LOBBY
subnets:
  1 nlhe-hu    12 miners  13.2 mbb/h  ACTIVE
  2 nlhe-6max  18 miners  15.8 mbb/h  ACTIVE

>
```

### Info Format

```
SUBNET {id} — {name}
best_exploitability: {N} mbb/h
your_history: {N} sessions, {result} lifetime
miners: {N} active
```

Note: Chain discovery is stubbed for Phase 0. Hardcoded lobby data until
`chain:runtime` lane provides miner axon endpoint.

---

## Integration Slices

### Slice 1: AgentContext + Journal (Foundation)
**Files**: `crates/myosu-tui/src/agent_context.rs`, `crates/myosu-tui/src/journal.rs`
**What**: Basic structs, load/save, append-only journal. No narration, no reflection.
**Proof**: `cargo test -p myosu-tui agent_context::tests` + `cargo test -p myosu-tui journal::tests`

### Slice 2: --context Flag Wiring
**Files**: `crates/myosu-tui/src/pipe.rs`
**What**: Wire `AgentContext` into `PipeMode::new()` and `shutdown()`
**Proof**: `cargo test -p myosu-tui pipe::tests::context_preserved_across_sessions`

### Slice 3: NarrationEngine + --narrate Flag
**Files**: `crates/myosu-tui/src/narration.rs`, `crates/myosu-tui/src/pipe.rs`
**What**: Board texture analysis, session arc, prose generation. `--narrate` flag wired.
**Proof**: `cargo test -p myosu-tui narration::tests`

### Slice 4: Reflect Prompt + Journal Integration
**Files**: `crates/myosu-tui/src/pipe.rs`
**What**: `reflect>` after each hand, append to journal entry
**Proof**: `cargo test -p myosu-tui pipe::tests::reflection_prompt_*`

### Slice 5: Lobby in Pipe Mode (Stubbed)
**Files**: `crates/myosu-tui/src/pipe.rs`
**What**: Lobby output, `info <id>` command, subnet selection. Chain data stubbed.
**Proof**: `cargo test -p myosu-tui pipe::tests::lobby_*`

### Slice 6: SpectatorRelay (Phase 0)
**Files**: `crates/myosu-play/src/spectate.rs`
**What**: Unix socket relay, GameEvent emission, fog-of-war enforcement
**Proof**: `cargo test -p myosu-play spectate::tests::relay_*`

### Slice 7: SpectateScreen
**Files**: `crates/myosu-tui/src/screens/spectate.rs`
**What**: Screen variant, fog-of-war rendering, `r` key for reveal
**Proof**: `cargo test -p myosu-tui spectate::tests::renders_*`

---

## Dependencies

| Surface | Upstream | Status |
|---------|----------|--------|
| `GameRenderer` trait | `tui:shell` | Trusted — 82 tests pass |
| `CfrGame`, `GameType` | `games:traits` | Trusted — 14 tests pass |
| `myosu-play` binary | `play:tui` | Partial — skeleton exists |
| `docs/api/game-state.json` | `agent:experience` | Trusted — schema complete |
| `schema.rs` | `agent:experience` | Trusted — 16 tests pass |
| Chain data (lobby) | `chain:runtime` | **MISSING** — stubbed for Phase 0 |
| Miner axon (spectator WS) | `chain:runtime` | **MISSING** — Phase 1 |

---

## Integration Invariants

1. **Context privacy**: Agent context file is never sent to opponents or observers
2. **Journal append-only**: Journal file is never truncated or overwritten
3. **Fog-of-war at relay**: Hole cards never flow to spectator socket during active play
4. **Pipe mode purity**: No ANSI codes, no cursor manipulation, no TTY dependency
5. **Same game state**: `--narrate` and default pipe mode render the same underlying game state
