# Phase 1 Quality — Myosu Runtime

**Lane**: `chain:runtime`
**Date**: 2026-03-20

---

## Code Quality

### Clean Compile
- `SKIP_WASM_BUILD=1 cargo build -p myosu-runtime --release --lib` — zero errors, zero warnings on our code
- The only warnings are `unused` hints from `trie-db` (a transitive dependency), not our code

### Dependency Integrity
- All polkadot-sdk dependencies pinned to `branch = "stable2407"` — the same ref confirmed working in `game-solver` pallet
- No version drift between Substrate/Polkadot crates — all pulled from the same git commit
- `Cargo.lock` entries for all git dependencies provide reproducible builds

### Pattern Fidelity
The runtime follows the official Substrate solochain template (`templates/solochain/runtime/src/lib.rs`) patterns:
- `construct_runtime!` with `pub enum Runtime` — standard visibility
- `#[derive_impl]` for system config defaults — matches substrate template
- `Executive` type using `frame_executive::Executive<..., AllPalletsWithSystem, Migrations>` — correct type order
- `impl_runtime_apis!` block with all 6 required trait implementations — Core, Metadata, BlockBuilder, TaggedTransactionQueue, OffchainWorkerApi, SessionKeys
- `SignedExtra` tuple uses `frame_system::*` extensions, not `sp_runtime::*` — correct source
- `parameter_types!` macro used consistently for `BlockWeights`, `BlockLength`, `ExistentialDeposit`, `Version`

### Type Safety
- All associated types in `frame_system::Config` are explicitly provided — no relying on implicit defaults where explicit is better
- `AccountId = sp_runtime::AccountId32`, `Balance = u64`, `Hash = sp_core::H256` — standard Substrate types
- `Block = generic::Block<Header, UncheckedExtrinsic>` — correct Substrate block type alias
- `UncheckedExtrinsic` uses `SignedExtra` via the `generic::UncheckedExtrinsic<Address, RuntimeCall, SignedSignature, SignedExtra>` pattern

### Limitations (Phase 1 scope)

| Limitation | Rationale |
|------------|-----------|
| No Aura/consensus | Phase 1 is runtime-only; block production belongs to Phase 2 (node binary) |
| No TransactionPayment pallet | Not needed for WASM build verification; adds `WeightToFee`/`OnChargeTransaction` complexity |
| No genesis account endowments | `chain_spec.rs` has empty balances; Phase 2 node will provide full spec |
| No session pallet | No staking/validator set in Phase 1 |
| No sudo origin check beyond type | `pallet_sudo::Config` uses only `type RuntimeEvent` and `type RuntimeCall` — minimal for Phase 1 |
| No benchmarking | Benchmarking weights require a full node to run; `pallet_timestamp::weights::()` is a placeholder |

---

## Code Style

- `PascalCase` for types, traits, enum variants
- `SCREAMING_SNAKE_CASE` for constant types (`NORMAL_DISPATCH_RATIO`, `ExistentialDeposit`)
- `snake_case` for functions, variables
- Module-level doc comments (`//!`) on lib.rs and build.rs
- No commented-out code — removed dead code rather than preserving it

---

## Architectural Decisions

### Why `#[derive_impl]` over manual impl

polkadot-sdk stable2407's `frame_system::Config` has many new associated types vs older Substrate versions:
`RuntimeTask`, `SingleBlockMigrations`, `MultiBlockMigrator`, `PreInherents`, `PostInherents`, `PostTransactions`, `OnSetCode`, `Nonce`, `Hash`, `BlockHashCount`, `DbWeight`, `AccountData`, `Block`, `BlockWeights`, `BlockLength`, `SS58Prefix`, `MaxConsumers`.

Writing each manually is error-prone and verbose. `#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]` provides sensible defaults for all of them, and we override only the types we explicitly need (`Block`, `BlockWeights`, `BlockLength`, `AccountId`, `Nonce`, `Hash`, `BlockHashCount`, `DbWeight`, `Version`, `AccountData`, `SS58Prefix`, `MaxConsumers`).

### Why not use `pallet_transaction_payment`

`TransactionPayment` requires `WeightToFee` (a concrete fee calculation type) and `OnChargeTransaction` (a trait for extracting fees from accounts). In Phase 1, these are distractions — the runtime compiles without them. The pallet can be added back in Phase 2 when the node binary is running actual transactions.

### Why `GenesisConfig` auto-generation is sufficient

`construct_runtime!` auto-generates `RuntimeGenesisConfig` and implements `BuildGenesisConfig` for it. The `chain_spec.rs` uses the auto-generated `GenesisConfig` struct (aliased from `RuntimeGenesisConfig` by the macro). No manual `impl GenesisBuild` is needed — the auto-impl also handles storage genesis building.
