# 002: Dead Code Removal -- Eliminate pallet-subtensor Copy

## Objective

Remove the `crates/myosu-chain/pallets/subtensor/` directory and its associated
RPC/runtime-api crates. This directory is a complete copy of the original
subtensor pallet that was forked into `pallet-game-solver`. Both copies exist
in parallel with identical test suites (44 test files each). The subtensor
copy is not referenced by the default-build runtime.

## Context

When `pallet-game-solver` was created by forking `pallet-subtensor`, the
original was preserved as a reference. That reference value has been consumed.
The game-solver pallet has diverged (reduced extrinsics, swap stub, stage-0
defaults) and the subtensor copy now serves only as confusion surface.

## Acceptance Criteria

- `crates/myosu-chain/pallets/subtensor/` directory is deleted
- `crates/myosu-chain/pallets/subtensor/rpc/` directory is deleted
- `crates/myosu-chain/pallets/subtensor/runtime-api/` directory is deleted
- Workspace `Cargo.toml` no longer references `pallet-subtensor`, `subtensor-custom-rpc`, or `subtensor-custom-rpc-runtime-api` as workspace members
- Workspace dependency declarations for these crates are removed
- `SKIP_WASM_BUILD=1 cargo check --workspace` succeeds
- `cargo test -p pallet-game-solver --quiet` passes (game-solver tests are authoritative)
- No file in the remaining codebase imports from `pallet_subtensor`

## Verification

```bash
# Confirm deletion
test ! -d crates/myosu-chain/pallets/subtensor

# Workspace compiles
SKIP_WASM_BUILD=1 cargo check --workspace

# Authoritative pallet tests pass
cargo test -p pallet-game-solver stage_0_flow --quiet

# No dangling references
rg 'pallet.subtensor' --type rust | grep -v '#.*legacy\|//.*subtensor'
```

## Dependencies

- None. This is an independent cleanup that reduces risk for all subsequent plans.

## Estimated Scope

Deletion of ~150K lines of duplicated source. The risk is that some downstream
crate has an undocumented dependency on the subtensor copy. The verification
step (`cargo check --workspace`) catches this.
