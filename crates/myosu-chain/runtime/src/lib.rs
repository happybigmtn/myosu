#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;
use codec::{Compact, CompactAs, Decode, Encode};
use core::fmt::{self, Display, Formatter};
use frame_support::{construct_runtime, derive_impl, parameter_types};
use scale_info::TypeInfo;
use sp_version::{NativeVersion, RuntimeVersion, runtime_version};

/// Domain subnet identifier retained from the prior runtime surface.
#[repr(transparent)]
#[derive(
    Clone, Copy, Debug, Default, Decode, Encode, Eq, Hash, Ord, PartialEq, PartialOrd, TypeInfo,
)]
pub struct NetUid(u16);

impl NetUid {
    pub const fn new(inner: u16) -> Self {
        Self(inner)
    }
}

impl Display for NetUid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl CompactAs for NetUid {
    type As = u16;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(value: Self::As) -> Result<Self, codec::Error> {
        Ok(Self(value))
    }
}

impl From<Compact<NetUid>> for NetUid {
    fn from(value: Compact<NetUid>) -> Self {
        value.0
    }
}

impl From<NetUid> for u16 {
    fn from(value: NetUid) -> Self {
        value.0
    }
}

impl From<u16> for NetUid {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

#[runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: sp_version::create_runtime_str!("myosu"),
    impl_name: sp_version::create_runtime_str!("myosu-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: sp_version::create_apis_vec!([]),
    transaction_version: 1,
    state_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

construct_runtime!(
    pub enum Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        Sudo: pallet_sudo,
    }
);

pub type Block = frame_system::mocking::MockBlock<Runtime>;
pub type Header = <Block as sp_runtime::traits::Block>::Header;
pub type RuntimeExecutive =
    frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    type Block = Block;
    type Version = Version;
    type AccountData = pallet_balances::AccountData<<Runtime as pallet_balances::Config>::Balance>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Runtime {
    type AccountStore = System;
}

#[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
impl pallet_sudo::Config for Runtime {}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Runtime {}

/// Re-exports the node-side runtime interface types expected by future slices.
pub mod interface {
    use super::Runtime;

    pub type Block = super::Block;
    pub type AccountId = <Runtime as frame_system::Config>::AccountId;
    pub type Nonce = <Runtime as frame_system::Config>::Nonce;
    pub type Hash = <Runtime as frame_system::Config>::Hash;
    pub type Balance = <Runtime as pallet_balances::Config>::Balance;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_uses_myosu_identity() {
        assert_eq!(core::str::from_utf8(VERSION.spec_name.as_ref()), Ok("myosu"));
        assert_eq!(
            core::str::from_utf8(VERSION.impl_name.as_ref()),
            Ok("myosu-runtime")
        );
        assert_eq!(VERSION.spec_version, 1);
    }

    #[test]
    fn netuid_roundtrips_through_compact() {
        let encoded = Compact(NetUid::new(7)).encode();
        let decoded = Compact::<NetUid>::decode(&mut &encoded[..]).expect("compact decode works");
        assert_eq!(NetUid::from(decoded), NetUid::new(7));
    }
}
