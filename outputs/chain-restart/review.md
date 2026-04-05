# Chain Restart — Review

**Plan:** 007-chain-restart
**Date:** 2026-04-03

---

## Outcome

The stage-0 goal is met: `cargo check -p myosu-chain` and `cargo build -p myosu-chain` both pass. A binary exists at `target/debug/myosu-chain`.

---

## Verification Results

| Check | Result |
|---|---|
| `cargo check -p myosu-chain` | ✅ PASS — 10 warnings (8 in pallet-game-solver, 2 in runtime) |
| `cargo build -p myosu-chain` | ✅ PASS — binary at `target/debug/myosu-chain` (1.1 GB) |
| `game-solver` in runtime | ✅ Wired as `SubtensorModule` index 7 |
| `admin-utils` in runtime | ✅ Wired as index 19 |
| `pallet_subtensor` alias | ✅ Correctly points to `pallet-game-solver` |

---

## Corrections to Prior Output

The inventory phase incorrectly stated that `crates/myosu-chain/pallets/admin-utils` is a workspace member. It is **not** listed in `workspace.members` in `Cargo.toml`. It works because `pallet-admin-utils` is declared as a workspace-level dependency; the runtime references it with `pallet-admin-utils.workspace = true`. This is an intentional pattern — the pallet is available as a dep without being a top-level workspace target.

---

## Issues

### 1. Pre-existing Warnings in `pallet-game-solver` (8 warnings, not fixed)

These are dead-code and unused-import warnings in `pallet-game-solver/src/` and `pallet-game-solver/src/macros/dispatches.rs`. They predate this session and do not affect correctness.

| # | Warning |
|---|---|
| 1 | Unused imports: `StorePreimage`, `UnfilteredDispatchable`, `dispatch::GetDispatchInfo` |
| 2 | Unused import: `frame_support::traits::schedule::v3::Anon` |
| 3 | Unused import: `sp_core::ecdsa::Signature` |
| 4 | Unused import: `sp_runtime::traits::Hash` |
| 5 | Unused imports: `MAX_NUM_ROOT_CLAIMS`, `MAX_ROOT_CLAIM_THRESHOLD`, `MAX_SUBNET_CLAIMS` |
| 6 | Unused import: `QueryPreimage` |
| 7 | Unused import: `Saturating` |
| 8 | `do_recycle_alpha`, `do_burn_alpha`, `do_add_stake_burn` are never called |

### 2. Runtime Unused Imports (2 warnings, fixed)

Two `use` statements in `runtime/src/lib.rs` imported types only used under `#[cfg(feature = "full-runtime")]`, causing warnings in the default build:
- `fungible::HoldConsideration` — used only by `pallet_preimage::Config`
- `EnsureRoot`, `EnsureRootWithSuccess` — used only by full-runtime pallet configs

**Fix:** Removed the unused import branches. Confirmed build still passes.

---

## Open Questions

### Remaining Pallets: Wire or Defer?

`commitments`, `registry`, and `swap` pallets exist and compile individually but are not in `construct_runtime!`. The stage-0 binary is functional without them. The question of whether to wire them belongs to the next planning cycle.

---

## Next Steps

1. **Verify block production** — run `./target/debug/myosu-chain --dev --rpc-port 9933` and confirm block authoring
2. **Decide on remaining pallets** — wire `commitments`/`registry`/`swap` or explicitly defer to a later phase
3. **Clean pallet-game-solver warnings** — remove dead imports and functions (low priority, no correctness impact)
4. **Full-runtime build** — `cargo build -p myosu-chain --features full-runtime` for a production-equivalent binary
