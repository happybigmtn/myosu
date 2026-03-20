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
    (30 lines omitted)
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
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 84 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-games-poker/Cargo.toml, crates/myosu-games-poker/src/exploit.rs, crates/myosu-games-poker/src/lib.rs, crates/myosu-games-poker/src/query.rs, crates/myosu-games-poker/src/solver.rs, crates/myosu-games-poker/src/training.rs, crates/myosu-games-poker/src/wire.rs, ops/evidence/games-poker-engine-implementation.md, ops/evidence/games-poker-engine-quality.md, ops/evidence/games-poker-engine-verification.md
- **verify**: fail
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
    (65 lines omitted)
    ---- wire::tests::nlhe_info_roundtrip stdout ----
    
    thread 'wire::tests::nlhe_info_roundtrip' (3753879) panicked at /home/r/.cargo/git/checkouts/robopoker-092d043dee5e8d7f/0471631/crates/nlhe/src/encoder.rs:33:14:
    isomorphism not found in abstraction lookup
    
    ---- training::tests::session_checkpoint_frequency stdout ----
    
    thread 'training::tests::session_checkpoint_frequency' (3753876) panicked at /home/r/.cargo/git/checkouts/robopoker-092d043dee5e8d7f/0471631/crates/nlhe/src/encoder.rs:33:14:
    isomorphism not found in abstraction lookup
    
    
    failures:
        exploit::tests::random_strategy_high_exploit
        exploit::tests::remote_matches_local
        exploit::tests::trained_strategy_low_exploit
        query::tests::handle_valid_query
        query::tests::response_probabilities_sum_to_one
        solver::tests::checkpoint_roundtrip
        solver::tests::exploitability_decreases
        solver::tests::strategy_is_valid_distribution
        solver::tests::train_100_iterations
        training::tests::session_checkpoint_frequency
        wire::tests::nlhe_info_roundtrip
    
    test result: FAILED. 4 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    (99 lines omitted)
    ...
    42 |     CorruptedFile(String),
       |     ^^^^^^^^^^^^^
       |
       = note: `SolverError` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis
       = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
    
    warning: trait `WireEncode` is never used
      --> crates/myosu-games-poker/src/wire.rs:33:11
       |
    33 | pub trait WireEncode {
       |           ^^^^^^^^^^
    
    warning: function `chips_to_mbbh` is never used
      --> crates/myosu-games-poker/src/exploit.rs:73:8
       |
    73 | pub fn chips_to_mbbh(utility: Utility) -> Utility {
       |        ^^^^^^^^^^^^^
    
       Compiling myosu-games-poker v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J35965VYNMEPA52FM82R2/worktree/crates/myosu-games-poker)
    warning: `myosu-games-poker` (lib) generated 8 warnings (run `cargo fix --lib -p myosu-games-poker` to apply 4 suggestions)
    warning: `myosu-games-poker` (lib test) generated 4 warnings (4 duplicates)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 0.76s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_games_poker-0587e8d928b5eb58)
    error: test failed, to rerun pass `-p myosu-games-poker --lib`
    ```

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n> ## stdout running <n> tests test query::tests::handle_invalid_info_bytes ... ok test exploit::tests::random_strategy_high_exploit ... failed test exploit::tests::remote_matches_local ... failed test exploit


# Poker Engine Implementation Lane — Fixup

Fix only the current slice for `poker-engine-implement`.

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
