#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// QUAL-001: Blanket-allowed because construct_runtime!, impl_runtime_apis!,
// and PerThing types generate unchecked arithmetic. Custom pallet code is
// protected by the workspace-level deny(clippy::arithmetic_side_effects).
// Any hand-written arithmetic in this file MUST use saturating_* methods.
#![allow(clippy::arithmetic_side_effects)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod check_nonce;
mod migrations;
pub mod sudo_wrapper;
pub mod transaction_payment_wrapper;

extern crate alloc;

use codec::Encode;
use frame_support::{
    dispatch::DispatchResult,
    genesis_builder_helper::{build_state, get_preset},
    traits::{Contains, InsideBoth, LinearStoragePrice, fungible::HoldConsideration},
};
use frame_system::{EnsureRoot, EnsureRootWithSuccess};
use pallet_grandpa::{AuthorityId as GrandpaId, fg_primitives};
use pallet_subtensor::macros::config::GetCommitments;
use pallet_subtensor::rpc_info::neuron_info::NeuronInfoLite;
use pallet_subtensor::{CommitmentsInterface, ProxyInterface};
use pallet_subtensor_proxy as pallet_proxy;
use pallet_subtensor_utility as pallet_utility;
use runtime_common::prod_or_fast;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_babe::BabeConfiguration;
use sp_consensus_babe::BabeEpochConfiguration;
use sp_core::{
    OpaqueMetadata,
    crypto::{ByteArray, KeyTypeId},
};
use sp_runtime::Cow;
use sp_runtime::{
    AccountId32, ApplyExtrinsicResult, Percent, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, One, Verify},
    transaction_validity::{TransactionSource, TransactionValidity},
};
use sp_std::cmp::Ordering;
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use subtensor_runtime_common::{AlphaCurrency, AuthorshipInfo, TaoCurrency, time::*, *};
use subtensor_swap_interface::{Order, SwapEngine, SwapHandler, SwapResult};

// A few exports that help ease life for downstream crates.
pub use frame_support::{
    StorageValue, construct_runtime, parameter_types,
    traits::{
        ConstBool, ConstU8, ConstU32, ConstU64, ConstU128, FindAuthor, InstanceFilter,
        KeyOwnerProofSystem, OnFinalize, OnTimestampSet, PrivilegeCmp, Randomness, StorageInfo,
    },
    weights::{
        IdentityFee, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
        WeightToFeePolynomial,
        constants::{
            BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
        },
    },
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ConstFeeMultiplier, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

// STAGE-0 STUB: Identity swap (1:1 TAO↔Alpha, zero fees).
// This must be replaced with a real AMM or pricing mechanism before any
// public network launch. All subnet pricing, emission, and staking math
// flows through this swap surface. See genesis/plans/005 for the pallet
// simplification roadmap that will eventually replace this stub.
pub struct Stage0NoopSwap;

impl<PaidIn: Currency, PaidOut: Currency>
    subtensor_swap_interface::DefaultPriceLimit<PaidIn, PaidOut> for Stage0NoopSwap
{
    fn default_price_limit<C: Currency>() -> C {
        C::MAX
    }
}

impl<O> SwapEngine<O> for Stage0NoopSwap
where
    O: Order,
{
    fn swap(
        _netuid: NetUid,
        order: O,
        _price_limit: TaoCurrency,
        _drop_fees: bool,
        _should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, sp_runtime::DispatchError> {
        let amount_paid_in = order.amount();
        let amount_paid_out = amount_paid_in.to_u64().into();

        Ok(SwapResult {
            amount_paid_in,
            amount_paid_out,
            fee_paid: O::PaidIn::ZERO,
            fee_to_block_author: O::PaidIn::ZERO,
        })
    }
}

impl SwapHandler for Stage0NoopSwap {
    fn swap<O: Order>(
        netuid: NetUid,
        order: O,
        price_limit: TaoCurrency,
        drop_fees: bool,
        should_rollback: bool,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, sp_runtime::DispatchError>
    where
        Self: SwapEngine<O>,
    {
        <Self as SwapEngine<O>>::swap(netuid, order, price_limit, drop_fees, should_rollback)
    }

    fn sim_swap<O: Order>(
        netuid: NetUid,
        order: O,
    ) -> Result<SwapResult<O::PaidIn, O::PaidOut>, sp_runtime::DispatchError>
    where
        Self: SwapEngine<O>,
    {
        <Self as SwapEngine<O>>::swap(netuid, order, TaoCurrency::MAX, true, true)
    }

    fn approx_fee_amount<T: Currency>(_netuid: NetUid, _amount: T) -> T {
        T::ZERO
    }

    fn current_alpha_price(_netuid: NetUid) -> substrate_fixed::types::U96F32 {
        substrate_fixed::types::U96F32::from_num(1)
    }

    fn get_protocol_tao(_netuid: NetUid) -> TaoCurrency {
        TaoCurrency::ZERO
    }

    fn max_price<C: Currency>() -> C {
        C::MAX
    }

    fn min_price<C: Currency>() -> C {
        C::ZERO
    }

    fn adjust_protocol_liquidity(
        _netuid: NetUid,
        _tao_delta: TaoCurrency,
        _alpha_delta: AlphaCurrency,
    ) {
    }

    fn is_user_liquidity_enabled(_netuid: NetUid) -> bool {
        false
    }

    fn dissolve_all_liquidity_providers(_netuid: NetUid) -> DispatchResult {
        Ok(())
    }

    fn toggle_user_liquidity(_netuid: NetUid, _enabled: bool) {}

    fn clear_protocol_liquidity(_netuid: NetUid) -> DispatchResult {
        Ok(())
    }
}

#[cfg(test)]
mod stage0_noop_swap_tests {
    use super::*;

    #[test]
    fn stage0_noop_swap_is_identity_and_fee_free() {
        let netuid = NetUid::from(7);
        let tao = TaoCurrency::from(123_456_u64);
        let alpha = AlphaCurrency::from(654_321_u64);

        let tao_to_alpha = pallet_subtensor::GetAlphaForTao::<Runtime>::with_amount(tao);
        let alpha_to_tao = pallet_subtensor::GetTaoForAlpha::<Runtime>::with_amount(alpha);

        let tao_to_alpha_result = <Stage0NoopSwap as SwapHandler>::swap(
            netuid,
            tao_to_alpha,
            TaoCurrency::MAX,
            false,
            false,
        )
        .expect("stage-0 noop swap should succeed");
        let alpha_to_tao_result = <Stage0NoopSwap as SwapHandler>::swap(
            netuid,
            alpha_to_tao,
            TaoCurrency::MAX,
            false,
            false,
        )
        .expect("stage-0 noop swap should succeed");

        assert_eq!(tao_to_alpha_result.amount_paid_in, tao);
        assert_eq!(
            tao_to_alpha_result.amount_paid_out,
            AlphaCurrency::from(tao.to_u64())
        );
        assert_eq!(tao_to_alpha_result.fee_paid, TaoCurrency::ZERO);
        assert_eq!(tao_to_alpha_result.fee_to_block_author, TaoCurrency::ZERO);

        assert_eq!(alpha_to_tao_result.amount_paid_in, alpha);
        assert_eq!(
            alpha_to_tao_result.amount_paid_out,
            TaoCurrency::from(alpha.to_u64())
        );
        assert_eq!(alpha_to_tao_result.fee_paid, AlphaCurrency::ZERO);
        assert_eq!(alpha_to_tao_result.fee_to_block_author, AlphaCurrency::ZERO);
    }

    #[test]
    fn stage0_noop_swap_reports_zero_fee_and_unit_price() {
        let netuid = NetUid::from(9);

        assert_eq!(
            Stage0NoopSwap::approx_fee_amount(netuid, TaoCurrency::from(1_000_u64)),
            TaoCurrency::ZERO
        );
        assert_eq!(
            Stage0NoopSwap::approx_fee_amount(netuid, AlphaCurrency::from(1_000_u64)),
            AlphaCurrency::ZERO
        );
        assert_eq!(
            Stage0NoopSwap::current_alpha_price(netuid),
            substrate_fixed::types::U96F32::from_num(1)
        );
        assert_eq!(Stage0NoopSwap::get_protocol_tao(netuid), TaoCurrency::ZERO);
        assert_eq!(Stage0NoopSwap::max_price::<TaoCurrency>(), TaoCurrency::MAX);
        assert_eq!(
            Stage0NoopSwap::min_price::<TaoCurrency>(),
            TaoCurrency::ZERO
        );
    }

    #[test]
    fn stage0_runtime_stake_fee_surface_is_zero() {
        let hotkey_a = AccountId32::new([1_u8; 32]);
        let hotkey_b = AccountId32::new([2_u8; 32]);
        let coldkey_a = AccountId32::new([3_u8; 32]);
        let coldkey_b = AccountId32::new([4_u8; 32]);
        let netuid = NetUid::from(13);

        assert_eq!(
            SubtensorModule::get_stake_fee(
                Some((hotkey_a.clone(), netuid)),
                coldkey_a.clone(),
                Some((hotkey_a.clone(), netuid)),
                coldkey_a.clone(),
                1_000_u64,
            ),
            0_u64
        );
        assert_eq!(
            SubtensorModule::get_stake_fee(
                None,
                coldkey_a.clone(),
                Some((hotkey_a, netuid)),
                coldkey_b.clone(),
                1_000_u64,
            ),
            0_u64
        );
        assert_eq!(
            SubtensorModule::get_stake_fee(
                Some((hotkey_b, NetUid::ROOT)),
                coldkey_b,
                None,
                coldkey_a,
                1_000_u64,
            ),
            0_u64
        );
    }
}

impl<C> frame_system::offchain::CreateTransactionBase<C> for Runtime
where
    RuntimeCall: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type RuntimeCall = RuntimeCall;
}

// Subtensor module
pub use pallet_scheduler;
pub use pallet_subtensor;

// Method used to calculate the fee of an extrinsic
pub const fn deposit(items: u32, bytes: u32) -> Balance {
    pub const ITEMS_FEE: Balance = 2_000 * 10_000;
    pub const BYTES_FEE: Balance = 100 * 10_000;
    (items as Balance)
        .saturating_mul(ITEMS_FEE)
        .saturating_add((bytes as Balance).saturating_mul(BYTES_FEE))
}

// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
// the specifics of the runtime. They can then be made to be agnostic over specific formats
// of data like extrinsics, allowing for them to continue syncing the network through upgrades
// to even the core data structures.
pub mod opaque {
    use super::*;

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

    // Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    // Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    // Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub aura: Aura,
            pub grandpa: Grandpa,
        }
    }
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("myosu-chain"),
    impl_name: Cow::Borrowed("myosu-chain"),
    authoring_version: 1,
    // The version of the runtime specification. A full node will not attempt to use its native
    //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
    //   `spec_version`, and `authoring_version` are the same between Wasm and native.
    // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
    //   the compatible custom types.
    spec_version: 385,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

pub const MAXIMUM_BLOCK_WEIGHT: Weight =
    Weight::from_parts(4u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX);

// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    // We allow for 2 seconds of compute with a 6 second average block time.
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            MAXIMUM_BLOCK_WEIGHT,
            NORMAL_DISPATCH_RATIO,
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(10 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

pub struct NoNestingCallFilter;

impl Contains<RuntimeCall> for NoNestingCallFilter {
    fn contains(call: &RuntimeCall) -> bool {
        match call {
            RuntimeCall::Utility(inner) => {
                let calls = match inner {
                    pallet_utility::Call::force_batch { calls } => calls,
                    pallet_utility::Call::batch { calls } => calls,
                    pallet_utility::Call::batch_all { calls } => calls,
                    _ => &Vec::new(),
                };

                !calls.iter().any(|call| {
					matches!(call, RuntimeCall::Utility(inner) if matches!(inner, pallet_utility::Call::force_batch { .. } | pallet_utility::Call::batch_all { .. } | pallet_utility::Call::batch { .. }))
				})
            }
            _ => true,
        }
    }
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
    // The basic call filter to use in dispatchable.
    type BaseCallFilter = InsideBoth<SafeMode, NoNestingCallFilter>;
    // Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    // The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    // The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    // The aggregated dispatch type that is available for extrinsics.
    type RuntimeCall = RuntimeCall;
    // The aggregated runtime tasks.
    type RuntimeTask = RuntimeTask;
    // The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = AccountIdLookup<AccountId, ()>;
    // The type for hashing blocks and tries.
    type Hash = Hash;
    // The hashing algorithm used.
    type Hashing = BlakeTwo256;
    // The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    // The ubiquitous origin type.
    type RuntimeOrigin = RuntimeOrigin;
    // Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    // The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    // Version of the runtime.
    type Version = Version;
    // Converts a module to the index of the module in `construct_runtime!`.
    //
    // This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    // What to do if a new account is created.
    type OnNewAccount = ();
    // What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    // The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    // Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = frame_system::weights::SubstrateWeight<Runtime>;
    // This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    // The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type Nonce = Nonce;
    type Block = Block;
    type SingleBlockMigrations = Migrations;
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type ExtensionsWeightInfo = frame_system::SubstrateExtensionsWeight<Runtime>;
    type DispatchGuard = pallet_subtensor::CheckColdkeySwap<Runtime>;
}

// SEC-001: This pallet provides weak randomness derived from block hashes.
// It is NOT consumed by game-solver or any production on-chain logic today.
// Retained at pallet index 1 to avoid breaking storage key layout.
// Must be replaced or removed before any public network launch where
// randomness-dependent operations (e.g., PoW registration) are enabled.
impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ConstBool<false>;
    type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;

    type KeyOwnerProof = sp_core::Void;

    type WeightInfo = (); // pallet_grandpa exports only default implementation
    type MaxAuthorities = ConstU32<32>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type MaxNominators = ConstU32<20>;

    type EquivocationReportSystem = ();
}

/// Babe epoch duration.
///
/// Staging this Babe constant prior to enacting the full Babe upgrade so the node
/// can build itself a `BabeConfiguration` prior to the upgrade taking place.
pub const EPOCH_DURATION_IN_SLOTS: u64 = prod_or_fast!(4 * HOURS as u64, MINUTES as u64 / 6);

/// 1 in 4 blocks (on average, not counting collisions) will be primary babe blocks.
/// The choice of is done in accordance to the slot duration and expected target
/// block time, for safely resisting network delays of maximum two seconds.
/// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
///
/// Staging this Babe constant prior to enacting the full Babe upgrade so the node
/// can build itself a `BabeConfiguration` prior to the upgrade taking place.
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

/// The BABE epoch configuration at genesis.
///
/// Staging this Babe constant prior to enacting the full Babe upgrade so the node
/// can build itself a `BabeConfiguration` prior to the upgrade taking place.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
    sp_consensus_babe::BabeEpochConfiguration {
        c: PRIMARY_PROBABILITY,
        allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
    };

impl pallet_timestamp::Config for Runtime {
    // A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const DisallowPermissionlessEnterDuration: BlockNumber = 0;
    pub const DisallowPermissionlessExtendDuration: BlockNumber = 0;

    pub const RootEnterDuration: BlockNumber = 5 * 60 * 24; // 24 hours

    pub const RootExtendDuration: BlockNumber = 5 * 60 * 12; // 12 hours

    pub const DisallowPermissionlessEntering: Option<Balance> = None;
    pub const DisallowPermissionlessExtending: Option<Balance> = None;
    pub const DisallowPermissionlessRelease: Option<BlockNumber> = None;
}

pub struct SafeModeWhitelistedCalls;
impl Contains<RuntimeCall> for SafeModeWhitelistedCalls {
    fn contains(call: &RuntimeCall) -> bool {
        matches!(
            call,
            RuntimeCall::Sudo(_)
                | RuntimeCall::Multisig(_)
                | RuntimeCall::System(_)
                | RuntimeCall::SafeMode(_)
                | RuntimeCall::Timestamp(_)
                | RuntimeCall::SubtensorModule(
                    pallet_subtensor::Call::set_weights { .. }
                        | pallet_subtensor::Call::serve_axon { .. }
                )
        )
    }
}

impl pallet_safe_mode::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type WhitelistedCalls = SafeModeWhitelistedCalls;
    type EnterDuration = DisallowPermissionlessEnterDuration;
    type ExtendDuration = DisallowPermissionlessExtendDuration;
    type EnterDepositAmount = DisallowPermissionlessEntering;
    type ExtendDepositAmount = DisallowPermissionlessExtending;
    type ForceEnterOrigin = EnsureRootWithSuccess<AccountId, RootEnterDuration>;
    type ForceExtendOrigin = EnsureRootWithSuccess<AccountId, RootExtendDuration>;
    type ForceExitOrigin = EnsureRoot<AccountId>;
    type ForceDepositOrigin = EnsureRoot<AccountId>;
    type Notify = ();
    type ReleaseDelay = DisallowPermissionlessRelease;
    type WeightInfo = pallet_safe_mode::weights::SubstrateWeight<Runtime>;
}

// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u64 = 500;

impl pallet_balances::Config for Runtime {
    type MaxLocks = ConstU32<50>;
    type MaxReserves = ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    // The type for recording an account's balance.
    type Balance = Balance;
    // The ubiquitous event type.
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;

    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = ConstU32<50>;
    type DoneSlashHandler = ();
}

// Implement AuthorshipInfo trait for Runtime to satisfy pallet transaction
// fee OnUnbalanced trait bounds
pub struct BlockAuthorFromAura<F>(core::marker::PhantomData<F>);

impl<F: FindAuthor<u32>> BlockAuthorFromAura<F> {
    pub fn get_block_author() -> Option<AccountId32> {
        let binding = frame_system::Pallet::<Runtime>::digest();
        let digest_logs = binding.logs();
        let author_index = F::find_author(digest_logs.iter().filter_map(|d| d.as_pre_runtime()))?;
        let authority_id = pallet_aura::Authorities::<Runtime>::get()
            .get(author_index as usize)?
            .clone();
        Some(AccountId32::new(authority_id.to_raw_vec().try_into().ok()?))
    }
}

impl AuthorshipInfo<AccountId32> for Runtime {
    fn author() -> Option<AccountId32> {
        BlockAuthorFromAura::<Aura>::get_block_author()
    }
}

impl<F: FindAuthor<u32>> AuthorshipInfo<sp_runtime::AccountId32> for BlockAuthorFromAura<F> {
    fn author() -> Option<sp_runtime::AccountId32> {
        Self::get_block_author()
    }
}

parameter_types! {
    pub const OperationalFeeMultiplier: u8 = 5;
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
    // Convert dispatch weight to a chargeable fee.
    type WeightToFee = subtensor_transaction_fee::LinearWeightToFee;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;

    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // According to multisig pallet, key and value size be computed as follows:
    // value size is `4 + sizeof((BlockNumber, Balance, AccountId))` bytes
    // key size is `32 + sizeof(AccountId)` bytes.
    // For our case, One storage item; key size is 32+32=64 bytes; value is size 4+4+8+32 bytes = 48 bytes.
    pub const DepositBase: Balance = deposit(1, 112);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);
    pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = MaxSignatories;
    type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
    type BlockNumberProvider = System;
}

// Proxy Pallet config
parameter_types! {
    // One storage item; key size sizeof(AccountId) = 32, value sizeof(Balance) = 8; 40 total
    pub const ProxyDepositBase: Balance = deposit(1, 40);
    // Adding 32 bytes + sizeof(ProxyType) = 32 + 1
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const MaxProxies: u32 = 20; // max num proxies per acct
    pub const MaxPending: u32 = 15 * 5; // max blocks pending ~15min
    // 16 bytes
    pub const AnnouncementDepositBase: Balance =  deposit(1, 16);
    // 68 bytes per announcement
    pub const AnnouncementDepositFactor: Balance = deposit(0, 68);
}

impl InstanceFilter<RuntimeCall> for ProxyType {
    fn filter(&self, c: &RuntimeCall) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::NonTransfer => !matches!(
                c,
                RuntimeCall::Balances(..)
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::schedule_swap_coldkey { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_coldkey { .. })
            ),
            ProxyType::NonFungible => !matches!(
                c,
                RuntimeCall::Balances(..)
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_limit { .. }
                    )
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_full_limit { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::unstake_all { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::unstake_all_alpha { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::move_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::schedule_swap_coldkey { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_coldkey { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey { .. })
            ),
            ProxyType::Transfer => matches!(
                c,
                RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { .. })
                    | RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death { .. })
                    | RuntimeCall::Balances(pallet_balances::Call::transfer_all { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake { .. })
            ),
            ProxyType::SmallTransfer => match c {
                RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
                    value, ..
                }) => *value < SMALL_TRANSFER_LIMIT,
                RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
                    value,
                    ..
                }) => *value < SMALL_TRANSFER_LIMIT,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::transfer_stake {
                    alpha_amount,
                    ..
                }) => *alpha_amount < SMALL_TRANSFER_LIMIT.into(),
                _ => false,
            },
            ProxyType::Owner => {
                matches!(
                    c,
                    RuntimeCall::AdminUtils(..)
                        | RuntimeCall::SubtensorModule(
                            pallet_subtensor::Call::set_subnet_identity { .. }
                        )
                        | RuntimeCall::SubtensorModule(
                            pallet_subtensor::Call::update_symbol { .. }
                        )
                ) && !matches!(
                    c,
                    RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_sn_owner_hotkey { .. }
                    )
                )
            }
            ProxyType::NonCritical => !matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::dissolve_network { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::root_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::Sudo(..)
            ),
            ProxyType::Triumvirate => false, // deprecated
            ProxyType::Senate => false,      // deprecated
            ProxyType::Governance => false,  // deprecated
            ProxyType::Staking => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::remove_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::unstake_all { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::unstake_all_alpha { .. }
                    )
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::move_stake { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::add_stake_limit { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_limit { .. }
                    )
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::remove_stake_full_limit { .. }
                    )
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::set_root_claim_type { .. }
                    )
            ),
            ProxyType::Registration => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::burned_register { .. })
                    | RuntimeCall::SubtensorModule(pallet_subtensor::Call::register { .. })
            ),
            ProxyType::RootWeights => false, // deprecated
            ProxyType::ChildKeys => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::set_children { .. })
                    | RuntimeCall::SubtensorModule(
                        pallet_subtensor::Call::set_childkey_take { .. }
                    )
            ),
            ProxyType::SudoUncheckedSetCode => match c {
                RuntimeCall::Sudo(pallet_sudo::Call::sudo_unchecked_weight { call, weight: _ }) => {
                    let inner_call: RuntimeCall = *call.clone();

                    matches!(
                        inner_call,
                        RuntimeCall::System(frame_system::Call::set_code { .. })
                    )
                }
                _ => false,
            },
            ProxyType::SwapHotkey => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::swap_hotkey { .. })
            ),
            ProxyType::SubnetLeaseBeneficiary => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::start_call { .. })
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_serving_rate_limit { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_min_difficulty { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_max_difficulty { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_weights_version_key { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_adjustment_alpha { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_immunity_period { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_min_allowed_weights { .. }
                    )
                    | RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_kappa { .. })
                    | RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_rho { .. })
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_activity_cutoff { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_network_registration_allowed { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_network_pow_registration_allowed { .. }
                    )
                    | RuntimeCall::AdminUtils(pallet_admin_utils::Call::sudo_set_max_burn { .. })
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_bonds_moving_average { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_bonds_penalty { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_commit_reveal_weights_enabled { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_liquid_alpha_enabled { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_alpha_values { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_commit_reveal_weights_interval { .. }
                    )
                    | RuntimeCall::AdminUtils(
                        pallet_admin_utils::Call::sudo_set_toggle_transfer { .. }
                    )
            ),
            ProxyType::RootClaim => matches!(
                c,
                RuntimeCall::SubtensorModule(pallet_subtensor::Call::claim_root { .. })
            ),
        }
    }
    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            (ProxyType::NonTransfer, _) => {
                // NonTransfer is NOT a superset of Transfer or SmallTransfer
                !matches!(o, ProxyType::Transfer | ProxyType::SmallTransfer)
            }
            (ProxyType::Transfer, ProxyType::SmallTransfer) => true,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type RuntimeCall = RuntimeCall;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = MaxProxies;
    type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
    type MaxPending = MaxPending;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
    type BlockNumberProvider = System;
}

pub struct Proxier;
impl ProxyInterface<AccountId> for Proxier {
    fn add_lease_beneficiary_proxy(lease: &AccountId, beneficiary: &AccountId) -> DispatchResult {
        pallet_proxy::Pallet::<Runtime>::add_proxy_delegate(
            lease,
            beneficiary.clone(),
            ProxyType::SubnetLeaseBeneficiary,
            0,
        )
    }

    fn remove_lease_beneficiary_proxy(
        lease: &AccountId,
        beneficiary: &AccountId,
    ) -> DispatchResult {
        pallet_proxy::Pallet::<Runtime>::remove_proxy_delegate(
            lease,
            beneficiary.clone(),
            ProxyType::SubnetLeaseBeneficiary,
            0,
        )
    }
}

pub struct CommitmentsI;
impl CommitmentsInterface for CommitmentsI {
    fn purge_netuid(_netuid: NetUid) {}
}

parameter_types! {
    pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
        BlockWeights::get().max_block;
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u32> = Some(10);
}

/// Used the compare the privilege of an origin inside the scheduler.
pub struct OriginPrivilegeCmp;

impl PrivilegeCmp<OriginCaller> for OriginPrivilegeCmp {
    fn cmp_privilege(left: &OriginCaller, right: &OriginCaller) -> Option<Ordering> {
        if left == right {
            return Some(Ordering::Equal);
        }

        match (left, right) {
            // Root is greater than anything.
            (OriginCaller::system(frame_system::RawOrigin::Root), _) => Some(Ordering::Greater),
            // For every other origin we don't care, as they are not used for `ScheduleOrigin`.
            _ => None,
        }
    }
}

impl pallet_scheduler::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeEvent = RuntimeEvent;
    type PalletsOrigin = OriginCaller;
    type RuntimeCall = RuntimeCall;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
    type OriginPrivilegeCmp = OriginPrivilegeCmp;
    type Preimages = Preimage;
    type BlockNumberProvider = System;
}

parameter_types! {
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub const PreimageBaseDeposit: Balance = deposit(2, 64);
    pub const PreimageByteDeposit: Balance = deposit(0, 1);
    pub const PreimageHoldReason: RuntimeHoldReason =
        RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type Consideration = HoldConsideration<
        AccountId,
        Balances,
        PreimageHoldReason,
        LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
    >;
}

pub struct GetCommitmentsStruct;
impl GetCommitments<AccountId> for GetCommitmentsStruct {
    fn get_commitments(_netuid: NetUid) -> Vec<(AccountId, Vec<u8>)> {
        Vec::new()
    }
}

pub const INITIAL_SUBNET_TEMPO: u16 = prod_or_fast!(360, 10);

// 30 days at 12 seconds per block = 216000
pub const INITIAL_CHILDKEY_TAKE_RATELIMIT: u64 = prod_or_fast!(216000, 5);

pub const EVM_KEY_ASSOCIATE_RATELIMIT: u64 = prod_or_fast!(7200, 1); // 24 * 60 * 60 / 12; // 1 day

// Configure the pallet subtensor.
parameter_types! {
    pub const SubtensorInitialRho: u16 = 10;
    pub const SubtensorInitialAlphaSigmoidSteepness: i16 = 1000;
    pub const SubtensorInitialKappa: u16 = 32_767; // 0.5 = 65535/2
    pub const SubtensorInitialMaxAllowedUids: u16 = 256;
    pub const SubtensorInitialIssuance: u64 = 0;
    pub const SubtensorInitialMinAllowedWeights: u16 = 1024;
    pub const SubtensorInitialEmissionValue: u16 = 0;
    pub const SubtensorInitialValidatorPruneLen: u64 = 1;
    pub const SubtensorInitialScalingLawPower: u16 = 50; // 0.5
    pub const SubtensorInitialMaxAllowedValidators: u16 = 128;
    pub const SubtensorInitialTempo: u16 = INITIAL_SUBNET_TEMPO;
    pub const SubtensorInitialDifficulty: u64 = 10_000_000;
    pub const SubtensorInitialAdjustmentInterval: u16 = 100;
    pub const SubtensorInitialAdjustmentAlpha: u64 = 0; // no weight to previous value.
    pub const SubtensorInitialTargetRegistrationsPerInterval: u16 = 2;
    pub const SubtensorInitialImmunityPeriod: u16 = 4096;
    pub const SubtensorInitialActivityCutoff: u16 = 5000;
    pub const SubtensorInitialMaxRegistrationsPerBlock: u16 = 1;
    pub const SubtensorInitialPruningScore : u16 = u16::MAX;
    pub const SubtensorInitialBondsMovingAverage: u64 = 900_000;
    pub const SubtensorInitialBondsPenalty: u16 = u16::MAX;
    pub const SubtensorInitialBondsResetOn: bool = false;
    pub const SubtensorInitialDefaultTake: u16 = 11_796; // 18% honest number.
    pub const SubtensorInitialMinDelegateTake: u16 = 0; // Allow 0% delegate take
    pub const SubtensorInitialDefaultChildKeyTake: u16 = 0; // Allow 0% childkey take
    pub const SubtensorInitialMinChildKeyTake: u16 = 0; // 0 %
    pub const SubtensorInitialMaxChildKeyTake: u16 = 11_796; // 18 %
    pub const SubtensorInitialWeightsVersionKey: u64 = 0;
    pub const SubtensorInitialMinDifficulty: u64 = 10_000_000;
    pub const SubtensorInitialMaxDifficulty: u64 = u64::MAX / 4;
    pub const SubtensorInitialServingRateLimit: u64 = 50;
    pub const SubtensorInitialBurn: u64 = 100_000_000; // 0.1 tao
    pub const SubtensorInitialMinBurn: u64 = 500_000; // 500k RAO
    pub const SubtensorInitialMaxBurn: u64 = 100_000_000_000; // 100 tao
    pub const MinBurnUpperBound: TaoCurrency = TaoCurrency::new(1_000_000_000); // 1 TAO
    pub const MaxBurnLowerBound: TaoCurrency = TaoCurrency::new(100_000_000); // 0.1 TAO
    pub const SubtensorInitialTxRateLimit: u64 = 1000;
    pub const SubtensorInitialTxDelegateTakeRateLimit: u64 = 216000; // 30 days at 12 seconds per block
    pub const SubtensorInitialTxChildKeyTakeRateLimit: u64 = INITIAL_CHILDKEY_TAKE_RATELIMIT;
    pub const SubtensorInitialRAORecycledForRegistration: u64 = 0; // 0 rao
    pub const SubtensorInitialRequiredStakePercentage: u64 = 1; // 1 percent of total stake
    pub const SubtensorInitialNetworkImmunity: u64 = 1_296_000;
    pub const SubtensorInitialMinAllowedUids: u16 = 64;
    pub const SubtensorInitialMinLockCost: u64 = 1_000_000_000_000; // 1000 TAO
    pub const SubtensorInitialSubnetOwnerCut: u16 = 11_796; // 18 percent
    pub const SubtensorInitialNetworkLockReductionInterval: u64 = 14 * 7200;
    pub const SubtensorInitialNetworkRateLimit: u64 = 7200;
    pub const SubtensorInitialKeySwapCost: u64 = 100_000_000; // 0.1 TAO
    pub const InitialAlphaHigh: u16 = 58982; // Represents 0.9 as per the production default
    pub const InitialAlphaLow: u16 = 45875; // Represents 0.7 as per the production default
    pub const InitialLiquidAlphaOn: bool = false; // Default value for LiquidAlphaOn
    pub const InitialYuma3On: bool = false; // Default value for Yuma3On
    pub const InitialColdkeySwapAnnouncementDelay: BlockNumber = prod_or_fast!(5 * 24 * 60 * 60 / 12, 50); // 5 days
    pub const InitialColdkeySwapReannouncementDelay: BlockNumber = prod_or_fast!(24 * 60 * 60 / 12, 10); // 1 day
    pub const InitialDissolveNetworkScheduleDuration: BlockNumber = 5 * 24 * 60 * 60 / 12; // 5 days
    pub const SubtensorInitialTaoWeight: u64 = 971_718_665_099_567_868; // 0.05267697438728329% tao weight.
    pub const InitialEmaPriceHalvingPeriod: u64 = 201_600_u64; // 4 weeks
    // 0 days
    pub const InitialStartCallDelay: u64 = 0;
    pub const SubtensorInitialKeySwapOnSubnetCost: u64 = 1_000_000; // 0.001 TAO
    pub const HotkeySwapOnSubnetInterval : BlockNumber = 5 * 24 * 60 * 60 / 12; // 5 days
    pub const LeaseDividendsDistributionInterval: BlockNumber = 100; // 100 blocks
    pub const MaxImmuneUidsPercentage: Percent = Percent::from_percent(80);
    pub const EvmKeyAssociateRateLimit: u64 = EVM_KEY_ASSOCIATE_RATELIMIT;
}

impl pallet_subtensor::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type SudoRuntimeCall = RuntimeCall;
    type Currency = Balances;
    type Scheduler = Scheduler;
    type InitialRho = SubtensorInitialRho;
    type InitialAlphaSigmoidSteepness = SubtensorInitialAlphaSigmoidSteepness;
    type InitialKappa = SubtensorInitialKappa;
    type InitialMinAllowedUids = SubtensorInitialMinAllowedUids;
    type InitialMaxAllowedUids = SubtensorInitialMaxAllowedUids;
    type InitialBondsMovingAverage = SubtensorInitialBondsMovingAverage;
    type InitialBondsPenalty = SubtensorInitialBondsPenalty;
    type InitialBondsResetOn = SubtensorInitialBondsResetOn;
    type InitialIssuance = SubtensorInitialIssuance;
    type InitialMinAllowedWeights = SubtensorInitialMinAllowedWeights;
    type InitialEmissionValue = SubtensorInitialEmissionValue;
    type InitialValidatorPruneLen = SubtensorInitialValidatorPruneLen;
    type InitialScalingLawPower = SubtensorInitialScalingLawPower;
    type InitialTempo = SubtensorInitialTempo;
    type InitialDifficulty = SubtensorInitialDifficulty;
    type InitialAdjustmentInterval = SubtensorInitialAdjustmentInterval;
    type InitialAdjustmentAlpha = SubtensorInitialAdjustmentAlpha;
    type InitialTargetRegistrationsPerInterval = SubtensorInitialTargetRegistrationsPerInterval;
    type InitialImmunityPeriod = SubtensorInitialImmunityPeriod;
    type InitialActivityCutoff = SubtensorInitialActivityCutoff;
    type InitialMaxRegistrationsPerBlock = SubtensorInitialMaxRegistrationsPerBlock;
    type InitialPruningScore = SubtensorInitialPruningScore;
    type InitialMaxAllowedValidators = SubtensorInitialMaxAllowedValidators;
    type InitialDefaultDelegateTake = SubtensorInitialDefaultTake;
    type InitialDefaultChildKeyTake = SubtensorInitialDefaultChildKeyTake;
    type InitialMinDelegateTake = SubtensorInitialMinDelegateTake;
    type InitialMinChildKeyTake = SubtensorInitialMinChildKeyTake;
    type InitialWeightsVersionKey = SubtensorInitialWeightsVersionKey;
    type InitialMaxDifficulty = SubtensorInitialMaxDifficulty;
    type InitialMinDifficulty = SubtensorInitialMinDifficulty;
    type InitialServingRateLimit = SubtensorInitialServingRateLimit;
    type InitialBurn = SubtensorInitialBurn;
    type InitialMaxBurn = SubtensorInitialMaxBurn;
    type InitialMinBurn = SubtensorInitialMinBurn;
    type MinBurnUpperBound = MinBurnUpperBound;
    type MaxBurnLowerBound = MaxBurnLowerBound;
    type InitialTxRateLimit = SubtensorInitialTxRateLimit;
    type InitialTxDelegateTakeRateLimit = SubtensorInitialTxDelegateTakeRateLimit;
    type InitialTxChildKeyTakeRateLimit = SubtensorInitialTxChildKeyTakeRateLimit;
    type InitialMaxChildKeyTake = SubtensorInitialMaxChildKeyTake;
    type InitialRAORecycledForRegistration = SubtensorInitialRAORecycledForRegistration;
    type InitialNetworkImmunityPeriod = SubtensorInitialNetworkImmunity;
    type InitialNetworkMinLockCost = SubtensorInitialMinLockCost;
    type InitialNetworkLockReductionInterval = SubtensorInitialNetworkLockReductionInterval;
    type InitialSubnetOwnerCut = SubtensorInitialSubnetOwnerCut;
    type InitialNetworkRateLimit = SubtensorInitialNetworkRateLimit;
    type KeySwapCost = SubtensorInitialKeySwapCost;
    type AlphaHigh = InitialAlphaHigh;
    type AlphaLow = InitialAlphaLow;
    type LiquidAlphaOn = InitialLiquidAlphaOn;
    type Yuma3On = InitialYuma3On;
    type InitialTaoWeight = SubtensorInitialTaoWeight;
    type Preimages = Preimage;
    type InitialColdkeySwapAnnouncementDelay = InitialColdkeySwapAnnouncementDelay;
    type InitialColdkeySwapReannouncementDelay = InitialColdkeySwapReannouncementDelay;
    type InitialDissolveNetworkScheduleDuration = InitialDissolveNetworkScheduleDuration;
    type InitialEmaPriceHalvingPeriod = InitialEmaPriceHalvingPeriod;
    type InitialStartCallDelay = InitialStartCallDelay;
    type SwapInterface = Stage0NoopSwap;
    type KeySwapOnSubnetCost = SubtensorInitialKeySwapOnSubnetCost;
    type HotkeySwapOnSubnetInterval = HotkeySwapOnSubnetInterval;
    type ProxyInterface = Proxier;
    type LeaseDividendsDistributionInterval = LeaseDividendsDistributionInterval;
    type GetCommitments = GetCommitmentsStruct;
    type MaxContributors = MaxContributors;
    type MaxImmuneUidsPercentage = MaxImmuneUidsPercentage;
    type CommitmentsInterface = CommitmentsI;
    type EvmKeyAssociateRateLimit = EvmKeyAssociateRateLimit;
    type AuthorshipProvider = BlockAuthorFromAura<Aura>;
}

use sp_runtime::BoundedVec;

pub struct AuraPalletIntrf;
impl pallet_admin_utils::AuraInterface<AuraId, ConstU32<32>> for AuraPalletIntrf {
    fn change_authorities(new: BoundedVec<AuraId, ConstU32<32>>) {
        Aura::change_authorities(new);
    }
}

pub struct GrandpaInterfaceImpl;
impl pallet_admin_utils::GrandpaInterface<Runtime> for GrandpaInterfaceImpl {
    fn schedule_change(
        next_authorities: Vec<(pallet_grandpa::AuthorityId, u64)>,
        in_blocks: BlockNumber,
        forced: Option<BlockNumber>,
    ) -> sp_runtime::DispatchResult {
        Grandpa::schedule_change(next_authorities, in_blocks, forced)
    }
}

impl pallet_admin_utils::Config for Runtime {
    type AuthorityId = AuraId;
    type MaxAuthorities = ConstU32<32>;
    type Aura = AuraPalletIntrf;
    type Grandpa = GrandpaInterfaceImpl;
    type Balance = Balance;
}

parameter_types! {
    pub const MaxContributors: u32 = 32;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub struct Runtime
    {
        System: frame_system = 0,
        RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip = 1,
        Timestamp: pallet_timestamp = 2,
        Aura: pallet_aura = 3,
        Grandpa: pallet_grandpa = 4,
        Balances: pallet_balances = 5,
        TransactionPayment: pallet_transaction_payment = 6,
        SubtensorModule: pallet_subtensor = 7,
        // pallet_collective::<Instance1> (triumvirate) was 8
        // pallet_membership::<Instance1> (triumvirate members) was 9
        // pallet_membership::<Instance2> (senate members) was 10
        Utility: pallet_utility = 11,
        Sudo: pallet_sudo = 12,
        Multisig: pallet_multisig = 13,
        Preimage: pallet_preimage = 14,
        Scheduler: pallet_scheduler = 15,
        Proxy: pallet_proxy = 16,
        AdminUtils: pallet_admin_utils = 19,
        SafeMode: pallet_safe_mode = 20,
    }
);

// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
// The extensions to the basic transaction logic.
pub type TransactionExtensions = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    check_nonce::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);

type Migrations = (
    // Leave this migration in the runtime, so every runtime upgrade tiny rounding errors (fractions of fractions
    // of a cent) are cleaned up. These tiny rounding errors occur due to floating point coversion.
    pallet_subtensor::migrations::migrate_init_total_issuance::initialise_total_issuance::Migration<
        Runtime,
    >,
);

// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TransactionExtensions>;

// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, TransactionExtensions>;
// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
        [pallet_sudo, Sudo]
        [pallet_admin_utils, AdminUtils]
        [pallet_subtensor, SubtensorModule]
        [pallet_subtensor_proxy, Proxy]
    );
}

fn generate_genesis_json() -> Vec<u8> {
    let json_str = r#"{
      "aura": {
        "authorities": [
          "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        ]
      },
      "balances": {
        "balances": [
          [
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            1000000000000000
          ],
          [
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            1000000000000000
          ]
        ]
      },
      "grandpa": {
        "authorities": [
          [
            "5FA9nQDVg267DEd8m1ZypXLBnvN7SFxYwV7ndqSYGiN9TTpu",
            1
          ]
        ]
      },
      "sudo": {
        "key": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
      },
      "subtensorModule": {
        "balancesIssuance": 0,
        "stakes": []
      }
    }"#;

    json_str.as_bytes().to_vec()
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> sp_std::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
            build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            get_preset::<RuntimeGenesisConfig>(id, |preset_id| {
                let benchmark_id: sp_genesis_builder::PresetId = "benchmark".into();
                if *preset_id == benchmark_id {
                    Some(generate_genesis_json())
                } else {
                    None
                }
            })
        }

        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
            vec!["benchmark".into()]
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            use codec::DecodeLimit;
            use frame_support::pallet_prelude::{InvalidTransaction, TransactionValidityError};
            use sp_runtime::traits::ExtrinsicCall;
            let encoded = tx.call().encode();
            if RuntimeCall::decode_all_with_depth_limit(8, &mut encoded.as_slice()).is_err() {
                log::warn!("failed to decode with depth limit of 8");
                return Err(TransactionValidityError::Invalid(InvalidTransaction::Call));
            }
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            pallet_aura::Authorities::<Runtime>::get().into_inner()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> Vec<(GrandpaId, u64)> {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            let key_owner_proof = key_owner_proof.decode()?;

            Grandpa::submit_unsigned_equivocation_report(
                equivocation_proof,
                key_owner_proof,
            )
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: fg_primitives::AuthorityId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
        for Runtime
    {
        fn query_call_info(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_call_info(call, len)
        }
        fn query_call_fee_details(
            call: RuntimeCall,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_call_fee_details(call, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
            use frame_benchmarking::{baseline, BenchmarkBatch};
            use sp_storage::TrackedStorageKey;

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            #[allow(non_local_definitions)]
            impl frame_system_benchmarking::Config for Runtime {}

            #[allow(non_local_definitions)]
            impl baseline::Config for Runtime {}

            use frame_support::traits::WhitelistedStorageKeys;
            let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        #[allow(clippy::unwrap_used)]
        fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade(checks).unwrap();
            (weight, BlockWeights::get().max_block)
        }

        #[allow(clippy::expect_used)]
        fn execute_block(
            block: Block,
            state_root_check: bool,
            signature_check: bool,
            select: frame_try_runtime::TryStateSelect
        ) -> Weight {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here.
            Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
        }
    }

    impl subtensor_custom_rpc_runtime_api::NeuronInfoRuntimeApi<Block> for Runtime {
        fn get_neurons_lite(netuid: NetUid) -> Vec<NeuronInfoLite<AccountId32>> {
            SubtensorModule::get_neurons_lite(netuid)
        }
    }

    impl sp_consensus_babe::BabeApi<Block> for Runtime {
        fn configuration() -> BabeConfiguration {
            let config = BabeEpochConfiguration::default();
            BabeConfiguration {
                slot_duration: Default::default(),
                epoch_length: Default::default(),
                authorities: vec![],
                randomness: Default::default(),
                c: config.c,
                allowed_slots: config.allowed_slots,

            }
        }

        fn current_epoch_start() -> sp_consensus_babe::Slot {
            Default::default()
        }

        fn current_epoch() -> sp_consensus_babe::Epoch {
            sp_consensus_babe::Epoch {
                epoch_index: Default::default(),
                start_slot: Default::default(),
                duration: Default::default(),
                authorities: vec![],
                randomness: Default::default(),
                config: BabeEpochConfiguration::default(),
            }
        }

        fn next_epoch() -> sp_consensus_babe::Epoch {
            sp_consensus_babe::Epoch {
                epoch_index: Default::default(),
                start_slot: Default::default(),
                duration: Default::default(),
                authorities: vec![],
                randomness: Default::default(),
                config: BabeEpochConfiguration::default(),
            }
        }

        fn generate_key_ownership_proof(
            _slot: sp_consensus_babe::Slot,
            _authority_id: sp_consensus_babe::AuthorityId,
        ) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
            None
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
            _key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }
    }
}

#[test]
fn check_whitelist() {
    use crate::*;
    use frame_support::traits::WhitelistedStorageKeys;
    use sp_core::hexdisplay::HexDisplay;
    use std::collections::HashSet;
    let whitelist: HashSet<String> = AllPalletsWithSystem::whitelisted_storage_keys()
        .iter()
        .map(|e| HexDisplay::from(&e.key).to_string())
        .collect();

    // Block Number
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac"));
    // Total Issuance
    assert!(whitelist.contains("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80"));
    // Execution Phase
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a"));
    // Event Count
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850"));
    // System Events
    assert!(whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7"));
}
