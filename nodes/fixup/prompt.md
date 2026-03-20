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
    (39 lines omitted)
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
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
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 27 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-games-poker/Cargo.toml, crates/myosu-games-poker/src/exploit.rs, crates/myosu-games-poker/src/lib.rs, crates/myosu-games-poker/src/query.rs, crates/myosu-games-poker/src/solver.rs, crates/myosu-games-poker/src/training.rs, crates/myosu-games-poker/src/wire.rs, outputs/games/poker-engine/implementation.md, outputs/games/poker-engine/quality.md, outputs/games/poker-engine/verification.md
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
    (47 lines omitted)
    thread 'solver::tests::exploitability_decreases' (3371520) panicked at /home/r/.cargo/git/checkouts/robopoker-092d043dee5e8d7f/0471631/crates/nlhe/src/encoder.rs:33:14:
    isomorphism not found in abstraction lookup
    
    ---- solver::tests::train_100_iterations stdout ----
    
    thread 'solver::tests::train_100_iterations' (3371522) panicked at /home/r/.cargo/git/checkouts/robopoker-092d043dee5e8d7f/0471631/crates/nlhe/src/encoder.rs:33:14:
    isomorphism not found in abstraction lookup
    
    ---- training::tests::session_checkpoint_frequency stdout ----
    
    thread 'training::tests::session_checkpoint_frequency' (3371523) panicked at /home/r/.cargo/git/checkouts/robopoker-092d043dee5e8d7f/0471631/crates/nlhe/src/encoder.rs:33:14:
    isomorphism not found in abstraction lookup
    
    
    failures:
        exploit::tests::random_strategy_high_exploit
        exploit::tests::remote_matches_local
        exploit::tests::trained_strategy_low_exploit
        query::tests::response_probabilities_sum_to_one
        solver::tests::checkpoint_roundtrip
        solver::tests::exploitability_decreases
        solver::tests::train_100_iterations
        training::tests::session_checkpoint_frequency
    
    test result: FAILED. 7 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    ```
  - Stderr:
    ```
    Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on artifact directory
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.23s
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
       Compiling myosu-games-poker v0.1.0 (/home/r/.fabro/runs/20260320-01KM6J48QT2T9EWKBMSE5FSWA9/worktree/crates/myosu-games-poker)
    warning: unused import: `rbp_core::Arbitrary`
      --> crates/myosu-games-poker/src/exploit.rs:72:9
       |
    72 |     use rbp_core::Arbitrary;
       |         ^^^^^^^^^^^^^^^^^^^
       |
       = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default
    
    warning: `myosu-games-poker` (lib test) generated 1 warning (run `cargo fix --lib -p myosu-games-poker --tests` to apply 1 suggestion)
        Finished `test` profile [unoptimized + debuginfo] target(s) in 1.12s
         Running unittests src/lib.rs (/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/deps/myosu_games_poker-3d57a602d5d5e2c9)
    error: test failed, to rerun pass `-p myosu-games-poker --lib`
    ```

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n> ## stdout running <n> tests test query::tests::handle_invalid_info_bytes ... ok test exploit::tests::random_strategy_high_exploit ... failed test exploit::tests::trained_strategy_low_exploit ... failed test


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
