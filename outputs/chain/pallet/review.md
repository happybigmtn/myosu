# Chain Pallet Restart Review

**Lane**: `chain:pallet`
**Date**: 2026-03-19

---

## Restart Justification

### Why the current path cannot be trusted

1. **`cargo check -p pallet-game-solver` produces 50+ unique errors.** The first wave of failures are missing crates (`subtensor_runtime_common`, `subtensor_macros`, `subtensor_swap_interface`, `safe_math`, `log`, `substrate_fixed`). The second wave is broken API shapes (`codec` crate doesn't exist, `DispatchGuard` removed from `frame_support`, `AsSystemOriginSigner`/`TransactionExtension` removed from `sp_runtime::traits`). The third wave is type mismatches (the pallet's `Config` trait references types that don't exist). This is not a surface-level import fix — every module has embedded subtensor workspace-key dependencies that must be surgically removed.

2. **The `lib.rs` is a forward-port of a subtensor pallet, not a Myosu pallet.** The `Config` trait declares trait bounds for `subtensor_swap_interface::GetAlphaForTao`, `pallet_balances`, `CommitmentsInterface`, and dozens of other types that do not exist in the workspace. The 80+ storage items include Alpha/TAO dual-currency accounting, subnet leasing, voting power, and EVM address maps — none of which are Myosu game-solving requirements.

3. **`epoch/math.rs` depends on `safe_math` which is not a published crate in this workspace.** The file uses `safe_div` extension methods on `I32F32`/`I64F64` that are provided by a `safe_math` crate. This crate is not in `Cargo.toml` and is not the same as the `substrate-fixed` library already in use. The fix is non-trivial: requires either publishing a local `safe_math` crate or replacing all `safe_div` calls with inline checked arithmetic.

4. **`extensions/subtensor.rs` uses sp_runtime API removed in stable2407.** `AsSystemOriginSigner`, `Implication`, `TransactionExtension`, and `impl_tx_ext_default` do not exist in `sp_runtime::traits` on the polkadot-sdk `stable2407` branch. This module cannot be patched — it must be removed.

5. **The pallet has 36 one-time migration files** (`src/migrations/migrate_*.rs`) that encode subtensor's upgrade history. These are dead code for Myosu and carry the same broken imports. They are a maintenance burden that should be stripped.

---

## Salvageable Inputs

These files or patterns from the current state are worth preserving in the restart:

| Input | Location | What to keep |
|-------|----------|--------------|
| `stubs.rs` | `src/stubs.rs` | `ProxyStub`, `CommitmentsStub`, `AuthorshipStub`, `ColdkeySwapStub` implementations. Already self-contained and tested. |
| `swap_stub.rs` | `src/swap_stub.rs` | `NoOpSwap`, `SwapHandler`, `SwapEngine`, `SwapBalance` traits and implementations. Correctly models Myosu's single-token identity-swap. |
| `benchmarks.rs` | `src/benchmarks.rs` | FRAME benchmark scaffolding; may be useful for future performance work |
| pallet structure | `lib.rs` (top-level) | FRAME pallet conventions: `#[pallet]`, `#[import_section]`, storage items pattern, `Config` trait shape |
| polkadot-sdk git ref | `Cargo.toml` line 22 | `git = "https://github.com/paritytech/polkadot-sdk", branch = "stable2407"` — confirmed working |
| `AxonInfo` data type | `lib.rs` lines 140-159 | Self-contained struct; no missing deps; useful for serving |
| `PrometheusInfo` data type | `lib.rs` lines 196-208 | Self-contained struct; no missing deps |
| `NeuronCertificate` | `lib.rs` lines 163-189 | Self-contained; implements `TryFrom<Vec<u8>>`; no missing deps |
| `RateLimitKey` enum | `lib.rs` lines 2650-2673 | Domain type for rate limiting; clean SCALE encoding; no missing deps once `NetUid` is resolved |
| module tree | subdirectories | The directory structure (`staking/`, `subnets/`, `guards/`, `epoch/`, `utils/`) is a reasonable organization for the pallet's concerns. The implementation inside needs rewrite; the structure is sound. |
| `Dispatchable` trait bounds | `lib.rs` | `sp_runtime::traits::{Dispatchable, TrailingZeroInput}` usage is correct and on the right `stable2407` API |
| `BTreeMap`/`BTreeSet` usage | Throughout | Correct use of `sp_std::collections::btree_{map,set}`; these types are available |
| `VecDeque` usage | `lib.rs` | Correct use of `sp_std::collections::vec_deque::VecDeque` |
| `H160`/`H256` usage | `lib.rs` | `sp_core::{H160, H256}` types are available in the dep tree |

---

## Inputs That Are Not Salvageable

| Input | Reason |
|-------|--------|
| `src/extensions/subtensor.rs` | Uses `sp_runtime` APIs removed in stable2407; must be deleted |
| `src/migrations/` (36 files) | One-time subtensor upgrade scripts; `subtensor_runtime_common` imports throughout; must be deleted |
| `src/rpc_info/` (7 files) | Runtime API surface; `subtensor_runtime_common` imports throughout; must be deleted |
| `src/swap/` (3 files) | Subtensor AMM; `subtensor_swap_interface` imports throughout; must be deleted |
| `src/coinbase/` (3 files) | Emission distribution; `subtensor_swap_interface` + `subtensor_runtime_common` imports; must be deleted |
| `src/epoch/math.rs` | `safe_math` crate missing; must be rewritten with local `SafeDiv` trait |
| `src/utils/rate_limiting.rs` | `subtensor_runtime_common::NetUid` import; `NetUid` must be defined locally |
| All `use codec::` imports | `codec` crate does not exist; must be `parity_scale_codec` |
| `use pallet_balances::Call as BalancesCall` | `pallet_balances` not in deps; call encoding is subtensor-specific |
| `use subtensor_runtime_common::*` (throughout) | No such crate in workspace |
| `use subtensor_macros::freeze_struct` | No such crate in workspace |
| `use subtensor_swap_interface::*` (throughout) | No such crate in workspace |
| `use runtime_common::prod_or_fast` | No such crate in workspace |
| `use safe_math::*` in epoch/math.rs | No such crate in workspace |
| `use substrate_fixed::types::{I64F64, I96F32, U64F64, U96F32}` in lib.rs | Not in deps; `I32F32`/`I64F64` available via `substrate-fixed` package |
| `#[crate::freeze_struct("...")]` proc macro calls | `subtensor_macros` not in workspace; replace with standard derives |
| Full 80+ storage item set | Alpha/TAO accounting, subnet leasing, voting power, EVM maps not needed for Myosu game-solving |
| `SubtensorInfo` trait impl | `subtensor_runtime_common::SubnetInfo` not available |
| `BalanceOps` trait impl | `subtensor_runtime_common::BalanceOps` not available |
| `CurrencyReserve` impls | `subtensor_runtime_common::CurrencyReserve` not available |

---

## Verdict

**Restart from Phase 1.** The pallet is a subtensor fork with subtensor workspace-key dependencies embedded at every level — in imports, in trait bounds, in storage items, and in 36 migration files. The workspace keys (`subtensor_runtime_common`, `subtensor_macros`, `subtensor_swap_interface`) were never defined. The `epoch/math.rs` depends on a `safe_math` crate that does not exist. The `extensions/subtensor.rs` uses polkadot-sdk APIs that were removed in `stable2407`.

**Salvageable**: `stubs.rs`, `swap_stub.rs`, `benchmarks.rs`, the pallet module structure, `AxonInfo`/`PrometheusInfo`/`NeuronCertificate` data types, `RateLimitKey` enum, and the confirmed `polkadot-sdk stable2407` dependency line in `Cargo.toml`.

**Not salvageable**: Everything that imports a `subtensor_*` workspace key, the `safe_math` dependency in `epoch/math.rs`, the `extensions/subtensor.rs` file, all 36 migration files, all RPC info files, all swap files, all coinbase files, and the majority of the 80+ storage items that encode Alpha/TAO dual-currency state.

The implementation lane is **unblocked after bootstrap** — Phase 1 is purely mechanical (add deps + delete broken modules + fix imports). There are no architectural decisions to make before starting.
