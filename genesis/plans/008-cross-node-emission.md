# 008: Cross-Node Emission Agreement

## Objective

Prove that emission distribution produces identical results across all nodes
in the multi-node devnet. This validates that the emission accounting invariant
holds at network scale, not just in single-node tests.

## Context

Plan 003 hardens emission accounting in unit tests and single-node E2E.
This plan extends that proof to the multi-node case. The specific risk is
that floating-point or fixed-point determinism breaks when different nodes
process the same blocks in slightly different order or timing.

The Yuma Consensus code uses `substrate_fixed` (I32F32, I64F64, I96F32)
for bit-identical computation. The test must verify that this determinism
holds across independent node processes.

## Acceptance Criteria

- A new E2E script `tests/e2e/cross_node_emission.sh` that:
  - Starts three authority nodes
  - Creates a subnet with known tempo
  - Registers miners and validators across nodes
  - Advances through multiple epochs
  - Queries emission state from each node's RPC
  - Asserts all nodes report identical emission distributions
  - Asserts the accounting invariant holds on each node independently
- The comparison tolerance is zero (bit-identical) for fixed-point values
  and epsilon < 1e-6 for any floating-point derived values

## Verification

```bash
bash tests/e2e/cross_node_emission.sh
```

## Dependencies

- 003 (emission hardening) -- emission accounting must be proven locally first
- 006 (multi-node devnet) -- requires the multi-node infrastructure
