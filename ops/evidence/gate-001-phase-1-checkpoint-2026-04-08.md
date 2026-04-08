# GATE-001 Phase 1 Checkpoint

- Timestamp: `2026-04-08T05:00:53Z`
- Outcome: `Phase 1 complete, proceed to Phase 2`
- Governing queue task: [IMPLEMENTATION_PLAN.md](/home/r/coding/myosu/IMPLEMENTATION_PLAN.md)
- Spec reference used during verification: [specs/070426-runtime-architecture.md](/home/r/coding/myosu/specs/070426-runtime-architecture.md)

## Decision

Phase 1 is green on the live tree. The repo satisfies the queue gate:

- `crates/myosu-chain/pallets/subtensor/` is absent
- `pallet-game-solver` is the sole live game-solving pallet
- active runtime/node/chain-client code paths no longer use `pallet_subtensor`,
  `SubtensorModule`, or `subtensorModule`
- active migration file count is `2`
- workspace compile/test proofs passed
- pallet stage-0 proof passed
- required E2E local loop and emission flow proofs passed
- doctrine/root-doc integrity remained truthful

## Evidence

Executed proofs:

```bash
test ! -d crates/myosu-chain/pallets/subtensor
ls crates/myosu-chain/pallets/game-solver/src/migrations/migrate_*.rs | wc -l
SKIP_WASM_BUILD=1 cargo check --workspace
SKIP_WASM_BUILD=1 cargo test --workspace --quiet
cargo test -p pallet-game-solver --quiet -- stage_0
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet
bash tests/e2e/local_loop.sh
bash tests/e2e/emission_flow.sh
bash .github/scripts/check_doctrine_integrity.sh
```

Observed outcomes:

- `cargo check --workspace`: passed. Residual note: Cargo reported a
  future-incompat warning from external dependency `trie-db v0.30.0`.
- `cargo test --workspace --quiet`: passed after removing an avoidable dead-code
  warning in `pallet-subtensor-utility` test code.
- `cargo test -p pallet-game-solver --quiet -- stage_0`: `26 passed`.
- `cargo test -p myosu-chain --test stage0_local_loop --quiet`: `12 passed, 1 ignored`.
- `bash tests/e2e/local_loop.sh`: passed with `LOCAL_LOOP myosu e2e ok`.
- `bash tests/e2e/emission_flow.sh`: passed with `EMISSION_FLOW ok` and
  `distribution_rounding_loss=6`.

## Contract Conflicts

The queue task and live code agree, but the referenced spec is stale in one
important way:

- [specs/070426-runtime-architecture.md](/home/r/coding/myosu/specs/070426-runtime-architecture.md)
  still describes the runtime as mapping `pallet-game-solver` through a
  `pallet_subtensor` alias and `SubtensorModule`.
- The live runtime now uses `pallet_game_solver` and `GameSolver`, and the
  queue acceptance criterion correctly requires no `pallet_subtensor` alias in
  active code paths.

That drift is recorded as `DOC-RUNTIME-001` in
[WORKLIST.md](/home/r/coding/myosu/WORKLIST.md).

## Notes

The first E2E rerun failed for environment/runtime-proof reasons, not for
Phase 1 logic:

- this machine was missing `wasm32v1-none`, which was installed before re-run
- several repo-owned E2E shell scripts assumed a repo-local `target/debug/`
  path even when `CARGO_TARGET_DIR` redirected builds elsewhere

The scripts were updated to derive binaries/runtime wasm from
`${CARGO_TARGET_DIR:-$repo_root/target}` so the gate remains truthful across
shared target-cache environments.
