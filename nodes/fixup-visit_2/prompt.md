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
    test traits::tests::game_config_nlhe_params ... ok
    test traits::tests::game_config_serializes ... ok
    test traits::tests::game_type_from_bytes_custom ... ok
    test traits::tests::game_type_from_bytes_known ... ok
    test traits::tests::game_type_num_players ... ok
    test traits::tests::game_type_to_bytes_roundtrip ... ok
    test traits::tests::reexports_compile ... ok
    test traits::tests::strategy_response_probability_for ... ok
    test traits::tests::strategy_query_response_roundtrip ... ok
    test traits::tests::strategy_response_validates ... ok
    
    test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 4 tests
    test crates/myosu-games/src/traits.rs - traits::GameType::from_bytes (line 75) ... ok
    test crates/myosu-games/src/../README.md - (line 20) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::to_bytes (line 99) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::num_players (line 118) ... ok
    
    test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    all doctests ran in 0.16s; merged doctests compilation took 0.16s
    ```
  - Stderr:
    ```
    (74 lines omitted)
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.03s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.07s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.00s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_games-2a3f1549b9da37a9)
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
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 65 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-games-liars-dice/Cargo.toml, crates/myosu-games-liars-dice/src/edge.rs, crates/myosu-games-liars-dice/src/encoder.rs, crates/myosu-games-liars-dice/src/game.rs, crates/myosu-games-liars-dice/src/info.rs, crates/myosu-games-liars-dice/src/lib.rs, crates/myosu-games-liars-dice/src/profile.rs, crates/myosu-games-liars-dice/src/solver.rs, crates/myosu-games-liars-dice/src/turn.rs, crates/myosu-games/src/lib.rs, crates/myosu-games/src/registry.rs, crates/myosu-play/Cargo.toml, crates/myosu-play/src/lib.rs, crates/myosu-play/src/spectate.rs, crates/myosu-tui/src/lib.rs, crates/myosu-tui/src/screens.rs, crates/myosu-tui/src/screens/spectate.rs
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
    (136 lines omitted)
    test registry::tests::weight_max_for_nash_strategy ... ok
    test registry::tests::weight_zero_for_random_strategy ... ok
    test traits::tests::game_config_nlhe_params ... ok
    test traits::tests::game_type_from_bytes_custom ... ok
    test traits::tests::game_config_serializes ... ok
    test traits::tests::game_type_from_bytes_known ... ok
    test traits::tests::game_type_num_players ... ok
    test traits::tests::game_type_to_bytes_roundtrip ... ok
    test traits::tests::reexports_compile ... ok
    test traits::tests::strategy_response_probability_for ... ok
    test traits::tests::strategy_query_response_roundtrip ... ok
    test traits::tests::strategy_response_validates ... ok
    
    test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 4 tests
    test crates/myosu-games/src/../README.md - (line 20) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::from_bytes (line 75) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::num_players (line 118) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::to_bytes (line 99) ... ok
    
    test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    all doctests ran in 0.19s; merged doctests compilation took 0.18s
    ```
  - Stderr:
    ```
    (1169 lines omitted)
        Blocking waiting for file lock on package cache
       Compiling myosu-tui v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J5CFFW4JCF5YSNBV96JGK/worktree/crates/myosu-tui)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.97s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on artifact directory
       Compiling myosu-tui v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J5CFFW4JCF5YSNBV96JGK/worktree/crates/myosu-tui)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 2.91s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on artifact directory
       Compiling myosu-tui v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J5CFFW4JCF5YSNBV96JGK/worktree/crates/myosu-tui)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 2.79s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on artifact directory
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.44s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_games-2a3f1549b9da37a9)
       Doc-tests myosu_games
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
    ```
- **fixup**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 10 out
  - Files: outputs/games/multi-game/implementation.md, outputs/games/multi-game/verification.md
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
    (136 lines omitted)
    test registry::tests::weight_max_for_nash_strategy ... ok
    test registry::tests::weight_zero_for_random_strategy ... ok
    test traits::tests::game_config_nlhe_params ... ok
    test traits::tests::game_type_from_bytes_custom ... ok
    test traits::tests::game_config_serializes ... ok
    test traits::tests::game_type_from_bytes_known ... ok
    test traits::tests::game_type_num_players ... ok
    test traits::tests::game_type_to_bytes_roundtrip ... ok
    test traits::tests::reexports_compile ... ok
    test traits::tests::strategy_response_probability_for ... ok
    test traits::tests::strategy_query_response_roundtrip ... ok
    test traits::tests::strategy_response_validates ... ok
    
    test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    
    running 4 tests
    test crates/myosu-games/src/../README.md - (line 20) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::from_bytes (line 75) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::num_players (line 118) ... ok
    test crates/myosu-games/src/traits.rs - traits::GameType::to_bytes (line 99) ... ok
    
    test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    
    all doctests ran in 0.19s; merged doctests compilation took 0.18s
    ```
  - Stderr:
    ```
    (1169 lines omitted)
        Blocking waiting for file lock on package cache
       Compiling myosu-tui v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J5CFFW4JCF5YSNBV96JGK/worktree/crates/myosu-tui)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.97s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on artifact directory
       Compiling myosu-tui v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J5CFFW4JCF5YSNBV96JGK/worktree/crates/myosu-tui)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 2.91s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on artifact directory
       Compiling myosu-tui v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J5CFFW4JCF5YSNBV96JGK/worktree/crates/myosu-tui)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 2.79s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_tui-3fecc5329b048298)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on artifact directory
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.44s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_games-2a3f1549b9da37a9)
       Doc-tests myosu_games
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
    ```

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n> ## stdout running <n> tests test game::tests::all_trait_bounds_satisfied ... ok test game::tests::challenge_resolves_game ... ok test game::tests::payoff_is_zero_sum ... ok test game::tests::legal_bids_incr


# Multi-Game Implementation Lane — Fixup

Fix only the current slice for `multi-game-implement`.

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Settle stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
