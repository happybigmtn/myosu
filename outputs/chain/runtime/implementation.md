# `chain:runtime` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Phase 0 workspace wiring plus a proofable minimal runtime crate**

This slice implements the first honest runtime restart step from the reviewed lane artifacts:

- wire the chain runtime surfaces into the root Cargo workspace
- add real manifests for the chain marker crate, common crate, and runtime crate
- replace the non-buildable subtensor-forward-port runtime with a minimal FRAME runtime that compiles to Wasm
- keep node bring-up out of scope for the next slice

## What Changed

### Workspace wiring

- `Cargo.toml`
  - added `crates/myosu-chain`, `crates/myosu-chain/common`, and `crates/myosu-chain/runtime` as workspace members
  - added the `stable2407` Polkadot SDK dependency line needed by the new runtime/common crates

### Chain marker crate

- `crates/myosu-chain/Cargo.toml`
- `crates/myosu-chain/src/lib.rs`

Added a tiny top-level marker crate so the `crates/myosu-chain` path is now an honest Cargo package instead of a dead directory root.

### Common crate cleanup

- `crates/myosu-chain/common/Cargo.toml`
- `crates/myosu-chain/common/src/lib.rs`
- `crates/myosu-chain/common/src/currency.rs`

Added a real `myosu-chain-common` manifest and stripped the missing `subtensor_macros` / `runtime_common` dependencies out of the preserved common types surface. The crate now compiles with:

- `NetUid`, `MechId`, `NetUidStorageIndex`
- `AlphaCurrency`, `TaoCurrency`, and the simplified `Currency` trait
- `is_in_evm()` / `with_evm_context()`

### Minimal runtime bring-up

- `crates/myosu-chain/runtime/Cargo.toml`
- `crates/myosu-chain/runtime/build.rs`
- `crates/myosu-chain/runtime/src/lib.rs`

Replaced the non-buildable subtensor runtime with a minimal runtime composed of:

- `frame_system`
- `pallet_timestamp`
- `pallet_balances`
- `pallet_transaction_payment`
- `pallet_sudo`

The runtime now has:

- a valid `RuntimeVersion`
- block/extrinsic types
- the standard core/metadata/block-builder/tx-queue/offchain/runtime-payment/genesis APIs
- a working `substrate-wasm-builder` path that emits Wasm artifacts

## Files Changed

- `Cargo.toml`
- `Cargo.lock`
- `crates/myosu-chain/Cargo.toml`
- `crates/myosu-chain/src/lib.rs`
- `crates/myosu-chain/common/Cargo.toml`
- `crates/myosu-chain/common/src/lib.rs`
- `crates/myosu-chain/common/src/currency.rs`
- `crates/myosu-chain/runtime/Cargo.toml`
- `crates/myosu-chain/runtime/build.rs`
- `crates/myosu-chain/runtime/src/lib.rs`

## Proof Commands For This Slice

```bash
CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain-common

CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain

WASM_BUILD_WORKSPACE_HINT="$PWD" \
CARGO_NET_OFFLINE=true \
CARGO_TARGET_DIR=/tmp/myosu-chain-target \
cargo check --offline -p myosu-runtime

WASM_BUILD_WORKSPACE_HINT="$PWD" \
CARGO_NET_OFFLINE=true \
CARGO_TARGET_DIR=/tmp/myosu-chain-target \
cargo build --offline --release -p myosu-runtime
```

## What Remains

- add the `myosu-node` crate manifest and replace the current node scaffold with a real CLI/service wiring
- add the dev chain spec and node startup path for the minimal runtime
- keep the runtime minimal until the node slice proves block production
- only after node bring-up, start integrating Myosu-specific pallets back into the runtime

