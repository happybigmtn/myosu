# `chain:runtime` Integration — Restart Slice 1 Fixup

## Active Slice Boundary

- the integrated runtime surface is still Phase 1 only:
  `System`, `Timestamp`, `Balances`, `Sudo`
- `crates/myosu-chain/node/` and `crates/myosu-chain/common/` remain outside the
  active proof boundary
- Phase 2 node proof must not be run as part of this slice

## Proof Integration Contract

- the runtime spec now carries a dedicated "current approved slice proof"
  section so the active gate is unambiguous
- the Phase 1 proof commands now use
  `env CARGO_TARGET_DIR=.raspberry/cargo-target ...` because this sandbox’s
  default shared Cargo target directory is read-only
- the future Phase 2 node proof is described as future work instead of an active
  command for this lane

## Runtime Surface Integration

- `crates/myosu-chain/runtime/src/lib.rs` remains the minimal FRAME runtime
  introduced in Slice 1
- `crates/myosu-chain/runtime/src/chain_spec.rs` remains the runtime-side preset
  hook for `sp_genesis_builder`
- no subtensor-derived pallets were reintroduced during this fixup

## Next Integration Step

Run the Phase 1 proof commands to completion in a clean verification pass. Only
after those finish green should the lane widen toward `myosu-node`.
