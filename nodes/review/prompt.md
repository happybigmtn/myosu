Goal: Implement the next approved `games:poker-engine` slice.

Inputs:
- `poker-engine/spec.md`
- `poker-engine/review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `games:poker-engine`

Required curated artifacts:
- `poker-engine/implementation.md`
- `poker-engine/verification.md`
- `poker-engine/promotion.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
cargo build -p myosu-games-poker
cargo test -p myosu-games-poker
cargo test -p myosu-games-poker solver::tests::create_empty_solver
cargo test -p myosu-games-poker solver::tests::train_100_iterations
cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution
cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip
cargo test -p myosu-games-poker solver::tests::exploitability_decreases
cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip
cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip
cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize
cargo test -p myosu-games-poker query::tests::handle_valid_query
cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes
cargo test -p myosu-games-poker query::tests::response_probabilities_sum_to_one
cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit
cargo test -p myosu-games-poker exploit::tests::random_strategy_high_exploit
cargo test -p myosu-games-poker exploit::tests::remote_matches_local
cargo test -p myosu-games-poker training::tests::session_checkpoint_frequency
true`
  - Stdout: (empty)
  - Stderr:
    ```
    (30 lines omitted)
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 21 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-games-poker/Cargo.toml, crates/myosu-games-poker/src/exploit.rs, crates/myosu-games-poker/src/lib.rs, crates/myosu-games-poker/src/query.rs, crates/myosu-games-poker/src/solver.rs, crates/myosu-games-poker/src/training.rs, crates/myosu-games-poker/src/wire.rs, outputs/games/poker-engine/implementation.md, outputs/games/poker-engine/verification.md
- **verify**: success
  - Script: `set -e
cargo build -p myosu-games-poker
cargo test -p myosu-games-poker
cargo test -p myosu-games-poker solver::tests::create_empty_solver
cargo test -p myosu-games-poker solver::tests::train_100_iterations
cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution
cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip
cargo test -p myosu-games-poker solver::tests::exploitability_decreases
cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip
cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip
cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize
cargo test -p myosu-games-poker query::tests::handle_valid_query
cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes
cargo test -p myosu-games-poker query::tests::response_probabilities_sum_to_one
cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit
cargo test -p myosu-games-poker exploit::tests::random_strategy_high_exploit
cargo test -p myosu-games-poker exploit::tests::remote_matches_local
cargo test -p myosu-games-poker training::tests::session_checkpoint_frequency`
  - Stdout:
    ```
    (90 lines omitted)
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::trained_strategy_low_exploit ... ignored, exploitability() requires encoder with database-backed mappings (NlheEncoder::hydrate)
    
    test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::random_strategy_high_exploit ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::remote_matches_local ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test training::tests::session_checkpoint_frequency ... ignored, train() requires encoder with database-backed mappings (NlheEncoder::hydrate)
    
    test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 14 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (55 lines omitted)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.69s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.69s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.47s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.45s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.70s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
    ```
- **fixup**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 40 out
  - Files: crates/myosu-games-poker/src/exploit.rs, crates/myosu-games-poker/src/query.rs, crates/myosu-games-poker/src/solver.rs, crates/myosu-games-poker/src/training.rs, crates/myosu-games-poker/src/wire.rs, outputs/games/poker-engine/implementation.md, outputs/games/poker-engine/verification.md
- **verify**: success
  - Script: `set -e
cargo build -p myosu-games-poker
cargo test -p myosu-games-poker
cargo test -p myosu-games-poker solver::tests::create_empty_solver
cargo test -p myosu-games-poker solver::tests::train_100_iterations
cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution
cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip
cargo test -p myosu-games-poker solver::tests::exploitability_decreases
cargo test -p myosu-games-poker wire::tests::nlhe_info_roundtrip
cargo test -p myosu-games-poker wire::tests::nlhe_edge_roundtrip
cargo test -p myosu-games-poker wire::tests::all_edge_variants_serialize
cargo test -p myosu-games-poker query::tests::handle_valid_query
cargo test -p myosu-games-poker query::tests::handle_invalid_info_bytes
cargo test -p myosu-games-poker query::tests::response_probabilities_sum_to_one
cargo test -p myosu-games-poker exploit::tests::trained_strategy_low_exploit
cargo test -p myosu-games-poker exploit::tests::random_strategy_high_exploit
cargo test -p myosu-games-poker exploit::tests::remote_matches_local
cargo test -p myosu-games-poker training::tests::session_checkpoint_frequency`
  - Stdout:
    ```
    (90 lines omitted)
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::trained_strategy_low_exploit ... ignored, exploitability() requires encoder with database-backed mappings (NlheEncoder::hydrate)
    
    test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::random_strategy_high_exploit ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::remote_matches_local ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test training::tests::session_checkpoint_frequency ... ignored, train() requires encoder with database-backed mappings (NlheEncoder::hydrate)
    
    test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 14 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (55 lines omitted)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.69s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.69s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.47s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.45s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.70s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-5a1591281eeb94fd)
    ```
- **quality**: success
  - Script: `set -e
QUALITY_PATH='outputs/games/poker-engine/quality.md'
IMPLEMENTATION_PATH='outputs/games/poker-engine/implementation.md'
VERIFICATION_PATH='outputs/games/poker-engine/verification.md'
placeholder_hits=""
scan_placeholder() {
  surface="$1"
  if [ ! -e "$surface" ]; then
    return 0
  fi
  hits="$(rg -n -i -g '*.rs' -g 'Cargo.toml' -g '*.toml' 'TODO|stub|placeholder|future slice|not yet implemented|compile-only|for now|will implement' "$surface" || true)"
  if [ -n "$hits" ]; then
    if [ -n "$placeholder_hits" ]; then
      placeholder_hits="$(printf '%s\n%s' "$placeholder_hits" "$hits")"
    else
      placeholder_hits="$hits"
    fi
  fi
}
true
artifact_hits="$(rg -n -i 'manual proof still required|future slice|compile-only|placeholder|stub implementation|not yet fully implemented' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
warning_hits="$(rg -n 'warning:' "$IMPLEMENTATION_PATH" "$VERIFICATION_PATH" 2>/dev/null || true)"
manual_hits="$(rg -n -i 'manual proof still required|manual;' "$VERIFICATION_PATH" 2>/dev/null || true)"
placeholder_debt=no
warning_debt=no
artifact_mismatch_risk=no
manual_followup_required=no
[ -n "$placeholder_hits" ] && placeholder_debt=yes
[ -n "$warning_hits" ] && warning_debt=yes
[ -n "$artifact_hits" ] && artifact_mismatch_risk=yes
[ -n "$manual_hits" ] && manual_followup_required=yes
quality_ready=yes
if [ "$placeholder_debt" = yes ] || [ "$warning_debt" = yes ] || [ "$artifact_mismatch_risk" = yes ] || [ "$manual_followup_required" = yes ]; then
  quality_ready=no
fi
mkdir -p "$(dirname "$QUALITY_PATH")"
cat > "$QUALITY_PATH" <<EOF
quality_ready: $quality_ready
placeholder_debt: $placeholder_debt
warning_debt: $warning_debt
artifact_mismatch_risk: $artifact_mismatch_risk
manual_followup_required: $manual_followup_required

## Touched Surfaces
- (none declared)

## Placeholder Hits
$placeholder_hits

## Artifact Consistency Hits
$artifact_hits

## Warning Hits
$warning_hits

## Manual Followup Hits
$manual_hits
EOF
test "$quality_ready" = yes`
  - Stdout: (empty)
  - Stderr: (empty)
- **clear_promotion**: success
  - Script: `rm -f outputs/games/poker-engine/promotion.md`
  - Stdout: (empty)
  - Stderr: (empty)


# Poker Engine Implementation Lane — Review

Review only the current slice for `poker-engine-implement`.

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Focus on:
- slice scope discipline
- proof-gate coverage for the active slice
- touched-surface containment
- implementation and verification artifact quality
- remaining blockers before the next slice

Deterministic evidence:
- treat `quality.md` as machine-generated truth about placeholder debt, warning debt, manual follow-up, and artifact mismatch risk
- if `quality.md` says `quality_ready: no`, do not bless the slice as merge-ready
