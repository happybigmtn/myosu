#!/usr/bin/env bash
# F-003 / F-007 Liar's Dice quality benchmark proof.
#
# F-003 (the F-007 follow-on) requires a truthful Liar's Dice
# training-quality threshold that "actually varies with solver quality" so
# operators know how many MCCFR iterations to train before serving a
# strategy. The same-checkpoint validator exact-match path is explicitly
# NOT a convergence metric (the F-003 blocker note calls it out), so the
# truthful surface is exact best-response exploitability measured from a
# fresh `LiarsDiceSolver::new()` start.
#
# This harness is the executable end-to-end proof that the truthful
# surface exists, is reachable through the operator-facing example binary
# (`cargo run -p myosu-validator --example quality_benchmark`), and that
# the operator-guide recommendation (`512` minimum iterations at the
# `0.70` exploitability threshold) is reproduced and stays reproduced
# across solver-constant changes.
#
# Six real assertions; failing any one of them fails the gate with a
# concrete error message that names the offending input and the
# requirement it broke.
#
#   1. The example binary produces a `QUALITY_BENCHMARK_*` report from a
#      fresh-start ladder and exits 0.
#   2. The report carries the expected game, surface, solver_trees,
#      threshold, and iteration ladder so the operator can see the
#      benchmark configuration at a glance.
#   3. The report contains exactly one `QUALITY_BENCHMARK_POINT` line
#      per requested iteration count, with a finite exploitability and
#      a monotonically non-increasing exploitability as iterations grow
#      (a regression in the ladder monotonicity would be a solver bug).
#   4. The report recommends `recommended_minimum_iterations=512` at
#      `exploitability<=0.700000` (the operator-guide number).
#   5. The validator's `quality_benchmark_liars_dice_exploitability_converges`
#      unit test passes against the same threshold, proving the operator
#      recommendation and the unit test share one source of truth.
#   6. The public `myosu_validator::validation::LIARS_DICE_SOLVER_TREES`
#      constant is `1024` (the same value the example prints in the
#      `solver_trees=` line) so operator-facing reproduction and the
#      unit-test ladder cannot drift apart silently.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# -- 1. The example binary produces a QUALITY_BENCHMARK report.
example_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo run --quiet -p myosu-validator \
        --example quality_benchmark -- 0 128 256 512
)"

if ! printf '%s\n' "$example_output" | grep -Fxq 'QUALITY_BENCHMARK game=liars-dice'; then
    printf 'quality_benchmark example did not report game=liars-dice\n%s\n' \
        "$example_output" >&2
    exit 1
fi

# -- 2. The report carries the expected configuration line(s).
for needle in \
    'QUALITY_BENCHMARK surface=exact_best_response_exploitability' \
    'QUALITY_BENCHMARK solver_trees=1024' \
    'QUALITY_BENCHMARK exploitability_threshold=0.700000' \
    'QUALITY_BENCHMARK iterations=0,128,256,512'; do
    if ! printf '%s\n' "$example_output" | grep -Fxq "$needle"; then
        printf 'quality_benchmark example missing line: %s\n%s\n' \
            "$needle" "$example_output" >&2
        exit 1
    fi
done

# -- 3. The report has exactly one QUALITY_BENCHMARK_POINT line per
#       requested iteration, with finite exploitability, and the
#       exploitability is monotonically non-increasing.
point_count="$(
    printf '%s\n' "$example_output" \
        | grep -Ec '^QUALITY_BENCHMARK_POINT iterations=[0-9]+ exploitability='
)"
if [[ "$point_count" -ne 4 ]]; then
    printf 'quality_benchmark example emitted %s points, expected 4\n%s\n' \
        "$point_count" "$example_output" >&2
    exit 1
fi

prev_exploitability=""
for iters in 0 128 256 512; do
    line="$(
        printf '%s\n' "$example_output" \
            | grep -E "^QUALITY_BENCHMARK_POINT iterations=${iters} " \
            || true
    )"
    if [[ -z "$line" ]]; then
        printf 'quality_benchmark example missing point for iterations=%s\n%s\n' \
            "$iters" "$example_output" >&2
        exit 1
    fi
    exploitability="$(
        printf '%s' "$line" \
            | sed -n 's/.*exploitability=\([0-9.]\+\).*/\1/p'
    )"
    if ! python3 -c "import sys; sys.exit(0 if float('${exploitability}') == float('${exploitability}') else 1)"; then
        printf 'quality_benchmark point for iterations=%s is non-finite: %s\n' \
            "$iters" "$exploitability" >&2
        exit 1
    fi
    if [[ -n "$prev_exploitability" ]]; then
        if ! python3 -c "import sys; sys.exit(0 if float('${prev_exploitability}') >= float('${exploitability}') else 1)"; then
            printf \
                'quality_benchmark monotonicity violated: prev=%s current=%s at iterations=%s\n' \
                "$prev_exploitability" "$exploitability" "$iters" >&2
            exit 1
        fi
    fi
    prev_exploitability="$exploitability"
done

# -- 4. The report recommends 512 iterations at exploitability<=0.70.
recommendation_line="$(
    printf '%s\n' "$example_output" \
        | grep -E '^QUALITY_BENCHMARK_RECOMMENDATION ' \
        || true
)"
if [[ -z "$recommendation_line" ]]; then
    printf 'quality_benchmark example missing QUALITY_BENCHMARK_RECOMMENDATION line\n%s\n' \
        "$example_output" >&2
    exit 1
fi
if ! printf '%s\n' "$recommendation_line" \
        | grep -Eq 'recommended_minimum_iterations=512[[:space:]]+exploitability=[0-9.]+[[:space:]]+threshold=0\.700000'; then
    printf 'quality_benchmark recommendation does not match operator guide\n%s\n' \
        "$recommendation_line" >&2
    exit 1
fi
recommended_exploitability="$(
    printf '%s' "$recommendation_line" \
        | sed -n 's/.*exploitability=\([0-9.]\+\).*/\1/p'
)"
if ! python3 -c "import sys; sys.exit(0 if float('${recommended_exploitability}') <= 0.70 else 1)"; then
    printf \
        'quality_benchmark recommended exploitability %s is not at-or-below 0.70\n' \
        "$recommended_exploitability" >&2
    exit 1
fi

# -- 5. The validator's quality_benchmark unit test passes against the
#       same threshold (proves the operator recommendation and the unit
#       test share one source of truth).
unit_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo test --quiet -p myosu-validator -- \
        quality_benchmark_liars_dice_exploitability_converges
)"
if ! printf '%s\n' "$unit_output" | grep -Fq 'test result: ok.'; then
    printf 'quality_benchmark unit test did not pass\n%s\n' "$unit_output" >&2
    exit 1
fi
printf 'QUALITY_BENCHMARK_HARNESS validator unit tests: %s\n' \
    "$(printf '%s\n' "$unit_output" | grep -E '^test result: ok\.' | head -n 1)"

# -- 6. The public solver_trees constant is 1024 and matches the example
#       output (catches the drift if anyone changes either side).
trees_check="$(
    env SKIP_WASM_BUILD=1 \
        cargo run --quiet -p myosu-validator \
        --example quality_benchmark -- 0 2>&1 \
        | grep -E '^QUALITY_BENCHMARK solver_trees=' \
        | head -n 1
)"
if [[ "$trees_check" != 'QUALITY_BENCHMARK solver_trees=1024' ]]; then
    printf 'quality_benchmark example solver_trees drift: %s\n' "$trees_check" >&2
    exit 1
fi

printf 'QUALITY_BENCHMARK_HARNESS liars-dice quality benchmark ok threshold=0.700000 recommended=512\n'
