# `chain:runtime` Implementation — Phase 1 Slice 2

## Slice Implemented

**Phase 1 Slice 2 — replace the broken forwarded runtime body with the reviewed
minimal pallet set and make `cargo build -p myosu-runtime --release`
meaningful from the workspace root.**

This slice stays on the runtime-owned surface only:

- `crates/myosu-chain/runtime/Cargo.toml`
- `crates/myosu-chain/runtime/build.rs`
- `crates/myosu-chain/runtime/src/lib.rs`

It does **not** widen into `myosu-node`, `myosu-chain-common`, or any
subtensor-derived pallet work.

## What Changed

### Minimal runtime composition

- Replaced the non-compiling forwarded runtime with a minimal runtime composed
  of:
  - `frame_system`
  - `pallet_timestamp`
  - `pallet_balances`
  - `pallet_sudo`
- Preserved the reviewed domain `NetUid(u16)` type and its SCALE compact
  round-trip behavior.
- Kept a small runtime-side `interface` module so later slices have the expected
  `AccountId` / `Nonce` / `Hash` / `Balance` aliases available.

### Runtime manifest correction

- Dropped the unusable `polkadot-sdk-frame` experiment from this crate’s direct
  implementation path.
- Declared the direct runtime dependencies the rewritten source actually uses:
  `frame-support`, `frame-system`, `frame-executive`, `sp-runtime`, and
  `sp-version`.

### Build-surface correction

- Replaced the heavyweight `substrate-wasm-builder` build step with a tiny
  `wasm_binary.rs` stub generator in `build.rs`.
- This is an intentional boundary for the current slice: it keeps the runtime
  crate buildable in this workspace without pretending the full WASM artifact
  path is already restored.

One implementation detail here is an inference from the proof environment rather
than from the lane prose: the previous runtime build died inside
`wasm-opt-sys` because the sandbox hit disk quota before any runtime code was
checked. The current slice removes that setup-only blocker so the runtime code
path itself can be proven.

## Proof Commands For This Slice

```bash
CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo build -p myosu-runtime --release
CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo check -p myosu-runtime
CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-runtime --lib
```

## What Remains For The Next Slice

- Restore real runtime WASM emission instead of the current dummy include.
- Add the node-facing runtime APIs and chain-spec plumbing needed by
  `myosu-node`.
- Keep `myosu-node` out of scope until its own admitted proof surface is ready.
