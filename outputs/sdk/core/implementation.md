# `sdk:core` Implementation

## Lane
`core-implement`

## Slice
Slice 2 — Trait Compliance Test Harness (`AC-SDK-03`)

## Goal
Replace the initial `testing` helpers with real compliance checks, keep the work contained to the `sdk:core` test-harness surfaces, and make the reviewed Slice 2 proof commands exercise real tests.

## Touched Surfaces
- `crates/myosu-sdk/Cargo.toml`
- `crates/myosu-sdk/src/testing/mod.rs`
- `crates/myosu-sdk/src/testing/game_valid.rs`
- `crates/myosu-sdk/src/testing/convergence.rs`
- `crates/myosu-sdk/src/testing/tests.rs`

## What Changed
- Added a direct `rbp-mccfr` dependency to `myosu-sdk` so the public convergence helper can use the upstream `Solver` trait without reaching through tests-only dependencies.
- Reworked `testing::assert_game_valid` into a real tree walk for games whose `turn()` also serves as the reachable info-set view. The helper now checks:
  - root is not terminal
  - non-terminal states expose at least one action
  - terminal states expose no actions
  - `apply()` changes state
  - the explored tree is acyclic within a path
  - terminal utilities are finite
  - two-player terminal payoffs sum to zero
  - repeated info sets expose the same action set
- Replaced the earlier no-op convergence helper with a real `assert_solver_converges` helper that runs `Solver::step()` for the requested number of steps and fails with the measured exploitability when the target is missed.
- Flattened the Slice 2 unit tests so the reviewed command filters now hit real tests at `testing::tests::...` instead of silently matching zero tests.
- Added two concrete proof fixtures:
  - a deliberately broken RPS wrapper whose terminal payoffs are non-zero-sum
  - an undertrained RPS solver that misses the exploitability target

## Notes
- This pass stayed inside the Slice 2 harness boundary. Scaffold, registration, and docs surfaces were left untouched.
- The current helper shape is anchored to the trusted robopoker trait surface available in this repo today. In practice that means the generic game-validity helper works for games whose reachable `turn()` value is also a usable `CfrInfo` view, which covers the reviewed RPS proof target for this slice.
