#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]

use super::mock::*;
use crate::*;

use frame_support::assert_ok;
use sp_core::U256;
use substrate_fixed::types::I64F64;
use subtensor_runtime_common::{AlphaCurrency, NetUid, NetUidStorageIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
struct EpochSnapshot {
    epoch_output: Vec<(u64, u64, u64)>,
    combined_emission: Vec<u64>,
    consensus: Vec<u16>,
    incentive: Vec<u16>,
    dividends: Vec<u16>,
    validator_trust: Vec<u16>,
    validator_permit: Vec<bool>,
}

fn assert_ratios_within_epsilon(left: &[u64], right: &[u64], total: u64, label: &str) {
    assert_eq!(left.len(), right.len(), "{label} vector lengths must match");

    let epsilon = I64F64::from_num(1e-6_f64);
    let total = I64F64::from_num(total);

    for (uid, (left_value, right_value)) in left.iter().zip(right.iter()).enumerate() {
        let left_ratio = I64F64::from_num(*left_value) / total;
        let right_ratio = I64F64::from_num(*right_value) / total;
        let diff = if left_ratio >= right_ratio {
            left_ratio - right_ratio
        } else {
            right_ratio - left_ratio
        };

        assert!(
            diff <= epsilon,
            "{label} diverged past epsilon for uid {uid}: left={left_ratio:?} right={right_ratio:?} diff={diff:?}"
        );
    }
}

fn run_yuma_epoch_snapshot() -> EpochSnapshot {
    new_test_ext(1).execute_with(|| {
        let netuid: NetUid = 19_u16.into();
        let rao_emission: AlphaCurrency = 1_000_000_000_u64.into();
        let hotkeys = [
            U256::from(10_u64),
            U256::from(11_u64),
            U256::from(12_u64),
            U256::from(13_u64),
            U256::from(14_u64),
        ];
        let validator_stakes = [700_000_u64, 300_000_u64, 0_u64, 0_u64, 0_u64];

        add_network_disable_commit_reveal(netuid, u16::MAX - 1, 0);
        SubtensorModule::set_max_allowed_uids(netuid, hotkeys.len() as u16);
        SubtensorModule::set_max_allowed_validators(netuid, 2);
        SubtensorModule::set_weights_set_rate_limit(netuid, 0);
        SubtensorModule::set_stake_threshold(0);

        for (hotkey, stake) in hotkeys.iter().zip(validator_stakes) {
            SubtensorModule::add_balance_to_coldkey_account(hotkey, stake);
            SubtensorModule::append_neuron(netuid, hotkey, 0);
            SubtensorModule::increase_stake_for_hotkey_and_coldkey_on_subnet(
                hotkey,
                hotkey,
                netuid,
                stake.into(),
            );
        }

        assert_eq!(
            SubtensorModule::get_subnetwork_n(netuid),
            hotkeys.len() as u16
        );

        SubtensorModule::epoch(netuid, 1_u64.into());
        assert_eq!(
            SubtensorModule::get_validator_permit(netuid),
            vec![true, true, false, false, false],
            "bootstrap epoch should select the two staked validators"
        );

        let server_uids = vec![2_u16, 3_u16, 4_u16];
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkeys[0]),
            netuid,
            server_uids.clone(),
            vec![40_000_u16, 20_000_u16, 5_535_u16],
            0
        ));
        assert_ok!(SubtensorModule::set_weights(
            RuntimeOrigin::signed(hotkeys[1]),
            netuid,
            server_uids,
            vec![10_000_u16, 25_000_u16, 30_535_u16],
            0
        ));

        let epoch_output = SubtensorModule::epoch(netuid, rao_emission);

        EpochSnapshot {
            epoch_output: epoch_output
                .into_iter()
                .map(|(hotkey, server_emission, validator_emission)| {
                    (
                        hotkey.low_u64(),
                        u64::from(server_emission),
                        u64::from(validator_emission),
                    )
                })
                .collect(),
            combined_emission: SubtensorModule::get_emission(netuid)
                .into_iter()
                .map(u64::from)
                .collect(),
            consensus: SubtensorModule::get_consensus(netuid),
            incentive: SubtensorModule::get_incentive(NetUidStorageIndex::from(netuid)),
            dividends: SubtensorModule::get_dividends(netuid),
            validator_trust: SubtensorModule::get_validator_trust(netuid),
            validator_permit: SubtensorModule::get_validator_permit(netuid),
        }
    })
}

#[test]
fn determinism_identical_yuma_inputs_produce_stable_epoch_outputs() {
    let first_run = run_yuma_epoch_snapshot();
    let second_run = run_yuma_epoch_snapshot();

    assert_eq!(
        first_run, second_run,
        "stage-0 expects bit-stable Yuma output for identical inputs"
    );

    let total_emission = 1_000_000_000_u64;
    let first_server_emission: Vec<u64> = first_run
        .epoch_output
        .iter()
        .map(|(_, server_emission, _)| *server_emission)
        .collect();
    let second_server_emission: Vec<u64> = second_run
        .epoch_output
        .iter()
        .map(|(_, server_emission, _)| *server_emission)
        .collect();
    let first_validator_emission: Vec<u64> = first_run
        .epoch_output
        .iter()
        .map(|(_, _, validator_emission)| *validator_emission)
        .collect();
    let second_validator_emission: Vec<u64> = second_run
        .epoch_output
        .iter()
        .map(|(_, _, validator_emission)| *validator_emission)
        .collect();

    assert_ratios_within_epsilon(
        &first_server_emission,
        &second_server_emission,
        total_emission,
        "server emission",
    );
    assert_ratios_within_epsilon(
        &first_validator_emission,
        &second_validator_emission,
        total_emission,
        "validator emission",
    );
    assert_ratios_within_epsilon(
        &first_run.combined_emission,
        &second_run.combined_emission,
        total_emission,
        "combined emission",
    );
}
