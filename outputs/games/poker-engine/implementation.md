# `games:poker-engine` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Create `myosu-games-poker` Crate Skeleton**

This slice creates the greenfield `myosu-games-poker` crate, registers it in the workspace, and verifies that the pinned robopoker NLHE types compile with serde support enabled. No solver, wire, query, exploitability, or training-session logic was implemented yet; those remain in later approved slices.

## What Changed

### Workspace root: `Cargo.toml`

- Added `crates/myosu-games-poker` to the workspace `members` list so `cargo build -p myosu-games-poker` can resolve the package.
- Added workspace dependency entries for:
  - `rbp-nlhe`
  - `rbp-mccfr`

Both point at the reviewed robopoker git source and pinned rev `04716310143094ab41ec7172e6cea5a2a66744ef`.

### New crate: `crates/myosu-games-poker/Cargo.toml`

Created the new package with:

- `name = "myosu-games-poker"`
- `crate-type = ["lib"]`
- dependency on `myosu-games`
- dependency on `serde`
- dependency on `rbp-nlhe` with `features = ["serde"]`
- dependency on `rbp-mccfr` with `features = ["serde"]`

This is the minimal dependency surface required to prove the NLHE wire types are serde-ready before Slice 3.

### New crate root: `crates/myosu-games-poker/src/lib.rs`

Created a minimal public surface that:

- re-exports `GameConfig`, `GameType`, `StrategyQuery`, and `StrategyResponse` from `myosu-games`
- re-exports `Flagship`, `NlheInfo`, and `NlheEdge` from `rbp-nlhe`
- reserves the `Poker`, `PokerSolver`, and `TrainingSession` public type names so later slices can fill in their behavior without reshaping the crate root
- adds a compile-time serde guard for `NlheInfo` and `NlheEdge`
- adds two narrow smoke tests:
  - serde support is enabled for the NLHE types
  - `GameType::NlheHeadsUp` is reachable through the re-export path

### Lockfile: `Cargo.lock`

The new crate pulled the expected additional packages into the lockfile:

- `myosu-games-poker`
- `rbp-nlhe`
- `rbp-cards`
- `rbp-gameplay`
- `petgraph 0.6.5`
- `fixedbitset 0.4.2`

This lockfile update is part of the slice because the package did not previously exist in the workspace.

## Proof Commands Run for This Slice

```bash
CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo build -p myosu-games-poker --offline
CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-games-poker --offline
cargo tree -p myosu-games-poker -e features --offline
```

All three commands exited 0.

## Stage-Owned Artifacts Left Untouched

- `outputs/games/poker-engine/quality.md` was not authored here; it is owned by the Quality Gate stage.
- `outputs/games/poker-engine/promotion.md` was not authored here; it is owned by the Review stage.

## What Remains for Future Slices

| Slice | Description | Status |
|-------|-------------|--------|
| Slice 2 | `solver.rs`: `PokerSolver` wrapper + checkpoint format | Pending |
| Slice 3 | `wire.rs`: bincode roundtrip for `NlheInfo` and `NlheEdge` | Pending |
| Slice 4 | `query.rs`: `handle_query` bridge | Pending |
| Slice 5 | `exploit.rs`: local + remote exploitability | Pending |
| Slice 6 | `training.rs`: `TrainingSession` | Pending |
