# 009: Phase 2 Decision Gate

## Objective

Evaluate whether Phase 1 (reduce and harden) and Phase 2 (network proof) results
justify proceeding to Phase 3 (operator packaging). This is a decision checkpoint,
not an implementation plan.

## Gate Criteria

All of the following must be true before Phase 3 proceeds:

1. **Dead code removed** (Plan 002): `pallet-subtensor` directory does not exist.
2. **Emission invariant proven** (Plan 003): `emission_accounting_invariant` test
   passes and `emission_flow.sh` asserts the invariant on live devnet.
3. **Test suite clean** (Plan 004): Default-build pallet tests exercise only
   stage-0 active code paths.
4. **Storage audited** (Plan 005): ≤100 storage items in default-build metadata.
5. **Multi-node finality** (Plans 006-007): Three-node devnet reaches and
   maintains GRANDPA finality, including across node restart.
6. **Cross-node emission agreement** (Plan 008): All nodes produce identical
   emission distributions.

## Decision Outcomes

### Proceed to Phase 3
If all gate criteria pass, proceed to operator packaging (Plans 010-012).

### Return to Phase 1
If emission accounting fails at network scale:
- Investigate whether the identity-swap stub causes divergence
- Consider whether `substrate_fixed` determinism holds across architectures
- Add targeted fixes and re-test before proceeding

### Escalate
If multi-node finality cannot be achieved:
- The chain spec or consensus configuration may need changes
- This would require a new plan focused on consensus debugging
- Phase 3 is blocked until finality is proven

## Verification

This plan is closed by a written decision document at
`ops/decision-gate-009.md` that records:
- Date of evaluation
- Status of each gate criterion (pass/fail with evidence)
- Decision taken (proceed / return / escalate)
- Any conditions or scope adjustments for Phase 3

## Dependencies

- 002 through 008 must all be complete (pass or fail) before this gate runs.
