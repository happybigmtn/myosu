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

## What Changed

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
- `quality.md` and `promotion.md` were intentionally left untouched because the
  implementation lane does not own those artifacts.

## What Remains for Future Slices

| Slice | Description | Status |
|-------|-------------|--------|
| Slice 2 | Add `game.rs`, `edge.rs`, `turn.rs`, and `info.rs` for the Liar's Dice CFR game engine | Pending |
| Slice 3 | Add encoder/profile/solver behavior and Nash proof tests | Pending |
| Slice 4 | Add `ExploitMetric` registration in `myosu-games` | Pending |
| Slice 5 | Add `SpectatorRelay` in `myosu-play` | Pending |
| Slice 6 | Add spectator TUI rendering in `myosu-tui` | Pending |
| Slice 7 | Prove zero-change architecture across existing game crates | Pending |
