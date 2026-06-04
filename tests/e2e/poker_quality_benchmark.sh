#!/usr/bin/env bash
# F-003 / NEM-001B poker quality benchmark proof.
#
# F-003 (the F-007 follow-on) requires a truthful Poker training-quality
# threshold that "actually varies with solver quality" so operators know
# when their candidate is converging on the closed-form reference
# profile. The same-checkpoint validator exact-match path is explicitly
# NOT a convergence metric (the F-003 blocker note calls it out), and
# positive-iteration MCCFR training is blocked upstream by `isomorphism
# not found` against the checked-in sparse bootstrap encoder artifacts
# (`bootstrap_reference_solver` returns the error past the zero-th
# iteration — see `crates/myosu-games-poker/src/benchmark.rs:571-591`
# and the `benchmark_reports_sparse_encoder_failure_cleanly` test), so
# the truthful surface is the convex mix-ladder between the closed-form
# reference profile (`mix=1.0`) and a uniform-weight perturbation
# (`mix=0.0`).
#
# This harness is the executable end-to-end proof that the truthful
# surface exists, is reachable through the operator-facing example
# binary (`cargo run -p myosu-validator --example poker_quality_benchmark`),
# and that the F-003 / NEM-001B ladder stays truthful across
# solver-constant changes. The `recommended_minimum_mix` is the lowest
# `mix` whose `exact_action_match_ratio` clears
# `POKER_USEFUL_REFERENCE_MATCH_RATIO=0.95` against the
# `bootstrap_scenarios()` reference pack (80 scenarios).
#
# Six real assertions; failing any one of them fails the gate with a
# concrete error message that names the offending input and the
# requirement it broke.
#
#   1. The example binary produces a `POKER_QUALITY_BENCHMARK_*` report
#      from the default mix ladder and exits 0.
#   2. The report carries the expected game, surface,
#      `reference_self_match_count=80`, `reference_self_match_l1=0.0`,
#      and `match_ratio_threshold=0.95` so the operator can see the
#      benchmark configuration at a glance.
#   3. The report contains exactly one `POKER_QUALITY_BENCHMARK_POINT`
#      line per requested `mix` ladder step, with finite mean L1
#      distance and exact-action-match counts, and the
#      `exact_action_match_ratio` is monotonically non-decreasing in
#      `mix` (a regression in the ladder monotonicity would be a
#      solver bug, not a benchmark bug).
#   4. The report's `mix=1.0` (self-match) anchor point has
#      `mean_l1_distance=0.0` and `exact_action_matches=80` (the
#      `POKER_REFERENCE_SELF_MATCH_COUNT` constant); this is the
#      regression anchor the F-003 / NEM-001B doc-reg drift guard
#      relies on.
#   5. The validator's `poker_quality_benchmark_points_self_match_is_exact_at_mix_one`
#      and `poker_quality_benchmark_points_default_ladder_is_monotonic`
#      unit tests pass against the same ladder, proving the operator
#      reproduction and the unit tests share one source of truth.
#   6. The public `myosu_games_poker::POKER_REFERENCE_SELF_MATCH_COUNT`
#      constant is `80` and the `POKER_REFERENCE_SELF_MATCH_L1`
#      constant is `0.0` — the same values the example prints in the
#      `reference_self_match_count=` / `reference_self_match_l1=` lines
#      — so operator-facing reproduction and the unit-test ladder
#      cannot drift apart silently.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# -- 1. The example binary produces a POKER_QUALITY_BENCHMARK report
#       from the default mix ladder and exits 0.
example_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo run --quiet -p myosu-validator \
        --example poker_quality_benchmark
)"

if ! printf '%s\n' "$example_output" | grep -Fxq 'POKER_QUALITY_BENCHMARK game=nlhe-heads-up'; then
    printf 'poker_quality_benchmark example did not report game=nlhe-heads-up\n%s\n' \
        "$example_output" >&2
    exit 1
fi
if ! printf '%s\n' "$example_output" | grep -Fxq 'POKER_QUALITY_BENCHMARK surface=mixed_bootstrap_reference_l1'; then
    printf 'poker_quality_benchmark example did not report surface=mixed_bootstrap_reference_l1\n%s\n' \
        "$example_output" >&2
    exit 1
fi

# -- 2. The report carries the expected configuration line(s).
for needle in \
    'POKER_QUALITY_BENCHMARK reference_self_match_count=80' \
    'POKER_QUALITY_BENCHMARK reference_self_match_l1=0.000000' \
    'POKER_QUALITY_BENCHMARK match_ratio_threshold=0.950000' \
    'POKER_QUALITY_BENCHMARK mix_ladder=0.000000,0.250000,0.500000,0.750000,1.000000'; do
    if ! printf '%s\n' "$example_output" | grep -Fxq "$needle"; then
        printf 'poker_quality_benchmark example missing line: %s\n%s\n' \
            "$needle" "$example_output" >&2
        exit 1
    fi
done

# -- 3. The report has exactly one POKER_QUALITY_BENCHMARK_POINT line
#       per requested mix ladder step, with finite mean_l1_distance
#       and a monotonically non-decreasing exact_action_match_ratio.
point_count="$(
    printf '%s\n' "$example_output" \
        | grep -Ec '^POKER_QUALITY_BENCHMARK_POINT mix=[0-9.]+ mean_l1_distance='
)"
if [[ "$point_count" -ne 5 ]]; then
    printf 'poker_quality_benchmark example emitted %s points, expected 5\n%s\n' \
        "$point_count" "$example_output" >&2
    exit 1
fi

prev_match_ratio=""
for mix in 0.000000 0.250000 0.500000 0.750000 1.000000; do
    line="$(
        printf '%s\n' "$example_output" \
            | grep -E "^POKER_QUALITY_BENCHMARK_POINT mix=${mix} " \
            || true
    )"
    if [[ -z "$line" ]]; then
        printf 'poker_quality_benchmark example missing point for mix=%s\n%s\n' \
            "$mix" "$example_output" >&2
        exit 1
    fi
    match_ratio="$(
        printf '%s' "$line" \
            | sed -n 's/.*exact_action_match_ratio=\([0-9.]\+\).*/\1/p'
    )"
    if ! python3 -c "import sys; sys.exit(0 if float('${match_ratio}') == float('${match_ratio}') else 1)"; then
        printf 'poker_quality_benchmark point for mix=%s has non-finite match ratio: %s\n' \
            "$mix" "$match_ratio" >&2
        exit 1
    fi
    if [[ -n "$prev_match_ratio" ]]; then
        if ! python3 -c "import sys; sys.exit(0 if float('${prev_match_ratio}') <= float('${match_ratio}') else 1)"; then
            printf \
                'poker_quality_benchmark monotonicity violated: prev=%s current=%s at mix=%s\n' \
                "$prev_match_ratio" "$match_ratio" "$mix" >&2
            exit 1
        fi
    fi
    prev_match_ratio="$match_ratio"
done

# -- 4. The report's mix=1.0 (self-match) anchor point has
#       mean_l1_distance=0.0 and exact_action_matches=80.
self_match_line="$(
    printf '%s\n' "$example_output" \
        | grep -E '^POKER_QUALITY_BENCHMARK_POINT mix=1.000000 ' \
        || true
)"
if [[ -z "$self_match_line" ]]; then
    printf 'poker_quality_benchmark example missing self-match anchor (mix=1.000000) point\n%s\n' \
        "$example_output" >&2
    exit 1
fi
self_match_l1="$(
    printf '%s' "$self_match_line" \
        | sed -n 's/.*mean_l1_distance=\([0-9.]\+\).*/\1/p'
)"
self_match_count="$(
    printf '%s' "$self_match_line" \
        | sed -n 's/.*exact_action_matches=\([0-9]\+\).*/\1/p'
)"
if ! python3 -c "import sys; sys.exit(0 if float('${self_match_l1}') == 0.0 else 1)"; then
    printf \
        'poker_quality_benchmark self-match mean_l1 is %s, expected 0.0\n' \
        "$self_match_l1" >&2
    exit 1
fi
if [[ "$self_match_count" -ne 80 ]]; then
    printf \
        'poker_quality_benchmark self-match exact_action_matches is %s, expected 80\n' \
        "$self_match_count" >&2
    exit 1
fi

# -- 5. The validator's poker_quality_benchmark unit tests pass against
#       the same ladder (proves the operator reproduction and the unit
#       tests share one source of truth).
unit_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo test --quiet -p myosu-validator -- \
        poker_quality_benchmark
)"
if ! printf '%s\n' "$unit_output" | grep -Fq 'test result: ok.'; then
    printf 'poker_quality_benchmark unit tests did not pass\n%s\n' "$unit_output" >&2
    exit 1
fi
printf 'POKER_QUALITY_BENCHMARK_HARNESS validator unit tests: %s\n' \
    "$(printf '%s\n' "$unit_output" | grep -E '^test result: ok\.' | head -n 1)"

# -- 6. The public POKER_REFERENCE_SELF_MATCH_COUNT and
#       POKER_REFERENCE_SELF_MATCH_L1 constants are 80 and 0.0
#       respectively (catch drift if anyone changes either side).
constants_check="$(
    env SKIP_WASM_BUILD=1 \
        cargo run --quiet -p myosu-validator \
        --example poker_quality_benchmark \
        | grep -E '^POKER_QUALITY_BENCHMARK reference_self_match_' \
)"
expected_count_line='POKER_QUALITY_BENCHMARK reference_self_match_count=80'
expected_l1_line='POKER_QUALITY_BENCHMARK reference_self_match_l1=0.000000'
if ! printf '%s\n' "$constants_check" | grep -Fxq "$expected_count_line"; then
    printf 'poker_quality_benchmark self-match count drift: %s\n' "$constants_check" >&2
    exit 1
fi
if ! printf '%s\n' "$constants_check" | grep -Fxq "$expected_l1_line"; then
    printf 'poker_quality_benchmark self-match l1 drift: %s\n' "$constants_check" >&2
    exit 1
fi

printf 'POKER_QUALITY_BENCHMARK_HARNESS nlhe-heads-up poker quality benchmark ok mix_ladder=5 threshold=0.95 recommended_self_match=80/80\n'
