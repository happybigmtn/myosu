# `chain:runtime` Verification — Phase 0 Slice 1

## Automated Proof Commands

| Command | Exit Code | Outcome |
|---------|-----------|---------|
| `cargo metadata --format-version 1 --no-deps` | 0 | Passed. The workspace now resolves with `crates/myosu-chain` as an admitted member. |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo check -p myosu-chain` | 0 | Passed. The new workspace anchor package builds cleanly. |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo check -p myosu-games` | 0 | Passed. Existing non-chain workspace members still check after the workspace wiring change. |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo check -p pallet-game-solver` | 101 | Failed out of scope. The pallet still has pre-existing unresolved `subtensor_*`, `safe_math`, `substrate_fixed`, `log`, and newer Substrate API mismatches, which is consistent with the restart review. |

Note: `cargo check` had to be run with a local `CARGO_TARGET_DIR` because the
default shared target directory is read-only in this sandbox.

## What This Slice Now Guarantees

- The root workspace no longer treats the chain lane as a commented future
  placeholder; it has a real `myosu-chain` package entry.
- The owned manifest surfaces that were missing at restart time now exist:
  `crates/myosu-chain/Cargo.toml`,
  `crates/myosu-chain/runtime/Cargo.toml`,
  `crates/myosu-chain/common/Cargo.toml`, and
  `crates/myosu-chain/node/Cargo.toml`.
- The root workspace now carries the reviewed `polkadot-sdk stable2407`
  dependency line for the minimal runtime pallet set.

## Residual Risks

- `myosu-runtime`, `myosu-chain-common`, and `myosu-node` are still seeded
  manifests only. Their source trees have not been rewritten to match the
  minimal restart shape yet.
- `cargo build -p myosu-runtime --release` is still not meaningful from the root
  workspace because `crates/myosu-chain/runtime` has not been admitted as a
  direct workspace member yet.
- `pallet-game-solver` remains broken outside this slice, which means the wider
  chain subtree still cannot be trusted as buildable despite the Phase 0
  manifest recovery.

## Next Slice

**Phase 1 Slice 2 — rewrite `crates/myosu-chain/runtime/src/lib.rs` down to the
minimal reviewed runtime, add the dependencies/build wiring that runtime
actually needs, and add `crates/myosu-chain/runtime` as a direct workspace
member so the lane proof `cargo build -p myosu-runtime --release` can run from
the workspace root.**
