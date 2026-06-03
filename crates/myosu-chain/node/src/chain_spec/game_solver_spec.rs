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
use subtensor_runtime_common::{AccountId, AlphaCurrency, NetUid, NetUidStorageIndex, TaoCurrency};

/// Endowments (RAO) for the bootstrap authorities. Matches the historic devnet
/// endownments so refactors are bit-identical against the working devnet
/// compose proof.
pub const DEFAULT_AUTHORITY_ENDOWMENT: u128 = 1_000_000_000_000_000;

/// Endowments (RAO) for the bootstrap operators (subnet owner, miner, validators).
pub const DEFAULT_OPERATOR_ENDOWMENT: u128 = 250_000_000_000_000;

/// Endowments (RAO) for the standalone hotkey accounts (separate from the cold
/// operator accounts above). Mirrors devnet so the refactor is regression-safe.
pub const DEFAULT_HOTKEY_ENDOWMENT: u128 = 2_000_000_000_000;

/// Subnet pool balance seeded into both `SubnetAlphaIn` and `SubnetTAO` for the
/// bootstrapped subnet.
pub const DEFAULT_SUBNET_POOL_BALANCE: u64 = 10_000_000_000;

/// Stake placed against the subnet owner's hotkey to back the `SubnetAlphaOut`
/// / `TotalHotkeyAlpha` accounting on genesis.
pub const DEFAULT_SUBNET_OWNER_STAKE: u64 = 1_000_000_000;

/// Locked TAO seeded into `SubnetLocked` / `LargestLocked` for the bootstrapped
/// subnet. Must stay non-zero so emission flow has a reserve to draw from at
/// the first block.
pub const DEFAULT_SUBNET_LOCKED_TAO: u64 = 1;

/// Inputs to the shared genesis-with-game-solver builder.
///
/// The builder is parameterized over the chain-specific bits (authority URIs,
/// operator URIs, subnet owner URIs, naming, chain type) and the subnet-level
/// economic constants, so a single implementation covers devnet, testnet, and
/// any future chain that wants the full game-solver bootstrap on genesis.
#[derive(Clone)]
pub struct GameSolverGenesisSpec {
    /// Display name written into the chain spec (`Myosu Devnet`, `Myosu Testnet`).
    pub name: &'static str,
    /// Substrate protocol id (`myosu-devnet`, `myosu-testnet`).
    pub protocol_id: &'static str,
    /// Chain id used by `build-spec --chain <id>`.
    pub id: &'static str,
    /// Chain type tag.
    pub chain_type: ChainType,
    /// Authority URIs. Aura + Grandpa keys are derived from each entry.
    pub authority_uris: &'static [&'static str],
    /// Operator URIs that get an operator-level endowment in addition to the
    /// chain-specific subnet owner + hotkey accounts (e.g. miner / validator
    /// service accounts on a multi-node testnet).
    pub operator_uris: &'static [&'static str],
    /// Coldkey URI for the subnet owner.
    pub subnet_owner_uri: &'static str,
    /// Hotkey URI for the subnet owner (must be a distinct sr25519 keypair).
    pub subnet_owner_hotkey_uri: &'static str,
    /// Subnet UID the builder will bootstrap. Pinned to 7 today for parity
    /// with the historic devnet.
    pub subnet_uid: u16,
    /// Per-authority endowment in RAO. Defaults to [`DEFAULT_AUTHORITY_ENDOWMENT`].
    pub authority_endowment: u128,
    /// Per-operator (subnet owner coldkey + operator URIs) endowment in RAO.
    /// Defaults to [`DEFAULT_OPERATOR_ENDOWMENT`].
    pub operator_endowment: u128,
    /// Standalone hotkey endowment in RAO. Defaults to [`DEFAULT_HOTKEY_ENDOWMENT`].
    pub hotkey_endowment: u128,
    /// Subnet pool balance seeded into `SubnetAlphaIn` / `SubnetTAO`. Defaults
    /// to [`DEFAULT_SUBNET_POOL_BALANCE`].
    pub subnet_pool_balance: u64,
    /// Subnet owner stake used to back the genesis alpha shares. Defaults to
    /// [`DEFAULT_SUBNET_OWNER_STAKE`].
    pub subnet_owner_stake: u64,
    /// Locked TAO seeded into `SubnetLocked` / `LargestLocked`. Defaults to
    /// [`DEFAULT_SUBNET_LOCKED_TAO`].
    pub subnet_locked_tao: u64,
}

impl GameSolverGenesisSpec {
    /// Standard defaults: subnet 7 with the historic devnet economic constants.
    /// Callers only need to override the chain-specific fields (name, ids,
    /// URIs, chain type) and the rest stays identical to the devnet baseline.
    pub const fn with_default_economics(
        name: &'static str,
        protocol_id: &'static str,
        id: &'static str,
        chain_type: ChainType,
        authority_uris: &'static [&'static str],
        operator_uris: &'static [&'static str],
        subnet_owner_uri: &'static str,
        subnet_owner_hotkey_uri: &'static str,
        subnet_uid: u16,
    ) -> Self {
        Self {
            name,
            protocol_id,
            id,
            chain_type,
            authority_uris,
            operator_uris,
            subnet_owner_uri,
            subnet_owner_hotkey_uri,
            subnet_uid,
            authority_endowment: DEFAULT_AUTHORITY_ENDOWMENT,
            operator_endowment: DEFAULT_OPERATOR_ENDOWMENT,
            hotkey_endowment: DEFAULT_HOTKEY_ENDOWMENT,
            subnet_pool_balance: DEFAULT_SUBNET_POOL_BALANCE,
            subnet_owner_stake: DEFAULT_SUBNET_OWNER_STAKE,
            subnet_locked_tao: DEFAULT_SUBNET_LOCKED_TAO,
        }
    }

    fn endowed_accounts(&self) -> Vec<(AccountId, u128)> {
        let mut accounts: Vec<(AccountId, u128)> = self
            .authority_uris
            .iter()
            .map(|uri| {
                (
                    get_account_id_from_uri::<sr25519::Public>(uri),
                    self.authority_endowment,
                )
            })
            .collect();
        accounts.extend(self.operator_uris.iter().map(|uri| {
            (
                get_account_id_from_uri::<sr25519::Public>(uri),
                self.operator_endowment,
            )
        }));
        // Always endow the subnet owner coldkey (URIs in `operator_uris` do
        // NOT have to include it; the coldkey gets its own row regardless).
        if !self
            .operator_uris
            .iter()
            .any(|uri| *uri == self.subnet_owner_uri)
        {
            accounts.push((
                get_account_id_from_uri::<sr25519::Public>(self.subnet_owner_uri),
                self.operator_endowment,
            ));
        }
        accounts.push((
            get_account_id_from_uri::<sr25519::Public>(self.subnet_owner_hotkey_uri),
            self.hotkey_endowment,
        ));
        accounts
    }
}

/// Builds a [`ChainSpec`] whose genesis includes only the typed
/// `runtimeGenesis.patch` for `spec` (aura, grandpa, balances, game-solver
/// `balancesIssuance`). The subnet-7 storage bootstrap is intentionally NOT
/// applied — this is the shape `build-spec --chain <id>` writes before any
/// operator entrypoint flips the bootstrap step on. It is also the shape the
/// IMPLEMENTATION_PLAN's "fail-closed if the game-solver patch is absent"
/// gate inspects.
///
/// Most callers should use [`genesis_with_game_solver`] instead; this
/// function is only useful for tests that need to read back the typed
/// `runtimeGenesis.patch` from the spec JSON (since `set_storage` collapses
/// the patch into raw form).
pub fn build_patch_only_spec(spec: &GameSolverGenesisSpec) -> Result<ChainSpec, String> {
    let wasm_binary = crate::chain_spec::WASM_BINARY
        .ok_or_else(|| "development wasm is not available".to_string())?;

    let mut properties = sc_service::Properties::new();
    properties.insert("tokenSymbol".into(), "TAO".into());
    properties.insert("tokenDecimals".into(), 9.into());
    properties.insert("ss58Format".into(), 42.into());

    let initial_authorities = spec
        .authority_uris
        .iter()
        .map(|uri| authority_keys_from_uri(uri))
        .collect::<Vec<_>>();

    let endowed_accounts = spec.endowed_accounts();
    let balances_issuance = endowed_accounts
        .iter()
        .map(|(_, balance)| *balance)
        .sum::<u128>();

    Ok(ChainSpec::builder(wasm_binary, Extensions::default())
        .with_name(spec.name)
        .with_protocol_id(spec.protocol_id)
        .with_id(spec.id)
        .with_chain_type(spec.chain_type.clone())
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

/// Builds a [`ChainSpec`] whose genesis includes the full game-solver subnet
/// bootstrap (subnet 7, registered subnet owner + hotkey, staking pools,
/// validator permits, emission plumbing). This is the shared implementation
/// used by both the devnet and testnet chain specs.
///
/// The chain-specific bits (authority / operator / owner URIs, naming, chain
/// type) come from `spec`. All subnet-level economic constants default to the
/// values historically used by the devnet and may be overridden via
/// [`GameSolverGenesisSpec`] for chains that need different numbers.
pub fn genesis_with_game_solver(spec: &GameSolverGenesisSpec) -> Result<ChainSpec, String> {
    let mut sealed = build_patch_only_spec(spec)?;
    bootstrap_subnet(
        &mut sealed,
        spec.subnet_uid,
        get_account_id_from_uri::<sr25519::Public>(spec.subnet_owner_uri),
        get_account_id_from_uri::<sr25519::Public>(spec.subnet_owner_hotkey_uri),
        spec.subnet_pool_balance,
        spec.subnet_owner_stake,
        spec.subnet_locked_tao,
    )?;
    Ok(sealed)
}

fn bootstrap_subnet(
    spec: &mut ChainSpec,
    subnet_uid: u16,
    owner_coldkey: AccountId,
    owner_hotkey: AccountId,
    subnet_pool_balance: u64,
    subnet_owner_stake: u64,
    subnet_locked_tao: u64,
) -> Result<(), String> {
    let netuid = NetUid::from(subnet_uid);
    let mut ext = BasicExternalities::new(
        spec.build_storage()
            .map_err(|error| format!("failed to build chain storage: {error}"))?,
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

        SubnetAlphaIn::<Runtime>::insert(netuid, AlphaCurrency::from(subnet_pool_balance));
        SubnetTAO::<Runtime>::insert(netuid, TaoCurrency::from(subnet_pool_balance));
        SubnetOwner::<Runtime>::insert(netuid, owner_coldkey.clone());
        SubnetOwnerHotkey::<Runtime>::insert(netuid, owner_hotkey.clone());
        NetworkRegisteredAt::<Runtime>::insert(netuid, registration_block);
        SubnetLocked::<Runtime>::insert(netuid, TaoCurrency::from(subnet_locked_tao));
        LargestLocked::<Runtime>::insert(netuid, subnet_locked_tao);
        MinAllowedWeights::<Runtime>::insert(netuid, 0u16);

        Alpha::<Runtime>::insert(
            (owner_hotkey.clone(), owner_coldkey.clone(), netuid),
            U64F64::saturating_from_num(subnet_owner_stake),
        );
        TotalHotkeyAlpha::<Runtime>::insert(
            owner_hotkey.clone(),
            netuid,
            AlphaCurrency::from(subnet_owner_stake),
        );
        TotalHotkeyShares::<Runtime>::insert(
            owner_hotkey.clone(),
            netuid,
            U64F64::saturating_from_num(subnet_owner_stake),
        );
        SubnetAlphaOut::<Runtime>::insert(netuid, AlphaCurrency::from(subnet_owner_stake));

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
    use sp_io::TestExternalities;

    fn assert_subnet_bootstrapped(
        spec: &ChainSpec,
        subnet_uid: u16,
        expected_owner_coldkey: AccountId,
        expected_owner_hotkey: AccountId,
    ) {
        let mut ext = TestExternalities::from(spec.build_storage().expect("storage"));
        ext.execute_with(|| {
            let netuid = NetUid::from(subnet_uid);
            assert!(
                NetworksAdded::<Runtime>::get(netuid),
                "subnet {subnet_uid} should be present in NetworksAdded"
            );
            assert_eq!(
                SubnetOwner::<Runtime>::get(netuid),
                expected_owner_coldkey,
                "subnet owner coldkey should match the configured URI"
            );
            assert_eq!(
                SubnetOwnerHotkey::<Runtime>::get(netuid),
                expected_owner_hotkey,
                "subnet owner hotkey should match the configured URI"
            );
            assert_eq!(
                SubnetworkN::<Runtime>::get(netuid),
                1,
                "subnet should have exactly one registered member"
            );
            assert_eq!(
                Uids::<Runtime>::get(netuid, expected_owner_hotkey.clone()),
                Some(0)
            );
            assert_eq!(Keys::<Runtime>::get(netuid, 0), expected_owner_hotkey);
            assert!(SubtokenEnabled::<Runtime>::get(netuid));
            assert_eq!(MinAllowedWeights::<Runtime>::get(netuid), 0);
            assert_eq!(FirstEmissionBlockNumber::<Runtime>::get(netuid), Some(0));
            // Pool balances and alpha shares should be the same as the devnet
            // baseline: this is the regression gate that protects the working
            // compose proof.
            assert_eq!(
                SubnetAlphaIn::<Runtime>::get(netuid),
                AlphaCurrency::from(DEFAULT_SUBNET_POOL_BALANCE)
            );
            assert_eq!(
                SubnetTAO::<Runtime>::get(netuid),
                TaoCurrency::from(DEFAULT_SUBNET_POOL_BALANCE)
            );
            assert_eq!(
                SubnetAlphaOut::<Runtime>::get(netuid),
                AlphaCurrency::from(DEFAULT_SUBNET_OWNER_STAKE)
            );
            assert_eq!(
                TotalHotkeyAlpha::<Runtime>::get(expected_owner_hotkey.clone(), netuid),
                AlphaCurrency::from(DEFAULT_SUBNET_OWNER_STAKE)
            );
            assert_eq!(
                TotalHotkeyShares::<Runtime>::get(expected_owner_hotkey.clone(), netuid),
                U64F64::saturating_from_num(DEFAULT_SUBNET_OWNER_STAKE)
            );
            assert_eq!(
                SubnetLocked::<Runtime>::get(netuid),
                TaoCurrency::from(DEFAULT_SUBNET_LOCKED_TAO)
            );
            assert_eq!(
                LargestLocked::<Runtime>::get(netuid),
                DEFAULT_SUBNET_LOCKED_TAO
            );
            assert!(
                ValidatorPermit::<Runtime>::get(netuid)
                    .into_iter()
                    .all(|permit| !permit)
            );
        });
    }

    #[test]
    fn shared_builder_bootstraps_subnet_with_overridden_owner_accounts() {
        // Synthetic chain spec that does NOT use the devnet URIs to prove the
        // shared builder is parameterized over the owner accounts, not
        // hard-coded to devnet's keys.
        const SYNTHETIC_AUTHORITIES: &[&str] = &[
            "//myosu//shared//authority-a",
            "//myosu//shared//authority-b",
        ];
        const SYNTHETIC_OPERATORS: &[&str] = &["//myosu//shared//miner-1"];
        const SYNTHETIC_OWNER: &str = "//myosu//shared//subnet-owner";
        const SYNTHETIC_OWNER_HOTKEY: &str = "//myosu//shared//subnet-owner//hotkey";
        let spec = GameSolverGenesisSpec::with_default_economics(
            "Myosu Shared",
            "myosu-shared",
            "myosu-shared",
            ChainType::Custom("shared-test".into()),
            SYNTHETIC_AUTHORITIES,
            SYNTHETIC_OPERATORS,
            SYNTHETIC_OWNER,
            SYNTHETIC_OWNER_HOTKEY,
            7,
        );

        let built = genesis_with_game_solver(&spec).expect("shared builder should succeed");
        assert_eq!(
            sc_service::ChainSpec::chain_type(&built),
            ChainType::Custom("shared-test".into())
        );
        assert_eq!(sc_service::ChainSpec::name(&built), "Myosu Shared");
        assert_eq!(sc_service::ChainSpec::id(&built), "myosu-shared");
        assert_eq!(
            sc_service::ChainSpec::protocol_id(&built),
            Some("myosu-shared")
        );

        let expected_owner_coldkey = get_account_id_from_uri::<sr25519::Public>(SYNTHETIC_OWNER);
        let expected_owner_hotkey =
            get_account_id_from_uri::<sr25519::Public>(SYNTHETIC_OWNER_HOTKEY);
        assert_subnet_bootstrapped(&built, 7, expected_owner_coldkey, expected_owner_hotkey);
    }
}
