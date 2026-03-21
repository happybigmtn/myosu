# `games:multi-game` Verification — Slice 2

## Verification Scope

This verification covers the Slice 1 + Slice 2 proof surface that belongs to the newly implemented Liar's Dice game engine. It does not claim Slice 3 through Slice 7 are complete.

Commands were run with `CARGO_TARGET_DIR=/tmp/myosu-mg-slice2` because the workspace metadata still points Cargo at a target directory outside this run's writable sandbox.

## Automated Commands Run

| Command | Exit Code | Outcome |
|---------|-----------|---------|
| `cargo build -p myosu-games-liars-dice` | 0 | Passed; the Slice 2 crate compiles with the real game-engine modules |
| `cargo test -p myosu-games-liars-dice` | 0 | Passed; 7 unit tests passed and 0 doctests ran |
| `cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied` | 0 | Passed |
| `cargo test -p myosu-games` | 0 | Passed; existing `myosu-games` unit tests and doctests still pass unchanged |

## Slice 2 Proof Status

The approved MG-01 proof gate is green:

- root state is a chance node
- applying a root roll transitions to Player 0
- legal bids strictly increase
- challenge resolution handles both truthful bids and bluffs
- terminal payoffs are zero-sum
- `LiarsDiceGame` satisfies the required trait bounds, including `Copy`

## Current Test Totals Observed

- `myosu-games-liars-dice`: 7 unit tests passed
- `myosu-games`: 10 unit tests passed, 4 doctests passed

## Known Remaining Lane Blockers

The full review command list was not rerun in this slice because its first deterministic failure remains outside the Slice 2 surface:

- `myosu-play` is still absent from the current workspace snapshot
- `myosu-games-poker` is still absent from the current workspace snapshot

Those package-not-found failures were already documented in the prior fixup and are unchanged by this Slice 2 implementation.

## Not Yet Verified

- Slice 3 solver/Nash tests
- Slice 4 `ExploitMetric` registration
- Slice 5 spectator relay tests
- Slice 6 spectator TUI tests
- Slice 7 zero-change verification against `myosu-games-poker`
