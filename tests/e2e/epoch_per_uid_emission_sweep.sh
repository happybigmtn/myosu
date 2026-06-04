#!/usr/bin/env bash
# NEM-003A: epoch per-UID emission accumulation sweep test.
#
# `epoch_mechanism` (in
# `crates/myosu-chain/pallets/game-solver/src/epoch/run_epoch.rs`) computes
# per-UID `server_emission`, `validator_emission`, and `combined_emission` as
# `I96F32` fixed-point values and then truncates them to `u64` independently
# per UID. The per-UID truncation drops the fractional rao (each fraction is
# strictly less than 1 rao), so the sum of the per-UID combined emissions
# can be at most `n_neurons` rao short of the requested `rao_emission`. The
# sum of the per-UID `server_emission` + `validator_emission` pairs can
# also drift by at most `n_neurons` rao (same bound, two independent
# truncations).
#
# This harness is the executable end-to-end proof that the truncation-bound
# invariant is enforced in code (not just asserted in prose) and that the
# pallet-level unit test that pins it actually runs green.
#
# Four real assertions; failing any one of them fails the gate with a
# concrete error message naming the missing piece.
#
#   1. The pallet-level test file carries the two `epoch_per_uid_*` test
#      functions (`epoch_per_uid_emission_sum_equals_total_within_truncation_bound`
#      for the sweep, and
#      `epoch_per_uid_emission_per_uid_dominance_holds_for_uniform_stake` for
#      the per-UID dominance pin).
#   2. The sweep test exercises a representative grid of `(n_neurons,
#      n_validators, rao_emission)` cells, including the small-emission
#      cells (`rao_emission = 1`, `7`) where truncation can dominate the
#      math.
#   3. The sweep test's per-cell assertions are the documented truncation-
#      bound invariants: `sum(per_uid_combined) <= rao_emission` and
#      `rao_emission - sum(per_uid_combined) <= n_neurons`, plus the same
#      pair for the per-UID `server_emission + validator_emission` sum.
#   4. Both `epoch_per_uid_*` tests pass under `cargo test` (the live
#      executable gate, not just the file presence check).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

stage_0_flow_test="crates/myosu-chain/pallets/game-solver/src/tests/stage_0_flow.rs"
pallet_crate="pallet-game-solver"

fail() {
    printf 'epoch_per_uid_emission_sweep: FAIL %s\n' "$1" >&2
    exit 1
}

# -- 1. Both test functions are present in the pallet test file.
for needle in \
    'fn epoch_per_uid_emission_sum_equals_total_within_truncation_bound' \
    'fn epoch_per_uid_emission_per_uid_dominance_holds_for_uniform_stake'; do
    if ! grep -Fq -- "$needle" "$stage_0_flow_test"; then
        fail "pallet test file is missing the test function: $needle"
    fi
done

# -- 2. The sweep test exercises the small-emission cells where truncation
#       dominates the math. Without `rao_emission = 1` in the grid, the
#       truncation drift for n=1 is exactly 0 (the I96F32 share is exactly
#       1.0 and truncates to 1), which would silently let a bug through.
#       Assert the grid contains 1 and a couple of larger values so the
#       invariant is exercised on the cells where the math is non-trivial.
#       The literal `N_neuron_counts`, `validator_splits`, and `rao_emissions`
#       identifiers pin the grid shape (no anonymous inline construction).
for needle in \
    'let neuron_counts' \
    'let validator_splits' \
    'let rao_emissions'; do
    if ! grep -Fq -- "$needle" "$stage_0_flow_test"; then
        fail "sweep test grid is missing the documented marker: $needle"
    fi
done
# The rao_emissions grid must include 1, 7, 1_000, 1_000_003, and
# 100_000_001 — the small magnitudes are the cells where the truncation
# invariant is tightest.
for needle in \
    'rao_emissions: ' \
    '1, 7, 1_000' \
    '1_000, 1_000_003' \
    '1_000_003, 100_000_001'; do
    if ! grep -Fq -- "$needle" "$stage_0_flow_test"; then
        fail "sweep test rao grid is missing the documented cell: $needle"
    fi
done

# -- 3. The sweep test's per-cell assertions are the documented truncation
#       bounds. The markers below match the exact assertion messages
#       inside `epoch_per_uid_emission_sum_equals_total_within_truncation_bound`.
for needle in \
    'per-UID truncation drift must be <= n_neurons rao' \
    'per-UID truncated sum cannot exceed rao_emission' \
    'per-UID server+validator sum cannot exceed rao_emission' \
    'per-UID server+validator drift must be <= n_neurons rao'; do
    if ! grep -Fq -- "$needle" "$stage_0_flow_test"; then
        fail "sweep test is missing the documented truncation-bound assertion: $needle"
    fi
done

# -- 4. Both tests pass under `cargo test`. The live executable gate:
#       the unit tests are the source of truth, not the file markers.
#       We deliberately do NOT pass `--quiet` here so the per-test
#       `test tests::stage_0_flow::<name> ... ok` lines land in the
#       captured output (the strict name-matching assertions below rely
#       on those exact lines, not on the `..` shorthand `--quiet` emits).
test_output="$(
    env SKIP_WASM_BUILD=1 cargo test -p "$pallet_crate" --no-fail-fast -- \
        epoch_per_uid 2>&1
)"
if ! printf '%s\n' "$test_output" | grep -Eq '^test result: ok\. 2 passed'; then
    fail "pallet epoch_per_uid tests did not pass: $test_output"
fi
# The dominance test is the second test in the filtered run; assert the
# exact name is in the output so a future refactor cannot rename it to
# something that still compiles but loses its semantic content.
if ! printf '%s\n' "$test_output" | grep -Eq '^test tests::stage_0_flow::epoch_per_uid_emission_per_uid_dominance_holds_for_uniform_stake \.\.\. ok'; then
    fail "pallet epoch_per_uid_emission_per_uid_dominance_holds_for_uniform_stake test did not appear as 'ok' in: $test_output"
fi
if ! printf '%s\n' "$test_output" | grep -Eq '^test tests::stage_0_flow::epoch_per_uid_emission_sum_equals_total_within_truncation_bound \.\.\. ok'; then
    fail "pallet epoch_per_uid_emission_sum_equals_total_within_truncation_bound test did not appear as 'ok' in: $test_output"
fi

printf 'EPOCH_PER_UID_EMISSION_SWEEP_HARNESS myosu e2e ok sweep_grid=neurons_x_validators_x_rao truncation_bound=n_neurons_rao tests=epoch_per_uid_emission_{sum_equals_total_within_truncation_bound,per_uid_dominance_holds_for_uniform_stake}\n'
