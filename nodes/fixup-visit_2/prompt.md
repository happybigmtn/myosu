Goal: Implement the next approved `chain:runtime` slice.

Inputs:
- `runtime/spec.md`
- `runtime/review.md`

Scope:
- work only inside the smallest next approved implementation slice
- treat the reviewed lane artifacts as the source of truth
- keep changes aligned with the owned surfaces for `chain:runtime`

Required curated artifacts:
- `runtime/implementation.md`
- `runtime/verification.md`
- `runtime/quality.md`
- `runtime/promotion.md`
- `runtime/integration.md`


## Completed stages
- **preflight**: success
  - Script: `set +e
cargo build -p myosu-runtime --release` exits 0; `cargo check` on runtime crate passes.
cargo build -p myosu-node --release` exits 0.
true`
  - Stdout: (empty)
  - Stderr:
    ```
    /bin/bash: line 2: exits: command not found
    /bin/bash: line 2: on: command not found
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-node` did not match any packages
    error: unexpected argument '--releasecargo' found
    
      tip: a similar argument exists: '--release'
    
    Usage: cargo build --package [<SPEC>] --release
    
    For more information, try '--help'.
    ```
- **implement**: success
  - Model: MiniMax-M2.7-highspeed, 145 tokens in / 102 out
  - Files: Cargo.lock, Cargo.toml, crates/myosu-chain/Cargo.toml, crates/myosu-chain/runtime/Cargo.toml, crates/myosu-chain/runtime/build.rs, crates/myosu-chain/runtime/src/chain_spec.rs, crates/myosu-chain/runtime/src/lib.rs, crates/myosu-chain/src/lib.rs, outputs/chain/runtime/implementation.md, outputs/chain/runtime/integration.md, outputs/chain/runtime/promotion.md, outputs/chain/runtime/quality.md, outputs/chain/runtime/verification.md
- **verify**: fail
  - Script: `set -e
cargo build -p myosu-runtime --release` exits 0; `cargo check` on runtime crate passes.
cargo build -p myosu-node --release` exits 0.`
  - Stdout: (empty)
  - Stderr:
    ```
    /bin/bash: line 2: exits: command not found
    /bin/bash: line 2: on: command not found
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-node` did not match any packages
    error: unexpected argument '--releasecargo' found
    
      tip: a similar argument exists: '--release'
    
    Usage: cargo build --package [<SPEC>] --release
    
    For more information, try '--help'.
    ```
- **fixup**: success
  - Model: MiniMax-M2.7-highspeed, 109.4k tokens in / 2.6k out
  - Files: outputs/chain/runtime/verification.md
- **verify**: fail
  - Script: `set -e
cargo build -p myosu-runtime --release` exits 0; `cargo check` on runtime crate passes.
cargo build -p myosu-node --release` exits 0.`
  - Stdout: (empty)
  - Stderr:
    ```
    /bin/bash: line 2: exits: command not found
    /bin/bash: line 2: on: command not found
        Blocking waiting for file lock on package cache
        Blocking waiting for file lock on package cache
    error: package ID specification `myosu-node` did not match any packages
    error: unexpected argument '--releasecargo' found
    
      tip: a similar argument exists: '--release'
    
    Usage: cargo build --package [<SPEC>] --release
    
    For more information, try '--help'.
    ```

## Context
- failure_class: deterministic
- failure_signature: verify|deterministic|script failed with exit code: <n> ## stderr /bin/bash: line <n>: exits: command not found /bin/bash: line <n>: on: command not found blocking waiting for file lock on package cache blocking waiting for file lock on package cache error: pack


# Runtime Restart Implementation Lane — Fixup

Fix only the current slice for `runtime-implement`.


First proof gate
- ``cargo build -p myosu-runtime --release``

Verification artifact must cover
- record whether `cargo build -p myosu-runtime --release` passed and what it proved
- summarize the automated proof commands that ran and their outcomes

Priorities:
- unblock the active slice's first proof gate
- stay within the named slice and touched surfaces
- preserve setup constraints before expanding implementation scope
- keep implementation and verification artifacts durable and specific
- do not create or rewrite `promotion.md` during Fixup; that file is owned by the Settle stage
- do not hand-author `quality.md`; the Quality Gate rewrites it after verification
