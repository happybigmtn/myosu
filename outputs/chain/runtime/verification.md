# `chain:runtime` Verification — Slice 1

## Automated Proof Commands That Ran

| Command | Exit Code | Outcome | Notes |
|---------|-----------|---------|-------|
| `./fabro/checks/chain-runtime-reset.sh` | 0 | Passed | Fabro/Raspberry proof entrypoint now runs the Slice 1 build checks and Wasm artifact assertions end to end, while forcing a writable target dir instead of inheriting the sandbox's read-only ambient `CARGO_TARGET_DIR` |
| `CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain-common` | 0 | Passed | Emits a future-incompat warning for `trie-db v0.29.1` |
| `CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-chain` | 0 | Passed | Marker crate compiles cleanly |
| `WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo check --offline -p myosu-runtime` | 0 | Passed | Runtime crate compiles, including nested Wasm build setup |
| `WASM_BUILD_WORKSPACE_HINT="$PWD" CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/tmp/myosu-chain-target cargo build --offline --release -p myosu-runtime` | 0 | Passed | Release build completed and produced Wasm artifacts |

## Verification Scope

This fixup verified only the approved Phase 1 runtime slice. It did not run any
node package or devnet proof, because those belong to the next slice.

## Wasm Artifact Evidence

The release build produced these non-zero artifacts under
`/tmp/myosu-chain-target/release/wbuild/myosu-runtime/`:

- `myosu_runtime.wasm` — 954500 bytes
- `myosu_runtime.compact.wasm` — 904887 bytes
- `myosu_runtime.compact.compressed.wasm` — 220087 bytes

## Risks Reduced

- The runtime lane is no longer a dead code scaffold. `myosu-runtime` is a real workspace package with a repeatable release build.
- The Wasm build path is now proven instead of assumed.
- Fabro and Raspberry now have an executable runtime proof entrypoint instead of a path-existence stub.
- The preserved common surface no longer depends on missing `subtensor_*` support crates.
- The current slice proof is now explicit and bounded to runtime-owned surfaces instead of bleeding into future node work.

## Risks That Remain

- `myosu-node` is still not a workspace package, so there is no node binary or devnet proof yet.
- `crates/myosu-chain/node/src/` remains scaffold code without a verified manifest or service wiring.
- The runtime is intentionally generic. It does not yet include `pallet-game-solver` or any Myosu-specific chain logic.
- The common crate was reduced to the subset needed to compile honestly in this restart slice. If downstream work needs removed fixed-point or subtensor-era helpers, they must be restored intentionally later.
- The proof logs still emit a future-incompat warning for `trie-db v0.29.1`.
- In this network-restricted environment, the runtime proof depends on `WASM_BUILD_WORKSPACE_HINT` and `CARGO_NET_OFFLINE=true` so the nested Wasm build uses the checked-in lockfile and local cache.
- The proof script must override the shell's ambient `CARGO_TARGET_DIR`, which points at a read-only `.raspberry` path in this sandbox.

## Next Slice

**Slice 2 — Minimal node bring-up**

Create the `myosu-node` manifest and basic CLI/service wiring for the existing
runtime, add the dev chain spec, and verify the node binary in a dedicated
follow-on slice.
