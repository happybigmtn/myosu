# COMPLETED

- `RT-001` commit `baa9c11951259ee4f2b96dd6fb1f983321ff65eb`; validation: `SKIP_WASM_BUILD=1 cargo check -p myosu-chain-runtime`; `cargo test -p pallet-game-solver stage_0_flow --quiet`; `rustup target add wasm32v1-none`; `cargo build -p myosu-chain-runtime`; `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`
- `RT-002` commit `a1f70c04106bf818b685b33b57a4f317033761bb`; validation: `cargo check -p pallet-admin-utils --quiet`; `SKIP_WASM_BUILD=1 cargo check -p myosu-chain --quiet`; `cargo tree -p myosu-chain --prefix none | rg '^(fc-|fp-|pallet-evm|pallet-ethereum)'`; `SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet`
- `RT-003` commit `550adc735e70767db56b5167669d6b47eac5c944`; validation: `cargo test -p pallet-game-solver stage_0_flow --quiet`; `SKIP_WASM_BUILD=1 cargo test -p myosu-miner -p myosu-validator --quiet`
- `EM-001` commit `0a5273c76d735fc85e50df80218e64492765626c`; validation: `cargo test -p pallet-game-solver coinbase --quiet`; `cargo test -p pallet-game-solver stage_0_flow --quiet`
