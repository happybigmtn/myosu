# `chain:runtime` Verification — Phase 1 Slice 2

## Automated Proof Commands

All commands were run from the workspace root with an explicit
`CARGO_TARGET_DIR=/tmp/myosu-cargo-target` because the default shared target
path is read-only in this environment.

| Command | Exit Code | Outcome |
|---------|-----------|---------|
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo build -p myosu-runtime --release` | 0 | Passed. The admitted runtime crate now builds in release mode from the workspace root. |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo check -p myosu-runtime` | 0 | Passed. The minimal runtime source type-checks in the dev profile. |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-runtime --lib` | 0 | Passed. Runtime unit tests and generated integrity tests both passed. |

## What This Slice Now Guarantees

- `myosu-runtime` is a real, root-buildable package instead of a manifest-only
  placeholder.
- The runtime now composes only the reviewed minimal pallet set:
  `System`, `Timestamp`, `Balances`, and `Sudo`.
- The runtime preserves the reviewed `NetUid` domain type and proves its compact
  SCALE encoding round trip in tests.

## Residual Risks

- `build.rs` currently emits a dummy `wasm_binary.rs`, so this slice does **not**
  claim that the full runtime WASM artifact path is restored yet.
- `myosu-node` is still outside the current slice and is not admitted as a
  direct workspace proof surface.
- Node-facing runtime APIs, genesis presets, and chain-spec integration remain
  future work.

## Notes From Proof Execution

- The earlier `wasm-opt-sys` disk-quota failure is no longer on the proof path.
- Cargo still reports a future-incompatibility warning in `trie-db v0.29.1`;
  that warning did not block this slice’s proof commands.
