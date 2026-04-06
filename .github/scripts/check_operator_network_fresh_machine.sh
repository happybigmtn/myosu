#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
container_repo_root="${MYOSU_FRESH_MACHINE_CONTAINER_REPO_ROOT:-$repo_root}"
docker_bin="${MYOSU_FRESH_MACHINE_DOCKER_BIN:-docker}"
image="${MYOSU_FRESH_MACHINE_IMAGE:-ubuntu:22.04}"

require_contains() {
  local haystack="$1"
  local needle="$2"
  local context="$3"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "fresh-machine operator proof mismatch: expected ${context} to contain: ${needle}" >&2
    printf '%s\n' "$haystack" >&2
    exit 1
  fi
}

run_capture() {
  local label="$1"
  shift
  local stdout_file="$work_dir/${label}.stdout"
  local stderr_file="$work_dir/${label}.stderr"

  if "$@" >"$stdout_file" 2>"$stderr_file"; then
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

if [[ "${MYOSU_FRESH_MACHINE_INNER:-0}" != "1" ]]; then
  if ! command -v "$docker_bin" >/dev/null 2>&1; then
    echo "fresh-machine operator proof requires docker; missing: $docker_bin" >&2
    exit 1
  fi

  exec "$docker_bin" run --rm --pull=missing \
    -v "${repo_root}:${container_repo_root}:ro" \
    -w "$container_repo_root" \
    -e MYOSU_FRESH_MACHINE_INNER=1 \
    -e MYOSU_FRESH_MACHINE_CONTAINER_REPO_ROOT="$container_repo_root" \
    -e CARGO_TARGET_DIR=/tmp/myosu-target \
    -e MYOSU_KEY_PASSWORD=replace-me \
    -e MYOSU_OPERATOR_PASSWORD_ENV=MYOSU_KEY_PASSWORD \
    -e MYOSU_OPERATOR_CHAIN=ws://127.0.0.1:9944 \
    -e MYOSU_OPERATOR_BOOTNODE_BASE_PATH=/tmp/myosu-bootnode \
    -e MYOSU_OPERATOR_BOOTNODE_RPC_PORT=9944 \
    -e MYOSU_OPERATOR_BOOTNODE_P2P_PORT=30333 \
    -e MYOSU_OPERATOR_BOOTNODE_PROMETHEUS_PORT=9615 \
    "$image" \
    bash "$container_repo_root/.github/scripts/check_operator_network_fresh_machine.sh"
fi

export DEBIAN_FRONTEND=noninteractive
export MYOSU_KEY_PASSWORD="${MYOSU_KEY_PASSWORD:-replace-me}"
export MYOSU_OPERATOR_PASSWORD_ENV="${MYOSU_OPERATOR_PASSWORD_ENV:-MYOSU_KEY_PASSWORD}"
export MYOSU_OPERATOR_CHAIN="${MYOSU_OPERATOR_CHAIN:-ws://127.0.0.1:9944}"
export MYOSU_OPERATOR_BOOTNODE_BASE_PATH="${MYOSU_OPERATOR_BOOTNODE_BASE_PATH:-/tmp/myosu-bootnode}"
export MYOSU_OPERATOR_BOOTNODE_RPC_PORT="${MYOSU_OPERATOR_BOOTNODE_RPC_PORT:-9944}"
export MYOSU_OPERATOR_BOOTNODE_P2P_PORT="${MYOSU_OPERATOR_BOOTNODE_P2P_PORT:-30333}"
export MYOSU_OPERATOR_BOOTNODE_PROMETHEUS_PORT="${MYOSU_OPERATOR_BOOTNODE_PROMETHEUS_PORT:-9615}"

work_dir="$(mktemp -d /tmp/myosu-fresh-machine.XXXXXX)"
bundle_dir="${work_dir}/operator-bundle"
config_dir="${work_dir}/operator-config"
poker_dir="${work_dir}/poker"
node_base_path="${work_dir}/node"
node_log="${work_dir}/node.log"
miner_http_log="${work_dir}/miner-http.log"
checkpoint_path="${poker_dir}/miner-data/checkpoints/latest.bin"
response_file="${poker_dir}/response.bin"
query_file="${poker_dir}/query.bin"
encoder_dir="${poker_dir}/encoder"
operator_funding_amount="120000000000000"
node_pid=""
miner_http_pid=""

cleanup() {
  if [[ -n "$miner_http_pid" ]] && kill -0 "$miner_http_pid" 2>/dev/null; then
    kill "$miner_http_pid" 2>/dev/null || true
    wait "$miner_http_pid" 2>/dev/null || true
  fi
  if [[ -n "$node_pid" ]] && kill -0 "$node_pid" 2>/dev/null; then
    kill "$node_pid" 2>/dev/null || true
    wait "$node_pid" 2>/dev/null || true
  fi
  rm -rf "$work_dir"
}
trap cleanup EXIT

wait_for_chain_block() {
  local target_block="$1"
  local timeout_secs="${2:-180}"
  local deadline=$((SECONDS + timeout_secs))
  local request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'
  local rpc_url="http://127.0.0.1:${MYOSU_OPERATOR_BOOTNODE_RPC_PORT}"

  while (( SECONDS < deadline )); do
    if [[ -n "$node_pid" ]] && ! kill -0 "$node_pid" 2>/dev/null; then
      echo "fresh-machine operator proof: node exited before reaching block ${target_block}" >&2
      tail -n 120 "$node_log" >&2 || true
      exit 1
    fi

    local response
    response="$(curl -fsS -H 'Content-Type: application/json' -d "$request" "$rpc_url" 2>/dev/null || true)"
    local block_hex
    block_hex="$(printf '%s' "$response" | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p')"
    if [[ -n "$block_hex" ]]; then
      local current_block=$((16#$block_hex))
      if (( current_block >= target_block )); then
        return 0
      fi
    fi

    sleep 1
  done

  echo "fresh-machine operator proof timed out waiting for block ${target_block}" >&2
  tail -n 120 "$node_log" >&2 || true
  exit 1
}

wait_for_http_health() {
  local timeout_secs="${1:-180}"
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    if [[ -n "$miner_http_pid" ]] && ! kill -0 "$miner_http_pid" 2>/dev/null; then
      echo "fresh-machine operator proof: live miner exited before /health became ready" >&2
      tail -n 120 "$miner_http_log" >&2 || true
      exit 1
    fi

    local response
    response="$(curl -fsS http://127.0.0.1:8080/health 2>/dev/null || true)"
    if [[ "$response" == *'"status":"ok"'* ]]; then
      return 0
    fi

    sleep 1
  done

  echo "fresh-machine operator proof timed out waiting for live miner /health" >&2
  tail -n 120 "$miner_http_log" >&2 || true
  exit 1
}

echo "installing fresh-machine Ubuntu prerequisites"
apt-get update >/dev/null
apt-get install -y \
  ca-certificates \
  curl \
  build-essential \
  clang \
  cmake \
  pkg-config \
  libssl-dev \
  protobuf-compiler \
  git \
  python3 \
  >/dev/null

echo "installing Rust toolchain and required targets"
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal >/dev/null
fi

# shellcheck disable=SC1090
source "$HOME/.cargo/env"
rustup toolchain install stable --profile minimal >/dev/null
rustup default stable >/dev/null
rustup target add wasm32v1-none wasm32-unknown-unknown >/dev/null

mkdir -p "$bundle_dir" "$config_dir" "$poker_dir" "$node_base_path"

echo "building operator-facing surfaces"
run_capture build_runtime cargo build -p myosu-chain-runtime --quiet
run_capture build_stage0_node env SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet
run_capture build_operator_bins env SKIP_WASM_BUILD=1 cargo build --quiet \
  -p myosu-chain-client \
  -p myosu-keys \
  -p myosu-games-poker \
  -p myosu-miner \
  -p myosu-validator

echo "creating operator key"
run_capture create_operator_key cargo run -p myosu-keys --quiet -- create --config-dir "$config_dir" --network devnet >/dev/null
show_active_output="$(run_capture show_active cargo run -p myosu-keys --quiet -- show-active --config-dir "$config_dir")"
require_contains "$show_active_output" "Active Address:" "myosu-keys show-active output"
active_address="$(printf '%s\n' "$show_active_output" | sed -n 's/^Active Address: //p')"
if [[ -z "$active_address" ]]; then
  echo "fresh-machine operator proof: missing active address in show-active output" >&2
  printf '%s\n' "$show_active_output" >&2
  exit 1
fi

echo "materializing operator bundle"
bundle_output="$(run_capture prepare_bundle bash .github/scripts/prepare_operator_network_bundle.sh "$bundle_dir" "$config_dir")"
require_contains "$bundle_output" "Bundle: $bundle_dir" "bundle preparation output"
verify_output="$(run_capture verify_bundle "$bundle_dir/verify-bundle.sh")"
require_contains "$verify_output" "operator bundle ok" "bundle verification output"

echo "starting local authority-backed devnet for bundle validation"
env MYOSU_NODE_AUTHORITY_SURI="//myosu//devnet//authority-1" \
  "$CARGO_TARGET_DIR/debug/myosu-chain" \
  --chain "$bundle_dir/devnet-spec.json" \
  --base-path "$node_base_path" \
  --node-key-file "${MYOSU_OPERATOR_BOOTNODE_BASE_PATH}/config/node-key" \
  --validator \
  --name "Fresh Machine Authority" \
  --rpc-port "${MYOSU_OPERATOR_BOOTNODE_RPC_PORT}" \
  --port "${MYOSU_OPERATOR_BOOTNODE_P2P_PORT}" \
  --allow-private-ip \
  --prometheus-port "${MYOSU_OPERATOR_BOOTNODE_PROMETHEUS_PORT}" \
  --force-authoring \
  >"$node_log" 2>&1 &
node_pid="$!"
wait_for_chain_block 1 180

echo "funding the generated devnet operator key for local registration and staking"
fund_operator_output="$(run_capture fund_operator cargo run --quiet -p myosu-chain-client --example fund_account -- \
  "$MYOSU_OPERATOR_CHAIN" \
  "//myosu//devnet//subnet-owner" \
  "$active_address" \
  "$operator_funding_amount")"
require_contains "$fund_operator_output" "TRANSFER myosu-chain-client keep-alive ok" "funding output"
require_contains "$fund_operator_output" "dest=$active_address" "funding output"
require_contains "$fund_operator_output" "value=$operator_funding_amount" "funding output"

echo "probing the bundle wrappers against the live chain"
miner_probe_output="$(run_capture miner_probe "$bundle_dir/start-miner.sh")"
require_contains "$miner_probe_output" "MINER myosu-miner bootstrap ok" "miner probe output"
validator_probe_output="$(run_capture validator_probe "$bundle_dir/start-validator.sh")"
require_contains "$validator_probe_output" "VALIDATOR myosu-validator bootstrap ok" "validator probe output"

echo "writing bounded poker artifacts"
run_capture bootstrap_artifacts env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
  "$encoder_dir" \
  "$query_file"
test -s "$query_file"

echo "running one-shot miner bootstrap through the bundle"
miner_bootstrap_output="$(run_capture miner_bootstrap "$bundle_dir/start-miner.sh" \
  --register \
  --serve-axon \
  --port 8080 \
  --encoder-dir "$encoder_dir" \
  --query-file "$query_file" \
  --response-file "$response_file" \
  --data-dir "${poker_dir}/miner-data")"
require_contains "$miner_bootstrap_output" "REGISTRATION myosu-miner subnet ok" "miner bootstrap output"
require_contains "$miner_bootstrap_output" "AXON myosu-miner publish ok" "miner bootstrap output"
require_contains "$miner_bootstrap_output" "TRAINING myosu-miner batch ok" "miner bootstrap output"
require_contains "$miner_bootstrap_output" "STRATEGY myosu-miner query ok" "miner bootstrap output"
test -s "$checkpoint_path"
test -s "$response_file"

echo "starting live HTTP miner through the bundle"
"$bundle_dir/start-miner.sh" \
  --port 8080 \
  --encoder-dir "$encoder_dir" \
  --checkpoint "$checkpoint_path" \
  --serve-http \
  >"$miner_http_log" 2>&1 &
miner_http_pid="$!"
wait_for_http_health 180
require_contains "$(cat "$miner_http_log")" "HTTP myosu-miner axon ok" "live miner output"

echo "running validator bootstrap and scoring through the bundle"
validator_bootstrap_output="$(run_capture validator_bootstrap "$bundle_dir/start-validator.sh" \
  --register \
  --stake-amount 100000000000000 \
  --submit-weights \
  --encoder-dir "$encoder_dir" \
  --checkpoint "$checkpoint_path" \
  --query-file "$query_file" \
  --response-file "$response_file")"
require_contains "$validator_bootstrap_output" "REGISTRATION myosu-validator subnet ok" "validator bootstrap output"
require_contains "$validator_bootstrap_output" "PERMIT myosu-validator ready ok" "validator bootstrap output"
require_contains "$validator_bootstrap_output" "VALIDATION myosu-validator score ok" "validator bootstrap output"
require_contains "$validator_bootstrap_output" "WEIGHTS myosu-validator submission ok" "validator bootstrap output"

echo "fresh-machine operator bundle ok"
