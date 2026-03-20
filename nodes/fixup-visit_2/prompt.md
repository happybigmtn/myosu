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
- `poker-engine/quality.md`
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
    (27 lines omitted)
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
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 61 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-games-poker/Cargo.toml, crates/myosu-games-poker/README.md, crates/myosu-games-poker/src/exploit.rs, crates/myosu-games-poker/src/lib.rs, crates/myosu-games-poker/src/query.rs, crates/myosu-games-poker/src/solver.rs, crates/myosu-games-poker/src/training.rs, crates/myosu-games-poker/src/wire.rs, outputs/games/poker-engine/implementation.md, outputs/games/poker-engine/verification.md
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
    (88 lines omitted)
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::trained_strategy_low_exploit ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::random_strategy_high_exploit ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::remote_matches_local ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test training::tests::session_checkpoint_frequency ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (277 lines omitted)
      --> crates/myosu-games-poker/src/wire.rs:51:11
       |
    51 | pub trait WireSerializable: Sized {
       |           ^^^^^^^^^^^^^^^^
    
    warning: `myosu-games-poker` (lib test) generated 2 warnings
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-85aa73997acd1a31)
    warning: struct `Poker` is never constructed
     --> crates/myosu-games-poker/src/wire.rs:9:12
      |
    9 | pub struct Poker;
      |            ^^^^^
      |
      = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
    
    warning: trait `WireSerializable` is never used
      --> crates/myosu-games-poker/src/wire.rs:51:11
       |
    51 | pub trait WireSerializable: Sized {
       |           ^^^^^^^^^^^^^^^^
    
    warning: `myosu-games-poker` (lib test) generated 2 warnings
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-85aa73997acd1a31)
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
scan_placeholder 'crates/myosu-games-poker/Cargo.toml'
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
- crates/myosu-games-poker/Cargo.toml

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
- **settle**: success
  - Model: gpt-5.4, 5.4m tokens in / 35.9k out
  - Files: crates/myosu-games-poker/src/lib.rs, crates/myosu-games-poker/src/solver.rs, outputs/games/poker-engine/implementation.md, outputs/games/poker-engine/promotion.md, outputs/games/poker-engine/quality.md, outputs/games/poker-engine/verification.md
- **audit**: fail
  - Script: `test -f outputs/games/poker-engine/implementation.md && test -f outputs/games/poker-engine/verification.md && test -f outputs/games/poker-engine/quality.md && test -f outputs/games/poker-engine/promotion.md && test -f outputs/games/poker-engine/integration.md && grep -Eq '^merge_ready: yes$' outputs/games/poker-engine/promotion.md && grep -Eq '^manual_proof_pending: no$' outputs/games/poker-engine/promotion.md && grep -Eq '^quality_ready: yes$' outputs/games/poker-engine/quality.md && grep -Eq '^placeholder_debt: no$' outputs/games/poker-engine/quality.md && grep -Eq '^warning_debt: no$' outputs/games/poker-engine/quality.md && grep -Eq '^artifact_mismatch_risk: no$' outputs/games/poker-engine/quality.md && grep -Eq '^manual_followup_required: no$' outputs/games/poker-engine/quality.md`
  - Stdout: (empty)
  - Stderr: (empty)
- **fixup**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 110 out
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
    (88 lines omitted)
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::trained_strategy_low_exploit ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::random_strategy_high_exploit ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test exploit::tests::remote_matches_local ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    
    
    running 1 test
    test training::tests::session_checkpoint_frequency ... ok
    
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (277 lines omitted)
      --> crates/myosu-games-poker/src/wire.rs:51:11
       |
    51 | pub trait WireSerializable: Sized {
       |           ^^^^^^^^^^^^^^^^
    
    warning: `myosu-games-poker` (lib test) generated 2 warnings
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-85aa73997acd1a31)
    warning: struct `Poker` is never constructed
     --> crates/myosu-games-poker/src/wire.rs:9:12
      |
    9 | pub struct Poker;
      |            ^^^^^
      |
      = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
    
    warning: trait `WireSerializable` is never used
      --> crates/myosu-games-poker/src/wire.rs:51:11
       |
    51 | pub trait WireSerializable: Sized {
       |           ^^^^^^^^^^^^^^^^
    
    warning: `myosu-games-poker` (lib test) generated 2 warnings
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/fabro/programs/../../.raspberry/cargo-target/debug/deps/myosu_games_poker-85aa73997acd1a31)
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
scan_placeholder 'crates/myosu-games-poker/Cargo.toml'
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
- crates/myosu-games-poker/Cargo.toml

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
- **settle**: success
  - Model: gpt-5.4, 5.4m tokens in / 35.9k out
  - Files: crates/myosu-games-poker/src/lib.rs, crates/myosu-games-poker/src/solver.rs, outputs/games/poker-engine/implementation.md, outputs/games/poker-engine/promotion.md, outputs/games/poker-engine/quality.md, outputs/games/poker-engine/verification.md
- **audit**: fail
  - Script: `test -f outputs/games/poker-engine/implementation.md && test -f outputs/games/poker-engine/verification.md && test -f outputs/games/poker-engine/quality.md && test -f outputs/games/poker-engine/promotion.md && test -f outputs/games/poker-engine/integration.md && grep -Eq '^merge_ready: yes$' outputs/games/poker-engine/promotion.md && grep -Eq '^manual_proof_pending: no$' outputs/games/poker-engine/promotion.md && grep -Eq '^quality_ready: yes$' outputs/games/poker-engine/quality.md && grep -Eq '^placeholder_debt: no$' outputs/games/poker-engine/quality.md && grep -Eq '^warning_debt: no$' outputs/games/poker-engine/quality.md && grep -Eq '^artifact_mismatch_risk: no$' outputs/games/poker-engine/quality.md && grep -Eq '^manual_followup_required: no$' outputs/games/poker-engine/quality.md`
  - Stdout: (empty)
  - Stderr: (empty)

## Context
- failure_class: deterministic
- failure_signature: audit|deterministic|script failed with exit code: <n>


# Poker Engine Implementation Lane — Fixup

Fix only the current slice for `poker-engine-implement`.


Current slice
- Slice 1: Create myosu-games-poker Crate Skeleton

Touched surfaces
- ``crates/myosu-games-poker/Cargo.toml``

Setup checks
- Add crate to workspace members. Cargo.toml:
- Dependency on `rbp-nlhe` and `rbp-mccfr` (git) at same rev as `myosu-games` (`04716310143094ab41ec7172e6cea5a2a66744ef`)
- **Verify and enable `serde` feature** on the robopoker crates (needed for AC-PE-03)
- Dependency on `myosu-games`
- `crate-type = ["lib"]`

First proof gate
- ``cargo build -p myosu-games-poker``

Execution guidance
- Start: The specification is stable and self-contained. The implementation lane can begin with Slice 1 (crate skeleton) immediately. The only prerequisite from the upstream `games:traits` lane is satisfied (git dependency migration is done).
- Order: The `serde` feature verification in Slice 1 must succeed before Slice 3 is started

Implementation artifact must cover
- state whether Slice 1: Create myosu-games-poker Crate Skeleton was completed or partially completed
- list the touched files/modules for this slice
- note which setup steps were completed, deferred, or intentionally skipped
- call out anything that still blocks the next slice from starting

Verification artifact must cover
- record whether `cargo build -p myosu-games-poker` passed and what it proved
- summarize the automated proof commands that ran and their outcomes
- say whether the slice is complete enough to move to the next ordered slice

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Settle stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
