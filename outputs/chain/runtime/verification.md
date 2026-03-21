# `chain:runtime` Verification — Restart Slice 1 Fixup

## Automated Commands Run

| Command | Exit / State | Outcome |
|---------|--------------|---------|
| `cargo metadata --no-deps --format-version 1 --manifest-path crates/myosu-chain/runtime/Cargo.toml` | 0 | Passed. Cargo still recognizes `myosu-runtime` as a workspace package. |
| `cargo check -p myosu-runtime` | 101 | Failed immediately in this sandbox because Cargo targeted `/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/.../.cargo-lock`, which is read-only here. |
| `cargo build -p myosu-runtime --release` | 101 | Same immediate read-only target-dir failure as `cargo check`. |
| `env CARGO_TARGET_DIR=.raspberry/cargo-target cargo check -p myosu-runtime` | in progress during fixup | Reached real dependency compilation through the polkadot-sdk stack and advanced as far as `pallet-sudo` / `pallet-balances` with no runtime Rust error emitted during this turn. |
| `env CARGO_TARGET_DIR=.raspberry/cargo-target cargo build -p myosu-runtime --release` | in progress during fixup | Reached the real release build path and compiled deep into the runtime dependency graph instead of failing on the shared read-only target directory. |

## What This Fixup Proves

- the previous verify failure was not caused by an immediate syntax or manifest
  problem in `crates/myosu-chain/runtime/`
- the active proof gate needed an explicit writable target-dir override in this
  sandbox
- the active slice proof had to be restricted to Phase 1 runtime commands only;
  the Phase 2 node proof was incorrectly being pulled into this lane too early

## What Is Not Yet Proven

- a final exit-0 completion for `env CARGO_TARGET_DIR=.raspberry/cargo-target cargo check -p myosu-runtime`
- a final exit-0 completion for `env CARGO_TARGET_DIR=.raspberry/cargo-target cargo build -p myosu-runtime --release`
- any `myosu-node` build result; that remains out of scope for this slice

## Active Proof Commands

```bash
env CARGO_TARGET_DIR=.raspberry/cargo-target cargo check -p myosu-runtime
env CARGO_TARGET_DIR=.raspberry/cargo-target cargo build -p myosu-runtime --release
```

These are the commands the verification step should run for Restart Slice 1.
