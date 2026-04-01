# Chain Pallet Restart Spec

**Lane**: `chain:pallet`
**Date**: 2026-03-27
**Status**: Compile restored, real extension/guard/RPC surfaces restored, and runtime-integrated as a side-by-side pallet

---

## Current State Inventory

Fresh runtime/node restoration temporarily invalidated the earlier pallet proof, but the lane is
green from the repo root again after the manifest was collapsed onto the shared workspace graph.
It is also now configured directly in the runtime as `GameSolver`.

### What exists now

| Surface | Path | Current reality |
|---------|------|-----------------|
| Pallet crate | `crates/myosu-chain/pallets/game-solver/` | Large FRAME pallet with many carried-over subtensor modules |
| Stub surfaces | `src/stubs.rs`, `src/swap_stub.rs` | Local no-op replacements already exist and are worth preserving |
| Main pallet file | `src/lib.rs` | Still the main salvageable pallet surface after restoring aliases, config shape, and missing commit storage |
| Migration tree | `src/migrations/` | Reduced to Stage 0 no-op helpers for restart stability |
| Extension tree | `src/extensions/` | Real game-solver transaction-extension implementation restored and active again |
| Guard tree | `src/guards/` | Real coldkey-swap dispatch guard restored and active again |
| RPC info tree | `src/rpc_info/` | Real RPC payload types are active again and now back the runtime API crate |

### Verified current state

`cargo check -p pallet-game-solver --quiet` on 2026-03-27 now passes again from the repo root.

The crate did move through an honest earlier recovery:

- temporarily stripping the old extension and guard surfaces to no-op placeholders
- zeroing the migration hook path
- restoring a real `Config` section
- restoring the real extension, guard, and RPC-info surfaces once the pallet was stable enough to
  carry them again
- reconstructing the local shared chain crates the pallet still expects
- restoring the missing swap aliases and timelocked commit storage
- explicitly degrading the crowdloan leasing entry point instead of pretending it still works

The restored opentensor workspace dependency spine did change the active repo-root truth, and the
earlier pallet proof had to be re-established on that same line. The shortest honest path was to
collapse this crate's manifest onto the shared workspace graph and restore its missing `std`
feature edge for `pallet-balances`. After that, the next shortest honest path was to configure the
crate directly in `runtime/src/lib.rs` and add `GameSolver: pallet_game_solver = 31` to
`construct_runtime!`.

---

## Restart Boundary

The restart begins at the point where the crate becomes an honest Myosu pallet
instead of a partially carried-over subtensor fork.

That means:

1. the crate's dependency line matches the restored runtime/node workspace
2. `cargo check -p pallet-game-solver` exits successfully from the repo root on that same line
3. the active extension, guard, and RPC-info surfaces are the real game-solver implementations
   rather than local placeholders
4. crowdloan leasing is explicitly degraded instead of silently half-wired

---

## Salvageable Inputs

These surfaces are still worth preserving:

- `src/stubs.rs`
- `src/swap_stub.rs`
- the overall module organization (`staking/`, `subnets/`, `guards/`, `epoch/`)
- self-contained domain types already present in the crate

The crate is not empty scaffolding. It has authored material worth reducing and
repairing rather than deleting wholesale.

---

## Must-Strip or Must-Rewrite Inputs

The following shapes are currently not trustworthy as-is:

- the earlier placeholder versions of `src/extensions/` and `src/guards/`
- the historical migration tree as an active blocker
- optional runtime-common adapters that are not part of the active restart boundary
- any assumption that compile restoration alone means the runtime already exposes this pallet

---

## Next Implementation Slices

### Slice 1: Preserve the restored pallet compile boundary
Keep the recovered pallet green on the same workspace line as the restored runtime/node crates.

Proof target:

```bash
cargo check -p pallet-game-solver --quiet
```

### Slice 2: Preserve the restored runtime/node baseline
Keep the now-runnable chain baseline honest while pallet work catches up.

Proof target:

```bash
cargo metadata --format-version 1 --no-deps
cargo check -p myosu-chain-runtime --quiet
cargo check -p myosu-chain --quiet
```

### Slice 3: Final cutover
Compile restoration, real extension/guard recovery, and first runtime integration are no longer the
blockers inside this lane. The next honest question is how this crate relates to `SubtensorModule`
now that both live in the runtime together.

Proof target:

```bash
rg -n "GameSolver: pallet_game_solver|SubtensorModule: pallet_subtensor" crates/myosu-chain/runtime/src/lib.rs
```
