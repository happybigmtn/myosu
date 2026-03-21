#!/usr/bin/env bash
set -euo pipefail

test -f outputs/games/multi-game/spec.md
test -f outputs/games/multi-game/review.md

cargo build -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node
cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase
cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game
cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum
cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied
cargo test -p myosu-games-liars-dice profile::tests::train_to_nash
cargo test -p myosu-games-liars-dice profile::tests::exploitability_near_zero
cargo test -p myosu-games-liars-dice profile::tests::strategy_is_nontrivial
cargo test -p myosu-games-liars-dice profile::tests::wire_serialization_works
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
cargo test -p myosu-games-liars-dice