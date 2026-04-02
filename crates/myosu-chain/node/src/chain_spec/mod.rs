#![allow(clippy::expect_used, clippy::unwrap_used)]

pub mod devnet;
pub mod finney;
pub mod localnet;
pub mod testnet;

use myosu_chain_runtime::{Block, WASM_BINARY};
use sc_chain_spec_derive::ChainSpecExtension;
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use subtensor_runtime_common::{AccountId, Signature};

/// Node `ChainSpec` extensions.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Block numbers with known hashes.
    pub fork_blocks: sc_client_api::ForkBlocks<Block>,
    /// Known bad block hashes.
    pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec` type for the current node restore.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

type AccountPublic = <Signature as Verify>::Signer;

/// Generate a crypto pair from a dev seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static development seeds are valid")
        .public()
}

/// Generate a crypto pair from a full secret URI.
pub fn get_from_uri<TPublic: Public>(uri: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(uri, None)
        .expect("static chain spec secret URIs are valid")
        .public()
}

/// Generate an account ID from a dev seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an account ID from a full secret URI.
pub fn get_account_id_from_uri<TPublic: Public>(uri: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_uri::<TPublic>(uri)).into_account()
}

/// Generate Aura and Grandpa authority keys from the same dev seed.
pub fn authority_keys_from_seed(seed: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
    )
}

/// Generate Aura and Grandpa authority keys from a full secret URI.
pub fn authority_keys_from_uri(uri: &str) -> (AuraId, GrandpaId) {
    (get_from_uri::<AuraId>(uri), get_from_uri::<GrandpaId>(uri))
}
