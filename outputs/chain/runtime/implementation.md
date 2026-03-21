# `chain:runtime` Implementation — Phase 0

## Slice Implemented

**Phase 0 — Workspace Wiring**

This slice made the chain restart surfaces honest Cargo packages without
rewriting the inherited runtime, common, or node source. The workspace now
recognizes `myosu-chain`, `myosu-runtime`, `myosu-chain-common`, and
`myosu-node` as real package identities, which moves the restart boundary from
"package not found" to the actual source-level blockers that Phase 1 is meant
to replace.

## What Changed

### Root workspace activation

`Cargo.toml` now:

- activates `crates/myosu-chain` as a workspace member
- keeps `default-members` limited to the previously active crates plus the new
  `myosu-chain` anchor crate
- pins the baseline `polkadot-sdk` `stable2407` dependency line for the
  minimal runtime surface in `[workspace.dependencies]`

### Chain anchor crate

Added:

- `crates/myosu-chain/Cargo.toml`
- `crates/myosu-chain/src/lib.rs`

This crate is intentionally tiny. Its job is to give the root workspace a
stable chain entrypoint while keeping the downstream restart crates addressable
by package name.

### Explicit manifests for restart-owned crates

Added:

- `crates/myosu-chain/runtime/Cargo.toml`
- `crates/myosu-chain/common/Cargo.toml`
- `crates/myosu-chain/node/Cargo.toml`

These manifests establish the canonical package names and minimal dependency
baselines for the approved restart plan:

- `myosu-runtime`
- `myosu-chain-common`
- `myosu-node`

The runtime manifest also defines empty `runtime-benchmarks` and `try-runtime`
feature flags so verification output reflects the real blockers rather than
manifest noise.

### Lockfile update

`Cargo.lock` was refreshed by offline Cargo resolution so the new workspace
shape and runtime dependency line are recorded in the repo.

## What This Slice Guarantees Now

- `cargo metadata` resolves the chain subtree as concrete packages.
- `cargo build -p myosu-chain --release --offline` succeeds from the repo root.
- `cargo check -p myosu-runtime --offline` now reaches
  `crates/myosu-chain/runtime/src/lib.rs` and fails on the inherited
  subtensor-era runtime source itself.

## What Remains For The Next Slice

**Phase 1 — Minimal Runtime**

The next approved slice should stay inside `crates/myosu-chain/runtime/` and
replace the inherited runtime definition with the reviewed minimal runtime:

- remove the missing local modules (`check_nonce`, `migrations`,
  `sudo_wrapper`, `transaction_payment_wrapper`) unless the minimal runtime
  actually needs replacements
- add the real WASM build path (`build.rs` / `wasm_binary.rs`)
- replace subtensor/frontier imports with the approved minimal pallet set:
  `frame_system`, `pallet_balances`, `pallet_sudo`, `pallet_timestamp`,
  `frame_executive`

## Stage-Owned Artifacts Not Touched Here

- `outputs/chain/runtime/quality.md` remains quality-gate owned.
- `outputs/chain/runtime/promotion.md` remains review-stage owned.
