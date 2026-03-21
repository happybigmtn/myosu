# `games:multi-game` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Create `myosu-games-liars-dice` Crate Skeleton**

This lane is still at the approved Slice 1 boundary. The only code surface implemented for `games:multi-game` is the new `myosu-games-liars-dice` crate skeleton:

- added `crates/myosu-games-liars-dice` as a workspace member
- updated `Cargo.lock` for the new package
- created `crates/myosu-games-liars-dice/Cargo.toml`
- created `crates/myosu-games-liars-dice/src/lib.rs`

This fixup does not advance into Slice 2 or later. It keeps the code surface unchanged and refreshes the lane record so it matches the current workspace truth.

## Current Workspace Truth

`cargo metadata --no-deps --format-version 1` shows the workspace currently contains these relevant packages:

- `myosu-games`
- `myosu-games-liars-dice`
- `myosu-tui`
- `pallet-game-solver`

There is no `myosu-play` package and no `myosu-games-poker` package in this workspace snapshot. Those later review-script failures are therefore outside the code added by Slice 1.

## What Slice 1 Added

### Workspace wiring

The workspace manifest now includes:

```toml
"crates/myosu-games-liars-dice",
```

That makes the new lane crate addressable as `myosu-games-liars-dice` for Cargo build and test commands.

### New crate manifest

`crates/myosu-games-liars-dice/Cargo.toml` establishes the bootstrap crate contract:

- workspace package metadata (`version`, `edition`, `license`)
- `[lib] crate-type = ["lib"]`
- path dependency on `myosu-games`
- pinned `rbp-core` and `rbp-mccfr` git dependencies at `04716310143094ab41ec7172e6cea5a2a66744ef`

### New public API shell

`crates/myosu-games-liars-dice/src/lib.rs` exports the stable public names reserved for later slices:

```rust
pub use stub::{
    LiarsDiceEdge, LiarsDiceEncoder, LiarsDiceGame, LiarsDiceInfo, LiarsDiceProfile,
    LiarsDiceTurn,
};

pub const GAME_TYPE: GameType = GameType::LiarsDice;
```

The exported types are still zero-sized placeholders. That is intentional for Slice 1: the crate boundary and canonical names now exist, while the actual `CfrGame`, `CfrEdge`, `CfrTurn`, `CfrInfo`, `Encoder`, and `Profile` implementations remain deferred to the next approved slices.

Two smoke tests live in `lib.rs`:

- `exposes_liars_dice_game_type`
- `stub_public_api_is_constructible`

## Boundaries Preserved

- `crates/myosu-games/src/` was not modified by Slice 1
- no `game.rs`, `edge.rs`, `turn.rs`, `info.rs`, `encoder.rs`, or `profile.rs` files were added yet
- no `ExploitMetric` registration work was started
- no spectator relay or spectator TUI code was started
- no placeholder `myosu-play` or `myosu-games-poker` package was introduced just to satisfy later review commands
- `quality.md` was not authored in this fixup
- `promotion.md` was not touched in this fixup

## Next Approved Slice

**Slice 2 — `game.rs` + `edge.rs` + `turn.rs` + `info.rs`: Liar's Dice Game Engine**

That remains the next approved implementation increment from the reviewed lane artifacts.
