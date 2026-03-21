Goal: Implement the next approved `tui:shell` slice.

Inputs:
- `spec.md`
- `review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `tui:shell`

Required curated artifacts:
- `implementation.md`
- `verification.md`
- `quality.md`
- `promotion.md`
- `integration.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
cargo test events:: --no-ignore
cargo test shell:: --integration
cargo test schema::all_game_types
cargo test shell::shell_draw_
cargo test pipe::is_plain_text
true`
  - Stdout: (empty)
  - Stderr:
    ```
    file:///home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/artifacts/values/command.stderr.json
    ```
- **implement**: success
  - Model: gpt-5.4, 1.9m tokens in / 19.0k out
  - Files: crates/myosu-chain/pallets/game-solver/src/coinbase/block_step.rs, crates/myosu-chain/pallets/game-solver/src/lib.rs, crates/myosu-chain/pallets/game-solver/src/macros/config.rs, crates/myosu-tui/src/events.rs, outputs/tui/shell/implementation.md, outputs/tui/shell/integration.md, outputs/tui/shell/verification.md
- **verify**: fail
  - Script: `set -e
cargo test events:: --no-ignore
cargo test shell:: --integration
cargo test schema::all_game_types
cargo test shell::shell_draw_
cargo test pipe::is_plain_text`
  - Stdout: (empty)
  - Stderr:
    ```
    error: unexpected argument '--no-ignore' found
    
      tip: a similar argument exists: '--no-run'
      tip: to pass '--no-ignore' as a value, use '-- --no-ignore'
    
    Usage: cargo test --no-run <TESTNAME> [-- [ARGS]...]
    
    For more information, try '--help'.
    ```

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n> ## stderr error: unexpected argument '--no-ignore' found tip: a similar argument exists: '--no-run' tip: to pass '--no-ignore' as a value,use '-- --no-ignore' usage: cargo test --no-run <testname> [-- [args


# Shell Implementation Lane — Fixup

Fix only the current slice for `shell-implement`.

Current Slice Contract:
Inspect the relevant repo surfaces, preserve existing doctrine, and produce the lane artifacts honestly.


Verification artifact must cover
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Review stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
