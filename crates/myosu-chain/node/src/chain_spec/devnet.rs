use crate::chain_spec::{ChainSpec, Extensions, authority_keys_from_seed};
use sc_service::ChainType;

/// Builds the devnet chain spec.
pub fn devnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = crate::chain_spec::WASM_BINARY
        .ok_or_else(|| "development wasm is not available".to_string())?;

    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    Ok(ChainSpec::builder(wasm_binary, Extensions::default())
        .with_name("Myosu Devnet")
        .with_protocol_id("myosu-devnet")
        .with_id("myosu-devnet")
        .with_chain_type(ChainType::Local)
        .with_genesis_config_patch(super::localnet::localnet_genesis(vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
            authority_keys_from_seed("Charlie"),
        ]))
        .with_properties(properties)
        .build())
}
