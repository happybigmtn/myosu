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
    use super::RateLimitKey;
    use frame_support::pallet_prelude::*;
    use scale_info::TypeInfo;
    use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, SaturatedConversion};

    #[pallet::config]
    pub trait Config: frame_system::Config {
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

    #[pallet::storage]
    pub type RateLimitedLastBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, RateLimitKey<T::AccountId>, u64, ValueQuery>;

    #[pallet::storage]
    pub type NetworkLastLockBlock<T: Config> = StorageValue<_, u64, ValueQuery>;

    impl<T: Config> Pallet<T> {
        pub fn get_current_block_as_u64() -> u64 {
            frame_system::Pallet::<T>::block_number().saturated_into::<u64>()
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
