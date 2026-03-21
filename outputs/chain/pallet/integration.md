# Chain Pallet Integration

Date: 2026-03-20
Lane: `chain:pallet`
Slice: `Phase 2 restore core storage and registration/serving`

## Integration effect

`pallet-game-solver` now exposes a coherent minimal runtime surface instead of a
compile-only shell. The pallet can allocate subnet ids, register hotkeys with
stable coldkey ownership, and persist serving metadata for registered hotkeys.

## What downstream work can rely on now

- Runtime integration can build against the restored pallet `Config`, events,
  errors, and core storage in
  [lib.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/lib.rs#L147).
- Pallet callers can use the new registration and serving extrinsics in
  [lib.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/lib.rs#L254).
- Subnet logic is now Myosu-owned in
  [registration.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs#L8)
  and [serving.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/serving.rs#L6),
  with no revived `subtensor_*` workspace dependencies.

## Next integration blocker

The next approved blocker remains Phase 3: restore `epoch/math.rs` to the
reviewed fixed-point shape so later staking, subnet, and emission slices have
the math surface they depend on.
