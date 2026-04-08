#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
helpers_dir="$repo_root/tests/e2e/helpers"
runtime_dir="${MYOSU_E2E_DIR:-$repo_root/target/e2e/devnet}"
state_file="$runtime_dir/devnet.env"
work_parent="$repo_root/target/e2e"
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
cargo_bin_dir="$cargo_target_dir/debug"
driver_examples_dir="$repo_root/crates/myosu-chain-client/examples"
driver_example_name="local_loop_driver_$$"
driver_source="$driver_examples_dir/${driver_example_name}.rs"
work_root=""
started_devnet=0
miner_http_pid=""
miner_http_log=""

chain_endpoint="${MYOSU_E2E_CHAIN_ENDPOINT:-ws://127.0.0.1:9955}"
owner_key="${MYOSU_E2E_OWNER_KEY:-//Alice}"
miner_key="${MYOSU_E2E_MINER_KEY:-//Alice}"
validator_key="${MYOSU_E2E_VALIDATOR_KEY:-//Bob}"
validator_stake="${MYOSU_E2E_VALIDATOR_STAKE:-100000000000000}"

cleanup() {
  if [[ -n "$miner_http_pid" ]] && kill -0 "$miner_http_pid" 2>/dev/null; then
    kill "$miner_http_pid" 2>/dev/null || true
    wait "$miner_http_pid" 2>/dev/null || true
  fi

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

select_local_port() {
  python - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
    listener.bind(("127.0.0.1", 0))
    print(listener.getsockname()[1])
PY
}

wait_for_http_health() {
  local endpoint="$1"
  local pid="$2"
  local log_file="$3"
  local timeout_secs="${4:-120}"
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "miner HTTP process exited before becoming healthy" >&2
      tail -n 120 "$log_file" >&2 || true
      exit 1
    fi

    local response
    response="$(curl -fsS "http://${endpoint}/health" 2>/dev/null || true)"
    if [[ "$response" == *'"status":"ok"'* ]]; then
      return 0
    fi

    sleep 1
  done

  echo "timed out waiting for miner HTTP health at ${endpoint}" >&2
  tail -n 120 "$log_file" >&2 || true
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

trap cleanup EXIT

mkdir -p "$work_parent"
mkdir -p "$driver_examples_dir"
work_root="$(mktemp -d "$work_parent/local-loop.XXXXXX")"
miner_http_log="$work_root/miner-http.log"

cat >"$driver_source" <<'EOF'
use std::env;
use std::error::Error;
use std::time::Duration;

use myosu_chain_client::ChainClient;
use subtensor_runtime_common::NetUid;

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
            let hotkey = ChainClient::account_id_from_uri(&signer_uri)?;
            let owner_uid = client
                .get_uid_for_net_and_hotkey(report.subnet, &hotkey)
                .await?
                .ok_or("missing owner uid after register_network")?;

            println!("subnet={}", u16::from(report.subnet));
            println!("owner_uid={owner_uid}");
            println!("inclusion_block={}", report.inclusion_block);
            println!(
                "network_last_registered_at_inclusion={}",
                report.network_last_registered_at_inclusion
            );
            println!(
                "network_last_registered_at_head={}",
                report.network_last_registered_at_head
            );
        }
        "verify_state" => {
            let netuid = NetUid::from(require_arg(&mut args, "subnet")?.parse::<u16>()?);
            let miner_uri = require_arg(&mut args, "miner_uri")?;
            let validator_uri = require_arg(&mut args, "validator_uri")?;
            let expected_port = require_arg(&mut args, "expected_port")?.parse::<u16>()?;

            let miner_hotkey = ChainClient::account_id_from_uri(&miner_uri)?;
            let validator_hotkey = ChainClient::account_id_from_uri(&validator_uri)?;
            let miner_uid = client
                .get_uid_for_net_and_hotkey(netuid, &miner_hotkey)
                .await?
                .ok_or("missing miner uid")?;
            let validator_uid = client
                .wait_for_validator_permit(netuid, &validator_hotkey, ACTION_TIMEOUT)
                .await?;
            let weights = client.get_weights_for_uid(netuid, validator_uid).await?;
            let expected_weights = vec![(miner_uid, u16::MAX)];
            if weights != expected_weights {
                return Err(format!(
                    "validator weights mismatch: expected {:?}, got {:?}",
                    expected_weights, weights
                )
                .into());
            }

            let outcome = client
                .wait_for_epoch_outcome(netuid, miner_uid, validator_uid, ACTION_TIMEOUT)
                .await?;
            let visible_miner = client
                .get_chain_visible_miners(netuid)
                .await?
                .into_iter()
                .find(|candidate| candidate.hotkey == miner_hotkey)
                .ok_or("expected miner to become chain-visible")?;
            let visible_endpoint = visible_miner
                .endpoint_hint()
                .ok_or("expected chain-visible miner endpoint")?;
            if !visible_endpoint.ends_with(&format!(":{expected_port}")) {
                return Err(format!(
                    "expected chain-visible miner endpoint to end with :{expected_port}, got {visible_endpoint}"
                )
                .into());
            }

            println!("miner_uid={miner_uid}");
            println!("validator_uid={validator_uid}");
            println!("weights={weights:?}");
            println!("chain_visible_miner_uid={}", visible_miner.uid);
            println!("chain_visible_endpoint={visible_endpoint}");
            println!("chain_visible_incentive={}", visible_miner.incentive);
            println!("miner_incentive={}", outcome.miner_incentive);
            println!("validator_dividend={}", outcome.validator_dividend);
            println!("miner_emission={}", outcome.miner_emission);
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

echo "building stage-0 operator binaries"
run_logged \
  "build_stage0_binaries" \
  env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-miner -p myosu-validator -p myosu-play

MYOSU_E2E_WAIT_TIMEOUT="${MYOSU_E2E_WAIT_TIMEOUT:-120}" \
  bash "$helpers_dir/wait_for_block.sh" 1

poker_root="$work_root/poker"
encoder_dir="$poker_root/encoder"
query_file="$poker_root/query.bin"
response_file="$poker_root/response.bin"
miner_data_dir="$poker_root/miner-data"
checkpoint_path="$miner_data_dir/checkpoints/latest.bin"
miner_port="${MYOSU_E2E_MINER_PORT:-$(select_local_port)}"

echo "registering poker subnet"
register_output="$(run_chain_driver register_subnet "$owner_key")"
subnet="$(require_kv "$register_output" "subnet")"

echo "writing poker bootstrap artifacts"
bootstrap_output="$(
  run_logged \
    "bootstrap_artifacts" \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
    "$encoder_dir" "$query_file"
)"
assert_contains "$bootstrap_output" "BOOTSTRAP encoder_dir=${encoder_dir}" "bootstrap_artifacts"
assert_contains "$bootstrap_output" "BOOTSTRAP query_file=${query_file}" "bootstrap_artifacts"

echo "running miner bootstrap"
miner_output="$(
  run_logged \
    "miner_bootstrap" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-miner" \
    --chain "$chain_endpoint" \
    --subnet "$subnet" \
    --key "$miner_key" \
    --port "$miner_port" \
    --register \
    --serve-axon \
    --encoder-dir "$encoder_dir" \
    --query-file "$query_file" \
    --response-file "$response_file" \
    --data-dir "$miner_data_dir"
)"
assert_contains "$miner_output" "MINER myosu-miner bootstrap ok" "miner_bootstrap"
assert_contains "$miner_output" "REGISTRATION myosu-miner subnet ok" "miner_bootstrap"
assert_contains "$miner_output" "AXON myosu-miner publish ok" "miner_bootstrap"
assert_contains "$miner_output" "TRAINING myosu-miner batch ok" "miner_bootstrap"
assert_contains "$miner_output" "STRATEGY myosu-miner query ok" "miner_bootstrap"
if [[ ! -f "$checkpoint_path" ]]; then
  echo "expected miner checkpoint at ${checkpoint_path}" >&2
  exit 1
fi
if [[ ! -f "$response_file" ]]; then
  echo "expected miner response file at ${response_file}" >&2
  exit 1
fi

echo "enabling subnet staking"
owner_validator_output="$(
  run_logged \
    "owner_enable_subtoken" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-validator" \
    --chain "$chain_endpoint" \
    --subnet "$subnet" \
    --key "$owner_key" \
    --enable-subtoken
)"
assert_contains "$owner_validator_output" "VALIDATOR myosu-validator bootstrap ok" "owner_enable_subtoken"
assert_contains "$owner_validator_output" "SUBTOKEN myosu-validator subnet ok" "owner_enable_subtoken"

echo "running validator scoring and weight submission"
validator_output="$(
  run_logged \
    "validator_bootstrap" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-validator" \
    --chain "$chain_endpoint" \
    --subnet "$subnet" \
    --key "$validator_key" \
    --register \
    --stake-amount "$validator_stake" \
    --submit-weights \
    --weight-hotkey "$miner_key" \
    --encoder-dir "$encoder_dir" \
    --checkpoint "$checkpoint_path" \
    --query-file "$query_file" \
    --response-file "$response_file"
)"
assert_contains "$validator_output" "VALIDATOR myosu-validator bootstrap ok" "validator_bootstrap"
assert_contains "$validator_output" "REGISTRATION myosu-validator subnet ok" "validator_bootstrap"
assert_contains "$validator_output" "PERMIT myosu-validator ready ok" "validator_bootstrap"
assert_contains "$validator_output" "VALIDATION myosu-validator score ok" "validator_bootstrap"
assert_contains "$validator_output" "exact_match=true" "validator_bootstrap"
assert_contains "$validator_output" "WEIGHTS myosu-validator submission ok" "validator_bootstrap"

echo "verifying chain-visible stage-0 state"
verification_output="$(run_chain_driver verify_state "$subnet" "$miner_key" "$validator_key" "$miner_port")"
miner_uid="$(require_kv "$verification_output" "miner_uid")"
assert_contains "$verification_output" "weights=[(${miner_uid}, 65535)]" "verify_state"

echo "starting live miner HTTP axon"
(
  cd "$repo_root"
  env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-miner" \
    --chain "$chain_endpoint" \
    --subnet "$subnet" \
    --key "$miner_key" \
    --port "$miner_port" \
    --encoder-dir "$encoder_dir" \
    --checkpoint "$checkpoint_path" \
    --serve-http \
    >"$miner_http_log" 2>&1
) &
miner_http_pid="$!"
wait_for_http_health "127.0.0.1:${miner_port}" "$miner_http_pid" "$miner_http_log" "${MYOSU_E2E_HTTP_TIMEOUT:-120}"

echo "running gameplay smoke"
play_output="$(
  run_logged \
    "play_smoke" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-play" \
    --smoke-test \
    --chain "$chain_endpoint" \
    --subnet "$subnet" \
    --require-discovery \
    --require-live-query \
    --require-artifact \
    --smoke-checkpoint "$checkpoint_path" \
    --smoke-encoder-dir "$encoder_dir"
)"
assert_contains "$play_output" "SMOKE myosu-play ok" "play_smoke"
assert_contains "$play_output" "advice_source=artifact" "play_smoke"
assert_contains "$play_output" "miner_discovery=chain_visible" "play_smoke"
assert_contains "$play_output" "live_miner_query=live_http" "play_smoke"
assert_contains "$play_output" "final_state=complete" "play_smoke"
assert_contains "$play_output" "discovered_miner_uid=${miner_uid}" "play_smoke"
assert_contains "$play_output" "live_miner_connect_endpoint=127.0.0.1:${miner_port}" "play_smoke"

echo "LOCAL_LOOP myosu e2e ok"
echo "subnet=${subnet}"
echo "miner_uid=${miner_uid}"
echo "validator_uid=$(require_kv "$verification_output" "validator_uid")"
echo "chain_visible_endpoint=$(require_kv "$verification_output" "chain_visible_endpoint")"
echo "chain_visible_incentive=$(require_kv "$verification_output" "chain_visible_incentive")"
echo "miner_incentive=$(require_kv "$verification_output" "miner_incentive")"
echo "validator_dividend=$(require_kv "$verification_output" "validator_dividend")"
echo "miner_emission=$(require_kv "$verification_output" "miner_emission")"
echo "play_final_state=$(require_kv "$play_output" "final_state")"
