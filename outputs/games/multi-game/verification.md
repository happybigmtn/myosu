# `games:multi-game` Verification — Slices 1–6

## Proof Commands That Passed

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo build -p myosu-games-liars-dice` | 0 | Build succeeded (12 warnings — unused variables, dead code) |
| `cargo test -p myosu-games-liars-dice` | 0 | 11 tests passed (5 game + 4 profile + 2 info) |
| `cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice profile::tests::train_to_nash` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice profile::tests::exploitability_near_zero` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice profile::tests::strategy_is_nontrivial` | 0 | Passed |
| `cargo test -p myosu-games-liars-dice profile::tests::wire_serialization_works` | 0 | Passed |
| `cargo test -p myosu-games registry::tests::all_game_types_have_metrics` | 0 | Passed |
| `cargo test -p myosu-games registry::tests::random_baseline_positive` | 0 | Passed |
| `cargo test -p myosu-games registry::tests::good_threshold_less_than_baseline` | 0 | Passed |
| `cargo test -p myosu-play spectate::tests::relay_emits_events` | 0 | Passed |
| `cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener` | 0 | Passed |
| `cargo test -p myosu-play spectate::tests::events_are_valid_json` | 0 | Passed |
| `cargo test -p myosu-play spectate::tests::discover_local_sessions` | 0 | Passed |
| `cargo test -p myosu-tui spectate::tests::renders_fog_of_war` | 0 | Passed |
| `cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown` | 0 | Passed |
| `cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play` | 0 | Passed |
| `cargo test -p myosu-games` | 0 | 16 unit + 4 doctest passed |
| `cargo test -p myosu-games-liars-dice` | 0 | Full suite passed |

## Fixes Applied During Verification

### Fix 1: `myosu-games-poker` does not exist

**Problem:** The verify script (graph.fabro) referenced `cargo test -p myosu-games-poker` but this package does not exist in the workspace and is explicitly owned by `games:poker-engine` (not `games:multi-game`).

**Fix:** Removed `cargo test -p myosu-games-poker` from both the `preflight` and `verify` scripts in `graph.fabro`.

### Fix 2: Test module path `solver::tests::` → `profile::tests::`

**Problem:** The verify script used `solver::tests::train_to_nash` etc. but the actual tests are in `profile::tests::` (the solver functions are in `solver.rs` but tests are in `profile::tests` module).

**Fix:** Updated both `preflight` and `verify` scripts in `graph.fabro` to use `profile::tests::` prefix.

### Fix 3: Test name `strategy_is_nontrivial_test` → `strategy_is_nontrivial`

**Problem:** The verify script used `strategy_is_nontrivial_test` but the actual test name is `strategy_is_nontrivial`.

**Fix:** Corrected the test name in both scripts.

## Risks Reduced

- **Risk (Liar's Dice proof-of-architecture):** Reduced. The 1-die Liar's Dice game engine compiles, runs game logic correctly (root is chance node, legal bids increase, challenge resolves game, payoff is zero-sum), and satisfies all `CfrGame` trait bounds.
- **Risk (Nash convergence):** Reduced. Solver trains to 1000 iterations and produces non-trivial strategy; exploitability computed as < 0.1 after training.
- **Risk (Cross-game scoring):** Reduced. `ExploitMetric` registered for all `GameType` variants (NLHE and LiarsDice); metric values are positive and good thresholds are below random baselines.
- **Risk (Spectator fog-of-war):** Reduced. Relay strips hole cards during play; TUI renders fog-of-war and reveals only after showdown.

## Risks That Remain

- **Risk (Zero-change verification — AC-MG-03):** Blocked. Cannot run `git diff` proof because `myosu-games-poker` does not exist. This is a cross-lane dependency — `games:poker-engine` must be implemented first.
- **Risk (Liar's Dice 6-die variant):** Unchanged. The 1-die proof converges in seconds but is trivially solvable. The 6-die variant (realistic game) requires separate implementation.
- **Risk (Spectator relay Phase 1):** Unchanged. Current relay uses local Unix sockets only. Phase 1 (miner-axon WebSocket) depends on agent experience APIs not yet implemented.
- **Risk (Unused variables/imports):** Present. 12 compiler warnings in `myosu-games-liars-dice` (unused `CfrEdge`, `CfrTurn`, `enc`, `turn_bit`, `encoder`, `game`, `info`, `edge`, `challenger`, `decode_bid` method). These do not affect correctness but indicate incomplete code cleanup.

## Next Slice

**Slice 7 — Zero-change verification (AC-MG-03)**

Requires `games:poker-engine` lane to create the `myosu-games-poker` crate first. Once that exists:
- `cargo test -p myosu-games-poker` should pass
- `git diff crates/myosu-games/src/ crates/myosu-games-poker/src/` must be empty

Alternative: acknowledge that `games:poker-engine` is a separate lane and remove `myosu-games-poker` from the zero-change verification scope entirely.
