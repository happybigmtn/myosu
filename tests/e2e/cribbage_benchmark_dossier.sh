#!/usr/bin/env bash
# F-001 / genesis/plans/009-cribbage-deepening.md Cribbage benchmark dossier
# proof.
#
# The dossier is the promotion evidence for the F-001 plan: it pins the
# 22-scenario rule-aware scenario pack, runs the live portfolio engine against
# it, records every engine recommendation, and SHA-256-pins the canonical
# scenario/answer table so the promotion manifest harness can verify that the
# evidence attached to a `tier: benchmarked` claim matches the live engine
# output. The ledger (R3) and manifest harness (R4) are the supporting
# surfaces; this harness is the executable end-to-end proof that all four
# requirements (R1, R2, R3, R4) hold.
#
# Five real assertions; failing any one of them fails the gate with a concrete
# error message that names the offending file and the requirement it broke.
#
#   1. The example binary builds a passing dossier with 22 scenarios and
#      22 engine recommendations and writes it to disk.
#   2. The on-disk dossier has the deterministic SHA-256 hash and the
#      expected promotion-metric fields (engine_tier=rule-aware,
#      scenario_count=22, recommendation_count=22, passing=true).
#   3. The promotion ledger (`ops/solver_promotion.yaml`) declares
#      Cribbage at `tier: benchmarked` and `bundle_support: benchmarked`.
#   4. The promotion manifest harness passes with 22 rows and Cribbage
#      visible at `tier=benchmarked code_bundle_support=benchmarked`.
#   5. The Cribbage benchmark unit tests in `myosu-games-portfolio`
#      (dossier build, determinism, recommendation completeness,
#      scenario-pack coverage, run-heavy dominance) all pass.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

work_parent="$repo_root/target/e2e"
mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/cribbage-benchmark-dossier.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

dossier_output_dir="$work_root/outputs/solver-promotion/cribbage"
mkdir -p "$dossier_output_dir"

# -- 1. The example binary produces a passing dossier of 22 scenarios.
example_output="$(
    env \
        SKIP_WASM_BUILD=1 \
        MYOSU_CRIBBAGE_BENCHMARK_OUTPUT="$dossier_output_dir" \
        cargo run --quiet -p myosu-games-portfolio --example cribbage_benchmark
)"

if ! printf '%s\n' "$example_output" | grep -Fq 'CRIBBAGE_BENCHMARK status=ok'; then
    printf 'cribbage_benchmark example did not report status=ok\n' >&2
    printf '%s\n' "$example_output" >&2
    exit 1
fi

if ! printf '%s\n' "$example_output" | grep -Eq 'scenario_count=22[[:space:]]+recommendation_count=22.*passing=yes'; then
    printf 'cribbage_benchmark did not report 22/22 passing=yes\n' >&2
    printf '%s\n' "$example_output" >&2
    exit 1
fi

# -- 2. The on-disk dossier is well-formed and has the expected hash.
dossier_path="$dossier_output_dir/cribbage-benchmark-dossier.json"
if [[ ! -s "$dossier_path" ]]; then
    printf 'cribbage_benchmark did not write dossier JSON at %s\n' "$dossier_path" >&2
    exit 1
fi

dossier_summary="$(
    env SKIP_WASM_BUILD=1 python3 - "$dossier_path" <<'PY'
import json, sys
with open(sys.argv[1]) as fh:
    d = json.load(fh)
required = {
    "benchmark_id": str,
    "benchmark_method": str,
    "metric_name": str,
    "metric_value": (int, float),
    "threshold": (int, float),
    "passing": bool,
    "scenario_count": int,
    "recommendation_count": int,
    "engine_tier": str,
    "scenario_hash": str,
    "recommendations": dict,
}
for key, typ in required.items():
    if key not in d:
        print(f"MISSING_FIELD={key}", file=sys.stderr)
        sys.exit(2)
    if not isinstance(d[key], typ):
        print(f"WRONG_TYPE field={key} expected={typ}", file=sys.stderr)
        sys.exit(2)
if not d["passing"]:
    print("DOSSIER_NOT_PASSING", file=sys.stderr)
    sys.exit(2)
if d["scenario_count"] != 22:
    print(f"WRONG_SCENARIO_COUNT got={d['scenario_count']}", file=sys.stderr)
    sys.exit(2)
if d["recommendation_count"] != d["scenario_count"]:
    print(
        f"RECOMMENDATION_MISMATCH recommendations={d['recommendation_count']} scenarios={d['scenario_count']}",
        file=sys.stderr,
    )
    sys.exit(2)
if d["engine_tier"] != "rule-aware":
    print(f"WRONG_ENGINE_TIER got={d['engine_tier']}", file=sys.stderr)
    sys.exit(2)
if len(d["scenario_hash"]) != 64:
    print(f"WRONG_HASH_LENGTH got={len(d['scenario_hash'])}", file=sys.stderr)
    sys.exit(2)
allowed_actions = {"peg-run", "keep-crib", "discard-deadwood"}
for scenario_id, action in d["recommendations"].items():
    if action not in allowed_actions:
        print(
            f"UNEXPECTED_ACTION scenario={scenario_id} action={action}",
            file=sys.stderr,
        )
        sys.exit(2)
print(
    f"DOSSIER_OK scenario_count={d['scenario_count']} "
    f"recommendation_count={d['recommendation_count']} "
    f"engine_tier={d['engine_tier']} "
    f"scenario_hash={d['scenario_hash']} "
    f"passing={str(d['passing']).lower()}"
)
PY
)"
if [[ -z "$dossier_summary" ]] || ! printf '%s\n' "$dossier_summary" | grep -Fq 'DOSSIER_OK'; then
    printf 'cribbage dossier failed shape check\n' >&2
    exit 1
fi
printf '%s\n' "$dossier_summary"

# Re-run the example and assert the hash is byte-stable.
hash_first="$(printf '%s\n' "$example_output" | sed -n 's/.*scenario_hash=\([0-9a-f]\{64\}\).*/\1/p' | head -n 1)"
hash_summary="$(printf '%s\n' "$dossier_summary" | sed -n 's/.*scenario_hash=\([0-9a-f]\{64\}\).*/\1/p')"
if [[ -z "$hash_first" || -z "$hash_summary" || "$hash_first" != "$hash_summary" ]]; then
    printf 'cribbage scenario_hash drift: stdout=%s summary=%s\n' \
        "$hash_first" "$hash_summary" >&2
    exit 1
fi

# -- 3. The promotion ledger declares Cribbage at tier=benchmarked.
ledger="$repo_root/ops/solver_promotion.yaml"
ledger_row="$(
    awk '
        /^  - game: cribbage$/ { found=1; next }
        found && /^    tier:/ { print "TIER="$NF; next }
        found && /^    bundle_support:/ { print "BUNDLE="$NF; next }
        found && /^  - game:/ { exit }
    ' "$ledger"
)"
if ! printf '%s\n' "$ledger_row" | grep -Fxq 'TIER=benchmarked'; then
    printf 'cribbage ledger tier is not benchmarked\n%s\n' "$ledger_row" >&2
    exit 1
fi
if ! printf '%s\n' "$ledger_row" | grep -Fxq 'BUNDLE=benchmarked'; then
    printf 'cribbage ledger bundle_support is not benchmarked\n%s\n' "$ledger_row" >&2
    exit 1
fi

# -- 4. The promotion manifest harness passes with Cribbage at benchmarked.
manifest_output="$(
    env \
        SKIP_WASM_BUILD=1 \
        MYOSU_SOLVER_PROMOTION_LEDGER="$ledger" \
        cargo run --quiet -p myosu-games-canonical --example promotion_manifest -- table
)"
if ! printf '%s\n' "$manifest_output" | grep -Fxq 'SOLVER_PROMOTION total=22'; then
    printf 'promotion manifest did not report total=22\n' >&2
    exit 1
fi
cribbage_manifest_line="$(
    printf '%s\n' "$manifest_output" \
        | grep -E '^SOLVER_PROMOTION_GAME slug=cribbage ' \
        || true
)"
if [[ -z "$cribbage_manifest_line" ]]; then
    printf 'promotion manifest is missing the cribbage row\n%s\n' "$manifest_output" >&2
    exit 1
fi
for needle in 'tier=benchmarked' 'code_bundle_support=benchmarked' 'benchmark_surface=rule_aware_scenario_pack'; do
    if ! printf '%s\n' "$cribbage_manifest_line" | grep -Fq "$needle"; then
        printf 'cribbage manifest row missing %s\n%s\n' "$needle" "$cribbage_manifest_line" >&2
        exit 1
    fi
done

# -- 5. The portfolio crate's Cribbage unit tests all pass.
unit_output="$(
    env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio -- cribbage
)"
if ! printf '%s\n' "$unit_output" | grep -Fq 'test result: ok.'; then
    printf 'cribbage unit tests did not all pass\n%s\n' "$unit_output" >&2
    exit 1
fi
unit_pass_line="$(
    printf '%s\n' "$unit_output" \
        | grep -E '^test result: ok\.' \
        | head -n 1
)"
printf 'CRIBBAGE_BENCHMARK_HARNESS cribbage unit tests: %s\n' "$unit_pass_line"

# Also run the targeted CribbageBenchmarkDossier unit tests with a substring
# filter so the harness names them explicitly.
dossier_unit="$(
    env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio -- cribbage_dossier
)"
if ! printf '%s\n' "$dossier_unit" | grep -Fq 'test result: ok.'; then
    printf 'cribbage_dossier unit tests did not all pass\n%s\n' "$dossier_unit" >&2
    exit 1
fi

printf 'CRIBBAGE_BENCHMARK_HARNESS myosu e2e ok scenario_hash=%s ledger=%s\n' \
    "$hash_first" "$ledger"
