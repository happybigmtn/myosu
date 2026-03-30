# Specification: Game-Solving Pallet — On-Chain Incentive Engine for Solver Markets

Source: Master spec AC-CH-02, subtensor pallet analysis at coding/subtensor/pallets/subtensor/
Status: Draft
Date: 2026-03-30
Depends-on: CF-01..05 (chain fork scaffold must compile and produce blocks)

## Purpose

Implement `pallet_game_solver` as a FRAME pallet that coordinates decentralized
game-solving markets. Miners register on subnets, produce strategy profiles,
and serve them via axon endpoints. Validators score miners by computing strategy
exploitability and submit weight vectors. Every `tempo` blocks, Yuma Consensus
aggregates validator weights, clips outliers, computes bond EMAs, and distributes
token emissions to the highest-quality solvers.

This pallet replaces subtensor's `pallet_subtensor` at runtime index 7. It
ports the essential incentive mechanics (Yuma Consensus, neuron registration,
weight submission, commit-reveal) while stripping AI-specific complexity
(Alpha token AMM, root network, coldkey swap, dynamic TAO, childkey delegation).

The primary consumer is the miner and validator binaries. The secondary consumer
is the gameplay layer, which queries the chain to discover the best-ranked miner.

**Key design constraint**: the Yuma Consensus math must produce identical results
to subtensor for identical inputs (INV-003 verification determinism). Port the
algorithm, don't reinvent it.

## Whole-System Goal

Current state:
- Chain fork scaffold (CF-01..05) produces blocks with no game logic
- subtensor's pallet_subtensor has ~66 storage items, 50+ extrinsics, 3200
  lines of epoch/math code, and handles AI subnet incentives
- `crates/myosu-chain/pallets/game-solver/` already exists and is the live
  stage-0 incentive surface used by the owned local loop

This spec adds:
- the truthful narrowed contract for the live pallet that now exists
- the remaining stage-0 hardening and reduction work on top of the inherited
  pallet carry
- a canonical ownership map that matches the real module layout

If all ACs land:
- A subnet can be created for any game type
- Miners register, serve axons, and receive emissions proportional to quality
- Validators set weights and earn bond-weighted dividends
- Yuma Consensus runs every tempo blocks and produces identical scores to subtensor
- Staking determines validator voting power

Still not solved here:
- Alpha token per subnet / AMM pool (future economics spec)
- Root network and cross-subnet emission allocation
- Childkey delegation
- Coldkey/hotkey swap
- Dynamic difficulty adjustment
- Most hyperparameter tuning (use defaults)
- Governance beyond Sudo

12-month direction:
- Per-subnet tokenomics with AMM pricing
- Dynamic emission allocation across subnets based on activity
- On-chain governance replacing Sudo
- Benchmarked runtime weights for all extrinsics

## Why This Spec Exists As One Unit

- The pallet's extrinsics, storage, consensus algorithm, and emission logic
  are deeply intertwined — `on_initialize` calls Yuma which reads weights
  which were set by extrinsics which require registered neurons
- Testing any piece in isolation is possible but proving the incentive loop
  works requires the whole pallet
- The ~9 ACs follow the dependency chain: scaffold → storage → registration →
  weights → consensus → emission → serving → staking → runtime integration

## Scope

In scope:
- FRAME pallet with Config trait, storage, extrinsics, events, errors
- Subnet creation/dissolution with game_type metadata
- Neuron registration (burn-based, pruning of weakest)
- Weight submission (set_weights, commit_weights, reveal_weights)
- Yuma Consensus epoch execution (ported from subtensor)
- Fixed-point math utilities (ported from subtensor)
- Token emission to miners and validators
- Axon endpoint registration
- Native token staking per subnet
- Runtime integration at index 7

Out of scope:
- Alpha token / AMM pool per subnet — future economics spec
- Root network (subnet 0) — unnecessary for bootstrap
- PoW registration — burn-only is simpler
- Childkey delegation — unnecessary complexity
- Coldkey/hotkey swap — can be added later
- Dynamic difficulty — fixed burn cost for bootstrap
- Prometheus serving — nice-to-have, not critical
- Benchmarked weights — use placeholder weights initially

## Current State

- `/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/lib.rs` —
  live pallet root with imported config, dispatch, events, hooks, and stage-0
  swap seam
- `/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/subnets/`
  — live subnet registration, serving, UID, and weight paths
- `/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/epoch/` —
  live Yuma math and epoch execution surfaces
- `/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/coinbase/`
  — live block-step and emission execution seams
- `/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/staking/`
  — live staking and ownership flows still carried for stage 0
- `/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/rpc_info/`
  — typed query helpers for chain/operator consumers
- `/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs`
  — live stage-0 pallet proof coverage

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| Pallet root and storage | `crates/myosu-chain/pallets/game-solver/src/lib.rs` | extend | This is the live pallet, not a missing future crate |
| Subnet management | `crates/myosu-chain/pallets/game-solver/src/subnets/` | extend | Registration, serving, weights, and UID paths already live here |
| Yuma execution | `crates/myosu-chain/pallets/game-solver/src/epoch/` | extend | Current epoch mechanism and math live here now |
| Emission loop | `crates/myosu-chain/pallets/game-solver/src/coinbase/` | extend | Current block-step and emission paths already exist |
| Staking and ownership | `crates/myosu-chain/pallets/game-solver/src/staking/` | extend with care | Stage-0 still carries inherited staking seams here |
| Typed query helpers | `crates/myosu-chain/pallets/game-solver/src/rpc_info/` | reuse and extend | Current node/operator query surface builds on these helpers |
| Stage-0 pallet proof | `crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs` | reuse | The live local truth is already covered here |

## Non-goals

- Token economics design — use flat emission, no AMM, no halving for bootstrap
- Root network governance — Sudo controls subnet parameters
- Competitive subnet registration — no lock cost auction, fixed cost
- Coldkey rotation or key management — basic staking only
- Migration from subtensor state — this is a fresh chain
- Benchmark-derived weights — use placeholder extrinsic weights

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Pallet scaffold | Implemented | crates/myosu-chain/pallets/game-solver/src/lib.rs |
| Config, events, hooks, errors | Implemented | crates/myosu-chain/pallets/game-solver/src/macros/ |
| Subnet management | Implemented | crates/myosu-chain/pallets/game-solver/src/subnets/ |
| Yuma Consensus and math | Implemented | crates/myosu-chain/pallets/game-solver/src/epoch/ |
| Emission and block step | Implemented | crates/myosu-chain/pallets/game-solver/src/coinbase/ |
| Staking | Implemented | crates/myosu-chain/pallets/game-solver/src/staking/ |
| Typed RPC info helpers | Implemented | crates/myosu-chain/pallets/game-solver/src/rpc_info/ |
| Stage-0 proof tests | Implemented | crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs |

## Architecture / Runtime Contract

```
construct_runtime! {
    ...
    GameSolver: pallet_game_solver = 7,   // replaces SubtensorModule
    ...
}

on_initialize(block_number) {
    for subnet in active_subnets {
        if block_number % subnet.tempo == 0 {
            let weights = read_weight_matrix(subnet);
            let stakes = read_validator_stakes(subnet);
            let epoch_output = yuma_consensus(weights, stakes, subnet.kappa);
            distribute_emissions(subnet, epoch_output);
            update_bonds(subnet, epoch_output);
        }
    }
}
```

Primary loop:
- Trigger: `on_initialize` every block, epoch fires at tempo boundary
- Source of truth: Weights storage, Stake storage, subnet hyperparameters
- Processing: Yuma Consensus computes incentive, dividends, bonds
- Persisted truth: Emission, Bonds, Incentive, Rank, Trust storage maps
- Consumer: miners (emission rewards), validators (dividend rewards)

Failure loop:
- No weights submitted → all emissions zero → subnet stalls
- Invalid weights → rejected by extrinsic validation → error event
- Consensus divergence → INV-003 violation → freeze emissions

## Adoption / Consumption Path

- Producer: pallet extrinsics create subnets, register neurons, submit weights
- First consumer: miner/validator binaries call extrinsics via RPC
- Operator-visible surface: chain RPC queries for subnet state, neuron scores
- Why it changes behavior now: enables the solver incentive loop
- If not consumed yet: gameplay layer (GP-01) queries best miner via RPC

---

## A. Pallet Foundation

### AC-GS-01: Pallet Scaffold with Config and Storage

- Where: `crates/myosu-chain/pallets/game-solver/src/lib.rs (new)`
- How: Create the FRAME pallet skeleton with `#[pallet::pallet]`,
  `#[pallet::config]`, `#[pallet::storage]`, `#[pallet::event]`,
  `#[pallet::error]`.

  **Config trait** (simplified from subtensor — remove drand, crowdloan, swap deps):
  ```rust
  #[pallet::config]
  pub trait Config: frame_system::Config {
      type RuntimeEvent: From<Event<Self>> + IsType<...>;
      type Currency: fungible::Balanced<Self::AccountId, Balance = u64>
                   + fungible::Mutate<Self::AccountId>;
      #[pallet::constant]
      type InitialTempo: Get<u16>;         // default 100
      #[pallet::constant]
      type InitialBurnCost: Get<u64>;      // default 1_000_000_000 (1 MYOSU)
      #[pallet::constant]
      type MaxSubnets: Get<u16>;           // default 32
      #[pallet::constant]
      type MaxNeuronsPerSubnet: Get<u16>;  // default 256
  }
  ```

  **Storage items** (~25, down from subtensor's 66):

  Subnet management:
  - `NetworksAdded`: StorageMap<u16, bool>
  - `TotalNetworks`: StorageValue<u16>
  - `SubnetworkN`: StorageMap<u16, u16> — neuron count per subnet
  - `GameType`: StorageMap<u16, Vec<u8>> — opaque game type identifier
  - `Tempo`: StorageMap<u16, u16> — blocks per epoch
  - `Kappa`: StorageMap<u16, u16> — consensus threshold (default 32768 = 0.5)
  - `MaxAllowedUids`: StorageMap<u16, u16>
  - `MaxAllowedValidators`: StorageMap<u16, u16>
  - `SubnetOwner`: StorageMap<u16, T::AccountId>
  - `ImmunityPeriod`: StorageMap<u16, u16>

  Neuron registry:
  - `Keys`: StorageDoubleMap<u16, u16, T::AccountId> — (subnet, uid) → hotkey
  - `Uids`: StorageDoubleMap<u16, T::AccountId, u16> — (subnet, hotkey) → uid
  - `IsNetworkMember`: StorageDoubleMap<T::AccountId, u16, bool>
  - `BlockAtRegistration`: StorageDoubleMap<u16, u16, u64>

  Weights & consensus:
  - `Weights`: StorageDoubleMap<u16, u16, Vec<(u16, u16)>>
  - `Bonds`: StorageDoubleMap<u16, u16, Vec<(u16, u16)>>
  - `Incentive`: StorageMap<u16, Vec<u16>>
  - `Rank`: StorageMap<u16, Vec<u16>>
  - `Trust`: StorageMap<u16, Vec<u16>>
  - `Dividends`: StorageMap<u16, Vec<u16>>
  - `Emission`: StorageMap<u16, Vec<u64>>
  - `ValidatorPermit`: StorageMap<u16, Vec<bool>>
  - `PruningScores`: StorageMap<u16, Vec<u16>>
  - `LastUpdate`: StorageMap<u16, Vec<u64>>

  Commit-reveal:
  - `WeightCommits`: StorageDoubleMap<u16, T::AccountId, (H256, u64)> — (subnet, hotkey) → (hash, block)

  Staking:
  - `TotalStake`: StorageValue<u64>
  - `Stake`: StorageDoubleMap<T::AccountId, u16, u64> — (hotkey, subnet) → amount

  Emission config:
  - `EmissionSplit`: StorageMap<u16, u8> — subnet → miner percentage (default 50, meaning 50/50)
  - `BlockEmission`: StorageValue<u64> — tokens minted per block (default 1_000_000_000)

  Serving:
  - `Axons`: StorageDoubleMap<u16, T::AccountId, AxonInfo>

  **AxonInfo struct**:
  ```rust
  #[derive(Encode, Decode, Clone, TypeInfo, MaxEncodedLen)]
  pub struct AxonInfo {
      pub ip: u128,      // IPv4 or IPv6
      pub port: u16,
      pub ip_type: u8,   // 4 or 6
      pub protocol: u8,  // 0=gRPC, 1=HTTP
      pub version: u32,
  }
  ```

- Whole-system effect: defines the data model that all other ACs operate on.
  Without storage items, nothing can be registered, weighted, or rewarded.
- State: all storage items listed above.
- Wiring contract:
  - Trigger: pallet included in runtime via `construct_runtime!`
  - Callsite: `crates/myosu-chain/runtime/src/lib.rs`
  - State effect: storage items available in runtime state
  - Persistence effect: on-chain storage in block database
  - Observable signal: `cargo test -p pallet-game-solver` compiles
- Required tests:
  - `cargo test -p pallet-game-solver scaffold::tests::pallet_compiles`
  - `cargo test -p pallet-game-solver scaffold::tests::storage_defaults`
  - `cargo test -p pallet-game-solver scaffold::tests::config_constants`
- Pass/fail:
  - Pallet compiles with `cargo build -p pallet-game-solver`
  - All storage items have correct default values (0, false, empty vec)
  - Config constants (InitialTempo=100, MaxSubnets=32, etc.) are accessible
  - Events and errors are defined and encodable
  - `AxonInfo` encodes/decodes correctly
  - Mock runtime test infrastructure works (`mock.rs` with `construct_runtime!`,
    `impl Config for Test`, genesis builder, test helpers)
- Blocking note: every other GS-* AC reads or writes these storage items.
  The mock runtime is required for ALL pallet unit tests (GS-02 through GS-08).
  The scaffold must be solid before adding logic.
- Rollback condition: storage layout is incompatible with Yuma's access patterns.

---

## B. Subnet & Neuron Management

### AC-GS-02: Subnet Registry

- Where: `crates/myosu-chain/pallets/game-solver/src/subnets/`
- How: Implement extrinsics for subnet lifecycle:

  **`create_subnet(origin, game_type: Vec<u8>, tempo: u16)`**:
  1. Ensure signed origin (subnet owner)
  2. Check `TotalNetworks < MaxSubnets`
  3. Burn `InitialBurnCost` from caller
  4. Assign next available subnet_id (scan for first unused ≥ 1)
  5. Initialize storage: `NetworksAdded`, `GameType`, `Tempo`, `Kappa` (default),
     `MaxAllowedUids` (default 256), `MaxAllowedValidators` (default 64),
     `SubnetOwner`, `ImmunityPeriod` (default 100)
  6. Emit `SubnetCreated { subnet_id, game_type, owner }`

  **`dissolve_subnet(origin, subnet_id: u16)`**:
  1. Ensure Sudo or subnet owner
  2. Clear all storage for subnet_id (neurons, weights, bonds, etc.)
  3. Decrement `TotalNetworks`
  4. Emit `SubnetDissolved { subnet_id }`

  **`set_subnet_hyperparams(origin, subnet_id, params)`** (Sudo only):
  - Tempo, Kappa, MaxAllowedUids, MaxAllowedValidators, ImmunityPeriod

  Simplified from subtensor: no lock cost auction, no subnet pruning based on
  emission, no root network voting on subnet allocation. Fixed burn cost.

- Whole-system effect: subnets are the markets. Without them, miners and
  validators have nowhere to register.
- State: SubnetRegistry storage items (NetworksAdded, GameType, Tempo, etc.)
- Wiring contract:
  - Trigger: `create_subnet` extrinsic from signed origin
  - Callsite: pallet dispatchable call
  - State effect: new subnet registered with game_type and default params
  - Persistence effect: 10 storage items written per subnet
  - Observable signal: `SubnetCreated` event emitted, queryable via RPC
- Required tests:
  - `cargo test -p pallet-game-solver subnets::tests::create_subnet_basic`
  - `cargo test -p pallet-game-solver subnets::tests::create_subnet_burns_tokens`
  - `cargo test -p pallet-game-solver subnets::tests::create_subnet_max_limit`
  - `cargo test -p pallet-game-solver subnets::tests::dissolve_subnet_clears_state`
  - `cargo test -p pallet-game-solver subnets::tests::set_hyperparams_sudo_only`
- Pass/fail:
  - `create_subnet(b"nlhe_hu", 100)` → subnet_id 1, event emitted
  - Caller's balance decreased by InitialBurnCost
  - Second subnet gets id 2; `TotalNetworks` == 2
  - 33rd subnet fails with `MaxSubnetsReached` (if MaxSubnets=32)
  - `dissolve_subnet(1)` clears all storage; `NetworksAdded(1)` == false
  - Non-sudo, non-owner calling `set_subnet_hyperparams` → `BadOrigin`
  - Re-creating after dissolve reuses the subnet_id
- Blocking note: neurons can only register on existing subnets. This AC
  unblocks GS-03.
- Rollback condition: game_type storage can't represent the needed metadata
  for different game variants.

### AC-GS-03: Neuron Registration and Pruning

- Where: `crates/myosu-chain/pallets/game-solver/src/registration.rs (new)`
- How: Implement burn-based neuron registration (simplified from subtensor —
  no PoW, no dynamic difficulty).

  **`register_neuron(origin, subnet_id: u16)`**:
  1. Ensure signed origin (the signing account acts as both hotkey and funder)
  2. Check subnet exists and registration enabled
  3. Check account not already registered on this subnet
  4. Burn `InitialBurnCost` from the signing account (no separate coldkey for bootstrap)

  Note: any registered neuron may act as both miner and validator. The role
  distinction is off-chain: miners serve axons, validators set weights.
  `ValidatorPermit` (top N by stake) gates weight submission.
  5. If `SubnetworkN < MaxAllowedUids`:
     - Append neuron at uid = current_n
     - Increment `SubnetworkN`
  6. If subnet full:
     - Find neuron with lowest `PruningScores[subnet][uid]`
     - Skip neurons in immunity period (`block - BlockAtRegistration < ImmunityPeriod`)
     - Replace the weakest neuron's UID with new hotkey
  7. Write: `Keys`, `Uids`, `IsNetworkMember`, `BlockAtRegistration`
  8. Initialize vectors: extend `Incentive`, `Rank`, `Trust`, `Dividends`,
     `Emission`, `ValidatorPermit`, `PruningScores`, `LastUpdate` with zeros
  9. Emit `NeuronRegistered { subnet_id, uid, hotkey }`

  **Validator permit logic**: top `MaxAllowedValidators` neurons by stake
  get `ValidatorPermit[subnet][uid] = true`. Recomputed each epoch.

- Whole-system effect: neurons are the participants. Without registration,
  no one can submit weights or serve strategies.
- State: neuron registry storage items.
- Wiring contract:
  - Trigger: `register_neuron` extrinsic
  - Callsite: pallet dispatchable
  - State effect: neuron assigned UID, storage vectors extended
  - Persistence effect: Keys, Uids, IsNetworkMember written
  - Observable signal: `NeuronRegistered` event, UID queryable via RPC
- Required tests:
  - `cargo test -p pallet-game-solver registration::tests::register_basic`
  - `cargo test -p pallet-game-solver registration::tests::register_burns_tokens`
  - `cargo test -p pallet-game-solver registration::tests::register_assigns_sequential_uids`
  - `cargo test -p pallet-game-solver registration::tests::register_prunes_weakest_when_full`
  - `cargo test -p pallet-game-solver registration::tests::register_respects_immunity`
  - `cargo test -p pallet-game-solver registration::tests::duplicate_registration_fails`
- Pass/fail:
  - First registration on subnet 1 → uid 0, SubnetworkN → 1
  - Second registration → uid 1, SubnetworkN → 2
  - Registration burns tokens from caller
  - On full subnet (256 neurons), new registration replaces uid with lowest pruning score
  - Neuron within immunity period (100 blocks) is never pruned
  - Duplicate hotkey on same subnet → `HotKeyAlreadyRegistered`
  - Registration on nonexistent subnet → `SubnetNotFound`
- Blocking note: without neurons, there's nothing to weight or reward.
  This AC unblocks GS-04 (weights) and GS-07 (serving).
- Rollback condition: pruning logic incorrectly removes high-scoring neurons.

---

## C. Consensus Mechanics

### AC-GS-04: Weight Submission

- Where: `crates/myosu-chain/pallets/game-solver/src/subnets/weights.rs`
- How: Port weight submission from subtensor (direct and commit-reveal).

  **`set_weights(origin, subnet_id, uids: Vec<u16>, values: Vec<u16>)`**:
  1. Ensure signed origin (hotkey)
  2. Check hotkey registered on subnet
  3. Check hotkey has `ValidatorPermit`
  4. Validate weights:
     - No duplicate UIDs
     - All UIDs exist (< SubnetworkN)
     - Length ≥ MinAllowedWeights (default 1)
     - No single weight > MaxWeightLimit (default 65535)
  5. Store `Weights[subnet][uid] = zip(uids, values)`
  6. Update `LastUpdate[subnet][uid] = current_block`
  7. Emit `WeightsSet { subnet_id, uid, weights_count }`

  **`commit_weights(origin, subnet_id, hash: H256)`**:
  1. Validate origin and registration (same as set_weights)
  2. Store hash in `WeightCommits[subnet][hotkey]` with block number
  3. Emit `WeightsCommitted { subnet_id, hotkey }`

  **`reveal_weights(origin, subnet_id, uids, values, salt)`**:
  1. Verify `blake2_256(encode(uids, values, salt)) == stored_hash`
  2. Check within reveal window: `commit_block + tempo <= current <= commit_block + 2*tempo`
  3. Validate and store weights (same as set_weights step 4-6)
  4. Remove commit from storage
  5. Emit `WeightsRevealed { subnet_id, uid }`

  Rate limiting: one weight update per `tempo/2` blocks per validator per subnet.

- Whole-system effect: weights are the input to Yuma Consensus. Without them,
  the epoch produces zero emissions.
- State: Weights, WeightCommits, LastUpdate storage.
- Wiring contract:
  - Trigger: `set_weights` or `commit_weights`/`reveal_weights` extrinsic
  - Callsite: pallet dispatchable
  - State effect: weight matrix updated for next epoch
  - Persistence effect: Weights storage map written
  - Observable signal: `WeightsSet` event, weights queryable via RPC
- Required tests:
  - `cargo test -p pallet-game-solver weights::tests::set_weights_basic`
  - `cargo test -p pallet-game-solver weights::tests::set_weights_validates_uids`
  - `cargo test -p pallet-game-solver weights::tests::set_weights_requires_validator_permit`
  - `cargo test -p pallet-game-solver weights::tests::commit_reveal_full_flow`
  - `cargo test -p pallet-game-solver weights::tests::reveal_wrong_hash_fails`
  - `cargo test -p pallet-game-solver weights::tests::reveal_outside_window_fails`
  - `cargo test -p pallet-game-solver weights::tests::rate_limit_enforced`
- Pass/fail:
  - `set_weights(1, [0,1], [100,200])` → stored, event emitted
  - Duplicate UIDs `[0,0]` → `DuplicateUids`
  - UID 999 on subnet with 10 neurons → `InvalidUid`
  - Non-validator setting weights → `NoValidatorPermit`
  - Commit hash then reveal with matching data → weights stored
  - Reveal with wrong salt → `InvalidRevealHash`
  - Reveal before tempo elapses → `RevealTooEarly`
  - Reveal after 2*tempo → `RevealTooLate`
  - Two weight sets within rate limit → second rejected
- Blocking note: weights feed Yuma Consensus. This is the bridge between
  off-chain evaluation and on-chain incentives.
- Rollback condition: weight validation rejects valid game-solving weight
  patterns that differ from AI subnet patterns.

### AC-GS-05: Yuma Consensus Port

- Where: `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`,
  `crates/myosu-chain/pallets/game-solver/src/epoch/math.rs`
- How: Port Yuma Consensus from subtensor's `epoch/run_epoch.rs` (lines 560-1591)
  and `epoch/math.rs` (1594 lines).

  **math.rs**: port these functions verbatim (they're pure math, no chain deps):
  - `inplace_normalize()`, `inplace_normalize_64()`
  - `vec_fixed64_to_fixed32()`, `fixed_to_u16()`, `u16_to_fixed()`
  - `matmul()`, `matmul_transpose()`
  - `row_hadamard()`, `inplace_col_clip()`
  - `weighted_median_col()`
  - `mat_ema()`
  - `vecdiv()`

  **epoch.rs**: port the core algorithm:
  ```rust
  pub fn run_epoch(netuid: u16) -> Vec<(u16, u64, u64)> {
      // 1. Read inputs
      let n = SubnetworkN::<T>::get(netuid);
      let weights = read_weight_matrix(netuid, n);
      let stake = read_validator_stakes(netuid, n);
      let active = compute_active_neurons(netuid, n);
      let kappa = Kappa::<T>::get(netuid);

      // 2. Yuma Consensus
      let preranks = matmul(&weights, &stake);
      let consensus = weighted_median_col(&stake, &weights, kappa);
      let mut clipped = weights.clone();
      inplace_col_clip(&mut clipped, &consensus);
      let ranks = matmul(&clipped, &stake);
      let incentive = inplace_normalize(&mut ranks.clone());
      let bonds_delta = inplace_col_normalize(row_hadamard(&clipped, &stake));
      let ema_bonds = mat_ema(&bonds_delta, &old_bonds, alpha);
      let dividends = inplace_normalize(matmul_transpose(&ema_bonds, &incentive));

      // 3. Emission split (50/50 miner/validator)
      let emission_per_uid = compute_emission(netuid, &incentive, &dividends);

      // 4. Persist
      write_incentive(netuid, &incentive);
      write_bonds(netuid, &ema_bonds);
      write_rank(netuid, &ranks);
      write_dividends(netuid, &dividends);
      write_emission(netuid, &emission_per_uid);
      update_pruning_scores(netuid, &ranks);
      update_validator_permits(netuid, &stake);

      emission_per_uid
  }
  ```

  Use `substrate_fixed::types::I32F32` and `I64F64` for fixed-point arithmetic.
  **Pin `substrate_fixed` to the exact version from subtensor's Cargo.lock** to
  ensure bit-exact results.

  **Empty-weights guard**: if the weight matrix is entirely zero (no validator
  submitted weights since last epoch), skip consensus computation and emit
  zero for all UIDs. This prevents division-by-zero in normalization functions.

  **Verification**: generate concrete test vectors by running subtensor's
  `epoch_mechanism()` on synthetic inputs. Ship as JSON fixtures:
  - Scenario 1: 2 equal-stake validators, 3 miners, uniform weights
  - Scenario 2: 2 unequal-stake validators, 3 miners, divergent weights (tests clipping)
  - Scenario 3: 1 validator, 1 miner (degenerate)
  - Scenario 4: all validators identical weights (consensus = weights)
  - Scenario 5: multi-epoch bond EMA accumulation (3 consecutive epochs)
  Each fixture captures: inputs (weights, stakes, kappa, bonds) and all
  intermediate + final values (preranks, consensus, clipped, ranks, incentive,
  dividends, bonds_delta, ema_bonds, emission per UID).

- Whole-system effect: Yuma Consensus is the incentive engine. Without it,
  weights have no effect and emissions don't flow.
- State: reads Weights, Stake, active flags; writes Incentive, Bonds, Rank,
  Dividends, Emission, PruningScores, ValidatorPermit.
- Wiring contract:
  - Trigger: `on_initialize` when `block_number % tempo == 0`
  - Callsite: `lib.rs::Hooks::on_initialize()`
  - State effect: consensus scores computed, bonds updated, emissions allocated
  - Persistence effect: 7 storage maps updated per epoch
  - Observable signal: `EpochCompleted { subnet_id, block }` event
- Required tests:
  - `cargo test -p pallet-game-solver epoch::tests::yuma_basic_two_validators`
  - `cargo test -p pallet-game-solver epoch::tests::yuma_clips_above_consensus`
  - `cargo test -p pallet-game-solver epoch::tests::yuma_bond_ema_accumulates`
  - `cargo test -p pallet-game-solver epoch::tests::yuma_matches_subtensor_output`
  - `cargo test -p pallet-game-solver epoch::tests::emission_split_50_50`
  - `cargo test -p pallet-game-solver math::tests::matmul_correctness`
  - `cargo test -p pallet-game-solver math::tests::weighted_median_correctness`
  - `cargo test -p pallet-game-solver math::tests::fixed_point_round_trip`
- Pass/fail:
  - 2 validators, 3 miners, identical weights → equal emissions
  - 2 validators, 3 miners, one validator overscores miner A → A's weight clipped
  - Bonds accumulate over 5 epochs for consistent validator
  - **Critical**: `yuma_matches_subtensor_output` — synthetic input produces
    bit-identical output compared to running subtensor's epoch function
  - 50% of emission goes to miners (incentive), 50% to validators (dividends)
  - Zero-weight neurons receive zero emission
  - Epoch with no weights submitted → all emissions zero
- Blocking note: this is the core algorithm. Getting it wrong means the
  incentive mechanism is broken. Port, don't reinvent.
- Rollback condition: fixed-point math diverges from subtensor due to
  dependency version mismatch (substrate_fixed).

### AC-GS-06: Emission Distribution

- Where: `crates/myosu-chain/pallets/game-solver/src/coinbase/`
- How: Implement simplified emission (no Alpha token, no AMM, no halving).

  **Block emission**: fixed amount per block (configurable via Sudo).
  Default: 1,000,000,000 rao (1 MYOSU) per block.

  **Per-subnet share**: equal split across active subnets (subnets with ≥1
  neuron that submitted weights in the last epoch). Future spec will add
  activity-weighted allocation.

  **Per-neuron distribution** (within a subnet):
  - Miner share (50%): proportional to `Incentive[subnet][uid]`
  - Validator share (50%): proportional to `Dividends[subnet][uid]`
  - Owner share: 0% for bootstrap (configurable via Sudo)

  **Minting**: use `T::Currency::deposit_creating()` to mint new tokens
  into neuron accounts. Track `TotalIssuance`.

  Called by `on_initialize` after Yuma Consensus completes for each subnet.

- Whole-system effect: emission is what makes the chain valuable to miners.
  Without it, there's no economic incentive to solve games.
- State: reads Incentive, Dividends from epoch; writes balances via Currency.
- Wiring contract:
  - Trigger: `on_initialize` after `run_epoch` completes
  - Callsite: `lib.rs::Hooks::on_initialize()`
  - State effect: token balances increased for reward recipients
  - Persistence effect: balance updates + TotalIssuance increment
  - Observable signal: `EmissionDistributed { subnet_id, total }` event
- Required tests:
  - `cargo test -p pallet-game-solver emission::tests::equal_subnet_split`
  - `cargo test -p pallet-game-solver emission::tests::miner_validator_50_50`
  - `cargo test -p pallet-game-solver emission::tests::proportional_to_scores`
  - `cargo test -p pallet-game-solver emission::tests::no_emission_without_weights`
  - `cargo test -p pallet-game-solver emission::tests::total_issuance_tracks`
- Pass/fail:
  - 2 active subnets → each gets 50% of block emission
  - Within subnet, miner with 2x incentive gets 2x miner emission
  - Validator with 3x dividends gets 3x validator emission
  - Subnet with no weight submissions in last epoch → 0 emission
  - TotalIssuance increases by exactly BlockEmission per block
  - No tokens created from thin air; sum of all distributions == block emission
- Blocking note: emission is the revenue model. Miners won't run solvers
  without economic reward.
- Rollback condition: emission accounting error creates or destroys tokens.

---

## D. Network Participation

### AC-GS-07: Axon Serving

- Where: `crates/myosu-chain/pallets/game-solver/src/serving.rs (new)`
- How: Port `serve_axon` from subtensor to register miner endpoints.

  **`serve_axon(origin, subnet_id, ip, port, ip_type, protocol, version)`**:
  1. Ensure signed origin (hotkey)
  2. Check hotkey registered on subnet
  3. Validate IP (non-zero, valid type 4 or 6)
  4. Validate port (non-zero)
  5. Store `Axons[subnet][hotkey] = AxonInfo { ip, port, ip_type, protocol, version }`
  6. Emit `AxonServed { subnet_id, hotkey, ip, port }`

  Rate limited: one update per 50 blocks per hotkey per subnet.

- Whole-system effect: validators need to know where miners are to query them.
  Without axon registration, the off-chain evaluation loop can't work.
- State: Axons storage map.
- Wiring contract:
  - Trigger: `serve_axon` extrinsic from miner
  - Callsite: pallet dispatchable
  - State effect: AxonInfo stored for hotkey on subnet
  - Persistence effect: Axons storage map updated
  - Observable signal: `AxonServed` event, axon queryable via RPC
- Required tests:
  - `cargo test -p pallet-game-solver serving::tests::serve_axon_basic`
  - `cargo test -p pallet-game-solver serving::tests::serve_axon_requires_registration`
  - `cargo test -p pallet-game-solver serving::tests::serve_axon_validates_ip`
  - `cargo test -p pallet-game-solver serving::tests::serve_axon_rate_limited`
- Pass/fail:
  - Registered neuron serves axon → stored and queryable
  - Unregistered hotkey → `HotKeyNotRegistered`
  - IP 0 → `InvalidIp`
  - Port 0 → `InvalidPort`
  - Two serve_axon calls within 50 blocks → second rate-limited
  - Query `Axons[1][hotkey]` via RPC → returns AxonInfo
- Blocking note: without axon discovery, validators can't find miners.
- Rollback condition: AxonInfo encoding is incompatible with miner's endpoint format.

### AC-GS-08: Basic Staking

- Where: `crates/myosu-chain/pallets/game-solver/src/staking.rs (new)`
- How: Simplified staking (no Alpha token, no AMM, direct native token).

  **`add_stake(origin, hotkey, subnet_id, amount)`**:
  1. Ensure signed origin (staker)
  2. Check hotkey registered on subnet
  3. Transfer `amount` from staker to reserved (or burn)
  4. Increment `Stake[hotkey][subnet] += amount`
  5. Increment `TotalStake += amount`
  6. Emit `StakeAdded { staker, hotkey, subnet_id, amount }`

  **`remove_stake(origin, hotkey, subnet_id, amount)`**:
  1. Ensure signed origin (staker, must match original staker or hotkey owner)
  2. Check `Stake[hotkey][subnet] >= amount`
  3. Unreserve/mint `amount` back to staker
  4. Decrement `Stake[hotkey][subnet] -= amount`
  5. Decrement `TotalStake -= amount`
  6. Emit `StakeRemoved { staker, hotkey, subnet_id, amount }`

  Stake determines validator voting power in Yuma Consensus. Validators with
  more stake have proportionally more influence on miner scores.

  Simplified from subtensor: no Alpha/TAO conversion, no delegation take, no
  childkeys, no auto-claim, no moving stake between hotkeys.

- Whole-system effect: staking determines who is a validator (top N by stake
  get ValidatorPermit) and how much their weights count in Yuma.
- State: Stake, TotalStake storage.
- Wiring contract:
  - Trigger: `add_stake` / `remove_stake` extrinsic
  - Callsite: pallet dispatchable
  - State effect: Stake balance updated, validator permit may change
  - Persistence effect: Stake storage map + balance reserve
  - Observable signal: `StakeAdded`/`StakeRemoved` events
- Required tests:
  - `cargo test -p pallet-game-solver staking::tests::add_stake_basic`
  - `cargo test -p pallet-game-solver staking::tests::remove_stake_basic`
  - `cargo test -p pallet-game-solver staking::tests::stake_determines_validator_power`
  - `cargo test -p pallet-game-solver staking::tests::insufficient_balance_fails`
  - `cargo test -p pallet-game-solver staking::tests::remove_more_than_staked_fails`
- Pass/fail:
  - `add_stake(alice_hotkey, 1, 1000)` → Stake[alice][1] == 1000
  - Staker's free balance decreases by 1000
  - `remove_stake(alice_hotkey, 1, 500)` → Stake[alice][1] == 500
  - Staker's free balance increases by 500
  - `add_stake` with amount > free balance → `InsufficientBalance`
  - `remove_stake` with amount > staked → `InsufficientStake`
  - Highest-staked neurons get ValidatorPermit after epoch
- Blocking note: without staking, all validators have equal power. Staking
  creates skin-in-the-game alignment.
- Rollback condition: stake accounting error creates or loses tokens.

---

## E. Runtime Integration

### AC-GS-09: Add Pallet to Runtime at Index 7

- Where: `crates/myosu-chain/runtime/src/lib.rs (extend)`
- How: Add `pallet_game_solver` to the runtime:

  1. Add dependency to `runtime/Cargo.toml`
  2. Implement `pallet_game_solver::Config for Runtime`:
     ```rust
     impl pallet_game_solver::Config for Runtime {
         type RuntimeEvent = RuntimeEvent;
         type Currency = Balances;
         type InitialTempo = ConstU16<100>;
         type InitialBurnCost = ConstU64<1_000_000_000>;
         type MaxSubnets = ConstU16<32>;
         type MaxNeuronsPerSubnet = ConstU16<256>;
     }
     ```
  3. Add to `construct_runtime!` at index 7:
     ```rust
     GameSolver: pallet_game_solver = 7,
     ```
  4. Update chain spec genesis to initialize GameSolver storage if needed
  5. Verify the full runtime compiles and produces blocks with the new pallet

- Whole-system effect: integrates the pallet into the live chain. Without
  this, the pallet exists as a standalone crate but doesn't run on-chain.
- State: runtime includes pallet in block execution.
- Wiring contract:
  - Trigger: runtime compilation
  - Callsite: `crates/myosu-chain/runtime/src/lib.rs`
  - State effect: pallet's `on_initialize` called every block
  - Persistence effect: pallet storage available in chain state
  - Observable signal: `create_subnet` extrinsic callable via RPC
- Required tests:
  - `cargo test -p myosu-runtime runtime::tests::game_solver_pallet_present`
  - `cargo test -p myosu-runtime runtime::tests::create_subnet_via_runtime`
  - Integration: `cargo test -p myosu-chain integration::tests::full_incentive_loop`
- Pass/fail:
  - `cargo build -p myosu-runtime` succeeds with pallet included
  - Pallet index 7 is occupied by GameSolver
  - `create_subnet` extrinsic callable and processes correctly
  - `register_neuron` → `set_weights` → epoch fires → emission distributed
  - Full incentive loop works end-to-end on devnet
  - No regression in CF-05 smoke test (balance transfer still works)
- Blocking note: this is the integration gate. Until the pallet is in the
  runtime, it's just library code.
- Rollback condition: pallet's Config requirements conflict with existing
  runtime configuration, or pallet panics during block execution.

### AC-GS-10: Runtime API for State Queries

- Where: `crates/myosu-chain/pallets/game-solver/src/rpc.rs (new)`,
  `crates/myosu-chain/runtime/src/lib.rs (extend)`
- How: Implement a `GameSolverRuntimeApi` so miners and validators can
  efficiently query subnet state without iterating storage keys:

  ```rust
  sp_api::decl_runtime_apis! {
      pub trait GameSolverApi {
          fn subnet_info(netuid: u16) -> Option<SubnetInfo>;
          fn neuron_info(netuid: u16, uid: u16) -> Option<NeuronInfo>;
          fn all_axons(netuid: u16) -> Vec<(u16, AxonInfo)>;
          fn all_incentives(netuid: u16) -> Vec<(u16, u16)>;  // uid → incentive
          fn subnet_count() -> u16;
      }
  }
  ```

  Without this, the validator's miner discovery (VO-02) must iterate storage
  keys one by one, which is slow and error-prone.

- Whole-system effect: enables efficient off-chain queries for miners, validators,
  and gameplay.
- State: no new state — reads existing storage.
- Wiring contract:
  - Trigger: RPC call from miner/validator/player client
  - Callsite: runtime API implementation
  - State effect: N/A (read-only)
  - Persistence effect: N/A
  - Observable signal: `all_axons(1)` returns list of miner endpoints
- Required tests:
  - `cargo test -p myosu-runtime runtime::tests::runtime_api_subnet_info`
  - `cargo test -p myosu-runtime runtime::tests::runtime_api_all_axons`
- Pass/fail:
  - `subnet_info(1)` returns SubnetInfo with game_type, tempo, neuron count
  - `all_axons(1)` returns all registered axon endpoints
  - `all_incentives(1)` returns per-UID scores after epoch
  - Nonexistent subnet → None
- Blocking note: without efficient queries, the off-chain participants
  (MN, VO, GP) have no practical way to discover chain state.
- Rollback condition: runtime API types incompatible with subxt codegen.

---

## Operational Controls

Phase order:
1. GS-01 (scaffold) — pallet compiles, storage defined
2. GS-02 (subnets) — can create/dissolve subnets
3. GS-03 (neurons) — can register/prune neurons
4. GS-04 (weights) — can submit/commit/reveal weights
5. GS-05 (Yuma) — consensus runs and produces scores
6. GS-06 (emission) — tokens distributed proportionally
7. GS-07 (serving) — miners advertise endpoints
8. GS-08 (staking) — stake determines voting power
9. GS-09 (integration) — pallet runs in the live runtime

Gate rules:
- GS-01 must compile before any other GS-* AC
- GS-02 must work before GS-03 (neurons register on subnets)
- GS-03 must work before GS-04 (only registered neurons set weights)
- GS-04 must work before GS-05 (Yuma reads weights)
- GS-05 must work before GS-06 (emission uses Yuma output)
- GS-09 can only proceed after GS-01..08 all pass independently

Failure modes:
| Codepath | Realistic failure | Test needed | Error handling needed | User-visible if broken |
|----------|-------------------|-------------|-----------------------|------------------------|
| Subnet creation | Insufficient burn balance | Yes | Yes | Clear error |
| Neuron registration | Subnet full, no prunable neuron | Yes | Yes | Clear error |
| Weight validation | Invalid UIDs in weight vector | Yes | Yes | Transaction reverted |
| Commit-reveal | Hash mismatch on reveal | Yes | Yes | Clear error |
| Yuma Consensus | Empty weight matrix | Yes | Yes | Zero emission (safe) |
| Emission minting | Overflow on large emission | Yes | Yes | Saturating math |
| Staking | Double-spend on stake/unstake | Yes | Yes | Token accounting invariant |

## Decision Log

- 2026-03-16: No Alpha token/AMM for bootstrap — adds massive complexity
  (subtensor's swap pallet is 24KB alone). Native token staking is sufficient
  to prove the incentive loop works.
- 2026-03-16: No root network — unnecessary without multi-subnet emission
  allocation. Equal split across active subnets for bootstrap.
- 2026-03-16: Fixed burn cost for subnet/neuron registration — dynamic
  difficulty adds complexity without value during bootstrap.
- 2026-03-16: 50/50 miner/validator emission split — matches subtensor's
  default. Configurable via Sudo for tuning.
- 2026-03-16: Port Yuma math verbatim including substrate_fixed types —
  bit-exact reproduction is required for INV-003 verification determinism.
- 2026-03-16: Keep commit-reveal — essential for game solving to prevent
  miners from memorizing validator test positions.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | `create_subnet(b"nlhe_hu", 100)` succeeds on devnet | Subnet registry | GS-01, GS-02 |
| 2 | Register 3 miners and 2 validators on subnet 1 | Neuron registration | GS-03 |
| 3 | Validators set weights on miners | Weight submission | GS-04 |
| 4 | After 100 blocks, Yuma runs and scores match expected values | Consensus | GS-05 |
| 5 | Miners receive emission proportional to scores | Emission | GS-06 |
| 6 | Miners serve axons, validators discover them | Serving | GS-07 |
| 7 | Validator stakes more → gets more voting power in next epoch | Staking | GS-08 |
| 8 | Full loop on devnet: create subnet → register → stake → weights → epoch → emission | Integration | GS-09 |
| 9 | Yuma output matches subtensor for identical synthetic inputs | INV-003 | GS-05 |
