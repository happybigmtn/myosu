# `games:multi-game` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Create `myosu-games-liars-dice` Crate Skeleton**

This slice implemented only the approved bootstrap surface for the multi-game lane:

- added `crates/myosu-games-liars-dice` as a new workspace member
- updated `Cargo.lock` for the new workspace package
- created `crates/myosu-games-liars-dice/Cargo.toml`
- created `crates/myosu-games-liars-dice/src/lib.rs`

No Slice 2 game-engine logic was started, and no spectator or cross-game scoring work was included.

## What Changed

### Workspace wiring

`Cargo.toml` now includes:

```toml
"crates/myosu-games-liars-dice",
```

This makes the new lane crate addressable via `cargo build -p myosu-games-liars-dice` and `cargo test -p myosu-games-liars-dice`.

`Cargo.lock` now records the new workspace package and its direct dependencies:

- `myosu-games`
- `rbp-core`
- `rbp-mccfr`

### New crate manifest

`crates/myosu-games-liars-dice/Cargo.toml` establishes the approved skeleton:

- workspace package metadata (`version`, `edition`, `license`)
- `[lib] crate-type = ["lib"]`
- `myosu-games` path dependency for the existing myosu game registry/types
- pinned `rbp-core` and `rbp-mccfr` git dependencies at `04716310143094ab41ec7172e6cea5a2a66744ef`, matching the reviewed lane inputs

### New crate public surface

`crates/myosu-games-liars-dice/src/lib.rs` currently exposes placeholder public API names for the later slices:

```rust
pub use stub::{
    LiarsDiceEdge, LiarsDiceEncoder, LiarsDiceGame, LiarsDiceInfo, LiarsDiceProfile,
    LiarsDiceTurn,
};

pub const GAME_TYPE: GameType = GameType::LiarsDice;
```

The placeholder structs are zero-sized stubs. This keeps the crate boundary and public names stable now, while leaving the actual `CfrGame`, `CfrEdge`, `CfrTurn`, `CfrInfo`, `Encoder`, and `Profile` implementations to Slice 2 and Slice 3.

Two smoke tests were added in `lib.rs`:

- `exposes_liars_dice_game_type`
- `stub_public_api_is_constructible`

These verify the crate is wired to the existing `GameType::LiarsDice` registry hook and that the initial public API builds cleanly.

## Slice Boundaries Preserved

- `crates/myosu-games/src/` was not modified
- `crates/myosu-games-poker/src/` was not modified
- no `game.rs`, `edge.rs`, `turn.rs`, `info.rs`, `encoder.rs`, or `profile.rs` files were added yet
- no `myosu-play` or `myosu-tui` spectator work was started
- `quality.md` was not hand-authored
- `promotion.md` was not authored

## Next Approved Slice

**Slice 2 — `game.rs` + `edge.rs` + `turn.rs` + `info.rs`: Liar's Dice Game Engine**

That slice owns the first real game state and trait implementation work. This Slice 1 change intentionally stops before that boundary.
