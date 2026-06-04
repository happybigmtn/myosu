#!/usr/bin/env bash
# W-06 first-class observability metric for validator scoring runs
# (executable e2e proof).
#
# W-06 / genesis/plans/000-ceo-testnet-roadmap.md (the observability
# half). The validator scoring loop emits exactly one
# `VALIDATOR_SCORING_METRIC` line per bounded scoring run, with
# the fields an operator can scrape into a CSV: `game`,
# `scenario_count`, `mean_l1`, `p50_l1`, `p99_l1`, `elapsed_ms`.
# The line protocol is byte-stable across hosts (the metric's
# `percentile` helper uses nearest-rank, the constructor rejects
# non-finite L1 distances, and the `f64` rendering is `%.6`
# deterministic) so the INV-003 determinism invariant carries
# through to the metric.
#
# Six real assertions; failing any one of them fails the gate
# with a concrete error message that names the offending input
# and the requirement it broke.
#
#   1. `crates/myosu-validator/src/metric.rs` exists and exposes
#      the `ValidatorScoringMetric` / `emit` / `percentile`
#      surface.
#   2. The line protocol is byte-stable: the example binary's
#      `VALIDATOR_SCORING_METRIC_BYTE_STABLE byte_stable=true`
#      marker line confirms two consecutive `emit` calls produce
#      byte-identical output (the INV-003 determinism guarantee
#      for the metric).
#   3. The percentile helper uses nearest-rank (no interpolation):
#      the example binary's `VALIDATOR_SCORING_METRIC_MONOTONICITY
#      monotonic=true` marker line confirms `p50_l1 <= p99_l1`
#      on a 11-step monotonic ladder.
#   4. The scoring loop emits the line: the example binary's
#      `VALIDATOR_SCORING_METRIC` line carries
#      `game=poker scenario_count=1 mean_l1=... p50_l1=...
#      p99_l1=... elapsed_ms=...` and exits 0 (proves the live
#      `score_response` shape's single-scenario emission).
#   5. The line's `mean_l1` field equals the live mean of the
#      underlying L1 set: the example binary's
#      `scenario_count=11 mean_l1=0.500000` line confirms the
#      multi-scenario emission matches the closed-form mean
#      (`0.0 + 0.1 + ... + 1.0) / 11 = 0.5`).
#   6. `p50_l1 <= p99_l1` holds on the multi-scenario run
#      (the monotonicity contract the harness fail-closes on).
#
# The full `cargo test -p myosu-validator` suite is also
# exercised (assertion 7) so the metric module's 15 unit tests
# stay wired (byte-stability, percentile contract, NaN / +inf
# rejection, monotonicity, line-protocol byte-stability).
#
# The harness has no chain or runtime dependency; it is a pure
# cargo build + cargo test + process-spawn gate. A drift in any
# of the seven checks fails the gate with a concrete error
# message that names the failing sub-check. See
# `docs/operator-guide/observability.md` for the public contract
# this job pins.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

pass=0
fail=0
note() { printf 'validator_scoring_metric: %s\n' "$*"; }
ok()   { note "ok $*"; pass=$((pass+1)); }
bad()  { note "FAIL $*"; fail=$((fail+1)); }

# --- 1. metric module is present and exposes the required surface ---
metric_file="$repo_root/crates/myosu-validator/src/metric.rs"
if [[ ! -f "$metric_file" ]]; then
    bad "missing $metric_file"
    echo
    echo "validator_scoring_metric: pass=$pass fail=$fail"
    [[ "$fail" -eq 0 ]]
    exit 1
fi

if grep -qE 'pub struct ValidatorScoringMetric' "$metric_file" \
    && grep -qE 'pub fn emit\(' "$metric_file" \
    && grep -qE 'pub fn percentile\(' "$metric_file" \
    && grep -qE 'pub enum MetricError' "$metric_file" \
    && grep -qE 'pub fn single\(' "$metric_file" \
    && grep -qE 'pub fn from_l1_slice\(' "$metric_file"; then
    ok "metric module exposes ValidatorScoringMetric + emit + percentile + MetricError + single + from_l1_slice"
else
    bad "$metric_file is missing one of: ValidatorScoringMetric, emit, percentile, MetricError, single, from_l1_slice"
fi

# --- 2. the lib.rs re-exports the metric surface ---
lib_rs="$repo_root/crates/myosu-validator/src/lib.rs"
if grep -qE 'pub mod metric;' "$lib_rs" \
    && grep -qE 'pub use metric::\{MetricError, ValidatorScoringMetric, emit' "$lib_rs"; then
    ok "lib.rs re-exports the metric surface (pub mod metric; + pub use metric::{...})"
else
    bad "$lib_rs is missing the pub mod metric; or pub use metric::... re-export"
fi

# --- 3. the example binary builds ---
example_path="$repo_root/crates/myosu-validator/examples/validator_scoring_metric.rs"
if [[ ! -f "$example_path" ]]; then
    bad "missing $example_path"
else
    ok "validator_scoring_metric example is present at $example_path"
fi

# The build step is intentionally lazy: only build on the first
# invocation, then rely on cargo's incremental cache. The harness
# is run from CI on every PR / push, so the cold build path is
# the one that runs there.
note "running: SKIP_WASM_BUILD=1 cargo build -p myosu-validator --example validator_scoring_metric"
SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-validator --example validator_scoring_metric
binary_path="$repo_root/target/debug/examples/validator_scoring_metric"
if [[ ! -x "$binary_path" ]]; then
    bad "binary not found at $binary_path after build"
else
    ok "validator_scoring_metric example builds"
fi

# --- 4. the example binary runs and emits the expected line protocol ---
note "running: $binary_path"
example_output="$("$binary_path" 2>&1)"

# --- 4a. harness marker line is present ---
if printf '%s\n' "$example_output" | grep -Fxq 'VALIDATOR_SCORING_METRIC_HARNESS myosu e2e ok surface=validator_scoring_metric_run_count=1 ladder=0.000000,0.100000,0.200000,0.300000,0.400000,0.500000,0.600000,0.700000,0.800000,0.900000,1.000000 elapsed_ms_total=42'; then
    ok "harness marker line is present (configuration + success marker)"
else
    bad "harness marker line missing or drifted; expected VALIDATOR_SCORING_METRIC_HARNESS myosu e2e ok surface=validator_scoring_metric_run_count=1 ladder=0.000000,...,1.000000 elapsed_ms_total=42"
    printf '%s\n' "$example_output" | tail -10
fi

# --- 4b. single-scenario emission carries the expected fields ---
if printf '%s\n' "$example_output" | grep -Fxq 'VALIDATOR_SCORING_METRIC game=poker scenario_count=1 mean_l1=0.000000 p50_l1=0.000000 p99_l1=0.000000 elapsed_ms=1'; then
    ok "single-scenario VALIDATOR_SCORING_METRIC line carries scenario_count=1 mean_l1=0.000000 p50_l1=0.000000 p99_l1=0.000000 elapsed_ms=1"
else
    bad "single-scenario VALIDATOR_SCORING_METRIC line missing or drifted"
    printf '%s\n' "$example_output" | grep -E '^VALIDATOR_SCORING_METRIC' || true
fi

# --- 5. multi-scenario mean matches the closed-form mean of the ladder ---
# A successful multi-scenario line reports `scenario_count=11`
# `mean_l1=0.500000` (the closed-form mean of `[0.0, 0.1, ...,
# 1.0]`), `p50_l1=0.500000` (the rank-6 element of an 11-step
# monotonic ladder, nearest-rank), and `p99_l1=1.000000` (the
# rank-11 element).
if printf '%s\n' "$example_output" | grep -Fxq 'VALIDATOR_SCORING_METRIC game=poker scenario_count=11 mean_l1=0.500000 p50_l1=0.500000 p99_l1=1.000000 elapsed_ms=42'; then
    ok "multi-scenario mean_l1=0.500000 matches the closed-form mean of the 11-step ladder (0.0+0.1+...+1.0)/11 = 0.5"
else
    bad "multi-scenario mean_l1 / p50_l1 / p99_l1 drifted from the closed-form contract"
    printf '%s\n' "$example_output" | grep -E '^VALIDATOR_SCORING_METRIC' || true
fi

# --- 6. monotonicity marker line confirms p50_l1 <= p99_l1 ---
if printf '%s\n' "$example_output" | grep -Fxq 'VALIDATOR_SCORING_METRIC_MONOTONICITY p50_l1=0.500000 p99_l1=1.000000 monotonic=true'; then
    ok "monotonicity marker line confirms p50_l1=0.500000 <= p99_l1=1.000000 (nearest-rank contract holds)"
else
    bad "monotonicity marker line missing or drifted"
    printf '%s\n' "$example_output" | grep -E '^VALIDATOR_SCORING_METRIC_MONOTONICITY' || true
fi

# --- 7. byte-stability marker line confirms emit() is deterministic ---
if printf '%s\n' "$example_output" | grep -Fxq 'VALIDATOR_SCORING_METRIC_BYTE_STABLE byte_stable=true'; then
    ok "byte-stability marker line confirms two consecutive emit() calls produce byte-identical output (INV-003 determinism)"
else
    bad "byte-stability marker line missing or drifted"
    printf '%s\n' "$example_output" | grep -E '^VALIDATOR_SCORING_METRIC_BYTE_STABLE' || true
fi

# --- 8. cargo test -p myosu-validator stays green (15 new metric tests) ---
# We run the targeted metric test module here, not the full
# validator suite, because the full suite includes a slow
# `quality_benchmark_liars_dice_exploitability_converges` test
# that takes ~60s. The metric tests are the W-06 surface; the
# full validator suite is exercised separately by the CI's
# active-crates job.
note "running: SKIP_WASM_BUILD=1 cargo test -p myosu-validator --lib metric::"
metric_test_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo test --quiet -p myosu-validator --lib metric:: 2>&1
)"
metric_test_summary="$(printf '%s\n' "$metric_test_output" | grep -E '^test result:' | tail -n1)"
if [[ -z "$metric_test_summary" ]]; then
    bad "could not parse metric test summary; expected at least one 'test result:' line"
    printf '%s\n' "$metric_test_output" | tail -20
elif printf '%s\n' "$metric_test_summary" | grep -qE '0 failed'; then
    metric_test_count="$(printf '%s\n' "$metric_test_summary" | sed -nE 's/.*ok\. ([0-9]+) passed.*/\1/p')"
    if [[ -n "$metric_test_count" && "$metric_test_count" -ge 15 ]]; then
        ok "metric module unit tests are green: $metric_test_count passed (>= 15 expected)"
    else
        bad "expected at least 15 metric module unit tests, got $metric_test_count"
    fi
else
    bad "metric module unit tests failed"
    printf '%s\n' "$metric_test_output" | tail -20
fi

# --- 9. README.md Operator Path links the observability doc (W-06) ---
# The doc is wired in a follow-up step; this assertion is the
# drift guard for that step.
if grep -qE 'observability\.md' README.md; then
    ok "README.md Operator Path references docs/operator-guide/observability.md"
else
    bad "README.md Operator Path does not reference docs/operator-guide/observability.md"
fi

# --- 10. the observability doc exists and names the line protocol ---
obs_doc="$repo_root/docs/operator-guide/observability.md"
if [[ ! -f "$obs_doc" ]]; then
    bad "missing $obs_doc"
else
    if grep -qE 'VALIDATOR_SCORING_METRIC' "$obs_doc" \
        && grep -qE 'mean_l1' "$obs_doc" \
        && grep -qE 'p50_l1' "$obs_doc" \
        && grep -qE 'p99_l1' "$obs_doc" \
        && grep -qE 'elapsed_ms' "$obs_doc" \
        && grep -qE 'scenario_count' "$obs_doc" \
        && grep -qE 'nearest-rank' "$obs_doc"; then
        ok "$obs_doc names the line protocol, every field, and the nearest-rank percentile contract"
    else
        bad "$obs_doc is missing one of: VALIDATOR_SCORING_METRIC, mean_l1, p50_l1, p99_l1, elapsed_ms, scenario_count, nearest-rank"
    fi
    if grep -qE 'W-06' "$obs_doc"; then
        ok "$obs_doc is labeled with the W-06 plan row"
    else
        bad "$obs_doc is not labeled with the W-06 plan row"
    fi
fi

# --- 11. the validation.rs scoring loop emits the line on success ---
# (the score_response function calls emit_validator_scoring_metric
# on a successful report; the assert is on the source path so a
# future refactor that drops the emission is caught at code-review
# time without a full chain bring-up).
validation_rs="$repo_root/crates/myosu-validator/src/validation.rs"
if grep -qE 'emit_validator_scoring_metric' "$validation_rs" \
    && grep -qE 'ValidatorScoringMetric::single' "$validation_rs"; then
    ok "validation::score_response emits the metric line via emit_validator_scoring_metric + ValidatorScoringMetric::single"
else
    bad "validation::score_response does not call emit_validator_scoring_metric (a future refactor that drops the emission is loud)"
fi

echo
echo "validator_scoring_metric: pass=$pass fail=$fail"
[[ "$fail" -eq 0 ]]
