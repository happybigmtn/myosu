# Specification: Chain RPC Client

Source: Reverse-engineered from crates/myosu-chain-client (lib.rs, Cargo.toml)
Status: Draft
Depends-on: 010-game-solver-pallet

## Purpose

The chain RPC client provides a shared JSON-RPC interface for interacting with
the Myosu Substrate chain. It abstracts WebSocket connectivity, SCALE-encoded
storage queries, signed extrinsic construction, and polling-based transaction
confirmation into a single library used by miners, validators, and the gameplay
surface. Without this shared seam, each binary would duplicate chain interaction
logic.

The primary consumers are the miner binary (registration, axon serving), the
validator binary (registration, staking, weight submission), and the gameplay
surface (miner discovery).

## Whole-System Goal

Current state: The client library is fully implemented with 12 RPC methods, 18+
storage query patterns, 13 transaction types, and 13 polling operations. It
handles extrinsic signing with Substrate's multi-extension system.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: All three downstream binaries can register, query, and transact
on the chain through a single shared library with consistent error handling and
polling behavior.

Still not solved here: The chain runtime and pallet behavior that the client
queries, multi-node networking, and operator key management are separate
concerns.

## Scope

In scope:
- WebSocket JSON-RPC connectivity to a Substrate node
- System health, RPC method discovery, and block header queries
- SCALE-encoded storage reads for pallet-game-solver and pallet-admin-utils
  state
- Neuron info retrieval via custom RPC endpoint
- Signed extrinsic construction with sr25519 and Substrate's extension chain
- 13 transaction types covering registration, axon serving, staking, subnet
  management, and weight submission
- Polling-based transaction confirmation with configurable timeouts
- Structured report types for all operations

Out of scope:
- Key generation or encryption (handled by myosu-keys)
- Training, scoring, or gameplay logic
- Chain runtime or pallet implementation
- Multi-node peer discovery or gossip

## Current State

The crate exists at crates/myosu-chain-client with approximately 2,560 lines of
code. It depends on jsonrpsee for WebSocket RPC, sp-core and sp-runtime for
Substrate primitives, codec for SCALE encoding, and the chain runtime and pallet
crates for type definitions.

The client connects via WebSocket (default ws://127.0.0.1:9944) and provides
methods grouped into: system queries (health, RPC methods, block headers,
runtime version), storage queries (UID lookup, hotkey lookup, axon info, stake,
subnet membership, validator permits, weights, incentive/dividend/emission
vectors, commit-reveal state, tempo, rate limits), and transactions (burned
registration, axon serving, subnet registration, staking, sudo parameter
overrides, weight submission via direct set or commit-reveal).

Extrinsic signing collects a signing context (genesis hash, best block hash and
number, nonce, spec and transaction versions) and constructs a signed extrinsic
with eight extensions: CheckEra, CheckNonZeroSender, CheckWeight, CheckTxVersion,
CheckGenesis, CheckSpecVersion, CheckMetadataHash, and ChargeTransactionPayment.

All high-level operations return structured report types that include operation
metadata (hotkey, subnet, UID), success indicators (already-registered,
already-published), and optional extrinsic hashes. Polling operations use a
default 500ms interval and configurable timeout, retrying storage reads until
the expected state appears or the timeout expires.

Error handling uses a comprehensive ChainClientError enum with 12 variants
covering empty endpoints, RPC failures, parsing errors, SCALE decoding failures,
timeouts, missing subnet members, and validator permit checks.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| WebSocket RPC | jsonrpsee ws-client with 12 RPC methods | Reuse | Standard Substrate RPC interface |
| Storage queries | 18+ SCALE-decoded storage key patterns | Reuse | Covers all stage-0 pallet state |
| Extrinsic signing | 8-extension signing context with sr25519 | Reuse | Matches chain runtime extensions |
| Transaction types | 13 transaction builders | Reuse | Covers registration through weight submission |
| Polling | 13 poll operations with 500ms default interval | Reuse | Reliable confirmation for all operations |
| Report types | Structured reports for all operations | Reuse | Downstream binaries consume for display |

## Non-goals

- Providing a generic Substrate client for arbitrary pallets.
- Managing WebSocket reconnection or failover.
- Handling key generation, storage, or encryption.
- Implementing batch or parallel transaction submission.
- Providing event subscription or real-time chain monitoring.

## Behaviors

The client connects to a WebSocket endpoint and provides three categories of
operations.

System queries fetch node health (peer count, sync status), advertised RPC
method lists, block headers by hash or latest, and runtime version information.
These are used during startup to validate connectivity.

Storage queries read SCALE-encoded values from chain state. Each query
constructs a storage key from pallet name, storage item name, and optional map
keys (using Blake2_128Concat or Identity hashers depending on the key type),
fetches the hex-encoded value via state_getStorage, and decodes it. Failed
decoding produces a StorageDecode error with the storage key for diagnostics.
Storage queries support both current-head and at-block-hash variants.

Transactions construct SCALE-encoded call data, collect signing context from
the chain, sign with sr25519, and submit via author_submitExtrinsic. The
signing context fetches the current nonce, genesis hash, best block, and runtime
version in a single batch. Eight transaction-validity extensions are applied.

High-level operations compose storage queries and transactions with polling.
For example, ensure_burned_registration submits a registration transaction then
polls for the UID to appear in storage, returning a RegistrationReport with the
UID, hotkey, subnet, already-registered flag, and transaction hash.

Weight submission supports two modes: direct set_weights when commit-reveal is
disabled, or a two-phase commit then reveal when commit-reveal is enabled. The
client checks the CommitRevealWeightsEnabled storage value to select the mode
automatically.

Miner discovery queries all chain-visible miners on a subnet by fetching neuron
info and axon info, constructing ChainVisibleMiner records with incentive values
for sorting.

## Acceptance Criteria

- The client connects to a WebSocket endpoint and successfully queries system
  health.
- Storage queries correctly decode SCALE-encoded values for all 18+ storage
  patterns.
- Signed extrinsics are accepted by the chain without signature validation
  errors.
- Polling operations detect state changes within the configured timeout.
- Polling operations produce a Timeout error when the expected state does not
  appear.
- Weight submission automatically selects direct or commit-reveal mode based on
  chain configuration.
- All operations return structured report types with correct metadata.
- Error variants include sufficient context (method name, storage key, operation
  name) for diagnostics.
