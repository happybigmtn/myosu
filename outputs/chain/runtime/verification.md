# `chain:runtime` Verification — Phase 1

## Automated Proof Commands Run

| Command | Exit Code | Outcome |
|---------|-----------|---------|
| `CARGO_TARGET_DIR=/home/r/.cache/rust-tmp/myosu-runtime-phase1 CARGO_NET_OFFLINE=true cargo check -p myosu-runtime --offline` | 0 | Runtime host build passes with the wasm build path enabled. |
| `CARGO_TARGET_DIR=/home/r/.cache/rust-tmp/myosu-runtime-phase1 CARGO_NET_OFFLINE=true cargo build -p myosu-runtime --release --offline` | 0 | Optimized runtime build passes and emits the wasm artifacts under the external target dir. |
| `ls -lh /home/r/.cache/rust-tmp/myosu-runtime-phase1/release/wbuild/myosu-runtime/myosu_runtime.wasm /home/r/.cache/rust-tmp/myosu-runtime-phase1/release/wbuild/myosu-runtime/myosu_runtime.compact.wasm /home/r/.cache/rust-tmp/myosu-runtime-phase1/release/wbuild/myosu-runtime/myosu_runtime.compact.compressed.wasm` | 0 | Confirms the generated runtime blobs exist and are non-zero: `888K`, `842K`, and `207K` respectively. |

## Proof Notes

### Environment-specific adjustment

The proof commands were run with:

- `CARGO_TARGET_DIR` set to a writable cache root inside the sandbox
- `CARGO_NET_OFFLINE=true` plus `--offline` so the nested wasm build stayed on
  the cached dependency graph

Without the runtime build-script fix, the wasm builder could not locate the
workspace `Cargo.lock` from that external target dir and attempted forbidden
network fetches. The new `WASM_BUILD_WORKSPACE_HINT` setup eliminated that
failure mode.

### Artifacts Produced

Release wasm outputs were generated at:

- `/home/r/.cache/rust-tmp/myosu-runtime-phase1/release/wbuild/myosu-runtime/myosu_runtime.wasm`
- `/home/r/.cache/rust-tmp/myosu-runtime-phase1/release/wbuild/myosu-runtime/myosu_runtime.compact.wasm`
- `/home/r/.cache/rust-tmp/myosu-runtime-phase1/release/wbuild/myosu-runtime/myosu_runtime.compact.compressed.wasm`

## Residual Warnings

Both successful Cargo commands emitted the same future-incompatibility warning:

- `trie-db v0.29.1` contains code that a future Rust version will reject

This warning does **not** block the phase-1 proof gate for the current
toolchain, but it should be tracked when the chain fork is rebased or the Rust
toolchain moves forward.

## Scope Boundary Confirmed

This verification intentionally stops at the phase-1 runtime gate:

- `myosu-runtime` now builds and produces wasm artifacts
- `myosu-node` was not re-verified in this slice because node wiring is phase 2
- `myosu-chain-common` was not rewritten in this slice because its cleanup is
  outside the approved phase-1 runtime scope
