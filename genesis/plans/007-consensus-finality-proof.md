# 007: Consensus Finality Proof

## Objective

Prove that the multi-node devnet reaches GRANDPA finality and survives
a single-node restart without chain fork. This validates that the chain
can tolerate the expected failure mode of an operator restarting a node.

## Context

GRANDPA finality requires ⅔+1 of validators to agree on a block.
With three authorities, all three must participate for finality (2/3 = 1.33,
so 2 of 3 is sufficient). The test must prove:
1. Finality advances during normal operation
2. Finality pauses when a node goes down (if only 2 of 3 are needed, it should continue)
3. A restarted node catches up and participates in finality

## Acceptance Criteria

- A new E2E script `tests/e2e/consensus_resilience.sh` that:
  - Starts three authority nodes
  - Waits for finality to advance past block N
  - Stops one node
  - Verifies remaining two nodes continue producing blocks
  - Verifies finality continues (2/3 threshold met)
  - Restarts the stopped node
  - Verifies it syncs to current block
  - Verifies finality includes all three nodes again
  - Completes within 180 seconds
- The script exits nonzero if any assertion fails
- The script cleans up all processes on exit

## Verification

```bash
bash tests/e2e/consensus_resilience.sh
```

## Dependencies

- 006 (multi-node devnet) -- reuses the three-node helper infrastructure
