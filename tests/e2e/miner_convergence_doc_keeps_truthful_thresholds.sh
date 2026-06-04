#!/usr/bin/env bash
# F-003 / genesis/plans (operator-stack spec) miner convergence doc-reg drift guard.
#
# F-003 (Miner convergence gate research) requires a truthful, per-game
# training-iteration recommendation that operators can defend against a
# real benchmark surface, not the same-checkpoint validator self-score.
# The Liar's Dice side was resolved by the F-007 work (the public
# `liars_dice_benchmark_points` helper + `quality_benchmark` example +
# `tests/e2e/quality_benchmark_liars_dice.sh` harness + a documented
# 512-iteration recommendation at exploitability <= 0.70). The poker
# side is intentionally blocked on richer encoder artifacts (the
# checked-in `bootstrap_artifacts.rs` is sparse, so a real exploitability
# ladder cannot be recorded against the stage-0 bootstrap fixtures).
#
# This harness is the F-003 doc-reg drift guard: it fail-closes if the
# operator-facing truth (this document, the operator quickstart's
# pointer to it, and the live `recommended_minimum_iterations` value
# emitted by the `quality_benchmark` example) drift apart. A solver
# constant change that moves the live recommendation must be reflected
# in `docs/operator-guide/miner-convergence.md`, the quickstart's
# "Poker remains blocked" paragraph must still link to the new doc, and
# the new doc's per-game table must agree with the live benchmark.
#
# Six real assertions; failing any one of them fails the gate with a
# concrete error message that names the offending file and the
# requirement it broke.
#
#   1. `docs/operator-guide/miner-convergence.md` exists, is non-empty,
#      and carries the F-003 row label so a future reader can locate
#      the source row in `IMPLEMENTATION_PLAN.md`.
#   2. The new doc publishes the Liar's Dice 512-iteration
#      recommendation at the `0.70` exploitability threshold, and names
#      the source-of-truth executable proof
#      (`tests/e2e/quality_benchmark_liars_dice.sh`) so the operator can
#      reproduce it.
#   3. The new doc marks `nlhe-heads-up` as "blocked" with the
#      `isomorphism not found` / `postflop_complete=false` rationale
#      so the operator does not see a placeholder recommendation.
#   4. The new doc names `kuhn_poker` as closed-form / 0-iteration so
#      operators do not try to MCCFR-train a closed-form toy solver.
#   5. The operator quickstart still links to the new doc from the
#      Liar's Dice quality section, so a new operator sees the
#      per-game table on the same page as the per-step walkthrough.
#   6. The live `quality_benchmark` example's
#      `QUALITY_BENCHMARK_RECOMMENDATION` line still emits
#      `recommended_minimum_iterations=512` at `threshold=0.700000` and
#      the live `LIARS_DICE_SOLVER_TREES` constant is still 1024, so
#      the doc and the live benchmark cannot drift apart silently.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

doc_path="$repo_root/docs/operator-guide/miner-convergence.md"
quickstart_path="$repo_root/docs/operator-guide/quickstart.md"

# -- 1. The new doc exists, is non-empty, and is labeled with the F-003
#       row name.
if [[ ! -s "$doc_path" ]]; then
    printf 'miner-convergence doc missing or empty: %s\n' "$doc_path" >&2
    exit 1
fi
if ! grep -Fq 'F-003' "$doc_path"; then
    printf 'miner-convergence doc does not label itself with F-003: %s\n' \
        "$doc_path" >&2
    exit 1
fi

# -- 2. The new doc publishes the 512-iteration / 0.70 Liar's Dice
#       recommendation and points at the executable proof.
for needle in \
    'recommended_minimum_iterations=512' \
    '`0.70`' \
    'tests/e2e/quality_benchmark_liars_dice.sh' \
    'LIARS_DICE_USEFUL_EXPLOITABILITY_THRESHOLD'; do
    if ! grep -Fq "$needle" "$doc_path"; then
        printf 'miner-convergence doc missing required marker: %s\n%s\n' \
            "$needle" "$doc_path" >&2
        exit 1
    fi
done

# -- 3. The new doc marks poker as blocked with the truthful rationale.
for needle in \
    'nlhe-heads-up' \
    'Blocked' \
    'isomorphism not found' \
    'postflop_complete=false'; do
    if ! grep -Fq "$needle" "$doc_path"; then
        printf 'miner-convergence doc missing poker-blocker marker: %s\n' \
            "$needle" >&2
        exit 1
    fi
done

# -- 4. The new doc records `kuhn_poker` as closed-form / 0 iterations
#       so operators do not MCCFR-train a toy solver.
for needle in \
    'kuhn_poker' \
    'closed form' \
    'Closed-form'; do
    if ! grep -Fiq "$needle" "$doc_path"; then
        printf 'miner-convergence doc missing kuhn closed-form marker: %s\n' \
            "$needle" >&2
        exit 1
    fi
done

# -- 5. The operator quickstart links to the new doc from the Liar's
#       Dice quality section (so a new operator sees the per-game
#       table on the same page as the per-step walkthrough).
if ! grep -Fq 'miner-convergence.md' "$quickstart_path"; then
    printf 'operator quickstart does not link to miner-convergence.md: %s\n' \
        "$quickstart_path" >&2
    exit 1
fi

# -- 6. The live `quality_benchmark` example still agrees with the
#       published 512 / 0.70 number and the live solver constant is
#       still 1024 (the same number the F-007 example prints).
example_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo run --quiet -p myosu-validator \
        --example quality_benchmark -- 0 128 256 512 \
        2>/dev/null
)"
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
    printf 'live quality_benchmark recommendation does not match miner-convergence doc:\n%s\n' \
        "$recommendation_line" >&2
    exit 1
fi
recommended_exploitability="$(
    printf '%s' "$recommendation_line" \
        | sed -n 's/.*exploitability=\([0-9.]*\).*/\1/p'
)"
if ! python3 -c "import sys; sys.exit(0 if float('${recommended_exploitability}') <= 0.70 else 1)"; then
    printf 'live quality_benchmark recommended exploitability %s is not at-or-below 0.70\n' \
        "$recommended_exploitability" >&2
    exit 1
fi
trees_check="$(
    env SKIP_WASM_BUILD=1 \
        cargo run --quiet -p myosu-validator \
        --example quality_benchmark -- 0 2>/dev/null \
        | grep -E '^QUALITY_BENCHMARK solver_trees=' \
        | head -n 1
)"
if [[ "$trees_check" != 'QUALITY_BENCHMARK solver_trees=1024' ]]; then
    printf 'live quality_benchmark solver_trees drift: %s\n' "$trees_check" >&2
    exit 1
fi

printf 'MINER_CONVERGENCE_HARNESS doc-reg drift guard ok recommended=512 threshold=0.700000 solver_trees=1024\n'
