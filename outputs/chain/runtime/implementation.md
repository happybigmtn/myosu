# `chain:runtime` Implementation — Phase 1

## Slice Implemented

**Phase 1 — Minimal Runtime**

This slice replaces the inherited subtensor/frontier runtime definition with a
minimal Substrate runtime that matches the reviewed restart boundary:

- `frame_system`
- `pallet_timestamp`
- `pallet_balances`
- `pallet_sudo`
- `frame_executive`

The work stayed inside the runtime-owned surfaces plus the lockfile that Cargo
updated while resolving the new runtime dependency set.

## Runtime-Owned Changes

### Minimal runtime source

`crates/myosu-chain/runtime/src/lib.rs` now contains a clean phase-1 runtime
instead of the inherited 2500-line subtensor fork:

- defines a minimal runtime version and native-version surface
- composes only `System`, `Timestamp`, `Balances`, and `Sudo`
- uses standard FRAME default config preludes with the few required overrides
- defines the signed extensions needed for a basic solochain transaction flow
- implements the minimal runtime APIs for:
  - core execution
  - metadata
  - block building
  - transaction validation
  - offchain worker
  - account nonce
  - genesis builder
- exports a small `interface` module for downstream node wiring in the next
  slice

### WASM build path

Added `crates/myosu-chain/runtime/build.rs` to generate the runtime wasm blob.

The build script also sets `WASM_BUILD_WORKSPACE_HINT` to the repo root. This
was necessary because proof commands in this environment use an external
`CARGO_TARGET_DIR`, and the wasm builder otherwise fails to locate the
workspace `Cargo.lock`.

### Runtime manifest alignment

`crates/myosu-chain/runtime/Cargo.toml` now describes the actual phase-1
runtime:

- adds `build = "build.rs"`
- switches from the inherited direct runtime crate set to the
  `polkadot-sdk-frame` runtime facade used by the current template stack
- keeps the pallet set minimal: balances, sudo, timestamp
- adds `sp-genesis-builder` and the wasm-builder std path needed by the new
  runtime source

### Lockfile refresh

`Cargo.lock` now records the additional phase-1 runtime dependencies pulled in
by the minimal FRAME runtime surface, including:

- `polkadot-sdk-frame`
- `sp-block-builder`
- `sp-offchain`
- `sp-session`
- `sp-transaction-pool`
- `frame-system-rpc-runtime-api`

## What This Slice Guarantees Now

- `myosu-runtime` is a real minimal runtime, not a non-buildable subtensor
  forward-port.
- the runtime builds through the wasm pipeline with an external target dir in
  this sandboxed environment
- the release build emits:
  - `myosu_runtime.wasm`
  - `myosu_runtime.compact.wasm`
  - `myosu_runtime.compact.compressed.wasm`

## Deliberate Non-Changes

- `crates/myosu-chain/common/` was not advanced here; its cleanup remains a
  later reviewed slice
- `crates/myosu-chain/node/` was not rewritten here; node service wiring
  remains phase 2 work
- `outputs/chain/runtime/promotion.md` remains review-owned
- `outputs/chain/runtime/quality.md` remains quality-gate owned
