#![allow(clippy::unwrap_used)]

use crate::chain_spec::{
    ChainSpec, Extensions, authority_keys_from_seed, get_account_id_from_seed,
};
use sc_service::ChainType;
use sp_core::sr25519;
use subtensor_runtime_common::AccountId;

/// Builds the local development chain spec.
pub fn localnet_config(single_authority: bool) -> Result<ChainSpec, String> {
    let wasm_binary = crate::chain_spec::WASM_BINARY
        .ok_or_else(|| "development wasm is not available".to_string())?;

    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    Ok(ChainSpec::builder(wasm_binary, Extensions::default())
        .with_name("Myosu Local")
        .with_protocol_id("myosu-local")
        .with_id("myosu-local")
        .with_chain_type(ChainType::Local)
        .with_genesis_config_patch(localnet_genesis(if single_authority {
            vec![authority_keys_from_seed("Alice")]
        } else {
            vec![
                authority_keys_from_seed("Alice"),
                authority_keys_from_seed("Bob"),
                authority_keys_from_seed("Charlie"),
            ]
        }))
        .with_properties(properties)
        .build())
}

pub(super) fn localnet_balances() -> Vec<(AccountId, u128)> {
    vec![
        (
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            1_000_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            1_000_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Charlie"),
            1_000_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Dave"),
            2_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Eve"),
            2_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Ferdie"),
            2_000_000_000_000u128,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
            2_000_000_000_000u128,
        ),
    ]
}

pub(super) fn localnet_genesis(
    initial_authorities: Vec<(
        sp_consensus_aura::sr25519::AuthorityId,
        sp_consensus_grandpa::AuthorityId,
    )>,
) -> serde_json::Value {
    genesis_patch(initial_authorities, localnet_balances(), None)
}

pub(super) fn genesis_patch(
    initial_authorities: Vec<(
        sp_consensus_aura::sr25519::AuthorityId,
        sp_consensus_grandpa::AuthorityId,
    )>,
    endowed_accounts: Vec<(AccountId, u128)>,
    game_solver: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut genesis = serde_json::json!({
        "aura": {
            "authorities": initial_authorities
                .iter()
                .map(|(aura, _)| aura.clone())
                .collect::<Vec<_>>()
        },
        "balances": {
            "balances": endowed_accounts,
        },
        "grandpa": {
            "authorities": initial_authorities
                .iter()
                .map(|(_, grandpa)| (grandpa.clone(), 1))
                .collect::<Vec<_>>()
        }
    });

    if let Some(game_solver) = game_solver {
        genesis
            .as_object_mut()
            .expect("genesis patch must stay as an object")
            // The default-build runtime still deserializes this inherited
            // genesis field name even though live storage is under GameSolver.
            .insert("subtensorModule".into(), game_solver);
    }

    genesis
}
