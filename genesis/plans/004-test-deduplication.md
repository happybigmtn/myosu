# 004: Test Suite Deduplication

## Objective

After Plan 002 removes the subtensor pallet copy, audit the remaining
`pallet-game-solver` test suite for tests that exercise inherited behavior
no longer reachable in the stage-0 default build. Gate legacy tests behind
`#[cfg(feature = "full-runtime")]` or `#[cfg(feature = "legacy-subtensor-tests")]`.

## Context

The game-solver pallet carries 44 test files inherited from subtensor. Many
test AMM swap behavior, root-network operations, EVM chain ID, coldkey swaps,
and multi-mechanism epochs -- none of which are exercised by the stage-0
reduced extrinsic surface. These tests are currently either:
- Behind `legacy-subtensor-tests` feature gate (partially done)
- Using `#[cfg(not(...))]` to skip in default builds
- Or running against code paths that still compile but are unreachable

## Acceptance Criteria

- Every test file in `pallet-game-solver/src/tests/` is categorized as:
  - **stage-0 active**: runs in default build, exercises live code paths
  - **legacy**: feature-gated, exercises inherited paths not in stage-0 surface
- Stage-0 active tests include at minimum:
  - `stage_0_flow.rs` (existing)
  - `epoch.rs` (Yuma mechanism)
  - `weights.rs` (commit-reveal)
  - `registration.rs` (burned_register)
  - `staking.rs` (add_stake)
  - `serving.rs` (serve_axon)
  - `determinism.rs` (epoch determinism)
- Default `cargo test -p pallet-game-solver --quiet` runs only stage-0 active tests
- `cargo test -p pallet-game-solver --features legacy-subtensor-tests --quiet` runs all
- CI `chain-core` job runs default (stage-0) tests
- Total default-build test time is reduced (measured before and after)

## Verification

```bash
# Stage-0 tests pass
cargo test -p pallet-game-solver --quiet

# Legacy tests still compile and pass with feature
cargo test -p pallet-game-solver --features legacy-subtensor-tests --quiet

# Count active vs gated tests
rg '#\[test\]' crates/myosu-chain/pallets/game-solver/src/tests/ --count
rg '#\[cfg.*legacy' crates/myosu-chain/pallets/game-solver/src/tests/ --count
```

## Dependencies

- 002 (dead code removal) -- must complete before this plan starts, to avoid
  auditing tests against a codebase that still has the duplicate pallet.
