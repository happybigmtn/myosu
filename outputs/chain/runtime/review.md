# Chain Runtime Review

**Lane**: `chain:runtime`
**Date**: 2026-03-29

## Judgment Summary

**Judgment: KEEP — Reduced runtime is real and locally provable**

The runtime lane is no longer a restart exercise. The live `myosu-chain`
runtime/node pair has already crossed into a stripped, stage-0-real state:

- `GameSolver` is the live runtime identity at index `7`
- the remaining non-stage-0 subtensor/runtime baggage has been aggressively cut
- the core node-owned local loop has its own passing integration harness
- the honest verification path in this environment uses `SKIP_WASM_BUILD=1`
  because `wasm32-unknown-unknown` is missing here

## Verified Today

Fresh proof on 2026-03-29:

```bash
SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime
SKIP_WASM_BUILD=1 cargo check -p myosu-chain
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
```

Results:

- runtime check passes
- node check passes
- `stage0_local_loop` is `16 passed, 0 failed, 1 ignored`

Residual notice:

- upstream `trie-db v0.30.0` future-incompat warning

## Surface Assessment

| Surface | Status | Rationale |
|---------|--------|-----------|
| runtime identity (`GameSolver` at index 7) | **KEEP** | This is the live stage-0 contract now |
| node/runtime package wiring | **KEEP** | Both compile on the current workspace line |
| stage0 local loop harness | **KEEP** | Real integration gate, not a narrative placeholder |
| stripped swap/EVM/placeholder surfaces | **RESET LANDED** | Removed from the active path |
| benchmark support lane | **KEEP WITH CAUTION** | Still downstream/support-lane work, not core-path truth |

## What Changed Since The Old Review

The stale review was already closer to the truth than the older restart-era
writeup, but it still centered the runtime cutover story. The current truth is
further along:

- plan `003` is closed
- the final allowlist/index reconciliation is done
- the `MevShield` carryover is gone from the stage-0 path
- the node-owned local loop is no longer just a runtime boot proof; it also
  owns startup, registration, weight-routing, economics, and gameplay
  completion invariants

## Residual Risks

- Non-`SKIP_WASM_BUILD` proof remains environment-gated here by the missing
  target, so the stripped proof route must stay explicit.
- Benchmark support remains a secondary lane rather than part of the core
  stage-0 contract.

## Recommendation

Treat the runtime lane as completed on the active stage-0 path. Preserve the
current stripped compile proof, preserve the node-owned local loop harness, and
avoid re-describing the runtime as a restart target when the live code already
proves otherwise.
