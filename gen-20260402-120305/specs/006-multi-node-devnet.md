# Specification: Multi-Node Devnet

Source: Genesis Plan 008 (Multi-Node Devnet), ASSESSMENT.md half-built items
Status: Draft
Depends-on: 002-single-token-emission-accounting, 005-integration-test-harness

## Purpose

Stage-0 runs on a single local node. Stage-1 requires a persistent multi-node
devnet where independent operators run their own miners and validators against a
shared chain. Without multi-node operation, the protocol cannot demonstrate its
core proposition — that independent parties can compete for emissions by
producing higher-quality game strategies. Building the devnet configuration,
bootnode infrastructure, and peer discovery verification is the gate from
stage-0 (solo prototype) to stage-1 (public network).

## Whole-System Goal

Current state: The chain builds and produces blocks on a single local node.
Devnet and test_finney chain specs at
`crates/myosu-chain/node/src/chain_spec/` are placeholder quality. The operator
bundle assumes local-only operation. No bootnode, persistent state directory, or
multi-node peer discovery has been tested.

This spec adds: A production-quality devnet chain spec with initial authorities
and pre-configured subnets, bootnode deployment capability, verified peer
discovery and block synchronization between independent nodes, and an operator
bundle updated for devnet connection.

If all ACs land: Two independent nodes discover each other, synchronize blocks,
and an operator can join the devnet using the provided bundle without manual
chain spec editing.

Still not solved here: Devnet persistence across reboots, monitoring and
alerting for devnet health, chain spec governance for adding new authorities,
and public internet deployment.

## Scope

In scope:
- Production-quality devnet chain spec with genesis configuration
- Bootnode deployment scripts and documentation
- Peer discovery and block synchronization verification between two nodes
- Updating the operator bundle for multi-node devnet connection

Out of scope:
- Public internet deployment or cloud infrastructure provisioning
- Devnet monitoring, alerting, or health dashboards
- Chain spec governance for adding or rotating authorities
- Runtime upgrades on a live devnet
- Performance testing under network load

## Current State

The chain binary supports named networks via chain spec files. Two placeholder
chain specs exist: `devnet.rs` and `testnet.rs` (test_finney) in
`crates/myosu-chain/node/src/chain_spec/`. These are functional but minimal —
they lack proper initial authority configuration, pre-funded operator accounts,
and subnet pre-registration.

The operator bundle produced by
`.github/scripts/prepare_operator_network_bundle.sh` contains startup scripts,
pre-built chain specs, and a verification script, but all assume single-node
local operation. The bundle includes `build-devnet-spec.sh` and
`build-test-finney-spec.sh` for chain spec generation.

The subtensor fork uses libp2p for peer-to-peer networking, which is standard
for Substrate chains. However, the fork may contain custom networking code that
affects peer discovery behavior.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Chain spec framework | `crates/myosu-chain/node/src/chain_spec/` | Extend | Placeholder specs need real genesis config |
| Named network support | Chain binary CLI flags | Reuse | Already supports `--chain` flag |
| Operator bundle | `.github/scripts/prepare_operator_network_bundle.sh` | Extend | Add devnet connection parameters |
| Bundle startup scripts | `start-miner.sh`, `start-validator.sh` | Extend | Add `--bootnodes` flag |
| Chain spec generators | `build-devnet-spec.sh`, `build-test-finney-spec.sh` | Replace | Generate production-quality specs |
| P2P networking | Substrate libp2p (via subtensor fork) | Reuse | Standard peer discovery |

## Non-goals

- Hosting a public-facing devnet with guaranteed uptime.
- Automating node provisioning or orchestration (Kubernetes, Terraform, etc.).
- Supporting more than a handful of nodes in the initial devnet.
- Chain governance or validator set rotation on the live devnet.
- Benchmarking transaction throughput on the devnet.

## Behaviors

The devnet chain spec defines a genesis state with initial authorities,
pre-funded operator accounts, and at least one subnet (index 7) pre-registered
for game-solving. The chain spec includes bootnode multiaddresses so that new
nodes can discover the network without manual peer configuration.

A bootnode runs as a persistent chain node with a stable network identity and
known multiaddress. The bootnode stores chain state persistently across restarts.
The bootnode exposes P2P and RPC endpoints.

When a second node starts with the devnet chain spec, it discovers the bootnode
via the embedded multiaddress, connects, and synchronizes blocks. After
synchronization, both nodes produce and finalize blocks. A miner or validator
connecting to either node can interact with the same chain state.

The operator bundle includes the devnet chain spec, bootnode addresses, and
startup scripts that pass the correct `--bootnodes` flag. An operator following
the bundle instructions can join the devnet without editing configuration files.

## Acceptance Criteria

- The devnet chain spec includes initial authorities, pre-funded accounts,
  subnet 7 pre-registered, and bootnode multiaddresses.
- Two independent nodes started with the devnet chain spec discover each other
  and synchronize blocks.
- A miner or validator connected to either node can read consistent chain state.
- The operator bundle includes devnet connection parameters and updated startup
  scripts.
- A bootnode deployment script produces a running node with persistent storage
  and stable network identity.
