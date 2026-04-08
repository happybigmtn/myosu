#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use alloc::collections::BTreeMap;

use super::mock::*;

use crate::epoch::math::fixed64_to_u64;
use crate::*;
use codec::{Compact, Decode, Encode};
use frame_support::{assert_noop, assert_ok};
use safe_math::FixedExt;
use scale_info::{Registry, TypeDef, meta_type};
use sp_core::U256;
use substrate_fixed::types::{I32F32, I96F32, U64F64, U96F32};
use subtensor_runtime_common::{AlphaCurrency, MechId, NetUid, NetUidStorageIndex, TaoCurrency};
use subtensor_swap_interface::SwapHandler;

#[derive(Debug, Decode, Encode, PartialEq, Eq)]
struct DecodedStakeInfo {
    hotkey: U256,
    coldkey: U256,
    netuid: Compact<NetUid>,
    stake: Compact<AlphaCurrency>,
    emission: Compact<AlphaCurrency>,
    is_registered: bool,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq)]
struct DecodedDynamicInfo {
    netuid: Compact<NetUid>,
    owner_hotkey: U256,
    owner_coldkey: U256,
    subnet_name: Vec<Compact<u8>>,
    token_symbol: Vec<Compact<u8>>,
    tempo: Compact<u16>,
    last_step: Compact<u64>,
    blocks_since_last_step: Compact<u64>,
    alpha_in: Compact<AlphaCurrency>,
    alpha_out: Compact<AlphaCurrency>,
    tao_in: Compact<TaoCurrency>,
    alpha_out_emission: Compact<AlphaCurrency>,
    alpha_in_emission: Compact<AlphaCurrency>,
    tao_in_emission: Compact<TaoCurrency>,
    pending_alpha_emission: Compact<AlphaCurrency>,
    subnet_volume: Compact<u128>,
    network_registered_at: Compact<u64>,
    subnet_identity: Option<SubnetIdentityV3>,
    moving_price: I96F32,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq)]
struct DecodedSubnetInfoV2 {
    netuid: Compact<NetUid>,
    rho: Compact<u16>,
    kappa: Compact<u16>,
    difficulty: Compact<u64>,
    immunity_period: Compact<u16>,
    max_allowed_validators: Compact<u16>,
    min_allowed_weights: Compact<u16>,
    max_weights_limit: Compact<u16>,
    scaling_law_power: Compact<u16>,
    subnetwork_n: Compact<u16>,
    max_allowed_uids: Compact<u16>,
    blocks_since_last_step: Compact<u64>,
    tempo: Compact<u16>,
    burn: Compact<TaoCurrency>,
    owner: U256,
    identity: Option<SubnetIdentityV3>,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq)]
struct DecodedSubnetHyperparamsV2 {
    rho: Compact<u16>,
    kappa: Compact<u16>,
    immunity_period: Compact<u16>,
    min_allowed_weights: Compact<u16>,
    max_weights_limit: Compact<u16>,
    tempo: Compact<u16>,
    min_difficulty: Compact<u64>,
    max_difficulty: Compact<u64>,
    weights_version: Compact<u64>,
    weights_rate_limit: Compact<u64>,
    adjustment_interval: Compact<u16>,
    activity_cutoff: Compact<u16>,
    registration_allowed: bool,
    target_regs_per_interval: Compact<u16>,
    min_burn: Compact<TaoCurrency>,
    max_burn: Compact<TaoCurrency>,
    bonds_moving_avg: Compact<u64>,
    max_regs_per_block: Compact<u16>,
    serving_rate_limit: Compact<u64>,
    max_validators: Compact<u16>,
    adjustment_alpha: Compact<u64>,
    difficulty: Compact<u64>,
    commit_reveal_period: Compact<u64>,
    commit_reveal_weights_enabled: bool,
    alpha_high: Compact<u16>,
    alpha_low: Compact<u16>,
    liquid_alpha_enabled: bool,
    alpha_sigmoid_steepness: I32F32,
    yuma_version: Compact<u16>,
    subnet_is_active: bool,
    transfers_enabled: bool,
    bonds_reset_enabled: bool,
}

#[derive(Debug, Decode, Encode, PartialEq, Eq)]
struct DecodedSubnetState {
    netuid: Compact<NetUid>,
    hotkeys: Vec<U256>,
    coldkeys: Vec<U256>,
    active: Vec<bool>,
    validator_permit: Vec<bool>,
    pruning_score: Vec<Compact<u16>>,
    last_update: Vec<Compact<u64>>,
    emission: Vec<Compact<AlphaCurrency>>,
    dividends: Vec<Compact<u16>>,
    incentives: Vec<Compact<u16>>,
    consensus: Vec<Compact<u16>>,
    trust: Vec<Compact<u16>>,
    rank: Vec<Compact<u16>>,
    block_at_registration: Vec<Compact<u64>>,
    alpha_stake: Vec<Compact<AlphaCurrency>>,
    tao_stake: Vec<Compact<TaoCurrency>>,
    total_stake: Vec<Compact<TaoCurrency>>,
}

#[derive(Debug, Decode, Encode)]
struct DecodedNeuronInfoLite {
    hotkey: U256,
    coldkey: U256,
    uid: Compact<u16>,
    netuid: Compact<NetUid>,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(U256, Compact<AlphaCurrency>)>,
    rank: Compact<u16>,
    emission: Compact<AlphaCurrency>,
    incentive: Compact<u16>,
    consensus: Compact<u16>,
    trust: Compact<u16>,
    validator_trust: Compact<u16>,
    dividends: Compact<u16>,
    last_update: Compact<u64>,
    validator_permit: bool,
    pruning_score: Compact<u16>,
}

fn decode_stake_info(info: &impl Encode) -> DecodedStakeInfo {
    let bytes = info.encode();

    DecodedStakeInfo::decode(&mut &bytes[..]).expect("stake info should decode")
}

fn decode_dynamic_info(info: &impl Encode) -> DecodedDynamicInfo {
    let bytes = info.encode();
    let mut input = &bytes[..];
    let decoded = DecodedDynamicInfo::decode(&mut input).expect("dynamic info should decode");

    assert!(
        input.is_empty(),
        "dynamic info should not carry trailing fields"
    );

    decoded
}

fn decode_subnet_info_v2(info: &impl Encode) -> DecodedSubnetInfoV2 {
    let bytes = info.encode();
    let mut input = &bytes[..];
    let decoded = DecodedSubnetInfoV2::decode(&mut input).expect("subnet info v2 should decode");

    assert!(
        input.is_empty(),
        "subnet info v2 should not carry trailing fields"
    );

    decoded
}

fn decode_subnet_hyperparams_v2(info: &impl Encode) -> DecodedSubnetHyperparamsV2 {
    let bytes = info.encode();
    let mut input = &bytes[..];
    let decoded = DecodedSubnetHyperparamsV2::decode(&mut input)
        .expect("subnet hyperparams v2 should decode");

    assert!(
        input.is_empty(),
        "subnet hyperparams v2 should not carry trailing fields"
    );

    decoded
}

fn decode_subnet_state(info: &impl Encode) -> DecodedSubnetState {
    let bytes = info.encode();
    let mut input = &bytes[..];
    let decoded = DecodedSubnetState::decode(&mut input).expect("subnet state should decode");

    assert!(
        input.is_empty(),
        "subnet state should not carry trailing fields"
    );

    decoded
}

fn decode_neuron_info_lite(info: &impl Encode) -> DecodedNeuronInfoLite {
    let bytes = info.encode();
    let mut input = &bytes[..];
    let decoded =
        DecodedNeuronInfoLite::decode(&mut input).expect("neuron info lite should decode");

    assert!(
        input.is_empty(),
        "neuron info lite should not carry trailing fields"
    );

    decoded
}

fn decode_neuron_info_lite_by_uid(
    infos: &[crate::rpc_info::neuron_info::NeuronInfoLite<U256>],
    uid: u16,
) -> DecodedNeuronInfoLite {
    let info = infos
        .iter()
        .find(|info| {
            let decoded = decode_neuron_info_lite(*info);
            decoded.uid.0 == uid
        })
        .expect("neuron info lite should exist");

    decode_neuron_info_lite(info)
}

fn decode_subnet_infos_v2(info: &impl Encode) -> Vec<DecodedSubnetInfoV2> {
    let bytes = info.encode();
    let mut input = &bytes[..];
    let decoded =
        Vec::<DecodedSubnetInfoV2>::decode(&mut input).expect("subnet info v2 list should decode");

    assert!(
        input.is_empty(),
        "subnet info v2 list should not carry trailing fields"
    );

    decoded
}

fn decode_dynamic_infos(info: &impl Encode) -> Vec<DecodedDynamicInfo> {
    let bytes = info.encode();
    let mut input = &bytes[..];
    let decoded =
        Vec::<DecodedDynamicInfo>::decode(&mut input).expect("dynamic info list should decode");

    assert!(
        input.is_empty(),
        "dynamic info list should not carry trailing fields"
    );

    decoded
}

fn stage_0_pending_distribution_loss(block_emission: u64, blocks: u64) -> u64 {
    let block_emission = U96F32::from_num(block_emission);
    let owner_cut = U96F32::from_num(0.1);
    let miner_split = U96F32::from_num(0.5);
    let mut distributed = U96F32::from_num(0);

    for _ in 0..blocks {
        let owner_cut_amount = block_emission.saturating_mul(owner_cut);
        let remaining = block_emission.saturating_sub(owner_cut_amount);
        let server_amount = remaining.saturating_mul(miner_split);
        let validator_amount = remaining.saturating_sub(server_amount);

        distributed = distributed
            .saturating_add(U96F32::from_num(
                owner_cut_amount.saturating_to_num::<u64>(),
            ))
            .saturating_add(U96F32::from_num(server_amount.saturating_to_num::<u64>()))
            .saturating_add(U96F32::from_num(
                validator_amount.saturating_to_num::<u64>(),
            ));
    }

    block_emission
        .saturating_mul(U96F32::from_num(blocks))
        .saturating_sub(distributed)
        .saturating_to_num::<u64>()
}

#[test]
fn stage_0_flow_dispatch_surface_matches_live_chain_loop() {
    let mut registry = Registry::new();
    let type_id = registry.register_type(&meta_type::<crate::Call<Test>>());
    let registry: scale_info::PortableRegistry = registry.into();
    let type_info = registry
        .resolve(type_id.id)
        .expect("pallet call type should resolve");

    let TypeDef::Variant(variants) = &type_info.type_def else {
        panic!("pallet call type should be an enum");
    };

    let mut actual = variants
        .variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect::<Vec<_>>();
    actual.sort_unstable();

    let mut expected = vec![
        "add_stake",
        "burned_register",
        "commit_weights",
        "register_network",
        "reveal_weights",
        "serve_axon",
        "set_weights",
        "start_call",
    ];
    expected.sort_unstable();

    assert_eq!(actual, expected);
    assert!(
        variants.variants.len() <= 20,
        "stage-0 default call surface exceeded budget: {}",
        variants.variants.len()
    );
}

#[test]
fn stage_0_flow_registers_stakes_serves_and_emits() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(10);
        let owner_coldkey = U256::from(11);
        let validator_hotkey = U256::from(20);
        let validator_coldkey = U256::from(21);
        let miner_hotkey = U256::from(30);
        let miner_coldkey = U256::from(31);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let stake_amount: u64 = 100_000_000_000_000;
        let reserve_amount: u64 = stake_amount * 1_000;
        let miner_ip: u128 = 1_676_056_785;
        let miner_port: u16 = 128;

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tempo(netuid, 2);
        GameSolver::set_tao_weight(0);
        GameSolver::set_weights_set_rate_limit(netuid, 0);
        GameSolver::set_max_allowed_uids(netuid, 3);
        GameSolver::set_max_allowed_validators(netuid, 1);

        GameSolver::add_balance_to_coldkey_account(
            &validator_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );
        GameSolver::add_balance_to_coldkey_account(
            &miner_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 0);

        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(miner_coldkey),
            miner_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));

        assert_ok!(GameSolver::serve_axon(
            RuntimeOrigin::signed(miner_hotkey),
            netuid,
            1,
            miner_ip,
            miner_port,
            4,
            0,
            0,
            0,
        ));

        let validator_uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &validator_hotkey)
            .expect("validator uid should exist");
        let miner_uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &miner_hotkey)
            .expect("miner uid should exist");

        run_to_block_no_epoch(netuid, 30);
        GameSolver::epoch(netuid, AlphaCurrency::ZERO);

        assert!(GameSolver::get_validator_permit_for_uid(
            netuid,
            validator_uid
        ));
        assert!(!GameSolver::get_validator_permit_for_uid(netuid, miner_uid));

        next_block_no_epoch(netuid);
        assert_ok!(GameSolver::set_weights(
            RuntimeOrigin::signed(validator_hotkey),
            netuid,
            vec![miner_uid],
            vec![u16::MAX],
            0,
        ));

        let axon = GameSolver::get_axon_info(netuid, &miner_hotkey);
        assert_eq!(axon.ip, miner_ip);
        assert_eq!(axon.port, miner_port);

        Incentive::<Test>::remove(NetUidStorageIndex::from(netuid));
        Dividends::<Test>::remove(netuid);

        let blocks_to_next_epoch = GameSolver::blocks_until_next_epoch(
            netuid,
            GameSolver::get_tempo(netuid),
            GameSolver::get_current_block_as_u64(),
        );
        step_block(blocks_to_next_epoch as u16);
        assert!(GameSolver::should_run_epoch(netuid, System::block_number()));

        GameSolver::run_coinbase(U96F32::from_num(100_000_000_u64));

        assert_eq!(BlocksSinceLastStep::<Test>::get(netuid), 0);
        assert!(
            Incentive::<Test>::get(NetUidStorageIndex::from(netuid))
                .iter()
                .sum::<u16>()
                > 0
        );
        assert!(Dividends::<Test>::get(netuid).iter().sum::<u16>() > 0);
        assert!(
            Incentive::<Test>::get(NetUidStorageIndex::from(netuid))
                .get(miner_uid as usize)
                .copied()
                .unwrap_or_default()
                > 0
        );
        assert!(
            Dividends::<Test>::get(netuid)
                .get(validator_uid as usize)
                .copied()
                .unwrap_or_default()
                > 0
        );
    });
}

#[test]
fn stage_0_coinbase_root_proportion_and_pending_root_divs_stay_zero() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(35);
        let owner_coldkey = U256::from(36);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);

        setup_reserves(
            netuid,
            TaoCurrency::from(1_000_000_000_000u64),
            AlphaCurrency::from(1_000_000_000_000u64),
        );
        GameSolver::set_tao_weight(u64::MAX);
        SubnetTAO::<Test>::set(NetUid::ROOT, TaoCurrency::from(9_000_000_000_000u64));

        let subnet_emissions = BTreeMap::from([(netuid, U96F32::from_num(1_000_000_000u64))]);

        assert_eq!(GameSolver::root_proportion(netuid), U96F32::from_num(0));

        GameSolver::emit_to_subnets(&[netuid], &subnet_emissions, true);

        assert_eq!(
            PendingRootAlphaDivs::<Test>::get(netuid),
            AlphaCurrency::ZERO
        );
        assert!(PendingServerEmission::<Test>::get(netuid) > AlphaCurrency::ZERO);
        assert!(PendingValidatorEmission::<Test>::get(netuid) > AlphaCurrency::ZERO);
    });
}

#[test]
fn stage_0_coinbase_dividend_distribution_folds_root_bucket_into_alpha_dividends() {
    new_test_ext(1).execute_with(|| {
        let first_hotkey = U256::from(45);
        let second_hotkey = U256::from(46);
        let pending_alpha = AlphaCurrency::from(600u64);
        let pending_root_alpha = AlphaCurrency::from(400u64);

        let (alpha_dividends, root_alpha_dividends) = GameSolver::calculate_dividend_distribution(
            pending_alpha,
            pending_root_alpha,
            U96F32::from_num(1.0),
            BTreeMap::from([
                (
                    first_hotkey,
                    (AlphaCurrency::from(10u64), AlphaCurrency::from(10_000u64)),
                ),
                (
                    second_hotkey,
                    (AlphaCurrency::from(10u64), AlphaCurrency::from(1u64)),
                ),
            ]),
            BTreeMap::from([
                (first_hotkey, U96F32::from_num(3)),
                (second_hotkey, U96F32::from_num(1)),
            ]),
        );

        assert!(root_alpha_dividends.is_empty());
        assert_eq!(
            alpha_dividends
                .get(&first_hotkey)
                .copied()
                .expect("first hotkey should receive dividends")
                .saturating_to_num::<u64>(),
            750
        );
        assert_eq!(
            alpha_dividends
                .get(&second_hotkey)
                .copied()
                .expect("second hotkey should receive dividends")
                .saturating_to_num::<u64>(),
            250
        );
        assert_eq!(
            alpha_dividends
                .values()
                .copied()
                .sum::<U96F32>()
                .saturating_to_num::<u64>(),
            u64::from(pending_alpha.saturating_add(pending_root_alpha))
        );
    });
}

#[test]
fn stage_0_coinbase_zero_dividend_distribution_falls_back_to_weighted_stake() {
    new_test_ext(1).execute_with(|| {
        let first_hotkey = U256::from(47);
        let second_hotkey = U256::from(48);
        let pending_alpha = AlphaCurrency::from(600u64);
        let pending_root_alpha = AlphaCurrency::from(400u64);

        let (alpha_dividends, root_alpha_dividends) = GameSolver::calculate_dividend_distribution(
            pending_alpha,
            pending_root_alpha,
            U96F32::from_num(0.0),
            BTreeMap::from([
                (
                    first_hotkey,
                    (AlphaCurrency::from(300u64), AlphaCurrency::ZERO),
                ),
                (
                    second_hotkey,
                    (AlphaCurrency::from(100u64), AlphaCurrency::ZERO),
                ),
            ]),
            BTreeMap::new(),
        );

        assert!(root_alpha_dividends.is_empty());
        assert_eq!(
            alpha_dividends
                .get(&first_hotkey)
                .copied()
                .expect("first hotkey should receive fallback dividends")
                .saturating_to_num::<u64>(),
            750
        );
        assert_eq!(
            alpha_dividends
                .get(&second_hotkey)
                .copied()
                .expect("second hotkey should receive fallback dividends")
                .saturating_to_num::<u64>(),
            250
        );
        assert_eq!(
            alpha_dividends
                .values()
                .copied()
                .sum::<U96F32>()
                .saturating_to_num::<u64>(),
            u64::from(pending_alpha.saturating_add(pending_root_alpha))
        );
    });
}

#[test]
fn stage_0_coinbase_truncation_drift_stays_below_two_rao_per_block_sweep() {
    let block_counts = [1_u64, 3, 100, 1_000, 10_000];

    for block_count in block_counts {
        let mut observed_max_drift = 0_u64;

        for block_emission in (1_u64..=128).chain([1_000_003_u64, 100_000_001].into_iter()) {
            let drift = stage_0_pending_distribution_loss(block_emission, block_count);
            observed_max_drift = observed_max_drift.max(drift);

            assert!(
                drift <= block_count.saturating_mul(2),
                "truncation drift exceeded sweep budget: emission={} blocks={} drift={}",
                block_emission,
                block_count,
                drift
            );
        }

        assert_eq!(
            observed_max_drift,
            block_count.saturating_mul(2),
            "sweep should keep measuring the live worst-case drift envelope"
        );
    }
}

#[test]
fn stage_0_try_state_delta_stays_well_above_default_epoch_drift() {
    let default_epoch_blocks = 3_u64;
    let measured_worst_case = (1_u64..=128)
        .chain([1_000_003_u64, 100_000_001])
        .map(|block_emission| {
            stage_0_pending_distribution_loss(block_emission, default_epoch_blocks)
        })
        .max()
        .expect("sweep should produce a measured drift");

    assert_eq!(
        measured_worst_case, 6,
        "default tempo-2 epochs should keep measuring the live 6-rao dust envelope"
    );
    assert!(
        crate::TOTAL_ISSUANCE_TRY_STATE_ALERT_DELTA >= measured_worst_case.saturating_mul(5),
        "try-state alert threshold should stay comfortably above the measured default-epoch drift"
    );
}

#[test]
fn legacy_epoch_skip_emits_event_when_state_is_inconsistent() {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 155.into();
        let hotkey = U256::from(42u64);

        Keys::<Test>::insert(netuid, 0u16, hotkey);
        Keys::<Test>::insert(netuid, 1u16, hotkey);
        System::reset_events();

        let output = GameSolver::epoch(netuid, 1_000.into());

        assert!(output.is_empty());
        assert_last_event::<Test>(
            Event::EpochSkipped {
                netuid,
                reason: EpochSkipReason::InconsistentInputState,
            }
            .into(),
        );
    });
}

#[test]
fn stage_0_coinbase_emission_accounting_matches_accrued_epoch_budget() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(110);
        let owner_coldkey = U256::from(111);
        let validator_hotkey = U256::from(120);
        let validator_coldkey = U256::from(121);
        let miner_hotkey = U256::from(130);
        let miner_coldkey = U256::from(131);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let stake_amount: u64 = 100_000_000_000;
        let reserve_amount: u64 = stake_amount * 1_000;
        let tempo = 2_u16;

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tempo(netuid, tempo);
        GameSolver::set_tao_weight(0);
        GameSolver::set_subnet_owner_cut(u16::MAX / 10);
        GameSolver::set_weights_set_rate_limit(netuid, 0);
        GameSolver::set_max_allowed_uids(netuid, 3);
        GameSolver::set_max_allowed_validators(netuid, 1);

        GameSolver::add_balance_to_coldkey_account(
            &validator_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );
        GameSolver::add_balance_to_coldkey_account(
            &miner_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 0);

        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(miner_coldkey),
            miner_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));
        assert_ok!(GameSolver::serve_axon(
            RuntimeOrigin::signed(miner_hotkey),
            netuid,
            1,
            1_676_056_785,
            128,
            4,
            0,
            0,
            0,
        ));

        let validator_uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &validator_hotkey)
            .expect("validator uid should exist");
        let miner_uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &miner_hotkey)
            .expect("miner uid should exist");

        run_to_block_no_epoch(netuid, 30);
        GameSolver::epoch(netuid, AlphaCurrency::ZERO);
        next_block_no_epoch(netuid);
        assert_ok!(GameSolver::set_weights(
            RuntimeOrigin::signed(validator_hotkey),
            netuid,
            vec![miner_uid],
            vec![u16::MAX],
            0,
        ));

        let block_emission = GameSolver::get_block_emission_for_issuance(
            GameSolver::get_alpha_issuance(netuid).into(),
        )
        .expect("block emission should derive from subnet issuance");
        assert!(block_emission > 0);

        let accrual_blocks = u64::from(tempo) + 1;
        let mut cycle_start_block = GameSolver::get_current_block_as_u64();
        while GameSolver::blocks_until_next_epoch(netuid, tempo, cycle_start_block)
            != u64::from(tempo)
        {
            cycle_start_block = cycle_start_block.saturating_add(1);
        }
        System::set_block_number(cycle_start_block);
        BlocksSinceLastStep::<Test>::insert(netuid, 0);
        PendingServerEmission::<Test>::insert(netuid, AlphaCurrency::ZERO);
        PendingValidatorEmission::<Test>::insert(netuid, AlphaCurrency::ZERO);
        PendingRootAlphaDivs::<Test>::insert(netuid, AlphaCurrency::ZERO);
        PendingOwnerCut::<Test>::insert(netuid, AlphaCurrency::ZERO);

        let mut final_summary = None;
        for offset in 0..accrual_blocks {
            System::set_block_number(cycle_start_block.saturating_add(offset));
            let summary = GameSolver::run_coinbase(U96F32::from_num(block_emission));
            if summary.drained_epoch_count > 0 {
                final_summary = Some(summary);
            }
        }

        let summary = final_summary.expect("coinbase should drain one epoch");
        let emission_sum = Emission::<Test>::get(netuid)
            .into_iter()
            .map(u64::from)
            .sum::<u64>();
        let expected_emission_sum = u64::from(summary.server_alpha_distributed)
            .saturating_add(u64::from(summary.validator_alpha_distributed));
        let actual_total_distribution =
            expected_emission_sum.saturating_add(u64::from(summary.owner_cut_distributed));
        let expected_epoch_distribution = block_emission.saturating_mul(accrual_blocks);
        let rounding_tolerance = accrual_blocks.saturating_mul(2);

        assert_eq!(summary.drained_epoch_count, 1);
        assert_eq!(u64::from(summary.root_alpha_distributed), 0);
        assert_eq!(emission_sum, expected_emission_sum);
        assert!(
            actual_total_distribution <= expected_epoch_distribution,
            "distribution exceeded accrued budget: actual={} expected={}",
            actual_total_distribution,
            expected_epoch_distribution
        );
        assert!(
            expected_epoch_distribution.saturating_sub(actual_total_distribution)
                <= rounding_tolerance,
            "distribution drift exceeded tolerance: actual={} expected={} tolerance={}",
            actual_total_distribution,
            expected_epoch_distribution,
            rounding_tolerance
        );
        assert!(
            Incentive::<Test>::get(NetUidStorageIndex::from(netuid))
                .get(miner_uid as usize)
                .copied()
                .unwrap_or_default()
                > 0
        );
        assert!(
            Dividends::<Test>::get(netuid)
                .get(validator_uid as usize)
                .copied()
                .unwrap_or_default()
                > 0
        );
    });
}

#[test]
fn stage_0_commit_reveal_v2_is_the_only_live_weight_hiding_path() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(40);
        let owner_coldkey = U256::from(41);
        let validator_hotkey = U256::from(50);
        let validator_coldkey = U256::from(51);
        let miner_hotkey = U256::from(60);
        let miner_coldkey = U256::from(61);
        let netuid = add_dynamic_network(&owner_hotkey, &owner_coldkey);
        let stake_amount: u64 = 100_000_000_000_000;
        let reserve_amount: u64 = stake_amount * 1_000;
        let version_key: u64 = 0;
        let salt = vec![7, 11, 13, 17];

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tempo(netuid, 2);
        GameSolver::set_reveal_period(netuid, 1).expect("reveal period should be valid");
        GameSolver::set_tao_weight(0);
        GameSolver::set_weights_set_rate_limit(netuid, 0);
        GameSolver::set_max_allowed_uids(netuid, 3);
        GameSolver::set_max_allowed_validators(netuid, 1);
        GameSolver::set_commit_reveal_weights_enabled(netuid, true);

        GameSolver::add_balance_to_coldkey_account(
            &validator_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );
        GameSolver::add_balance_to_coldkey_account(
            &miner_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        register_ok_neuron(netuid, miner_hotkey, miner_coldkey, 0);

        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(miner_coldkey),
            miner_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));

        run_to_block_no_epoch(netuid, 30);
        GameSolver::epoch(netuid, AlphaCurrency::ZERO);

        let validator_uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &validator_hotkey)
            .expect("validator uid should exist");
        let miner_uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &miner_hotkey)
            .expect("miner uid should exist");

        assert!(GameSolver::get_validator_permit_for_uid(
            netuid,
            validator_uid
        ));

        assert_noop!(
            GameSolver::set_weights(
                RuntimeOrigin::signed(validator_hotkey),
                netuid,
                vec![miner_uid],
                vec![u16::MAX],
                version_key,
            ),
            Error::<Test>::CommitRevealEnabled
        );

        let commit_hash = GameSolver::get_commit_hash(
            &validator_hotkey,
            NetUidStorageIndex::from(netuid),
            &[miner_uid],
            &[u16::MAX],
            &salt,
            version_key,
        );

        assert_ok!(GameSolver::commit_weights(
            RuntimeOrigin::signed(validator_hotkey),
            netuid,
            commit_hash,
        ));

        assert_noop!(
            GameSolver::reveal_weights(
                RuntimeOrigin::signed(validator_hotkey),
                netuid,
                vec![miner_uid],
                vec![u16::MAX],
                salt.clone(),
                version_key,
            ),
            Error::<Test>::RevealTooEarly
        );

        let (first_reveal_block, _) =
            GameSolver::get_reveal_blocks(netuid, GameSolver::get_current_block_as_u64());
        run_to_block_no_epoch(netuid, first_reveal_block);

        assert_ok!(GameSolver::reveal_weights(
            RuntimeOrigin::signed(validator_hotkey),
            netuid,
            vec![miner_uid],
            vec![u16::MAX],
            salt,
            version_key,
        ));

        assert_eq!(
            WeightCommits::<Test>::get(NetUidStorageIndex::from(netuid), validator_hotkey),
            None
        );
        assert_eq!(
            Weights::<Test>::get(NetUidStorageIndex::from(netuid), validator_uid),
            vec![(miner_uid, u16::MAX)]
        );
    });
}

#[test]
fn stage_0_stake_summaries_follow_noop_swap_pricing() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(110);
        let owner_coldkey = U256::from(111);
        let staker_hotkey = U256::from(120);
        let staker_coldkey = U256::from(121);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let stake_amount: u64 = 100_000_000_000_000;
        let reserve_amount: u64 = stake_amount * 1_000;

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tao_weight(0);
        GameSolver::add_balance_to_coldkey_account(
            &staker_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, staker_hotkey, staker_coldkey, 0);
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(staker_coldkey),
            staker_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));

        let alpha = GameSolver::get_stake_for_hotkey_and_coldkey_on_subnet(
            &staker_hotkey,
            &staker_coldkey,
            netuid,
        );
        let expected: TaoCurrency = U96F32::saturating_from_num(alpha)
            .saturating_mul(<Test as Config>::SwapInterface::current_alpha_price(netuid))
            .saturating_to_num::<u64>()
            .into();

        assert_eq!(
            GameSolver::get_total_stake_for_hotkey(&staker_hotkey),
            expected
        );
        assert_eq!(
            GameSolver::get_total_stake_for_coldkey(&staker_coldkey),
            expected
        );
        assert_eq!(
            GameSolver::get_total_stake_for_coldkey_on_subnet(&staker_coldkey, netuid),
            expected
        );
    });
}

#[test]
fn stage_0_neuron_lite_stake_map_matches_live_coldkey_entries() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_hotkey = U256::from(210);
        let subnet_owner_coldkey = U256::from(211);
        let neuron_hotkey = U256::from(220);
        let neuron_owner_coldkey = U256::from(221);
        let delegator_coldkey = U256::from(222);
        let netuid =
            add_dynamic_network_disable_commit_reveal(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let neuron_owner_stake_amount: u64 = 100_000_000_000_000;
        let delegator_stake_amount: u64 = 50_000_000_000_000;
        let reserve_amount: u64 = (neuron_owner_stake_amount + delegator_stake_amount) * 1_000;

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tao_weight(0);
        GameSolver::add_balance_to_coldkey_account(
            &neuron_owner_coldkey,
            neuron_owner_stake_amount + ExistentialDeposit::get(),
        );
        GameSolver::add_balance_to_coldkey_account(
            &delegator_coldkey,
            delegator_stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, neuron_hotkey, neuron_owner_coldkey, 0);

        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(neuron_owner_coldkey),
            neuron_hotkey,
            netuid,
            TaoCurrency::from(neuron_owner_stake_amount),
        ));
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(delegator_coldkey),
            neuron_hotkey,
            netuid,
            TaoCurrency::from(delegator_stake_amount),
        ));

        let uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &neuron_hotkey)
            .expect("owner neuron uid should exist");
        let neurons = GameSolver::get_neurons_lite(netuid);
        let decoded = decode_neuron_info_lite_by_uid(&neurons, uid);
        let owner_alpha = GameSolver::get_stake_for_hotkey_and_coldkey_on_subnet(
            &neuron_hotkey,
            &neuron_owner_coldkey,
            netuid,
        );
        let delegator_alpha = GameSolver::get_stake_for_hotkey_and_coldkey_on_subnet(
            &neuron_hotkey,
            &delegator_coldkey,
            netuid,
        );

        assert_eq!(decoded.hotkey, neuron_hotkey);
        assert_eq!(decoded.coldkey, neuron_owner_coldkey);
        assert_eq!(decoded.uid.0, uid);
        assert_eq!(decoded.netuid.0, netuid);
        let mut actual_stake = decoded.stake;
        actual_stake.sort_by_key(|(coldkey, _)| *coldkey);
        let mut expected_stake = vec![
            (neuron_owner_coldkey, owner_alpha.into()),
            (delegator_coldkey, delegator_alpha.into()),
        ];
        expected_stake.sort_by_key(|(coldkey, _)| *coldkey);

        assert_eq!(actual_stake, expected_stake);
    });
}

#[test]
fn stage_0_delegate_info_keeps_sparse_live_netuids() {
    new_test_ext(1).execute_with(|| {
        let first_subnet_owner_hotkey = U256::from(310);
        let first_subnet_owner_coldkey = U256::from(311);
        let second_subnet_owner_hotkey = U256::from(320);
        let second_subnet_owner_coldkey = U256::from(321);
        let delegate_hotkey = U256::from(330);
        let delegate_owner_coldkey = U256::from(331);
        let nominator_coldkey = U256::from(332);
        let first_netuid = add_dynamic_network_disable_commit_reveal(
            &first_subnet_owner_hotkey,
            &first_subnet_owner_coldkey,
        );
        let second_netuid = add_dynamic_network_disable_commit_reveal(
            &second_subnet_owner_hotkey,
            &second_subnet_owner_coldkey,
        );
        let delegate_owner_stake_amount: u64 = 100_000_000_000_000;
        let nominator_stake_amount: u64 = 50_000_000_000_000;
        let reserve_amount: u64 = (delegate_owner_stake_amount + nominator_stake_amount) * 1_000;

        setup_reserves(second_netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tao_weight(0);
        GameSolver::add_balance_to_coldkey_account(
            &delegate_owner_coldkey,
            delegate_owner_stake_amount + ExistentialDeposit::get(),
        );
        GameSolver::add_balance_to_coldkey_account(
            &nominator_coldkey,
            nominator_stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(second_netuid, delegate_hotkey, delegate_owner_coldkey, 0);
        GameSolver::maybe_become_delegate(&delegate_hotkey);

        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(delegate_owner_coldkey),
            delegate_hotkey,
            second_netuid,
            TaoCurrency::from(delegate_owner_stake_amount),
        ));
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(nominator_coldkey),
            delegate_hotkey,
            second_netuid,
            TaoCurrency::from(nominator_stake_amount),
        ));

        NetworksAdded::<Test>::insert(first_netuid, false);

        let delegate_info =
            GameSolver::get_delegate(delegate_hotkey).expect("delegate info should exist");
        let owner_alpha = GameSolver::get_stake_for_hotkey_and_coldkey_on_subnet(
            &delegate_hotkey,
            &delegate_owner_coldkey,
            second_netuid,
        );
        let nominator_alpha = GameSolver::get_stake_for_hotkey_and_coldkey_on_subnet(
            &delegate_hotkey,
            &nominator_coldkey,
            second_netuid,
        );

        let mut actual_nominators = delegate_info.nominators;
        actual_nominators.sort_by_key(|(coldkey, _)| *coldkey);
        for (_, stakes) in &mut actual_nominators {
            stakes.sort_by_key(|(netuid, _)| netuid.0);
        }

        let mut expected_nominators = vec![
            (
                delegate_owner_coldkey,
                vec![(second_netuid.into(), u64::from(owner_alpha).into())],
            ),
            (
                nominator_coldkey,
                vec![(second_netuid.into(), u64::from(nominator_alpha).into())],
            ),
        ];
        expected_nominators.sort_by_key(|(coldkey, _)| *coldkey);

        assert_eq!(delegate_info.registrations, vec![second_netuid.into()]);
        assert_eq!(actual_nominators, expected_nominators);
    });
}

#[test]
fn stage_0_delegate_return_per_1000_uses_total_hotkey_stake() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_hotkey = U256::from(410);
        let subnet_owner_coldkey = U256::from(411);
        let delegate_hotkey = U256::from(420);
        let delegate_owner_coldkey = U256::from(421);
        let nominator_coldkey = U256::from(422);
        let netuid =
            add_dynamic_network_disable_commit_reveal(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let delegate_owner_stake_amount: u64 = 100_000_000_000_000;
        let nominator_stake_amount: u64 = 50_000_000_000_000;
        let reserve_amount: u64 = (delegate_owner_stake_amount + nominator_stake_amount) * 1_000;
        let emission_per_epoch = AlphaCurrency::from(1_000_000_000u64);

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tao_weight(0);
        GameSolver::set_tempo(netuid, 2);
        GameSolver::add_balance_to_coldkey_account(
            &delegate_owner_coldkey,
            delegate_owner_stake_amount + ExistentialDeposit::get(),
        );
        GameSolver::add_balance_to_coldkey_account(
            &nominator_coldkey,
            nominator_stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, delegate_hotkey, delegate_owner_coldkey, 0);
        GameSolver::maybe_become_delegate(&delegate_hotkey);

        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(delegate_owner_coldkey),
            delegate_hotkey,
            netuid,
            TaoCurrency::from(delegate_owner_stake_amount),
        ));
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(nominator_coldkey),
            delegate_hotkey,
            netuid,
            TaoCurrency::from(nominator_stake_amount),
        ));

        let uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &delegate_hotkey)
            .expect("delegate uid should exist");
        Emission::<Test>::mutate(netuid, |emissions| {
            if emissions.len() <= uid as usize {
                emissions.resize(uid as usize + 1, AlphaCurrency::ZERO);
            }
            emissions[uid as usize] = emission_per_epoch;
        });

        let delegate_info =
            GameSolver::get_delegate(delegate_hotkey).expect("delegate info should exist");
        let total_stake: U64F64 =
            u64::from(GameSolver::get_total_stake_for_hotkey(&delegate_hotkey)).into();
        let tempo: U64F64 = GameSolver::get_tempo(netuid).into();
        let epochs_per_day = U64F64::saturating_from_num(7200).safe_div(tempo);
        let emissions_per_day =
            U64F64::from_num(u64::from(emission_per_epoch)).saturating_mul(epochs_per_day);
        let expected_return_per_1000 = GameSolver::return_per_1000_tao_test(
            delegate_info.take,
            total_stake,
            emissions_per_day,
        );

        assert!(delegate_info.total_daily_return.0 > 0);
        assert_eq!(
            delegate_info.return_per_1000.0,
            expected_return_per_1000.saturating_to_num::<u64>()
        );
    });
}

#[test]
fn stage_0_delegate_total_daily_return_applies_take() {
    new_test_ext(1).execute_with(|| {
        let subnet_owner_hotkey = U256::from(430);
        let subnet_owner_coldkey = U256::from(431);
        let delegate_hotkey = U256::from(440);
        let delegate_owner_coldkey = U256::from(441);
        let nominator_coldkey = U256::from(442);
        let netuid =
            add_dynamic_network_disable_commit_reveal(&subnet_owner_hotkey, &subnet_owner_coldkey);
        let delegate_owner_stake_amount: u64 = 100_000_000_000_000;
        let nominator_stake_amount: u64 = 50_000_000_000_000;
        let reserve_amount: u64 = (delegate_owner_stake_amount + nominator_stake_amount) * 1_000;
        let emission_per_epoch = AlphaCurrency::from(1_000_000_000u64);
        let take = Compact(u16::MAX / 2);

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tao_weight(0);
        GameSolver::set_tempo(netuid, 2);
        GameSolver::add_balance_to_coldkey_account(
            &delegate_owner_coldkey,
            delegate_owner_stake_amount + ExistentialDeposit::get(),
        );
        GameSolver::add_balance_to_coldkey_account(
            &nominator_coldkey,
            nominator_stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, delegate_hotkey, delegate_owner_coldkey, 0);
        GameSolver::maybe_become_delegate(&delegate_hotkey);
        Delegates::<Test>::insert(delegate_hotkey, take.0);

        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(delegate_owner_coldkey),
            delegate_hotkey,
            netuid,
            TaoCurrency::from(delegate_owner_stake_amount),
        ));
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(nominator_coldkey),
            delegate_hotkey,
            netuid,
            TaoCurrency::from(nominator_stake_amount),
        ));

        let uid = GameSolver::get_uid_for_net_and_hotkey(netuid, &delegate_hotkey)
            .expect("delegate uid should exist");
        Emission::<Test>::mutate(netuid, |emissions| {
            if emissions.len() <= uid as usize {
                emissions.resize(uid as usize + 1, AlphaCurrency::ZERO);
            }
            emissions[uid as usize] = emission_per_epoch;
        });

        let delegate_info =
            GameSolver::get_delegate(delegate_hotkey).expect("delegate info should exist");
        let tempo: U64F64 = GameSolver::get_tempo(netuid).into();
        let epochs_per_day = U64F64::saturating_from_num(7200).safe_div(tempo);
        let emissions_per_day =
            U64F64::from_num(u64::from(emission_per_epoch)).saturating_mul(epochs_per_day);
        let expected_total_daily_return =
            GameSolver::delegator_daily_return_test(take, emissions_per_day);

        assert!(delegate_info.total_daily_return.0 > 0);
        assert!(delegate_info.total_daily_return.0 < emissions_per_day.saturating_to_num::<u64>());
        assert_eq!(
            delegate_info.total_daily_return.0,
            expected_total_daily_return.saturating_to_num::<u64>()
        );
    });
}

#[test]
fn stage_0_stake_info_omits_swap_era_fields() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(210);
        let owner_coldkey = U256::from(211);
        let staker_hotkey = U256::from(220);
        let staker_coldkey = U256::from(221);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let stake_amount: u64 = 100_000_000_000_000;
        let reserve_amount: u64 = stake_amount * 1_000;

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tao_weight(0);
        GameSolver::add_balance_to_coldkey_account(
            &staker_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, staker_hotkey, staker_coldkey, 0);
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(staker_coldkey),
            staker_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));

        let alpha = GameSolver::get_stake_for_hotkey_and_coldkey_on_subnet(
            &staker_hotkey,
            &staker_coldkey,
            netuid,
        );
        let direct_info = GameSolver::get_stake_info_for_hotkey_coldkey_netuid(
            staker_hotkey,
            staker_coldkey,
            netuid,
        )
        .expect("stake info should exist");
        let coldkey_info = GameSolver::get_stake_info_for_coldkey(staker_coldkey);

        assert_eq!(coldkey_info.len(), 1);

        let decoded_direct = decode_stake_info(&direct_info);
        let decoded_coldkey = decode_stake_info(&coldkey_info[0]);

        for decoded in [decoded_direct, decoded_coldkey] {
            assert_eq!(decoded.hotkey, staker_hotkey);
            assert_eq!(decoded.coldkey, staker_coldkey);
            assert_eq!(decoded.netuid.0, netuid);
            assert_eq!(decoded.stake.0, alpha);
            assert_eq!(decoded.emission.0, AlphaCurrency::ZERO);
            assert!(decoded.is_registered);
        }
    });
}

#[test]
fn stage_0_dynamic_info_omits_dead_zero_fields() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(310);
        let owner_coldkey = U256::from(311);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let reserve_amount: u64 = 100_000_000_000_000;

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());

        let info = GameSolver::get_dynamic_info(netuid).expect("dynamic info should exist");
        let decoded = decode_dynamic_info(&info);

        assert_eq!(decoded.netuid.0, netuid);
        assert_eq!(decoded.owner_hotkey, owner_hotkey);
        assert_eq!(decoded.owner_coldkey, owner_coldkey);
        assert_eq!(decoded.alpha_in.0, SubnetAlphaIn::<Test>::get(netuid));
        assert_eq!(decoded.alpha_out.0, SubnetAlphaOut::<Test>::get(netuid));
        assert_eq!(decoded.tao_in.0, SubnetTAO::<Test>::get(netuid));
        assert_eq!(
            decoded.pending_alpha_emission.0,
            PendingValidatorEmission::<Test>::get(netuid)
                .saturating_add(PendingServerEmission::<Test>::get(netuid))
        );
        assert_eq!(decoded.subnet_volume.0, SubnetVolume::<Test>::get(netuid));
    });
}

#[test]
fn stage_0_subnet_info_omits_dead_zero_fields() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(410);
        let owner_coldkey = U256::from(411);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);

        let info_v2 = GameSolver::get_subnet_info_v2(netuid).expect("subnet info v2 should exist");
        let decoded_v2 = decode_subnet_info_v2(&info_v2);

        assert_eq!(decoded_v2.netuid.0, netuid);
        assert_eq!(decoded_v2.rho.0, GameSolver::get_rho(netuid));
        assert_eq!(decoded_v2.kappa.0, GameSolver::get_kappa(netuid));
        assert_eq!(
            decoded_v2.difficulty.0,
            GameSolver::get_difficulty_as_u64(netuid)
        );
        assert_eq!(
            decoded_v2.immunity_period.0,
            GameSolver::get_immunity_period(netuid)
        );
        assert_eq!(
            decoded_v2.max_allowed_validators.0,
            GameSolver::get_max_allowed_validators(netuid)
        );
        assert_eq!(
            decoded_v2.min_allowed_weights.0,
            GameSolver::get_min_allowed_weights(netuid)
        );
        assert_eq!(
            decoded_v2.max_weights_limit.0,
            GameSolver::get_max_weight_limit(netuid)
        );
        assert_eq!(
            decoded_v2.scaling_law_power.0,
            GameSolver::get_scaling_law_power(netuid)
        );
        assert_eq!(
            decoded_v2.subnetwork_n.0,
            GameSolver::get_subnetwork_n(netuid)
        );
        assert_eq!(
            decoded_v2.max_allowed_uids.0,
            GameSolver::get_max_allowed_uids(netuid)
        );
        assert_eq!(
            decoded_v2.blocks_since_last_step.0,
            GameSolver::get_blocks_since_last_step(netuid)
        );
        assert_eq!(decoded_v2.tempo.0, GameSolver::get_tempo(netuid));
        assert_eq!(decoded_v2.burn.0, GameSolver::get_burn(netuid));
        assert_eq!(decoded_v2.owner, GameSolver::get_subnet_owner(netuid));

        assert_eq!(decoded_v2.identity, SubnetIdentitiesV3::<Test>::get(netuid));
    });
}

#[test]
fn stage_0_subnet_hyperparams_surface_is_v2_only() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(510);
        let owner_coldkey = U256::from(511);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);

        let info_v2 = GameSolver::get_subnet_hyperparams_v2(netuid)
            .expect("subnet hyperparams v2 should exist");
        let decoded_v2 = decode_subnet_hyperparams_v2(&info_v2);

        assert_eq!(decoded_v2.rho.0, GameSolver::get_rho(netuid));
        assert_eq!(decoded_v2.kappa.0, GameSolver::get_kappa(netuid));
        assert_eq!(
            decoded_v2.immunity_period.0,
            GameSolver::get_immunity_period(netuid)
        );
        assert_eq!(
            decoded_v2.min_allowed_weights.0,
            GameSolver::get_min_allowed_weights(netuid)
        );
        assert_eq!(
            decoded_v2.max_weights_limit.0,
            GameSolver::get_max_weight_limit(netuid)
        );
        assert_eq!(decoded_v2.tempo.0, GameSolver::get_tempo(netuid));
        assert_eq!(
            decoded_v2.min_difficulty.0,
            GameSolver::get_min_difficulty(netuid)
        );
        assert_eq!(
            decoded_v2.max_difficulty.0,
            GameSolver::get_max_difficulty(netuid)
        );
        assert_eq!(
            decoded_v2.weights_version.0,
            GameSolver::get_weights_version_key(netuid)
        );
        assert_eq!(
            decoded_v2.weights_rate_limit.0,
            GameSolver::get_weights_set_rate_limit(netuid)
        );
        assert_eq!(
            decoded_v2.adjustment_interval.0,
            GameSolver::get_adjustment_interval(netuid)
        );
        assert_eq!(
            decoded_v2.activity_cutoff.0,
            GameSolver::get_activity_cutoff(netuid)
        );
        assert_eq!(
            decoded_v2.registration_allowed,
            GameSolver::get_network_registration_allowed(netuid)
        );
        assert_eq!(
            decoded_v2.target_regs_per_interval.0,
            GameSolver::get_target_registrations_per_interval(netuid)
        );
        assert_eq!(decoded_v2.min_burn.0, GameSolver::get_min_burn(netuid));
        assert_eq!(decoded_v2.max_burn.0, GameSolver::get_max_burn(netuid));
        assert_eq!(
            decoded_v2.bonds_moving_avg.0,
            GameSolver::get_bonds_moving_average(netuid)
        );
        assert_eq!(
            decoded_v2.max_regs_per_block.0,
            GameSolver::get_max_registrations_per_block(netuid)
        );
        assert_eq!(
            decoded_v2.serving_rate_limit.0,
            GameSolver::get_serving_rate_limit(netuid)
        );
        assert_eq!(
            decoded_v2.max_validators.0,
            GameSolver::get_max_allowed_validators(netuid)
        );
        assert_eq!(
            decoded_v2.adjustment_alpha.0,
            GameSolver::get_adjustment_alpha(netuid)
        );
        assert_eq!(
            decoded_v2.difficulty.0,
            GameSolver::get_difficulty_as_u64(netuid)
        );
        assert_eq!(
            decoded_v2.commit_reveal_period.0,
            GameSolver::get_reveal_period(netuid)
        );
        assert_eq!(
            decoded_v2.commit_reveal_weights_enabled,
            GameSolver::get_commit_reveal_weights_enabled(netuid)
        );
        assert_eq!(
            decoded_v2.alpha_high.0,
            GameSolver::get_alpha_values(netuid).1
        );
        assert_eq!(
            decoded_v2.alpha_low.0,
            GameSolver::get_alpha_values(netuid).0
        );
        assert_eq!(
            decoded_v2.liquid_alpha_enabled,
            GameSolver::get_liquid_alpha_enabled(netuid)
        );
        assert_eq!(
            decoded_v2.alpha_sigmoid_steepness,
            GameSolver::get_alpha_sigmoid_steepness(netuid)
        );
        assert_eq!(decoded_v2.yuma_version.0, 2);
        assert_eq!(
            decoded_v2.subnet_is_active,
            GameSolver::get_subtoken_enabled(netuid)
        );
        assert_eq!(
            decoded_v2.transfers_enabled,
            GameSolver::get_transfer_toggle(netuid)
        );
        assert_eq!(
            decoded_v2.bonds_reset_enabled,
            GameSolver::get_bonds_reset(netuid)
        );
    });
}

#[test]
fn stage_0_subnet_state_omits_cross_subnet_emission_history() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(610);
        let owner_coldkey = U256::from(611);
        let validator_hotkey = U256::from(620);
        let validator_coldkey = U256::from(621);
        let netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let stake_amount: u64 = 100_000_000_000_000;
        let reserve_amount: u64 = stake_amount * 1_000;

        setup_reserves(netuid, reserve_amount.into(), reserve_amount.into());
        GameSolver::set_tao_weight(0);
        GameSolver::add_balance_to_coldkey_account(
            &validator_coldkey,
            stake_amount + ExistentialDeposit::get(),
        );

        register_ok_neuron(netuid, validator_hotkey, validator_coldkey, 0);
        assert_ok!(GameSolver::add_stake(
            RuntimeOrigin::signed(validator_coldkey),
            validator_hotkey,
            netuid,
            TaoCurrency::from(stake_amount),
        ));

        let state = GameSolver::get_subnet_state(netuid).expect("subnet state should exist");
        let decoded = decode_subnet_state(&state);

        let expected_hotkeys = (0..GameSolver::get_subnetwork_n(netuid))
            .map(|uid| Keys::<Test>::get(netuid, uid))
            .collect::<Vec<_>>();
        let expected_coldkeys = expected_hotkeys
            .iter()
            .map(Owner::<Test>::get)
            .collect::<Vec<_>>();

        assert_eq!(decoded.netuid.0, netuid);
        assert_eq!(decoded.hotkeys, expected_hotkeys);
        assert_eq!(decoded.coldkeys, expected_coldkeys);
        assert_eq!(decoded.active, Active::<Test>::get(netuid));
        assert_eq!(
            decoded.validator_permit,
            ValidatorPermit::<Test>::get(netuid)
        );
        assert_eq!(
            decoded.pruning_score,
            PruningScores::<Test>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.last_update,
            LastUpdate::<Test>::get(NetUidStorageIndex::from(netuid))
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.emission,
            Emission::<Test>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.dividends,
            Dividends::<Test>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.incentives,
            Incentive::<Test>::get(NetUidStorageIndex::from(netuid))
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.consensus,
            Consensus::<Test>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.trust,
            Trust::<Test>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            decoded.rank,
            Rank::<Test>::get(netuid)
                .into_iter()
                .map(Compact::from)
                .collect::<Vec<_>>()
        );
        let expected_block_at_registration = (0..GameSolver::get_subnetwork_n(netuid))
            .map(|uid| BlockAtRegistration::<Test>::get(netuid, uid).into())
            .collect::<Vec<_>>();
        let (total_stake_fl, alpha_stake_fl, tao_stake_fl) =
            GameSolver::get_stake_weights_for_network(netuid);
        let expected_alpha = alpha_stake_fl
            .into_iter()
            .map(|value| Compact::from(AlphaCurrency::from(fixed64_to_u64(value))))
            .collect::<Vec<_>>();
        let expected_tao = tao_stake_fl
            .into_iter()
            .map(|value| Compact::from(TaoCurrency::from(fixed64_to_u64(value))))
            .collect::<Vec<_>>();
        let expected_total = total_stake_fl
            .into_iter()
            .map(|value| Compact::from(TaoCurrency::from(fixed64_to_u64(value))))
            .collect::<Vec<_>>();

        assert_eq!(
            decoded.block_at_registration,
            expected_block_at_registration
        );
        assert_eq!(decoded.alpha_stake, expected_alpha);
        assert_eq!(decoded.tao_stake, expected_tao);
        assert_eq!(decoded.total_stake, expected_total);
    });
}

#[test]
fn stage_0_all_dynamic_info_omits_dead_subnet_entries() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(710);
        let owner_coldkey = U256::from(711);
        let live_netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let reserve_amount: u64 = 100_000_000_000_000;
        let dead_netuid = NetUid::from(u16::from(live_netuid) + 100);

        setup_reserves(live_netuid, reserve_amount.into(), reserve_amount.into());
        NetworksAdded::<Test>::insert(dead_netuid, false);

        let infos = GameSolver::get_all_dynamic_info();
        let decoded = decode_dynamic_infos(&infos);
        let expected_live_netuids = GameSolver::get_all_subnet_netuids()
            .into_iter()
            .filter(|netuid| GameSolver::get_dynamic_info(*netuid).is_some())
            .collect::<Vec<_>>();

        assert_eq!(infos.len(), decoded.len());
        assert_eq!(decoded.len(), expected_live_netuids.len());
        assert!(decoded.iter().all(|info| info.netuid.0 != dead_netuid));
        assert!(decoded.iter().any(|info| info.netuid.0 == live_netuid));
    });
}

#[test]
fn stage_0_all_metagraphs_omit_dead_subnet_entries() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(810);
        let owner_coldkey = U256::from(811);
        let live_netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let reserve_amount: u64 = 100_000_000_000_000;
        let dead_netuid = NetUid::from(u16::from(live_netuid) + 100);

        setup_reserves(live_netuid, reserve_amount.into(), reserve_amount.into());
        NetworksAdded::<Test>::insert(dead_netuid, false);

        let metagraphs = GameSolver::get_all_metagraphs();
        let expected = GameSolver::get_all_subnet_netuids()
            .into_iter()
            .filter_map(GameSolver::get_metagraph)
            .collect::<Vec<_>>();

        assert_eq!(metagraphs.len(), expected.len());
        assert_eq!(
            metagraphs.iter().map(Encode::encode).collect::<Vec<_>>(),
            expected.iter().map(Encode::encode).collect::<Vec<_>>()
        );
    });
}

#[test]
fn stage_0_subnets_info_v2_omit_dead_subnet_entries() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(910);
        let owner_coldkey = U256::from(911);
        let live_netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let dead_netuid = NetUid::from(u16::from(live_netuid) + 100);

        NetworksAdded::<Test>::insert(dead_netuid, false);

        let infos = GameSolver::get_subnets_info_v2();
        let decoded = decode_subnet_infos_v2(&infos);
        let expected = GameSolver::get_all_subnet_netuids()
            .into_iter()
            .filter_map(GameSolver::get_subnet_info_v2)
            .collect::<Vec<_>>();

        assert_eq!(infos.len(), expected.len());
        assert_eq!(decoded.len(), expected.len());
        assert_eq!(
            infos.iter().map(Encode::encode).collect::<Vec<_>>(),
            expected.iter().map(Encode::encode).collect::<Vec<_>>()
        );
    });
}

#[test]
fn stage_0_all_mechagraphs_omit_dead_entries() {
    new_test_ext(1).execute_with(|| {
        let owner_hotkey = U256::from(1110);
        let owner_coldkey = U256::from(1111);
        let live_netuid = add_dynamic_network_disable_commit_reveal(&owner_hotkey, &owner_coldkey);
        let dead_netuid = NetUid::from(u16::from(live_netuid) + 100);

        MechanismCountCurrent::<Test>::insert(live_netuid, MechId::from(2u8));
        NetworksAdded::<Test>::insert(dead_netuid, false);

        let mechagraphs = GameSolver::get_all_mechagraphs();
        let expected = GameSolver::get_all_subnet_netuids()
            .into_iter()
            .flat_map(|netuid| {
                let mechanism_count = u8::from(MechanismCountCurrent::<Test>::get(netuid));

                (0..mechanism_count).filter_map(move |mecid| {
                    GameSolver::get_mechagraph(netuid, MechId::from(mecid))
                })
            })
            .collect::<Vec<_>>();

        assert_eq!(mechagraphs.len(), expected.len());
        assert_eq!(
            mechagraphs.iter().map(Encode::encode).collect::<Vec<_>>(),
            expected.iter().map(Encode::encode).collect::<Vec<_>>()
        );
    });
}

#[test]
fn stage_0_register_network_allows_second_registration_when_rate_limit_is_zero() {
    new_test_ext(0).execute_with(|| {
        let first_cold = U256::from(61_u64);
        let first_hot = U256::from(62_u64);
        let second_cold = U256::from(63_u64);
        let second_hot = U256::from(64_u64);
        let lock_cost: u64 = GameSolver::get_network_lock_cost().into();

        GameSolver::set_network_rate_limit(0);
        GameSolver::add_balance_to_coldkey_account(&first_cold, lock_cost.saturating_mul(10));
        GameSolver::add_balance_to_coldkey_account(&second_cold, lock_cost.saturating_mul(10));
        System::set_block_number(10);

        assert_ok!(GameSolver::do_register_network(
            RuntimeOrigin::signed(first_cold),
            &first_hot,
            1,
            None,
        ));
        assert_eq!(GameSolver::get_network_last_lock_block(), 10);
        assert_eq!(
            GameSolver::get_owning_coldkey_for_hotkey(&first_hot),
            first_cold
        );

        assert_ok!(GameSolver::do_register_network(
            RuntimeOrigin::signed(second_cold),
            &second_hot,
            1,
            None,
        ));
        assert_eq!(GameSolver::get_network_last_lock_block(), 10);
        assert_eq!(
            GameSolver::get_owning_coldkey_for_hotkey(&second_hot),
            second_cold
        );
    });
}

#[test]
fn stage_0_register_network_error_indices_snapshot() {
    new_test_ext(0).execute_with(|| {
        println!(
            "NetworkTxRateLimitExceeded={:?}",
            Error::<Test>::NetworkTxRateLimitExceeded.encode()
        );
        println!(
            "SubNetRegistrationDisabled={:?}",
            Error::<Test>::SubNetRegistrationDisabled.encode()
        );
        println!(
            "SubnetLimitReached={:?}",
            Error::<Test>::SubnetLimitReached.encode()
        );
        println!(
            "CannotAffordLockCost={:?}",
            Error::<Test>::CannotAffordLockCost.encode()
        );
        println!(
            "MechanismDoesNotExist={:?}",
            Error::<Test>::MechanismDoesNotExist.encode()
        );
    });
}
