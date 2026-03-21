# `chain:runtime` Verification — Restart Slice 1

## Automated Commands Run

| Command | Exit | Outcome |
|---------|------|---------|
| `cargo metadata --offline --no-deps --format-version 1 --manifest-path crates/myosu-chain/runtime/Cargo.toml` | 0 | Passed. Cargo recognizes `myosu-runtime` as a workspace package with `myosu_runtime` lib and `build-script-build` targets. |
| `rustfmt --check crates/myosu-chain/runtime/build.rs crates/myosu-chain/runtime/src/chain_spec.rs crates/myosu-chain/runtime/src/lib.rs` | 0 | Passed. The new runtime files parse and format cleanly. |
| `cargo check -p myosu-runtime` | 101 | Blocked by sandbox network restrictions. Cargo attempted to update `crates.io` and failed to resolve `index.crates.io`. |
| `cargo check -p myosu-runtime --offline` | 101 | Blocked by missing cached registry artifacts for the newly resolved dependency graph. First missing crate reported: `finality-grandpa v0.16.3`. |
| `cargo check -p myosu-runtime --no-default-features --offline` | 101 | Same blocker as above. The failure happens before Rust type-checking reaches the crate body. |

## What The Passing Checks Prove

- The root workspace now includes `myosu-runtime`.
- The runtime manifest is syntactically valid and discoverable by Cargo.
- Cargo can resolve the runtime package metadata shape offline.
- The new runtime source files parse cleanly enough for `rustfmt`.

## What Is Not Yet Proven

- `cargo check -p myosu-runtime` reaching Rust type-checking in this sandbox
- `cargo build -p myosu-runtime --release` producing the WASM artifact under `target/release/wbuild/`

## Verification Boundary

The current verification boundary is environmental, not yet semantic:

- the first failure is dependency acquisition, not a Rust compiler error in `crates/myosu-chain/runtime/src/lib.rs`
- because of that, this run cannot honestly claim the runtime crate is fully build-green yet

## Recommended Follow-Up Proof

Run these in an environment with the required registry access or complete local crate cache:

```bash
cargo check -p myosu-runtime
cargo build -p myosu-runtime --release
```

If the sandbox must remain fully offline, the next slice should either vendor/patch the missing registry crates or reduce the runtime dependency surface until the offline cache is sufficient.
