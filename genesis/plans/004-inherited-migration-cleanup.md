# 004 — Inherited Migration Cleanup

## Objective

Remove or gate the 44 inherited subtensor-era migration files from `pallet-game-solver`. These migrations reference chain state that myosu has never had (Alpha/TAO dual-token, root network, EVM, specific subnet IDs, etc.). They add ~15K lines of dead code that compiles on every build.

## Context

The `crates/myosu-chain/pallets/game-solver/src/migrations/` directory contains 44 migration files, all inherited from the Bittensor subtensor pallet. Examples:

- `migrate_rao.rs` — RAO token migration (myosu uses different denomination)
- `migrate_delete_subnet_21.rs` — Deletes a specific Bittensor subnet
- `migrate_init_tao_flow.rs` — Initializes TAO flow (myosu single-token model)
- `migrate_fix_root_subnet_tao.rs` — Fixes root network state (myosu has no root network in stage-0)
- `migrate_commit_reveal_v2.rs` — CRV2 migration (may still be relevant)

The runtime `Migrations` type in `runtime/src/lib.rs` references a subset of these. Some are in the active migration tuple; most are just compiled source.

## Acceptance Criteria

- Migrations that reference chain state myosu never had are either deleted or moved behind the `legacy-subtensor-tests` feature gate
- The `Migrations` type in `runtime/src/lib.rs` contains only migrations that are relevant to myosu's fresh-genesis chain state
- Each retained migration has a one-line comment explaining why it is retained
- `cargo test -p pallet-game-solver --quiet -- stage_0` passes
- `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --features fast-runtime,try-runtime --quiet devnet_runtime_upgrade_smoke_test_passes_on_fresh_genesis` passes
- Total migration file count reduced by at least 30 (from 44)

## Verification

```bash
# Count remaining migration files
ls crates/myosu-chain/pallets/game-solver/src/migrations/migrate_*.rs | wc -l
# Should be <= 14

# Confirm stage-0 tests
cargo test -p pallet-game-solver --quiet -- stage_0

# Confirm migration smoke test
env -u SKIP_WASM_BUILD cargo build -p myosu-chain-runtime --features fast-runtime --quiet
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --features fast-runtime,try-runtime --quiet devnet_runtime_upgrade_smoke_test_passes_on_fresh_genesis
```

## Dependencies

- Plan 002 (dead pallet removal) — removing the original pallet first avoids confusion about which migrations directory to clean
