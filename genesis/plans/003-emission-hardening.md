# 003: Emission Accounting Hardening

## Objective

Prove the emission accounting invariant (`sum(distributions) == block_emission * epochs`)
with explicit test coverage, and simplify the inherited coinbase/epoch code to
remove dead paths that the stage-0 single-token identity-swap model cannot exercise.

## Context

The inherited `run_coinbase.rs` (~80+ lines visible, hundreds more in helpers)
and `run_epoch.rs` (~80+ lines of Yuma mechanism) carry logic for:
- Root-network weighted emission distribution across subnets
- AMM-based Alpha/TAO token conversion in emission
- Multi-mechanism epoch processing (MechId variants)
- Protocol TAO liquidity adjustment

Under the stage-0 identity-swap model (`NoOpSwap`), all token conversions are
1:1. The root-network and AMM paths produce the same result as direct distribution,
but they add complexity without adding correctness signal.

## Acceptance Criteria

- A new test `emission_accounting_invariant` in `stage_0_flow.rs` that:
  - Creates a subnet with known tempo
  - Registers miners and validators with known stakes
  - Advances blocks through multiple epochs
  - Asserts `sum(all_distributions) == block_emission * epoch_count` within tolerance
  - Asserts no tokens are created or destroyed during distribution
- Dead coinbase code paths gated behind `#[cfg(feature = "full-runtime")]`:
  - Root-network emission weighting (subnets already filter `!= NetUid::ROOT`)
  - AMM liquidity adjustment calls (no-op under identity swap)
  - Multi-mechanism epoch variants beyond `MechId::MAIN`
- The `emission_flow.sh` E2E test is updated to assert the accounting invariant
  on the live devnet, not just successful block production

## Verification

```bash
# Unit test
cargo test -p pallet-game-solver emission_accounting_invariant --quiet

# Existing stage-0 tests still pass
cargo test -p pallet-game-solver stage_0_flow --quiet

# E2E emission flow
bash tests/e2e/emission_flow.sh
```

## Dependencies

- 002 (dead code removal) -- removing the subtensor copy first avoids confusion
  about which pallet's tests are authoritative.
