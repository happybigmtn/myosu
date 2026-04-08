# COMPLETED

- `DEBT-001` Relocated the live RPC and runtime-api crates from
  `pallets/subtensor/` to `pallets/game-solver/`, rewired workspace/runtime
  path dependencies, and updated the chain support tool's runtime-api path.
  Validation: `SKIP_WASM_BUILD=1 cargo check --workspace`; `cargo test -p pallet-game-solver --quiet -- stage_0`.
  Commit: `2066e0e`

- `DEBT-002` Deleted the dead `crates/myosu-chain/pallets/subtensor/` tree,
  removed the root workspace dependency on `pallet-subtensor`, and dropped the
  old pallet path from the chain support version-bump tool.
  Validation: `test ! -d crates/myosu-chain/pallets/subtensor`; `SKIP_WASM_BUILD=1 cargo check --workspace`; `cargo test -p pallet-game-solver --quiet -- stage_0`; `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`; `if grep -rq 'pallets/subtensor' crates/; then exit 1; fi`.
  Commit: `4d7415c`
