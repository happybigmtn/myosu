#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""

dedicated_research_games=()
portfolio_routed_games=()
research_games=()

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

trap cleanup EXIT

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/research-games.XXXXXX")"

research_manifest="$(
  run_logged \
    research_manifest_all \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_manifest -- \
    all-slugs
)"
mapfile -t research_games < <(printf '%s\n' "$research_manifest")
if [[ "${#research_games[@]}" -ne 22 ]]; then
  echo "expected 22 distinct research game identities, got ${#research_games[@]}" >&2
  printf '%s\n' "${research_games[@]}" >&2
  exit 1
fi

portfolio_manifest="$(
  run_logged \
    research_manifest_portfolio \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_manifest -- \
    portfolio-slugs
)"
mapfile -t portfolio_routed_games < <(printf '%s\n' "$portfolio_manifest")
if [[ "${#portfolio_routed_games[@]}" -ne 20 ]]; then
  echo "expected 20 portfolio-routed research games, got ${#portfolio_routed_games[@]}" >&2
  printf '%s\n' "${portfolio_routed_games[@]}" >&2
  exit 1
fi

dedicated_manifest="$(
  run_logged \
    research_manifest_dedicated \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_manifest -- \
    dedicated-slugs
)"
mapfile -t dedicated_research_games < <(printf '%s\n' "$dedicated_manifest")
if [[ "${#dedicated_research_games[@]}" -ne 2 ]]; then
  echo "expected 2 dedicated research games, got ${#dedicated_research_games[@]}" >&2
  printf '%s\n' "${dedicated_research_games[@]}" >&2
  exit 1
fi

research_table="$(
  run_logged \
    research_manifest_table \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_manifest -- \
    table
)"
assert_contains "$research_table" "RESEARCH_GAMES total=22" "research_manifest_table"
assert_contains "$research_table" "RESEARCH_GAME slug=nlhe-heads-up chain_id=nlhe_hu route=dedicated players=2 rule_file=research/game-rules/01-nlhe-heads-up.md" "research_manifest_table"
assert_contains "$research_table" "RESEARCH_GAME slug=liars-dice chain_id=liars_dice route=dedicated players=2 rule_file=research/game-rules/15-liars-dice.md" "research_manifest_table"
assert_contains "$research_table" "RESEARCH_GAME slug=hearts chain_id=hearts route=portfolio players=4 rule_file=research/game-rules/21-hearts-cribbage.md" "research_manifest_table"
assert_contains "$research_table" "RESEARCH_GAME slug=cribbage chain_id=cribbage route=portfolio players=2 rule_file=research/game-rules/21-hearts-cribbage.md" "research_manifest_table"

run_logged \
  research_portfolio_harness \
  bash tests/e2e/research_portfolio_harness.sh

run_logged \
  research_strength_harness \
  bash tests/e2e/research_strength_harness.sh

run_logged \
  miner_heads_up_slug_test \
  env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-miner \
  cli_parses_research_slug_for_heads_up_poker

run_logged \
  validator_heads_up_slug_test \
  env SKIP_WASM_BUILD=1 cargo test --quiet -p myosu-validator \
  cli_parses_research_slug_for_heads_up_poker

run_logged \
  play_binary_build \
  env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-play

heads_up_encoder_dir="$work_root/nlhe-heads-up/encoder"
heads_up_query_file="$work_root/nlhe-heads-up/query.bin"
heads_up_output="$(
  run_logged \
    nlhe_heads_up_bootstrap_artifacts \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
    "$heads_up_encoder_dir" "$heads_up_query_file"
)"
assert_contains "$heads_up_output" "BOOTSTRAP encoder_dir=${heads_up_encoder_dir}" "nlhe_heads_up_bootstrap_artifacts"
assert_contains "$heads_up_output" "BOOTSTRAP query_file=${heads_up_query_file}" "nlhe_heads_up_bootstrap_artifacts"
assert_nonempty_file "$heads_up_query_file" "NLHE heads-up query file"

heads_up_scenario_pack_dir="$work_root/nlhe-heads-up/scenario-pack"
heads_up_scenario_pack_output="$(
  run_logged     nlhe_heads_up_scenario_pack     env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_scenario_pack --     "$heads_up_scenario_pack_dir"
)"
assert_contains "$heads_up_scenario_pack_output" "SCENARIO_PACK scenario_count=80" "nlhe_heads_up_scenario_pack"
assert_contains "$heads_up_scenario_pack_output" "SCENARIO_PACK by_street=preflop=8,flop=24,turn=24,river=24" "nlhe_heads_up_scenario_pack"
assert_contains "$heads_up_scenario_pack_output" "SCENARIO_PACK unique_query_keys=73" "nlhe_heads_up_scenario_pack"

liars_dice_query_file="$work_root/liars-dice/query.bin"
liars_dice_output="$(
  run_logged \
    liars_dice_bootstrap_query \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-liars-dice --example bootstrap_query -- \
    "$liars_dice_query_file"
)"
assert_contains "$liars_dice_output" "BOOTSTRAP query_file=${liars_dice_query_file}" "liars_dice_bootstrap_query"
assert_nonempty_file "$liars_dice_query_file" "Liar's Dice query file"

heads_up_roundtrip_dir="$work_root/nlhe-heads-up/roundtrip"
heads_up_roundtrip_output="$(
  run_logged \
    nlhe_heads_up_roundtrip \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_roundtrip -- \
    "$heads_up_roundtrip_dir" 0
)"
assert_contains "$heads_up_roundtrip_output" "BOOTSTRAP game=nlhe-heads-up" "nlhe_heads_up_roundtrip"
assert_contains "$heads_up_roundtrip_output" "BOOTSTRAP exact_match=true" "nlhe_heads_up_roundtrip"
assert_contains "$heads_up_roundtrip_output" "BOOTSTRAP score=1.000000" "nlhe_heads_up_roundtrip"
assert_nonempty_file "$heads_up_roundtrip_dir/checkpoint.bin" "NLHE heads-up checkpoint file"
assert_nonempty_file "$heads_up_roundtrip_dir/query.bin" "NLHE heads-up roundtrip query file"
assert_nonempty_file "$heads_up_roundtrip_dir/response.bin" "NLHE heads-up roundtrip response file"

liars_dice_roundtrip_dir="$work_root/liars-dice/roundtrip"
liars_dice_roundtrip_output="$(
  run_logged \
    liars_dice_roundtrip \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-liars-dice --example bootstrap_roundtrip -- \
    "$liars_dice_roundtrip_dir" 8
)"
assert_contains "$liars_dice_roundtrip_output" "BOOTSTRAP game=liars-dice" "liars_dice_roundtrip"
assert_contains "$liars_dice_roundtrip_output" "BOOTSTRAP exact_match=true" "liars_dice_roundtrip"
assert_contains "$liars_dice_roundtrip_output" "BOOTSTRAP score=1.000000" "liars_dice_roundtrip"
assert_nonempty_file "$liars_dice_roundtrip_dir/checkpoint.bin" "Liar's Dice checkpoint file"
assert_nonempty_file "$liars_dice_roundtrip_dir/query.bin" "Liar's Dice roundtrip query file"
assert_nonempty_file "$liars_dice_roundtrip_dir/response.bin" "Liar's Dice roundtrip response file"

heads_up_strength_dir="$work_root/nlhe-heads-up/strength"
heads_up_strength_output="$(
  run_logged \
    nlhe_heads_up_strength \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example strength_roundtrip -- \
    "$heads_up_strength_dir" 0
)"
assert_contains "$heads_up_strength_output" "STRENGTH game=nlhe-heads-up" "nlhe_heads_up_strength"
assert_contains "$heads_up_strength_output" "STRENGTH exact_match=true" "nlhe_heads_up_strength"
assert_contains "$heads_up_strength_output" "STRENGTH engine_tier=dedicated-sparse-blueprint" "nlhe_heads_up_strength"
assert_contains "$heads_up_strength_output" "STRENGTH benchmark_surface=repo-owned-reference-pack" "nlhe_heads_up_strength"
assert_contains "$heads_up_strength_output" "STRENGTH benchmark_scenarios=80" "nlhe_heads_up_strength"
assert_contains "$heads_up_strength_output" "STRENGTH benchmark_unique_queries=73" "nlhe_heads_up_strength"
assert_nonempty_file "$heads_up_strength_dir/checkpoint.bin" "NLHE heads-up strength checkpoint file"
assert_nonempty_file "$heads_up_strength_dir/strength-query.bin" "NLHE heads-up strength query file"
assert_nonempty_file "$heads_up_strength_dir/strength-response.bin" "NLHE heads-up strength response file"

liars_dice_strength_dir="$work_root/liars-dice/strength"
liars_dice_strength_output="$(
  run_logged \
    liars_dice_strength \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-liars-dice --example strength_roundtrip -- \
    "$liars_dice_strength_dir" 64
)"
assert_contains "$liars_dice_strength_output" "STRENGTH game=liars-dice" "liars_dice_strength"
assert_contains "$liars_dice_strength_output" "STRENGTH exact_match=true" "liars_dice_strength"
assert_contains "$liars_dice_strength_output" "STRENGTH engine_tier=dedicated-mccfr" "liars_dice_strength"
assert_nonempty_file "$liars_dice_strength_dir/checkpoint.bin" "Liar's Dice strength checkpoint file"
assert_nonempty_file "$liars_dice_strength_dir/strength-query.bin" "Liar's Dice strength query file"
assert_nonempty_file "$liars_dice_strength_dir/strength-response.bin" "Liar's Dice strength response file"

kuhn_roundtrip_dir="$work_root/kuhn/roundtrip"
kuhn_roundtrip_output="$(
  run_logged \
    kuhn_roundtrip \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-kuhn --example bootstrap_roundtrip -- \
    "$kuhn_roundtrip_dir" 0
)"
assert_contains "$kuhn_roundtrip_output" "BOOTSTRAP game=kuhn" "kuhn_roundtrip"
assert_contains "$kuhn_roundtrip_output" "BOOTSTRAP exact_match=true" "kuhn_roundtrip"
assert_contains "$kuhn_roundtrip_output" "BOOTSTRAP score=1.000000" "kuhn_roundtrip"
assert_nonempty_file "$kuhn_roundtrip_dir/checkpoint.bin" "Kuhn checkpoint file"
assert_nonempty_file "$kuhn_roundtrip_dir/query.bin" "Kuhn roundtrip query file"
assert_nonempty_file "$kuhn_roundtrip_dir/response.bin" "Kuhn roundtrip response file"

portfolio_checkpoint_file="$work_root/bridge/checkpoint.bin"
portfolio_checkpoint_output="$(
  run_logged \
    bridge_bootstrap_checkpoint \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-portfolio --example bootstrap_checkpoint -- \
    bridge "$portfolio_checkpoint_file" 3
)"
assert_contains "$portfolio_checkpoint_output" "BOOTSTRAP game=bridge" "bridge_bootstrap_checkpoint"
assert_contains "$portfolio_checkpoint_output" "BOOTSTRAP checkpoint_file=${portfolio_checkpoint_file}" "bridge_bootstrap_checkpoint"
assert_contains "$portfolio_checkpoint_output" "BOOTSTRAP iterations=3" "bridge_bootstrap_checkpoint"
assert_nonempty_file "$portfolio_checkpoint_file" "Bridge portfolio checkpoint file"

bridge_artifact_smoke_output="$(
  run_logged \
    bridge_play_artifact_smoke \
    env SKIP_WASM_BUILD=1 "$repo_root/target/debug/myosu-play" \
    --game bridge \
    --smoke-test \
    --require-artifact \
    --smoke-checkpoint "$portfolio_checkpoint_file"
)"
assert_contains "$bridge_artifact_smoke_output" "SMOKE myosu-play ok" "bridge_play_artifact_smoke"
assert_contains "$bridge_artifact_smoke_output" "game=bridge" "bridge_play_artifact_smoke"
assert_contains "$bridge_artifact_smoke_output" "advice_source=artifact" "bridge_play_artifact_smoke"

for game in "${research_games[@]}"; do
  output="$(
    run_logged \
      "play_smoke_${game//-/_}" \
      env SKIP_WASM_BUILD=1 "$repo_root/target/debug/myosu-play" --game "$game" --smoke-test
  )"
  assert_contains "$output" "SMOKE myosu-play ok" "play_smoke_${game}"
  assert_contains "$output" "final_state=" "play_smoke_${game}"
done

echo "RESEARCH_GAMES_HARNESS myosu e2e ok"
echo "research_rule_files=21"
echo "research_games=${research_games[*]}"
echo "dedicated_research_games=${dedicated_research_games[*]}"
echo "portfolio_routed_games=${portfolio_routed_games[*]}"
