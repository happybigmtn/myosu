# Specification: Chain Node

Source: Reverse-engineered from crates/myosu-chain/node (main.rs, cli.rs, command.rs, chain_spec/, consensus/, service.rs, rpc.rs)
Status: Draft
Depends-on: 011-chain-runtime

## Purpose

The chain node binary runs the Substrate blockchain with configurable consensus
and chain specifications. It assembles the networking layer, consensus engine,
RPC endpoints, and telemetry into a full node that produces and validates
blocks. The node is the infrastructure layer that makes the runtime's state
transition function available to the network.

The primary consumer is a node operator running the blockchain on a local devnet
or multi-node network.

## Whole-System Goal

Current state: The node binary is implemented with support for Aura and BABE
consensus via a trait abstraction, four chain spec templates (devnet, localnet,
testnet, finney), WebSocket RPC on port 9944, and a stage-0 local loop test.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: An operator can start a blockchain node that produces blocks,
serves RPC requests, and participates in consensus on a local or multi-node
network.

Still not solved here: Multi-node peer discovery configuration, production
deployment, monitoring, and runtime upgrade mechanics are separate concerns.

## Scope

In scope:
- Substrate node binary with CLI command handling
- Consensus mechanism abstraction (Aura and BABE implementations)
- Chain spec templates: devnet, localnet, testnet, finney
- WebSocket RPC endpoint serving
- Full node service assembly (networking, consensus, transaction pool)
- Grandpa finality gadget
- Stage-0 local loop smoke test
- Chain spec build command (build-spec)

Out of scope:
- Runtime logic (separate spec)
- Pallet behavior (separate spec)
- Multi-node networking configuration details
- Production deployment and monitoring
- Key management for authority nodes

## Current State

The node exists at crates/myosu-chain/node. The main binary parses CLI commands
including standard Substrate subcommands (build-spec, check-block, export-blocks,
import-blocks, purge-chain, revert) and the default run command.

The consensus mechanism trait abstracts over Aura and BABE implementations. Each
implementation provides: block import queue construction, slot duration
configuration, inherent data providers, block authoring startup, essential task
spawning, and RPC method registration. The trait allows switching consensus
without modifying the service assembly.

Four chain spec templates configure genesis state: devnet (development with known
authorities), localnet (local testing), testnet (public test network), and
finney (named production-like network). Each spec defines initial authority keys
(Aura + Grandpa), initial balances, and custom chain extensions (fork blocks,
bad blocks).

The service module assembles a full node from: client, backend storage, network
layer, transaction pool, consensus engine, block import pipeline, Grandpa
finality voter, RPC handlers, and telemetry. The node serves WebSocket RPC on
port 9944 by default.

A stage-0 local loop integration test validates that the node can produce blocks
and process game-solver pallet extrinsics in a local devnet configuration.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Consensus abstraction | ConsensusMechanism trait with Aura/BABE | Reuse | Clean separation of consensus from service |
| Chain specs | devnet, localnet, testnet, finney templates | Reuse | Stage-0 adequate |
| Service assembly | Full node with networking, consensus, RPC | Reuse | Standard Substrate service |
| RPC serving | WebSocket on port 9944 with neuron info | Reuse | Client-compatible endpoints |
| Local loop test | stage0_local_loop integration test | Reuse | Validates end-to-end pallet function |
| Build-spec command | Chain spec generation from templates | Reuse | Operator bundle uses this |

## Non-goals

- Implementing custom networking protocols beyond Substrate defaults.
- Providing a light client or browser-embedded node.
- Managing validator key rotation or authority set changes.
- Implementing runtime upgrade proposal or governance.
- Providing built-in monitoring or alerting.

## Behaviors

On startup, the node parses CLI arguments to determine the command (run, or a
Substrate subcommand). The run command selects a chain spec by name (devnet,
localnet, testnet, or finney) and initializes the consensus mechanism.

The service assembly creates a Substrate client with the chain runtime, opens
the database backend, initializes the network layer with peer discovery, sets up
the transaction pool, constructs the block import pipeline with Grandpa
justification import, and starts the consensus engine.

For Aura consensus, the node uses slot-based authority rotation with a
configurable slot duration. For BABE consensus, the node uses VRF-based slot
assignment with primary and secondary slot types.

Grandpa finality runs as a separate voter task, finalizing blocks when
two-thirds of authorities agree. Finalized blocks are irreversible.

The RPC layer serves standard Substrate methods (system, chain, state, author)
plus custom neuron info endpoints used by the chain client library. RPC is
available over WebSocket.

The build-spec command generates a chain spec JSON file from a named template,
used by the operator bundle to produce devnet and test-finney specs.

The stage-0 local loop test starts a node in test mode, submits pallet
extrinsics (registration, weight setting), and verifies state changes.

## Acceptance Criteria

- The node starts and produces blocks on a local devnet chain spec.
- The node serves WebSocket RPC requests on the configured port.
- The build-spec command generates valid chain spec JSON for devnet and
  test-finney templates.
- The consensus mechanism can be configured for either Aura or BABE.
- Grandpa finality finalizes blocks on a multi-authority network.
- The stage-0 local loop test passes, demonstrating end-to-end pallet function.
- Standard Substrate RPC methods (system_health, chain_getHeader,
  state_getStorage) respond correctly.
- Custom neuron info RPC endpoints return SCALE-encoded neuron data.
