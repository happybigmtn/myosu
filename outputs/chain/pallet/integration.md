# Chain Pallet Integration

Date: 2026-03-20
Lane: `chain:pallet`
Slice: `Phase 1 restart scaffolding`

## Integration effect

The `pallet-game-solver` crate is back to a clean standalone compile and test
state with a Myosu-owned module graph. This gives the restart lane a stable
base for the next approved pallet slices instead of continuing to patch the
subtensor forward-port in place.

## Surfaces now safe to build on

- The pallet exports local `NetUid`, `Balance`, `Currency`, `Hyperparameter`,
  and `RateLimitKey` types.
- `AxonInfo`, `PrometheusInfo`, and `NeuronCertificate` remain available as
  clean pallet-owned data types.
- `stubs.rs` and `swap_stub.rs` stay live and verified.
- The reduced rate-limit helpers can be extended without depending on any
  `subtensor_*` workspace crates.

## Still pending before deeper chain integration

- Restore the minimal Myosu `Config` surface needed by the runtime.
- Reintroduce pallet storage beyond rate limiting.
- Restore registration / serving dispatchables.
- Replace the `epoch/math.rs` placeholder with real fixed-point logic.
- Wire staking and subnet modules back in slice-by-slice.

## Stage ownership note

`quality.md` is owned by the Quality Gate and `promotion.md` is owned by the
Review stage. They were intentionally not hand-authored in this implementation
slice.
