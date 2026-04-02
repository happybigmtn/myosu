# Specification: Poker Solver Engine

Source: Reverse-engineered from crates/myosu-games-poker (solver.rs, robopoker.rs, wire.rs, artifacts.rs, state modules)
Status: Draft
Depends-on: 001-game-trait-framework

## Purpose

The poker solver engine provides No-Limit Hold'em strategy computation,
checkpointing, wire encoding, and blueprint inference for the Myosu system. It
wraps the robopoker MCCFR solver to produce trained strategies, persists them in
a versioned checkpoint format, and exposes wire codecs for network transport of
strategy queries and responses. This crate is the primary game vertical that
proves the system works end-to-end from training through validation to gameplay.

The primary consumers are the miner (training and serving), the validator
(scoring), and the gameplay surface (blueprint-backed advice).

## Whole-System Goal

Current state: The poker engine is fully implemented with MCCFR training,
checkpointing, wire protocol, artifact loading, and blueprint inference. It
supports both heads-up and six-max NLHE variants.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: Miners can train NLHE strategies, checkpoint them to disk,
serve them over HTTP via wire-encoded queries, and validators can
deterministically score them. Players can receive blueprint-backed strategy
advice during gameplay.

Still not solved here: Multi-game coordination on-chain, validator
cross-determinism proof, and live miner discovery are handled by other system
components.

## Scope

In scope:
- MCCFR solver wrapper with step, train, query, recommend, and exploitability
- Versioned binary checkpoint format (magic bytes, version, bincode payload)
- Wire codec for strategy queries and responses (encode/decode)
- NlheBlueprint for trained strategy inference
- Artifact bundle loading for card abstraction encoders
- Poker game state types (streets, positions, actions, snapshots)

Out of scope:
- The robopoker MCCFR algorithm internals
- TUI rendering of poker game state (handled by myosu-tui integration)
- Miner training loop orchestration
- Validator scoring logic
- On-chain game type registration

## Current State

The crate exists at crates/myosu-games-poker with approximately 9,400 lines of
code. It depends on the robopoker fork (rbp-core, rbp-mccfr, rbp-nlhe,
rbp-cards, rbp-gameplay) and on myosu-games for trait re-exports.

PokerSolver wraps robopoker's NlheFlagshipSolver. It exposes step (one MCCFR
iteration), train (N iterations), query (strategy lookup by info set),
recommend (best action selection), exploitability (Nash distance), and
checkpoint_bytes (versioned binary serialization).

The checkpoint format uses a 4-byte magic header (MYOS), a 4-byte little-endian
version number (currently 1), and a bincode-encoded NlheProfile payload with a
256 MB decode size limit.

NlheBlueprint provides read-only inference from a trained profile, supporting
query, answer, and recommend operations without solver state.

Wire codecs encode and decode NlheInfoKey (subgame, bucket, choices packed into
u64/i16/u64), strategy queries, and strategy responses for network transport
between miners and validators.

Artifact bundles provide abstraction manifests and encoder data for MCCFR
training, mapping streets to buckets via NlheAbstractionManifest and
NlheEncoderArtifactBundle structures.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| MCCFR training | PokerSolver wrapping NlheFlagshipSolver | Reuse | Proven solver with exploitability measurement |
| Checkpointing | MYOS magic + version + bincode | Reuse | Versioned format prevents silent corruption |
| Wire protocol | encode/decode for InfoKey, Query, Response | Reuse | Binary codec used by miner HTTP and validator |
| Blueprint inference | NlheBlueprint with query/recommend | Reuse | Read-only inference for gameplay advice |
| Card abstraction | NlheAbstractionManifest, artifact bundles | Reuse | Disk-backed encoder loading for training |
| Game state types | NlheStreet, NlheAction, NlheSnapshot | Reuse | Shared by TUI, wire, and solver |

## Non-goals

- Modifying the robopoker MCCFR algorithm.
- Providing a generic solver interface that works across game types.
- Handling miner registration or chain interaction.
- Rendering poker game state in a terminal.
- Defining on-chain storage for poker strategies.

## Behaviors

The solver accepts an encoder and produces a solver instance capable of
iterative MCCFR training. Each step call advances the solver by one iteration.
The train method runs a specified number of iterations sequentially.

Query accepts an NlheInfo (information set) and returns a strategy response with
action probabilities. Recommend returns the single highest-probability action.
Exploitability computes the Nash distance of the current strategy.

Checkpoint serialization writes the magic bytes MYOS, the version number 1 as
four little-endian bytes, then the bincode-encoded profile. Deserialization
validates the magic bytes, rejects unknown versions, and enforces a 256 MB size
limit on the bincode payload.

NlheBlueprint loads a profile and encoder and provides the same query and
recommend interface as the solver, without training capability.

Wire encoding packs NlheInfoKey into a compact binary representation. The
strategy query encoder wraps an info key. The strategy response encoder
serializes a vector of (edge, probability) pairs. All encode/decode operations
are symmetric and round-trip safe.

Artifact loading reads abstraction manifests from disk, maps streets to bucket
definitions, and constructs encoder bundles that the solver uses for card
abstraction during MCCFR training.

## Acceptance Criteria

- The solver produces a non-trivial strategy after training iterations, measured
  by exploitability decreasing from the initial value.
- Checkpoint format round-trips: a serialized checkpoint deserializes to an
  equivalent profile that produces identical query results.
- Checkpoint deserialization rejects payloads with wrong magic bytes, unknown
  version numbers, or payloads exceeding 256 MB.
- Wire codec round-trips: encoding then decoding a strategy query or response
  produces the original value.
- NlheBlueprint produces identical query results to the solver for the same
  profile and information set.
- The recommend method returns the action with the highest probability from the
  strategy response.
- Artifact bundles load from disk and provide valid encoders for MCCFR training.
