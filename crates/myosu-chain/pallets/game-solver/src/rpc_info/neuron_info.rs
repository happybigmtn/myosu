use super::*;
use frame_support::pallet_prelude::{Decode, Encode};
extern crate alloc;
use codec::Compact;
use subtensor_runtime_common::{AlphaCurrency, NetUid, NetUidStorageIndex};

#[freeze_struct("b9fdff7fc6e023c7")]
#[derive(Decode, Encode, PartialEq, Eq, Clone, Debug, TypeInfo)]
pub struct NeuronInfoLite<AccountId: TypeInfo + Encode + Decode> {
    hotkey: AccountId,
    coldkey: AccountId,
    uid: Compact<u16>,
    netuid: Compact<NetUid>,
    active: bool,
    axon_info: AxonInfo,
    prometheus_info: PrometheusInfo,
    stake: Vec<(AccountId, Compact<AlphaCurrency>)>, // map of coldkey to stake on this neuron/hotkey (includes delegations)
    rank: Compact<u16>,
    emission: Compact<AlphaCurrency>,
    incentive: Compact<u16>,
    consensus: Compact<u16>,
    trust: Compact<u16>,
    validator_trust: Compact<u16>,
    dividends: Compact<u16>,
    last_update: Compact<u64>,
    validator_permit: bool,
    // has no weights or bonds
    pruning_score: Compact<u16>,
}

impl<T: Config> Pallet<T> {
    fn get_neuron_stake_entries(
        hotkey: &T::AccountId,
        netuid: NetUid,
    ) -> Vec<(T::AccountId, Compact<AlphaCurrency>)> {
        let mut stake = Vec::new();
        for ((coldkey, coldkey_netuid), alpha_shares) in Alpha::<T>::iter_prefix((hotkey.clone(),))
        {
            if coldkey_netuid != netuid || alpha_shares == 0 {
                continue;
            }

            let alpha_stake =
                Self::get_stake_for_hotkey_and_coldkey_on_subnet(hotkey, &coldkey, netuid);
            if alpha_stake == AlphaCurrency::ZERO {
                continue;
            }

            stake.push((coldkey, alpha_stake.into()));
        }

        stake
    }

    fn get_neuron_lite_subnet_exists(
        netuid: NetUid,
        uid: u16,
    ) -> Option<NeuronInfoLite<T::AccountId>> {
        let hotkey = match Self::get_hotkey_for_net_and_uid(netuid, uid) {
            Ok(h) => h,
            Err(_) => return None,
        };

        let axon_info = Self::get_axon_info(netuid, &hotkey.clone());

        let prometheus_info = Self::get_prometheus_info(netuid, &hotkey.clone());

        let coldkey = Owner::<T>::get(hotkey.clone()).clone();

        let active = Self::get_active_for_uid(netuid, uid);
        let rank = Self::get_rank_for_uid(netuid, uid);
        let emission = Self::get_emission_for_uid(netuid, uid);
        let incentive = Self::get_incentive_for_uid(netuid.into(), uid);
        let consensus = Self::get_consensus_for_uid(netuid, uid);
        let trust = Self::get_trust_for_uid(netuid, uid);
        let validator_trust = Self::get_validator_trust_for_uid(netuid, uid);
        let dividends = Self::get_dividends_for_uid(netuid, uid);
        let pruning_score = Self::get_pruning_score_for_uid(netuid, uid);
        let last_update = Self::get_last_update_for_uid(NetUidStorageIndex::from(netuid), uid);
        let validator_permit = Self::get_validator_permit_for_uid(netuid, uid);

        let stake = Self::get_neuron_stake_entries(&hotkey, netuid);

        let neuron = NeuronInfoLite {
            hotkey: hotkey.clone(),
            coldkey: coldkey.clone(),
            uid: uid.into(),
            netuid: netuid.into(),
            active,
            axon_info,
            prometheus_info,
            stake,
            rank: rank.into(),
            emission: emission.into(),
            incentive: incentive.into(),
            consensus: consensus.into(),
            trust: trust.into(),
            validator_trust: validator_trust.into(),
            dividends: dividends.into(),
            last_update: last_update.into(),
            validator_permit,
            pruning_score: pruning_score.into(),
        };

        Some(neuron)
    }

    pub fn get_neurons_lite(netuid: NetUid) -> Vec<NeuronInfoLite<T::AccountId>> {
        if !Self::if_subnet_exist(netuid) {
            return Vec::new();
        }

        let mut neurons: Vec<NeuronInfoLite<T::AccountId>> = Vec::new();
        let n = Self::get_subnetwork_n(netuid);
        for uid in 0..n {
            let neuron = match Self::get_neuron_lite_subnet_exists(netuid, uid) {
                Some(n) => n,
                None => break, // No more neurons
            };

            neurons.push(neuron);
        }
        neurons
    }
}
