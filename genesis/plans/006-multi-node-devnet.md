# 006: Multi-Node Devnet Proof

## Objective

Extend the proven two-node sync test to a three-node devnet with automated
startup, block production, finality, and shutdown. This is the minimum viable
network proof for stage-0 exit.

## Context

`tests/e2e/two_node_sync.sh` already proves:
- Bootnode starts with authority keys from `MYOSU_NODE_AUTHORITY_SURI`
- Second node discovers bootnode via multiaddr
- Blocks sync between nodes

What is not yet proven:
- Three or more nodes reaching GRANDPA finality
- Automated startup/teardown of multi-node devnet
- Block finality (not just production) across nodes

## Acceptance Criteria

- A new E2E script `tests/e2e/three_node_devnet.sh` that:
  - Starts three authority nodes on isolated ports
  - Waits for all three to produce blocks
  - Verifies GRANDPA finality (finalized block number advances)
  - Verifies block hash agreement across all three nodes
  - Cleans up all node processes on exit (including on failure)
  - Completes within 120 seconds on a standard CI runner
- `tests/e2e/helpers/` contains reusable functions for:
  - Starting a node with authority keys
  - Waiting for block production
  - Querying finalized block number via RPC
  - Comparing block hashes across nodes
- The devnet chain spec supports three named authorities
- CI `integration-e2e` job runs the three-node test

## Verification

```bash
# Three-node devnet proof
bash tests/e2e/three_node_devnet.sh

# Existing two-node test still passes
bash tests/e2e/two_node_sync.sh

# Existing local loop still passes
bash tests/e2e/local_loop.sh
```

## Dependencies

- None for the script itself, but Plans 002-005 should land first so the
  chain binary being tested reflects the reduced surface.
