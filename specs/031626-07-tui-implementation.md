# Specification: TUI Implementation — Shared Game Renderer Architecture

Canonical note: this content was promoted into the numbered `031626-07` slot on
2026-03-29 after living in the non-numbered mirror during the earlier bootstrap
phase. Historical note: this spec predates the current `myosu-play`
subcommand CLI. For the live Stage 0 surface, read `myosu-play --pipe` as
`myosu-play pipe`. Future flags described here remain spec intent until they
land in code.

Source: design.md interface system + GP-01..04 gameplay CLI spec
Status: Draft
Date: 2026-03-16
Depends-on: GT-01..05 (game engine traits), GP-01..04 (gameplay CLI), design.md

## Purpose

Implement the shared TUI rendering architecture that all 20 games use.
`design.md` defines WHAT the interface looks like. This spec defines HOW
to build it in ratatui — the widget architecture, the game-agnostic renderer
trait, the event loop, input handling, and screen management.

The key design decision: the TUI is a **game-agnostic shell** with a
pluggable **game state panel**. Adding a new game means implementing one
trait (`GameRenderer`) — no changes to the shell, event loop, input handling,
log panel, or declaration system.

## Whole-System Goal

Current state:
- design.md defines 15 screens (9 games, 4 operational, lobby, coaching)
- GP-01..04 spec defines gameplay logic but not rendering architecture
- No TUI code exists

This spec adds:
- `myosu-tui` crate with the shared rendering shell
- `GameRenderer` trait — the only per-game customization point
- Event loop with async miner queries and keyboard input
- Readline-style input with history, tab-complete, /commands
- Screen management (lobby → game → stats → lobby)
- `--pipe` mode stripping all formatting for agent stdin/stdout

If all ACs land:
- Any game implementing `GameRenderer` renders correctly in the shared shell
- Human and agent input flows through the same code path
- The TUI works at 60-120 columns with responsive layout
- All 15 design.md screens are implementable

Still not solved here:
- Game-specific `GameRenderer` implementations (each game spec owns that)
- Network layer (miner queries — MN spec owns that)
- Chain queries (subnet discovery — shared chain client owns that)

12-month direction:
- 20 game renderers sharing one shell
- Spectator mode (watch agent vs agent)
- Tournament bracket display
- Coaching overlay panel

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
- Screen state machine (lobby → game → stats)
- Declaration system (state → declaration text)
- Color semantic application
- `--pipe` mode for agent protocol
- NLHE HU `GameRenderer` as reference implementation

Out of scope:
- Non-poker `GameRenderer` implementations
- Miner query networking (uses injected async function)
- Chain RPC queries (uses injected async function)
- Operational screens (network console, miner inspection — separate spec)

## Current State

- design.md defines all visual contracts
- `crates/myosu-play/` stub exists but has no TUI code
- ratatui + crossterm are available in the Rust ecosystem

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| TUI framework | ratatui 0.29+ | reuse | Standard Rust TUI framework |
| Terminal backend | crossterm | reuse | Cross-platform terminal manipulation |
| Input widget | tui-textarea crate | evaluate | May provide readline-style input |
| Scrollable panel | ratatui built-in Paragraph with scroll | reuse | For log panel |
| Color system | ratatui::style::{Color, Style, Modifier} | reuse | Maps to design.md color semantics |
| Layout | ratatui::layout::{Layout, Constraint, Direction} | reuse | 5-panel vertical layout |

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| TUI crate | New | crates/myosu-tui/src/lib.rs |
| GameRenderer trait | New | crates/myosu-tui/src/renderer.rs |
| Shell (5-panel layout) | New | crates/myosu-tui/src/shell.rs |
| Event loop | New | crates/myosu-tui/src/events.rs |
| Input handler | New | crates/myosu-tui/src/input.rs |
| Declaration system | New | crates/myosu-tui/src/declaration.rs |
| Screen state machine | New | crates/myosu-tui/src/screens.rs |
| Pipe mode | New | crates/myosu-tui/src/pipe.rs |
| Color theme | New | crates/myosu-tui/src/theme.rs |
| NLHE renderer (reference) | New | crates/myosu-games-poker/src/renderer.rs |

## Architecture / Runtime Contract

```
┌─────────────────────────────────────────────────────┐
│ myosu-play binary                                    │
│                                                      │
│  ┌─────────────────────────────────────────────┐    │
│  │ myosu-tui (shared shell)                     │    │
│  │                                              │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  │    │
│  │  │ header   │  │ events   │  │ screens  │  │    │
│  │  │ declare  │  │ loop     │  │ state    │  │    │
│  │  │ log      │  │ (async)  │  │ machine  │  │    │
│  │  │ input    │  │          │  │          │  │    │
│  │  └──────────┘  └──────────┘  └──────────┘  │    │
│  │                                              │    │
│  │  ┌──────────────────────────────────────┐   │    │
│  │  │ GameRenderer (trait object)           │   │    │
│  │  │ provided by game-specific crate       │   │    │
│  │  │ e.g. myosu-games-poker::NlheRenderer │   │    │
│  │  └──────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────┘    │
│                                                      │
│  ┌──────────────────┐  ┌──────────────────┐         │
│  │ myosu-chain-client│  │ HTTP client      │         │
│  │ (subnet discovery)│  │ (miner queries)  │         │
│  └──────────────────┘  └──────────────────┘         │
└─────────────────────────────────────────────────────┘
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

  ```rust
  use ratatui::Frame;
  use ratatui::layout::Rect;

  pub trait GameRenderer: Send {
      /// Render the game-specific state panel into the given area.
      /// This is the ONLY method a game must implement.
      fn render_state(&self, frame: &mut Frame, area: Rect);

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

  `render_state` receives a `Rect` (the state panel area) and draws into it
  using ratatui primitives. The shell handles header, declaration, log, and
  input — the game only draws its state.

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
- How: Implement the 5-panel vertical layout from design.md:

  ```rust
  let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
          Constraint::Length(1),        // header
          Constraint::Length(2),        // declaration
          Constraint::Min(4),           // state (game-specific, flex)
          Constraint::Percentage(40),   // log (scrollable)
          Constraint::Length(1),        // input
      ])
      .split(frame.area());
  ```

  The state panel uses `Constraint::Min(4)` — it gets whatever space the log
  doesn't need, with a minimum of 4 lines. This allows games with large state
  (mahjong: 12 lines) to push the log smaller, while games with small state
  (Liar's Dice: 4 lines) give more space to the log.

  The header renders: `MYOSU / {game_label} / {context_label}`
  The declaration renders the `GameRenderer::declaration()` text.
  The log renders a scrollable `Vec<String>` of action history.
  The input renders the current input buffer with cursor.

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
      Lobby,          // game selection (design.md 8.12)
      Game,           // active gameplay (design.md 8.1-8.9)
      Stats,          // session summary (design.md 9.4)
      Coaching,       // /analyze output (design.md 8.11)
      History,        // /history output
  }
  ```

  Transitions:
  - Lobby → Game (player selects subnet)
  - Game → Stats (hand/game completes or /stats)
  - Game → Coaching (/analyze)
  - Game → History (/history)
  - Stats → Game (new session)
  - Stats → Lobby (/quit)
  - Coaching → Game (any key)
  - History → Game (any key)

  Each screen has its own render method. The Lobby and Stats screens don't
  need a GameRenderer — they use static layouts from design.md.

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
- Pass/fail:
  - Typing "1" in lobby transitions to Game with subnet 1
  - /stats during game shows Stats screen
  - /quit from Stats returns to Lobby
  - Any key from Coaching returns to Game
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

  This enables: `agent_a | myosu-play pipe | agent_b`

- Whole-system effect: agents play through the same binary as humans.
- State: no TUI state — just stdin/stdout.
- Wiring contract:
  - Trigger: `pipe` subcommand
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
- How: Map design.md color semantics to ratatui styles:

  ```rust
  pub struct Theme {
      pub fg: Color,           // #c0c0c0
      pub fg_bright: Color,    // #ffffff
      pub fg_dim: Color,       // #606060
      pub converge: Color,     // #00cc66
      pub diverge: Color,      // #cc3333
      pub unstable: Color,     // #ccaa00
      pub focus: Color,        // #4488cc
      pub protocol: Color,     // #8844cc
  }

  impl Theme {
      pub fn style_declaration(&self, state: GameState) -> Style {
          match state {
              GameState::Normal => Style::default().fg(self.fg_bright),
              GameState::Converging => Style::default().fg(self.converge),
              GameState::Diverging => Style::default().fg(self.diverge),
              GameState::Unstable => Style::default().fg(self.unstable),
              _ => Style::default().fg(self.fg_bright),
          }
      }
  }
  ```

  Color is applied as an overlay — the interface MUST be readable without
  color (design.md invariant). Theme is injected, not hardcoded, for future
  customization.

- Required tests:
  - `cargo test -p myosu-tui theme::tests::all_colors_defined`
  - `cargo test -p myosu-tui theme::tests::readable_without_color`
- Pass/fail:
  - All 8 color tokens from design.md are defined
  - Removing all color from a rendered frame still produces readable output
- Blocking note: visual consistency across all screens.
- Rollback condition: N/A — theme is pure configuration.

---

## Operational Controls

Phase order:
1. TU-01 (GameRenderer trait) — defines the contract
2. TU-07 (theme) — needed by TU-02
3. TU-02 (shell layout) — the visual frame
4. TU-04 (input) — keyboard handling
5. TU-03 (event loop) — ties shell + input together
6. TU-05 (screens) — navigation between views
7. TU-06 (pipe mode) — agent protocol

## Decision Log

- 2026-03-16: `GameRenderer` as trait object (`Box<dyn GameRenderer>`) rather
  than generic — enables runtime game selection from subnet game_type.
- 2026-03-16: `Constraint::Min(4)` for state panel — games with large state
  (mahjong) expand naturally, small games (Liar's Dice) leave room for log.
- 2026-03-16: Separate `myosu-tui` crate from `myosu-play` — the TUI shell
  is reusable for operational screens (network console, etc.) not just gameplay.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | Mock renderer draws in 5-panel layout | Shell + trait | TU-01, TU-02 |
| 2 | Type "raise 15" + Enter, text appears in log | Input + events | TU-03, TU-04 |
| 3 | Tab completes "ra" → "raise" | Input | TU-04 |
| 4 | /stats switches to Stats screen | Screen mgmt | TU-05 |
| 5 | `pipe` outputs plain text, accepts stdin | Agent protocol | TU-06 |
| 6 | NLHE game plays one complete hand in TUI | End-to-end | All |
