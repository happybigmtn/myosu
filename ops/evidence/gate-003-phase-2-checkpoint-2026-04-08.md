# GATE-003 Phase 2 Checkpoint

- Timestamp: `2026-04-08T07:03:21Z`
- Outcome: `Phase 2 complete, proceed to Phase 3`
- Governing queue task: [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md)
- Spec reference used during verification: [specs/070426-validator-subsystem.md](/home/r/coding/myosu/specs/070426-validator-subsystem.md)

## Decision

Phase 2 is green on the live tree after repairing one stale proof surface.

The repo now satisfies the gate contract:

- emission dust policy is accepted in
  [docs/adr/011-emission-dust-policy.md](/home/r/coding/myosu/docs/adr/011-emission-dust-policy.md)
- `try_state` carries the post-`EMIT-001` one-rao alert threshold in
  [crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs](/home/r/coding/myosu/crates/myosu-chain/pallets/game-solver/src/utils/try_state.rs)
- miner HTTP axon security tests remain present in
  [crates/myosu-miner/src/axon.rs](/home/r/coding/myosu/crates/myosu-miner/src/axon.rs)
- corrupted-key recovery coverage remains present in
  [crates/myosu-keys/src/storage.rs](/home/r/coding/myosu/crates/myosu-keys/src/storage.rs)
- cross-game scoring coverage and the Liar's Dice exploitability benchmark
  remain present in
  [crates/myosu-validator/src/validation.rs](/home/r/coding/myosu/crates/myosu-validator/src/validation.rs)
- the operator guide still documents `512` Liar's Dice iterations as the
  current minimum meaningful training floor in
  [docs/operator-guide/quickstart.md](/home/r/coding/myosu/docs/operator-guide/quickstart.md)
- `WORKLIST.md` resolves `EM-DUST-001` and narrows `MINER-QUAL-001` to the
  remaining poker-only blocker with a concrete follow-on
- the broad workspace suite and all seven repo-owned E2E shell proofs passed

## Evidence

Executed proofs:

```bash
test -f docs/adr/011-emission-dust-policy.md
SKIP_WASM_BUILD=1 cargo test --workspace --quiet
bash tests/e2e/local_loop.sh
bash tests/e2e/two_node_sync.sh
bash tests/e2e/four_node_finality.sh
bash tests/e2e/consensus_resilience.sh
bash tests/e2e/cross_node_emission.sh
bash tests/e2e/validator_determinism.sh
bash tests/e2e/emission_flow.sh
bash .github/scripts/check_stage0_repo_shape.sh
bash .github/scripts/check_doctrine_integrity.sh
bash .github/scripts/check_plan_quality.sh
```

Observed outcomes:

- `cargo test --workspace --quiet`: passed
- `bash tests/e2e/local_loop.sh`: passed with `LOCAL_LOOP myosu e2e ok`
- `bash tests/e2e/two_node_sync.sh`: passed with `SYNC two_node_devnet ok`
- `bash tests/e2e/four_node_finality.sh`: passed with
  `FINALITY four_node_devnet ok`
- `bash tests/e2e/consensus_resilience.sh`: passed with
  `RESILIENCE consensus_restart ok`
- `bash tests/e2e/cross_node_emission.sh`: first failed with
  `target subnet 2 has zero emission sum at finalized snapshot`; after fixing
  the stale storage prefix in
  [tests/e2e/cross_node_emission.sh](/home/r/coding/myosu/tests/e2e/cross_node_emission.sh),
  reran green with `CROSS_NODE_EMISSION ok` and
  `target_subnet_emission_sum=2460013734`
- `bash tests/e2e/validator_determinism.sh`: passed with
  `VALIDATOR_DETERMINISM myosu e2e ok`
- `bash tests/e2e/emission_flow.sh`: passed with `EMISSION_FLOW ok` and
  `distribution_rounding_loss=0`
- doctrine/repo-shape/plan-quality checks all passed

## Contract Conflicts

The plan described `GATE-003` as verification-only, but the live codebase
disagreed on first proof run:

- [tests/e2e/cross_node_emission.sh](/home/r/coding/myosu/tests/e2e/cross_node_emission.sh)
  was still querying runtime storage under the old `SubtensorModule` prefix.
- The live runtime and chain client use `GameSolver`, so the proof silently
  read default-zero emission state and failed its non-zero emission assertion.

The gate closes only after that script is updated to the live storage prefix.

## Notes

`MINER-QUAL-001` remains open only for poker, which matches the gate contract:
the worklist entry now points at a concrete follow-on instead of leaving the
quality story ambiguous. Phase 3 packaging can proceed without pretending poker
quality measurement is already solved.
