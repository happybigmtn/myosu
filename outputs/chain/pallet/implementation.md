# Chain Pallet Implementation

Date: 2026-03-20
Lane: `chain:pallet`
Slice: `Phase 1 restart scaffolding`

## Implemented in this slice

This implementation takes the smallest approved restart cut that makes
`pallet-game-solver` compile as a Myosu-owned pallet core again.

- Replaced the forward-ported subtensor `lib.rs` with a minimal FRAME pallet core.
- Added local Myosu-owned domain types:
  - `NetUid = u16`
  - `Balance = u64`
  - `Currency` + `SingleTokenCurrency`
  - `Hyperparameter`
  - `RateLimitKey`
  - preserved `AxonInfo`, `PrometheusInfo`, and `NeuronCertificate`
- Restored only the storage needed by the stripped rate-limit helpers:
  - `RateLimitedLastBlock`
  - `NetworkLastLockBlock`
- Kept the approved salvageable modules in build:
  - `stubs.rs`
  - `swap_stub.rs`
  - a reduced `utils/rate_limiting.rs`
- Stripped the broken subtensor-forwarded surfaces from the active build by
  narrowing the module graph:
  - `macros/` now exposes only the trait definitions used by `stubs.rs`
  - `guards/check_coldkey_swap.rs` is now a local no-op guard
  - `staking/mod.rs` and `subnets/mod.rs` are reduced restart surfaces
  - `epoch/math.rs` is a local checked-arithmetic surface for the restart cut
  - `epoch/run_epoch.rs`, `extensions/`, `migrations/`, `rpc_info/`, `swap/`,
    and `coinbase/` are no longer on the active compile path
- Updated `Cargo.toml` to add `log` and declare local `runtime-benchmarks` /
  `try-runtime` features for the stripped crate shape.
- Preserved the downstream-owned handoff artifacts as file surfaces:
  - `outputs/chain/pallet/quality.md`
  - `outputs/chain/pallet/promotion.md`
  These are carried as empty files in this slice so later gates can take
  ownership and overwrite them.

## Notes

- The reviewed spec called for `parity-scale-codec` `encode` / `decode` feature
  flags, but the workspace lock resolves to `3.7.5`, which does not expose
  those features. The manifest keeps the lock-compatible `derive` feature.
- The fixed-point `substrate-fixed` reintroduction is still deferred. This slice
  keeps `epoch/math.rs` as a local checked-arithmetic restart surface so the crate
  compiles cleanly at the Phase 1 boundary without pulling the old subtensor
  math back into scope.

## Files touched

- `crates/myosu-chain/pallets/game-solver/Cargo.toml`
- `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- `crates/myosu-chain/pallets/game-solver/src/macros/mod.rs`
- `crates/myosu-chain/pallets/game-solver/src/macros/config.rs`
- `crates/myosu-chain/pallets/game-solver/src/guards/check_coldkey_swap.rs`
- `crates/myosu-chain/pallets/game-solver/src/utils/mod.rs`
- `crates/myosu-chain/pallets/game-solver/src/utils/rate_limiting.rs`
- `crates/myosu-chain/pallets/game-solver/src/epoch/mod.rs`
- `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs`
- `crates/myosu-chain/pallets/game-solver/src/staking/mod.rs`
- `crates/myosu-chain/pallets/game-solver/src/subnets/mod.rs`
