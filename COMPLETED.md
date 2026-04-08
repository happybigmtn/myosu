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
  Commit: `75414e5`

- `DEBT-003` Renamed the live runtime pallet surface from
  `pallet_subtensor` / `SubtensorModule` to `pallet_game_solver` / `GameSolver`,
  updated the chain client's hard-coded storage prefix and call/event variants,
  renamed the relocated RPC crates to `game-solver-rpc*`, and fixed local/devnet
  genesis patch builders to emit `gameSolver` instead of the stale
  `subtensorModule` key. The truthful alias proof is an exact-word search because
  the repo still intentionally contains distinct pallets such as
  `pallet_subtensor_proxy`, `pallet_subtensor_swap`, and `pallet_subtensor_utility`.
  Validation: `rg -n '\\bpallet_subtensor\\b|\\bSubtensorModule\\b|subtensorModule' crates/myosu-chain crates/myosu-chain-client Cargo.toml`; `SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime -p myosu-chain`; `cargo test -p pallet-game-solver --quiet -- stage_0`; `cargo clean -p myosu-chain-runtime && SKIP_WASM_BUILD=1 cargo build -p myosu-chain-runtime --quiet`; `SKIP_WASM_BUILD=1 cargo run -p myosu-chain --quiet -- build-spec --chain devnet --raw > /dev/null`.
  Commit: `b59351c`
