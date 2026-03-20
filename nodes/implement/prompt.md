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


# Poker Engine Implementation Lane — Plan

Lane: `poker-engine-implement`

Goal:
- Implement the next approved `games:poker-engine` slice.

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


Current slice
- Slice 1: Create myosu-games-poker Crate Skeleton

Touched surfaces
- ``crates/myosu-games-poker/Cargo.toml``

Set up first
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

Stage ownership:
- do not write `promotion.md` during Plan/Implement
- do not hand-author `quality.md`; it is regenerated by the Quality Gate
- `promotion.md` is owned by the Settle stage only
- keep source edits inside the named slice and touched surfaces
