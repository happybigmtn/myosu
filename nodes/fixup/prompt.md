Goal: Implement the next approved `games:multi-game` slice.

Inputs:
- `multi-game/spec.md`
- `multi-game/review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `games:multi-game`

Required curated artifacts:
- `multi-game/implementation.md`
- `multi-game/verification.md`
- `multi-game/quality.md`
- `multi-game/promotion.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
cargo build -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node
cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase
cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game
cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum
cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied
cargo test -p myosu-games-liars-dice solver::tests::train_to_nash
cargo test -p myosu-games-liars-dice solver::tests::exploitability_near_zero
cargo test -p myosu-games-liars-dice solver::tests::strategy_is_nontrivial
cargo test -p myosu-games-liars-dice solver::tests::wire_serialization_works
cargo test -p myosu-games registry::tests::all_game_types_have_metrics
cargo test -p myosu-games registry::tests::random_baseline_positive
cargo test -p myosu-games registry::tests::good_threshold_less_than_baseline
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener
cargo test -p myosu-play spectate::tests::events_are_valid_json
cargo test -p myosu-play spectate::tests::discover_local_sessions
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown
cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play
cargo test -p myosu-games
cargo test -p myosu-games-poker
cargo test -p myosu-games-liars-dice
true`
  - Stdout:
    ```
    (29 lines omitted)
    
    running 10 tests
    test traits::tests::game_type_from_bytes_custom ... ok
    test traits::tests::game_config_nlhe_params ... ok
    test traits::tests::game_type_from_bytes_known ... ok
    test traits::tests::game_config_serializes ... ok
    test traits::tests::game_type_num_players ... ok
    test traits::tests::reexports_compile ... ok
    test traits::tests::game_type_to_bytes_roundtrip ... ok
    test traits::tests::strategy_response_probability_for ... ok
    test traits::tests::strategy_query_response_roundtrip ... ok
    test traits::tests::strategy_response_validates ... ok
    
    test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 4 tests
    test crates/myosu-games/src/../README.md - (line 20) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::to_bytes (line 99) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::num_players (line 118) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::from_bytes (line 75) ... ok
    
    test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    all doctests ran in 0.13s; merged doctests compilation took 0.13s
    ```
  - Stderr:
    ```
    (73 lines omitted)
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.50s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.13s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
       Compiling myosu-games v0.1.0 (/home/r/.fabro/runs/20260320-01KM5QGR535ZE65FB54MPYF81P/worktree/crates/myosu-games)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.06s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games-2a3f1549b9da37a9)
       Doc-tests myosu_games
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-liars-dice` did not match any packages
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 51 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-games-liars-dice/Cargo.toml, crates/myosu-games-liars-dice/README.md, crates/myosu-games-liars-dice/src/edge.rs, crates/myosu-games-liars-dice/src/encoder.rs, crates/myosu-games-liars-dice/src/game.rs, crates/myosu-games-liars-dice/src/info.rs, crates/myosu-games-liars-dice/src/lib.rs, crates/myosu-games-liars-dice/src/profile.rs, crates/myosu-games-liars-dice/src/turn.rs, outputs/games/multi-game/implementation.md, outputs/games/multi-game/verification.md
- **verify**: fail
  - Script: `set -e
cargo build -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node
cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase
cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game
cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum
cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied
cargo test -p myosu-games-liars-dice solver::tests::train_to_nash
cargo test -p myosu-games-liars-dice solver::tests::exploitability_near_zero
cargo test -p myosu-games-liars-dice solver::tests::strategy_is_nontrivial
cargo test -p myosu-games-liars-dice solver::tests::wire_serialization_works
cargo test -p myosu-games registry::tests::all_game_types_have_metrics
cargo test -p myosu-games registry::tests::random_baseline_positive
cargo test -p myosu-games registry::tests::good_threshold_less_than_baseline
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener
cargo test -p myosu-play spectate::tests::events_are_valid_json
cargo test -p myosu-play spectate::tests::discover_local_sessions
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown
cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play
cargo test -p myosu-games
cargo test -p myosu-games-poker
cargo test -p myosu-games-liars-dice`
  - Stdout:
    ```
    (43 lines omitted)
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
    
    
    running 0 tests
    
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (30 lines omitted)
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_liars_dice-c099a45eb7ab1e06)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.49s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_liars_dice-c099a45eb7ab1e06)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.46s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_liars_dice-c099a45eb7ab1e06)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.45s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games-2a3f1549b9da37a9)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.45s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games-2a3f1549b9da37a9)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.45s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games-2a3f1549b9da37a9)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
    ```

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n> ## stdout running <n> tests test result: ok. <n> passed; <n> failed; <n> ignored; <n> measured; <n> filtered out; finished in <n>.00s running <n> tests test result: ok. <n> passed; <n> failed; <n> ignored; 


# Multi-Game Implementation Lane — Fixup

Fix only the current slice for `multi-game-implement`.


Current slice
- Slice 1: Create myosu-games-liars-dice Crate Skeleton

Touched surfaces
- ``crates/myosu-games-liars-dice/Cargo.toml``

Setup checks
- Add crate to workspace members. Cargo.toml:
- Dependency on `rbp-mccfr` (git) at same rev as `myosu-games` (`04716310143094ab41ec7172e6cea5a2a66744ef`)
- Dependency on `myosu-games`
- `crate-type = ["lib"]`
- lib.rs re-exports stub types initially.

First proof gate
- ``cargo build -p myosu-games-liars-dice``

Execution guidance
- Scope: **Spectator relay is Phase 0 only**: The local Unix socket relay is for single-machine use. Phase 1 (miner-axon WebSocket) requires agent experience APIs (AX-01..06). The implementation lane should implement Phase 0 only and leave Phase 1 markers in the code.
- Start: The specification is stable. The implementation lane can begin with Slice 1 (crate skeleton) immediately. The `GameType::LiarsDice` and `GameParams::LiarsDice` already exist in `myosu-games`, meaning the traits hook is prepared. No further spec work is needed before the implementation lane starts.
- Order: Slice 2 (game engine) must succeed before Slice 3 (solver) and Slice 7 (zero-change verification) can run
- Order: Slice 4 (ExploitMetric) must succeed before cross-game scoring can be integrated with the validator oracle
- Order: Slice 5 (spectator relay) must succeed before Slice 6 (spectator TUI)

Implementation artifact must cover
- state whether Slice 1: Create myosu-games-liars-dice Crate Skeleton was completed or partially completed
- list the touched files/modules for this slice
- note which setup steps were completed, deferred, or intentionally skipped
- call out anything that still blocks the next slice from starting

Verification artifact must cover
- record whether `cargo build -p myosu-games-liars-dice` passed and what it proved
- summarize the automated proof commands that ran and their outcomes
- say whether the slice is complete enough to move to the next ordered slice

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Settle stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
