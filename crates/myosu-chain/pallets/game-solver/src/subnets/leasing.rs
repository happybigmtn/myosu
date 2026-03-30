//! Legacy subnet-leasing storage types retained only so stage-0 teardown can
//! clear old state safely.
//!
//! Stage 0 no longer exposes or executes crowdloan/leasing behavior. The live
//! pallet keeps just these type definitions so storage declarations and root
//! cleanup can remove carried lease-era state without reviving the mechanism.

use super::*;
use frame_support::traits::fungible;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Percent;
use subtensor_runtime_common::NetUid;

pub type LeaseId = u32;

pub type CurrencyOf<T> = <T as Config>::Currency;

pub type BalanceOf<T> =
    <CurrencyOf<T> as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;

#[freeze_struct("8cc3d0594faed7dd")]
#[derive(Encode, Decode, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub struct SubnetLease<AccountId, BlockNumber, Balance> {
    /// The beneficiary of the lease, able to operate the subnet through
    /// a proxy and taking ownership of the subnet at the end of the lease (if defined).
    pub beneficiary: AccountId,
    /// The coldkey of the lease.
    pub coldkey: AccountId,
    /// The hotkey of the lease.
    pub hotkey: AccountId,
    /// The share of the emissions that the contributors will receive.
    pub emissions_share: Percent,
    /// The block at which the lease will end. If not defined, the lease is perpetual.
    pub end_block: Option<BlockNumber>,
    /// The netuid of the subnet that the lease is for.
    pub netuid: NetUid,
    /// The cost of the lease including the network registration and proxy.
    pub cost: Balance,
}

pub type SubnetLeaseOf<T> =
    SubnetLease<<T as frame_system::Config>::AccountId, BlockNumberFor<T>, BalanceOf<T>>;
