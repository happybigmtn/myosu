# `sdk:core` Promotion

## Decision
Promote the lane from Slice 2 (`AC-SDK-03`) to Slice 3 (`AC-SDK-02`).

## Why
- The Slice 2 implementation stayed contained to `crates/myosu-sdk/Cargo.toml` and `crates/myosu-sdk/src/testing/*`.
- The reviewed Slice 2 proof commands now execute real tests at the expected `testing::tests::...` paths and pass.
- A broader `cargo build -p myosu-sdk` and `cargo test -p myosu-sdk` sanity pass also stayed green after the harness changes.

## Remaining Scope
- Scaffold generation remains the next owned `sdk:core` surface to implement and verify.
- Registration and documentation remain outside this completed slice.
