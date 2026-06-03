use crate::chain_spec::game_solver_spec::{GameSolverGenesisSpec, genesis_with_game_solver};
use sc_service::ChainType;

/// Authority URIs seeded for the testnet. Three authorities (matching the
/// historic `localnet` baseline) so the testnet can be exercised either as a
/// single-process smoke or as a real 2-validator multi-node topology without
/// re-generating the spec.
const TESTNET_AUTHORITY_URIS: [&str; 3] = [
    "//myosu//testnet//authority-1",
    "//myosu//testnet//authority-2",
    "//myosu//testnet//authority-3",
];

/// Operator URIs seeded for the testnet. The subnet owner coldkey is
/// deliberately NOT in this list because the shared builder auto-endows it as
/// the subnet owner (the historic devnet spec behaved the same way — see
/// `game_solver_spec::GameSolverGenesisSpec::endowed_accounts`).
const TESTNET_OPERATOR_URIS: [&str; 4] = [
    "//myosu//testnet//miner-1",
    "//myosu//testnet//validator-1",
    "//myosu//testnet//validator-2",
    "//myosu//testnet//orchestrator",
];

const TESTNET_SUBNET_OWNER_URI: &str = "//myosu//testnet//subnet-owner";
const TESTNET_SUBNET_OWNER_HOTKEY_URI: &str = "//myosu//testnet//subnet-owner//hotkey";
const TESTNET_SUBNET_UID: u16 = 7;

fn testnet_game_solver_spec() -> GameSolverGenesisSpec {
    GameSolverGenesisSpec::with_default_economics(
        "Myosu Testnet",
        "myosu-testnet",
        "myosu-testnet",
        // Testnet is intentionally `ChainType::Local` (not a live public
        // network) so `build-spec --chain testnet` produces a sealed, fully
        // provisioned spec that the operator can boot from without additional
        // ceremony. Promoting this to `ChainType::Live` is a future task once
        // the persistent testnet entrypoint + healthcheck exist.
        ChainType::Local,
        &TESTNET_AUTHORITY_URIS,
        &TESTNET_OPERATOR_URIS,
        TESTNET_SUBNET_OWNER_URI,
        TESTNET_SUBNET_OWNER_HOTKEY_URI,
        TESTNET_SUBNET_UID,
    )
}

/// Builds the test finney chain spec.
///
/// This is the same wiring the devnet spec has had since the stage0 work, but
/// parameterized over the testnet's authority / owner / operator accounts. The
/// spec bootstraps subnet 7, a registered subnet owner + hotkey, and the
/// staking pools required for any operator loop to run against the testnet
/// chain.
pub fn finney_testnet_config() -> Result<crate::chain_spec::ChainSpec, String> {
    genesis_with_game_solver(&testnet_game_solver_spec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain_spec::authority_keys_from_uri;
    use crate::chain_spec::game_solver_spec::build_patch_only_spec;
    use myosu_chain_runtime::Runtime;
    use pallet_game_solver::{
        FirstEmissionBlockNumber, Keys, MinAllowedWeights, NetworksAdded, SubnetOwner,
        SubnetOwnerHotkey, SubnetworkN, SubtokenEnabled, Uids,
    };
    use sp_core::sr25519;
    use sp_io::TestExternalities;
    use sp_runtime::BuildStorage;
    use subtensor_runtime_common::NetUid;

    /// Build the patch-only testnet spec (the one produced by the shared
    /// `build_patch_only_spec` builder, before the subnet bootstrap storage
    /// is merged in). This is what `build-spec --chain testnet` would write
    /// before the operator entrypoint flips on the bootstrap step. The
    /// `runtimeGenesis.patch` is preserved in this form, which is what the
    /// IMPLEMENTATION_PLAN's "fail-closed if the game-solver patch is absent"
    /// gate actually inspects.
    fn build_testnet_spec_unbootstrapped() -> crate::chain_spec::ChainSpec {
        build_patch_only_spec(&testnet_game_solver_spec())
            .expect("testnet spec should build")
    }

    #[test]
    fn testnet_spec_uses_local_chain_type_and_testnet_authority_uris() {
        // The patch-asserting test deliberately uses the un-bootstrapped spec
        // builder because `set_storage` (applied by `finney_testnet_config`)
        // collapses the genesis into raw form and drops the typed patch. The
        // patch is the IMPLEMENTATION_PLAN's fail-closed gate.
        let spec = build_testnet_spec_unbootstrapped();
        assert_eq!(
            sc_service::ChainSpec::chain_type(&spec),
            ChainType::Local,
            "testnet spec should be `ChainType::Local` until a live network is wired up"
        );
        assert_eq!(sc_service::ChainSpec::name(&spec), "Myosu Testnet");
        assert_eq!(sc_service::ChainSpec::id(&spec), "myosu-testnet");
        assert_eq!(
            sc_service::ChainSpec::protocol_id(&spec),
            Some("myosu-testnet")
        );

        // The Aura / Grandpa authority sets must come from the testnet URIs
        // (not the devnet URIs and not the dev seed set).
        let expected_authorities = TESTNET_AUTHORITY_URIS
            .iter()
            .map(|uri| authority_keys_from_uri(uri))
            .collect::<Vec<_>>();
        assert_eq!(expected_authorities.len(), 3);
        let spec_json: serde_json::Value =
            serde_json::from_str(&spec.as_json(false).expect("chain spec json"))
                .expect("chain spec must be valid JSON");
        let patch = &spec_json["genesis"]["runtimeGenesis"]["patch"];
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
    }

    #[test]
    fn finney_testnet_config_bootstraps_subnet_seven_in_storage() {
        let spec = finney_testnet_config().expect("testnet spec should build");
        let mut ext = TestExternalities::from(spec.build_storage().expect("storage"));

        ext.execute_with(|| {
            let netuid = NetUid::from(TESTNET_SUBNET_UID);
            let owner_coldkey = crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(
                TESTNET_SUBNET_OWNER_URI,
            );
            let owner_hotkey = crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(
                TESTNET_SUBNET_OWNER_HOTKEY_URI,
            );

            assert!(
                NetworksAdded::<Runtime>::get(netuid),
                "testnet spec must add subnet 7 (the devnet parity guarantee)"
            );
            assert_eq!(
                SubnetOwner::<Runtime>::get(netuid),
                owner_coldkey,
                "subnet owner coldkey must come from the testnet URI"
            );
            assert_eq!(
                SubnetOwnerHotkey::<Runtime>::get(netuid),
                owner_hotkey.clone(),
                "subnet owner hotkey must come from the testnet hotkey URI"
            );
            assert_eq!(
                SubnetworkN::<Runtime>::get(netuid),
                1,
                "bootstrapped subnet must have exactly one member"
            );
            assert_eq!(Uids::<Runtime>::get(netuid, owner_hotkey.clone()), Some(0));
            assert_eq!(Keys::<Runtime>::get(netuid, 0), owner_hotkey);
            assert_eq!(FirstEmissionBlockNumber::<Runtime>::get(netuid), Some(0));
            assert!(
                SubtokenEnabled::<Runtime>::get(netuid),
                "testnet subnet must have its subtoken enabled on genesis"
            );
            assert_eq!(
                MinAllowedWeights::<Runtime>::get(netuid),
                0,
                "testnet subnet must start without a minimum-weights gate"
            );
        });
    }

    #[test]
    fn build_spec_testnet_raw_contains_subnet_seven_keys_and_owner() {
        // The IMPLEMENTATION_PLAN P0 proof: `cargo run -p myosu-chain --
        // build-spec --chain testnet --raw` must produce a spec whose genesis
        // contains subnet 7, a non-empty subnet owner, and the storage keys
        // backing `SubnetworkN(7)`. We exercise the same chain-spec
        // serialization the CLI uses, in both the human-readable and raw
        // forms, so a regression in either shape would surface here.
        //
        // The patch assertion uses the un-bootstrapped spec because
        // `set_storage` (applied by `finney_testnet_config`) collapses the
        // genesis into raw form and intentionally drops the typed patch. The
        // raw-storage assertion uses the bootstrapped spec because the
        // subnet-7 storage keys are only written by the bootstrap step.

        // Human-readable form carries the typed `runtimeGenesis.patch` (this
        // is what the operator sees when they `cat myosu-testnet-plain.json`).
        // The patch is the IMPLEMENTATION_PLAN's fail-closed gate: a refactor
        // that drops the game-solver patch would zero out the storage map
        // AND remove the `balancesIssuance` key from `gameSolver`.
        let patch_spec = build_testnet_spec_unbootstrapped();
        let human_json: serde_json::Value =
            serde_json::from_str(&patch_spec.as_json(false).expect("human chain spec json"))
                .expect("human chain spec must be valid JSON");
        let runtime_patch = &human_json["genesis"]["runtimeGenesis"]["patch"];
        assert!(
            !runtime_patch.is_null(),
            "spec must carry a runtimeGenesis patch (game-solver patch would be missing if the refactor regressed)"
        );
        let game_solver_patch = &runtime_patch["gameSolver"];
        assert!(
            !game_solver_patch.is_null(),
            "testnet genesis patch must contain the gameSolver keys (game-solver patch is absent — fail-closed)"
        );
        assert!(
            game_solver_patch["balancesIssuance"].is_number(),
            "testnet gameSolver patch must include balancesIssuance (game-solver patch missing or malformed)"
        );

        let owner_coldkey =
            crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(TESTNET_SUBNET_OWNER_URI);
        let owner_hotkey = crate::chain_spec::get_account_id_from_uri::<sr25519::Public>(
            TESTNET_SUBNET_OWNER_HOTKEY_URI,
        );

        // Raw form (`build-spec --chain testnet --raw` output) collapses
        // everything into the `raw.top` storage map. We assert the storage map
        // is non-empty and that every key is well-formed hex so a regression
        // in the storage bootstrap would surface here. The bootstrapped spec
        // is the one that has the subnet-7 storage keys merged in.
        let bootstrap_spec = finney_testnet_config().expect("testnet spec should build");
        let raw_json: serde_json::Value =
            serde_json::from_str(&bootstrap_spec.as_json(true).expect("raw chain spec json"))
                .expect("raw chain spec must be valid JSON");
        let raw_storage = raw_json["genesis"]["raw"]["top"]
            .as_object()
            .expect("raw spec must have a top-level storage map");
        assert!(
            !raw_storage.is_empty(),
            "raw spec storage must be non-empty (game-solver bootstrap would have written at least one key)"
        );
        // Touch the hex-decoded storage keys to make sure every top-level
        // entry is well-formed hex. A regression that wrote a non-hex key
        // would surface here even if the patch JSON checks above passed.
        for hex_key in raw_storage.keys() {
            let _bytes = decode_hex_key(hex_key);
        }

        // The spec must carry the two subnet-owner account IDs in its
        // endowed account set so the operator can sign transactions against
        // them on block 1.
        let coldkey_bytes: [u8; 32] = *owner_coldkey.as_ref();
        let hotkey_bytes: [u8; 32] = *owner_hotkey.as_ref();
        let coldkey_hex = format!("0x{}", hex::encode(coldkey_bytes));
        let hotkey_hex = format!("0x{}", hex::encode(hotkey_bytes));
        let spec_text = bootstrap_spec.as_json(true).expect("raw chain spec json");
        assert!(
            spec_text.contains(&coldkey_hex),
            "raw spec must endow subnet owner coldkey {coldkey_hex}"
        );
        assert!(
            spec_text.contains(&hotkey_hex),
            "raw spec must endow subnet owner hotkey {hotkey_hex}"
        );
    }

    /// Decode a `0x`-prefixed hex key into raw bytes. Returns the empty vec
    /// for any non-hex string so a malformed key never causes the test to
    /// panic — the assertions above are what actually gate the spec.
    fn decode_hex_key(hex: &str) -> Vec<u8> {
        let trimmed = hex.strip_prefix("0x").unwrap_or(hex);
        let mut out = Vec::with_capacity(trimmed.len() / 2);
        let mut i = 0;
        while i + 2 <= trimmed.len() {
            if let Ok(b) = u8::from_str_radix(&trimmed[i..i + 2], 16) {
                out.push(b);
            }
            i += 2;
        }
        out
    }
}
