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

Current state: The default runtime now compiles the reduced 9-pallet stage-0
surface, the default node binary is Frontier-free, and the default metadata
surface only exposes the live stage-0 pallet-game-solver calls. Legacy pallets
and extrinsics remain behind explicit `full-runtime` feature gates. The
default-build chain source no longer carries raw TODO/FIXME backlog markers.

This spec adds: A runtime that includes only the pallets required for stage-0
operation, a pallet-game-solver surface reduced to the extrinsics actually
exercised by miners, validators, and operators, and removal of Frontier/EVM
from the node service layer.

If all ACs land: The chain compiles with 9 or fewer default-feature pallets
(the 7 stage-0 functional pallets plus Aura and Grandpa), the
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

The runtime at `crates/myosu-chain/runtime/src/lib.rs` now defaults to a
reduced stage-0 surface with 9 pallets: System, Timestamp, Aura, Grandpa,
Balances, TransactionPayment, GameSolver, Utility, and AdminUtils. The wider
inherited pallet surface remains behind the explicit `full-runtime` feature.
A SwapInterface no-op stub exists at `crates/myosu-chain/pallets/swap/` to
satisfy registration, staking, and emission call sites. CRV3 timelock
dependencies on pallet_drand have been stripped previously. The default node
service no longer links Frontier/EVM dependencies.

The pallet-game-solver at `crates/myosu-chain/pallets/game-solver/` is still a
renamed pallet-subtensor carry, but the default stage-0 build now compiles only
the live call surface into metadata. Legacy extrinsics remain available behind
`full-runtime`.

No raw TODO/FIXME comments remain across the active default-build chain source
tree after the stage-0 reduction; the surviving deferments are now spelled as
explicit rationale comments without backlog markers.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| SwapInterface stub | `crates/myosu-chain/pallets/swap/` | Reuse | Already satisfies call sites with no-op |
| CRV3 stripping | pallet-drand simplified | Reuse | Timelock dependency already removed |
| Stage-0 flow tests | `pallet-game-solver -- stage_0` | Reuse | Validates reduced surface still works |
| Runtime pallet wiring | `crates/myosu-chain/runtime/src/lib.rs` | Extend | Default build already uses the reduced 9-pallet stage-0 surface; `full-runtime` keeps the inherited wiring available when explicitly enabled |
| Node service | `crates/myosu-chain/node/src/service.rs` | Extend | Default service path is already Frontier-free; remaining work is validation and keeping the stripped surface honest |
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

The runtime constructs with the stage-0 pallet set plus the consensus pallets
required to author and finalize blocks: System, Timestamp, Aura, Grandpa,
Balances, TransactionPayment, GameSolver (index 7), AdminUtils, and Utility.
All other pallets are behind a cargo feature flag and are not compiled by
default.

Pallet-game-solver exposes only the extrinsics exercised by the stage-0 loop:
subnet registration, neuron registration, weight submission (commit-reveal v2),
staking, and administrative configuration. Removed extrinsics are compiled out
of the default metadata surface; enabling `full-runtime` restores the inherited
dispatchables for legacy tests and future stage-1 work.

The node binary builds and starts a local devnet without Frontier/EVM service
code. The fast-runtime feature flag continues to work for accelerated block
production in development and testing.

Existing stage-0 flow tests continue to pass against the reduced surface.
Default-build chain comments use explicit rationale where needed instead of raw
TODO/FIXME backlog markers.

## Acceptance Criteria

- The runtime compiles with 9 or fewer pallets in the default feature set
  (7 stage-0 functional pallets plus Aura and Grandpa).
- Pallet-game-solver exposes 20 or fewer extrinsics.
- Stage-0 flow tests pass on the reduced pallet surface.
- The node binary builds without Frontier/EVM service dependencies in the
  default feature set.
- Fewer than 20 TODO/FIXME comments remain in the default-build chain crate
  source files.
- The `substrate_fixed` pin to encointer fork v0.6.0 is preserved (required for
  Yuma Consensus determinism).
