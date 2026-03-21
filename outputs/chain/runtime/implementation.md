# `chain:runtime` Implementation — Slice 1

## Slice Implemented

**Slice 1 — workspace wiring plus a proofable minimal runtime crate**

This fixup keeps the lane on the first approved restart slice from
[`spec.md`](/home/r/.fabro/runs/20260320-01KM6ZFDDEXKQB27C3Y3WSP01R/worktree/outputs/chain/runtime/spec.md)
and
[`review.md`](/home/r/.fabro/runs/20260320-01KM6ZFDDEXKQB27C3Y3WSP01R/worktree/outputs/chain/runtime/review.md):

- wire the chain runtime surfaces into the root Cargo workspace
- make `crates/myosu-chain` and `crates/myosu-chain/common` honest packages
- replace the non-buildable subtensor forward-port runtime with a minimal FRAME runtime that compiles to Wasm
- stop before node packaging, chain spec wiring, and custom pallet reintegration

## Proof Boundary

Only the fenced commands in the proof section below belong to Slice 1. This
fixup does not advance the lane to node bring-up, and it does not claim proof
for any `myosu-node` package or devnet startup path.

## What Changed

### Workspace wiring

- `Cargo.toml`
  - added `crates/myosu-chain`, `crates/myosu-chain/common`, and `crates/myosu-chain/runtime` as workspace members
  - added the `stable2407` Polkadot SDK dependency line needed by the new runtime/common crates

### Chain marker crate

- `crates/myosu-chain/Cargo.toml`
- `crates/myosu-chain/src/lib.rs`

Added a tiny top-level marker crate so the `crates/myosu-chain` directory is a
real Cargo package instead of a dead root.

### Common crate cleanup

- `crates/myosu-chain/common/Cargo.toml`
- `crates/myosu-chain/common/src/lib.rs`
- `crates/myosu-chain/common/src/currency.rs`

Added a real `myosu-chain-common` manifest and stripped the missing
`subtensor_macros` / `runtime_common` dependencies out of the preserved common
types surface. The crate now compiles with:

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
- block and extrinsic types
- the standard core, metadata, block-builder, tx-queue, offchain, runtime-payment, and genesis APIs
- a working `substrate-wasm-builder` path that emits Wasm artifacts

## Owned Surfaces Changed

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

## Adjacent Carryover From The Prior Implement Run

The earlier implementation run also touched adjacent files outside the approved
`chain:runtime` proof boundary, including:

- `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- `crates/myosu-chain/pallets/game-solver/src/coinbase/block_step.rs`
- `crates/myosu-chain/pallets/game-solver/src/macros/config.rs`
- `crates/myosu-tui/src/events.rs`
- `crates/myosu-tui/src/renderer.rs`
- `crates/myosu-tui/src/shell.rs`

Those edits are not needed for the Slice 1 runtime proof and are not claimed as
validated by this fixup.

## Automated Proof Commands For This Slice

```bash
CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain-common
CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain
WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-runtime
WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo build --offline --release -p myosu-runtime
```

## What Remains

- create the `myosu-node` package manifest and wire the existing node scaffold into a real CLI/service crate
- add the dev chain spec and node startup path for the minimal runtime
- keep the runtime minimal until the node slice proves binary build and block production
- only after node bring-up, start integrating Myosu-specific pallets back into the runtime
