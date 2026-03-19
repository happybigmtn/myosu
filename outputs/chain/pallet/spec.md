# Chain Pallet Restart Spec

**Lane**: `chain:pallet`
**Date**: 2026-03-19
**Status**: Restart required

---

## 1. Current State Inventory

### 1.1 What exists

| Surface | Path | Content | Buildable? |
|---------|------|---------|-------------|
| Pallet crate | `crates/myosu-chain/pallets/game-solver/` | FRAME pallet (~2700 lines in `lib.rs`); 11 submodules; `frame_support`/`frame_system` from polkadot-sdk | **No** — see §1.2 |
| `Cargo.toml` | `crates/myosu-chain/pallets/game-solver/` | Minimal deps: `frame-support`, `frame-system`, `sp-runtime`, `sp-core`, `sp-std`, `sp-io`, `parity-scale-codec`, `scale-info` — pinned to `polkadot-sdk stable2407` | **Yes** — the manifest itself is buildable |
| Stub implementations | `src/stubs.rs` | No-op implementations of `ProxyInterface`, `CommitmentsInterface`, `AuthorshipProvider`, `CheckColdkeySwap` for subtensor trait deps | **Yes** — in `src/`, self-contained |
| Swap stub | `src/swap_stub.rs` | No-op `SwapHandler`, `SwapEngine`, `NoOpSwap` for subtensor AMM | **Yes** — in `src/`, self-contained |
| Benchmark module | `src/benchmarks.rs` | FRAME benchmark scaffolding | Not verified |

### 1.2 Why the pallet does not build

`cargo check -p pallet-game-solver` fails with 50+ unique error types. The broken surfaces are:

**`lib.rs` (top-level)**:
- `use codec::{Decode, Encode}` → `codec` is not a declared crate; `parity-scale-codec` is aliased as `parity_scale_codec` in scope
- `use pallet_balances::Call as BalancesCall` → `pallet_balances` not in deps
- `use subtensor_runtime_common::{AlphaCurrency, TaoCurrency, NetUid, Currency, CurrencyReserve, ...}` → `subtensor_runtime_common` not in workspace
- `use subtensor_macros::freeze_struct` → `subtensor_macros` not in workspace
- `use runtime_common::prod_or_fast` → no `runtime_common` crate
- `use substrate_fixed::types::{I64F64, I96F32, U64F64, U96F32}` → `substrate_fixed` not in deps
- `pub use pallet::*` → `pallet` is not a re-exported module name in frame

**`src/utils/rate_limiting.rs`**:
- `use subtensor_runtime_common::NetUid` → same missing dep

**`src/epoch/math.rs`**:
- `use safe_math::*` → `safe_math` is not a crate in workspace
- `use substrate_fixed::transcendental::{exp, ln}` → `substrate_fixed` missing
- `use substrate_fixed::types::{I32F32, I64F64}` → same

**`src/extensions/subtensor.rs`**:
- `use sp_runtime::traits::{AsSystemOriginSigner, Implication, TransactionExtension, ValidateResult}` → none of these exist in `sp_runtime::traits` on polkadot-sdk `stable2407`
- `use sp_runtime::impl_tx_ext_default` → not in root
- `use frame_support::dispatch::DispatchGuard` → not in `frame_support::dispatch`
- `use subtensor_macros::freeze_struct` → missing
- `use subtensor_runtime_common::{NetUid, NetUidStorageIndex}` → missing
- `use codec::{Decode, DecodeWithMemTracking, Encode}` → wrong crate name

**`src/guards/check_coldkey_swap.rs`**:
- `use frame_support::dispatch::DispatchGuard` → missing

**`src/migrations/*.rs`** (10 files):
- `use codec::{Decode, Encode}` → wrong crate name
- `use subtensor_runtime_common::NetUid` → missing

**`src/rpc_info/*.rs`** (7 files):
- Same `codec` + `subtensor_runtime_common` import failures

**`src/staking/*.rs`** (10 files):
- `use subtensor_swap_interface::GetAlphaForTao` / `GetTaoForAlpha` → missing
- `use subtensor_runtime_common::Currency` → missing

**`src/subnets/*.rs`** (6 files):
- Same missing deps

**`src/swap/*.rs`** (3 files):
- `use subtensor_swap_interface::...` → missing

**`src/coinbase/*.rs`** (3 files):
- Same missing deps

---

## 2. Restart Boundary

The restart begins at **the point where `cargo check -p pallet-game-solver` exits 0** after reducing the pallet to its Myosu-specific core.

The boundary is defined by **what the pallet needs to do for Myosu** versus what it was carrying from the subtensor fork:

### In-scope for Myosu (keep / rewrite):
- FRAME pallet boilerplate (`#[pallet]` macro, `Config` trait, `Pallet<T>` struct, storage items)
- The `Owner`, `Keys`, `Axons`, `Prometheus` storage maps and their associated data types (`AxonInfo`, `PrometheusInfo`)
- Basic subnet registration and serving logic
- Stake tracking (owner relationships, delegate takes)
- Rate limiting infrastructure
- Epoch/mechanism math (requires `substrate-fixed` + `safe_math` replacement)
- The `stubs.rs` and `swap_stub.rs` files already present in `src/`

### Out-of-scope for Myosu (strip or stub aggressively):
- Alpha/TAO dual-currency system (`AlphaCurrency`, `TaoCurrency`, `CurrencyReserve`)
- Subtensor swap/AMM interface (`SwapHandler`, `SwapEngine`, `subtensor_swap_interface`)
- Subtensor commitment system (`pallet_commitments`)
- EVM integration (`AssociatedEvmAddress`)
- Subnet leasing and Subtoken system
- Root claim / root network special-case logic
- Voting power tracking
- All 36 migration files (one-time subtensor upgrade scripts)
- All RPC info module (runtime API surface)
- `pallet_balances` integration at the call level
- `substrate_fixed` fixed-point types (only needed for epoch math — can be added as a dep)

---

## 3. Required Manifest Changes (Phase 0)

Before any source changes, `Cargo.toml` must add:

```toml
# Missing deps that lib.rs transitively requires
substrate-fixed = { git = "https://github.com/paritytech/polkadot-sdk", branch = "stable2407", package = "fixed" }
log = "0.4"
parity-scale-codec = { version = "3.6.12", default-features = false, features = ["derive", "encode", "decode"] }
```

And replace `codec` with `parity_scale_codec` in the `std` feature (the `codec` crate is an old alias that was removed from polkadot-sdk).

---

## 4. Phase Definitions

### Phase 1: Fix Deps + Strip Non-Myosu Modules

**Goal**: `cargo check -p pallet-game-solver` exits 0 with a reduced pallet that has no subtensor workspace-key dependencies.

**Actions**:
1. Add `substrate-fixed` and `log` to `Cargo.toml`
2. Replace all `use codec::` with `use parity_scale_codec::` throughout
3. Delete or empty these modules entirely:
   - `src/extensions/` — strip to empty `mod extensions;` (transaction extensions are polkadot-sdk specific and broken)
   - `src/migrations/` — strip to empty `mod migrations;` (one-time upgrade scripts, not needed for Myosu genesis)
   - `src/rpc_info/` — strip to empty `mod rpc_info;` (runtime API surface, not needed for pallet core)
   - `src/swap/` — strip to empty `mod swap;` (AMM is subtensor-specific)
   - `src/coinbase/` — strip to empty `mod coinbase;` (emission distribution, not needed yet)
4. Stub `src/epoch/math.rs` to use only `substrate_fixed` types with no `safe_math` crate — implement a local `SafeDiv` extension trait on `I32F32`/`I64F64` inside `math.rs`
5. Fix `src/utils/rate_limiting.rs`: remove `use subtensor_runtime_common::NetUid`, replace `NetUid` with a local `u16` newtype wrapper
6. Fix `src/guards/check_coldkey_swap.rs`: remove `DispatchGuard` import, use only local types
7. Fix `src/lib.rs`: remove all `subtensor_runtime_common`, `subtensor_macros`, `runtime_common`, `pallet_balances` imports; replace `codec` with `parity_scale_codec`
8. Fix `src/macros/dispatches.rs`, `src/macros/config.rs`, `src/macros/hooks.rs`, `src/macros/genesis.rs`, `src/macros/events.rs` — strip any `subtensor_runtime_common`, `runtime_common`, `pallet_balances` usage
9. Fix `src/staking/mod.rs` and all staking submodules — remove `subtensor_swap_interface` trait bounds
10. Fix all remaining submodules (`subnets/`, `staking/`, `guards/`) — remove any remaining `subtensor_*` workspace key references

**Stub types to add locally** (in `lib.rs` or a new `src/types.rs`):
```rust
// Local NetUid replacement — used everywhere as a subnet ID
pub type NetUid = u16;

// Local currency types — single-token model, no Alpha/TAO split
pub type Balance = u64;

// Local currency trait — minimal version of subtensor's Currency trait
pub trait Currency {
    type Balance;
    fn zero() -> Self::Balance;
    fn saturating_add(a: Self::Balance, b: Self::Balance) -> Self::Balance;
    fn saturating_sub(a: Self::Balance, b: Self::Balance) -> Self::Balance;
}
```

**Proof shape**:
```
$ cargo check -p pallet-game-solver
   Compiling pallet-game-solver v0.1.0
    Finished dev [unoptimized]
```

---

### Phase 2: Restore Core Storage + Config

**Goal**: Restore the minimal `Config` trait and storage items needed for a Myosu-specific game-solving pallet.

**Actions**:
1. Rewrite `src/macros/config.rs` to define `Config` with only Myosu-relevant bounds:
   - `type RuntimeEvent`, `type RuntimeOrigin`, `type AccountId`, `type Hash`
   - `type Balance` (single token, no dual-currency)
   - Remove all `SubtensorSwapInterface`, `SwapHandler`, `CommitmentsInterface`, `pallet_balances` trait bounds
2. Restore storage items in `lib.rs` that are actually needed:
   - `Keys<T>` — hotkey → UID mapping (needed for game solving)
   - `Axons<T>` — axon info per hotkey (needed for serving)
   - `Owner<T>` — hotkey → coldkey ownership (needed for stake)
   - `Delegates<T>` — hotkey delegation take (needed for stake)
   - `SubnetOwner<T>` — subnet ownership (needed for registration)
   - Minimal subset of the 80+ storage items (do not restore all)
3. Restore `AxonInfo`, `PrometheusInfo`, `NeuronCertificate` data types
4. Restore the basic `ensure_signed` dispatchable calls for registration and serving

**Proof shape**:
```
$ cargo check -p pallet-game-solver
$ cargo test -p pallet-game-solver
```

---

### Phase 3: Restore Epoch Math

**Goal**: `src/epoch/math.rs` compiles using `substrate_fixed` + a local `SafeDiv` trait.

**Actions**:
1. Implement `SafeDiv<I32F32>` and `SafeDiv<I64F64>` extension traits locally in `math.rs`
2. Remove `use safe_math::*` directive
3. Wire `substrate_fixed` `exp` and `ln` calls with proper error handling

**Proof shape**:
```
$ cargo check -p pallet-game-solver  # math.rs must not produce any errors
```

---

### Phase 4: Restore Staking + Subnet Modules

**Goal**: Restore the `staking/` and `subnets/` submodules for the minimal viable pallet.

**Actions**:
1. Strip `staking/` to: `add_stake.rs`, `remove_stake.rs`, `increase_take.rs`, `decrease_take.rs`, `move_stake.rs`
2. Strip `subnets/` to: `registration.rs`, `serving.rs`, `subnet.rs`
3. Remove all `subtensor_swap_interface` types from staking calls
4. Wire staking to `pallet_balances` if needed (or keep as stub until runtime is ready)

**Proof shape**:
```
$ cargo check -p pallet-game-solver
$ cargo test -p pallet-game-solver --lib
```

---

## 5. What Is NOT Salvageable From the Current Transplant

| Element | Reason not salvageable |
|---------|------------------------|
| `subtensor_runtime_common` imports throughout | No such crate in workspace |
| `subtensor_macros::freeze_struct` throughout | No such crate in workspace |
| `subtensor_swap_interface` trait bounds | No such crate in workspace |
| `runtime_common::prod_or_fast` | No such crate |
| `safe_math::*` in `epoch/math.rs` | No such crate; must replace with local trait |
| `pallet_balances::Call` in lib.rs | `pallet_balances` not in deps |
| `sp_runtime::traits::{AsSystemOriginSigner, Implication, TransactionExtension, ValidateResult}` | Removed from polkadot-sdk stable2407 |
| `frame_support::dispatch::DispatchGuard` | Removed from frame_support |
| `codec::{DecodeWithMemTracking, ...}` codec crate alias | Wrong crate name; `codec` doesn't exist |
| 36 migration files in `src/migrations/` | One-time subtensor upgrade scripts; no value in Myosu |
| All `rpc_info/` module code | Runtime API surface, not pallet core |
| Full `coinbase/` emission distribution | Out of scope for game-solver core |
| Full `swap/` AMM module | Subtensor-specific, not Myosu-relevant |
| 60+ of the 80+ storage items | Not needed for Myosu's minimal game-solving pallet |

---

## 6. Recommended Approach

1. **Do not try to fix everything at once.** Phase 1 alone (strip non-Myosu modules + fix deps) produces a buildable pallet. Each subsequent phase adds back a layer.
2. **Do not use `subtensor_macros::freeze_struct`.** Use standard `#[derive(...)]` with `parity-scale-codec` + `scale-info`.
3. **Keep `NetUid` as a local `u16` newtype** (defined in the pallet itself) rather than importing from a non-existent `subtensor_runtime_common`.
4. **Single-token model**: Myosu uses one token. Do not attempt to restore Alpha/TAO dual-currency until there is a concrete game mechanic that requires it.
5. **The stub files (`stubs.rs`, `swap_stub.rs`) are already correct.** Do not modify them.
6. **Add `substrate-fixed` as a dependency** for epoch math. This is a real, published crate that is compatible with the polkadot-sdk version already in use.
