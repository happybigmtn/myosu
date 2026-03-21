# `chain:runtime` Verification — Slice 1

## Proof Commands That Passed

| Command | Exit Code | Result |
|---------|-----------|--------|
| `CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain-common` | 0 | Common crate compiles |
| `CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain` | 0 | Chain marker crate compiles |
| `WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-runtime` | 0 | Runtime crate compiles, including nested Wasm build |
| `WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo build --offline --release -p myosu-runtime` | 0 | Release runtime build completes |

## Artifact Evidence

The release build produced Wasm artifacts at:

- `/tmp/myosu-chain-target/release/wbuild/myosu-runtime/myosu_runtime.wasm`
- `/tmp/myosu-chain-target/release/wbuild/myosu-runtime/myosu_runtime.compact.wasm`
- `/tmp/myosu-chain-target/release/wbuild/myosu-runtime/myosu_runtime.compact.compressed.wasm`

## Risks Reduced

- The runtime lane is no longer a dead code scaffold. `myosu-runtime` is now a real package in the workspace.
- The Wasm build path is now proven instead of assumed.
- The preserved common surface no longer depends on missing `subtensor_*` support crates.

## Risks That Remain

- `myosu-node` still does not exist as a buildable package, so there is no devnet proof yet.
- The runtime is intentionally generic. It does not yet include `pallet-game-solver` or any Myosu-specific chain logic.
- The common crate was reduced to the subset needed to compile honestly in this restart slice. If downstream code needs the removed fixed-point or subtensor-era helpers, they must be restored intentionally in a later slice.
- In a network-restricted environment, the runtime proof depends on `WASM_BUILD_WORKSPACE_HINT` and `CARGO_NET_OFFLINE=true` so the nested Wasm build uses the checked-in lockfile and local cache.

## Next Slice

**Slice 2 — Minimal node bring-up**

Create the `myosu-node` package, add the basic CLI/service wiring for `myosu-runtime`, define the dev chain spec, and prove `cargo build --release -p myosu-node` succeeds.

