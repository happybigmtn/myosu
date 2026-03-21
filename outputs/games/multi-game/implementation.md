# `games:multi-game` Implementation — Slice 2

## Slice Implemented

**Slice 2 — `game.rs` + `edge.rs` + `turn.rs` + `info.rs`: Liar's Dice game
engine**

This implementation follows the next approved slice from
[`outputs/games/multi-game/spec.md`](./spec.md) and
[`outputs/games/multi-game/review.md`](./review.md). The work stays inside the
owned `myosu-games-liars-dice` surfaces and stops before encoder/profile,
registry, or spectator work.

## What Changed

### `crates/myosu-games-liars-dice/Cargo.toml`

- added a direct `rbp-transport` dependency at the same robopoker revision as
  `rbp-mccfr`
- kept the crate pinned to the existing workspace and robopoker revision

### `Cargo.lock`

- updated the `myosu-games-liars-dice` package entry to record the new direct
  `rbp-transport` dependency

### `crates/myosu-games-liars-dice/src/lib.rs`

- replaced the slice-1 public stubs for game/edge/turn/info with real module
  wiring and re-exports
- kept `LiarsDiceEncoder` and `LiarsDiceProfile` as placeholders so slice 2 does
  not drift into solver work

### `crates/myosu-games-liars-dice/src/edge.rs`

- added `LiarsDiceEdge` with the three approved transition classes:
  `Roll { player0, player1 }`, `Bid { quantity, face }`, and `Challenge`
- added game constants for `NUM_PLAYERS`, `NUM_FACES`, and `MAX_BIDS`
- encoded bids as an internal rank (`1..=12`) so the game state can store bid
  history in a fixed-size `[u8; MAX_BIDS]` array with `0` as the empty-slot
  sentinel
- exposed deterministic helpers for enumerating all chance rolls and all legal
  bid ranks

### `crates/myosu-games-liars-dice/src/turn.rs`

- added `LiarsDiceTurn::{Chance, Player0, Player1, Terminal}`
- implemented `From<usize>` and `CfrTurn`
- added a small `actor_index()` helper so game logic and info projection can map
  player turns back to `0` or `1`

### `crates/myosu-games-liars-dice/src/info.rs`

- added `LiarsDiceInfo` plus the supporting public/private info components
- `LiarsDiceInfo::from_game(&LiarsDiceGame)` now projects the acting player's
  hidden die while preserving the public bid path
- chance nodes expose all 36 die-roll outcomes
- player nodes expose only strictly increasing bids, with `Challenge` appended
  only after at least one bid exists
- terminal nodes expose no outgoing choices

### `crates/myosu-games-liars-dice/src/game.rs`

- added a compact, `Copy`-safe `LiarsDiceGame` state:
  `dice: [u8; 2]`, `bids: [u8; MAX_BIDS]`, `bid_count`, `turn`, `winner`
- implemented `CfrGame` with:
  - `root()` as a chance node
  - `apply()` transitions for roll, bid, and challenge
  - terminal winner resolution based on whether the last bid is supported by the
    revealed dice
  - zero-sum `payoff()` returning `+1.0` to the winner and `-1.0` to the loser
- added the slice-2 proof tests directly under `game::tests`

## Slice-2 Proof Behavior

The implementation now satisfies the review-approved slice-2 checks:

- root is a chance node with 36 equiprobable roll branches
- legal bids increase monotonically through the fixed-size bid ladder
- challenge resolves immediately to a terminal winner
- terminal utilities are zero-sum
- all required robopoker trait bounds compile against the Liar's Dice types

## Scope Guardrails Preserved

- no encoder or profile behavior was implemented
- no `myosu-games` registry work was started
- no spectator relay or TUI work was started
- no edits were made under `crates/myosu-games/src/`
- no poker-engine surfaces were touched

## Remaining Approved Work

The next approved slice is still **Slice 3 — `encoder.rs` + `profile.rs`** for
solver integration and Nash verification. Slice 2 intentionally leaves those
surfaces as placeholders.
