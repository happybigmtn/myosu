# Specification: Chain Runtime

Source: Reverse-engineered from crates/myosu-chain/runtime (lib.rs)
Status: Draft
Depends-on: 010-game-solver-pallet

## Purpose

The chain runtime composes Substrate FRAME pallets into the stage-0 blockchain
execution environment. It defines the state transition function that all nodes
execute identically, configuring block production parameters, token economics,
transaction fees, and pallet integration. The runtime is the boundary between
application-specific logic (pallets) and the generic Substrate node
infrastructure.

The primary consumer is the chain node binary, which executes the runtime's
state transition function for every block.

## Whole-System Goal

Current state: The runtime is implemented at spec version 385 with
pallet-game-solver at index 7, a stage-0 NoopSwap (1:1 identity), dual
Aura/BABE consensus support, Grandpa finality, and SafeMode for emergency
control.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: The chain produces blocks with correct pallet composition,
token economics, and consensus configuration for stage-0 operation.

Still not solved here: Individual pallet behaviors, node networking, multi-node
consensus, and real AMM swap pricing are separate concerns.

## Scope

In scope:
- Pallet composition and index assignment
- Block production parameters (6-second block time, 4-hour BABE epoch)
- Token economics (21 quadrillion RAO total supply, 0.0 to 21M TAO)
- Transaction fee configuration
- Stage-0 NoopSwap integration (1:1 identity pricing)
- SafeMode with whitelisted pallets (Sudo, Multisig, System, Timestamp,
  weight-setting operations)
- Consensus configuration (Aura and BABE with Grandpa finality)
- Maximum block weight (4 seconds of compute)
- Inherited pallets: admin-utils, utility, proxy, registry, crowdloan, drand,
  swap, swap-interface, transaction-fee

Out of scope:
- Game-solver pallet internal behavior (separate spec)
- Node networking and peer discovery
- Chain spec generation
- EVM or Frontier integration (preserved but not active in stage-0)

## Current State

The runtime exists at crates/myosu-chain/runtime with spec version 385. It
composes 11 pallets: the active pallet-game-solver at runtime index 7, plus
inherited pallets from the Bittensor fork (admin-utils, utility, proxy,
registry, crowdloan, drand, swap, swap-interface, transaction-fee) and standard
Substrate system pallets (System, Timestamp, Balances, TransactionPayment).

The stage-0 NoopSwap implements the swap interface trait with identity pricing:
TAO and Alpha exchange 1:1 with zero fees. This allows the full
registration/staking/emission pipeline to function without real AMM complexity.
The NoopSwap is explicitly a temporary stub.

Token economics use RAO as the smallest unit with 21 quadrillion total supply
(21,000,000 TAO at 10^9 RAO per TAO).

Block time is 6 seconds. BABE epochs are 4 hours (2400 blocks). Maximum block
weight is 4 seconds of compute (two-thirds of block time).

SafeMode is enabled with a whitelist that allows Sudo, Multisig, System,
Timestamp, and specific weight-setting extrinsics to execute even during
emergency freeze conditions.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Pallet composition | 11 pallets at assigned indices | Reuse | Stage-0 adequate |
| NoopSwap | 1:1 identity with zero fees | Replace | Must be replaced for stage-1 |
| Consensus | Aura/BABE + Grandpa | Reuse | Standard Substrate consensus |
| Token economics | 21M TAO supply, 10^9 RAO/TAO | Reuse | Fixed supply model |
| SafeMode | Whitelist for emergency operations | Reuse | Safety mechanism |
| Block params | 6s block time, 4h epochs, 4s max weight | Reuse | Standard configuration |

## Non-goals

- Implementing custom consensus algorithms beyond Aura/BABE/Grandpa.
- Activating EVM or smart contract execution in stage-0.
- Providing runtime upgrade governance.
- Defining multi-chain relay interactions.
- Replacing inherited pallets that are unused but harmless in stage-0.

## Behaviors

The runtime produces blocks every 6 seconds. Each block executes the
initialize_block hook, applies all included extrinsics through their respective
pallets, and runs the on_finalize hooks. Maximum block weight caps computation
at 4 seconds to ensure blocks complete within the slot.

Pallet-game-solver at index 7 processes all game-solving extrinsics:
registration, staking, weight submission, and emission distribution. Other
pallets provide supporting functionality: admin-utils for parameter overrides,
utility for batch calls (nested batching disabled), proxy for account
delegation, and transaction-fee for delegation-aware fee calculation.

The NoopSwap intercepts all swap operations and returns identity results. When a
swap_tao_for_alpha call arrives, it returns the input amount as output. When
queried for alpha price, it returns 1.0. This bypass allows the full economic
pipeline to function in a single-token model without AMM complexity.

SafeMode, when activated, rejects all extrinsics except those from whitelisted
origins and pallets. This provides an emergency brake for the network operator.

Transaction fees are computed using Substrate's standard fee model with
delegation-aware adjustments from the transaction-fee pallet.

The runtime API exposes standard Substrate interfaces (Core, Metadata,
BlockBuilder, TaggedTransactionQueue) plus custom neuron info RPC endpoints
used by the chain client.

## Acceptance Criteria

- The runtime produces blocks at 6-second intervals under normal operation.
- Pallet-game-solver extrinsics execute correctly at runtime index 7.
- The NoopSwap returns identity pricing for all swap operations.
- SafeMode blocks non-whitelisted extrinsics when activated.
- Transaction fees are computed correctly for all extrinsic types.
- Block weight does not exceed the 4-second maximum.
- The runtime reports spec version 385.
- All standard Substrate runtime APIs are functional.
