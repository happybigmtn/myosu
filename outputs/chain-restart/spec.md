# Chain Restart — Specification

**Plan ID:** 007
**Status:** Completed
**Date:** 2026-04-03

---

## Purpose

Re-establish a buildable, runnable Substrate-based chain binary (`myosu-chain`) that wires `pallet-game-solver` (aliased as `pallet-subtensor`) as the primary runtime pallet alongside the base Substrate/Polkadot SDK consensus layer.

---

## What Was Done

### 1. Workspace Members Restored

The root `Cargo.toml` workspace members now include the three chain sub-packages:

```toml
"crates/myosu-chain/pallets/game-solver",  # Stage 0: Core pallet (CF-06)
"crates/myosu-chain/runtime",             # Runtime packaging
"crates/myosu-chain/node",                # Node binary packaging
```

Note: The parent directory `crates/myosu-chain/` is NOT a workspace member — it is a directory grouping only. The `myosu-chain` binary is produced by `crates/myosu-chain/node` which has `name = "myosu-chain"` in its `Cargo.toml`.

### 2. `codec` Dependency Fixed Across All Pallets

**Problem:** Multiple pallets used `codec = { workspace = true, features = ["derive"] }` which references the workspace alias `codec = { package = "parity-scale-codec", version = "3.7.5", default-features = false }`. This failed during derive macro expansion because the `parity-scale-codec-derive` proc-macro generates `parity_scale_codec::` paths that cargo cannot resolve when the crate is aliased as `codec`.

**Fix:** Changed to explicit `codec = { package = "parity-scale-codec", version = "3.7.5", default-features = false, features = ["derive"] }` in all affected pallets:

- `pallets/game-solver`
- `pallets/admin-utils`
- `pallets/swap` and sub-modules
- `pallets/swap-interface`
- `pallets/subtensor` and sub-modules
- `pallets/drand`
- `runtime/Cargo.toml`

### 3. Runtime `Cargo.toml` Made Standalone

**Problem:** The runtime's `Cargo.toml` used `workspace = true` for all dependencies. When `substrate-wasm-builder` builds the runtime for wasm32v1-none, it copies the `Cargo.toml` to a temporary wbuild directory and compiles it in isolation — without the root workspace context. This caused "workspace.dependencies was not defined" errors.

**Fix:** The runtime's `Cargo.toml` now uses explicit dependency URLs/revisions instead of `workspace = true`, making it fully self-contained:

- Git-based deps use full `git = "...", rev = "71629fd93b6c12a362a5cfb6331accef9b2b2b61"` form
- Path-based deps use explicit `path = ".."` form
- Package metadata (`license`, `edition`) is hardcoded, not inherited

### 4. Binary Verification

```bash
cargo check -p myosu-chain   # passes (0 errors)
cargo build -p myosu-chain  # produces binary (1.1G)
./target/debug/myosu-chain --help  # runs correctly
```

---

## Runtime Architecture

The runtime (`myosu-chain-runtime`) uses `construct_runtime!` with these pallets:

| Index | Pallet | Notes |
|-------|--------|-------|
| 0 | `frame_system` | System frame |
| 1 | `pallet_insecure_randomness_collective_flip` | Only with `full-runtime` |
| 2 | `pallet_timestamp` | Aura timestamp |
| 3 | `pallet_aura` | Aura consensus |
| 4 | `pallet_grandpa` | Grandpa consensus |
| 5 | `pallet_balances` | Balances |
| 6 | `pallet_transaction_payment` | Tx payment |
| 7 | `pallet_subtensor` | **Aliased from `pallet-game-solver`** |
| 11 | `pallet_utility` | Utility |
| 12–16 | Sudo, Multisig, Preimage, Scheduler, Proxy | Only with `full-runtime` |
| 19 | `pallet_admin_utils` | Admin utilities |
| 20 | `pallet_safe_mode` | Only with `full-runtime` |

---

## Key Design Decisions

### Decision: Alias `pallet-subtensor` → `pallet-game-solver`

The runtime uses `pallet_subtensor = { package = "pallet-game-solver", path = "../pallets/game-solver" }`. This means:
- The on-chain module name is `SubtensorModule`
- The pallet code comes from `game-solver`
- Other pallets (staking, emission) are NOT wired — they exist in `pallets/subtensor` but are not integrated

### Decision: Stage-0 No-Op Swap

`Stage0NoopSwap` is the `SwapHandler` implementation used at runtime initialization. It performs 1:1 identity swaps (TAO↔Alpha) with zero fees. This is intentional: subnet pricing, emission, and staking math all flow through this swap surface, which must be replaced before mainnet.

### Decision: `full-runtime` Feature Gate

The runtime has a `full-runtime` feature that enables:
- `pallet-sudo`
- `pallet-multisig`
- `pallet-preimage`
- `pallet-scheduler`
- `pallet-proxy`
- `pallet-safe-mode`
- `pallet-insecure-randomness-collective-flip`

Without `full-runtime`, only Aura/Grandpa consensus + Balances + TransactionPayment + SubtensorModule + Utility + AdminUtils are active.

---

## Remaining Work

1. **Verify block production** — Start a local devnet and confirm block authoring works
2. **Fix WASM build** — The wasm build fails when `substrate-wasm-builder` runs in isolation (without cached wasm). The runtime's `Cargo.toml` was made standalone to address this, but full verification requires a clean build.
3. **Wire remaining pallets** — `admin-utils`, `commitments`, `registry`, `swap`, `transaction-fee` are defined in the runtime but their `Config` traits may need runtime implementation
4. **Devnet genesis** — Generate a proper chain spec with initial validators and balances
5. **Replace Stage-0 swap** — The no-op swap must be replaced with a real AMM before any public deployment
