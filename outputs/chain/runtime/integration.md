# `chain:runtime` Integration — Phase 0 Slice 1

## Integrated In This Slice

- The root workspace now recognizes the chain runtime lane through the new
  `myosu-chain` anchor package.
- The runtime-owned manifest surfaces now exist at the paths called out by the
  restart spec:
  - `crates/myosu-chain/runtime/Cargo.toml`
  - `crates/myosu-chain/common/Cargo.toml`
  - `crates/myosu-chain/node/Cargo.toml`

## Intentionally Not Integrated Yet

- `myosu-runtime` is **not** yet a direct workspace member.
- `myosu-chain-common` is **not** yet wired into any admitted package.
- `myosu-node` is **not** yet wired into the workspace build graph.
- No subtensor-derived pallets were touched in this slice.

This boundary is deliberate. The reviewed lane artifacts require Phase 0
workspace honesty first, while the actual runtime rewrite belongs to the next
approved slice.

## Cross-Lane Notes

- The failed `pallet-game-solver` check confirms there is still pallet-side
  breakage outside `chain:runtime`. This slice records that evidence rather than
  trying to repair pallet surfaces owned by a different lane.
- The root `polkadot-sdk stable2407` dependency line now matches the working
  dependency family already used by `pallet-game-solver`, which reduces drift
  before the Phase 1 runtime rewrite.

## Next Integration Boundary

The next runtime integration step is to admit `crates/myosu-chain/runtime` as a
direct workspace member at the same time its source is rewritten to the minimal
restart runtime. Doing those together keeps the root build graph honest and
avoids surfacing the current broken subtensor-derived runtime as though it were
already buildable.
