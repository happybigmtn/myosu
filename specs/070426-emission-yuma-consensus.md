# Specification: Emission & Yuma Consensus

## Objective

Describe the emission distribution mechanism, Yuma Consensus weight processing, epoch pass mechanics, the Stage0NoopSwap identity pricing stub, and the emission dust policy surface that plan 007 targets.

## Evidence Status

### Verified (code-grounded)

- The core emission and consensus logic lives in the `pallet-game-solver` crate (aliased as `pallet_subtensor` in the runtime) at `crates/myosu-chain/pallets/game-solver/src/`.
- The pallet lib.rs is 2,868 lines / 96.7K (`crates/myosu-chain/pallets/game-solver/src/lib.rs`).
- Epoch logic is in `crates/myosu-chain/pallets/game-solver/src/epoch/`.
- Emission accounting uses `U96F32` and `I96F32` fixed-point types from `substrate-fixed` v0.6.0.
- Emission truncation: `U96F32→u64` floor conversion loses up to 2 RAO per block per the planning corpus (plan 007).
- `migrate_init_total_issuance` migration runs on every runtime upgrade to clean tiny floating-point rounding errors (`runtime/src/lib.rs:1303-1309`).
- The try-state alert delta for total issuance is 1,000 RAO (plan 007 corpus reference).
- `Stage0NoopSwap` (`runtime/src/lib.rs:99-120`) provides identity swap (1:1 TAO↔Alpha, zero fees). All subnet pricing, emission, and staking math flows through this swap surface.
- Stage-0 uses a single-token model (TAO only; Alpha emissions are identity-mapped through the noop swap).
- Validators submit weights on-chain; Yuma Consensus processes these weights during the epoch pass to compute incentive and dividend distribution.
- The E2E smoke test (`crates/myosu-chain/node/tests/stage0_local_loop.rs`) verifies that `alice_miner_incentive`, `bob_validator_dividend`, and `alice_miner_emission` are emitted and non-zero after a full stage-0 loop.
- The pallet-level proof command is `cargo test -p pallet-game-solver stage_0_flow --quiet` (`README.md:120`).
- INV-003 (INVARIANTS.md:33-41): Validator exploitability scores must agree within epsilon < 1e-6 for identical inputs. Violation is severity S0 — emissions freeze until determinism is restored.
- Zero-fee staking in stage-0 (all stake operations are free per Stage0NoopSwap).

### Recommendations (intended future direction)

- Plan 007 (emission dust policy) proposes three options for the U96F32→u64 truncation dust:
  1. Accept 2 RAO/block loss and tighten the try-state alert threshold.
  2. Accumulate dust in a dedicated treasury/dust account.
  3. Round-robin rounding (allocate the residual to a rotating recipient).
- Plan 014 (token economics research gate) will determine: single vs dual token, AMM type, fee model, registration cost, emission schedule, staking mechanics, cross-subnet flow, governance utility.
- The `Stage0NoopSwap` `default_price_limit()` returning `C::MAX` is intentionally unbounded for stage-0 and must be bounded before production.

### Hypotheses / Unresolved

- The exact epoch period (number of blocks between epoch passes) is configured per-subnet and was reported as ~13.6 seconds in the planning corpus, but the exact block count has not been verified from code.
- Whether emission dust accumulates monotonically or is reset by the `migrate_init_total_issuance` migration on each upgrade.
- The interaction between the dust policy decision and the total issuance cleanup migration.

## Acceptance Criteria

- After a full stage-0 local loop, at least one miner receives non-zero emission
- After a full stage-0 local loop, at least one validator receives non-zero dividend
- Total issuance drift (minted vs accounted) stays within the try-state alert delta
- `Stage0NoopSwap` returns `amount_paid_in == amount_paid_out` for any swap call
- The epoch pass produces weight-proportional incentive/dividend splits for the participating neurons
- The `stage_0_flow` pallet test passes: `cargo test -p pallet-game-solver stage_0_flow --quiet`

## Verification

```bash
# Pallet-level stage-0 flow test
cargo test -p pallet-game-solver stage_0_flow --quiet

# Full E2E smoke test (requires built node binary)
SKIP_WASM_BUILD=1 cargo test -p myosu-chain --test stage0_local_loop --quiet

# Verify fixed-point dependency
grep 'substrate-fixed' Cargo.toml

# Verify swap stub returns identity
grep -A10 'fn swap' crates/myosu-chain/runtime/src/lib.rs
```

## Open Questions

- What is the exact epoch period (block count) for subnet 7 on devnet?
- Does the `migrate_init_total_issuance` migration fully reset dust each upgrade, or does it only clean errors above a threshold?
- If dual-token economics are adopted (plan 014), does the emission accounting need to track two separate issuance totals?
- How does emission accounting behave when a subnet has zero registered neurons?
