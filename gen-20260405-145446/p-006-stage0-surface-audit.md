# P-006 Stage-0 Surface Audit

Date: 2026-04-05
Scope: `pallet-game-solver` default-build dispatch surface plus all declared pallet storage.

## Evidence

- Dispatch source: `crates/myosu-chain/pallets/game-solver/src/macros/dispatches.rs`
- Storage declarations: `crates/myosu-chain/pallets/game-solver/src/lib.rs`
- Default-build runtime wiring: `crates/myosu-chain/runtime/src/lib.rs`
- Live default-build proof: `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`
- Stage-0 client usage: `crates/myosu-chain-client/src/lib.rs`
- Stage-0 admin / query carry: `crates/myosu-chain/pallets/admin-utils/src/lib.rs`, `crates/myosu-chain/pallets/game-solver/src/rpc_info/`

## Contract Drift

The current plan/spec text is stale in one material way:

- `specs/050426-chain-runtime-pallet.md` and `IMPLEMENTATION_PLAN.md` still describe a 25-call default-build `SubtensorModule` surface.
- The live default-build surface is the 8-call set asserted by `stage_0_flow_dispatch_surface_matches_live_chain_loop`.
- `dispatches.rs` still carries a much larger inherited surface, but 55 of the 63 dispatch variants are already behind `#[cfg(feature = "full-runtime")]`.

This audit therefore treats the live code and tests as authoritative, and treats the 25-call list as historical evidence of an older stage-0 reduction step.

## Machine-Readable Counts

```yaml
dispatch_variants:
  total_in_dispatches_rs: 63
  default_build: 8
  full_runtime_only: 55
storage_items:
  source_declared_total: 193
  already_feature_gated_out_of_default_build: 8
  default_build_source_upper_bound: 185
  active: 128
  deferred: 50
  dead: 15
```

## Extrinsic Verdicts

### Live default-build calls: keep

These 8 calls are the actual stage-0 `SubtensorModule` surface proved by `stage_0_flow.rs` and exercised by the chain client / local loop:

| Extrinsic | Verdict | Why |
|---|---|---|
| `add_stake` | keep | Used by the live chain client and stage-0 flow to fund validator/miner stake. |
| `burned_register` | keep | Used by the live chain client to register subnet members in the local loop. |
| `commit_weights` | keep | One half of the live commit-reveal path; exercised by chain-client weight submission. |
| `register_network` | keep | Used by the chain client to create fresh subnets in stage 0. |
| `reveal_weights` | keep | Second half of the live commit-reveal path. |
| `serve_axon` | keep | Used by the miner/chain-client path to publish HTTP endpoints on-chain. |
| `set_weights` | keep | Live validator submission path when commit-reveal is disabled. |
| `start_call` | keep | Live owner path used by the chain client to enable subnet staking / subtoken flow. |

### Historical 25-call list: already removed from the default build

The following calls are still named in `specs/050426-chain-runtime-pallet.md`, but they are no longer default-build stage-0 calls. They are already behind `full-runtime` in `dispatches.rs`:

- `set_childkey_take`
- `faucet`
- `swap_stake`
- `swap_stake_limit`
- `try_associate_hotkey`
- `associate_evm_key`
- `recycle_alpha`
- `burn_alpha`
- `update_symbol`
- `claim_root`
- `set_root_claim_type`
- `sudo_set_root_claim_threshold`
- `announce_coldkey_swap`
- `swap_coldkey_announced`
- `dispute_coldkey_swap`
- `reset_coldkey_swap`
- `add_stake_burn`

### Extrinsic removal conclusion

- No live default-build `SubtensorModule` extrinsic should be removed in follow-on work.
- Any future extrinsic-reduction task must first be re-scoped, because the active stage-0 game-solver call surface is already the enforced 8-call set.

## Storage Census

Verdict meanings:

- `active`: read or written by the current default-build runtime path, default-build RPC/query surfaces, or the owned stage-0 loop.
- `deferred`: compiled into the default build and still touched by auxiliary/default-build code, but not required by the current stage-0 loop. Removal needs a dedicated slice because code still references it.
- `dead`: safe removal candidates from the stage-0 perspective. Either already feature-gated out of the default build or only serve features the default build cannot reach.

### Active storage (128)

- Economics / staking / emission core (31): `TaoWeight`, `CKBurn`, `Owner`, `AlphaDividendsPerSubnet`, `RootAlphaDividendsPerSubnet`, `BlockEmission`, `LastHotkeyEmissionOnNetuid`, `TotalIssuance`, `TotalStake`, `SubnetTAO`, `SubnetTaoProvided`, `SubnetAlphaInEmission`, `SubnetAlphaOutEmission`, `SubnetTaoInEmission`, `SubnetAlphaIn`, `SubnetAlphaInProvided`, `SubnetAlphaOut`, `StakingHotkeys`, `OwnedHotkeys`, `TotalHotkeyAlpha`, `TotalHotkeyAlphaLastEpoch`, `TotalHotkeyShares`, `Alpha`, `AlphaMapLastKey`, `PendingServerEmission`, `PendingValidatorEmission`, `PendingRootAlphaDivs`, `PendingOwnerCut`, `RAORecycledForRegistration`, `LastColdkeyHotkeyStakeBlock`, `StakingOperationRateLimiter`
- Subnet registry / pricing / lifecycle (40): `SubnetLimit`, `SubnetMovingAlpha`, `SubnetMovingPrice`, `RootProp`, `SubnetVolume`, `SubnetTaoFlow`, `SubnetEmaTaoFlow`, `TaoFlowCutoff`, `FlowNormExponent`, `FlowEmaSmoothingFactor`, `MaxRegistrationsPerBlock`, `TotalNetworks`, `NetworkImmunityPeriod`, `StartCallDelay`, `NetworkMinLockCost`, `NetworkLastLockCost`, `NetworkLockReductionInterval`, `SubnetOwnerCut`, `NetworkRateLimit`, `WeightsVersionKeyRateLimit`, `LastRateLimitedBlock`, `SubnetLocked`, `LargestLocked`, `Tempo`, `FirstEmissionBlockNumber`, `SubnetMechanism`, `SubnetworkN`, `NetworksAdded`, `IsNetworkMember`, `NetworkRegistrationAllowed`, `NetworkRegisteredAt`, `BlocksSinceLastStep`, `LastMechansimStepBlock`, `SubnetOwner`, `SubnetOwnerHotkey`, `RecycleOrBurn`, `ServingRateLimit`, `NetworkRegistrationStartBlock`, `MinNonImmuneUids`, `MechanismCountCurrent`
- Weights / consensus / pruning hyperparameters (36): `Rho`, `AlphaSigmoidSteepness`, `Kappa`, `RegistrationsThisInterval`, `BurnRegistrationsThisInterval`, `MinAllowedUids`, `MaxAllowedUids`, `ImmunityPeriod`, `ActivityCutoff`, `MaxWeightsLimit`, `WeightsVersionKey`, `MinAllowedWeights`, `MaxAllowedValidators`, `AdjustmentInterval`, `BondsMovingAverage`, `BondsPenalty`, `BondsResetOn`, `WeightsSetRateLimit`, `ValidatorPruneLen`, `ScalingLawPower`, `TargetRegistrationsPerInterval`, `AdjustmentAlpha`, `CommitRevealWeightsEnabled`, `Burn`, `LastAdjustmentBlock`, `RegistrationsThisBlock`, `EMAPriceHalvingBlocks`, `LiquidAlphaOn`, `Yuma3On`, `AlphaValues`, `SubtokenEnabled`, `ImmuneOwnerUidsLimit`, `StakeThreshold`, `WeightCommits`, `RevealPeriodEpochs`, `TxRateLimit`
- Metagraph / RPC-visible state (21): `TokenSymbol`, `StakeWeight`, `Uids`, `Keys`, `LoadedEmission`, `Active`, `Rank`, `Trust`, `Consensus`, `Incentive`, `Dividends`, `Emission`, `LastUpdate`, `ValidatorTrust`, `PruningScores`, `ValidatorPermit`, `Weights`, `Bonds`, `BlockAtRegistration`, `Axons`, `TransactionKeyLastBlock`

### Deferred storage (50)

- Delegation / childkey / autostake carry (14): `MaxDelegateTake`, `MinDelegateTake`, `MaxChildkeyTake`, `MinChildkeyTake`, `Delegates`, `ChildkeyTake`, `PendingChildKeys`, `ChildKeys`, `ParentKeys`, `AutoStakeDestination`, `AutoStakeDestinationColdkeys`, `TxDelegateTakeRateLimit`, `TxChildkeyTakeRateLimit`, `PendingChildKeyCooldown`
- PoW / faucet / dormant registration tuning (8): `NetworkPowRegistrationAllowed`, `Difficulty`, `MinBurn`, `MaxBurn`, `MinDifficulty`, `MaxDifficulty`, `UsedWork`, `POWRegistrationsThisInterval`
- Voting / identity / telemetry metadata (8): `VotingPower`, `VotingPowerTrackingEnabled`, `VotingPowerDisableAtBlock`, `VotingPowerEmaAlpha`, `NeuronCertificates`, `Prometheus`, `IdentitiesV2`, `SubnetIdentitiesV3`
- Root-claim carry and legacy rate-limit keys (11): `LastTxBlock`, `LastTxBlockChildKeyTake`, `LastTxBlockDelegateTake`, `RootClaimableThreshold`, `RootClaimable`, `RootClaimed`, `RootClaimType`, `StakingColdkeysByIndex`, `StakingColdkeys`, `NumStakingColdkeys`, `NumRootClaim`
- Admin / migration / mechanism caps (9): `MinActivityCutoff`, `AdminFreezeWindow`, `OwnerHyperparamRateLimit`, `DissolveNetworkScheduleDuration`, `NominatorMinRequiredStake`, `TransferToggle`, `MaxMechanismCount`, `MechanismEmissionSplit`, `HasMigrationRun`

### Dead storage (15)

- Already feature-gated out of the default build (8): `TimelockedWeightCommits`, `CRV3WeightCommits`, `CRV3WeightCommitsV2`, `SubnetLeases`, `SubnetLeaseShares`, `SubnetUidToLeaseId`, `NextSubnetLeaseId`, `AccumulatedLeaseDividends`
- Default-build-unreachable coldkey-swap surface (5): `LastHotkeySwapOnNetuid`, `ColdkeySwapAnnouncementDelay`, `ColdkeySwapReannouncementDelay`, `ColdkeySwapAnnouncements`, `ColdkeySwapDisputes`
- Orphaned default-build leftovers (2): `NextStakeJobId`, `AssociatedEvmAddress`

## Recommended Removal Order

### Batch 1: dead storage only

Remove the 15 `dead` items first. This batch has the clearest safety story:

1. Delete the 8 storages already hidden behind `legacy-subtensor-tests`.
2. Delete the 5 coldkey-swap storages that no default-build call can reach.
3. Delete `NextStakeJobId` and `AssociatedEvmAddress`, then remove their residual helpers / cleanup branches.

Required proof for Batch 1:

- `cargo test -p pallet-game-solver stage_0_flow_dispatch_surface_matches_live_chain_loop --quiet`
- `cargo test -p pallet-game-solver stage_0_flow_registers_stakes_serves_and_emits --quiet`
- metadata / grep assertion that the deleted keys no longer appear in the default-build pallet.

### Batch 2: deferred groups, one domain at a time

Do not remove the 50 `deferred` items as one sweep. Split by domain because each group still has live code references:

1. Delegation / childkey / autostake carry
2. PoW / faucet carry
3. Voting / identity / telemetry metadata
4. Root-claim carry and legacy rate-limit keys
5. Admin / migration / mechanism caps

Each batch should:

- remove the now-unneeded code paths, not just the storage items
- add negative tests proving the feature is absent from the default build
- re-run `stage_0_flow` plus any surviving focused pallet tests for the touched domain

## Verification Run

Executed for this audit:

- `cargo test -p pallet-game-solver stage_0_flow_dispatch_surface_matches_live_chain_loop --quiet`
- `cargo test -p pallet-game-solver stage_0_flow_registers_stakes_serves_and_emits --quiet`
- source census over `dispatches.rs` and `lib.rs` to count call/storage declarations and identify feature-gated storage

Result: both direct stage-0 proof tests pass, and the live default-build call surface is the 8-call set documented above.
