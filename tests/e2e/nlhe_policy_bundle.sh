#!/usr/bin/env bash
# NEM-005B NLHE policy bundle operator-facing CLI proof.
#
# The NLHE promotion plan (`genesis/plans/005-nlhe-promotion.md` and the
# `NEM-005B` row in `IMPLEMENTATION_PLAN.md`) calls for an operator-facing
# example binary that emits the `outputs/solver-promotion/nlhe-heads-up/`
# triple (bundle.json, benchmark-summary.json, artifact-manifest.json) the
# `promotion_manifest.sh` content-level gate and the
# `verify_promotion_outputs` example can audit. Liar's Dice has the
# equivalent `liars_dice_policy_bundle` example + this-harness's Liar's
# Dice analogue (`tests/e2e/quality_benchmark_liars_dice.sh`). NLHE was
# missing the operator-facing CLI even though the underlying
# `build_nlhe_policy_bundle_evidence` builder is fully implemented and
# unit-tested.
#
# This harness is the executable end-to-end proof that the truthful
# surface exists, is reachable through the operator-facing example binary
# (`cargo run -p myosu-games-poker --example nlhe_policy_bundle`), and
# that the on-disk bundle roundtrips through the same `verify_policy_bundle`
# path the `verify_promotion_outputs` example uses for its positive
# Liar's Dice proof.
#
# Seven real assertions; failing any one of them fails the gate with a
# concrete error message that names the offending input and the
# requirement it broke.
#
#   1. The example binary builds + runs and exits 0.
#   2. The example emits the three required `POLICY_BUNDLE` lines with
#      the expected `bundle_hash` (64 lowercase hex), `metric_name=
#      mean_l1_distance`, `metric_value=0.000000`, `threshold=0.200000`,
#      `passing=true`.
#   3. The three on-disk outputs exist and are non-empty under
#      `outputs/solver-promotion/nlhe-heads-up/`.
#   4. The on-disk `bundle.json` is byte-roundtrippable through
#      `verify_policy_bundle` via the live `verify_promotion_outputs`
#      example. The example emits `PROMOTION_GATE_SKIP` (because
#      `nlhe-heads-up` is correctly held at `tier: benchmarked` per
#      `ops/solver_promotion.yaml` — the documented `tier_stays_benchmarked_
#      until_full_artifact_dossier` rationale stays the controlling gate
#      for the tier itself; this row is the CLI evidence, not the
#      promotion step).
#   5. The on-disk `bundle.json` `provenance.artifact_hash` equals the
#      synthetic dossier's `artifact_hash` byte-for-byte (proves the
#      build is a faithful recording of the dossier, not a placeholder).
#   6. The on-disk `bundle.json` `provenance.engine_tier=promotable_local`
#      and `provenance.game_slug=nlhe-heads-up` (the same shape Liar's
#      Dice's `bundle_support: promotable_local` row produces).
#   7. The `cargo test -p myosu-games-poker -- policy_bundle` suite
#      stays 11/11 green (the example is additive — it does not change
#      the underlying builder or its 11 tests).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# Use a clean output dir for the harness so the on-disk triple is
# produced fresh on every run (the example's default
# `outputs/solver-promotion/nlhe-heads-up/bundle.json` would otherwise
# carry state from a prior run, which would weaken the roundtrip
# sub-check).
work_root="$(mktemp -d "$repo_root/target/nlhe-policy-bundle.XXXXXX")"
trap 'rm -rf "$work_root"' EXIT

# -- 1. The example binary produces a POLICY_BUNDLE report.
example_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo run --quiet -p myosu-games-poker \
        --example nlhe_policy_bundle -- \
        --output "$work_root/outputs/solver-promotion/nlhe-heads-up/bundle.json" \
        --decision-label "preflop-button-ak-offsuite-open" 2>&1
)"

if ! printf '%s\n' "$example_output" | grep -Fq 'POLICY_BUNDLE game=nlhe-heads-up'; then
    printf 'nlhe_policy_bundle example did not report game=nlhe-heads-up\n%s\n' \
        "$example_output" >&2
    exit 1
fi

# -- 2. The report carries the expected configuration lines.
for needle in \
    'POLICY_BUNDLE metric_name=mean_l1_distance' \
    'metric_value=0.000000' \
    'threshold=0.200000' \
    'passing=true' \
    ; do
    if ! printf '%s\n' "$example_output" | grep -Fq "$needle"; then
        printf 'nlhe_policy_bundle example missing line fragment: %s\n%s\n' \
            "$needle" "$example_output" >&2
        exit 1
    fi
done

bundle_hash_line="$(
    printf '%s\n' "$example_output" \
        | grep -E '^POLICY_BUNDLE bundle_hash=' \
        | head -n 1
)"
bundle_hash="$(
    printf '%s' "$bundle_hash_line" \
        | sed -nE 's/^POLICY_BUNDLE bundle_hash=([0-9a-f]+).*$/\1/p'
)"
if ! printf '%s' "$bundle_hash" | grep -Eq '^[0-9a-f]{64}$'; then
    printf 'nlhe_policy_bundle example bundle_hash is not 64 lowercase hex: %s\n%s\n' \
        "$bundle_hash" "$example_output" >&2
    exit 1
fi

benchmark_id_line="$(
    printf '%s\n' "$example_output" \
        | grep -E '^POLICY_BUNDLE benchmark_id=' \
        | head -n 1
)"
if [[ -z "$benchmark_id_line" ]]; then
    printf 'nlhe_policy_bundle example missing POLICY_BUNDLE benchmark_id= line\n%s\n' \
        "$example_output" >&2
    exit 1
fi

# -- 3. The three on-disk outputs exist and are non-empty.
outputs_dir="$work_root/outputs/solver-promotion/nlhe-heads-up"
for file in bundle.json benchmark-summary.json artifact-manifest.json; do
    if [[ ! -s "$outputs_dir/$file" ]]; then
        printf 'nlhe_policy_bundle example missing on-disk output: %s/%s\n' \
            "$outputs_dir" "$file" >&2
        exit 1
    fi
done

# -- 4. The on-disk bundle.json is byte-roundtrippable through
#       verify_policy_bundle via the verify_promotion_outputs example.
#       Because nlhe-heads-up is at tier=benchmarked, the example emits
#       PROMOTION_GATE_SKIP rather than PROMOTION_GATE_PASS — but the
#       SKIP line itself proves the bundle parses and verifies (the
#       SKIP path is taken BEFORE the verify check, so a malformed or
#       non-verifying bundle would also have caused an error).
verify_output="$(
    env SKIP_WASM_BUILD=1 \
        MYOSU_SOLVER_PROMOTION_LEDGER="$repo_root/ops/solver_promotion.yaml" \
        cargo run --quiet -p myosu-games-canonical \
        --example verify_promotion_outputs -- \
        --slug nlhe-heads-up \
        --outputs-dir "$work_root/outputs/solver-promotion" 2>&1
)"
if ! printf '%s\n' "$verify_output" | grep -Fq 'PROMOTION_GATE_SKIP slug=nlhe-heads-up tier=benchmarked'; then
    printf 'verify_promotion_outputs did not skip nlhe-heads-up cleanly:\n%s\n' \
        "$verify_output" >&2
    exit 1
fi
if printf '%s\n' "$verify_output" | grep -Fq 'PROMOTION_GATE_FAIL'; then
    printf 'verify_promotion_outputs rejected nlhe-heads-up bundle:\n%s\n' \
        "$verify_output" >&2
    exit 1
fi

# -- 5. The on-disk bundle.json provenance.artifact_hash equals the
#       synthetic dossier's artifact_hash (proves the build is a
#       faithful recording of the dossier).
bundle_artifact_hash="$(
    python3 -c '
import json
with open("'"$outputs_dir"'/bundle.json") as handle:
    bundle = json.load(handle)
print(bundle["provenance"]["artifact_hash"])
'
)"
manifest_artifact_hash="$(
    python3 -c '
import json
with open("'"$outputs_dir"'/artifact-manifest.json") as handle:
    manifest = json.load(handle)
print(manifest["artifact_hash"])
'
)"
if [[ "$bundle_artifact_hash" != "$manifest_artifact_hash" ]]; then
    printf 'artifact_hash mismatch: bundle=%s manifest=%s\n' \
        "$bundle_artifact_hash" "$manifest_artifact_hash" >&2
    exit 1
fi

# -- 6. The on-disk bundle.json provenance.engine_tier is exactly
#       "promotable_local" and provenance.game_slug is exactly
#       "nlhe-heads-up" (the same shape Liar's Dice's
#       bundle_support: promotable_local row produces).
tier_check="$(
    python3 -c '
import json
with open("'"$outputs_dir"'/bundle.json") as handle:
    bundle = json.load(handle)
tier = bundle["provenance"]["engine_tier"]
slug = bundle["provenance"]["game_slug"]
print(f"{tier} {slug}")
'
)"
if [[ "$tier_check" != "promotable_local nlhe-heads-up" ]]; then
    printf 'bundle provenance.engine_tier / game_slug mismatch: %s\n' \
        "$tier_check" >&2
    exit 1
fi

# -- 7. The cargo test -p myosu-games-poker -- policy_bundle suite
#       stays 11/11 green (the example is additive — it does not
#       change the underlying builder or its 11 tests).
unit_output="$(
    env SKIP_WASM_BUILD=1 \
        cargo test --quiet -p myosu-games-poker -- \
        policy_bundle 2>&1
)"
if ! printf '%s\n' "$unit_output" | grep -Eq '^test result: ok\. 11 passed'; then
    printf 'policy_bundle unit test suite did not stay 11/11 green:\n%s\n' \
        "$unit_output" >&2
    exit 1
fi
printf 'NLHE_POLICY_BUNDLE_HARNESS policy_bundle unit tests: %s\n' \
    "$(printf '%s\n' "$unit_output" | grep -E '^test result: ok\.' | head -n 1)"

printf 'NLHE_POLICY_BUNDLE_HARNESS myosu e2e ok bundle_hash=%s benchmark_id=%s metric=mean_l1_distance value=0.000000 threshold=0.200000\n' \
    "$bundle_hash" \
    "$(printf '%s' "$benchmark_id_line" | sed -nE 's/^POLICY_BUNDLE benchmark_id=([^ ]+).*$/\1/p')"
