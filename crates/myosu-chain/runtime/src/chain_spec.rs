//! Minimal chain spec for development and testing.

use sp_runtime::BuildStorage;
use myosu_runtime::{
    AccountId, BalancesConfig, GenesisConfig, RuntimeEvent, SudoConfig, SystemConfig,
    TimestampConfig, WASM_BINARY,
};

/// Development chain spec — single validator, no existential deposit worry.
pub fn dev_config() -> Result<GenesisConfig, String> {
    let endowed_accounts = vec![];
    Ok(GenesisConfig {
        system: SystemConfig {
            code: WASM_BINARY.ok_or("WASM binary not available")?,
            ..Default::default()
        },
        balances: BalancesConfig {
            balances: endowed_accounts
                .into_iter()
                .map(|acc| (acc, 1_000_000_000_000_u64))
                .collect(),
        },
        sudo: SudoConfig {
            key: Some(AccountId::from([1u8; 32])),
        },
        timestamp: TimestampConfig {
            minimum_period: 1000,
        },
        ..Default::default()
    })
}
