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
- **preflight**: fail
  - Script: `set +e
cargo test events:: --no-ignore
cargo test shell:: --integration
cargo test schema::all_game_types
cargo test shell::shell_draw_
cargo test pipe::is_plain_text
true`
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 43 out
  - Files: crates/myosu-tui/src/events.rs, outputs/tui/shell/implementation.md, outputs/tui/shell/integration.md, outputs/tui/shell/verification.md
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
- **fixup**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 47 out
  - Files: outputs/tui/shell/verification.md
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


Current slice
- Slice 1: Event Loop Headless Test

Touched surfaces
- ``crates/myosu-tui/src/events.rs``

First proof gate
- ``cargo test events:: --no-ignore``

Implementation artifact must cover
- state whether Slice 1: Event Loop Headless Test was completed or partially completed
- list the touched files/modules for this slice

Verification artifact must cover
- record whether `cargo test events:: --no-ignore` passed and what it proved
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Settle stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
