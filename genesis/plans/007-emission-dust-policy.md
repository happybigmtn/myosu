# 007 — Emission Dust Policy

## Objective

Decide and implement a dust-handling policy for the stage-0 coinbase truncation. Currently, U96F32 → u64 floor conversion loses up to 2 rao per accrued block (6 rao per default tempo-2 epoch). This dust silently disappears.

## Context

The `tou64!` macro in `run_coinbase.rs` uses `saturating_to_num::<u64>()` which floors. The truncation sweep test (`stage_0_coinbase_truncation_drift_stays_below_two_rao_per_block_sweep`) measures the loss but the repo has no policy for what happens to it.

WORKLIST.md `EM-DUST-001` defers this decision. The `try_state` threshold is set to 1000 rao intentionally above the measured drift to avoid false positives.

Three options:

1. **Accept and document** — The dust is small enough to ignore. Document the bound. Tighten the `try_state` threshold to ~10 rao per epoch.
2. **Accumulate in a dust account** — Track truncation remainder in a storage item. Periodically distribute or burn.
3. **Round-robin rounding** — Alternate between floor and ceil across distribution targets to keep the sum exact.

## Acceptance Criteria

- One of the three options (or a fourth if discovered) is chosen and documented in an ADR
- If option 1: `try_state` threshold tightened from 1000 rao to a value justified by the measured drift
- If option 2 or 3: implementation exists with unit tests proving sum(distributions) == block_emission * epochs within 1 rao
- WORKLIST.md `EM-DUST-001` is resolved (either closed or replaced by a follow-on)
- `cargo test -p pallet-game-solver -- truncation --quiet` still passes
- `bash tests/e2e/emission_flow.sh` still passes

## Verification

```bash
# Confirm dust policy test
cargo test -p pallet-game-solver -- truncation --quiet

# Confirm E2E emission
bash tests/e2e/emission_flow.sh

# Confirm ADR exists
test -f docs/adr/011-emission-dust-policy.md
```

## Dependencies

- Plan 006 (Phase 1 gate) — clean codebase before touching emission internals
