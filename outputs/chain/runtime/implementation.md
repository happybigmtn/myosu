# Phase 1 Implementation — Myosu Runtime

**Lane**: `chain:runtime`
**Date**: 2026-03-20
**Status**: Phase 1 complete

---

## What Was Built

A minimal Substrate runtime (`myosu-runtime`) compiling to WASM, implementing Phase 0 (workspace wiring) and Phase 1 (minimal runtime) from the restart spec.

### Phase 0 — Workspace Wiring

#### Root `Cargo.toml` changes

- Uncommented `crates/myosu-chain` in workspace members
- Added `crates/myosu-chain/runtime` as a workspace member
- Removed invalid `[patch."https://github.com/paritytech/polkadot-sdk"]` section (patches to the same git source are rejected by Cargo)

#### `crates/myosu-chain/Cargo.toml`

Converted from a nested workspace root (`[workspace]` block) to a regular package with `[lib]`. Nested workspaces are invalid in Cargo; this was the root cause of the "multiple workspace roots found" error.

```toml
[package]
name = "myosu-chain"
[lib]
path = "src/lib.rs"
```

#### `crates/myosu-chain/src/lib.rs`

Created as a stub documentation library. Phase 2 will expose the real API for sub-packages.

### Phase 1 — Minimal Runtime

#### `crates/myosu-chain/runtime/Cargo.toml`

Runtime package manifest with all polkadot-sdk dependencies pinned to `branch = "stable2407"` (matching the `game-solver` pallet):

- `frame-support`, `frame-system`, `frame-executive`
- `pallet-balances`, `pallet-sudo`, `pallet-timestamp`, `pallet-transaction-payment`
- `sp-api`, `sp-runtime`, `sp-core`, `sp-std`, `sp-io`, `sp-version`
- `sp-transaction-pool`, `sp-offchain`, `sp-block-builder`, `sp-inherents`, `sp-session`
- `sp-genesis-builder`, `parity-scale-codec`, `scale-info`
- `substrate-wasm-builder` as `[build-dependencies]`

#### `crates/myosu-chain/runtime/build.rs`

WASM builder invocation using the substrate template pattern:

```rust
fn main() {
    #[cfg(feature = "std")]
    {
        substrate_wasm_builder::WasmBuilder::build_using_defaults();
    }
}
```

#### `crates/myosu-chain/runtime/src/lib.rs`

The runtime definition with four pallets:

| Pallet | Role |
|--------|------|
| `frame_system` | Block header, account, digest, event emit |
| `pallet_balances` | Account balance, transfers, existential deposits |
| `pallet_sudo` | Single-account sudo for development |
| `pallet_timestamp` | Block timestamp for slot scheduling |

Key implementation decisions:

- **`#[derive_impl(...)]`**: Used `frame_system::config_preludes::SolochainDefaultConfig` to derive default associated types (`RuntimeTask`, `SingleBlockMigrations`, `PreInherents`, `PostInherents`, `PostTransactions`, `OnSetCode`) — avoiding the need to define each manually for polkadot-sdk stable2407.
- **Signed extensions**: `CheckNonZeroSender`, `CheckSpecVersion`, `CheckTxVersion`, `CheckGenesis`, `CheckEra`, `CheckNonce`, `CheckWeight` — all from `frame_system::*`, not `sp_runtime::*`.
- **`construct_runtime!` macro**: Auto-generates `RuntimeGenesisConfig`, `AllPalletsWithSystem`, `RuntimeEvent`, `RuntimeCall`, `RuntimeOrigin`, `RuntimeTask`. No manual `impl GenesisBuild` needed — it conflicts with the auto-generated impl.
- **`MinimumPeriod = ConstU64<1000>`**: `pallet_timestamp::Config::MinimumPeriod` requires `Get<u64>`, not `Get<u32>`. Uses `ConstU64` from `frame_support::traits`.
- **`OnTimestampSet = ()`**: The unit type `()` implements `OnTimestampSet` in substrate — valid as a no-op timestamp provider.
- **`decode_session_keys` tuple order**: Return type is `Option<Vec<(Vec<u8>, KeyTypeId)>>` — the tuple is `(key_data, key_type_id)`, not `(key_type_id, key_data)`.

#### `crates/myosu-chain/runtime/src/chain_spec.rs`

Basic development chain spec with:
- `dev_config()` returning `GenesisConfig` with system, balances, sudo, and timestamp configuration
- Sudo key set to `AccountId::from([1u8; 32])`
- No endowed accounts (Phase 1 is minimal)

---

## Build Artifacts

| Artifact | Path |
|----------|------|
| Native lib | `target/release/libmyosu_runtime.rlib` |
| WASM binary | `target/release/wbuild/myosu-runtime/myosu_runtime.wasm` |
| Compact WASM | `target/release/wbuild/myosu-runtime/myosu_runtime.compact.wasm` |
| Compressed WASM | `target/release/wbuild/myosu-runtime/myosu_runtime.compact.compressed.wasm` |

---

## Decisions Made

1. **Removed `TransactionPayment` pallet**: Required `OnChargeTransaction`, `WeightToFee`, `OperationalFeeMultiplier` — added complexity for no benefit in Phase 1. Can be added back in a later phase.

2. **Removed `pallet_aura`**: Phase 1 uses a minimal runtime without block production. Aura (and its `MinimumPeriodTimesTwo`) will be added when the node binary is built (Phase 2).

3. **No custom pallets**: Phase 1 spec called for System + Balances + Sudo + Timestamp only. Custom pallets (e.g., `game-solver`) are out-of-scope.

4. **`Migrations = ()`**: No migrations defined. The `Executive` type accepts a migrations tuple; Phase 1 uses the unit type (no-op).

5. **`CARGO_TARGET_DIR` override**: The environment had `CARGO_TARGET_DIR` pointing to a different worktree's build cache, causing wasm-builder path mismatches. The full WASM build required `CARGO_TARGET_DIR=/path/to/worktree/target` to succeed. This is a local environment issue, not a code issue.
