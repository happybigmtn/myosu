# Specification: Game Solver Pallet

Source: Reverse-engineered from crates/myosu-chain/pallets/game-solver (lib.rs, coinbase, epoch, staking, subnets, swap, guards, rpc_info, migrations)
Status: Draft
Depends-on: none

## Purpose

The game-solver pallet provides on-chain coordination for the Myosu
game-solving economy. It manages subnet lifecycle, neuron registration, staking,
validator weight submission, and emission distribution. This is the stage-0
core pallet (runtime index 7) that replaced and narrowed the inherited
pallet-subtensor from the Bittensor fork. All economic activity in the system
flows through this pallet.

The primary consumers are miners (registration, axon serving), validators
(weight submission, permit acquisition), and the chain runtime (emission
distribution, epoch processing).

## Whole-System Goal

Current state: The pallet is implemented with storage for up to 128 subnets,
hotkey/coldkey account hierarchy, staking, emission at 1 billion RAO per block,
Yuma Consensus weight processing, commit-reveal weight submission, and a
stage-0 identity swap interface (1:1 TAO to Alpha with zero fees).

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: The chain coordinates neuron registration, quality-weighted
emission distribution, and economic incentives for game solvers through a
single pallet with configurable per-subnet parameters.

Still not solved here: Off-chain strategy computation (miners), off-chain
quality scoring (validators), gameplay surfaces, and real AMM swap pricing are
separate concerns.

## Scope

In scope:
- Subnet lifecycle: creation, registration limits, subnet limit (max 128)
- Neuron registration: hotkey/coldkey model, burned registration
- Staking: per-subnet alpha staking, delegation with configurable take rates,
  childkey take
- Emission: 1 billion RAO per block, per-subnet distribution via Yuma Consensus
- Weight submission: direct set_weights and commit-reveal (hash-based, v2)
- Validator management: permits, pruning, immunity periods
- Stage-0 swap interface: identity pricing (1:1), zero fees
- Per-subnet hyperparameters: tempo, kappa, rho, alpha, immunity period,
  activity cutoff, weights version key, weights set rate limit, bonds moving
  average
- Rate limiting: per-transaction type, configurable per hyperparameter
- Axon info storage for miner endpoint advertisement
- Root claim handling with per-coldkey subnet selection

Out of scope:
- Real AMM swap pricing (stage-0 uses identity stub)
- EVM integration or smart contract execution
- Cross-chain bridging or relay chain interaction
- Off-chain worker logic
- Governance or council mechanics

## Current State

The pallet exists at crates/myosu-chain/pallets/game-solver with approximately
2,853 lines in the main lib.rs plus extensive submodules (coinbase, epoch,
staking, subnets, swap, guards, rpc_info, migrations, extensions). It is a
narrowed fork of pallet-subtensor from the Bittensor project.

Storage uses FRAME's StorageValue, StorageMap, and StorageDoubleMap with
Identity hashers for small keys (NetUid) and Blake2_128Concat for account
hashing. Key storage items include: Owner (hotkey to coldkey mapping),
Delegates (delegation take rates), TotalStake, BlockEmission (1 billion RAO),
SubnetTAO/SubnetAlphaIn/SubnetAlphaOut (per-subnet token pools),
AlphaDividendsPerSubnet (per-hotkey per-subnet dividends),
ValidatorPermit (per-UID boolean vector), CommitRevealWeightsEnabled (per-subnet
toggle, default true), SubtokenEnabled (per-subnet toggle), and per-subnet
hyperparameter maps.

The stage-0 swap interface provides three methods: stage0_swap_tao_for_alpha,
stage0_sim_swap_alpha_for_tao, and stage0_current_alpha_price. All currently
implement identity pricing (1:1 ratio) with zero fees. This is an explicit stub
that must be replaced before public network launch.

Commit-reveal v2 uses hash-based commitment: validators commit a hash of their
weights, then reveal in a subsequent epoch. CRV3 (timelock-based) is not
available because it depends on pallet_drand which has been stripped.

Maximum commits per block is configurable via rate limiting. Per-subnet tempo
controls epoch frequency. Weights version key provides version gating for
weight submission compatibility.

Root claim handling supports a RootClaimTypeEnum with variants for swapping root
emission to TAO or keeping as alpha, with per-coldkey KeepSubnets selection.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Subnet management | SubnetLimit (128), registration, lifecycle | Reuse | Stage-0 adequate |
| Neuron registration | burned_register with hotkey/coldkey model | Reuse | Working registration flow |
| Staking | Per-subnet alpha with delegation and childkey take | Reuse | Economic incentive structure |
| Emission | 1B RAO/block via coinbase module | Reuse | Stage-0 emission rate |
| Weight submission | Direct and commit-reveal v2 | Reuse | Dual-mode proven |
| Swap interface | Stage0SwapInterface with 1:1 identity pricing | Replace | Must be replaced with real AMM for stage-1 |
| Hyperparameters | Per-subnet configurable via storage maps | Reuse | Flexible configuration |

## Non-goals

- Implementing a real automated market maker for token swaps.
- Providing EVM execution or smart contract deployment.
- Managing cross-chain or relay chain interactions.
- Implementing governance voting or council mechanics.
- Providing off-chain computation or oracle services.

## Behaviors

Subnet creation registers a new network with a unique NetUid up to the
SubnetLimit of 128. Each subnet has independent hyperparameters, emission pools,
and neuron sets.

Neuron registration via burned_register associates a hotkey with a coldkey on a
specific subnet. The hotkey receives a UID within the subnet. Registration rate
is controlled by TargetRegistrationsPerInterval.

Staking allows coldkeys to stake alpha on specific subnets behind hotkeys.
Delegation allows hotkeys to accept stake from other coldkeys with a configurable
take rate. Childkey take provides per-subnet commission control.

Emission distributes 1 billion RAO per block across subnets. Within each subnet,
Yuma Consensus processes the validator weight matrix to compute per-neuron
incentive, dividend, and emission values. Validators with higher-quality weight
vectors (as determined by consensus agreement) receive larger dividends.

Weight submission operates in two modes. When CommitRevealWeightsEnabled is
false for a subnet, validators call set_weights directly with destination UIDs,
weight values, and a version key. When enabled, validators first commit a hash
of their weights, then reveal in a subsequent epoch. Rate limiting controls how
frequently a validator can submit weights.

The stage-0 swap interface translates between TAO (the root token) and Alpha
(subnet-specific tokens) at a 1:1 ratio with zero fees. This stub allows the
registration, staking, and emission flows to function without a real AMM.

Axon info storage records miner endpoints (IP, port, version, protocol) on-chain
so that validators and the gameplay surface can discover them.

Root claims allow coldkeys to control how their root-level emission is
distributed: swapped to TAO or kept as alpha in selected subnets.

Validator permits are granted based on staking position within a subnet. A
boolean vector per subnet indicates which UIDs hold validator permits. Only
neurons with permits can submit weights.

## Acceptance Criteria

- Up to 128 subnets can be created with unique NetUids.
- Neuron registration associates a hotkey with a coldkey and assigns a UID.
- Staking increases the hotkey's alpha balance on the specified subnet.
- Emission distributes tokens proportional to validator-weighted quality scores.
- Direct weight submission succeeds when commit-reveal is disabled.
- Commit-reveal weight submission succeeds through the commit then reveal cycle.
- The stage-0 swap interface returns 1:1 pricing with zero fees.
- Per-subnet hyperparameters are independently configurable.
- Axon info records miner endpoints that can be queried by validators and
  gameplay.
- Validator permits are correctly assigned based on staking position.
- Rate limiting prevents weight submission more frequently than the configured
  interval.
