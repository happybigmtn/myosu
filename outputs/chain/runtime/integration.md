# `chain:runtime` Integration — Phase 1 Slice 2

## Integrated In This Slice

- The root-admitted `myosu-runtime` crate now owns a concrete minimal runtime
  implementation instead of a forwarded, non-compiling subtensor surface.
- The runtime build surface is small and explicit:
  - `src/lib.rs` defines the minimal pallet composition and preserved domain
    types.
  - `Cargo.toml` matches the code that now exists.
  - `build.rs` keeps the include contract for `WASM_BINARY*` constants without
    pulling the full WASM optimizer stack into this slice.

## Intentionally Not Integrated Yet

- Real runtime WASM artifact generation.
- `myosu-node` as a workspace member or proof target.
- `myosu-chain-common` cleanup and re-export work.
- Any subtensor-derived pallet reintegration.

## Cross-Lane Notes

- This slice removes a setup-only runtime blocker without claiming progress on
  the chain node or pallet restart lanes.
- The runtime proof boundary is now honest enough for downstream work to build
  on: later slices can restore WASM output and node APIs without first undoing a
  broken forwarded runtime body.

## Next Integration Boundary

The next runtime integration step is to restore real WASM output and the
runtime APIs needed by the node boundary, then admit `myosu-node` only when its
own proof command is truthful from the workspace root.
