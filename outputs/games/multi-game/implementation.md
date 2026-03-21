# `games:multi-game` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Create `myosu-games-liars-dice` Crate Skeleton**

Implemented the smallest approved `games:multi-game` slice from
[`outputs/games/multi-game/spec.md`](./spec.md) and
[`outputs/games/multi-game/review.md`](./review.md):

- added `crates/myosu-games-liars-dice` as a new workspace member
- created a minimal crate manifest pinned to the same robopoker git revision as `myosu-games`
- created a narrow `src/lib.rs` that exposes the approved public Liar's Dice type names as placeholders for later slices

This slice intentionally stops at crate wiring. It does not implement the game
engine, solver, scoring registry, or spectator surfaces.

## Fixup Boundary

This fixup keeps the lane on the same approved slice. No slice-2 game-engine
work was started.

The package inventory in this checkout, confirmed via `cargo metadata --no-deps
--format-version 1`, is currently:

- `myosu-games`
- `myosu-games-liars-dice`
- `myosu-tui`
- `pallet-game-solver`

That means the verify-stage failure was not a slice-1 defect in
`myosu-games-liars-dice`. It came from proof orchestration continuing into
later or absent surfaces such as `myosu-play`. This fixup preserves the slice-1
boundary instead of manufacturing later-slice crates just to satisfy an
out-of-scope command.

## What Changed

No source changes were needed during fixup. The slice-1 code remains:

### `Cargo.toml`

Added the new workspace member:

```toml
members = [
    "crates/myosu-games",
    "crates/myosu-games-liars-dice",
    "crates/myosu-tui",
    "crates/myosu-chain/pallets/game-solver",
]
```

### `Cargo.lock`

Recorded the new workspace package entry:

```toml
[[package]]
name = "myosu-games-liars-dice"
version = "0.1.0"
dependencies = [
  "myosu-games",
  "rbp-mccfr",
]
```

### `crates/myosu-games-liars-dice/Cargo.toml`

Created the new package manifest with the approved dependency shape:

```toml
[package]
name = "myosu-games-liars-dice"
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["lib"]

[dependencies]
myosu-games = { path = "../myosu-games" }
rbp-mccfr = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }
```

### `crates/myosu-games-liars-dice/src/lib.rs`

Added placeholder public API types for the later Liar's Dice implementation:

- `LiarsDiceGame`
- `LiarsDiceEdge`
- `LiarsDiceTurn`
- `LiarsDiceInfo`
- `LiarsDiceEncoder`
- `LiarsDiceProfile`

All six are zero-sized placeholder structs with basic derives (`Clone`, `Copy`,
`Debug`, `Default`, `PartialEq`, `Eq`) so the crate exposes a stable surface
without prematurely committing to slice-2 game-state details.

A single smoke test, `tests::public_api_stubs_exist`, verifies the new crate can
be compiled and tested in isolation.

## Scope Guardrails Preserved

- No edits were made to `crates/myosu-games/src/`.
- No poker-engine surfaces were touched.
- No spectator or scoring work was started.
- This fixup does not author `quality.md` or `promotion.md`; those artifacts
  remain owned by the Quality and Review stages, and the fixup instructions
  explicitly forbid creating or rewriting them here.

## What Remains for Future Slices

| Slice | Description | Status |
|-------|-------------|--------|
| Slice 2 | Add `game.rs`, `edge.rs`, `turn.rs`, and `info.rs` for the Liar's Dice CFR game engine | Pending |
| Slice 3 | Add encoder/profile/solver behavior and Nash proof tests | Pending |
| Slice 4 | Add `ExploitMetric` registration in `myosu-games` | Pending |
| Slice 5 | Add `SpectatorRelay` in `myosu-play` | Pending |
| Slice 6 | Add spectator TUI rendering in `myosu-tui` | Pending |
| Slice 7 | Prove zero-change architecture across existing game crates | Pending |
