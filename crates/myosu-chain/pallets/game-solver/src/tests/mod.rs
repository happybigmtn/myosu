// QUAL-003 Triage (2026-03-30):
//
// Stage-0 relevant (promote once test mock drops drand/crowdloan/swap deps):
//   registration, weights, epoch, serving, staking, uids, networks, neuron_info
//
// Keep feature-gated (probably needed later):
//   coinbase, emission, consensus, math, difficulty, migration
//
// Subtensor-specific (delete when mock is simplified):
//   auto_stake_hotkey, batch_tx, children, claim_root, delegate_info, evm,
//   leasing, mechanism, move_stake, recycle_alpha, swap_coldkey, swap_hotkey,
//   swap_hotkey_with_subnet, voting_power, epoch_logs, ensure, subnet,
//   subnet_emissions
#[cfg(feature = "legacy-subtensor-tests")]
mod auto_stake_hotkey;
#[cfg(feature = "legacy-subtensor-tests")]
mod batch_tx;
#[cfg(feature = "legacy-subtensor-tests")]
mod children;
#[cfg(feature = "legacy-subtensor-tests")]
mod claim_root;
#[cfg(feature = "legacy-subtensor-tests")]
mod coinbase;
#[cfg(feature = "legacy-subtensor-tests")]
mod consensus;
#[cfg(feature = "legacy-subtensor-tests")]
mod delegate_info;
mod determinism;
#[cfg(feature = "legacy-subtensor-tests")]
mod difficulty;
#[cfg(feature = "legacy-subtensor-tests")]
mod emission;
#[cfg(feature = "legacy-subtensor-tests")]
mod ensure;
#[cfg(feature = "legacy-subtensor-tests")]
mod epoch;
#[cfg(feature = "legacy-subtensor-tests")]
mod epoch_logs;
#[cfg(feature = "legacy-subtensor-tests")]
mod evm;
#[cfg(feature = "legacy-subtensor-tests")]
mod leasing;
#[cfg(feature = "legacy-subtensor-tests")]
mod math;
#[cfg(feature = "legacy-subtensor-tests")]
mod mechanism;
#[cfg(feature = "legacy-subtensor-tests")]
mod migration;
pub(crate) mod mock;
#[cfg(feature = "legacy-subtensor-tests")]
mod move_stake;
#[cfg(feature = "legacy-subtensor-tests")]
mod networks;
#[cfg(feature = "legacy-subtensor-tests")]
mod neuron_info;
#[cfg(feature = "legacy-subtensor-tests")]
mod recycle_alpha;
#[cfg(feature = "legacy-subtensor-tests")]
mod registration;
#[cfg(feature = "legacy-subtensor-tests")]
mod serving;
mod stage_0_flow;
#[cfg(feature = "legacy-subtensor-tests")]
mod staking;
#[cfg(feature = "legacy-subtensor-tests")]
mod staking2;
#[cfg(feature = "legacy-subtensor-tests")]
mod subnet;
#[cfg(feature = "legacy-subtensor-tests")]
mod subnet_emissions;
#[cfg(feature = "legacy-subtensor-tests")]
mod swap_coldkey;
#[cfg(feature = "legacy-subtensor-tests")]
mod swap_hotkey;
#[cfg(feature = "legacy-subtensor-tests")]
mod swap_hotkey_with_subnet;
#[cfg(feature = "legacy-subtensor-tests")]
mod uids;
#[cfg(feature = "legacy-subtensor-tests")]
mod voting_power;
#[cfg(feature = "legacy-subtensor-tests")]
mod weights;
