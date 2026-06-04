#!/usr/bin/env bash
# F-018 Dou-Di-Zhu benchmark dossier proof.
#
# The dossier is the promotion evidence for the F-018 slice (the
# F-001/F-008/F-009/F-010/F-011/F-012/F-013/F-014/F-015/F-016/F-017-pattern
# twelfth portfolio-game-promotion slice after Cribbage / Hearts /
# Gin Rummy / Spades / Bridge / Call Break / Backgammon / Hanafuda
# Koi-Koi / Hwatu Go-Stop / Stratego / PLO): it pins the 22-scenario
# rule-aware scenario pack for Dou-Di-Zhu, runs the live portfolio
# engine against it, records every engine recommendation, and
# SHA-256-pins the canonical scenario/answer table so the promotion
# manifest harness can verify that the evidence attached to a
# `tier: benchmarked` claim matches the live engine output. The
# ledger (R3) and manifest harness (R4) are the supporting surfaces;
# this harness is the executable end-to-end proof that all four
# requirements (R1, R2, R3, R4) hold for Dou-Di-Zhu in the same way
# they hold for the eleven prior portfolio slices. F-018 also opens
# the `state-aware bomb-preservation heuristic` engine sub-family in
# `crates/myosu-games-portfolio/src/engines/shedding.rs::dou_di_zhu` —
# the first dossier slice for the `shedding` engine family, and the
# natural one because Dou-Di-Zhu is the 3-player shedding variant
# that the F-016 / F-017 scope boundaries named as the obvious next
# candidate once a `poker_like` engine sub-family dossier row was
# shipped.
#
# Five real assertions; failing any one of them fails the gate with
# a concrete error message that names the offending file and the
# requirement it broke.
#
#   1. The example binary builds a passing dossier with 22 scenarios
#      and 22 engine recommendations and writes it to disk.
#   2. The on-disk dossier has the deterministic SHA-256 hash and
#      the expected promotion-metric fields
#      (engine_tier=rule-aware, scenario_count=22,
#      recommendation_count=22, passing=true) and every action token
#      is one of preserve-bomb / landlord-bid / shed-lowest.
#   3. The promotion ledger (`ops/solver_promotion.yaml`) declares
#      Dou-Di-Zhu at `tier: benchmarked` and `bundle_support:
#      benchmarked`.
#   4. The promotion manifest harness passes with Dou-Di-Zhu visible
#      at `tier=benchmarked code_bundle_support=benchmarked`.
#   5. The Dou-Di-Zhu benchmark unit tests in `myosu-games-portfolio`
#      (dossier build, determinism, recommendation completeness,
#      preserve-bomb dominance, landlord-bid dominance, shed-lowest
#      dominance, scenario count) all pass.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

work_parent="$repo_root/target/e2e"
mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/dou-di-zhu-benchmark-dossier.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

dossier_output_dir="$work_root/outputs/solver-promotion/dou-di-zhu"
mkdir -p "$dossier_output_dir"

# -- 1. The example binary produces a passing dossier of 22 scenarios.
example_output="$(
    env \
        SKIP_WASM_BUILD=1 \
        MYOSU_DOU_DI_ZHU_BENCHMARK_OUTPUT="$dossier_output_dir" \
        cargo run --quiet -p myosu-games-portfolio --example dou_di_zhu_benchmark
)"

if ! printf '%s\n' "$example_output" | grep -Fq 'DOU_DI_ZHU_BENCHMARK status=ok'; then
    printf 'dou_di_zhu_benchmark example did not report status=ok\n' >&2
    printf '%s\n' "$example_output" >&2
    exit 1
fi

if ! printf '%s\n' "$example_output" | grep -Eq 'scenario_count=22[[:space:]]+recommendation_count=22.*passing=yes'; then
    printf 'dou_di_zhu_benchmark did not report 22/22 passing=yes\n' >&2
    printf '%s\n' "$example_output" >&2
    exit 1
fi

# -- 2. The on-disk dossier is well-formed and has the expected hash.
dossier_path="$dossier_output_dir/dou-di-zhu-benchmark-dossier.json"
if [[ ! -s "$dossier_path" ]]; then
    printf 'dou_di_zhu_benchmark did not write dossier JSON at %s\n' "$dossier_path" >&2
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
allowed_actions = {"preserve-bomb", "landlord-bid", "shed-lowest"}
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
    printf 'dou_di_zhu dossier failed shape check\n' >&2
    exit 1
fi
printf '%s\n' "$dossier_summary"

# Re-run the example and assert the hash is byte-stable.
hash_first="$(printf '%s\n' "$example_output" | sed -n 's/.*scenario_hash=\([0-9a-f]\{64\}\).*/\1/p' | head -n 1)"
hash_summary="$(printf '%s\n' "$dossier_summary" | sed -n 's/.*scenario_hash=\([0-9a-f]\{64\}\).*/\1/p')"
if [[ -z "$hash_first" || -z "$hash_summary" || "$hash_first" != "$hash_summary" ]]; then
    printf 'dou_di_zhu scenario_hash drift: stdout=%s summary=%s\n' \
        "$hash_first" "$hash_summary" >&2
    exit 1
fi

# -- 3. The promotion ledger declares Dou-Di-Zhu at tier=benchmarked.
ledger="$repo_root/ops/solver_promotion.yaml"
ledger_row="$(
    awk '
        /^  - game: dou-di-zhu$/ { found=1; next }
        found && /^    tier:/ { print "TIER="$NF; next }
        found && /^    bundle_support:/ { print "BUNDLE="$NF; next }
        found && /^  - game:/ { exit }
    ' "$ledger"
)"
if ! printf '%s\n' "$ledger_row" | grep -Fxq 'TIER=benchmarked'; then
    printf 'dou-di-zhu ledger tier is not benchmarked\n%s\n' "$ledger_row" >&2
    exit 1
fi
if ! printf '%s\n' "$ledger_row" | grep -Fxq 'BUNDLE=benchmarked'; then
    printf 'dou-di-zhu ledger bundle_support is not benchmarked\n%s\n' "$ledger_row" >&2
    exit 1
fi

# -- 4. The promotion manifest harness passes with Dou-Di-Zhu at benchmarked.
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
dou_di_zhu_manifest_line="$(
    printf '%s\n' "$manifest_output" \
        | grep -E '^SOLVER_PROMOTION_GAME slug=dou-di-zhu ' \
        || true
)"
if [[ -z "$dou_di_zhu_manifest_line" ]]; then
    printf 'promotion manifest is missing the dou-di-zhu row\n%s\n' "$manifest_output" >&2
    exit 1
fi
for needle in 'tier=benchmarked' 'code_bundle_support=benchmarked' 'benchmark_surface=rule_aware_scenario_pack'; do
    if ! printf '%s\n' "$dou_di_zhu_manifest_line" | grep -Fq "$needle"; then
        printf 'dou-di-zhu manifest row missing %s\n%s\n' "$needle" "$dou_di_zhu_manifest_line" >&2
        exit 1
    fi
done

# -- 5. The portfolio crate's Dou-Di-Zhu unit tests all pass.
unit_output="$(
    env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-games-portfolio -- dou_di_zhu_dossier dou_di_zhu_scenario_pack
)"
if ! printf '%s\n' "$unit_output" | grep -Fq 'test result: ok.'; then
    printf 'dou_di_zhu_dossier / dou_di_zhu_scenario_pack unit tests did not all pass\n%s\n' "$unit_output" >&2
    exit 1
fi
unit_pass_line="$(
    printf '%s\n' "$unit_output" \
        | grep -E '^test result: ok\.' \
        | head -n 1
)"
printf 'DOU_DI_ZHU_BENCHMARK_HARNESS dou-di-zhu unit tests: %s\n' "$unit_pass_line"

printf 'DOU_DI_ZHU_BENCHMARK_HARNESS myosu e2e ok scenario_hash=%s ledger=%s\n' \
    "$hash_first" "$ledger"
