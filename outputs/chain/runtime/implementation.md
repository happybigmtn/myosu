# `chain:runtime` Implementation — Restart Slice 1 Fixup

## Slice Completed

This fixup stays inside the approved Phase 1 runtime-only slice and closes its
first proof gate.

The implementation surface remains:

- `crates/myosu-chain/runtime/src/lib.rs`
- `crates/myosu-chain/runtime/src/chain_spec.rs`
- `outputs/chain/runtime/{implementation,verification,integration}.md`

## Concrete Changes

| File | Change |
|------|--------|
| `crates/myosu-chain/runtime/src/lib.rs` | Removed the unused `alloc::vec` import so the approved proof commands complete without a local runtime warning. |
| `outputs/chain/runtime/implementation.md` | Recorded the final fixup scope and the exact owned surfaces touched. |
| `outputs/chain/runtime/verification.md` | Replaced the prior in-progress note with the completed proof commands and their exit-0 outcomes. |
| `outputs/chain/runtime/integration.md` | Updated the slice integration note to reflect the now-proven Phase 1 runtime boundary. |

## Proof Gate Unblocked

The approved Phase 1 proof commands now complete successfully:

```bash
env CARGO_TARGET_DIR=.raspberry/cargo-target cargo check -p myosu-runtime
env CARGO_TARGET_DIR=.raspberry/cargo-target cargo build -p myosu-runtime --release
```

The release build also emits the expected runtime WASM artifacts under the
sandbox-local target directory, including:

- `.raspberry/cargo-target/release/wbuild/myosu-runtime/myosu_runtime.wasm`
- `.raspberry/cargo-target/release/wbuild/myosu-runtime/myosu_runtime.compact.wasm`
- `.raspberry/cargo-target/release/wbuild/myosu-runtime/myosu_runtime.compact.compressed.wasm`

## Slice Boundary Preserved

- No node work was added.
- No common-crate work was added.
- No subtensor-derived pallets were reintroduced.

Per fixup instructions, `outputs/chain/runtime/quality.md` was not hand-authored
and `outputs/chain/runtime/promotion.md` was not created or rewritten.
