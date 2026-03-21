# Chain Pallet Implementation

Date: 2026-03-20
Lane: `chain:pallet`
Slice: `Phase 2 restore core storage and registration/serving`

## Implemented in this slice

This slice advances the reviewed restart plan from the Phase 1 compile-only
boundary into the next approved cut: a minimal Myosu-owned pallet core with
restored config, core storage, and basic registration / serving dispatchables.

- Extended the pallet `Config` with `RuntimeEvent` and kept the single-token
  `Balance` surface in [lib.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/lib.rs#L147).
- Restored the approved storage subset in [lib.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/lib.rs#L186):
  `Keys`, `Axons`, `Prometheus`, `NeuronCertificates`, `Owner`, `Delegates`,
  `SubnetOwner`, `NextSubnetUid`, and `NextNeuronUid`.
- Added the minimal dispatchable surface in [lib.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/lib.rs#L254):
  `register_subnet`, `register_hotkey`, `serve_axon`, and `serve_prometheus`.
- Replaced the dormant forwarded subnet logic with Myosu-owned helpers in
  [registration.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs#L8)
  and [serving.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/serving.rs#L6).
- Preserved the reviewed trust boundary for hotkey ownership by rejecting
  attempts to bind an already-owned hotkey to a different coldkey in
  [registration.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs#L30).
- Added focused pallet tests in
  [phase2_tests.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/phase2_tests.rs#L44)
  covering subnet registration, hotkey registration, ownership conflicts,
  serving persistence, and invalid endpoint input rejection.

## Touched surfaces

- [lib.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/lib.rs)
- [subnets/mod.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/mod.rs)
- [subnets/registration.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/registration.rs)
- [subnets/serving.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/subnets/serving.rs)
- [phase2_tests.rs](/home/r/.fabro/runs/20260320-01KM71KWW01SKCJTVS36V85KY5/worktree/crates/myosu-chain/pallets/game-solver/src/phase2_tests.rs)

## Remaining blockers before the next slice

- `epoch/math.rs` is still on the reduced checked-arithmetic surface and has not
  yet been restored to the reviewed fixed-point design.
- `staking/` remains outside the active Phase 2 write set.
- The runtime has not yet wired this restored pallet surface into broader chain
  flows beyond standalone compile and pallet-local tests.
