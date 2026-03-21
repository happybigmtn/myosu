# Chain Pallet Implementation

slice: phase_1_restart_boundary
date: 2026-03-20

## Implemented State

- `crates/myosu-chain/pallets/game-solver/Cargo.toml` now carries the reduced dependency set the restarted pallet actually uses, including `parity-scale-codec`, `substrate-fixed`, `log`, and a declared `try-runtime` feature for FRAME macro cfg compatibility.
- `crates/myosu-chain/pallets/game-solver/src/lib.rs` exposes only the live restart surfaces, `epoch` and `macros`, plus local `NetUid` and `Balance` aliases for the single-token Myosu model.
- The live pallet body is reduced to a self-contained FRAME shell: `Config`, storage version, one event/error section, empty genesis build, zero-weight hooks, and no active dispatchables.
- `crates/myosu-chain/pallets/game-solver/src/macros/config.rs` replaces missing subtensor workspace-key bounds with local trait definitions for swap, proxy, commitments, authorship, and balance typing so the pallet config compiles without `subtensor_runtime_common`, `subtensor_macros`, or `subtensor_swap_interface`.
- `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs` carries a local `SafeDiv` trait for `I32F32`, `I64F64`, and `usize`, removing the missing `safe_math` crate from the live compile path while keeping the epoch math helpers available.
- Legacy forked directories such as `coinbase`, `extensions`, `migrations`, `rpc_info`, `staking`, `subnets`, `swap`, `utils`, and the old test tree remain on disk, but this slice keeps them outside the active module tree so they do not participate in the pallet build.
- Settlement cleanup for this lane added a pallet-local dead-code allowance on the generated FRAME pallet module so the skeletal event helper does not leave local warning debt while the pallet has no live calls yet.

## Slice Boundary

- This slice delivers the reviewed restart boundary from `outputs/chain/pallet/spec.md`: `cargo check -p pallet-game-solver` now succeeds against the reduced Myosu-specific core.
- This slice does not yet restore the minimal storage maps, registration/serving calls, or staking/subnet logic described for the following phase.
