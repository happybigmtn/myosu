#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
helpers_dir="$repo_root/tests/e2e/helpers"
work_parent="$repo_root/target/e2e"
driver_examples_dir="$repo_root/crates/myosu-chain-client/examples"
driver_example_name="validator_determinism_driver_$$"
driver_source="$driver_examples_dir/${driver_example_name}.rs"
work_root=""
started_devnet=0

chain_endpoint="${MYOSU_E2E_CHAIN_ENDPOINT:-ws://127.0.0.1:9955}"
owner_key="${MYOSU_E2E_OWNER_KEY:-//Alice}"
miner_key="${MYOSU_E2E_MINER_KEY:-//Alice}"
validator_a_key="${MYOSU_E2E_VALIDATOR_A_KEY:-//Bob}"
validator_b_key="${MYOSU_E2E_VALIDATOR_B_KEY:-//Charlie}"
validator_stake="${MYOSU_E2E_VALIDATOR_STAKE:-100000000000000}"
score_epsilon="${MYOSU_E2E_SCORE_EPSILON:-0.000001}"
requested_games_string="${MYOSU_E2E_GAMES:-poker liars-dice}"

read -r -a requested_games <<<"$requested_games_string"

if [[ ${#requested_games[@]} -eq 0 ]]; then
  echo "expected at least one game in MYOSU_E2E_GAMES" >&2
  exit 1
fi

cleanup() {
  if [[ -n "$driver_source" && -f "$driver_source" ]]; then
    rm -f "$driver_source"
  fi

  if (( started_devnet )); then
    bash "$helpers_dir/stop_devnet.sh" >/dev/null 2>&1 || true
  fi
}

require_kv() {
  local blob="$1"
  local key="$2"
  local value
  value="$(printf '%s\n' "$blob" | sed -n "s/^${key}=//p" | tail -n1)"
  if [[ -z "$value" ]]; then
    echo "missing output key ${key}" >&2
    printf '%s\n' "$blob" >&2
    exit 1
  fi
  printf '%s\n' "$value"
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

assert_equal() {
  local left="$1"
  local right="$2"
  local label="$3"
  if [[ "$left" != "$right" ]]; then
    echo "${label} mismatch" >&2
    echo "left=${left}" >&2
    echo "right=${right}" >&2
    exit 1
  fi
}

assert_within_epsilon() {
  local left="$1"
  local right="$2"
  local epsilon="$3"
  local label="$4"

  python - "$left" "$right" "$epsilon" "$label" <<'PY'
import math
import sys

left = float(sys.argv[1])
right = float(sys.argv[2])
epsilon = float(sys.argv[3])
label = sys.argv[4]

if not math.isfinite(left) or not math.isfinite(right):
    print(f"{label} contained non-finite values: {left} vs {right}", file=sys.stderr)
    sys.exit(1)

delta = abs(left - right)
if delta > epsilon:
    print(
        f"{label} diverged beyond epsilon: {left} vs {right} (delta={delta}, epsilon={epsilon})",
        file=sys.stderr,
    )
    sys.exit(1)
PY
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

run_chain_driver() {
  local command="$1"
  shift
  run_logged \
    "chain_driver_${command}" \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-chain-client --example "$driver_example_name" -- \
    "$command" "$chain_endpoint" "$@"
}

select_local_port() {
  python - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
    listener.bind(("127.0.0.1", 0))
    print(listener.getsockname()[1])
PY
}

validator_game_name() {
  local game_key="$1"
  case "$game_key" in
    poker)
      printf 'Poker\n'
      ;;
    liars-dice)
      printf 'LiarsDice\n'
      ;;
    *)
      echo "unsupported validator determinism game: ${game_key}" >&2
      exit 1
      ;;
  esac
}

run_game_case() {
  local game_key="$1"
  local game_slug="${game_key//-/_}"
  local expected_game_name
  expected_game_name="$(validator_game_name "$game_key")"

  local game_root="$work_root/$game_slug"
  local encoder_dir="$game_root/encoder"
  local query_file="$game_root/query.bin"
  local response_file="$game_root/response.bin"
  local miner_data_dir="$game_root/miner-data"
  local checkpoint_path="$miner_data_dir/checkpoints/latest.bin"
  local miner_port="${MYOSU_E2E_MINER_PORT:-$(select_local_port)}"

  local miner_game_args=()
  local validator_game_args=()
  case "$game_key" in
    poker)
      ;;
    liars-dice)
      validator_game_args=(--game "$game_key")
      miner_game_args=(--game "$game_key")
      encoder_dir=""
      ;;
  esac

  echo "registering ${game_key} subnet"
  local register_output
  register_output="$(run_chain_driver register_subnet "$owner_key")"
  local subnet
  subnet="$(require_kv "$register_output" "subnet")"

  echo "writing ${game_key} bootstrap artifacts"
  local bootstrap_output
  case "$game_key" in
    poker)
      bootstrap_output="$(
        run_logged \
          "${game_slug}_bootstrap_artifacts" \
          env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
          "$encoder_dir" "$query_file"
      )"
      assert_contains "$bootstrap_output" "BOOTSTRAP encoder_dir=${encoder_dir}" "${game_slug}_bootstrap_artifacts"
      assert_contains "$bootstrap_output" "BOOTSTRAP query_file=${query_file}" "${game_slug}_bootstrap_artifacts"
      ;;
    liars-dice)
      bootstrap_output="$(
        run_logged \
          "${game_slug}_bootstrap_query" \
          env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-liars-dice --example bootstrap_query -- \
          "$query_file"
      )"
      assert_contains "$bootstrap_output" "BOOTSTRAP query_file=${query_file}" "${game_slug}_bootstrap_query"
      ;;
  esac

  echo "running ${game_key} miner bootstrap"
  local miner_command=(
    env
    SKIP_WASM_BUILD=1
    "$repo_root/target/debug/myosu-miner"
    --chain "$chain_endpoint"
    --subnet "$subnet"
    --key "$miner_key"
    --port "$miner_port"
    --register
    --serve-axon
  )
  if [[ ${#miner_game_args[@]} -gt 0 ]]; then
    miner_command+=("${miner_game_args[@]}")
  fi
  if [[ -n "$encoder_dir" ]]; then
    miner_command+=(--encoder-dir "$encoder_dir")
  fi
  miner_command+=(
    --query-file "$query_file"
    --response-file "$response_file"
    --data-dir "$miner_data_dir"
  )
  local miner_output
  miner_output="$(
    run_logged \
      "${game_slug}_miner_bootstrap" \
      "${miner_command[@]}"
  )"
  assert_contains "$miner_output" "MINER myosu-miner bootstrap ok" "${game_slug}_miner_bootstrap"
  assert_contains "$miner_output" "REGISTRATION myosu-miner subnet ok" "${game_slug}_miner_bootstrap"
  assert_contains "$miner_output" "AXON myosu-miner publish ok" "${game_slug}_miner_bootstrap"
  assert_contains "$miner_output" "TRAINING myosu-miner batch ok" "${game_slug}_miner_bootstrap"
  assert_contains "$miner_output" "STRATEGY myosu-miner query ok" "${game_slug}_miner_bootstrap"
  assert_contains "$miner_output" "game=${expected_game_name}" "${game_slug}_miner_bootstrap"
  if [[ ! -f "$checkpoint_path" ]]; then
    echo "expected ${game_key} miner checkpoint at ${checkpoint_path}" >&2
    exit 1
  fi
  if [[ ! -f "$response_file" ]]; then
    echo "expected ${game_key} miner response file at ${response_file}" >&2
    exit 1
  fi

  echo "enabling ${game_key} subnet staking"
  local owner_validator_output
  owner_validator_output="$(
    run_logged \
      "${game_slug}_owner_enable_subtoken" \
      env SKIP_WASM_BUILD=1 "$repo_root/target/debug/myosu-validator" \
      --chain "$chain_endpoint" \
      --subnet "$subnet" \
      --key "$owner_key" \
      --enable-subtoken
  )"
  assert_contains "$owner_validator_output" "VALIDATOR myosu-validator bootstrap ok" "${game_slug}_owner_enable_subtoken"
  assert_contains "$owner_validator_output" "SUBTOKEN myosu-validator subnet ok" "${game_slug}_owner_enable_subtoken"

  echo "running first ${game_key} validator scoring pass"
  local validator_a_command=(
    env
    SKIP_WASM_BUILD=1
    "$repo_root/target/debug/myosu-validator"
    --chain "$chain_endpoint"
    --subnet "$subnet"
    --key "$validator_a_key"
    --register
    --stake-amount "$validator_stake"
  )
  if [[ ${#validator_game_args[@]} -gt 0 ]]; then
    validator_a_command+=("${validator_game_args[@]}")
  fi
  if [[ -n "$encoder_dir" ]]; then
    validator_a_command+=(--encoder-dir "$encoder_dir")
  fi
  validator_a_command+=(
    --checkpoint "$checkpoint_path"
    --query-file "$query_file"
    --response-file "$response_file"
  )
  local validator_a_output
  validator_a_output="$(
    run_logged \
      "${game_slug}_validator_a_bootstrap" \
      "${validator_a_command[@]}"
  )"
  assert_contains "$validator_a_output" "VALIDATOR myosu-validator bootstrap ok" "${game_slug}_validator_a_bootstrap"
  assert_contains "$validator_a_output" "REGISTRATION myosu-validator subnet ok" "${game_slug}_validator_a_bootstrap"
  assert_contains "$validator_a_output" "PERMIT myosu-validator ready ok" "${game_slug}_validator_a_bootstrap"
  assert_contains "$validator_a_output" "VALIDATION myosu-validator score ok" "${game_slug}_validator_a_bootstrap"
  assert_contains "$validator_a_output" "game=${expected_game_name}" "${game_slug}_validator_a_bootstrap"

  echo "running second ${game_key} validator scoring pass"
  local validator_b_command=(
    env
    SKIP_WASM_BUILD=1
    "$repo_root/target/debug/myosu-validator"
    --chain "$chain_endpoint"
    --subnet "$subnet"
    --key "$validator_b_key"
    --register
    --stake-amount "$validator_stake"
  )
  if [[ ${#validator_game_args[@]} -gt 0 ]]; then
    validator_b_command+=("${validator_game_args[@]}")
  fi
  if [[ -n "$encoder_dir" ]]; then
    validator_b_command+=(--encoder-dir "$encoder_dir")
  fi
  validator_b_command+=(
    --checkpoint "$checkpoint_path"
    --query-file "$query_file"
    --response-file "$response_file"
  )
  local validator_b_output
  validator_b_output="$(
    run_logged \
      "${game_slug}_validator_b_bootstrap" \
      "${validator_b_command[@]}"
  )"
  assert_contains "$validator_b_output" "VALIDATOR myosu-validator bootstrap ok" "${game_slug}_validator_b_bootstrap"
  assert_contains "$validator_b_output" "REGISTRATION myosu-validator subnet ok" "${game_slug}_validator_b_bootstrap"
  assert_contains "$validator_b_output" "PERMIT myosu-validator ready ok" "${game_slug}_validator_b_bootstrap"
  assert_contains "$validator_b_output" "VALIDATION myosu-validator score ok" "${game_slug}_validator_b_bootstrap"
  assert_contains "$validator_b_output" "game=${expected_game_name}" "${game_slug}_validator_b_bootstrap"

  local validator_a_game
  local validator_b_game
  local validator_a_action_count
  local validator_b_action_count
  local validator_a_exact_match
  local validator_b_exact_match
  local validator_a_l1_distance
  local validator_b_l1_distance
  local validator_a_score
  local validator_b_score
  local validator_a_expected_action
  local validator_b_expected_action
  local validator_a_observed_action
  local validator_b_observed_action

  validator_a_game="$(require_kv "$validator_a_output" "game")"
  validator_b_game="$(require_kv "$validator_b_output" "game")"
  validator_a_action_count="$(require_kv "$validator_a_output" "action_count")"
  validator_b_action_count="$(require_kv "$validator_b_output" "action_count")"
  validator_a_exact_match="$(require_kv "$validator_a_output" "exact_match")"
  validator_b_exact_match="$(require_kv "$validator_b_output" "exact_match")"
  validator_a_l1_distance="$(require_kv "$validator_a_output" "l1_distance")"
  validator_b_l1_distance="$(require_kv "$validator_b_output" "l1_distance")"
  validator_a_score="$(require_kv "$validator_a_output" "score")"
  validator_b_score="$(require_kv "$validator_b_output" "score")"
  validator_a_expected_action="$(require_kv "$validator_a_output" "expected_action")"
  validator_b_expected_action="$(require_kv "$validator_b_output" "expected_action")"
  validator_a_observed_action="$(require_kv "$validator_a_output" "observed_action")"
  validator_b_observed_action="$(require_kv "$validator_b_output" "observed_action")"

  assert_equal "$validator_a_game" "$validator_b_game" "game"
  assert_equal "$validator_a_game" "$expected_game_name" "validator game"
  assert_equal "$validator_a_action_count" "$validator_b_action_count" "action_count"
  assert_equal "$validator_a_exact_match" "$validator_b_exact_match" "exact_match"
  assert_equal "$validator_a_expected_action" "$validator_b_expected_action" "expected_action"
  assert_equal "$validator_a_observed_action" "$validator_b_observed_action" "observed_action"
  assert_within_epsilon \
    "$validator_a_l1_distance" \
    "$validator_b_l1_distance" \
    "$score_epsilon" \
    "${game_key} l1_distance"
  assert_within_epsilon \
    "$validator_a_score" \
    "$validator_b_score" \
    "$score_epsilon" \
    "${game_key} score"

  echo "VALIDATOR_DETERMINISM_CASE myosu e2e ok"
  echo "game=${validator_a_game}"
  echo "game_key=${game_key}"
  echo "subnet=${subnet}"
  echo "validator_a_key=${validator_a_key}"
  echo "validator_b_key=${validator_b_key}"
  echo "validator_a_action_count=${validator_a_action_count}"
  echo "validator_b_action_count=${validator_b_action_count}"
  echo "validator_a_exact_match=${validator_a_exact_match}"
  echo "validator_b_exact_match=${validator_b_exact_match}"
  echo "validator_a_l1_distance=${validator_a_l1_distance}"
  echo "validator_b_l1_distance=${validator_b_l1_distance}"
  echo "validator_a_score=${validator_a_score}"
  echo "validator_b_score=${validator_b_score}"
  echo "score_epsilon=${score_epsilon}"
  echo "expected_action=${validator_a_expected_action}"
  echo "observed_action=${validator_a_observed_action}"
}

trap cleanup EXIT

mkdir -p "$work_parent"
mkdir -p "$driver_examples_dir"
work_root="$(mktemp -d "$work_parent/validator-determinism.XXXXXX")"

cat >"$driver_source" <<'EOF'
use std::env;
use std::error::Error;
use std::time::Duration;

use myosu_chain_client::ChainClient;

const ACTION_TIMEOUT: Duration = Duration::from_secs(180);

fn require_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing required argument: {name}"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let command = require_arg(&mut args, "command")?;
    let endpoint = require_arg(&mut args, "endpoint")?;
    let client = ChainClient::connect(&endpoint).await?;

    match command.as_str() {
        "register_subnet" => {
            let signer_uri = require_arg(&mut args, "signer_uri")?;
            let report = client.register_network(&signer_uri, ACTION_TIMEOUT).await?;
            println!("subnet={}", u16::from(report.subnet));
            println!("inclusion_block={}", report.inclusion_block);
        }
        other => {
            return Err(format!("unknown command `{other}`").into());
        }
    }

    Ok(())
}
EOF

echo "starting local devnet"
bash "$helpers_dir/start_devnet.sh"
started_devnet=1

echo "building validator determinism binaries"
run_logged \
  "build_stage0_binaries" \
  env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-miner -p myosu-validator

MYOSU_E2E_WAIT_TIMEOUT="${MYOSU_E2E_WAIT_TIMEOUT:-120}" \
  bash "$helpers_dir/wait_for_block.sh" 1

for game_key in "${requested_games[@]}"; do
  run_game_case "$game_key"
done

echo "VALIDATOR_DETERMINISM myosu e2e ok"
echo "games=${requested_games_string}"
