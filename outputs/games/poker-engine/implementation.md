# Slice 1: Create myosu-games-poker Crate Skeleton — Implementation

## Slice Status

**COMPLETED**

## Touched Files / Modules

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Added `crates/myosu-games-poker` to workspace members |
| `crates/myosu-games-poker/Cargo.toml` | New file — crate definition with robopoker git deps |
| `crates/myosu-games-poker/README.md` | New file — crate documentation |
| `crates/myosu-games-poker/src/lib.rs` | New file — public API re-exports |
| `crates/myosu-games-poker/src/solver.rs` | New file — `PokerSolver` stub with `Flagship` type alias |
| `crates/myosu-games-poker/src/query.rs` | New file — `handle_query` stub |
| `crates/myosu-games-poker/src/wire.rs` | New file — `WireStrategy`, `NlheInfoCodec`, `NlheEdgeCodec` stubs |
| `crates/myosu-games-poker/src/exploit.rs` | New file — `poker_exploitability`, `remote_poker_exploitability` stubs |
| `crates/myosu-games-poker/src/training.rs` | New file — `TrainingSession` stub |

## Setup Steps Completed

1. **Workspace membership**: Added `crates/myosu-games-poker` to `members` in root `Cargo.toml`
2. **Robopoker git deps**: Pinned `rbp-core`, `rbp-mccfr`, `rbp-nlhe` at same rev as `myosu-games` (`04716310143094ab41ec7172e6cea5a2a66744ef`)
3. **serde feature verification**: Confirmed `serde` feature is needed on `rbp-mccfr` and `rbp-nlhe` (not on `rbp-core` which has no features)
4. **Dependency on myosu-games**: Added path dependency on `../myosu-games`
5. **crate-type = ["lib"]**: Set library crate type

## Setup Steps Deferred

- **None** — all Slice 1 setup steps completed

## Blocks for Next Slice

**None that are blocking.** Slice 2 (`solver.rs` full implementation) can proceed immediately.

## Notes

- `PokerSolver::new()` creates a `rbp_nlhe::Flagship` with default `NlheProfile` and `NlheEncoder`
- All public API surface is defined but body implementations are `unimplemented!("Slice N")` stubs
- `serde` feature on robopoker crates is confirmed working — `cargo build` with `features = ["serde"]` on `rbp-mccfr` and `rbp-nlhe` compiles successfully
- The `rbp-core` crate does NOT have a `serde` feature — error "package depends on rbp-core with feature serde but rbp-core does not have that feature" was fixed by removing the feature from `rbp-core` while keeping it on `rbp-mccfr` and `rbp-nlhe`
