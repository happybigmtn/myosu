# 006 — Phase 1 Decision Gate

## Objective

Verify that Phase 1 cleanup (Plans 002–005) completed successfully and the codebase is in a strictly better state than before. No Phase 2 work proceeds until this gate is green.

## Context

Phase 1 makes structural changes: deleting ~90K lines of dead pallet code, renaming the active pallet, removing inherited migrations, and cleaning stale docs. Any of these could introduce subtle regressions. This gate catches them before new feature work begins.

## Acceptance Criteria

All of the following must be true:

- `pallet-subtensor` directory does not exist
- `pallet-game-solver` is the sole game-solving pallet in the workspace
- The active pallet has a consistent name in code (no `pallet_subtensor` alias in active code paths)
- Inherited migration count reduced by at least 30
- CI pipeline passes: all 9 jobs green
- All E2E scripts pass locally:
  - `bash tests/e2e/local_loop.sh`
  - `bash tests/e2e/two_node_sync.sh`
  - `bash tests/e2e/four_node_finality.sh`
  - `bash tests/e2e/consensus_resilience.sh`
  - `bash tests/e2e/cross_node_emission.sh`
  - `bash tests/e2e/validator_determinism.sh`
  - `bash tests/e2e/emission_flow.sh`
- No root-level document references nonexistent paths without marking them as planned
- `SKIP_WASM_BUILD=1 cargo check --workspace` succeeds
- `cargo test -p pallet-game-solver --quiet -- stage_0` passes

## Verification

```bash
# Full CI-equivalent local check
SKIP_WASM_BUILD=1 cargo check --workspace
SKIP_WASM_BUILD=1 cargo test --workspace --quiet
cargo test -p pallet-game-solver --quiet -- stage_0
bash tests/e2e/local_loop.sh

# Structural checks
test ! -d crates/myosu-chain/pallets/subtensor
ls crates/myosu-chain/pallets/game-solver/src/migrations/migrate_*.rs | wc -l
# Should be <= 14
```

This gate produces a decision artifact: either "Phase 1 complete, proceed to Phase 2" or "Phase 1 incomplete, the following items need rework: [list]."

## Dependencies

- Plan 002 (dead pallet removal)
- Plan 003 (pallet naming normalization)
- Plan 004 (inherited migration cleanup)
- Plan 005 (stale document cleanup)
