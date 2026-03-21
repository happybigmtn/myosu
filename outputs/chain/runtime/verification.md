# `chain:runtime` Verification — Restart Slice 1 Fixup

## Automated Proof Commands

| Command | Exit | Outcome |
|---------|------|---------|
| `env CARGO_TARGET_DIR=.raspberry/cargo-target cargo check -p myosu-runtime` | 0 | Passed in `8.87s`. The approved Phase 1 runtime-only proof command completed successfully in the sandbox-local target directory. |
| `env CARGO_TARGET_DIR=.raspberry/cargo-target cargo build -p myosu-runtime --release` | 0 | Passed in `13.22s`. The release build completed successfully and produced the runtime WASM outputs. |

## Observed Artifacts

The release proof emitted these runtime artifacts:

| Path | Size |
|------|------|
| `.raspberry/cargo-target/release/wbuild/myosu-runtime/myosu_runtime.wasm` | `906767` bytes |
| `.raspberry/cargo-target/release/wbuild/myosu-runtime/myosu_runtime.compact.wasm` | `859513` bytes |
| `.raspberry/cargo-target/release/wbuild/myosu-runtime/myosu_runtime.compact.compressed.wasm` | `211415` bytes |

## Verification Notes

- The active slice proof is Phase 1 only and does not include `myosu-node`.
- The sandbox-local `CARGO_TARGET_DIR=.raspberry/cargo-target` override remains
  required for honest, reproducible proof in this environment.
- After removing the unused `alloc::vec` import from the runtime crate, the
  local runtime proof completed without a crate-local warning.
- Both commands still emit Cargo's future-incompatibility note for upstream
  dependency `trie-db v0.29.1`; this did not block the approved proof gate and
  no local fix was made in this slice.

## Slice Result

The first approved proof gate for `chain:runtime` is now unblocked and green on
the approved commands.
