#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
use alloc::vec::Vec;
use pallet_game_solver::rpc_info::neuron_info::NeuronInfoLite;
use sp_runtime::AccountId32;
use subtensor_runtime_common::NetUid;

// Here we declare the runtime API. It is implemented it the `impl` block in
// src/neuron_info.rs, src/subnet_info.rs, and src/delegate_info.rs
sp_api::decl_runtime_apis! {
    pub trait NeuronInfoRuntimeApi {
        fn get_neurons_lite(netuid: NetUid) -> Vec<NeuronInfoLite<AccountId32>>;
    }
}
