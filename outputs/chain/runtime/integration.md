# `chain:runtime` Integration — Restart Slice 1

## Workspace Integration

- `myosu-runtime` is now a first-class workspace member rooted at `crates/myosu-chain/runtime/`.
- The root workspace dependency table now carries the minimal runtime dependency set pinned to `polkadot-sdk` `stable2407`.
- `Cargo.lock` now includes the resolved runtime dependency graph needed for this new member.

## Runtime Surface Integration

- The previous `crates/myosu-chain/runtime/src/lib.rs` was a non-buildable subtensor forward-port with unresolved imports across `subtensor_*`, Frontier, drand, and chain-specific pallets.
- This slice replaces that surface with a minimal FRAME runtime that only composes:
  - `System`
  - `Timestamp`
  - `Balances`
  - `Sudo`
- The runtime exposes the core runtime APIs plus `sp_genesis_builder::GenesisBuilder`.

## Genesis / Chain-Spec Integration

- `crates/myosu-chain/runtime/src/chain_spec.rs` now provides a named `development` runtime preset hook.
- The current preset is intentionally minimal (`{}` patch over the default runtime genesis config). It is enough to prove the runtime-side preset plumbing exists without prematurely coupling this slice to node-side chain-spec decisions.

## Downstream Boundary

- `crates/myosu-chain/common/` is still outside the active compile path for this slice.
- `crates/myosu-chain/node/` is still outside the active compile path for this slice.
- No subtensor-derived pallets were pulled into the runtime composition in this slice.

## Next Integration Step

The next `chain:runtime` slice should choose one of these paths:

1. Finish runtime build proof in an environment that can satisfy the new dependency fetches.
2. Flatten the runtime dependency surface further if fully-offline proof is a hard requirement here.
3. Once the runtime build proof is green, start Phase 2 by wiring `common` and `node` against this minimized runtime surface.
