use super::mock::*;

use sp_core::U256;
use subtensor_runtime_common::NetUid;

#[test]
fn test_get_neurons_lite_empty() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let neurons = SubtensorModule::get_neurons_lite(netuid);
        assert!(neurons.is_empty());
    });
}

#[test]
fn test_get_neurons_lite_list() {
    new_test_ext(1).execute_with(|| {
        let netuid = NetUid::from(1);
        let tempo: u16 = 2;
        let modality: u16 = 2;
        let neuron_count = 2;

        add_network(netuid, tempo, modality);

        for index in 0..neuron_count {
            let hotkey = U256::from(index);
            let coldkey = U256::from(index);
            let nonce: u64 = 39_420_842 + index;
            register_ok_neuron(netuid, hotkey, coldkey, nonce);
        }

        let neurons = SubtensorModule::get_neurons_lite(netuid);
        assert_eq!(neurons.len(), neuron_count as usize);
    });
}
