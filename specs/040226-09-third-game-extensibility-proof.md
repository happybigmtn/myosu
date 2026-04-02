# Specification: Third Game Extensibility Proof

Source: Genesis Plan 011 (Third Game Integration), ASSESSMENT.md future-fit strengths
Status: Draft
Depends-on: 006-multi-node-devnet

## Purpose

The multi-game architecture has been proven additive with two games: No-Limit
Hold'em (poker) and Liar's Dice. Adding a third structurally different game
validates that the game trait abstraction, TUI rendering interface, wire
protocol pattern, and play binary game selection genuinely generalize — rather
than being coincidentally compatible with two similar card/dice games. A third
game that integrates without modifying existing game crates provides confidence
that the architecture supports an open-ended game ecosystem.

## Whole-System Goal

Current state: Two game crates exist — `myosu-games-poker` (9.4K lines) and
`myosu-games-liars-dice` (2.1K lines). Both implement the game traits from
`myosu-games/src/traits.rs`, provide renderers implementing the `GameRenderer`
trait, and define wire protocols with versioned magic bytes. The `GameType` enum
includes `NlheHeadsUp`, `NlheSixMax`, `LiarsDice`, and `Custom(String)`.
`myosu-play` selects games at startup. Liar's Dice was added without modifying
poker code, proving the architecture once.

This spec adds: A third game crate that integrates into the full stack (traits,
renderer, wire protocol, play binary, CI) without any modifications to the
poker or Liar's Dice crates.

If all ACs land: Three games run in the platform, the third was added with zero
changes to existing game crates, and the game trait abstraction is validated as
genuinely extensible.

Still not solved here: Game marketplace or discovery, third-party game developer
SDK, cross-game scoring normalization, and solver quality comparison across
games.

## Scope

In scope:
- A new game crate implementing the game traits from `myosu-games`
- A TUI renderer implementing the `GameRenderer` trait
- A wire protocol with versioned magic bytes
- Integration into `myosu-play` game selection
- CI test coverage for the new game

Out of scope:
- MCCFR solver training for the new game (exact solutions are acceptable for
  small games)
- Miner/validator integration for the new game subnet
- Cross-game scoring or economic design
- Game developer SDK or documentation
- Modifying the `GameType` enum to use `Custom(String)` — a named variant is
  preferred

## Current State

The game trait system at `crates/myosu-games/src/traits.rs` defines the
interfaces that game crates implement. The `GameType` enum at
`crates/myosu-games/src/lib.rs` enumerates known games. The `GameRenderer` trait
at `crates/myosu-tui/src/renderer.rs` defines the TUI rendering interface.

`myosu-games-liars-dice` was the second game added and serves as the primary
reference for how to integrate a new game. It includes: game state machine,
renderer, solver, and wire protocol — the same module structure needed for a
third game.

The research corpus (Python stack) evaluates 15+ game-solving approaches across
a 20-game corpus. Kuhn Poker (12 information sets, exact solution) and Leduc
Poker are the smallest candidates from this corpus and have well-understood
equilibria for validation.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Game traits | `crates/myosu-games/src/traits.rs` | Reuse | Third game implements these |
| GameType enum | `crates/myosu-games/src/lib.rs` | Extend | Add new variant |
| GameRenderer trait | `crates/myosu-tui/src/renderer.rs` | Reuse | Third game implements this |
| Wire protocol pattern | `crates/myosu-games-liars-dice/src/wire.rs` | Reference | Follow same versioned magic byte pattern |
| Game selection | `crates/myosu-play/src/main.rs` | Extend | Add third game to selection |
| Second game reference | `crates/myosu-games-liars-dice/` | Reference | Template for third game structure |
| CI active-crates job | `.github/workflows/ci.yml` | Extend | Add third game crate |

## Non-goals

- Producing a research-grade solver for the third game.
- Adding the third game to the economic incentive system (subnet, emissions).
- Supporting multiplayer or networked play for the third game.
- Benchmarking the third game's solver performance.
- Creating a game developer SDK from the third game experience.

## Behaviors

A new game crate provides a complete game state machine: game initialization,
legal move generation, state transitions, and terminal state detection. The
state machine implements the game traits defined in `myosu-games`.

The game crate provides a TUI renderer that implements the `GameRenderer` trait,
enabling the game to display in the existing five-panel TUI shell. The renderer
shows game state, available actions, and results using the same theme and layout
conventions as existing games.

The game crate provides a wire protocol with versioned magic bytes following
the same pattern as poker and Liar's Dice. The wire format round-trips correctly
through encode/decode.

The play binary includes the third game in its game selection. An operator can
start `myosu-play` with the third game and play a complete session. The smoke
test mode works for the third game.

No modifications are made to `myosu-games-poker` or `myosu-games-liars-dice`
source code as part of adding the third game. Changes are limited to
`myosu-games` (GameType enum variant), `myosu-play` (game selection), and the
new game crate itself.

## Acceptance Criteria

- A third game crate exists with a complete state machine, renderer, and wire
  protocol.
- The third game crate has passing unit tests including wire round-trip tests.
- Zero source code modifications to `myosu-games-poker` or
  `myosu-games-liars-dice`.
- `myosu-play` can start and play a complete session with the third game.
- The third game smoke test passes in CI.
