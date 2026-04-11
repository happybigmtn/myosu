#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""

portfolio_games=()

cleanup() {
  if [[ -n "$work_root" ]]; then
    rm -rf "$work_root"
  fi
}

run_logged() {
  local label="$1"
  shift
  local stdout_file="$work_root/${label}.stdout"
  local stderr_file="$work_root/${label}.stderr"

  if (cd "$repo_root" && "$@" >"$stdout_file" 2>"$stderr_file"); then
    cat "$stdout_file"
    return 0
  fi

  echo "${label} failed" >&2
  if [[ -s "$stdout_file" ]]; then
    echo "--- ${label} stdout ---" >&2
    cat "$stdout_file" >&2
  fi
  if [[ -s "$stderr_file" ]]; then
    echo "--- ${label} stderr ---" >&2
    cat "$stderr_file" >&2
  fi
  exit 1
}

run_expect_failure() {
  local label="$1"
  shift
  local stdout_file="$work_root/${label}.stdout"
  local stderr_file="$work_root/${label}.stderr"

  if (cd "$repo_root" && "$@" >"$stdout_file" 2>"$stderr_file"); then
    echo "${label} unexpectedly succeeded" >&2
    if [[ -s "$stdout_file" ]]; then
      echo "--- ${label} stdout ---" >&2
      cat "$stdout_file" >&2
    fi
    if [[ -s "$stderr_file" ]]; then
      echo "--- ${label} stderr ---" >&2
      cat "$stderr_file" >&2
    fi
    exit 1
  fi

  cat "$stdout_file"
  cat "$stderr_file"
}

assert_contains() {
  local blob="$1"
  local needle="$2"
  local label="$3"
  if ! printf '%s\n' "$blob" | grep -Fq "$needle"; then
    echo "${label} missing expected text: ${needle}" >&2
    printf '%s\n' "$blob" >&2
    exit 1
  fi
}

assert_nonempty_file() {
  local path="$1"
  local label="$2"
  if [[ ! -s "$path" ]]; then
    echo "expected non-empty ${label}: ${path}" >&2
    exit 1
  fi
}

assert_positive_value_line() {
  local blob="$1"
  local prefix="$2"
  local label="$3"
  local value
  value="$(printf '%s\n' "$blob" | awk -F= -v prefix="$prefix" '$1 == prefix {print $2; exit}')"
  if [[ -z "$value" || "$value" -le 0 ]]; then
    echo "${label} expected positive ${prefix}, got `${value}`" >&2
    printf '%s\n' "$blob" >&2
    exit 1
  fi
}

assert_positive_float_line() {
  local blob="$1"
  local prefix="$2"
  local label="$3"
  local value
  value="$(printf '%s\n' "$blob" | awk -F= -v prefix="$prefix" '$1 == prefix {print $2; exit}')"
  if [[ -z "$value" ]] || ! awk -v value="$value" 'BEGIN { exit !(value + 0 > 0) }'; then
    echo "${label} expected positive ${prefix}, got \`${value}\`" >&2
    printf '%s\n' "$blob" >&2
    exit 1
  fi
}

trap cleanup EXIT

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/research-strength.XXXXXX")"

portfolio_manifest="$(
  run_logged \
    portfolio_manifest \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_manifest -- \
    portfolio-slugs
)"
mapfile -t portfolio_games < <(printf '%s\n' "$portfolio_manifest")
if [[ "${#portfolio_games[@]}" -ne 20 ]]; then
  echo "expected 20 portfolio-routed research games, got ${#portfolio_games[@]}" >&2
  printf '%s\n' "${portfolio_games[@]}" >&2
  exit 1
fi

strength_table="$(
  run_logged \
    strength_manifest_table \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_manifest -- \
    table
)"
assert_contains "$strength_table" "STRENGTH_GAMES total=20" "strength_manifest_table"
if printf '%s\n' "$strength_table" | grep -Fq "engine_tier=static-baseline"; then
  echo "strength manifest should not include static-baseline portfolio engines" >&2
  printf '%s\n' "$strength_table" >&2
  exit 1
fi

for game in "${portfolio_games[@]}"; do
  roundtrip_dir="$work_root/${game}/strength"
  output="$(
    run_logged \
      "strength_${game//-/_}" \
      env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_roundtrip -- \
      "$game" "$roundtrip_dir" 16
  )"
  assert_contains "$output" "STRENGTH game=${game}" "strength_${game}"
  assert_contains "$output" "STRENGTH deterministic=true" "strength_${game}"
  assert_contains "$output" "STRENGTH score=" "strength_${game}"
  assert_contains "$output" "STRENGTH engine_tier=rule-aware" "strength_${game}"
  assert_positive_value_line "$output" "STRENGTH legal_actions" "strength_${game}"
  assert_nonempty_file "$roundtrip_dir/checkpoint.bin" "${game} strength checkpoint file"
  assert_nonempty_file "$roundtrip_dir/strength-query.bin" "${game} strength query file"
  assert_nonempty_file "$roundtrip_dir/strength-response.bin" "${game} strength response file"
done

heads_up_strength_dir="$work_root/nlhe-heads-up/strength"
heads_up_output="$(
  run_logged \
    nlhe_heads_up_strength \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example strength_roundtrip -- \
    "$heads_up_strength_dir" 0
)"
assert_contains "$heads_up_output" "STRENGTH game=nlhe-heads-up" "nlhe_heads_up_strength"
assert_contains "$heads_up_output" "STRENGTH exact_match=true" "nlhe_heads_up_strength"
assert_contains "$heads_up_output" "STRENGTH engine_tier=dedicated-sparse-blueprint" "nlhe_heads_up_strength"
assert_contains "$heads_up_output" "STRENGTH benchmark_surface=repo-owned-reference-pack" "nlhe_heads_up_strength"
assert_contains "$heads_up_output" "STRENGTH benchmark_scenarios=80" "nlhe_heads_up_strength"
assert_contains "$heads_up_output" "STRENGTH benchmark_unique_queries=73" "nlhe_heads_up_strength"
assert_positive_float_line "$heads_up_output" "STRENGTH benchmark_mean_l1_distance" "nlhe_heads_up_strength"
assert_contains "$heads_up_output" "STRENGTH benchmark_action_agreement=" "nlhe_heads_up_strength"
assert_positive_value_line "$heads_up_output" "STRENGTH legal_actions" "nlhe_heads_up_strength"
assert_nonempty_file "$heads_up_strength_dir/checkpoint.bin" "NLHE heads-up strength checkpoint file"
assert_nonempty_file "$heads_up_strength_dir/strength-query.bin" "NLHE heads-up strength query file"
assert_nonempty_file "$heads_up_strength_dir/strength-response.bin" "NLHE heads-up strength response file"

heads_up_benchmark_dir="$work_root/nlhe-heads-up/benchmark"
heads_up_benchmark_output="$(
  run_logged \
    nlhe_heads_up_benchmark \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example benchmark_scenario_pack -- \
    "$heads_up_benchmark_dir"
)"
assert_contains "$heads_up_benchmark_output" "BENCHMARK game=nlhe-heads-up" "nlhe_heads_up_benchmark"
assert_contains "$heads_up_benchmark_output" "BENCHMARK candidate_source=bootstrap-zero-checkpoint" "nlhe_heads_up_benchmark"
assert_contains "$heads_up_benchmark_output" "BENCHMARK benchmark_surface=repo-owned-reference-pack" "nlhe_heads_up_benchmark"
assert_contains "$heads_up_benchmark_output" "BENCHMARK engine_tier=dedicated-reference-pack" "nlhe_heads_up_benchmark"
assert_contains "$heads_up_benchmark_output" "BENCHMARK scenario_count=80" "nlhe_heads_up_benchmark"
assert_contains "$heads_up_benchmark_output" "BENCHMARK unique_query_count=73" "nlhe_heads_up_benchmark"
assert_positive_float_line "$heads_up_benchmark_output" "BENCHMARK mean_l1_distance" "nlhe_heads_up_benchmark"
assert_nonempty_file "$heads_up_benchmark_dir/candidate-checkpoint.bin" "NLHE heads-up benchmark candidate checkpoint file"
assert_nonempty_file "$heads_up_benchmark_dir/reference-checkpoint.bin" "NLHE heads-up benchmark reference checkpoint file"

liars_dice_strength_dir="$work_root/liars-dice/strength"
liars_dice_output="$(
  run_logged \
    liars_dice_strength \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-liars-dice --example strength_roundtrip -- \
    "$liars_dice_strength_dir" 64
)"
assert_contains "$liars_dice_output" "STRENGTH game=liars-dice" "liars_dice_strength"
assert_contains "$liars_dice_output" "STRENGTH exact_match=true" "liars_dice_strength"
assert_contains "$liars_dice_output" "STRENGTH engine_tier=dedicated-mccfr" "liars_dice_strength"
assert_positive_value_line "$liars_dice_output" "STRENGTH legal_actions" "liars_dice_strength"
assert_nonempty_file "$liars_dice_strength_dir/checkpoint.bin" "Liar's Dice strength checkpoint file"
assert_nonempty_file "$liars_dice_strength_dir/strength-query.bin" "Liar's Dice strength query file"
assert_nonempty_file "$liars_dice_strength_dir/strength-response.bin" "Liar's Dice strength response file"

kuhn_roundtrip_dir="$work_root/kuhn/roundtrip"
kuhn_output="$(
  run_logged \
    kuhn_roundtrip \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-kuhn --example bootstrap_roundtrip -- \
    "$kuhn_roundtrip_dir" 0
)"
assert_contains "$kuhn_output" "BOOTSTRAP game=kuhn" "kuhn_roundtrip"
assert_contains "$kuhn_output" "BOOTSTRAP exact_match=true" "kuhn_roundtrip"
assert_nonempty_file "$kuhn_roundtrip_dir/checkpoint.bin" "Kuhn checkpoint file"
assert_nonempty_file "$kuhn_roundtrip_dir/query.bin" "Kuhn query file"
assert_nonempty_file "$kuhn_roundtrip_dir/response.bin" "Kuhn response file"

mismatch_dir="$work_root/mismatch"
run_logged \
  mismatch_bridge_strength \
  env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_roundtrip -- \
  bridge "$mismatch_dir/bridge" 8 >/dev/null
run_logged \
  mismatch_cribbage_query \
  env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_query -- \
  cribbage "$mismatch_dir/cribbage/strength-query.bin" >/dev/null
bridge_cribbage_output="$(
  run_expect_failure \
    bridge_rejects_cribbage_strength_query \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_validate -- \
    cribbage \
    "$mismatch_dir/bridge/checkpoint.bin" \
    "$mismatch_dir/cribbage/strength-query.bin" \
    "$mismatch_dir/bridge/strength-response.bin"
)"
assert_contains \
  "$bridge_cribbage_output" \
  'portfolio checkpoint game `bridge` cannot answer query game `cribbage`' \
  "bridge_rejects_cribbage_strength_query"

run_logged \
  mismatch_tien_len_query \
  env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_query -- \
  tien-len "$mismatch_dir/tien-len/strength-query.bin" >/dev/null
tien_len_pusoy_output="$(
  run_expect_failure \
    pusoy_rejects_tien_len_strength_query \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_validate -- \
    pusoy-dos \
    "$work_root/pusoy-dos/strength/checkpoint.bin" \
    "$mismatch_dir/tien-len/strength-query.bin" \
    "$work_root/pusoy-dos/strength/strength-response.bin"
)"
assert_contains \
  "$tien_len_pusoy_output" \
  'strength query game mismatch: expected `pusoy-dos`, got `tien-len`' \
  "pusoy_rejects_tien_len_strength_query"

run_logged \
  mismatch_short_deck_query \
  env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_query -- \
  short-deck "$mismatch_dir/short-deck/strength-query.bin" >/dev/null
short_deck_nlhe_output="$(
  run_expect_failure \
    nlhe_six_max_rejects_short_deck_strength_query \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_validate -- \
    nlhe-six-max \
    "$work_root/nlhe-six-max/strength/checkpoint.bin" \
    "$mismatch_dir/short-deck/strength-query.bin" \
    "$work_root/nlhe-six-max/strength/strength-response.bin"
)"
assert_contains \
  "$short_deck_nlhe_output" \
  'strength query game mismatch: expected `nlhe-six-max`, got `short-deck`' \
  "nlhe_six_max_rejects_short_deck_strength_query"

bad_query_file="$mismatch_dir/bad/strength-query.bin"
mkdir -p "$(dirname "$bad_query_file")"
printf 'not-a-valid-strength-query' >"$bad_query_file"
malformed_output="$(
  run_expect_failure \
    malformed_strength_query_rejected \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_validate -- \
    bridge \
    "$mismatch_dir/bridge/checkpoint.bin" \
    "$bad_query_file" \
    "$mismatch_dir/bridge/strength-response.bin"
)"
assert_contains "$malformed_output" "failed to decode portfolio strength query" "malformed_strength_query_rejected"

echo "RESEARCH_STRENGTH_HARNESS myosu e2e ok"
echo "portfolio_games=${portfolio_games[*]}"
