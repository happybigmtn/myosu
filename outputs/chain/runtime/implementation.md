# `chain:runtime` Implementation — Restart Slice 1

## Slice Implemented

**Restart Slice 1 — Runtime workspace wiring plus a minimal `myosu-runtime` crate**

This slice implements the smallest honest restart step that turns the runtime into a real workspace package and replaces the dead subtensor-derived `runtime/src/lib.rs` with a minimal FRAME runtime surface.

The implemented scope is:

- add `crates/myosu-chain/runtime` as a root workspace member
- pin the minimal runtime dependency set to `polkadot-sdk` `stable2407`
- create `crates/myosu-chain/runtime/Cargo.toml`
- create `crates/myosu-chain/runtime/build.rs`
- replace `crates/myosu-chain/runtime/src/lib.rs` with a four-pallet runtime:
  `System`, `Timestamp`, `Balances`, `Sudo`
- add `crates/myosu-chain/runtime/src/chain_spec.rs` with a runtime-side `development` preset hook for `sp_genesis_builder`

This deliberately does **not** attempt the Phase 2 node/common port. The node and common crates remain outside this slice.

## Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Added `crates/myosu-chain/runtime` to workspace members and introduced the minimal runtime dependency set pinned to `stable2407` |
| `Cargo.lock` | Expanded to include the newly resolved runtime dependency graph |
| `crates/myosu-chain/runtime/Cargo.toml` | New runtime manifest for `myosu-runtime` |
| `crates/myosu-chain/runtime/build.rs` | Added WASM builder hook |
| `crates/myosu-chain/runtime/src/lib.rs` | Replaced the non-buildable subtensor runtime with a minimal FRAME runtime plus runtime APIs |
| `crates/myosu-chain/runtime/src/chain_spec.rs` | Added runtime preset plumbing for `GenesisBuilder` |

## Proof Commands For This Lane

The intended proof commands for this slice remain:

```bash
cargo check -p myosu-runtime
cargo build -p myosu-runtime --release
```

In this sandbox, I also used lower-level proof commands that do not require a full online dependency fetch:

```bash
cargo metadata --offline --no-deps --format-version 1 --manifest-path crates/myosu-chain/runtime/Cargo.toml
rustfmt --check crates/myosu-chain/runtime/build.rs crates/myosu-chain/runtime/src/chain_spec.rs crates/myosu-chain/runtime/src/lib.rs
```

The exact outcomes are recorded in `outputs/chain/runtime/verification.md`.

## What Remains Next

| Next slice | Description |
|-----------|-------------|
| Slice 2 | Get `cargo check -p myosu-runtime` and `cargo build -p myosu-runtime --release` fully green in an environment with the required cached/online registry access, or flatten dependencies further if offline proof is mandatory |
| Slice 3 | Bring `crates/myosu-chain/common/` into the workspace and rewrite its `freeze_struct` / `runtime_common` dependencies per the review boundary |
| Slice 4 | Add the `myosu-node` manifest and begin the Phase 2 node wiring against this minimal runtime |

## Stage-Owned Outputs

Per lane ownership rules, `outputs/chain/runtime/quality.md` and `outputs/chain/runtime/promotion.md` were **not** hand-authored in this implementation stage.
