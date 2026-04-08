# COMPLETED

- `DEBT-001` Relocated the live RPC and runtime-api crates from
  `pallets/subtensor/` to `pallets/game-solver/`, rewired workspace/runtime
  path dependencies, and updated the chain support tool's runtime-api path.
  Validation: `SKIP_WASM_BUILD=1 cargo check --workspace`; `cargo test -p pallet-game-solver --quiet -- stage_0`.
  Commit: `2066e0e`
