# `chain:runtime` Integration — Phase 1

## Workspace Integration

The `myosu-runtime` package now integrates into the repo as a buildable,
phase-1 runtime rather than a placeholder for the inherited subtensor fork.

- it stays on the reviewed `polkadot-sdk stable2407` line
- it builds from the repo root as `-p myosu-runtime`
- it emits the expected wasm runtime blobs through the standard
  `substrate-wasm-builder` flow

## Runtime Boundary After This Slice

The runtime crate is now the stable base for downstream chain work:

- the runtime surface is limited to standard FRAME pallets only
- the subtensor/frontier dependency graph is no longer on the phase-1 build
  path
- the build script is aware of external target dirs via
  `WASM_BUILD_WORKSPACE_HINT`, which keeps the wasm build reproducible in the
  current Fabro sandbox layout

## Downstream Contract

After this slice:

- phase-2 node work can target a real `myosu-runtime` crate instead of a broken
  inherited runtime definition
- phase-2 common-crate cleanup can proceed independently of the runtime build
  path
- no additional root-workspace surgery should be required to keep advancing the
  chain runtime lane
