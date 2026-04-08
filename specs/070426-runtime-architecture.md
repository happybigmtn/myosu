# Specification: Runtime Architecture & Pallet Surface

## Objective

Describe the current Substrate runtime composition, pallet surface, feature flags, SDK dependency, and the naming alias that maps the active `pallet-game-solver` crate to the `pallet_subtensor` symbol used throughout the runtime.

## Evidence Status

### Verified (code-grounded)

- The runtime crate is `myosu-chain-runtime` at `crates/myosu-chain/runtime/src/lib.rs`.
- The runtime Cargo.toml (`crates/myosu-chain/runtime/Cargo.toml:122`) declares:
  `pallet_subtensor = { package = "pallet-game-solver", path = "../pallets/game-solver" }`.
  This means the `SubtensorModule` symbol in `construct_runtime!` resolves to the `pallet-game-solver` crate, not the dead `pallet-subtensor` crate.
- `construct_runtime!` has two conditional variants gated on `full-runtime`:
  - **Full runtime** (`lib.rs:1240`): System(0), RandomnessCollectiveFlip(1), Timestamp(2), Aura(3), Grandpa(4), Balances(5), TransactionPayment(6), SubtensorModule(7), Utility(11), Sudo(12), Multisig(13), Preimage(14), Scheduler(15), Proxy(16), AdminUtils(19), SafeMode(20).
  - **Default (fast) runtime** (`lib.rs:1266`): System(0), Timestamp(2), Aura(3), Grandpa(4), Balances(5), TransactionPayment(6), SubtensorModule(7), Utility(11), AdminUtils(19).
- Polkadot SDK is pinned to the opentensor fork at rev `71629fd93b6c12a362a5cfb6331accef9b2b2b61` (`Cargo.toml:66`).
- Feature flags (`runtime/Cargo.toml:14-41`):
  - `fast-runtime`: empty (reduces AURA slot from 6s to 1s via `prod_or_fast!` macro).
  - `fast-blocks`: implies `fast-runtime`.
  - `full-runtime`: enables Sudo, Multisig, Preimage, Scheduler, Proxy, SafeMode, RandomnessCollectiveFlip.
  - `runtime-benchmarks`: implies `full-runtime` and enables all benchmark pallets.
  - `pow-faucet`: empty feature gate.
- Consensus: AURA (Authority Round) with GRANDPA finality. Max 32 authorities (`lib.rs:1228`).
- Block time: 6 seconds production, 1 second fast mode (via `runtime_common::prod_or_fast`).
- Existential deposit: 500 RAO (`subtensor-runtime-common`).
- Workspace edition: 2024 (`Cargo.toml:21`).
- Workspace version: 0.1.0 (`Cargo.toml:20`).
- 12 workspace members declared in root `Cargo.toml:3-17`.
- `substrate-fixed` v0.6.0 (encointer fork) for U96F32/I96F32 fixed-point arithmetic.
- Clippy workspace lints deny `arithmetic-side-effects`, `expect-used`, `indexing-slicing`, `unwrap-used` (`Cargo.toml:25-32`).
- `Stage0NoopSwap` (`lib.rs:99-107`) implements identity swap (1:1 TAO↔Alpha, zero fees). `default_price_limit()` returns `C::MAX`.
- Transaction extensions (`lib.rs:1291-1301`): CheckNonZeroSender, CheckSpecVersion, CheckTxVersion, CheckGenesis, CheckEra, CheckNonce, CheckWeight, ChargeTransactionPayment, CheckMetadataHash.
- Migrations (`lib.rs:1303-1309`): Only `migrate_init_total_issuance` runs on every upgrade.
- Genesis JSON builder (`lib.rs:1343-1380`) hardcodes Alice AURA authority and Alice+Bob balance allocation.
- Runtime APIs: Core, Metadata, BlockBuilder, TaggedTransactionQueue, OffchainWorkerApi, AuraApi, SessionKeys, GrandpaApi, TransactionPaymentApi, TransactionPaymentCallApi, GenesisBuilder, Benchmark, SubtensorCustomApi.

### Recommendations (intended future direction)

- Plan 003 (pallet naming normalization) recommends eliminating the `pallet_subtensor` alias so the symbol matches `pallet-game-solver` throughout.
- Plan 002 (dead pallet removal) recommends deleting `crates/myosu-chain/pallets/subtensor/` entirely — it is a 92.4K-line dead crate not used by the runtime.
- Plan 015 (SDK migration research) recommends classifying the 21 opentensor polkadot-sdk fork commits to determine which are myosu-needed vs subtensor-specific.
- The `Stage0NoopSwap` must be replaced before any public network launch per the inline comment and plan 014 (token economics research gate).

### Hypotheses / Unresolved

- Whether indices 8, 9, 10, 17, 18 (gaps in construct_runtime) can be reclaimed or must be preserved for storage compatibility is not documented.
- Whether `pow-faucet` feature flag has any consumers.

## Acceptance Criteria

- The `construct_runtime!` macro produces a valid runtime with the documented pallet indices for both full and default feature sets
- `cargo check -p myosu-chain-runtime` succeeds with default features
- `cargo check -p myosu-chain-runtime --features full-runtime` succeeds
- `cargo check -p myosu-chain-runtime --features runtime-benchmarks` succeeds
- `Stage0NoopSwap::swap()` returns `amount_paid_in == amount_paid_out` (identity) with zero fees
- `Stage0NoopSwap::default_price_limit()` returns `C::MAX` for any currency type
- All workspace clippy lints (`arithmetic-side-effects`, `expect-used`, `indexing-slicing`, `unwrap-used`) are enforced as `deny`
- AURA slot duration is 6 seconds without `fast-runtime` and 1 second with it
- GRANDPA max authorities is 32

## Verification

```bash
# Runtime compiles with all feature combinations
cargo check -p myosu-chain-runtime
cargo check -p myosu-chain-runtime --features full-runtime
cargo check -p myosu-chain-runtime --features runtime-benchmarks
cargo check -p myosu-chain-runtime --features fast-runtime

# Confirm the pallet_subtensor alias resolves to game-solver
grep 'pallet_subtensor.*package.*pallet-game-solver' crates/myosu-chain/runtime/Cargo.toml

# Workspace clippy lints are enforced
grep -A5 'workspace.lints.clippy' Cargo.toml

# Verify no direct dependency on the dead pallet-subtensor from the runtime
! grep 'path.*pallets/subtensor' crates/myosu-chain/runtime/Cargo.toml
```

## Open Questions

- Can pallet indices 8-10, 17-18 be safely reused, or does storage key derivation bind them permanently?
- Is the `pow-faucet` feature gate used anywhere? If not, should it be removed?
- What is the upgrade path for `Stage0NoopSwap` — will it be a runtime migration or a hard fork?
- The opentensor polkadot-sdk fork pin is a specific commit, not a release tag. What is the upstream update strategy?
