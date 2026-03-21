#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::pallet_prelude::{BoundedVec, MaxEncodedLen};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::ConstU32;

pub mod epoch;
pub mod guards;
pub mod macros;
#[cfg(test)]
mod phase2_tests;
pub mod staking;
pub mod stubs;
pub mod subnets;
pub mod swap_stub;
pub mod utils;

pub type NetUid = u16;
pub type Balance = u64;

pub trait Currency {
    type Balance;

    fn zero() -> Self::Balance;
    fn saturating_add(a: Self::Balance, b: Self::Balance) -> Self::Balance;
    fn saturating_sub(a: Self::Balance, b: Self::Balance) -> Self::Balance;
}

pub struct SingleTokenCurrency;

impl Currency for SingleTokenCurrency {
    type Balance = Balance;

    fn zero() -> Self::Balance {
        0
    }

    fn saturating_add(a: Self::Balance, b: Self::Balance) -> Self::Balance {
        a.saturating_add(b)
    }

    fn saturating_sub(a: Self::Balance, b: Self::Balance) -> Self::Balance {
        a.saturating_sub(b)
    }
}

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum Hyperparameter {
    Unknown = 0,
    ServingRateLimit = 1,
    MaxDifficulty = 2,
    AdjustmentAlpha = 3,
    MaxWeightLimit = 4,
    ImmunityPeriod = 5,
    MinAllowedWeights = 6,
    Kappa = 7,
    Rho = 8,
    ActivityCutoff = 9,
    PowRegistrationAllowed = 10,
    MinBurn = 11,
    MaxBurn = 12,
    BondsMovingAverage = 13,
    BondsPenalty = 14,
    CommitRevealEnabled = 15,
    LiquidAlphaEnabled = 16,
    AlphaValues = 17,
    WeightCommitInterval = 18,
    TransferEnabled = 19,
    AlphaSigmoidSteepness = 20,
    Yuma3Enabled = 21,
    BondsResetEnabled = 22,
    ImmuneNeuronLimit = 23,
    RecycleOrBurn = 24,
    MaxAllowedUids = 25,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum RateLimitKey<AccountId> {
    LastTxBlock(AccountId),
    LastTxBlockDelegateTake(AccountId),
    LastTxBlockChildKeyTake(AccountId),
    SetSNOwnerHotkey(NetUid),
    OwnerHyperparamUpdate(NetUid, Hyperparameter),
    Transaction(AccountId, NetUid, u16),
}

#[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug, MaxEncodedLen)]
pub struct AxonInfo {
    pub block: u64,
    pub version: u32,
    pub ip: u128,
    pub port: u16,
    pub ip_type: u8,
    pub protocol: u8,
    pub placeholder1: u8,
    pub placeholder2: u8,
}

#[derive(Decode, Encode, Default, TypeInfo, PartialEq, Eq, Clone, Debug, MaxEncodedLen)]
pub struct NeuronCertificate {
    pub public_key: BoundedVec<u8, ConstU32<64>>,
    pub algorithm: u8,
}

impl TryFrom<Vec<u8>> for NeuronCertificate {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() > 65 {
            return Err(());
        }

        let algorithm = *value.first().ok_or(())?;
        let certificate = value.get(1..).ok_or(())?.to_vec();

        Ok(Self {
            public_key: BoundedVec::try_from(certificate).map_err(|_| ())?,
            algorithm,
        })
    }
}

#[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug, MaxEncodedLen)]
pub struct PrometheusInfo {
    pub block: u64,
    pub version: u32,
    pub ip: u128,
    pub port: u16,
    pub ip_type: u8,
}

#[frame_support::pallet]
pub mod pallet {
    use super::{AxonInfo, NetUid, NeuronCertificate, PrometheusInfo, RateLimitKey};
    use alloc::vec::Vec;
    use frame_support::{pallet_prelude::*, weights::Weight};
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use scale_info::TypeInfo;
    use sp_runtime::traits::{
        AtLeast32BitUnsigned, MaybeSerializeDeserialize, SaturatedConversion,
    };

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SubnetRegistered(NetUid, T::AccountId),
        NeuronRegistered(NetUid, u16, T::AccountId),
        AxonServed(NetUid, T::AccountId),
        PrometheusServed(NetUid, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        SubnetNotFound,
        NetUidExhausted,
        HotkeyAlreadyRegistered,
        HotkeyOwnedByDifferentColdkey,
        HotkeyNotRegisteredInSubnet,
        InvalidIpType,
        InvalidIpAddress,
        InvalidPort,
        InvalidCertificate,
    }

    #[pallet::storage]
    #[pallet::getter(fn hotkey_uid)]
    pub type Keys<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, NetUid, Blake2_128Concat, T::AccountId, u16>;

    #[pallet::storage]
    #[pallet::getter(fn axon_info)]
    pub type Axons<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        AxonInfo,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn prometheus_info)]
    pub type Prometheus<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        PrometheusInfo,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn neuron_certificate)]
    pub type NeuronCertificates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        NetUid,
        Blake2_128Concat,
        T::AccountId,
        NeuronCertificate,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn owner)]
    pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn delegate_take)]
    pub type Delegates<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u16, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn subnet_owner)]
    pub type SubnetOwner<T: Config> = StorageMap<_, Blake2_128Concat, NetUid, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn next_subnet_uid)]
    pub type NextSubnetUid<T: Config> = StorageValue<_, NetUid, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_neuron_uid)]
    pub type NextNeuronUid<T: Config> = StorageMap<_, Blake2_128Concat, NetUid, u16, ValueQuery>;

    #[pallet::storage]
    pub type RateLimitedLastBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, RateLimitKey<T::AccountId>, u64, ValueQuery>;

    #[pallet::storage]
    pub type NetworkLastLockBlock<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn register_subnet(origin: OriginFor<T>) -> DispatchResult {
            let owner = ensure_signed(origin)?;
            let netuid = Self::do_register_subnet(&owner)?;
            Self::deposit_event(Event::SubnetRegistered(netuid, owner));
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn register_hotkey(
            origin: OriginFor<T>,
            netuid: NetUid,
            hotkey: T::AccountId,
        ) -> DispatchResult {
            let coldkey = ensure_signed(origin)?;
            let uid = Self::do_register_hotkey(netuid, &coldkey, &hotkey)?;
            Self::deposit_event(Event::NeuronRegistered(netuid, uid, hotkey));
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn serve_axon(
            origin: OriginFor<T>,
            netuid: NetUid,
            version: u32,
            ip: u128,
            port: u16,
            ip_type: u8,
            protocol: u8,
            placeholder1: u8,
            placeholder2: u8,
            certificate: Option<Vec<u8>>,
        ) -> DispatchResult {
            let hotkey = ensure_signed(origin)?;
            let certificate = certificate
                .map(NeuronCertificate::try_from)
                .transpose()
                .map_err(|_| Error::<T>::InvalidCertificate)?;
            let axon = AxonInfo {
                block: Self::get_current_block_as_u64(),
                version,
                ip,
                port,
                ip_type,
                protocol,
                placeholder1,
                placeholder2,
            };
            Self::do_serve_axon(netuid, &hotkey, axon, certificate)?;
            Self::deposit_event(Event::AxonServed(netuid, hotkey));
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn serve_prometheus(
            origin: OriginFor<T>,
            netuid: NetUid,
            version: u32,
            ip: u128,
            port: u16,
            ip_type: u8,
        ) -> DispatchResult {
            let hotkey = ensure_signed(origin)?;
            let info = PrometheusInfo {
                block: Self::get_current_block_as_u64(),
                version,
                ip,
                port,
                ip_type,
            };
            Self::do_serve_prometheus(netuid, &hotkey, info)?;
            Self::deposit_event(Event::PrometheusServed(netuid, hotkey));
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn get_current_block_as_u64() -> u64 {
            frame_system::Pallet::<T>::block_number().saturated_into::<u64>()
        }

        pub fn subnet_exists(netuid: NetUid) -> bool {
            SubnetOwner::<T>::contains_key(netuid)
        }

        pub fn set_rate_limited_last_block(key: &RateLimitKey<T::AccountId>, block: u64) {
            RateLimitedLastBlock::<T>::insert(key, block);
        }

        pub fn get_rate_limited_last_block(key: &RateLimitKey<T::AccountId>) -> u64 {
            RateLimitedLastBlock::<T>::get(key)
        }

        pub fn remove_rate_limited_last_block(key: &RateLimitKey<T::AccountId>) {
            RateLimitedLastBlock::<T>::remove(key);
        }

        pub fn set_network_last_lock_block(block: u64) {
            NetworkLastLockBlock::<T>::put(block);
        }

        pub fn get_network_last_lock_block() -> u64 {
            NetworkLastLockBlock::<T>::get()
        }
    }
}

pub use pallet::*;
