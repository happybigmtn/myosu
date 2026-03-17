# Specification: TUI Implementation — Shared Game Renderer Architecture

Source: design.md interface system + GP-01..04 gameplay CLI spec
Status: Draft
Date: 2026-03-16
Depends-on: GT-01..05 (game engine traits), GP-01..04 (gameplay CLI), design.md

## Purpose

Implement the shared TUI rendering architecture that all 20 games use, AND
the reference NLHE poker implementation including live gameplay against
trained MCCFR solvers.

`DESIGN.md` defines WHAT the interface looks like. This spec defines HOW
to build it in ratatui — the widget architecture, the game-agnostic renderer
trait, the event loop, input handling, screen management, AND the complete
NLHE poker experience including local training mode, blueprint strategy
loading, and solver-guided play.

The key design decisions:
1. The TUI is a **game-agnostic shell** with a pluggable **game state panel**.
   Adding a new game means implementing one trait (`GameRenderer`) — no
   changes to the shell, event loop, input handling, log panel, or
   declaration system.
2. The NLHE renderer is the **reference implementation** that proves the
   architecture works. It ships as Phase 0 with full local gameplay.
3. Players can **see what the trained solver recommends** for their current
   decision point — the solver advisor panel. This is the user-facing value
   proposition: play against the solver AND learn from it.

## Whole-System Goal

Current state:
- DESIGN.md defines 30+ screens (20 games, 4 operational, lobby, coaching, onboarding, wallet, spectator)
- GP-01..04 spec defines gameplay logic but not rendering architecture
- codexpoker repo has production TUI implementation (~33K lines) to port from
- No TUI code exists in myosu yet

This spec adds:
- `myosu-tui` crate with the shared rendering shell
- `GameRenderer` trait — the only per-game customization point
- Event loop with async miner queries and keyboard input
- Readline-style input with history, tab-complete, /commands
- Screen management (lobby → game → stats → lobby)
- `--pipe` mode stripping all formatting for agent stdin/stdout
- **NLHE poker renderer** — reference GameRenderer implementation
- **Training mode** — local practice against blueprint/heuristic bot
- **Blueprint strategy loading** — trained MCCFR artifacts from disk
- **Solver advisor** — display solver's recommended actions for hero's spot
- **Truth stream** — privacy-respecting event log with visual grammar

If all ACs land:
- Any game implementing `GameRenderer` renders correctly in the shared shell
- Human and agent input flows through the same code path
- The TUI works at 60-120 columns with responsive layout
- A human plays NLHE heads-up against a trained solver in the TUI
- The solver advisor shows action distributions during gameplay
- Training mode works without chain connectivity
- All DESIGN.md screens are implementable (20 games + system flows)

Still not solved here:
- Non-poker `GameRenderer` implementations (each game spec owns that)
- Network layer (miner queries — MN spec owns that)
- Chain queries (subnet discovery — shared chain client owns that)
- LLM coaching (external API dependency — Phase 1)

12-month direction:
- 20 game renderers sharing one shell
- Spectator mode (watch agent vs agent)
- Tournament bracket display
- LLM coaching overlay (streaming advice from GPT/Claude)
- Real-time solver comparison (hero play vs GTO recommendation)

## Why This Spec Exists As One Unit

- The shell, renderer trait, event loop, and input handling form one
  architectural surface. Testing any piece requires the others.
- The `GameRenderer` trait is the contract between the shell and every
  game — getting it wrong affects all 20 games.

## Scope

In scope:
- `myosu-tui` crate (shared shell)
- `GameRenderer` trait definition
- 5-panel layout implementation (header, declaration, state, log, input)
- Event loop with crossterm keyboard events
- Readline input with history and tab completion
- Screen state machine (onboarding → lobby → game → stats → wallet → spectate)
- Declaration system (state → declaration text)
- Color semantic application
- `--pipe` mode for agent protocol
- NLHE HU `GameRenderer` as reference implementation
- Training mode: local HU practice against blueprint/heuristic bot
- Blueprint strategy loading: artifact discovery, manifest, mmap lookup
- Solver advisor: display trained solver's action distribution for hero
- Truth stream: event processing, visual grammar, card rendering

Out of scope:
- Non-poker `GameRenderer` implementations
- Miner query networking (uses injected async function)
- Chain RPC queries (uses injected async function)
- Operational screens (network console, miner inspection — separate spec)
- LLM coaching (requires external API — Phase 1)
- Multiplayer / distributed table (network layer — separate spec)
- Real token wagering (play money only in training mode)

## Current State

- design.md defines all visual contracts
- `crates/myosu-play/` stub exists but has no TUI code
- ratatui + crossterm are available in the Rust ecosystem

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| TUI framework | ratatui 0.29+ | reuse | Standard Rust TUI framework |
| Terminal backend | crossterm (event-stream feature) | reuse | Cross-platform terminal + async EventStream |
| Input widget | tui-textarea 0.7+ | reuse | Built-in emacs keybindings (Ctrl-A/E/K/U/W/Y), undo/redo |
| Scrollable panel | tui-scrollview 0.6+ | reuse | Variable-height styled content scrolling |
| Color system | ratatui::style::{Color, Style, Modifier, Stylize} | reuse | Maps to design.md color semantics |
| Layout | ratatui::layout::{Layout, Constraint} with Fill | reuse | 5-panel vertical layout, Fill for flex |
| Async events | tokio + crossterm EventStream | reuse | `tokio::select!` for concurrent key + miner events |
| Training mode | codexpoker training.rs (40K lines) | port + adapt | TrainingTable, PendingCommands, bot dispatch |
| Blueprint loading | codexpoker blueprint.rs (206K lines) | port + adapt | Artifact discovery, manifest, mmap strategy lookup |
| Truth stream | codexpoker truth_stream.rs (65K lines) | port + adapt | Event processing, privacy model, visual grammar |
| Visual grammar | codexpoker tui/style.rs | port + adapt | Icons (→↑✕◈◉~), separators (·), SplitBorder (┃) |
| Gameboard | codexpoker tui/gameboard.rs | port + adapt | Pinned state panel, pot odds, MDF calculation |
| Card rendering | codexpoker tui/style.rs:cards_with_suit_symbols | port + adapt | Unicode suit symbols (♠♥♦♣), rank display |
| HUD stats | codexpoker tui/hud.rs + stats.rs | port + adapt | VPIP/PFR/AF tracking |
| Game engine | robopoker (vendored) Game, Recall, Action, Turn | reuse | Game state machine, action legality |
| Bot backend | codexpoker blueprint.rs BotBackend trait | port + adapt | Strategy fallback: Blueprint → Heuristic |

## Design Consistency: Reconciling design.md and codexpoker

Audit conducted 2026-03-17. Three sources compared: myosu `DESIGN.md` (the
design system), codexpoker TUI (33K lines, production prototype), and this
TUI spec. Where they conflict, decisions are documented below.

### Design philosophy: design.md is the vision, codexpoker is the engine

design.md already has an elevated, distinctive design system (sections 13-14).
codexpoker was built for information density — every pixel crammed with data.
myosu is a different product: a learning tool where players study GTO by
facing a solver and seeing its recommendations. The design should serve
**clarity and comprehension**, not density.

**What we take from design.md** (the visual identity):
- Declarations as hero text (signature element)
- `───` separator rhythm (breath between sections, not borders)
- Two-space indent as information hierarchy
- Field-label state panel (aligned columns)
- Prose action log ("solver raises to 6bb" not "↑ preflop · Seat 0 raises-to 6")
- One accent per screen (color restraint)
- ALLCAPS for declarations and section headers only
- The anti-lazygit test

**What we take from codexpoker** (the engine):
- `FlexRenderable` layout allocation (proven algorithm)
- Newline-gated streaming (prevents mid-line flicker)
- `Renderable` trait contract (render, desired_height, cursor_pos)
- Blueprint loading via mmap (< 1μs strategy lookup)
- `TrainingTable` game loop (bot dispatch, pending commands)
- `BotBackend` trait (strategy fallback chain)
- Action parser shorthands (f/c/r/s, bare numbers)
- Hand shadowing (`fg.dim` for completed hands)
- Card suit symbol rendering (A♠ K♥ Q♦ J♣)

**What we do NOT take from codexpoker** (visual noise):
- Icon-prefix log lines (→ ↑ ✕ ◈ ◉ ~) — replaced by prose
- SplitBorder prefix (┃) — replaced by two-space indent
- Separator-joined clauses (` · `) — replaced by column alignment
- 7 simultaneous ColorRole colors — replaced by design.md's restrained palette

### The reimagined NLHE screen

This is the target rendering for Phase 0 NLHE heads-up:

```
MYOSU / NLHE-HU / HAND 47

THE SYSTEM AWAITS YOUR DECISION

  board    T♠  7♥  2♣  ·  ·
  you      A♠ K♥    top pair        950bb   BB
  solver   ·· ··                   1050bb   SB
  pot      12bb     call 4bb        raise 8–950bb

  EQUILIBRIUM

  raise    53%      call 35%        fold 12%

───

  solver raises to 6bb               pot 9
  you call                           pot 13
  ─── flop: T♠ 7♥ 2♣
  solver checks                      pot 13

> raise 8
```

**What makes this work:**

1. **Declaration as headline**: "THE SYSTEM AWAITS YOUR DECISION" is the first
   thing the eye reads. Not a status bar — a statement.

2. **Field-label state panel**: Aligned columns, readable at a glance. Cards,
   stacks, positions all have clear spatial homes. No separators needed —
   whitespace creates the grid.

3. **Decision context on the pot line**: `call 4bb` and `raise 8–950bb` sit
   next to `pot 12bb` because that's where the player thinks about amounts.
   All the numbers are in one visual cluster.

4. **EQUILIBRIUM as a sub-section**: The solver advisor is a section header
   (flush left, ALLCAPS) with the distribution as field values. It reads as
   part of the interface, not a bolted-on feature.

5. **Prose action log with right-aligned pot**: "solver raises to 6bb" is
   immediately comprehensible. The pot running total sits on the right margin,
   available for glancing but not competing for attention. No icons — the verbs
   ARE the landmarks.

6. **Street transitions as separator + content**: `─── flop: T♠ 7♥ 2♣` uses
   the separator rhythm but adds information. It marks a visual break AND shows
   the new board state.

7. **Color restraint**: Only two colors in play during normal gameplay:
   - `fg.bright` (#ffffff) for cards, amounts, and the active declaration
   - `fg.dim` (#606060) for labels, separators, completed hands
   - `converge` (#00cc66) appears only for positive results (+14bb win)
   - `diverge` (#cc3333) appears only for errors or losses
   The default screen is essentially monochrome with bright/dim contrast.

### Color system: restraint over variety

design.md defines 8 tokens. Not all are visible at once. One accent dominates
per screen.

```
TOKEN          HEX        WHEN VISIBLE
──────────────────────────────────────────────────────
fg             #c0c0c0    always (default text)
fg.bright      #ffffff    cards, amounts, active declaration, hero line
fg.dim         #606060    labels, separators, completed hands, folded lines
converge       #00cc66    win results, positive amounts
diverge        #cc3333    errors, losses, timeout warnings
unstable       #ccaa00    fallback-active declaration, time pressure
focus          #4488cc    rarely — system/protocol info on operational screens
protocol       #8844cc    rarely — myosu branding on splash/lobby only
```

During normal gameplay, only `fg`, `fg.bright`, and `fg.dim` are visible.
Color enters sparingly: a green flash on "+14bb", a red flash on errors.
This makes the rare color moments impactful.

**Modifiers** (from codexpoker, proven useful):
- DIM modifier on `fg.dim` for completed hands (visual recession)
- Italic modifier for LLM/coach responses (Phase 1, signals "not fact")

### Layout: design.md's 5-panel with codexpoker's engine

```
┌────────────────────────────────────────────────────┐
│ MYOSU / NLHE-HU / HAND 47                         │  header (1 line)
├────────────────────────────────────────────────────┤
│ THE SYSTEM AWAITS YOUR DECISION                    │  declaration (1 line)
├────────────────────────────────────────────────────┤
│                                                    │
│   board    T♠  7♥  2♣  ·  ·                       │  state (4-8 lines)
│   you      A♠ K♥    top pair     950bb   BB        │
│   solver   ·· ··                1050bb   SB        │
│   pot      12bb    call 4bb     raise 8–950bb      │
│                                                    │
│   EQUILIBRIUM                                      │
│   raise    53%     call 35%     fold 12%           │
│                                                    │
├────────────────────────────────────────────────────┤
│ ───                                                │
│                                                    │
│   solver raises to 6bb            pot 9            │  log (flex, scrollable)
│   you call                        pot 13           │
│   ─── flop: T♠ 7♥ 2♣                              │
│   solver checks                   pot 13           │
│                                                    │
├────────────────────────────────────────────────────┤
│ > raise 8                                          │  input (1 line)
└────────────────────────────────────────────────────┘
```

Panel ordering follows design.md: state ABOVE log. Unlike codexpoker's
"gameboard pinned above composer" pattern, design.md's ordering puts the
decision context at the top of the viewport (most important = highest).
The log scrolls below.

Implementation via codexpoker's FlexRenderable:
- header: flex=0, 1 line
- declaration: flex=0, 1 line
- state: flex=0, 4-8 lines (desired_height varies by game + advisor)
- log: flex=1, takes remaining space
- input: flex=0, 1 line

### Log format: prose with right-aligned pot

design.md's prose format is cleaner and more comprehensible than codexpoker's
icon-prefix format. For a learning tool, readability beats scannability.

```
  solver raises to 6bb               pot 9
  you call                           pot 13
  ─── flop: T♠ 7♥ 2♣
  solver checks                      pot 13
  you bet 8bb                        pot 21
```

- Two-space indent (design.md hierarchy)
- Prose verbs: "raises to", "calls", "checks", "bets", "folds", "shoves"
- Right-aligned pot running total (glanceable, not competing)
- `─── flop: T♠ 7♥ 2♣` street transitions (separator + content)
- Hand boundaries: blank line + "hand 48" on next hand start
- Completed hands rendered in `fg.dim` (hand shadowing from codexpoker)

No icons. The verbs ARE the landmarks. "fold" reads clearly without ✕.
"raises to 6bb" reads clearly without ↑.

### Hidden card convention

- `··` (two dots) = opponent cards, known count (poker hole cards = 2)
- `░░░` (bars) = unknown count (remaining deck, mahjong wall)

### Streaming behavior (from codexpoker engine)

Newline-gated streaming prevents mid-line flicker:
- Text accumulates in hidden `pending_buffer`
- Only commits to display when `\n` received
- Each committed line gets a sequence number
- LLM responses (Phase 1) stream with visible cursor (▌)

### Input parsing (from codexpoker engine)

Port codexpoker's proven shorthands:
- `f` / `fold` → Fold
- `c` / `call` → Call (or Check if nothing to call)
- `x` / `check` → Check
- `r 15` / `raise 15` / `bet 15` → Raise(15bb)
- `s` / `shove` / `all-in` → Shove
- Bare `15` → Raise(15bb) when in decision context
- `?` → show legal actions with ranges

Confirmation follows design.md's prose style:
```
> r 15
you raise to 15bb.
```

Invalid input follows design.md's clarification pattern:
```
> raise
raise to how much? (min 4bb, max 94bb)
```

### Patterns NOT ported from codexpoker

| Pattern | Reason not ported |
|---------|-------------------|
| Icon-prefix log lines (→ ↑ ✕ ◈ ◉ ~) | Replaced by prose format |
| SplitBorder prefix (┃) | Replaced by two-space indent |
| Separator-joined clauses (` · `) | Replaced by column alignment |
| 7 simultaneous ColorRole colors | Replaced by restrained palette |
| Visual table mode | Not needed for HU Phase 0 |
| Coach panel (LLM) | External API dependency, Phase 1 |
| Fairness overlay | Ziffle crypto, not in myosu |
| Voice/audio | Not in scope |
| P2P networking | Separate from TUI layer |
| Splash menu | myosu goes straight to lobby |
| Auth flows / Economy system | myosu uses chain keys/tokens |
| Action modal | myosu uses text input, not buttons |

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| TUI crate | New | crates/myosu-tui/src/lib.rs |
| Renderable trait | New (port) | crates/myosu-tui/src/renderable.rs |
| FlexRenderable | New (port) | crates/myosu-tui/src/renderable.rs |
| GameRenderer trait | New | crates/myosu-tui/src/renderer.rs |
| Shell (5-panel layout) | New | crates/myosu-tui/src/shell.rs |
| Event loop | New | crates/myosu-tui/src/events.rs |
| Input handler | New | crates/myosu-tui/src/input.rs |
| Declaration system | New | crates/myosu-tui/src/declaration.rs |
| Screen state machine | New | crates/myosu-tui/src/screens.rs |
| Pipe mode | New | crates/myosu-tui/src/pipe.rs |
| Color theme | New | crates/myosu-tui/src/theme.rs |
| NLHE renderer | New (port) | crates/myosu-games-poker/src/renderer.rs |
| Truth stream | New (port) | crates/myosu-games-poker/src/truth_stream.rs |
| Training mode | New (port) | crates/myosu-play/src/training.rs |
| Blueprint loading | New (port) | crates/myosu-play/src/blueprint.rs |
| Bot backend trait | New (port) | crates/myosu-play/src/bot.rs |
| Solver advisor | New | crates/myosu-play/src/advisor.rs |

## Architecture / Runtime Contract

```
┌───────────────────────────────────────────────────────────────┐
│ myosu-play binary                                              │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ myosu-tui (shared shell)                                  │ │
│  │                                                           │ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────┐ │ │
│  │  │ header   │  │ events   │  │ screens  │  │ theme   │ │ │
│  │  │ declare  │  │ loop     │  │ state    │  │         │ │ │
│  │  │ log      │  │ (async)  │  │ machine  │  │         │ │ │
│  │  │ input    │  │          │  │          │  │         │ │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └─────────┘ │ │
│  │                                                           │ │
│  │  ┌───────────────────────────────────────────────────┐   │ │
│  │  │ GameRenderer (trait object)                        │   │ │
│  │  │                                                    │   │ │
│  │  │  NlheRenderer (TU-08)                             │   │ │
│  │  │  ├── TruthStreamEmitter (TU-12) → log panel       │   │ │
│  │  │  ├── GameboardState → state panel (cards/pot/etc)  │   │ │
│  │  │  └── SolverAdvisor (TU-11) → advisor line         │   │ │
│  │  └───────────────────────────────────────────────────┘   │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                                │
│  ┌─────────────────┐  ┌──────────────────┐  ┌──────────────┐ │
│  │ TrainingTable    │  │ myosu-chain-client│  │ HTTP client  │ │
│  │ (TU-09)         │  │ (subnet discovery)│  │ (miner axon) │ │
│  │                  │  └──────────────────┘  └──────────────┘ │
│  │  Game (robopoker)│                                         │
│  │  BotBackend ─────┤                                         │
│  │    ├─ Blueprint  │  ┌──────────────────┐                   │
│  │    └─ Heuristic  │  │ Blueprint loader │                   │
│  └─────────────────┘  │ (TU-10)          │                   │
│                        │ mmap artifacts   │                   │
│                        └──────────────────┘                   │
└───────────────────────────────────────────────────────────────┘
```

Mode dispatch:
```
myosu-play --train              → TrainingTable (local, no chain)
myosu-play --chain ws://...     → MinerQuery (chain-connected, GP-01..03)
myosu-play --pipe --train       → TrainingTable + pipe mode (agent)
```

Solver advisor data flow:
```
Hero decision pending
  │
  ├─ Training mode: Blueprint.action_distribution(observation)
  │    └─ Display: "SOLVER: fold 12% · call 35% · raise 53%"
  │
  └─ Chain mode: POST miner/strategy with hero's observation
       └─ Display: "SOLVER: fold 8% · call 42% · raise 50%"
```

Primary loop:
- Trigger: crossterm key event or async miner response
- Source of truth: game state (from game engine) + miner responses
- Processing: update game state, re-render affected panels
- Persisted truth: hand history JSON files
- Consumer: terminal display (human) or stdout (agent via --pipe)

---

## A. Core Trait

### AC-TU-01: GameRenderer Trait

- Where: `crates/myosu-tui/src/renderer.rs (new)`
- How: Define the trait that every game must implement to render in the shell:

  **Renderable trait** (from codexpoker renderable.rs, the base rendering contract):
  ```rust
  use ratatui::buffer::Buffer;
  use ratatui::layout::Rect;

  pub trait Renderable {
      fn render(&self, area: Rect, buf: &mut Buffer);
      fn desired_height(&self, width: u16) -> u16;
      fn cursor_pos(&self, _area: Rect) -> Option<(u16, u16)> { None }
  }
  ```

  All shell components (header, transcript, gameboard, composer) implement
  `Renderable`. Components report their desired height, and `FlexRenderable`
  allocates vertical space (flex=0 for fixed, flex>0 for proportional).

  **GameRenderer trait** (game-specific extension point):
  ```rust
  pub trait GameRenderer: Send {
      /// Render the game-specific state panel into the given area.
      /// This is the ONLY method a game must implement for rendering.
      fn render_state(&self, area: Rect, buf: &mut Buffer);

      /// Return desired state panel height for the given width.
      /// Returns 0 when no active hand (panel collapses).
      fn desired_height(&self, width: u16) -> u16;

      /// Return the current declaration text (ALLCAPS).
      fn declaration(&self) -> &str;

      /// Return available actions for tab completion.
      fn completions(&self) -> Vec<String>;

      /// Parse user input into a game action. Returns None if invalid.
      fn parse_input(&self, input: &str) -> Option<String>;

      /// Return clarification prompt for ambiguous input.
      fn clarify(&self, input: &str) -> Option<String>;

      /// Render game state as plain text for --pipe mode.
      fn pipe_output(&self) -> String;

      /// Header path component (e.g., "NLHE-HU", "RIICHI", "HWATU")
      fn game_label(&self) -> &str;

      /// Header context (e.g., "HAND 47", "EAST 1 ROUND 3", "ROUND 5")
      fn context_label(&self) -> &str;
  }
  ```

  `render_state` receives a `Rect` and `Buffer` (matching codexpoker's
  rendering contract). The shell handles header, declaration, log, and
  input — the game only draws its state panel. `desired_height` controls
  panel sizing (returns 0 when inactive, 2-4 when active).

- Whole-system effect: this is THE extension point for all 20 games. Get the
  trait wrong and every game implementation suffers.
- State: no runtime state — trait definition.
- Wiring contract:
  - Trigger: compile-time, consumed by game crates
  - Callsite: shell.rs calls `renderer.render_state()` each frame
  - State effect: N/A
  - Persistence effect: N/A
  - Observable signal: trait compiles and is implementable
- Required tests:
  - `cargo test -p myosu-tui renderer::tests::trait_is_object_safe`
  - `cargo test -p myosu-tui renderer::tests::mock_renderer_works`
- Pass/fail:
  - `GameRenderer` is object-safe (`Box<dyn GameRenderer>` compiles)
  - A mock renderer implementing all methods compiles and renders
  - `pipe_output()` returns valid structured text
  - `completions()` returns non-empty list for active game state
- Blocking note: every game depends on this trait. Must be stable before
  any game renderer is implemented.
- Rollback condition: trait requires types that are game-specific, breaking
  the game-agnostic contract.

### AC-TU-02: Five-Panel Shell Layout

- Where: `crates/myosu-tui/src/shell.rs (new)`
- How: Implement the 5-panel vertical layout from design.md using codexpoker's
  FlexRenderable engine (proven allocation: fixed-first, flex-second,
  remainder to last flex child).

  Panel ordering follows design.md: state above log. The decision context
  (cards, board, advisor) is the most important information — it sits highest
  in the viewport. The log scrolls below.

  ```rust
  let mut layout = FlexRenderable::new();
  layout.push(0, &header);          // flex=0, fixed 1 line
  layout.push(0, &declaration);     // flex=0, fixed 1 line
  layout.push(0, &state_panel);     // flex=0, 0-8 lines (game-specific)
  layout.push(1, &log);             // flex=1, takes remaining space
  layout.push(0, &input);           // flex=0, fixed 1 line
  ```

  The state panel `desired_height()` returns 0 when no active hand (collapses).
  Returns 4-8 lines when active (depending on game and EQUILIBRIUM visibility).
  Games with large state (mahjong: 12 lines) push the log smaller; games with
  small state (Liar's Dice: 4 lines) give more to the log.

  The header renders: `MYOSU / {game_label} / {context_label}`
  The declaration renders the `GameRenderer::declaration()` text (ALLCAPS).
  The state panel is game-specific (field-label format with column alignment).
  The log renders scrollable prose action history with right-aligned pot.
  The input renders: `> ` prompt + current buffer with cursor.

- Whole-system effect: universal layout for all 20 games.
- State: log entries `Vec<String>`, scroll offset, input buffer.
- Wiring contract:
  - Trigger: terminal resize or state change
  - Callsite: event loop calls `shell.render(frame, renderer)`
  - State effect: frame buffer updated
  - Persistence effect: N/A
  - Observable signal: 5 panels visible in terminal
- Required tests:
  - `cargo test -p myosu-tui shell::tests::layout_at_60_columns`
  - `cargo test -p myosu-tui shell::tests::layout_at_120_columns`
  - `cargo test -p myosu-tui shell::tests::state_panel_minimum_4_lines`
- Pass/fail:
  - All 5 panels render at 60 columns without overflow
  - State panel never goes below 4 lines even with long log
  - Header shows correct path from GameRenderer
  - Log scrolls when content exceeds panel height
- Blocking note: this is the visual frame all games sit inside.
- Rollback condition: layout constraints conflict at small terminal sizes.

### AC-TU-03: Event Loop and Async Updates

- Where: `crates/myosu-tui/src/events.rs (new)`
- How: Crossterm event loop with tokio for async miner queries:

  ```rust
  loop {
      // Render current state
      terminal.draw(|frame| shell.render(frame, &renderer))?;

      // Poll for events (16ms = ~60fps)
      if crossterm::event::poll(Duration::from_millis(16))? {
          match crossterm::event::read()? {
              Event::Key(key) => handle_key(key, &mut shell, &mut renderer),
              Event::Resize(w, h) => { /* re-layout */ },
              _ => {}
          }
      }

      // Check for async miner responses
      if let Ok(response) = miner_rx.try_recv() {
          renderer.apply_miner_response(response);
      }
  }
  ```

  Miner queries are sent via a `tokio::sync::mpsc` channel from a background
  task. The event loop polls the receive end non-blockingly.

- Whole-system effect: responsive UI with non-blocking miner queries.
- State: terminal state, channel endpoints.
- Wiring contract:
  - Trigger: keyboard input or miner response
  - Callsite: main.rs spawns event loop
  - State effect: game state updated, frame re-rendered
  - Persistence effect: N/A
  - Observable signal: UI responds within 16ms to key presses
- Required tests:
  - `cargo test -p myosu-tui events::tests::key_event_handled`
  - `cargo test -p myosu-tui events::tests::async_response_received`
- Pass/fail:
  - Key press triggers re-render within one frame (16ms)
  - Miner response updates game state without blocking input
  - Ctrl-C triggers clean shutdown
- Blocking note: the event loop is the heartbeat of the TUI.
- Rollback condition: crossterm and tokio event loops conflict.

### AC-TU-04: Readline Input with History

- Where: `crates/myosu-tui/src/input.rs (new)`
- How: Custom input handler with:
  - Character buffer with cursor position
  - Up/down arrow for command history (last 100 commands)
  - Tab completion from `GameRenderer::completions()`
  - `/` prefix detection for meta-commands
  - Enter submits, passes to `GameRenderer::parse_input()`
  - If parse returns None, show `GameRenderer::clarify()` prompt

  Keybindings (readline-compatible):
  - `Ctrl-A`: move to start
  - `Ctrl-E`: move to end
  - `Ctrl-W`: delete word backward
  - `Ctrl-U`: delete to start
  - `Ctrl-K`: delete to end
  - `Left/Right`: move cursor
  - `Up/Down`: history navigation
  - `Tab`: cycle completions
  - `Enter`: submit

- Whole-system effect: consistent input experience across all 20 games.
- State: input buffer, cursor position, history ring, completion state.
- Wiring contract:
  - Trigger: key events from event loop
  - Callsite: events.rs delegates key events to input handler
  - State effect: input buffer updated
  - Persistence effect: N/A
  - Observable signal: typed characters appear, completions cycle
- Required tests:
  - `cargo test -p myosu-tui input::tests::type_and_submit`
  - `cargo test -p myosu-tui input::tests::history_navigation`
  - `cargo test -p myosu-tui input::tests::tab_completion`
  - `cargo test -p myosu-tui input::tests::ctrl_w_deletes_word`
  - `cargo test -p myosu-tui input::tests::slash_command_detected`
- Pass/fail:
  - Typing "raise 15" + Enter submits "raise 15"
  - Up arrow recalls previous command
  - Tab completes "ra" → "raise" when "raise" is in completions
  - `/quit` detected as meta-command, not game action
  - Ctrl-W on "raise 15" leaves "raise "
- Blocking note: input quality determines gameplay feel.
- Rollback condition: readline keybindings conflict with game-specific keys.

### AC-TU-05: Screen State Machine

- Where: `crates/myosu-tui/src/screens.rs (new)`
- How: Simple enum state machine:

  ```rust
  enum Screen {
      Onboarding,     // first-run setup (DESIGN.md 8.0a-8.0c)
      Lobby,          // game selection (DESIGN.md 9.23)
      Game,           // active gameplay (DESIGN.md 9.1-9.20, all 20 games)
      Stats,          // session summary (DESIGN.md 10.4)
      Coaching,       // /analyze output (DESIGN.md 9.22)
      History,        // /history output
      Wallet,         // account + staking (DESIGN.md 8.0d)
      Spectate,       // watch agent vs agent (DESIGN.md 9.24)
  }
  ```

  Transitions:
  - First run: Onboarding → Lobby (automatic after setup completes)
  - Subsequent runs: straight to Lobby (key loaded from `~/.myosu/key`)
  - Lobby → Game (player selects subnet by id)
  - Game → Stats (hand/game completes or /stats)
  - Game → Coaching (/analyze)
  - Game → History (/history)
  - Game → Wallet (/wallet)
  - Stats → Game (new session)
  - Stats → Lobby (/quit)
  - Coaching → Game (any key)
  - History → Game (any key)
  - Lobby → Wallet (/wallet)
  - Wallet → previous screen (/back)
  - Lobby → Spectate (/spectate)
  - Spectate → Lobby ([q] quit)

  ```
  Onboarding ──► Lobby ◄──► Wallet
                   │  ▲
                   │  │ /quit
                   ▼  │
                 Game ◄──► Coaching
                   │  ▲     History
                   │  │      Wallet
                   ▼  │
                 Stats
                   │
                   ▼
                Spectate
  ```

  Each screen has its own render method. The Lobby, Stats, Wallet,
  Onboarding, and Spectate screens don't need a GameRenderer — they use
  static layouts from DESIGN.md.

  **20-game coverage:** DESIGN.md defines mockups for all 20 games from
  OS.md (sections 9.1-9.20). Of these, only the NLHE HU renderer (TU-08)
  ships in Phase 0. The remaining 19 game renderers are design-only —
  each will be implemented as its own `GameRenderer` when the
  corresponding game engine is built. The shell architecture (this spec)
  supports all 20 without modification.

- Whole-system effect: navigation between game states.
- State: current Screen enum.
- Wiring contract:
  - Trigger: /commands or game completion
  - Callsite: event loop checks screen transitions
  - State effect: screen changes, renderer swaps
  - Persistence effect: N/A
  - Observable signal: display switches between screens
- Required tests:
  - `cargo test -p myosu-tui screens::tests::lobby_to_game`
  - `cargo test -p myosu-tui screens::tests::game_to_stats`
  - `cargo test -p myosu-tui screens::tests::slash_analyze_to_coaching`
  - `cargo test -p myosu-tui screens::tests::onboarding_to_lobby`
  - `cargo test -p myosu-tui screens::tests::wallet_back_navigation`
  - `cargo test -p myosu-tui screens::tests::spectate_from_lobby`
- Pass/fail:
  - Typing "1" in lobby transitions to Game with subnet 1
  - /stats during game shows Stats screen
  - /quit from Stats returns to Lobby
  - Any key from Coaching returns to Game
  - First run with no key file starts at Onboarding
  - Onboarding completion transitions to Lobby
  - /wallet from Lobby or Game transitions to Wallet
  - /back from Wallet returns to previous screen
  - /spectate from Lobby transitions to Spectate
- Blocking note: screen management is how users navigate the app.
- Rollback condition: screen transitions lose game state.

### AC-TU-06: Pipe Mode for Agent Protocol

- Where: `crates/myosu-tui/src/pipe.rs (new)`
- How: When `--pipe` flag is set:
  - Disable ratatui alternate screen (no TUI rendering)
  - On each game state change, print `GameRenderer::pipe_output()` to stdout
  - Read lines from stdin as input commands
  - No color codes, no box-drawing, no cursor manipulation
  - Flush stdout after every write

  The pipe mode binary path:
  ```
  stdin → parse_input → game engine → pipe_output → stdout
  ```

  This enables: `agent_a | myosu-play --pipe | agent_b`

- Whole-system effect: agents play through the same binary as humans.
- State: no TUI state — just stdin/stdout.
- Wiring contract:
  - Trigger: `--pipe` CLI flag
  - Callsite: main.rs selects pipe mode vs TUI mode
  - State effect: game state updated per stdin line
  - Persistence effect: same hand history as TUI mode
  - Observable signal: structured text on stdout, accepts stdin
- Required tests:
  - `cargo test -p myosu-tui pipe::tests::pipe_output_no_ansi`
  - `cargo test -p myosu-tui pipe::tests::stdin_accepted`
  - `cargo test -p myosu-tui pipe::tests::pipe_output_matches_design_md`
- Pass/fail:
  - `--pipe` output contains zero ANSI escape codes
  - `--pipe` output matches design.md pipe format exactly
  - stdin line "call" produces game state update on stdout
  - Agent can play a complete hand via pipe
- Blocking note: agent-native design depends on pipe mode working.
- Rollback condition: game state rendering diverges between TUI and pipe mode.

### AC-TU-07: Color Theme Implementation

- Where: `crates/myosu-tui/src/theme.rs (new)`
- How: Map design.md's 8 semantic color tokens to ratatui styles, reconciled
  with codexpoker's proven ColorRole patterns (see Design Consistency section).

  ```rust
  pub struct Theme {
      // design.md 8-token palette (use Rgb for visual identity)
      pub fg: Color,           // #c0c0c0 — default text
      pub fg_bright: Color,    // #ffffff — emphasis, cards, active decision
      pub fg_dim: Color,       // #606060 — history, folded, metadata
      pub converge: Color,     // #00cc66 — positive: win, amounts, convergence
      pub diverge: Color,      // #cc3333 — negative: error, loss, violation
      pub unstable: Color,     // #ccaa00 — warning: time pressure, instability
      pub focus: Color,        // #4488cc — system info: icons, pot, solver
      pub protocol: Color,     // #8844cc — rare: protocol identity, branding
  }

  impl Theme {
      pub fn default_rgb() -> Self {
          Self {
              fg:        Color::Rgb(192, 192, 192),
              fg_bright: Color::White,
              fg_dim:    Color::Rgb(96, 96, 96),
              converge:  Color::Rgb(0, 204, 102),
              diverge:   Color::Rgb(204, 51, 51),
              unstable:  Color::Rgb(204, 170, 0),
              focus:     Color::Rgb(68, 136, 204),
              protocol:  Color::Rgb(136, 68, 204),
          }
      }
  }
  ```

  **Mapping to codexpoker ColorRole usage patterns:**
  - `focus` = icons (was codexpoker Accent/Cyan)
  - `converge` = amounts, pot, values (was codexpoker Value/Green)
  - `fg_dim` = separators, structure (was codexpoker Muted/Gray)
  - `diverge` = errors, rejected actions (was codexpoker Danger/Red)
  - `unstable` = street transitions, board (was codexpoker Highlight/Yellow)
  - `protocol` = hero prompts (was codexpoker UserAction/Magenta)
  - `fg_bright` = default bright text
  - `fg` = default text

  **Additional modifiers** (from codexpoker, not new color tokens):
  - `fold_style`: fg_dim + DIM modifier — for fold lines
  - `hand_shadow`: fg_dim + DIM — for non-current hand history
  - `assistant_style`: Italic — for LLM/coach text (not facts)

  Color is applied as an overlay — the interface MUST be readable without
  color (design.md invariant). Theme is injected, not hardcoded, for future
  customization.

- Required tests:
  - `cargo test -p myosu-tui theme::tests::all_colors_defined`
  - `cargo test -p myosu-tui theme::tests::readable_without_color`
  - `cargo test -p myosu-tui theme::tests::fold_style_is_dim`
  - `cargo test -p myosu-tui theme::tests::hand_shadow_is_dim`
- Pass/fail:
  - All 8 color tokens from design.md are defined with Rgb values
  - Removing all color from a rendered frame still produces readable output
  - Fold lines use DIM modifier (visually recede)
  - Non-current hand lines use hand_shadow style
- Blocking note: visual consistency across all screens.
- Rollback condition: N/A — theme is pure configuration.

---

## B. NLHE Poker Implementation

These ACs implement the live poker game in the TUI, ported from the
codexpoker prototype and adapted for the myosu solver network.

### AC-TU-08: NLHE Poker Renderer and Truth Stream

- Where: `crates/myosu-games-poker/src/renderer.rs (new)`, `crates/myosu-games-poker/src/truth_stream.rs (new)`
- How: Implement `GameRenderer` for NLHE heads-up, producing the full poker
  TUI experience. Port from codexpoker's `truth_stream.rs`, `tui/gameboard.rs`,
  and `tui/style.rs`.

  **State panel** (renders into the `GameRenderer::render_state` area):
  Uses design.md's field-label format with column alignment. No SplitBorder
  prefix, no separator-joined clauses. Two-space indent for all data lines,
  flush-left for section headers.

  When hero has a pending decision (full state):
  ```
    board    T♠  7♥  2♣  ·  ·
    you      A♠ K♥    top pair        950bb   BB
    solver   ·· ··                   1050bb   SB
    pot      12bb     call 4bb        raise 8–950bb

    EQUILIBRIUM

    raise    53%      call 35%        fold 12%
  ```

  When not hero's turn (no decision context, no advisor):
  ```
    board    T♠  7♥  2♣
    you      A♠ K♥    top pair        950bb   BB
    solver   ·· ··                   1050bb   SB
    pot      12bb
  ```

  Preflop (no board dealt):
  ```
    board    ·  ·  ·  ·  ·
    you      A♠ K♥                    950bb   BB
    solver   ·· ··                   1050bb   SB
    pot      3bb      call 1bb        raise 4–950bb

    EQUILIBRIUM

    raise    62%      call 28%        fold 10%
  ```

  Height: 4 lines (no advisor) to 8 lines (with EQUILIBRIUM section).
  Collapses to 0 when no active hand (desired_height returns 0).
  Replaced on every state change.

  **Color**: Restrained per design philosophy.
  - `fg.bright` (#ffffff) for cards (A♠ K♥), amounts (950bb), hand strength
  - `fg` (#c0c0c0) for labels (board, you, solver, pot), position (BB, SB)
  - `fg.dim` (#606060) for hidden cards (··), section headers (EQUILIBRIUM)
  - Empty board slots (·) rendered in `fg.dim`

  The state panel uses a `GameboardState` struct:
  ```rust
  pub struct GameboardState {
      pub street: String,        // preflop, flop, turn, river
      pub board: Vec<String>,    // ["T♠","7♥","2♣"] (len 0-5)
      pub pot: u32,
      pub hero_hole: String,     // "A♠ K♥"
      pub hero_stack: u32,
      pub hero_position: String, // "BB" or "SB"
      pub opponent_stack: u32,
      pub opponent_position: String,
      pub to_call: u32,
      pub has_decision: bool,
      pub raise_min: u32,
      pub raise_max: u32,
      pub strength: String,      // "top pair", "flush draw", etc.
  }
  ```

  **Log panel** (prose format from design.md, with right-aligned pot):
  ```
    solver raises to 6bb               pot 9
    you call                           pot 13
    ─── flop: T♠ 7♥ 2♣
    solver checks                      pot 13
    you bet 8bb                        pot 21
  ```

  Two-space indent. Prose verbs. Right-aligned pot running total.
  Street transitions use the `───` separator with board cards.
  No icons — verbs are the landmarks.

  The `TruthStreamEmitter` processes robopoker `Event`s into log lines:
  - `Event::Play(Action)` → prose log line with pot total
  - `Event::YourTurn(Recall)` → updates state panel (private, not logged)
  - `Event::ShowHand(_, Hole)` → showdown line
  - `Event::NextHand(_, Meta)` → blank line + hand start

  **Log line types:**
  ```
  HandStart    hand 48                                 (blank line before)
  Blind        solver posts SB 1bb             pot 3
  Action       you raise to 6bb                pot 9
  Fold         solver folds                    pot 9   (fg.dim)
  Street       ─── flop: T♠ 7♥ 2♣
  Showdown     solver shows Q♣ J♣  two pair
  Result       you win 14bb                            (converge for win)
  Error        miner 12 unreachable (timeout 500ms)    (diverge)
  Fallback     fallback: random over legal actions     (fg.dim)
  ```

  **Hand shadowing** (from codexpoker, proven pattern):
  - Non-current hand lines rendered in `fg.dim` + DIM modifier
  - Active hand in normal `fg` color
  - Provides visual separation without blank lines between hands

  **Streaming** (codexpoker engine pattern):
  - Text accumulates in hidden `pending_buffer`
  - Only commits to display when `\n` received
  - Prevents mid-line flicker and partial renders

  **GameRenderer methods**:
  - `render_state()`: draw gameboard (2-4 lines) + optional advisor line
  - `declaration()`: "YOUR TURN" / "BOT THINKING" / "SHOWDOWN" / "HAND COMPLETE"
  - `completions()`: ["fold", "check", "call", "raise", "shove", "/deal", "/board", "/advisor"]
  - `parse_input()`: "f"→Fold, "c"→Check/Call, "r 15"→Raise(15), "15"→Raise(15), "s"→Shove
  - `clarify()`: "call 8 or raise? (c/r [amount])"
  - `pipe_output()`: machine-readable state (street|board|pot|hero|stack|to_call|actions)
  - `game_label()`: "NLHE-HU"
  - `context_label()`: "HAND 47 · FLOP"

  **Pot odds and MDF** (ported from codexpoker `gameboard.rs:64-94`):
  - pot_odds_pct = to_call / (pot + to_call) × 100
  - mdf_pct = pot / (pot + to_call) × 100
  - Displayed when facing a bet: `pot odds 28% · MDF 72%`

- Whole-system effect: the reference poker UI that proves the GameRenderer
  architecture. Every subsequent game renderer follows this pattern.
- State: GameboardState, TruthStreamEmitter (hand#, street, pot, board, active seats).
- Wiring contract:
  - Trigger: game state change (action applied, street dealt, hand boundary)
  - Callsite: shell.rs calls `renderer.render_state()` each frame
  - State effect: gameboard updated, log lines appended
  - Persistence effect: N/A (hand recording is GP-04)
  - Observable signal: cards render with suit symbols, actions appear in log
- Required tests:
  - `cargo test -p myosu-games-poker renderer::tests::render_preflop_state`
  - `cargo test -p myosu-games-poker renderer::tests::render_flop_with_board`
  - `cargo test -p myosu-games-poker truth_stream::tests::action_produces_log_line`
  - `cargo test -p myosu-games-poker truth_stream::tests::showdown_shows_cards`
  - `cargo test -p myosu-games-poker renderer::tests::pipe_output_structured`
  - `cargo test -p myosu-games-poker renderer::tests::parse_raise_amount`
  - `cargo test -p myosu-games-poker renderer::tests::pot_odds_calculation`
- Pass/fail:
  - Preflop state shows hero cards and pot, no board
  - Flop state shows board cards with suit symbols
  - Each action produces exactly one truth stream line
  - Showdown reveals both players' cards
  - pipe_output has zero ANSI codes and is machine-parseable
  - "r 15" parses to Raise(15), "f" to Fold, "c" to Call/Check
  - Pot odds = 28.6% when facing 4 into pot of 10
- Blocking note: reference implementation that validates TU-01 trait design.
  Must be stable before non-poker renderers are attempted.
- Rollback condition: GameRenderer trait needs methods not anticipated by TU-01.

### AC-TU-09: Training Mode (Local Bot Play)

- Where: `crates/myosu-play/src/training.rs (new)`
- How: Port codexpoker's `TrainingTable` for local heads-up practice against a
  blueprint or heuristic bot. No network connectivity required.

  **TrainingTable** wraps robopoker's `Game` engine:
  ```rust
  pub struct TrainingTable {
      game: Game,
      history: Vec<Action>,
      hand_num: u32,
      practice_chips: u32,
      hero_seat: usize,
      pending: PendingTrainingCommands,
      bot_backend: Arc<dyn BotBackend>,
  }
  ```

  **BotBackend** trait (strategy fallback chain):
  ```rust
  pub trait BotBackend: Send + Sync + std::fmt::Debug {
      fn select_action(&self, recall: &Recall, seat: usize) -> Action;
      fn strategy_name(&self) -> &str;
      fn action_distribution(&self, recall: &Recall, seat: usize) -> Vec<(Action, f64)>;
  }
  ```

  Implementations:
  1. `BlueprintBackend` — trained MCCFR policy from artifact (TU-10)
  2. `HeuristicBackend` — equity-based GTO approximation (always available)

  On startup: attempt blueprint load. If fails, fall back to heuristic.
  Display fallback reason: `~ bot strategy: heuristic · blueprint not found`

  **Training commands** (ported from codexpoker):
  - `/deal A♠ K♥` — set hero's hole cards for next hand
  - `/board Q♥ J♥ 9♦` — set board cards
  - `/stack 200` — set hero stack (in BB)
  - `/bot-stack 200` — set bot stack (in BB)
  - `/showdown` — force to showdown (no further betting)

  **Game loop**:
  ```
  loop {
      deal hand (apply pending commands)
      loop {
          if hero's turn → wait for input
          if bot's turn → bot_backend.select_action(recall, bot_seat)
          apply action → emit Event → truth stream processes
          if hand complete → break
      }
      update practice chips balance
      display hand result in log
      alternating button (hero is BTN odd hands, BB even hands)
  }
  ```

  **Practice chips**: 10,000 default, resets on `/practice` or session entry.
  Displayed in header: `PRACTICE · 10,450 chips (+450)`

  **Bot thinking delay**: 200-500ms random delay before bot acts (feels natural).
  Configurable: `MYOSU_BOT_DELAY_MS` env var (0 disables for testing).

- Whole-system effect: standalone poker practice without chain infrastructure.
  Proves the gameplay works before miners/validators are running.
- State: TrainingTable (game, history, chips, pending commands, bot backend).
- Wiring contract:
  - Trigger: `myosu-play --train` or `/practice` command
  - Callsite: main.rs creates TrainingTable, passes NlheRenderer
  - State effect: game progresses, chips update
  - Persistence effect: hand history JSON (via GP-04 recorder, shared)
  - Observable signal: hands play to completion, bot acts within 500ms
- Required tests:
  - `cargo test -p myosu-play training::tests::hand_completes_fold`
  - `cargo test -p myosu-play training::tests::hand_completes_showdown`
  - `cargo test -p myosu-play training::tests::deal_command_sets_cards`
  - `cargo test -p myosu-play training::tests::bot_backend_fallback`
  - `cargo test -p myosu-play training::tests::practice_chips_update`
  - `cargo test -p myosu-play training::tests::alternating_button`
- Pass/fail:
  - Hero fold → hand ends, bot wins pot, practice chips deducted
  - Full showdown → correct hand evaluation, pot awarded, chips updated
  - `/deal A♠ K♥` → next hand hero has those cards
  - Blueprint load fails → heuristic fallback with actionable message
  - Practice chips start at 10,000, update correctly after each hand
  - Button alternates between hero and bot each hand
- Blocking note: training mode is the Phase 0 poker experience. Requires
  TU-08 (renderer) and either TU-10 (blueprint) or heuristic fallback.
- Rollback condition: robopoker's Game API insufficient for bot dispatch.

### AC-TU-10: Blueprint Strategy Loading

- Where: `crates/myosu-play/src/blueprint.rs (new)`
- How: Port codexpoker's blueprint artifact loading for trained MCCFR strategy
  lookup. This powers both the bot opponent (TU-09) and the solver advisor (TU-11).

  **Artifact discovery** (env var → default path):
  1. `MYOSU_BLUEPRINT_DIR` — explicit directory override
  2. `MYOSU_DATA_DIR/.myosu/blueprints/` — default location
  3. `~/.myosu/blueprints/` — fallback

  **Artifact files** (produced by miner training or AP-01 pipeline):
  - `blueprint.manifest.json` — metadata (schema version, game format, hash)
  - `blueprint.keys.bin` — strategy keys (memory-mapped)
  - `blueprint.values.bin` — strategy values (memory-mapped)
  - `blueprint.isomorphism.bin` — card abstraction map (obs → bucket index)

  **Manifest** (ported from codexpoker):
  ```rust
  pub struct BlueprintManifest {
      pub schema_version: u32,       // current: 1 (myosu schema)
      pub game_format: GameFormat,   // Cash
      pub player_count: u8,          // 2 (HU)
      pub abstraction_hash: String,  // SHA-256 of isomorphism file
      pub profile_hash: String,      // SHA-256 of keys+values
      pub iterations: u64,           // training iterations completed
      pub exploitability: f64,       // measured exploit (mbb/h)
  }
  ```

  **Strategy lookup path** (< 1μs per query):
  ```
  Game state (Recall)
    → Observation (hero pocket + board)
      → Isomorphism (canonical form, suit-agnostic)
        → Bucket index (from isomorphism.bin, mmap)
          → Action distribution (from keys.bin + values.bin, mmap)
  ```

  The key innovation from codexpoker: memory-mapped files avoid loading the
  entire strategy into RAM. A 50MB profile stays on disk, accessed via page faults.

  **BlueprintBackend** (implements BotBackend):
  ```rust
  impl BotBackend for BlueprintBackend {
      fn select_action(&self, recall: &Recall, seat: usize) -> Action {
          let dist = self.action_distribution(recall, seat);
          sample_from_distribution(&dist)
      }
      fn action_distribution(&self, recall: &Recall, seat: usize) -> Vec<(Action, f64)> {
          let obs = Observation::from(recall, seat);
          let iso = Isomorphism::from(&obs);
          let bucket = self.isomorphism_map.lookup(iso);
          self.profile.distribution(bucket, recall.street())
      }
  }
  ```

  **Error handling** (graceful fallback with actionable messages):
  ```
  ~ bot strategy: heuristic · blueprint not found (set MYOSU_BLUEPRINT_DIR)
  ~ bot strategy: heuristic · blueprint schema v2 unsupported (expected v1)
  ~ bot strategy: heuristic · abstraction hash mismatch (artifact corrupted)
  ```

- Whole-system effect: enables trained solver play without running chain infra.
  The same artifacts miners produce (via MCCFR training) can be used locally.
- State: memory-mapped files (read-only after load).
- Wiring contract:
  - Trigger: training mode startup or `/practice` command
  - Callsite: training.rs tries BlueprintBackend::load(), falls back to Heuristic
  - State effect: mmap files opened, strategy available for lookup
  - Persistence effect: read-only (artifacts produced elsewhere)
  - Observable signal: `~ bot strategy: blueprint · exploit 2.3 mbb/h`
- Required tests:
  - `cargo test -p myosu-play blueprint::tests::load_valid_artifact`
  - `cargo test -p myosu-play blueprint::tests::missing_dir_returns_error`
  - `cargo test -p myosu-play blueprint::tests::schema_mismatch_returns_error`
  - `cargo test -p myosu-play blueprint::tests::hash_mismatch_returns_error`
  - `cargo test -p myosu-play blueprint::tests::lookup_returns_valid_distribution`
  - `cargo test -p myosu-play blueprint::tests::distribution_sums_to_one`
- Pass/fail:
  - Valid artifact directory → BlueprintBackend loaded, strategy_name() = "blueprint"
  - Missing directory → BlueprintError::ArtifactNotFound with actionable message
  - Schema mismatch → BlueprintError::SchemaMismatch
  - Corrupted file → BlueprintError::HashMismatch
  - lookup returns distribution that sums to 1.0 ± epsilon
  - All returned actions are legal in the given game state
- Blocking note: without blueprints, training mode uses heuristic-only bot.
  Blueprint loading makes the local bot as strong as a trained miner.
- Rollback condition: myosu abstraction format incompatible with codexpoker format
  (addressed by using AP-01 output format from the start).

### AC-TU-11: Solver Advisor

- Where: `crates/myosu-play/src/advisor.rs (new)`
- How: Display the trained solver's recommended action distribution for the
  hero's current decision point. This is the key user-facing value proposition:
  play against the solver AND see what it would recommend for your spots.

  **Advisor display** (rendered as EQUILIBRIUM section in the state panel):
  ```
    EQUILIBRIUM

    raise    53%      call 35%        fold 12%
  ```

  When multiple raise sizes are significant:
  ```
    EQUILIBRIUM

    raise 8  47%      raise 12  19%     call 22%      check 12%
  ```

  The EQUILIBRIUM section uses design.md's established pattern: flush-left
  ALLCAPS header, two-space-indented field-label data with column alignment.
  It reads as a natural part of the state panel, not a bolted-on overlay.

  **Advisor data source** (depends on play mode):
  ```rust
  pub trait SolverAdvisor: Send {
      /// Get the solver's action distribution for hero's current spot.
      /// Returns None if no solver available or not hero's turn.
      fn advise(&self, recall: &Recall, hero_seat: usize) -> Option<Vec<(Action, f64)>>;
  }
  ```

  - **Training mode**: uses the SAME blueprint backend as the bot.
    The bot samples from the distribution; the advisor SHOWS the distribution.
    This means the hero sees exactly what the GTO solver recommends, while
    the bot opponent plays from that same distribution (with randomized sampling).

  - **Chain mode** (future): queries the miner axon with the hero's observation,
    receives the action distribution, displays it. Same display format.

  **Advisor toggle**:
  - `/advisor` — toggle advisor on/off
  - `/advisor on` — enable
  - `/advisor off` — disable
  - Default: ON in training mode, OFF in chain mode (to avoid revealing miner strategy)

  **Advisor formatting rules**:
  - Only show actions with probability > 1%
  - Round probabilities to nearest integer
  - Actions ordered by probability (highest first)
  - Color: `fg.dim` (#606060) for the EQUILIBRIUM header, `fg` for values
  - Uses field-label alignment (action name left, percentage right of tab stop)
  - Include raise sizing when multiple sizes are significant (> 10%)
  - When no advisor available: EQUILIBRIUM section absent entirely
  - Blank line before and after EQUILIBRIUM (breathing room)

  **Decision snapshot** for advisor context (adapted from codexpoker's DecisionSnapshot):
  ```rust
  pub struct AdvisorContext {
      pub street: Street,
      pub board: Vec<Card>,
      pub hero_hole: [Card; 2],
      pub pot: u32,
      pub hero_stack: u32,
      pub to_call: u32,
      pub raise_min: u32,
      pub raise_max: u32,
  }
  ```

  This struct is constructed from the game's `Recall` whenever hero has a pending
  decision. The advisor queries the solver backend with this context.

  **Integration with NlheRenderer (TU-08)**:
  The renderer's `render_state()` checks if advisor is enabled and hero has a
  pending decision. If so, it appends the advisor line to the state panel,
  growing it from 2 lines to 3 lines. The shell's `Constraint::Min(4)` for the
  state panel accommodates this.

- Whole-system effect: the solver advisor transforms the game from "play against
  a bot" into "learn GTO poker from a trained solver." Players see exactly what
  Nash-approximate play looks like for their specific situation.
- State: advisor enabled/disabled toggle, cached distribution for current decision.
- Wiring contract:
  - Trigger: hero decision pending + advisor enabled
  - Callsite: NlheRenderer::render_state() calls advisor.advise()
  - State effect: advisor line rendered in state panel
  - Persistence effect: N/A
  - Observable signal: "SOLVER: fold X% · call Y% · raise Z%" visible
- Required tests:
  - `cargo test -p myosu-play advisor::tests::blueprint_advisor_returns_distribution`
  - `cargo test -p myosu-play advisor::tests::no_decision_returns_none`
  - `cargo test -p myosu-play advisor::tests::distribution_filters_below_1pct`
  - `cargo test -p myosu-play advisor::tests::format_distribution_text`
  - `cargo test -p myosu-play advisor::tests::toggle_on_off`
- Pass/fail:
  - Hero decision pending → advisor shows distribution with probabilities
  - Not hero's turn → advisor line absent
  - Actions < 1% probability → filtered from display
  - `/advisor off` → advisor line disappears
  - Distribution probabilities sum to ~100% (rounding may cause ±1%)
  - Advisor works with both BlueprintBackend and HeuristicBackend
- Blocking note: the solver advisor is the differentiating feature. Without it,
  training mode is just "play poker against a bot." With it, training mode is
  "learn GTO from a trained solver in real-time."
- Rollback condition: BotBackend::action_distribution() too slow for real-time
  display (should be < 1ms from mmap lookup).

### AC-TU-12: Session Stats and HUD

- Where: `crates/myosu-play/src/stats.rs (new)`, `crates/myosu-games-poker/src/hud.rs (new)`
- How: Track and display session statistics during training mode play.
  Ported from codexpoker's `stats.rs` and `tui/hud.rs`.

  **Session stats** (displayed after each hand and on /stats):
  - Hands played
  - Win rate (BB/hand)
  - Total profit/loss (in practice chips)
  - Best hand
  - Showdown win %

  **HUD overlay** (toggle with `/hud`):
  ```
  ┌─ BOT STATS ──────────────────┐
  │ VPIP 72% · PFR 45% · AF 2.1 │
  │ sample: 47 hands              │
  └───────────────────────────────┘
  ```

  **Stat tracking** (ported from codexpoker):
  ```rust
  pub struct PlayerStats {
      pub hands_played: u32,
      pub vpip_hands: u32,      // voluntarily put money in pot
      pub pfr_hands: u32,       // preflop raise
      pub aggression_bets: u32, // bets + raises
      pub aggression_calls: u32,
      pub showdowns: u32,
      pub showdown_wins: u32,
  }
  ```

  Stats with < 30 hands sample marked with `*` (unreliable indicator).
  Stats reset on session entry (new `/practice` command).

  **Stats screen** (TU-05 Screen::Stats):
  ```
  SESSION SUMMARY

  hands    47
  profit   +450 chips (BB/h: +4.8)
  showdown 23/47 (49%)
  best     A♠ A♥ full house (hand #31)

  bot: blueprint · exploit 2.3 mbb/h

  /practice to play again · /quit to exit
  ```

- Whole-system effect: session feedback loop — player sees their performance
  and the bot's tendencies over time.
- State: PlayerStats (per session, reset on entry).
- Wiring contract:
  - Trigger: hand completion
  - Callsite: training.rs updates stats after each hand
  - State effect: stats counters incremented
  - Persistence effect: N/A (session-scoped)
  - Observable signal: header shows chip count, /stats shows full summary
- Required tests:
  - `cargo test -p myosu-play stats::tests::vpip_tracks_correctly`
  - `cargo test -p myosu-play stats::tests::bb_per_hand_calculation`
  - `cargo test -p myosu-play stats::tests::unreliable_marker_below_30`
  - `cargo test -p myosu-games-poker hud::tests::render_hud_overlay`
- Pass/fail:
  - VPIP incremented when hero voluntarily puts money in pot
  - BB/hand = total_profit / hands_played
  - Stats with < 30 hands show `*` marker
  - HUD overlay renders within state panel area without overlap
- Blocking note: stats provide session feedback; not blocking for core gameplay.
- Rollback condition: N/A — pure accumulation, no complex state.

---

## Operational Controls

Phase order:

```
Phase A: Shell infrastructure
  1. TU-01 (GameRenderer trait) — defines the contract
  2. TU-07 (theme) — needed by TU-02
  3. TU-02 (shell layout) — the visual frame
  4. TU-04 (input) — keyboard handling
  5. TU-03 (event loop) — ties shell + input together
  6. TU-05 (screens) — navigation between views
  7. TU-06 (pipe mode) — agent protocol

Phase B: NLHE poker (depends on Phase A)
  8. TU-10 (blueprint loading) — enables trained bot + advisor
  9. TU-08 (NLHE renderer + truth stream) — reference GameRenderer
  10. TU-09 (training mode) — local bot play
  11. TU-11 (solver advisor) — display solver recommendations
  12. TU-12 (session stats + HUD) — feedback loop
```

Phase B dependency graph:
```
TU-10 (blueprint loading)
  │
  ├──► TU-09 (training mode) ──► TU-12 (stats)
  │      │
  │      ▼
  │    TU-08 (NLHE renderer)
  │      │
  │      ▼
  └──► TU-11 (solver advisor)
```

## Decision Log

- 2026-03-16: `GameRenderer` as trait object (`Box<dyn GameRenderer>`) rather
  than generic — enables runtime game selection from subnet game_type.
- 2026-03-16: `Constraint::Min(4)` for state panel — games with large state
  (mahjong) expand naturally, small games (Liar's Dice) leave room for log.
- 2026-03-16: Separate `myosu-tui` crate from `myosu-play` — the TUI shell
  is reusable for operational screens (network console, etc.) not just gameplay.
- 2026-03-16: Port codexpoker patterns rather than build from scratch — the
  prototype has 33K lines of production-quality TUI code covering training mode,
  blueprint loading, truth stream, and visual grammar. Porting is lower risk
  and faster than reimplementation.
- 2026-03-16: Solver advisor ON by default in training mode — the whole point
  of training mode is to learn GTO. Showing the solver's recommendations is
  the value proposition, not a feature toggle buried in menus.
- 2026-03-16: Solver advisor OFF by default in chain mode — showing a remote
  miner's strategy while playing against it would be "cheating" (the miner's
  strategy is proprietary). Chain mode advisor requires explicit opt-in.
- 2026-03-16: BotBackend::action_distribution() is the shared API between bot
  decisions (sample one action) and solver advisor (display all actions). Same
  trait method serves both use cases, ensuring consistency.
- 2026-03-16: Memory-mapped blueprint files from codexpoker — avoids loading
  50MB+ profiles into RAM. Strategy lookup stays < 1μs per query via page faults.
- 2026-03-16: Training commands (/deal, /board, /stack, /showdown) ported from
  codexpoker — enables scenario drilling ("what does the solver recommend when I
  have A♠ K♥ on a Q♥ J♥ 9♦ board?").

## Reference Test Fixture: Codexpoker Blueprint

A trained MCCFR blueprint exists on disk and serves as the validation
fixture for TU-10 (blueprint loading) and TU-11 (solver advisor).

**Location:** `~/.codexpoker/blueprint/`

```
~/.codexpoker/blueprint/
├── blueprint.manifest.json    schema v1, 113M infosets, 335M edges
├── blueprint.keys.bin         3.2 GB (strategy keys, mmap)
├── blueprint.values.bin       4.0 GB (strategy values, mmap)
└── blueprint.isomorphism.bin  1.3 GB (obs → abstraction, mmap)
```

**Manifest:**
```json
{
  "schema_version": 1,
  "build_id": "bp-local-import",
  "position_aware": false,
  "num_infosets": 113192911,
  "num_edges": 334866228,
  "keys_hash": "a58efa575e5a5827555460adc47b754cde4b7baac2d9c4d8f0cd453174256fd6",
  "values_hash": "284e53651aa77ee74e1be3a25ffdb0a1cc0a1fa38af0562314d4b6ccb32716de"
}
```

**Validated edge metrics (2026-03-17):**

| Test | Hands | Result | Interpretation |
|------|-------|--------|----------------|
| Blueprint vs Random | 1000 | +2.35 chips/hand, 60% win rate | ~117 mbb/h edge over random |
| Blueprint vs Blueprint | 1000 | -0.54 chips/hand, 49.9% win rate | Near-zero (balanced self-play) |
| Practice Probe | 5 | Mixed strategies (fold 12%, raise 11%, etc.) | Policy lookups return valid distributions |

**How to use for testing:**

1. **TU-10 validation:** Load the artifact via `BlueprintBackend::load()`.
   Assert manifest parses, mmap succeeds, and `action_distribution()` returns
   distributions that sum to 1.0 for arbitrary game states.

2. **TU-11 validation:** With blueprint loaded, verify solver advisor shows
   distributions matching `practice_probe` output — mixed strategies with
   fold/call/raise probabilities, not degenerate pure-action responses.

3. **Edge regression:** Run bot_duel equivalent (1000 hands blueprint vs
   random). Assert edge > 1.0 chips/hand. This catches blueprint corruption
   or loading bugs that produce garbage strategies.

4. **Balance check:** Run mirror mode (blueprint vs itself). Assert
   |edge| < 1.0 chips/hand over 1000 hands. Deviation indicates loading
   asymmetry (e.g., seat-dependent bugs in isomorphism lookup).

**Codexpoker testing binaries** (for reference during porting):
- `crates/codexpoker/src/bin/bot_duel.rs` — edge measurement
- `crates/codexpoker/src/bin/practice_probe.rs` — policy visualization
- `crates/codexpoker/src/bin/test_blueprint_load.rs` — loading diagnostics

**Environment:** Set `MYOSU_BLUEPRINT_DIR=~/.codexpoker/blueprint` to use
this fixture with myosu's blueprint loading code. The myosu schema (v1) is
designed to be compatible with this artifact format.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | Mock renderer draws in 5-panel layout | Shell + trait | TU-01, TU-02 |
| 2 | Type "raise 15" + Enter, text appears in log | Input + events | TU-03, TU-04 |
| 3 | Tab completes "ra" → "raise" | Input | TU-04 |
| 4 | /stats switches to Stats screen | Screen mgmt | TU-05 |
| 5 | `--pipe` outputs plain text, accepts stdin | Agent protocol | TU-06 |
| 6 | Blueprint artifact loaded, bot uses trained strategy | Blueprint loading | TU-10 |
| 7 | Hero plays full hand vs blueprint bot in training mode | Training mode | TU-08, TU-09, TU-10 |
| 8 | Solver advisor shows action distribution on hero turn | Advisor | TU-11 |
| 9 | /deal A♠ K♥ then solver shows recommendation for those cards | Training cmds + advisor | TU-09, TU-11 |
| 10 | Session stats correct after 10 hands, HUD shows bot tendencies | Stats | TU-12 |
| 11 | Full session: 10 hands in TUI with advisor, stats, blueprint bot | End-to-end | All |
| 12 | Blueprint edge > 1.0 chips/hand vs random over 1000 hands | Strategy quality | TU-10 |
| 13 | Blueprint mirror |edge| < 1.0 chips/hand over 1000 hands | Balance check | TU-10 |
