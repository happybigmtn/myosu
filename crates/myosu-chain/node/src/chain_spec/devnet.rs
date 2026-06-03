use crate::chain_spec::{
    ChainSpec,
    game_solver_spec::{GameSolverGenesisSpec, genesis_with_game_solver},
};
use sc_service::ChainType;

const DEVNET_SUBNET_UID: u16 = 7;
const DEVNET_AUTHORITY_URIS: [&str; 4] = [
    "//myosu//devnet//authority-1",
    "//myosu//devnet//authority-2",
    "//myosu//devnet//authority-3",
    "//myosu//devnet//authority-4",
];
const DEVNET_OPERATOR_URIS: [&str; 4] = [
    "//myosu//devnet//subnet-owner",
    "//myosu//devnet//miner-1",
    "//myosu//devnet//validator-1",
    "//myosu//devnet//validator-2",
];
const DEVNET_SUBNET_OWNER_URI: &str = "//myosu//devnet//subnet-owner";
const DEVNET_SUBNET_OWNER_HOTKEY_URI: &str = "//myosu//devnet//subnet-owner//hotkey";

/// Pre-built [`GameSolverGenesisSpec`] for the devnet chain. Exposed as
/// `pub(crate)` so cross-crate test harnesses can exercise the same shape
/// `devnet_config` builds.
pub(crate) fn devnet_game_solver_spec() -> GameSolverGenesisSpec {
    GameSolverGenesisSpec::with_default_economics(
        "Myosu Devnet",
        "myosu-devnet",
        "myosu-devnet",
        ChainType::Custom("devnet".into()),
        &DEVNET_AUTHORITY_URIS,
        &DEVNET_OPERATOR_URIS,
        DEVNET_SUBNET_OWNER_URI,
        DEVNET_SUBNET_OWNER_HOTKEY_URI,
        DEVNET_SUBNET_UID,
    )
}

/// Builds the devnet chain spec.
pub fn devnet_config() -> Result<ChainSpec, String> {
    genesis_with_game_solver(&devnet_game_solver_spec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain_spec::authority_keys_from_seed;
    use crate::chain_spec::game_solver_spec::build_patch_only_spec;
    #[cfg(feature = "try-runtime")]
    use frame_try_runtime::{UpgradeCheckSelect, runtime_decl_for_try_runtime::TryRuntime};
    use myosu_chain_runtime::Runtime;
    use pallet_game_solver::{
        FirstEmissionBlockNumber, Keys, MinAllowedWeights, NetworksAdded, SubnetOwner,
        SubnetOwnerHotkey, SubnetworkN, SubtokenEnabled, Uids,
    };
    use sp_core::sr25519;
    use sp_io::TestExternalities;
    use sp_runtime::BuildStorage;
    use subtensor_runtime_common::NetUid;

    #[test]
    fn base_devnet_spec_uses_custom_chain_type_and_non_dev_authorities() {
        // The devnet spec is sealed through `genesis_with_game_solver`, which
        // writes the subnet bootstrap into the raw storage map and collapses
        // the typed `runtimeGenesis.patch`. We exercise the patch-only
        // builder here so the test can still read back the patch (this is
        // the same pattern `testnet::tests::testnet_spec_uses_local_chain_type_and_testnet_authority_uris`
        // uses for the testnet proof).
        let spec = build_patch_only_spec(&devnet_game_solver_spec())
            .expect("devnet patch-only spec should build");
        assert_eq!(
            sc_service::ChainSpec::chain_type(&spec),
            ChainType::Custom("devnet".into())
        );

        let chain_spec_json: serde_json::Value =
            serde_json::from_str(&spec.as_json(false).expect("chain spec json")).unwrap();
        let patch = &chain_spec_json["genesis"]["runtimeGenesis"]["patch"];
        let expected_authorities = DEVNET_AUTHORITY_URIS
            .iter()
            .map(|uri| crate::chain_spec::authority_keys_from_uri(uri))
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
    fn devnet_operator_set_endows_both_validators_with_distinct_accounts() {
        // The stage0 multi-validator compose path depends on TWO endowed
        // validator accounts (//myosu//devnet//validator-1 and
        // //myosu//devnet//validator-2) so the second-validator compose
        // service can register/stake/submit-weights against the same miner
        // alongside the first. This test guards the operator set shape: both
        // validator URIs must be present, both must resolve to distinct
        // sr25519 account ids, and the broader operator set must include the
        // //myosu//devnet//miner-1 hotkey that both validators will weight
        // toward. If a future change drops validator-2 from
        // `DEVNET_OPERATOR_URIS`, the second compose service would have
        // nothing to register against and the multi-validator proof would
        // fail to find a target UID for its `set_weights` extrinsic.
        use frame_system::Account;

        let validator_1_uri = "//myosu//devnet//validator-1";
        let validator_2_uri = "//myosu//devnet//validator-2";
        let miner_1_uri = "//myosu//devnet//miner-1";

        assert!(
            DEVNET_OPERATOR_URIS.contains(&validator_1_uri),
            "devnet operator URIs must endow //myosu//devnet//validator-1"
        );
        assert!(
            DEVNET_OPERATOR_URIS.contains(&validator_2_uri),
            "devnet operator URIs must endow //myosu//devnet//validator-2"
        );
        assert!(
            DEVNET_OPERATOR_URIS.contains(&miner_1_uri),
            "devnet operator URIs must endow //myosu//devnet//miner-1"
        );

        let validator_1_account =
            crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(validator_1_uri);
        let validator_2_account =
            crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(validator_2_uri);
        let miner_1_account =
            crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(miner_1_uri);

        assert_ne!(
            validator_1_account, validator_2_account,
            "validator-1 and validator-2 must resolve to distinct sr25519 accounts"
        );
        assert_ne!(
            validator_1_account, miner_1_account,
            "validator-1 and miner-1 must resolve to distinct sr25519 accounts"
        );
        assert_ne!(
            validator_2_account, miner_1_account,
            "validator-2 and miner-1 must resolve to distinct sr25519 accounts"
        );

        // Build the spec and confirm the operator endowments land in the
        // System account store (Balances delegates `AccountStore = System`).
        // The endowments flow through `endowed_accounts()` in
        // `game_solver_spec.rs`; verifying that each operator URI has a
        // non-zero free balance in the resulting externalities is the
        // cheapest way to catch a refactor that drops a row.
        let spec = devnet_config().expect("devnet spec should build");
        let mut ext = TestExternalities::from(spec.build_storage().expect("storage"));

        ext.execute_with(|| {
            for (label, account) in [
                ("validator-1", &validator_1_account),
                ("validator-2", &validator_2_account),
                ("miner-1", &miner_1_account),
                ("subnet-owner", &crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(
                    DEVNET_SUBNET_OWNER_URI,
                )),
            ] {
                let info = Account::<Runtime>::get(account);
                assert!(
                    info.data.free > 0,
                    "devnet genesis must endow {} ({} rao free) in the System account store",
                    label,
                    info.data.free
                );
            }
        });
    }

    #[test]
    fn devnet_config_bootstraps_subnet_seven_in_storage() {
        let spec = devnet_config().expect("devnet spec should build");
        let mut ext = TestExternalities::from(spec.build_storage().expect("storage"));

        ext.execute_with(|| {
            let netuid = NetUid::from(DEVNET_SUBNET_UID);
            let owner_coldkey = crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(
                DEVNET_SUBNET_OWNER_URI,
            );
            let owner_hotkey = crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(
                DEVNET_SUBNET_OWNER_HOTKEY_URI,
            );

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

    #[cfg(feature = "try-runtime")]
    #[test]
    fn devnet_runtime_upgrade_smoke_test_passes_on_fresh_genesis() {
        let spec = devnet_config().expect("devnet spec should build");
        let mut ext = TestExternalities::from(spec.build_storage().expect("storage"));

        ext.execute_with(|| {
            Runtime::on_runtime_upgrade(UpgradeCheckSelect::PreAndPost);
        });
    }
}
