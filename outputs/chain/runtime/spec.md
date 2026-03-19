# Chain Runtime Restart Spec

**Lane**: `chain:runtime`
**Date**: 2026-03-19
**Status**: Restart required

---

## 1. Current State Inventory

### 1.1 What exists

| Surface | Path | Content | Buildable? |
|---------|------|---------|-------------|
| Runtime definition | `crates/myosu-chain/runtime/src/lib.rs` | ~2500 lines; `construct_runtime!` macro, 9 module declarations | **No** — imports `subtensor-runtime-common`, `subtensor-macros`, `subtensor_precompiles`, `subtensor_swap_interface`, `subtensor_transaction_fee`, `pallet_shield`, `pallet_subtensor`, `pallet_subtensor_proxy`, `pallet_subtensor_swap`, `pallet_subtensor_utility`, `pallet_subtensor_swap_runtime_api` — none defined in workspace |
| Node CLI wrapper | `crates/myosu-chain/node/src/main.rs` | One-liner `command::run()` | **No** — module decls for `chain_spec`, `cli`, `client`, `command`, `conditional_evm_block_import`, `consensus`, `ethereum`, `mev_shield`, `rpc`, `service` have no corresponding `.rs` files |
| Currency types | `crates/myosu-chain/common/src/currency.rs` | `AlphaCurrency`, `TaoCurrency` wrappers + `Currency` trait | **No** — imports `subtensor_macros::freeze_struct` (not in workspace) |
| EVM context | `crates/myosu-chain/common/src/evm_context.rs` | `is_in_evm()`, `with_evm_context()` using `environmental` crate | **Yes** — self-contained; only dep is `environmental` (no workspace key needed) |
| Subtensor pallets | `crates/myosu-chain/pallets/{subtensor,shield,swap,registry,...}` | 13 pallets from opentensor/subtensor fork | **No** — each depends on `subtensor-runtime-common` and `subtensor-macros` workspace keys that are undefined |
| Game solver pallet | `crates/myosu-chain/pallets/game-solver/` | Pallet with `frame_support`/`frame_system` from `polkadot-sdk` | **Yes** — only active workspace member under `crates/myosu-chain/` |
| Workspace definition | `Cargo.toml` | `crates/myosu-chain` commented out as `# "crates/myosu-chain" # Stage 1` | N/A |

### 1.2 Missing critical workspace members

The root `Cargo.toml` defines no keys for:

```
subtensor-runtime-common  → no crate, no git, no version
subtensor-macros          → no crate, no git, no version
subtensor-swap-interface  → no crate, no git, no version
subtensor-tools           → defined in support/tools/Cargo.toml but not in workspace members
pallet-subtensor          → defined in pallets/subtensor/Cargo.toml but not in workspace members
... (all 13 pallets)
```

The `runtime/lib.rs` also calls `include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"))` — the WASM binary build path is entirely absent.

### 1.3 Why this is a restart, not a continuation

1. The chain runtime cannot be compiled. The `surface_check` script (`./fabro/checks/chain-runtime-reset.sh`) exits 0 with empty stdout/stderr because it is a no-op stub, not because the runtime builds.
2. The Substrate/polkadot-sdk dependency chain is broken — no `git` refs, no version locks, no `Cargo.lock` entry for the polkadot-sdk fork that the game-solver pallet successfully uses.
3. The node directory is a scaffold with no implementation files.
4. All subtensor-derived pallets reference workspace keys that were never wired into the workspace.

---

## 2. Restart Boundary

The restart begins at **the point where a minimal Substrate runtime compiles in this workspace**.

The only salvageable inputs from the current state are:

- **`crates/myosu-chain/common/src/evm_context.rs`** — `is_in_evm()` / `with_evm_context()` are self-contained. Can be reused verbatim after stripping `#[freeze_struct]` annotations.
- **`crates/myosu-chain/common/src/currency.rs`** — The `Currency` trait and `AlphaCurrency`/`TaoCurrency` types encode domain logic that must be preserved. The `freeze_struct` dependency on `subtensor_macros` must be replaced with a standard `#[derive(...)]` block using the available `scale-info` + `parity-scale-codec` stack.
- **`crates/myosu-chain/pallets/game-solver/`** — Already buildable; establishes the working Substrate/polkadot-sdk `git = "..."` dependency line.
- **The `NetUid` type in `runtime/src/lib.rs`** — Domain type worth preserving; rewrite without `subtensor_macros::freeze_struct`.

---

## 3. Required Manifests

For each phase, the following files must exist before declaring the phase complete:

### Phase 0 — Workspace Wiring

```
Cargo.toml                          # Uncomment crates/myosu-chain, add polkadot-sdk git ref, add all missing workspace keys
crates/myosu-chain/Cargo.toml      # Workspace member manifest for the chain crate
crates/myosu-chain/runtime/Cargo.toml
crates/myosu-chain/node/Cargo.toml
crates/myosu-chain/common/Cargo.toml
```

### Phase 1 — Minimal Runtime

```
crates/myosu-chain/runtime/src/lib.rs          # construct_runtime! with System + Balances + Sudo only
crates/myosu-chain/runtime/src/chain_spec.rs   # Basic chain spec
crates/myosu-chain/runtime/Cargo.toml          # Runtime manifest with all dependencies pinned
```

**Proof**: `cargo build -p myosu-runtime --release` exits 0; `cargo check` on runtime crate passes.

### Phase 2 — Node + Common

```
crates/myosu-chain/node/src/lib.rs             # Full Substrate node service wiring
crates/myosu-chain/node/src/service.rs
crates/myosu-chain/node/src/command.rs
crates/myosu-chain/node/src/rpc.rs
crates/myosu-chain/common/src/lib.rs           # Clean re-exports of currency + evm_context
crates/myosu-chain/common/src/currency.rs       # Rewritten without subtensor_macros
crates/myosu-chain/common/src/evm_context.rs    # Cleaned but preserved
```

**Proof**: `cargo build -p myosu-node --release` exits 0.

---

## 4. Phase Definitions

### Phase 0: Workspace Wiring

**Goal**: Make `crates/myosu-chain` an honest workspace member with all required dependencies.

**Actions**:
1. Add `polkadot-sdk` git dependency to root `Cargo.toml.dependencies` using the same `branch = "stable2407"` that `game-solver` uses.
2. Uncomment `# "crates/myosu-chain"` in workspace members.
3. Add `subtensor-runtime-common`, `subtensor-macros`, `subtensor-swap-interface` as git dependencies pointing to the local `support/macros` and `support/tools` paths, or remove their usage entirely from the pallets.
4. Define `runtime = "some-git-ref"` workspace key pointing at a minimal `myosu-runtime` crate.

**Blocker**: The 13 subtensor pallets cannot be built until their workspace keys are resolved. They are **not** in the critical path for Phase 1.

---

### Phase 1: Minimal Runtime (System + Balances + Sudo)

**Goal**: Produce a real `myosu-runtime` crate that compiles to a `.wasm` and `.compact` blob.

**Runtime composition**:
- `frame_system`
- `pallet_balances`
- `pallet_sudo`
- `pallet_timestamp`
- `frame_executive`
- No custom pallets yet.

**Chain spec**: Single validator, development chain ID.

**Proof shape**:
```
$ cargo build -p myosu-runtime --release
   Compiling myosu-runtime v0.1.0
    Finished release [optimized]
$ ls -la target/release/wbuild/myosu_runtime*.wasm
mysu_runtime.wasm   # exists, non-zero size
```

---

### Phase 2: Node Binary + Common Crate

**Goal**: Produce a `myosu-node` binary that runs the Phase 1 runtime.

**Node composition**:
- Full `sc_service::NewFull` setup
- `BasicAura` or `AuraConsensus` block production
- HTTP + WebSocket RPC
- CLI with `clap`

**Common crate cleanup**:
- Strip all `#[freeze_struct]` uses; replace with standard derive macros
- Re-export `AlphaCurrency`, `TaoCurrency`, `Currency` trait, `NetUid` (rewritten), `is_in_evm`, `with_evm_context`
- Publish as `myosu-chain-common`

**Proof shape**:
```
$ cargo build -p myosu-node --release
   Compiling myosu-node v0.1.0
    Finished release [optimized]
$ ./target/release/myosu-node --help
mysu-node 0.1.0
...
```

---

### Phase 3: Custom Pallets (Future)

**Not in scope for this restart spec.** The 13 subtensor pallets are out-of-scope until the minimal runtime/node is proven.

---

## 5. What Is NOT Salvageable From Current `runtime/src/lib.rs`

| Element | Reason not salvageable |
|---------|------------------------|
| `construct_runtime!` block | Imports pallets that don't exist in workspace |
| `subtensor_precompiles::Precompiles` | No `subtensor_precompiles` crate |
| `pallet_shield`, `pallet_subtensor`, `pallet_subtensor_*` | No workspace definitions |
| `runtime_common::prod_or_fast` | No `runtime_common` crate |
| `subtensor_macros::freeze_struct` | No `subtensor_macros` workspace key |
| `subtensor_swap_interface::{Order, SwapHandler}` | No `subtensor_swap_interface` |
| `subtensor_transaction_fee::{SubtensorTxFeeHandler, TransactionFeeHandler}` | No `subtensor_transaction_fee` |
| `pallet_commitments::{CanCommit, OnMetadataCommitment, GetCommitments}` | Depends on `pallet_subtensor` and `subtensor_runtime_common` |
| `pallet_registry::CanRegisterIdentity` | No `pallet_registry` workspace key |
| WASM binary `include!` | No build path established |

---

## 6. Recommended Restart Approach

1. **Fork no Substrate code on day one.** Start with raw `frame_system` + `pallet_balances` + `pallet_sudo` + `pallet_timestamp` pinned to `polkadot-sdk stable2407`.
2. **Do not attempt to port the subtensor pallets yet.** They bring in a deep dependency graph that will block the build path. Establish the runtime first.
3. **Publish `myosu-chain-common`** as a separate crate early, with clean public API surface.
4. **Do not use `subtensor_macros::freeze_struct`.** Use standard SCALE codec derives from `parity-scale-codec` + `scale-info`.
5. **Build the WASM runtime first.** Node binary is downstream of the runtime.

---

## 7. Review Summary

The current `chain:runtime` effort is a **design document in code form**, not a buildable artifact. The `runtime/src/lib.rs` is a Substrate runtime definition that imports 15+ crates, none of which exist in the workspace. The node directory is a scaffold. The workspace explicitly marks the chain as "Stage 1" and keeps it commented out.

**Salvageable**: `common/src/evm_context.rs`, the `Currency`/currency type idea from `common/src/currency.rs`, `pallets/game-solver/` as a template for the polkadot-sdk dependency line, and `NetUid` as a domain type.

**Not salvageable**: The entire runtime `construct_runtime!` block, all subtensor pallet references, all `subtensor_*` workspace key dependencies.

The next implementation slice must begin at **Phase 0** (workspace wiring) and prove each phase by building it before proceeding.
