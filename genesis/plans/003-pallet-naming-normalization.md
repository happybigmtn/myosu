# 003 — Pallet Naming Normalization

## Objective

Eliminate the confusing `pallet_subtensor` alias so the active pallet is consistently named `pallet_game_solver` (or a chosen canonical name) in both code references and Cargo manifests. New contributors should find one pallet with one name.

## Context

After Plan 002 deletes the original pallet-subtensor crate, the Cargo alias remains:
```
pallet_subtensor = { package = "pallet-game-solver", ... }
```

Every file in the runtime and runtime-common imports `pallet_subtensor::`. The `construct_runtime!` macro uses `SubtensorModule: pallet_subtensor = 7`. Storage prefixes are baked into on-chain state.

**Constraint:** The storage prefix in `construct_runtime!` is derived from the pallet name string. Since myosu has no deployed chain state to migrate, renaming the `construct_runtime!` entry is safe for fresh genesis. If there were deployed state, a storage migration would be required.

## Acceptance Criteria

- Runtime Cargo.toml uses `pallet_game_solver = { package = "pallet-game-solver", ... }` (drop the subtensor alias)
- `construct_runtime!` uses `GameSolver: pallet_game_solver = 7` (or equivalent chosen name)
- All `use pallet_subtensor::` imports in runtime, runtime-common, node, and chain-client are updated to the new name
- `grep -r "pallet_subtensor" crates/myosu-chain/` returns only comments explaining the rename, not active code
- All stage-0 tests pass
- Chain spec genesis config updated if pallet name affects genesis builder

## Verification

```bash
# Confirm no active code references old name
grep -rn "pallet_subtensor" crates/myosu-chain/runtime/src/ crates/myosu-chain/node/src/ | grep -v "^.*:.*//.*pallet_subtensor"
# Should return zero results

# Confirm compilation
SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime -p myosu-chain

# Confirm tests
cargo test -p pallet-game-solver --quiet -- stage_0
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet

# Confirm chain spec builds
SKIP_WASM_BUILD=1 cargo run -p myosu-chain --quiet -- build-spec --chain devnet --raw > /dev/null
```

## Dependencies

- Plan 002 (dead pallet removal) — must complete first so there is only one pallet to rename
