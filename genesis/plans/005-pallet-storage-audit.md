# 005: Pallet Storage Audit and Reduction

## Objective

Inventory all ~194 storage items in `pallet-game-solver`, identify the ~80
that are used by the stage-0 extrinsic surface, and feature-gate or remove
the remainder. This reduces on-chain state size, simplifies the genesis
config, and makes the pallet auditable.

## Context

`AGENTS.md` notes "~194 total, ~80 needed" for storage items. The inherited
subtensor pallet declares storage for EVM association rate limits, Alpha AMM
pool state, root-network claims, coldkey swap queues, and other surfaces
that are feature-gated out of the stage-0 extrinsic surface but still
declared in storage.

Declared-but-unused storage items:
- Consume genesis config space
- Appear in runtime metadata (confusing for client developers)
- Create migration burden for future runtime upgrades

## Acceptance Criteria

- A storage audit document `ops/storage-audit-stage0.md` listing every
  `#[pallet::storage]` item with its status: `active`, `inherited-dormant`,
  or `removed`
- Dormant storage items are gated behind `#[cfg(feature = "full-runtime")]`
- The default-build runtime metadata exposes ≤100 storage items for the
  game-solver pallet
- The genesis config for devnet chain spec only sets active storage items
- Existing stage-0 tests pass after the reduction
- The storage audit document includes a machine-readable count

## Verification

```bash
# Pallet compiles
SKIP_WASM_BUILD=1 cargo check -p pallet-game-solver

# Runtime compiles
SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime --features fast-runtime

# Stage-0 tests pass
cargo test -p pallet-game-solver stage_0_flow --quiet

# Storage count check (manual, then documented)
rg '#\[pallet::storage\]' crates/myosu-chain/pallets/game-solver/src/ --count
```

## Dependencies

- 002 (dead code removal) -- subtensor copy must be gone first
- 003 (emission hardening) -- emission paths must be tested before gating storage
- 004 (test deduplication) -- test suite must be clean before verifying against reduced storage
