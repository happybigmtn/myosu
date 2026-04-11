#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""

portfolio_games=()
strength_games=()

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

trap cleanup EXIT

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/research-portfolio.XXXXXX")"

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

strength_manifest="$(
  run_logged \
    strength_manifest \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_manifest -- \
    slugs
)"
mapfile -t strength_games < <(printf '%s\n' "$strength_manifest")
if [[ "${#strength_games[@]}" -ne 20 ]]; then
  echo "expected 20 typed strength research games, got ${#strength_games[@]}" >&2
  printf '%s\n' "${strength_games[@]}" >&2
  exit 1
fi

strength_table="$(
  run_logged \
    strength_manifest_table \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_manifest -- \
    table
)"
assert_contains "$strength_table" "STRENGTH_GAMES total=20" "strength_manifest_table"
assert_contains "$strength_table" "STRENGTH_GAME slug=bridge" "strength_manifest_table"
assert_contains "$strength_table" "engine_tier=rule-aware" "strength_manifest_table"
if printf '%s\n' "$strength_table" | grep -Fq "engine_tier=static-baseline"; then
  echo "strength manifest should not route portfolio games through static baseline" >&2
  printf '%s\n' "$strength_table" >&2
  exit 1
fi

run_logged \
  portfolio_crate_tests \
  cargo test --quiet -p myosu-games-portfolio

run_logged \
  portfolio_miner_tests \
  env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-miner portfolio

run_logged \
  portfolio_validator_tests \
  env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-validator portfolio

for game in "${portfolio_games[@]}"; do
  query_file="$work_root/${game}/query.bin"
  output="$(
    run_logged \
      "bootstrap_${game//-/_}" \
      env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_query -- \
      "$game" "$query_file"
  )"
  assert_contains "$output" "BOOTSTRAP game=${game}" "bootstrap_${game}"
  assert_contains "$output" "BOOTSTRAP query_file=${query_file}" "bootstrap_${game}"
  if [[ ! -s "$query_file" ]]; then
    echo "expected non-empty query file for ${game}: ${query_file}" >&2
    exit 1
  fi

  roundtrip_dir="$work_root/${game}/roundtrip"
  roundtrip_output="$(
    run_logged \
      "roundtrip_${game//-/_}" \
      env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_roundtrip -- \
      "$game" "$roundtrip_dir" 3
  )"
  assert_contains "$roundtrip_output" "BOOTSTRAP game=${game}" "roundtrip_${game}"
  assert_contains "$roundtrip_output" "BOOTSTRAP exact_match=true" "roundtrip_${game}"
  assert_contains "$roundtrip_output" "BOOTSTRAP score=1.000000" "roundtrip_${game}"
  for artifact in checkpoint.bin query.bin response.bin; do
    if [[ ! -s "$roundtrip_dir/$artifact" ]]; then
      echo "expected non-empty ${artifact} for ${game}: ${roundtrip_dir}/${artifact}" >&2
      exit 1
    fi
  done

  validation_output="$(
    run_logged \
      "validate_${game//-/_}" \
      env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_validate -- \
      "$game" \
      "$roundtrip_dir/checkpoint.bin" \
      "$roundtrip_dir/query.bin" \
      "$roundtrip_dir/response.bin"
  )"
  assert_contains "$validation_output" "VALIDATION game=${game}" "validate_${game}"
  assert_contains "$validation_output" "VALIDATION exact_match=true" "validate_${game}"
  assert_contains "$validation_output" "VALIDATION score=1.000000" "validate_${game}"

  strength_roundtrip_dir="$work_root/${game}/strength"
  strength_roundtrip_output="$(
    run_logged \
      "strength_${game//-/_}" \
      env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example strength_roundtrip -- \
      "$game" "$strength_roundtrip_dir" 3
  )"
  assert_contains "$strength_roundtrip_output" "STRENGTH game=${game}" "strength_${game}"
  assert_contains "$strength_roundtrip_output" "STRENGTH exact_match=true" "strength_${game}"
  assert_contains "$strength_roundtrip_output" "STRENGTH deterministic=true" "strength_${game}"
  for artifact in checkpoint.bin strength-query.bin strength-response.bin; do
    if [[ ! -s "$strength_roundtrip_dir/$artifact" ]]; then
      echo "expected non-empty ${artifact} for ${game}: ${strength_roundtrip_dir}/${artifact}" >&2
      exit 1
    fi
  done
done

mismatch_dir="$work_root/mismatch"
run_logged \
  mismatch_bridge_roundtrip \
  env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_roundtrip -- \
  bridge "$mismatch_dir/bridge" 3 >/dev/null
run_logged \
  mismatch_cribbage_query \
  env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_query -- \
  cribbage "$mismatch_dir/cribbage/query.bin" >/dev/null
mismatch_output="$(
  run_expect_failure \
    mismatch_rejects_cross_game_query \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_validate -- \
    bridge \
    "$mismatch_dir/bridge/checkpoint.bin" \
    "$mismatch_dir/cribbage/query.bin" \
    "$mismatch_dir/bridge/response.bin"
)"
assert_contains \
  "$mismatch_output" \
  'query game mismatch: expected `bridge`, got `cribbage`' \
  "mismatch_rejects_cross_game_query"

echo "RESEARCH_PORTFOLIO_HARNESS myosu e2e ok"
echo "portfolio_games=${portfolio_games[*]}"
