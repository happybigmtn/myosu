# Phase 1 Promotion — Myosu Runtime

**Lane**: `chain:runtime`
**Date**: 2026-03-20

---

## Phase 1 Gate: PASSED

| Proof | Value |
|-------|-------|
| `cargo build -p myosu-runtime --release` | Exit 0 |
| WASM binary produced | `myosu_runtime.wasm` (~1.8MB) |
| Compact WASM | `myosu_runtime.compact.wasm` (~1.2MB) |
| Compressed WASM | `myosu_runtime.compact.compressed.wasm` (~350KB) |
| Lib compiles clean | Zero errors, zero warnings on our code |

**Phase 1 is complete.** The minimal Substrate runtime (frame_system + pallet_balances + pallet_sudo + pallet_timestamp) builds to WASM.

---

## Next: Phase 2 — Node Binary + Common Crate

Per the restart spec, Phase 2 consists of:

### Phase 2A — `myosu-node` Binary

**Goal**: `cargo build -p myosu-node --release` exits 0.

Required files:
- `crates/myosu-chain/node/src/lib.rs` — service entry point
- `crates/myosu-chain/node/src/main.rs` — `fn main()` calling `command::run()`
- `crates/myosu-chain/node/src/service.rs` — `NewFull` service setup
- `crates/myosu-chain/node/src/command.rs` — CLI `command::run()` implementation
- `crates/myosu-chain/node/src/rpc.rs` — HTTP + WebSocket RPC
- `crates/myosu-chain/node/src/chain_spec.rs` — Full chain spec extending runtime `GenesisConfig`

Node composition:
- `sc_service::NewFull` with `BasicAura` block production
- `sc_rpc::create_full` for JSON-RPC
- `BasicLockId` forensic priority for EVM transactions (preserving `evm_context.rs` from salvageable inputs)

**Proof shape**:
```bash
$ cargo build -p myosu-node --release
   Compiling myosu-node v0.1.0
    Finished release [optimized]
$ ./target/release/myosu-node --help
myosu-node 0.1.0
```

### Phase 2B — `myosu-chain-common` Crate

**Goal**: Publish a clean `myosu-chain-common` crate with preserved domain types.

Required files:
- `crates/myosu-chain/common/Cargo.toml`
- `crates/myosu-chain/common/src/lib.rs` — re-exports
- `crates/myosu-chain/common/src/currency.rs` — `Currency` trait, `AlphaCurrency`, `TaoCurrency` (rewritten without `#[freeze_struct]`)
- `crates/myosu-chain/common/src/evm_context.rs` — `is_in_evm()`, `with_evm_context()` (already self-contained, minimal changes)

Key constraint from salvageable inputs:
- `#[freeze_struct]` must be replaced with standard `#[derive(...)]` using `scale-info` + `parity-scale-codec`
- `Currency` trait and `AlphaCurrency`/`TaoCurrency` newtypes encode domain logic that must be preserved

---

## What Is NOT In Phase 2 Scope

| Item | Reason |
|------|--------|
| Custom pallets (game-solver integration) | Phase 3 — after node binary is proven |
| Benchmarking | Requires full node to run |
| `pallet_transaction_payment` | Can be added to runtime when node handles transactions |
| Multi-node network / consensus | Beyond Phase 2 scope |
| 13 subtensor pallets | Requires `subtensor-runtime-common`, `subtensor-macros` workspace keys — blocked until Phase 3+ |

---

## Phase 2 Blocker

None. Phase 1 is complete. Phase 2 can begin immediately.

---

## Decision Points Before Phase 2

1. **Aura vs. Manual Sealing**: Phase 1 has no block production mechanism. The node needs `pallet_aura` or manual sealing. For a dev chain, manual sealing (`pallet_manual_seal`) is simpler and faster to get running. Recommend starting with manual seal.

2. **`myosu-chain` lib.rs surface**: Phase 2 should define what `myosu-chain` re-exports. Currently it's a stub. Decide whether to re-export runtime types, node command, and/or common types.

3. **`CARGO_TARGET_DIR` env var**: The build environment had `CARGO_TARGET_DIR` pointing to a stale worktree cache. Document in team runbook that builds must use a local target dir or unset `CARGO_TARGET_DIR`.
