# `chain:runtime` Verification — Phase 0

## Proof Commands Run

| Command | Exit Code | Outcome |
|---------|-----------|---------|
| `cargo metadata --no-deps --format-version 1` | 0 | Workspace resolves `myosu-chain`, `myosu-runtime`, `myosu-chain-common`, and `myosu-node`; `workspace_default_members` remains limited to `myosu-games`, `myosu-tui`, `pallet-game-solver`, and `myosu-chain`. |
| `CARGO_TARGET_DIR=/tmp/myosu-chain-phase0-anchor cargo build -p myosu-chain --release --offline` | 0 | The new chain anchor crate builds successfully from the repo root. |
| `CARGO_TARGET_DIR=/tmp/myosu-chain-phase0-runtime cargo build -p myosu-runtime --release --offline` | 101 | Infrastructure failure only: `/tmp` hit `Disk quota exceeded` while compiling dependencies. This did not indicate a manifest or package-resolution problem. |
| `CARGO_TARGET_DIR=/home/r/.cache/rust-tmp/myosu-chain-phase0-runtime-check cargo check -p myosu-runtime --offline` | 101 | Reaches `crates/myosu-chain/runtime/src/lib.rs` and fails on inherited source blockers, proving the package is now wired correctly. |

## Code-Level Blockers Confirmed By `cargo check`

The runtime package no longer fails with "package not found." The remaining
errors are the Phase 1 blockers described in the reviewed restart plan:

- missing runtime-owned modules:
  `check_nonce`, `migrations`, `sudo_wrapper`,
  `transaction_payment_wrapper`
- missing WASM build path:
  `env!("OUT_DIR")` / `wasm_binary.rs`
- unresolved subtensor/frontier-era imports:
  `pallet_subtensor`, `pallet_shield`, `pallet_evm`, `pallet_ethereum`,
  `subtensor_runtime_common`, `subtensor_swap_interface`,
  `subtensor_transaction_fee`, `subtensor_precompiles`, and related crates
- stale API usage in the inherited runtime source:
  `sp_runtime::Cow`, `sp_runtime::traits::ExtrinsicCall`

## Risks Reduced

- The lane now has canonical package identities and manifests instead of a
  commented-out subtree.
- The root workspace can target `myosu-runtime` and `myosu-node` by package
  name, which unblocks slice-by-slice runtime work.
- The next runtime slice can focus on source replacement instead of more Cargo
  graph surgery.

## Risks Still Present

- `myosu-runtime` does not compile yet; the source is still the inherited
  subtensor-era runtime.
- `myosu-node` and `myosu-chain-common` are discoverable but not yet validated.
- `Cargo.lock` now includes the new dependency line, but the runtime will keep
  failing until the minimal runtime source and WASM build path are implemented.

## Next Slice

**Phase 1 — Minimal Runtime**

Keep the work inside `crates/myosu-chain/runtime/` and replace the current
runtime source with the reviewed minimal runtime plus build-script/WASM wiring.
