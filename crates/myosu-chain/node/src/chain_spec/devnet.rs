use crate::chain_spec::{ChainSpec, Extensions, authority_keys_from_uri, get_account_id_from_uri};
use myosu_chain_runtime::{INITIAL_SUBNET_TEMPO, Runtime};
use pallet_game_solver::{
    Active, Alpha, BlockAtRegistration, Consensus, Dividends, Emission, FirstEmissionBlockNumber,
    Incentive, IsNetworkMember, Keys, LargestLocked, LastUpdate, MinAllowedWeights,
    NetworkRegisteredAt, NetworksAdded, OwnedHotkeys, Owner, Pallet as GameSolver, PruningScores,
    Rank, StakingHotkeys, SubnetAlphaIn, SubnetAlphaOut, SubnetLocked, SubnetMechanism,
    SubnetOwner, SubnetOwnerHotkey, SubnetTAO, SubnetworkN, SubtokenEnabled, TokenSymbol,
    TotalHotkeyAlpha, TotalHotkeyShares, Trust, Uids, ValidatorPermit, ValidatorTrust,
};
use sc_service::ChainType;
use sp_core::sr25519;
use sp_runtime::BuildStorage;
use sp_state_machine::BasicExternalities;
use substrate_fixed::types::U64F64;
use subtensor_runtime_common::{AlphaCurrency, NetUid, NetUidStorageIndex, TaoCurrency};

const DEVNET_SUBNET_UID: u16 = 7;
const DEVNET_AUTHORITY_URIS: [&str; 3] = [
    "//myosu//devnet//authority-1",
    "//myosu//devnet//authority-2",
    "//myosu//devnet//authority-3",
];
const DEVNET_OPERATOR_URIS: [&str; 4] = [
    "//myosu//devnet//subnet-owner",
    "//myosu//devnet//miner-1",
    "//myosu//devnet//validator-1",
    "//myosu//devnet//validator-2",
];
const DEVNET_SUBNET_OWNER_URI: &str = "//myosu//devnet//subnet-owner";
const DEVNET_SUBNET_OWNER_HOTKEY_URI: &str = "//myosu//devnet//subnet-owner//hotkey";
const DEVNET_AUTHORITY_ENDOWMENT: u128 = 1_000_000_000_000_000;
const DEVNET_OPERATOR_ENDOWMENT: u128 = 250_000_000_000_000;
const DEVNET_HOTKEY_ENDOWMENT: u128 = 2_000_000_000_000;
const DEVNET_SUBNET_POOL_BALANCE: u64 = 10_000_000_000;
const DEVNET_SUBNET_OWNER_STAKE: u64 = 1_000_000_000;
const DEVNET_SUBNET_LOCKED_TAO: u64 = 1;

/// Builds the devnet chain spec.
pub fn devnet_config() -> Result<ChainSpec, String> {
    let mut spec = build_devnet_spec()?;
    bootstrap_devnet_subnet(
        &mut spec,
        get_account_id_from_uri::<sr25519::Public>(DEVNET_SUBNET_OWNER_URI),
        get_account_id_from_uri::<sr25519::Public>(DEVNET_SUBNET_OWNER_HOTKEY_URI),
    )?;
    Ok(spec)
}

fn build_devnet_spec() -> Result<ChainSpec, String> {
    let wasm_binary = crate::chain_spec::WASM_BINARY
        .ok_or_else(|| "development wasm is not available".to_string())?;

    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    let initial_authorities = DEVNET_AUTHORITY_URIS
        .iter()
        .map(|uri| authority_keys_from_uri(uri))
        .collect::<Vec<_>>();

    let mut endowed_accounts = DEVNET_AUTHORITY_URIS
        .iter()
        .map(|uri| {
            (
                get_account_id_from_uri::<sr25519::Public>(uri),
                DEVNET_AUTHORITY_ENDOWMENT,
            )
        })
        .collect::<Vec<_>>();
    endowed_accounts.extend(DEVNET_OPERATOR_URIS.iter().map(|uri| {
        (
            get_account_id_from_uri::<sr25519::Public>(uri),
            DEVNET_OPERATOR_ENDOWMENT,
        )
    }));
    endowed_accounts.push((
        get_account_id_from_uri::<sr25519::Public>(DEVNET_SUBNET_OWNER_HOTKEY_URI),
        DEVNET_HOTKEY_ENDOWMENT,
    ));

    let balances_issuance = endowed_accounts
        .iter()
        .map(|(_, balance)| *balance)
        .sum::<u128>();

    Ok(ChainSpec::builder(wasm_binary, Extensions::default())
        .with_name("Myosu Devnet")
        .with_protocol_id("myosu-devnet")
        .with_id("myosu-devnet")
        .with_chain_type(ChainType::Custom("devnet".into()))
        .with_genesis_config_patch(super::localnet::genesis_patch(
            initial_authorities,
            endowed_accounts,
            Some(serde_json::json!({
                "balancesIssuance": balances_issuance,
            })),
        ))
        .with_properties(properties)
        .build())
}

fn bootstrap_devnet_subnet(
    spec: &mut ChainSpec,
    owner_coldkey: subtensor_runtime_common::AccountId,
    owner_hotkey: subtensor_runtime_common::AccountId,
) -> Result<(), String> {
    let netuid = NetUid::from(DEVNET_SUBNET_UID);
    let mut ext = BasicExternalities::new(
        spec.build_storage()
            .map_err(|error| format!("failed to build devnet storage: {error}"))?,
    );

    ext.execute_with(move || {
        let owner_uid = 0u16;
        let subnet_storage_index = NetUidStorageIndex::from(netuid);
        let registration_block = GameSolver::<Runtime>::get_current_block_as_u64();

        GameSolver::<Runtime>::init_new_network(netuid, INITIAL_SUBNET_TEMPO);
        GameSolver::<Runtime>::set_network_pow_registration_allowed(netuid, true);

        SubnetMechanism::<Runtime>::insert(netuid, 1u16);
        Owner::<Runtime>::insert(owner_hotkey.clone(), owner_coldkey.clone());
        OwnedHotkeys::<Runtime>::insert(owner_coldkey.clone(), vec![owner_hotkey.clone()]);
        StakingHotkeys::<Runtime>::insert(owner_coldkey.clone(), vec![owner_hotkey.clone()]);

        SubnetAlphaIn::<Runtime>::insert(netuid, AlphaCurrency::from(DEVNET_SUBNET_POOL_BALANCE));
        SubnetTAO::<Runtime>::insert(netuid, TaoCurrency::from(DEVNET_SUBNET_POOL_BALANCE));
        SubnetOwner::<Runtime>::insert(netuid, owner_coldkey.clone());
        SubnetOwnerHotkey::<Runtime>::insert(netuid, owner_hotkey.clone());
        NetworkRegisteredAt::<Runtime>::insert(netuid, registration_block);
        SubnetLocked::<Runtime>::insert(netuid, TaoCurrency::from(DEVNET_SUBNET_LOCKED_TAO));
        LargestLocked::<Runtime>::insert(netuid, DEVNET_SUBNET_LOCKED_TAO);
        MinAllowedWeights::<Runtime>::insert(netuid, 0u16);

        Alpha::<Runtime>::insert(
            (owner_hotkey.clone(), owner_coldkey.clone(), netuid),
            U64F64::saturating_from_num(DEVNET_SUBNET_OWNER_STAKE),
        );
        TotalHotkeyAlpha::<Runtime>::insert(
            owner_hotkey.clone(),
            netuid,
            AlphaCurrency::from(DEVNET_SUBNET_OWNER_STAKE),
        );
        TotalHotkeyShares::<Runtime>::insert(
            owner_hotkey.clone(),
            netuid,
            U64F64::saturating_from_num(DEVNET_SUBNET_OWNER_STAKE),
        );
        SubnetAlphaOut::<Runtime>::insert(netuid, AlphaCurrency::from(DEVNET_SUBNET_OWNER_STAKE));

        SubnetworkN::<Runtime>::insert(netuid, 1u16);
        Rank::<Runtime>::insert(netuid, vec![0u16]);
        Trust::<Runtime>::insert(netuid, vec![0u16]);
        Active::<Runtime>::insert(netuid, vec![true]);
        Emission::<Runtime>::insert(netuid, vec![AlphaCurrency::from(0u64)]);
        Consensus::<Runtime>::insert(netuid, vec![0u16]);
        Incentive::<Runtime>::insert(subnet_storage_index, vec![0u16]);
        Dividends::<Runtime>::insert(netuid, vec![0u16]);
        LastUpdate::<Runtime>::insert(subnet_storage_index, vec![registration_block]);
        PruningScores::<Runtime>::insert(netuid, vec![0u16]);
        ValidatorTrust::<Runtime>::insert(netuid, vec![0u16]);
        ValidatorPermit::<Runtime>::insert(netuid, vec![false]);

        Keys::<Runtime>::insert(netuid, owner_uid, owner_hotkey.clone());
        Uids::<Runtime>::insert(netuid, owner_hotkey.clone(), owner_uid);
        BlockAtRegistration::<Runtime>::insert(netuid, owner_uid, registration_block);
        IsNetworkMember::<Runtime>::insert(owner_hotkey.clone(), netuid, true);

        TokenSymbol::<Runtime>::insert(
            netuid,
            GameSolver::<Runtime>::get_symbol_for_subnet(netuid),
        );
        FirstEmissionBlockNumber::<Runtime>::insert(netuid, registration_block);
        SubtokenEnabled::<Runtime>::insert(netuid, true);

        debug_assert!(NetworksAdded::<Runtime>::get(netuid));
    });

    sc_service::ChainSpec::set_storage(spec, ext.into_storages());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain_spec::authority_keys_from_seed;
    use serde_json::Value;
    use sp_io::TestExternalities;

    #[test]
    fn base_devnet_spec_uses_custom_chain_type_and_non_dev_authorities() {
        let spec = build_devnet_spec().expect("devnet spec should build");
        assert_eq!(
            sc_service::ChainSpec::chain_type(&spec),
            ChainType::Custom("devnet".into())
        );

        let chain_spec_json: Value =
            serde_json::from_str(&spec.as_json(false).expect("chain spec json")).unwrap();
        let patch = &chain_spec_json["genesis"]["runtimeGenesis"]["patch"];
        let expected_authorities = DEVNET_AUTHORITY_URIS
            .iter()
            .map(|uri| authority_keys_from_uri(uri))
            .collect::<Vec<_>>();
        let dev_authorities = ["Alice", "Bob", "Charlie"]
            .into_iter()
            .map(authority_keys_from_seed)
            .collect::<Vec<_>>();

        assert_eq!(
            patch["aura"]["authorities"],
            serde_json::to_value(
                expected_authorities
                    .iter()
                    .map(|(aura, _)| aura)
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );
        assert_eq!(
            patch["grandpa"]["authorities"],
            serde_json::to_value(
                expected_authorities
                    .iter()
                    .map(|(_, grandpa)| (grandpa, 1u64))
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );
        assert_ne!(expected_authorities, dev_authorities);
    }

    #[test]
    fn devnet_config_bootstraps_subnet_seven_in_storage() {
        let spec = devnet_config().expect("devnet spec should build");
        let mut ext = TestExternalities::from(spec.build_storage().expect("storage"));

        ext.execute_with(|| {
            let netuid = NetUid::from(DEVNET_SUBNET_UID);
            let owner_coldkey = get_account_id_from_uri::<sr25519::Public>(DEVNET_SUBNET_OWNER_URI);
            let owner_hotkey =
                get_account_id_from_uri::<sr25519::Public>(DEVNET_SUBNET_OWNER_HOTKEY_URI);

            assert!(NetworksAdded::<Runtime>::get(netuid));
            assert_eq!(SubnetOwner::<Runtime>::get(netuid), owner_coldkey);
            assert_eq!(
                SubnetOwnerHotkey::<Runtime>::get(netuid),
                owner_hotkey.clone()
            );
            assert_eq!(SubnetworkN::<Runtime>::get(netuid), 1);
            assert_eq!(Uids::<Runtime>::get(netuid, owner_hotkey.clone()), Some(0));
            assert_eq!(Keys::<Runtime>::get(netuid, 0), owner_hotkey);
            assert_eq!(FirstEmissionBlockNumber::<Runtime>::get(netuid), Some(0));
            assert!(SubtokenEnabled::<Runtime>::get(netuid));
            assert_eq!(
                MinAllowedWeights::<Runtime>::get(netuid),
                0,
                "devnet subnet should start without a minimum-weights gate"
            );
        });
    }
}
