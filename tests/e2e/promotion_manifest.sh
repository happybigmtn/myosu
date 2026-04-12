#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

ledger_path="${MYOSU_SOLVER_PROMOTION_LEDGER:-$repo_root/ops/solver_promotion.yaml}"

manifest="$(
  MYOSU_SOLVER_PROMOTION_LEDGER="$ledger_path" \
    SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-canonical \
      --example promotion_manifest -- table
)"

rank_tier() {
  case "$1" in
    routed) printf '0\n' ;;
    benchmarked) printf '1\n' ;;
    promotable_local) printf '2\n' ;;
    promotable_funded) printf '3\n' ;;
    *)
      printf 'unknown promotion tier: %s\n' "$1" >&2
      return 1
      ;;
  esac
}
promotable_local_rank="$(rank_tier promotable_local)"

field_value() {
  local line="$1"
  local key="$2"
  local token

  for token in $line; do
    if [[ "$token" == "$key="* ]]; then
      printf '%s\n' "${token#*=}"
      return 0
    fi
  done

  printf 'missing field %s in line: %s\n' "$key" "$line" >&2
  return 1
}

assert_promotion_outputs() {
  local slug="$1"
  local dir="$repo_root/outputs/solver-promotion/$slug"
  local file

  for file in bundle.json benchmark-summary.json artifact-manifest.json; do
    if [[ ! -s "$dir/$file" ]]; then
      printf '%s declares promotable_local but is missing %s\n' "$slug" "$dir/$file" >&2
      exit 1
    fi
  done
}

assert_tier_at_least() {
  local slug="$1"
  local expected="$2"
  local line tier tier_rank expected_rank

  line="$(printf '%s\n' "$manifest" | grep -E "^SOLVER_PROMOTION_GAME slug=${slug} " || true)"
  if [[ -z "$line" ]]; then
    printf 'promotion manifest missing required game: %s\n' "$slug" >&2
    printf '%s\n' "$manifest" >&2
    exit 1
  fi

  tier="$(field_value "$line" tier)"
  tier_rank="$(rank_tier "$tier")"
  expected_rank="$(rank_tier "$expected")"
  if (( tier_rank < expected_rank )); then
    printf '%s tier %s is below required minimum %s\n' "$slug" "$tier" "$expected" >&2
    printf '%s\n' "$line" >&2
    exit 1
  fi
}

if ! printf '%s\n' "$manifest" | grep -Fxq 'SOLVER_PROMOTION total=22'; then
  printf 'promotion manifest did not report exactly 22 rows\n' >&2
  printf '%s\n' "$manifest" >&2
  exit 1
fi

row_count="$(printf '%s\n' "$manifest" | grep -c '^SOLVER_PROMOTION_GAME ')"
if [[ "$row_count" -ne 22 ]]; then
  printf 'expected 22 SOLVER_PROMOTION_GAME rows, found %s\n' "$row_count" >&2
  printf '%s\n' "$manifest" >&2
  exit 1
fi

while IFS= read -r line; do
  [[ -n "$line" ]] || continue
  tier="$(field_value "$line" tier)"
  code_support="$(field_value "$line" code_bundle_support)"
  tier_rank="$(rank_tier "$tier")"
  support_rank="$(rank_tier "$code_support")"

  if (( tier_rank > support_rank )); then
    slug="$(field_value "$line" slug)"
    printf '%s declares tier %s above live code support %s\n' \
      "$slug" "$tier" "$code_support" >&2
    printf '%s\n' "$line" >&2
    exit 1
  fi
  if (( tier_rank >= promotable_local_rank )); then
    slug="$(field_value "$line" slug)"
    assert_promotion_outputs "$slug"
  fi
done < <(printf '%s\n' "$manifest" | grep '^SOLVER_PROMOTION_GAME ')

assert_tier_at_least nlhe-heads-up benchmarked
assert_tier_at_least liars-dice promotable_local

printf '%s\n' "$manifest"
printf 'PROMOTION_MANIFEST_HARNESS myosu e2e ok rows=%s ledger=%s\n' \
  "$row_count" \
  "$ledger_path"
