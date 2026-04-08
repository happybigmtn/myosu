# 002 — Dead Pallet Removal

## Objective

Delete the original `pallet-subtensor` crate (90.6K lines) that exists alongside the active `pallet-game-solver`. The runtime already uses game-solver exclusively (aliased as `pallet_subtensor`). The original pallet is dead code that confuses search, bloats compilation, and risks accidental linking.

## Context

The runtime Cargo.toml contains:
```
pallet_subtensor = { package = "pallet-game-solver", path = "../pallets/game-solver" }
```

The original `crates/myosu-chain/pallets/subtensor/` is NOT referenced by any Cargo.toml in the workspace default build. It exists only as a reference copy.

Both pallets have near-identical test suites (106 staking tests each, 69 coinbase tests each, etc.). The game-solver version has stage-0 modifications (stage_0_flow.rs, determinism.rs, swap_stub.rs).

## Acceptance Criteria

- `crates/myosu-chain/pallets/subtensor/` directory is deleted from the working tree
- `cargo check --workspace` succeeds (with SKIP_WASM_BUILD=1)
- `cargo test -p pallet-game-solver --quiet -- stage_0` passes
- `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet` passes
- No remaining Cargo.toml references `pallet-subtensor` package (the original, not the alias)
- `grep -r "pallets/subtensor" crates/` returns zero results (excluding git history)

## Verification

```bash
# Confirm directory deleted
test ! -d crates/myosu-chain/pallets/subtensor

# Confirm workspace compiles
SKIP_WASM_BUILD=1 cargo check --workspace

# Confirm stage-0 tests pass
cargo test -p pallet-game-solver --quiet -- stage_0
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet

# Confirm no stale references
! grep -rq "pallets/subtensor" crates/myosu-chain/*/Cargo.toml
```

## Dependencies

- None. This is the highest-leverage independent action.
