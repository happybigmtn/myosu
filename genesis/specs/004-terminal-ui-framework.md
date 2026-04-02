# Specification: Terminal UI Framework

Source: Reverse-engineered from crates/myosu-tui (shell.rs, events.rs, pipe.rs, renderer.rs, screens.rs)
Status: Draft
Depends-on: none

## Purpose

The terminal UI framework provides a game-agnostic interactive shell for
rendering imperfect-information games in the terminal. It defines a five-panel
layout, an event loop bridging async background tasks with synchronous TUI
rendering, a screen navigation state machine, and a pipe protocol for
non-human consumers. Any game that implements the GameRenderer trait can be
played through this framework without modification.

The primary consumers are the gameplay binary (myosu-play) and any future game
crates that need interactive terminal rendering.

## Whole-System Goal

Current state: The framework is fully implemented and used by both poker and
Liar's Dice through the GameRenderer trait. It supports interactive TUI mode
and non-interactive pipe mode.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: New games can provide terminal rendering by implementing
GameRenderer, inheriting the five-panel layout, event loop, screen navigation,
input handling, and pipe protocol for free.

Still not solved here: Game-specific rendering content, strategy advice display,
and miner discovery integration are handled by the gameplay surface and
individual game crates.

## Scope

In scope:
- Five-panel layout: header, transcript, state, declaration, input
- Responsive layout tiers (narrow, compact, desktop)
- GameRenderer trait contract
- Event loop with async update channel
- Screen navigation state machine
- Interaction state tracking (Neutral, Loading, Empty, Partial, Error, Success)
- Pipe protocol I/O (stdin line reading, structured output)
- Input line editing with command history
- Theme-based color mapping for interaction states

Out of scope:
- Game-specific rendering implementations
- Blueprint loading and strategy advice
- Miner discovery and live query integration
- Chain interaction or network communication
- Smoke test orchestration

## Current State

The crate exists at crates/myosu-tui with approximately 3,200 lines of code. It
uses ratatui for terminal rendering and crossterm for input handling, with tokio
for async event bridging.

The five-panel layout arranges vertically: a 1-line header showing game and
context labels, a flexible-height transcript panel showing scrollable game
history (max 1000 lines), a variable-height state panel rendered by the
GameRenderer, a 1-line declaration panel showing centered status banners, and a
1-line input panel with a prompt character.

Three layout tiers adapt to terminal width: Narrow (below 80 columns) collapses
the state panel, Compact (80-120) stacks transcript above state vertically, and
Desktop (120+) places transcript beside state in a 46/54 horizontal split.

The event loop spawns a tokio task that selects across a configurable tick
interval, crossterm keyboard events, and an async update channel. UpdateEvent
variants include SolverAdvice, StateChanged, TrainingProgress, Status, and
Message. Background tasks send updates through the channel without blocking
the render loop.

Six interaction states (Neutral, Loading, Empty, Partial, Error, Success) drive
declaration banner text and color. Each state maps to a theme color: bright for
Neutral/Success, blue for Loading, amber for Empty/Partial, red for Error.

The screen state machine tracks eight screen types (Onboarding, Lobby, Game,
Stats, Coaching, History, Wallet, Spectate) with navigation history. Overlay
screens (Coaching, History) close on q or Escape. Screen transitions route
through numeric input on Onboarding and Lobby screens.

Pipe mode reads lines from stdin, skips empty lines, and returns None on EOF.
It provides no TUI rendering, operating purely through structured text output.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Five-panel layout | Shell with header/transcript/state/declaration/input | Reuse | Game-agnostic layout proven by two games |
| Responsive tiers | Narrow/Compact/Desktop based on terminal width | Reuse | Adapts to terminal size automatically |
| Event loop | Tokio select across tick, keyboard, update channel | Reuse | Bridges async and sync rendering |
| Screen navigation | ScreenManager with 8 screen types and history | Reuse | Extensible for future screens |
| Pipe protocol | PipeMode with stdin line reading | Reuse | Agent integration proven |
| Input handling | InputLine with edit keys and history | Reuse | Standard terminal input editing |

## Non-goals

- Providing game-specific rendering logic.
- Managing network connections or chain state.
- Implementing strategy advice or blueprint loading.
- Supporting graphical or web-based rendering.
- Handling game rules or action validation.

## Behaviors

On startup in TUI mode, the shell enters the alternate terminal screen, enables
raw mode, and begins the event loop at the configured tick rate.

Each render frame calls Shell::draw, which calculates the layout tier from
terminal width, allocates panel areas, and renders each panel in sequence. The
state panel delegates to GameRenderer::render_state with the allocated area and
buffer. The declaration panel shows either the renderer's declaration text (in
Neutral state) or an interaction state banner with color coding.

Keyboard input routes through the shell's handle_key method. Ctrl-C triggers
global quit. The ? key toggles a help overlay when the input line is empty.
Overlay screens intercept q and Escape for closing. All other keys route to the
input line, which handles standard editing (backspace, delete, arrow keys, home,
end) and maintains a command history navigable with up/down arrows.

Input submission routes through handle_submit, which dispatches based on the
current screen. On Game screens, input passes through the renderer's
parse_input for action recognition, clarify for ambiguous input guidance, or
falls through to an error response.

The event loop processes UpdateEvents from background tasks: SolverAdvice logs
action distributions to the transcript, StateChanged logs state transitions,
TrainingProgress logs iteration and exploitability, Status updates the
interaction state and declaration detail, and Message logs directly to the
transcript.

In pipe mode, the framework reads lines from stdin without rendering. Each line
is processed through the same parse_input/clarify pipeline, producing Action,
Clarify, Error, or Quit response types. Action responses advance game state and
trigger a new state output. Clarify and Error responses include the list of
legal actions.

The transcript panel maintains a VecDeque buffer capped at 1000 lines, showing
the most recent entries that fit the available height. In Narrow tier, only 3
lines are visible.

## Acceptance Criteria

- The five-panel layout renders correctly at terminal widths below 80, between
  80 and 120, and above 120 columns.
- Any type implementing GameRenderer can be rendered in the shell without
  modifications to the framework.
- The event loop processes keyboard events, tick events, and async update events
  without blocking.
- Interaction state changes update the declaration banner text and color
  immediately on the next render frame.
- Screen navigation correctly pushes and pops overlay screens, returning to the
  previous screen on close.
- Pipe mode produces structured text output without ANSI escape codes.
- Pipe mode responds to input with Action, Clarify, Error, or Quit responses
  that include legal action lists where appropriate.
- The transcript buffer does not exceed 1000 lines.
