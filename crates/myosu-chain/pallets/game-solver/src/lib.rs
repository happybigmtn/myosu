#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::zero_prefixed_literal)]

extern crate alloc;

use alloc::vec::Vec;

use frame_support::{BoundedVec, pallet_prelude::ConstU32};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

pub mod coinbase;
pub mod epoch;
pub mod extensions;
pub mod guards;
pub mod migrations;
pub mod rpc_info;
pub mod staking;
pub mod stubs;
pub mod subnets;
pub mod swap;
pub mod swap_stub;
pub mod utils;

pub use pallet::*;
pub use stubs::{
    AuthorshipProvider, AuthorshipStub, CheckColdkeySwap, ColdkeySwapStub,
    CommitmentsInterface, CommitmentsStub, ProxyInterface, ProxyStub,
};
pub use swap_stub::{
    DefaultPriceLimit, GetAlphaForTao, GetTaoForAlpha, NoOpSwap, SwapBalance, SwapEngine,
    SwapHandler, SwapResult,
};
pub use utils::RateLimitKey;

pub type NetUid = u16;
pub type Balance = u64;

pub type AxonInfoOf = AxonInfo;
pub type PrometheusInfoOf = PrometheusInfo;
pub type NeuronCertificateOf = NeuronCertificate;

#[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
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

#[derive(Decode, Encode, Default, TypeInfo, PartialEq, Eq, Clone, Debug)]
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
        let public_key = value.get(1..).ok_or(())?.to_vec();

        Ok(Self {
            public_key: BoundedVec::try_from(public_key).map_err(|_| ())?,
            algorithm,
        })
    }
}

#[derive(Encode, Decode, Default, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct PrometheusInfo {
    pub block: u64,
    pub version: u32,
    pub ip: u128,
    pub port: u16,
    pub ip_type: u8,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(_);

    #[pallet::event]
    pub enum Event<T: Config> {
        PhaseOneReady,
    }

    #[pallet::error]
    pub enum Error<T> {
        PhaseOneOnly,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

#[cfg(test)]
mod phase_one_tests {
    use super::{NeuronCertificate, RateLimitKey};
    use parity_scale_codec::{Decode, Encode};

    #[test]
    fn neuron_certificate_rejects_oversized_payloads() {
        assert!(NeuronCertificate::try_from(vec![0; 66]).is_err());
    }

    #[test]
    fn neuron_certificate_accepts_algorithm_plus_key_bytes() {
        let cert = NeuronCertificate::try_from(vec![7, 1, 2, 3]).expect("certificate should fit");
        assert_eq!(cert.algorithm, 7);
        assert_eq!(cert.public_key.into_inner(), vec![1, 2, 3]);
    }

    #[test]
    fn rate_limit_key_round_trips() {
        let encoded = RateLimitKey::ServeAxon(9).encode();
        let decoded = RateLimitKey::decode(&mut &encoded[..]).expect("key should decode");
        assert_eq!(decoded, RateLimitKey::ServeAxon(9));
    }
}
