# Chain Runtime Restart Review

**Lane**: `chain:runtime`
**Date**: 2026-03-19

---

## Restart Justification

### Why the current path cannot be trusted

1. **The workspace does not build `crates/myosu-chain`.** The root `Cargo.toml` explicitly comments it out: `# "crates/myosu-chain" # Stage 1: Substrate chain fork`. It is not compiled, tested, or type-checked in CI.

2. **`surface_check` is a no-op.** `./fabro/checks/chain-runtime-reset.sh` exits 0 with no output. This is not evidence that the runtime builds — it means the check script runs without finding any failure conditions. There is no proof that the runtime compiles.

3. **`runtime/src/lib.rs` imports a dependency graph that does not exist.** The file references `subtensor-runtime-common`, `subtensor-macros`, `subtensor-precompiles`, `subtensor-swap-interface`, `subtensor-transaction-fee`, and 10+ pallet crates — none of which have entries in the workspace `Cargo.toml`. Attempting `cargo build -p myosu-runtime` today would fail with "dependency not found" errors before reaching any code logic.

4. **The Substrate/polkadot-sdk dependency line is broken.** The `game-solver` pallet successfully pins `git = "https://github.com/paritytech/polkadot-sdk", branch = "stable2407"`. This line is not replicated to the runtime. There is no `Cargo.lock` entry establishing which revision of polkadot-sdk the chain targets.

5. **Node is a scaffold with no implementation.** `node/src/main.rs` calls `command::run()`. None of the 10 declared modules (`chain_spec`, `cli`, `client`, `command`, `service`, etc.) exist as files. This is a declared-but-not-defined structure.

6. **The WASM build path is entirely absent.** `runtime/src/lib.rs` has `include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"))` but there is no `build.rs`, no `wasm-builder` configuration, and no WASM compilation job in the workspace.

---

## Salvageable Inputs

These files or patterns from the current state are worth preserving in the restart:

| Input | Location | What to keep |
|-------|----------|--------------|
| EVM context guard | `crates/myosu-chain/common/src/evm_context.rs` | `is_in_evm()` / `with_evm_context()` implementations; `environmental` crate usage pattern. Drop no dependencies. |
| Currency domain types | `crates/myosu-chain/common/src/currency.rs` | `Currency` trait interface; `AlphaCurrency`, `TaoCurrency` as `u64` newtypes. Rewrite without `subtensor_macros::freeze_struct` — use standard `#[derive(...)]` with `scale-info` + `parity-scale-codec`. |
| NetUid type | `crates/myosu-chain/runtime/src/lib.rs` (line 47-130) | `NetUid(u16)` with `CompactAs`, `TypeInfo`, `Display`, `From<u16>` implementations. This is a clean domain type. Rewrite without `freeze_struct`. |
| polkadot-sdk git ref | `crates/myosu-chain/pallets/game-solver/Cargo.toml` (line 22) | `git = "https://github.com/paritytech/polkadot-sdk", branch = "stable2407"` — this is the confirmed working dependency line for the workspace. Must be the basis for all runtime dependencies. |
| Pallet scaffolding | `crates/myosu-chain/pallets/` | 13 pallet directories exist with `Cargo.toml` manifests. These are **not buildable now** but provide a structural template. Each manifest follows a consistent pattern (name, repository field, `subtensor-macros.workspace = true`). They cannot be used until the missing workspace keys are resolved. |
| Edition 2024 | Throughout | All existing `Cargo.toml` files use `edition = "2024"`. The restart should maintain this. |

---

## Inputs That Are Not Salvageable

| Input | Reason |
|-------|--------|
| `runtime/src/lib.rs` `construct_runtime!` block | Imports `pallet_subtensor`, `pallet_shield`, `pallet_subtensor_proxy`, `pallet_subtensor_swap`, `pallet_subtensor_swap_runtime_api`, `pallet_subtensor_utility`, `pallet_subtensor::rpc_info::*`, `pallet_commitments`, `pallet_registry` — none exist as workspace crates |
| `runtime_common::prod_or_fast` macro | No `runtime_common` crate defined anywhere in workspace |
| `subtensor_macros::freeze_struct` | No `subtensor_macros` workspace key; no local implementation |
| `subtensor_precompiles::Precompiles` | No `subtensor_precompiles` crate |
| `subtensor_swap_interface::{Order, SwapHandler}` | No `subtensor_swap_interface` |
| `subtensor_transaction_fee::{SubtensorTxFeeHandler, TransactionFeeHandler}` | No `subtensor_transaction_fee` |
| Node module declarations | `chain_spec`, `cli`, `client`, `command`, `conditional_evm_block_import`, `consensus`, `ethereum`, `mev_shield`, `rpc`, `service` — none have corresponding `.rs` files |
| All 13 pallet crates | Cannot be built without `subtensor-runtime-common`, `subtensor-macros`, `subtensor-swap-interface` workspace keys |

---

## Verdict

**Restart from Phase 0.** The current runtime definition is a forward-port of a subtensor fork that was never fully migrated into this workspace. The workspace dependency graph is incomplete, the node is unimplemented, and the surface check proves nothing.

The restart must begin with workspace wiring, establish a minimal Substrate runtime with `frame_system + pallet_balances + pallet_sudo + pallet_timestamp`, prove it builds to WASM, then layer in the node binary, then the common crate types.
