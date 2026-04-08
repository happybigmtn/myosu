# 010 — Phase 2 Decision Gate

## Objective

Verify that Phase 2 hardening (Plans 007–009) completed successfully. Confirm emission accounting is decided, test gaps are closed, and a quality benchmark exists. Evaluate readiness for operator packaging.

## Acceptance Criteria

All of the following must be true:

- Emission dust policy is decided and implemented (ADR exists)
- `try_state` threshold for emission accounting is justified by measured data, not an arbitrary large number
- HTTP axon security tests exist and pass
- Key management corruption recovery test exists and passes
- At least one game has a documented minimum training iteration recommendation
- Miner quality benchmark can produce a score independent of the self-scoring path
- All CI jobs green
- All E2E scripts pass
- WORKLIST.md items `EM-DUST-001` and `MINER-QUAL-001` are resolved or have concrete follow-on plans

## Verification

```bash
# Full test suite
SKIP_WASM_BUILD=1 cargo test --workspace --quiet
bash tests/e2e/local_loop.sh
bash tests/e2e/emission_flow.sh

# Check resolved worklist items
grep -c "RESOLVED\|CLOSED" WORKLIST.md || echo "Check manually"
```

This gate produces a decision artifact: either "Phase 2 complete, proceed to Phase 3 packaging" or "Phase 2 incomplete, the following items need rework: [list]."

## Dependencies

- Plan 007 (emission dust policy)
- Plan 008 (test gap closure)
- Plan 009 (miner quality benchmark)
