# Minimize Node for a Working Local Devnet

Status: Completed 2026-03-29.

This ExecPlan is a living document. Maintained per `genesis/PLANS.md`.

## Purpose / Big Picture

The runtime alone is not the product. Stage 0 requires a node that starts,
produces blocks, and exposes the RPC surface the miner and validator will use.
The current node still carries extra network machinery inherited from the
broader subtensor fork.

After this plan, `myosu-chain` will start as a minimal local devnet node using
the stripped runtime, author blocks with Aura/Grandpa, and expose the
game-solving RPC methods needed for the next layer.

## Progress

- [x] (2026-03-28) Confirmed that the node crate exists and still follows the
  runtime's larger inherited surface.
- [x] (2026-03-28) Aligned the node service with the stripped runtime from
  plan 003.
- [x] (2026-03-28) Removed node-side reliance on Frontier/EVM and other
  non-stage-0 paths.
- [x] (2026-03-28) Made `--dev --tmp` boot a working local node that produces
  blocks and advances finality.
- [x] (2026-03-28) Verified that the node exposes the game-solver RPC methods
  needed by the future chain client.
- [x] (2026-03-28) Added a reproducible local node smoke proof via
  `myosu-chain --smoke-test`.

## Surprises & Discoveries

- Observation: The node is downstream of the runtime problem, not separate from
  it.
  Evidence: Node service and RPC code still reference runtime surfaces that are
  supposed to disappear in plan 003.

- Observation: The stripped node still needed explicit local authority key
  seeding before `--dev --tmp` would author blocks.
  Evidence: The node reached best `#0` repeatedly until Aura and GRANDPA dev
  keys were inserted into the local keystore for the authority role.

## Decision Log

- Decision: Prove `--dev --tmp` first before inventing more named chain specs.
  Rationale: A single-node local devnet is the fastest proof that the stripped
  chain can actually run.
  Date/Author: 2026-03-28 / Codex

## Outcomes & Retrospective

The node portion of the chain strip-down is now materially real: the stripped
service boots, authors blocks, finalizes locally, has a bounded smoke gate, and
responds on both `system_health` and `neuronInfo_getNeuronsLite`. Plan 004 is
no longer blocked on "can the node run?" and instead hands off to later chain
integration work.

## Context and Orientation

The node lives under `crates/myosu-chain/node/`. The files that matter most are:

- `src/service.rs` for node startup and consensus wiring
- `src/rpc.rs` for RPC registration
- `src/command.rs` for CLI behavior and chain selection
- `src/chain_spec/` for local genesis configuration

The target behavior is simple: start a node locally, watch it produce blocks,
and query it over JSON-RPC.

## Milestones

### Milestone 1: Node follows the stripped runtime

Update the node so it builds cleanly against the runtime produced by plan 003.

Proof command:

    cargo check -p myosu-chain

### Milestone 2: Local node produces blocks

Start the node locally and verify it authors blocks.

Proof commands:

    cargo run -p myosu-chain -- --smoke-test

    curl -s -H "Content-Type: application/json" \
      -d '{"id":1,"jsonrpc":"2.0","method":"system_health","params":[]}' \
      http://localhost:9944

Acceptance is a passing smoke report plus a healthy RPC response from a running
local node when manually inspected.

### Milestone 3: Game-solver RPC is live

Verify the custom RPC surface responds well enough for a future typed chain
client to consume it.

Proof command:

    curl -s -H "Content-Type: application/json" \
      -d '{"id":1,"jsonrpc":"2.0","method":"neuronInfo_getNeuronsLite","params":[1]}' \
      http://localhost:9944

## Plan of Work

Strip node code that assumes the removed runtime paths still exist, then bring
the node up in the smallest possible local configuration. Once block production
is stable, codify that proof as a bounded smoke command, then check the custom
RPC registration and record the commands that prove it.

## Concrete Steps

From `/home/r/coding/myosu`:

    cargo check -p myosu-chain
    ls crates/myosu-chain/node/src
    sed -n '1,220p' crates/myosu-chain/node/src/service.rs
    sed -n '1,220p' crates/myosu-chain/node/src/rpc.rs

## Validation and Acceptance

This plan is complete when:

- `cargo check -p myosu-chain` passes.
- `cargo run -p myosu-chain -- --smoke-test` passes.
- JSON-RPC responds on `localhost:9944`.
- The game-solver RPC methods respond from the running local node.

Current status: all four acceptance checks have been satisfied on 2026-03-28.

## Idempotence and Recovery

Always test with `--tmp` so local runs are disposable. If the node fails to
start after a strip-down change, keep the stripped runtime and repair the node
service rather than reintroducing removed chain features.

## Interfaces and Dependencies

Depends on: plan 003.
Blocks: plan 007 and later integration proof work.
