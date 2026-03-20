Goal: Implement the next approved `play:tui` slice.

Inputs:
- `spec.md`
- `review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `play:tui`

Required curated artifacts:
- `implementation.md`
- `verification.md`
- `promotion.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
cargo build -p myosu-play
cargo test -p myosu-games-poker
cargo test -p myosu-play training::tests::hand_completes_fold
cargo test -p myosu-play training::tests::hand_completes_showdown
cargo test -p myosu-play blueprint::tests::load_valid_artifact
cargo test -p myosu-play advisor::tests::format_distribution_text
true`
  - Stdout: (empty)
  - Stderr:
    ```
    Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-play` did not match any packages
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 1.4m tokens in / 16.6k out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-play/Cargo.toml, crates/myosu-play/src/main.rs, outputs/play/tui/implementation.md, outputs/play/tui/verification.md
- **verify**: fail
  - Script: `set -e
cargo build -p myosu-play
cargo test -p myosu-games-poker
cargo test -p myosu-play training::tests::hand_completes_fold
cargo test -p myosu-play training::tests::hand_completes_showdown
cargo test -p myosu-play blueprint::tests::load_valid_artifact
cargo test -p myosu-play advisor::tests::format_distribution_text`
  - Stdout: (empty)
  - Stderr:
    ```
    Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.48s
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-games-poker` did not match any packages
    ```

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n> ## stderr blocking waiting for file lock on package cache blocking waiting for file lock on package cache blocking waiting for file lock on package cache blocking waiting for file lock on package cache fini


# Gameplay TUI Implementation Lane — Fixup

Fix only the current slice for `tui-implement`.

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Promote stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
