# `chain:runtime` Implementation — Phase 0 Slice 1

## Slice Implemented

**Phase 0 Slice 1 — admit `crates/myosu-chain` as a real workspace member and seed the missing manifest surfaces for the chain restart.**

This slice restores honest Cargo surfaces without pretending the broken runtime,
common, or node source trees already compile. The reviewed lane artifacts say
the restart begins with workspace wiring, so this cut stays at the manifest
boundary only.

One implementation detail here is an inference from Cargo rather than from the
lane prose: Cargo does not support nested workspaces, so `crates/myosu-chain`
was introduced as a lightweight workspace anchor package while
`myosu-runtime`, `myosu-chain-common`, and `myosu-node` were seeded as sibling
manifests for later admission.

## Files Changed

- `Cargo.toml`
- `crates/myosu-chain/Cargo.toml`
- `crates/myosu-chain/src/lib.rs`
- `crates/myosu-chain/runtime/Cargo.toml`
- `crates/myosu-chain/common/Cargo.toml`
- `crates/myosu-chain/node/Cargo.toml`

## What Changed

### Root workspace

- Added `crates/myosu-chain` to the root workspace member list.
- Added the reviewed `polkadot-sdk` `stable2407` dependency line to
  `workspace.dependencies` for the minimal runtime pallet set and supporting
  SCALE crates.

### Chain workspace anchor

- Created a minimal `myosu-chain` package at `crates/myosu-chain/` so the root
  workspace now has an honest chain entry instead of a commented placeholder.

### Missing lane manifests

- Created `Cargo.toml` manifests for:
  - `myosu-runtime`
  - `myosu-chain-common`
  - `myosu-node`

These manifests are intentionally light. They establish package identity and
owned surfaces now; the Phase 1 runtime rewrite will supply the real dependency
graph and buildable source shape.

## Proof Commands For This Slice

Commands that should pass for this slice:

```bash
cargo metadata --format-version 1 --no-deps
CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo check -p myosu-chain
CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo check -p myosu-games
```

The Phase 1 proof commands remain unchanged and are **not** satisfied by this
slice yet:

```bash
cargo build -p myosu-runtime --release
cargo check -p myosu-runtime
```

## What Remains For The Next Slice

**Next slice: Phase 1 Slice 2 — replace `crates/myosu-chain/runtime/src/lib.rs`
with the reviewed minimal runtime (`frame_system + pallet_balances +
pallet_sudo + pallet_timestamp + frame_executive`) and admit
`crates/myosu-chain/runtime` as a direct workspace member so the lane proof
`cargo build -p myosu-runtime --release` becomes meaningful from the workspace
root.**
