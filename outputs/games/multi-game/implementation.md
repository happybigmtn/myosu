# `games:multi-game` Implementation — Slice 2

## Slice Implemented

**Slice 2 — `game.rs` + `edge.rs` + `turn.rs` + `info.rs`: Liar's Dice Game Engine**

This implementation advances the lane from the Slice 1 crate skeleton to the approved MG-01 game-engine slice. The work stayed inside the `myosu-games-liars-dice` crate and the lane-owned implementation/verification artifacts.

## Code Added

### Real game-engine modules

The crate now exports concrete Slice 2 types instead of stubs for the game-engine surface:

- `crates/myosu-games-liars-dice/src/edge.rs`
- `crates/myosu-games-liars-dice/src/turn.rs`
- `crates/myosu-games-liars-dice/src/info.rs`
- `crates/myosu-games-liars-dice/src/game.rs`

`src/lib.rs` now re-exports:

- `LiarsDiceGame`
- `LiarsDiceEdge`
- `LiarsDiceTurn`
- `LiarsDiceInfo`

`LiarsDiceEncoder` and `LiarsDiceProfile` remain placeholders on purpose. They are reserved for Slice 3 and were not implemented here.

### Chance root represented explicitly

The reviewed AC requires the root state to be a chance node and to become Player 0's turn after the dice are dealt. To realize that under robopoker's `CfrGame::apply(edge)` contract, `LiarsDiceEdge` now includes:

- `Roll { p0, p1 }` for the 36 root chance outcomes
- `Bid { quantity, face }`
- `Challenge`

This keeps the player decision surface faithful to the spec while making the reviewed "chance root" behavior representable in the tree.

### `Copy`-safe bid history

The reviewed blocker around `CfrGame: Copy` is handled with a fixed-size sentinel-backed bid history:

- internal `BidHistory { bids: [u8; 12], len: u8 }`
- unused slots stay at `u8::MAX`
- stored values are compact bid ranks `0..11`

That keeps both `LiarsDiceGame` and `LiarsDiceInfo` `Copy` without using `Vec`.

### Game state and info-set behavior

`LiarsDiceGame` now models:

- root chance state with undealt dice
- dealt state with hidden dice and alternating player turns
- strictly increasing bid order across the 12 ordered claims
- challenge resolution into a terminal zero-sum payoff

`LiarsDiceInfo` now carries:

- public turn marker
- public bid history
- the acting player's private die face

`LiarsDicePublic::choices()` drives the tree shape:

- 36 `Roll` edges at the chance root
- legal higher bids at player nodes
- `Challenge` only after at least one bid
- no choices at terminal nodes

## Tests Added

Slice 2 added the MG-01 tests requested by review:

- `game::tests::root_is_chance_node`
- `game::tests::legal_bids_increase`
- `game::tests::challenge_resolves_game`
- `game::tests::payoff_is_zero_sum`
- `game::tests::all_trait_bounds_satisfied`

The crate-level smoke tests from Slice 1 still remain:

- `tests::exposes_liars_dice_game_type`
- `tests::public_api_is_constructible`

## Surfaces Touched

- `Cargo.lock`
- `crates/myosu-games-liars-dice/Cargo.toml`
- `crates/myosu-games-liars-dice/src/lib.rs`
- `crates/myosu-games-liars-dice/src/edge.rs`
- `crates/myosu-games-liars-dice/src/turn.rs`
- `crates/myosu-games-liars-dice/src/info.rs`
- `crates/myosu-games-liars-dice/src/game.rs`

`Cargo.lock` changed only because Slice 2 added a direct `rbp-transport` dependency so the crate can implement robopoker's `Support` trait for its edge, turn, and secret types.

## Boundaries Preserved

- `crates/myosu-games/src/` was not modified
- no `ExploitMetric` work was started
- no spectator relay or spectator TUI work was started
- no `myosu-play` or `myosu-games-poker` placeholder package was created
- `quality.md` was not authored here
- `promotion.md` was not touched here

## Next Approved Slice

**Slice 3 — `encoder.rs`, `profile.rs`, and solver/Nash verification**

That remains the next approved increment after this implementation.
