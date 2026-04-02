# Specification: Chain Runtime Reduction

Source: Genesis Plan 003 (Chain Runtime Reduction), ASSESSMENT.md (TD-01, TD-02, TD-03)
Status: Draft
Depends-on: none

## Purpose

The Substrate runtime carries 242K lines of inherited Bittensor subtensor fork
code across 11+ pallets, EVM/Frontier integration, dual-token AMM mechanics, and
DRAND VRF — none of which are required for stage-0 game-solving coordination.
This excess surface area increases compile times, expands the attack surface,
complicates audits, and obscures the actual stage-0 behavior. Reducing the
runtime to the minimal pallet set required for game-solving coordination makes
the system auditable, faster to build, and honest about what stage-0 actually
needs.

## Whole-System Goal

Current state: The runtime wires the full subtensor pallet surface with 69
extrinsics, 95+ error variants, 60+ events, and 50+ storage migrations.
Ten pallets beyond the core set are compiled and included. Frontier/EVM
service code is linked into the node binary. 122 TODO/FIXME comments exist
across the chain crate.

This spec adds: A runtime that includes only the pallets required for stage-0
operation, a pallet-game-solver surface reduced to the extrinsics actually
exercised by miners, validators, and operators, and removal of Frontier/EVM
from the node service layer.

If all ACs land: The chain compiles with fewer than 8 pallets, the
pallet-game-solver exposes 20 or fewer extrinsics, the node binary no longer
links Frontier, and outstanding TODO/FIXME comments are triaged below 20.

Still not solved here: Multi-node networking, emission accounting correctness,
upstream Substrate CVE tracking, and runtime upgrade governance.

## Scope

In scope:
- Removing or feature-gating pallets not required for stage-0
- Reducing pallet-game-solver extrinsic surface to stage-0 minimum
- Stripping Frontier/EVM service from the node binary
- Triaging TODO/FIXME comments in chain crate code

Out of scope:
- Rewriting Yuma Consensus or emission logic (see 002-single-token-emission-accounting)
- Multi-node devnet configuration (see 006-multi-node-devnet)
- Security audit of remaining pallet surface (see 004-security-audit-process)
- Upstream Substrate SDK version bumps

## Current State

The runtime at `crates/myosu-chain/runtime/src/lib.rs` wires the full subtensor
surface. A SwapInterface no-op stub exists at `crates/myosu-chain/pallets/swap/`
to satisfy registration, staking, and emission call sites. CRV3 timelock
dependencies on pallet_drand have been stripped previously. The node service at
`crates/myosu-chain/node/src/service.rs` still links Frontier/EVM.

The pallet-game-solver at `crates/myosu-chain/pallets/game-solver/` is a renamed
pallet-subtensor carrying the full extrinsic surface. Stage-0 flow tests pass
against a subset of this surface.

122 TODO/FIXME comments are distributed across the chain crate, many in
migration code and benchmark weight placeholders.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| SwapInterface stub | `crates/myosu-chain/pallets/swap/` | Reuse | Already satisfies call sites with no-op |
| CRV3 stripping | pallet-drand simplified | Reuse | Timelock dependency already removed |
| Stage-0 flow tests | `pallet-game-solver -- stage_0` | Reuse | Validates reduced surface still works |
| Runtime pallet wiring | `crates/myosu-chain/runtime/src/lib.rs` | Replace | Currently wires full subtensor surface |
| Node service | `crates/myosu-chain/node/src/service.rs` | Replace | Currently links Frontier |
| Safe-math primitives | `crates/myosu-chain/primitives/safe-math/` | Reuse | Required by Yuma Consensus in pallet-game-solver |
| Share-pool primitives | `crates/myosu-chain/primitives/share-pool/` | Reuse | Required by pallet-game-solver |

## Non-goals

- Rewriting the pallet-game-solver internals beyond surface reduction.
- Removing pallets from the source tree entirely — feature-gating behind a
  `full-runtime` flag is sufficient for stage-0.
- Upgrading the Polkadot SDK or Substrate dependency pins.
- Achieving a specific binary size target.
- Refactoring the storage migration ordering beyond what removal requires.

## Behaviors

The runtime constructs with only the stage-0 pallet set: System, Timestamp,
Balances, TransactionPayment, GameSolver (index 7), AdminUtils, and Utility.
All other pallets are behind a cargo feature flag and are not compiled by
default.

Pallet-game-solver exposes only the extrinsics exercised by the stage-0 loop:
subnet registration, neuron registration, weight submission (commit-reveal v2),
staking, and administrative configuration. Removed extrinsics return a dispatch
error indicating unavailability rather than silently disappearing.

The node binary builds and starts a local devnet without Frontier/EVM service
code. The fast-runtime feature flag continues to work for accelerated block
production in development and testing.

Existing stage-0 flow tests continue to pass against the reduced surface.
TODO/FIXME comments in the chain crate are resolved, deferred with
justification, or removed, leaving fewer than 20 remaining.

## Acceptance Criteria

- The runtime compiles with 7 or fewer pallets in the default feature set.
- Pallet-game-solver exposes 20 or fewer extrinsics.
- Stage-0 flow tests pass on the reduced pallet surface.
- The node binary builds without Frontier/EVM service dependencies in the
  default feature set.
- Fewer than 20 TODO/FIXME comments remain in the chain crate, each with a
  justification comment.
- The `substrate_fixed` pin to encointer fork v0.6.0 is preserved (required for
  Yuma Consensus determinism).
