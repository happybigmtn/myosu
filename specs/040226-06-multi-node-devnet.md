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

Current state: The chain now builds and produces blocks on both the single-node
local path and the two-node local devnet proof. `devnet.rs` seeds non-dev
authorities, funded operator accounts, and subnet 7 bootstrap storage. The
operator bundle already rewrites its bundled `devnet-spec.json` with bootnode
metadata from `ops/deploy-bootnode.sh --dry-run`; the remaining subtlety is
that direct `--chain devnet` output still omits embedded bootnodes unless that
bundle rewrite or an explicit `--bootnodes` flag is used.

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

The chain binary supports named networks via chain spec files. The checked-in
`devnet.rs` now seeds non-dev authorities, funded operator accounts, and subnet
7 bootstrap storage. Direct `--chain devnet` output still omits embedded
bootnodes, so the truthful operator surface is the rewritten bundle spec or an
explicit `--bootnodes` flag at startup.

The operator bundle produced by
`.github/scripts/prepare_operator_network_bundle.sh` contains startup scripts,
pre-built chain specs, and a verification script. It now reads bootnode
metadata from `ops/deploy-bootnode.sh --dry-run`, rewrites the bundled
`devnet-spec.json` with that bootnode, and verifies the bundle contract with
`.github/scripts/check_operator_network_bootstrap.sh`.

The subtensor fork uses libp2p for peer-to-peer networking, which is standard
for Substrate chains. However, the fork may contain custom networking code that
affects peer discovery behavior.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Chain spec framework | `crates/myosu-chain/node/src/chain_spec/` | Extend | Devnet genesis is now seeded; the remaining work is keeping the raw-spec vs bundled-spec bootnode story explicit |
| Named network support | Chain binary CLI flags | Reuse | Already supports `--chain` flag |
| Operator bundle | `.github/scripts/prepare_operator_network_bundle.sh` | Extend | Already injects bootnode metadata into the bundled devnet spec and README |
| Bundle startup scripts | `start-miner.sh`, `start-validator.sh` | Extend | Already carry the operator-facing devnet connection parameters |
| Chain spec generators | `build-devnet-spec.sh`, `build-test-finney-spec.sh` | Extend | Bundled generation path already rewrites bootnodes even though direct `--chain devnet` does not |
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
for game-solving. The operator-facing devnet bundle distributes a rewritten
`devnet-spec.json` that carries the bootnode multiaddr; direct `--chain devnet`
startup still requires either that bundled spec or an explicit `--bootnodes`
flag.

A bootnode runs as a persistent chain node with a stable network identity and
known multiaddress. The bootnode stores chain state persistently across restarts.
The bootnode exposes P2P and RPC endpoints.

When a second node starts with the bundled devnet spec or an explicit
`--bootnodes` flag, it discovers the bootnode, connects, and synchronizes
blocks. After synchronization, both nodes produce and finalize blocks. A miner
or validator connecting to either node can interact with the same chain state.

The operator bundle includes the devnet chain spec, bootnode addresses, and
startup scripts that pass the correct `--bootnodes` flag. An operator following
the bundle instructions can join the devnet without editing configuration files.

## Acceptance Criteria

- The devnet genesis includes initial authorities, pre-funded accounts, and
  subnet 7 pre-registered, and the operator-facing devnet bundle or startup
  commands carry truthful bootnode connection parameters.
- Two independent nodes started with the bundled devnet spec or explicit
  `--bootnodes` configuration discover each other and synchronize blocks.
- A miner or validator connected to either node can read consistent chain state.
- The operator bundle includes devnet connection parameters and updated startup
  scripts.
- A bootnode deployment script produces a running node with persistent storage
  and stable network identity.
