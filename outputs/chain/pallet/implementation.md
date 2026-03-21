slice: phase_1_fix_deps_and_strip_non_myosu_modules
status: complete

## Contract
- Source of truth: `outputs/chain/pallet/spec.md`
- Source of truth: `outputs/chain/pallet/review.md`
- Scope held to the Phase 1 restart boundary: make `pallet-game-solver` build as a reduced Myosu core without subtensor workspace-key dependencies.

## Changes
- Replaced the transplanted `crates/myosu-chain/pallets/game-solver/src/lib.rs` body with a compact FRAME pallet shell, local `NetUid` and `Balance` aliases, and the salvageable `AxonInfo`, `PrometheusInfo`, and `NeuronCertificate` domain types.
- Re-exported the retained Phase 1 helper surfaces from the crate root so downstream work can build against `RateLimitKey`, `NoOpSwap`, and the local proxy, commitment, authorship, and coldkey-swap interfaces.
- Moved the local proxy, commitment, and authorship trait definitions into `crates/myosu-chain/pallets/game-solver/src/stubs.rs` so the reduced pallet no longer depends on the old macro-driven config surface.
- Added `crates/myosu-chain/pallets/game-solver/src/utils/rate_limiting.rs` and updated `crates/myosu-chain/pallets/game-solver/src/utils/mod.rs` to keep a local SCALE-encoded `RateLimitKey` enum backed only by `NetUid`.
- Added `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs` and updated `crates/myosu-chain/pallets/game-solver/src/epoch/mod.rs` with a local `SafeDiv` trait over `substrate-fixed` fixed-point types.
- Tightened `crates/myosu-chain/pallets/game-solver/Cargo.toml` so `substrate-fixed` and `log` stay `no_std`-friendly and the crate advertises a `try-runtime` feature expected by FRAME macros.

## Deliberate Deferrals
- `coinbase/`, `extensions/`, `migrations/`, `rpc_info/`, `staking/`, `subnets/`, and `swap/` remain stripped module boundaries for later approved slices.
- Storage items, dispatchables, registration flow, serving flow, staking logic, and subnet logic are deferred to Phase 2 and Phase 4 as described in the reviewed lane artifacts.
