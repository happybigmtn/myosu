# ADR 002: Substrate Fork Strategy

- Status: Accepted
- Date: 2026-04-02
- Deciders: Myosu maintainers
- Consulted: `ops/decision_log.md`, `AGENTS.md`, `specs/031626-01-chain-fork-scaffold.md`, `specs/040226-01-chain-runtime-reduction.md`
- Informed: chain and node contributors, operators bringing up local devnets
- Related: `crates/myosu-chain/`, `tests/e2e/local_loop.sh`, `tests/e2e/two_node_sync.sh`, `crates/myosu-chain/node/tests/stage0_local_loop.rs`

## Context

This is a retroactive record of the 2026-03-16 decision to fork Bittensor's
subtensor rather than build Myosu as an attached subnet or a brand-new chain.

Myosu needs its own control over runtime shape, token mechanics, subnet
lifecycle, and game-specific incentive behavior. Running only as a native
Bittensor subnet would preserve upstream economics and governance constraints
that do not match the game-solving product. Building a new chain from scratch
would give full control, but it would also mean re-deriving networking,
consensus, staking, and much of the incentive scaffolding before the repo could
prove even a stage-0 local loop.

The stage-0 repo now already embodies the fork strategy: an owned chain runtime,
owned node binary, reduced pallet surface, and executable local-loop proofs all
live under `crates/myosu-chain/`.

## Decision

Myosu is an owned Substrate chain forked from subtensor.

The fork strategy covers:

- keeping the Substrate node/runtime structure and proven incentive baseline as
  the starting point
- reducing the inherited runtime aggressively for stage 0 instead of treating
  the upstream surface as sacred
- owning the chain crate, node crate, chain spec, and pallet behavior inside
  this repository
- using upstream subtensor as provenance and reference, not as an always-live
  runtime dependency boundary

## Alternatives Considered

### Option A: Fork subtensor into an owned chain

This won because it gave Myosu direct control over runtime and economic shape
while preserving a faster path to a working devnet than a clean-sheet chain.

### Option B: Deploy purely as a Bittensor subnet

This was rejected because it would leave chain parameters, economics, and
protocol boundaries under another network's constraints.

### Option C: Build a new chain from scratch

This was rejected because it would delay the first truthful end-to-end proof by
rebuilding consensus, node, and runtime primitives that subtensor already had.

## Consequences

### Positive

- Myosu can reduce pallets, extrinsics, and runtime APIs to match stage-0
  needs instead of carrying AI-specific baggage forever.
- Local devnet and e2e proofs live inside the repo rather than depending on an
  external network.
- Future product-specific chain decisions can land without negotiating another
  protocol's priorities first.

### Negative

- The fork inherits real upstream complexity, including pallets and test
  surfaces that need continuous trimming.
- Upstream drift and security posture must be tracked deliberately.

### Follow-up

- Keep documenting which inherited surfaces are still intentionally carried and
  which are stage-0 debt being removed.
- Use runtime and node proof commands as the truth source for the fork, not
  narrative claims alone.

## Reversibility

Hard to reverse.

Once operators, chain specs, and pallet state are centered on the owned fork,
moving back to "just a subnet" or to a fresh chain would require a full
migration of runtime behavior, operators, and proofs. The decision should only
be reopened if a later architecture spec shows that the owned-chain surface is
blocking the product more than it is enabling it.

## Validation / Evidence

- `crates/myosu-chain/` contains the owned runtime, node, pallet, and chain
  spec surfaces.
- `crates/myosu-chain/node/tests/stage0_local_loop.rs` exercises the local
  stage-0 chain loop from inside Cargo.
- `tests/e2e/local_loop.sh` and `tests/e2e/two_node_sync.sh` prove that the
  forked chain boots, produces blocks, and supports the live stage-0 workflow.
