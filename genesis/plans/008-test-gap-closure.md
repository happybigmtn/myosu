# 008 — Test Gap Closure

## Objective

Close the identified test gaps that affect confidence in stage-0 correctness. Focus on gaps that block the miner quality benchmark (Plan 009) or affect operator safety.

## Context

The ASSESSMENT identifies these test gaps:

1. **Miner HTTP axon security** — No tests for malformed requests or oversized payloads to the poker HTTP axon
2. **Key management edge cases** — No concurrent access or corruption recovery tests
3. **Cross-game scoring fairness** — No test verifying that different games produce comparable quality metrics
4. **Runtime upgrade path** — Only one smoke test; no real migration exercised (acceptable for stage-0 fresh genesis)

The highest-priority gap is HTTP axon security because it is the only network-exposed surface in the miner.

## Acceptance Criteria

- At least 3 new tests for `myosu-miner` HTTP axon:
  - Malformed request body returns error, does not panic
  - Oversized request body (>1 MiB) is rejected
  - Concurrent requests do not deadlock or corrupt state
- At least 1 new test for `myosu-keys` storage corruption:
  - Corrupted key file is detected and reported, does not panic
- At least 1 test documenting cross-game score comparability:
  - Same solver quality for poker vs. Liar's Dice produces scores in the same range (or explicitly documents that they do not)
- All existing tests still pass
- CI `active-crates` job remains green

## Verification

```bash
# Run new tests
SKIP_WASM_BUILD=1 cargo test -p myosu-miner --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-keys --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-validator --quiet

# Confirm full suite
SKIP_WASM_BUILD=1 cargo test --workspace --quiet
```

## Dependencies

- Plan 006 (Phase 1 gate) — clean codebase before adding tests
