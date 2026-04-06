#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""
node_bin="$repo_root/target/debug/myosu-chain"
runtime_wasm="$repo_root/target/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm"

rpc_ready_timeout="${MYOSU_E2E_RPC_READY_TIMEOUT:-120}"
initial_finality_timeout="${MYOSU_E2E_INITIAL_FINALITY_TIMEOUT:-120}"
offline_finality_timeout="${MYOSU_E2E_OFFLINE_FINALITY_TIMEOUT:-120}"
restart_catchup_timeout="${MYOSU_E2E_RESTART_CATCHUP_TIMEOUT:-180}"
post_restart_finality_timeout="${MYOSU_E2E_POST_RESTART_FINALITY_TIMEOUT:-120}"
initial_finalized_target="${MYOSU_E2E_INITIAL_FINALIZED_TARGET:-2}"
network_backend="${MYOSU_E2E_NETWORK_BACKEND:-}"

declare -a node_base_dirs node_key_files node_logs node_pids node_p2p_ports node_rpc_ports
declare -a node_prometheus_ports node_rpc_urls node_multiaddrs
declare -a shared_best_blocks shared_finalized_blocks shared_peer_counts
shared_finalized_hash=""
shared_block_hash=""

cleanup() {
  for idx in 4 3 2 1; do
    local pid="${node_pids[$idx]:-}"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
}

select_local_port() {
  python - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
    listener.bind(("127.0.0.1", 0))
    print(listener.getsockname()[1])
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

ensure_node_alive() {
  local pid="$1"
  local label="$2"
  local log_file="$3"

  if kill -0 "$pid" 2>/dev/null; then
    return 0
  fi

  echo "${label} exited unexpectedly" >&2
  tail -n 120 "$log_file" >&2 || true
  exit 1
}

rpc_request() {
  local url="$1"
  local method="$2"
  local params="${3:-[]}"

  curl -fsS \
    -H 'Content-Type: application/json' \
    -d "$(printf '{"jsonrpc":"2.0","id":1,"method":"%s","params":%s}' "$method" "$params")" \
    "$url"
}

parse_block_number() {
  python -c 'import json, sys; payload = json.load(sys.stdin); print(int(payload["result"]["number"], 16))'
}

parse_peer_count() {
  python -c 'import json, sys; payload = json.load(sys.stdin); print(int(payload["result"]["peers"]))'
}

parse_hash() {
  python -c 'import json, sys; payload = json.load(sys.stdin); print(payload["result"])'
}

wait_for_rpc() {
  local idx="$1"
  local url="${node_rpc_urls[$idx]}"
  local pid="${node_pids[$idx]}"
  local label="authority-${idx}"
  local log_file="${node_logs[$idx]}"
  local deadline=$((SECONDS + rpc_ready_timeout))

  while (( SECONDS < deadline )); do
    ensure_node_alive "$pid" "$label" "$log_file"
    if rpc_request "$url" "system_health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "timed out waiting ${rpc_ready_timeout}s for ${label} RPC at ${url}" >&2
  tail -n 120 "$log_file" >&2 || true
  exit 1
}

wait_for_shared_finality() {
  local timeout_secs="$1"
  local target_finalized_block="$2"
  shift 2
  local active_indices=("$@")
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    local all_ready=1
    local expected_hash=""
    shared_best_blocks=()
    shared_finalized_blocks=()
    shared_peer_counts=()
    shared_finalized_hash=""

    for idx in "${active_indices[@]}"; do
      local pid="${node_pids[$idx]}"
      local label="authority-${idx}"
      local log_file="${node_logs[$idx]}"
      local url="${node_rpc_urls[$idx]}"

      ensure_node_alive "$pid" "$label" "$log_file"

      local best_header finalized_head finalized_header health
      best_header="$(rpc_request "$url" "chain_getHeader" 2>/dev/null || true)"
      finalized_head="$(rpc_request "$url" "chain_getFinalizedHead" 2>/dev/null || true)"
      health="$(rpc_request "$url" "system_health" 2>/dev/null || true)"
      if [[ -z "$best_header" || -z "$finalized_head" || -z "$health" ]]; then
        all_ready=0
        break
      fi

      local finalized_hash best_block finalized_block peer_count
      finalized_hash="$(printf '%s' "$finalized_head" | parse_hash 2>/dev/null || true)"
      if [[ -z "$finalized_hash" ]]; then
        all_ready=0
        break
      fi

      finalized_header="$(rpc_request "$url" "chain_getHeader" "[\"$finalized_hash\"]" 2>/dev/null || true)"
      if [[ -z "$finalized_header" ]]; then
        all_ready=0
        break
      fi

      best_block="$(printf '%s' "$best_header" | parse_block_number 2>/dev/null || true)"
      finalized_block="$(printf '%s' "$finalized_header" | parse_block_number 2>/dev/null || true)"
      peer_count="$(printf '%s' "$health" | parse_peer_count 2>/dev/null || true)"
      if [[ -z "$best_block" || -z "$finalized_block" || -z "$peer_count" ]]; then
        all_ready=0
        break
      fi

      shared_best_blocks[$idx]="$best_block"
      shared_finalized_blocks[$idx]="$finalized_block"
      shared_peer_counts[$idx]="$peer_count"

      if (( finalized_block < target_finalized_block )); then
        all_ready=0
      fi

      if [[ -z "$expected_hash" ]]; then
        expected_hash="$finalized_hash"
      elif [[ "$expected_hash" != "$finalized_hash" ]]; then
        all_ready=0
      fi
    done

    if (( all_ready )) && [[ -n "$expected_hash" ]]; then
      shared_finalized_hash="$expected_hash"
      return 0
    fi

    sleep 1
  done

  echo "timed out waiting ${timeout_secs}s for shared finality at block ${target_finalized_block}" >&2
  for idx in "${active_indices[@]}"; do
    echo "--- authority-${idx} log ---" >&2
    tail -n 120 "${node_logs[$idx]}" >&2 || true
  done
  exit 1
}

wait_for_shared_block_hash() {
  local timeout_secs="$1"
  local target_block="$2"
  shift 2
  local active_indices=("$@")
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    local all_ready=1
    local expected_hash=""
    shared_best_blocks=()
    shared_peer_counts=()
    shared_block_hash=""

    for idx in "${active_indices[@]}"; do
      local pid="${node_pids[$idx]}"
      local label="authority-${idx}"
      local log_file="${node_logs[$idx]}"
      local url="${node_rpc_urls[$idx]}"

      ensure_node_alive "$pid" "$label" "$log_file"

      local best_header target_block_hash health
      best_header="$(rpc_request "$url" "chain_getHeader" 2>/dev/null || true)"
      target_block_hash="$(rpc_request "$url" "chain_getBlockHash" "[${target_block}]" 2>/dev/null || true)"
      health="$(rpc_request "$url" "system_health" 2>/dev/null || true)"
      if [[ -z "$best_header" || -z "$target_block_hash" || -z "$health" ]]; then
        all_ready=0
        break
      fi

      local best_block peer_count block_hash
      best_block="$(printf '%s' "$best_header" | parse_block_number 2>/dev/null || true)"
      peer_count="$(printf '%s' "$health" | parse_peer_count 2>/dev/null || true)"
      block_hash="$(printf '%s' "$target_block_hash" | parse_hash 2>/dev/null || true)"
      if [[ -z "$best_block" || -z "$peer_count" || -z "$block_hash" || "$block_hash" == "null" ]]; then
        all_ready=0
        break
      fi

      shared_best_blocks[$idx]="$best_block"
      shared_peer_counts[$idx]="$peer_count"

      if (( best_block < target_block )); then
        all_ready=0
      fi

      if [[ -z "$expected_hash" ]]; then
        expected_hash="$block_hash"
      elif [[ "$expected_hash" != "$block_hash" ]]; then
        all_ready=0
      fi
    done

    if (( all_ready )) && [[ -n "$expected_hash" ]]; then
      shared_block_hash="$expected_hash"
      return 0
    fi

    sleep 1
  done

  echo "timed out waiting ${timeout_secs}s for shared block hash at block ${target_block}" >&2
  for idx in "${active_indices[@]}"; do
    echo "--- authority-${idx} log ---" >&2
    tail -n 120 "${node_logs[$idx]}" >&2 || true
  done
  exit 1
}

start_authority() {
  local idx="$1"
  shift

  local cmd=(
    "$node_bin"
    --chain devnet
    --base-path "${node_base_dirs[$idx]}"
    --node-key-file "${node_key_files[$idx]}"
    --validator
    --force-authoring
    --name "myosu-consensus-resilience-authority-${idx}"
    --rpc-port "${node_rpc_ports[$idx]}"
    --port "${node_p2p_ports[$idx]}"
    --prometheus-port "${node_prometheus_ports[$idx]}"
    --allow-private-ip
  )

  if [[ -n "$network_backend" ]]; then
    cmd+=(--network-backend "$network_backend")
  fi

  while [[ $# -gt 0 ]]; do
    cmd+=(--bootnodes "$1")
    shift
  done

  env MYOSU_NODE_AUTHORITY_SURI="//myosu//devnet//authority-${idx}" "${cmd[@]}" \
    >>"${node_logs[$idx]}" 2>&1 &
  node_pids[$idx]="$!"
}

trap cleanup EXIT

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/consensus-resilience.XXXXXX")"

if [[ ! -f "$runtime_wasm" || ! -x "$node_bin" ]]; then
  if ! rustup target list --installed | grep -qx 'wasm32v1-none'; then
    echo "missing Rust target wasm32v1-none; run: rustup target add wasm32v1-none" >&2
    exit 1
  fi

  if [[ ! -f "$runtime_wasm" ]]; then
    echo "building myosu-chain runtime wasm cache"
    run_logged "build_runtime" env -u SKIP_WASM_BUILD cargo build -p myosu-chain-runtime --quiet
  else
    echo "reusing existing myosu-chain runtime wasm cache"
  fi

  if [[ ! -x "$node_bin" ]]; then
    echo "building myosu-chain node (fast-runtime)"
    run_logged "build_node" env SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet
  else
    echo "reusing existing myosu-chain node binary"
  fi
else
  echo "reusing existing myosu-chain runtime wasm cache and node binary"
fi

for idx in 1 2 3 4; do
  node_base_dirs[$idx]="$work_root/node${idx}"
  node_key_files[$idx]="${node_base_dirs[$idx]}/node-key"
  node_logs[$idx]="$work_root/node${idx}.log"
  mkdir -p "${node_base_dirs[$idx]}"

  "$node_bin" key generate-node-key --file "${node_key_files[$idx]}" >/dev/null
  peer_id="$("$node_bin" key inspect-node-key --file "${node_key_files[$idx]}")"
  node_p2p_ports[$idx]="$(select_local_port)"
  node_rpc_ports[$idx]="$(select_local_port)"
  node_prometheus_ports[$idx]="$(select_local_port)"
  node_rpc_urls[$idx]="http://127.0.0.1:${node_rpc_ports[$idx]}"
  node_multiaddrs[$idx]="/ip4/127.0.0.1/tcp/${node_p2p_ports[$idx]}/p2p/${peer_id}"
done

echo "starting devnet authority 1"
start_authority 1
wait_for_rpc 1

echo "starting devnet authority 2"
start_authority 2 "${node_multiaddrs[1]}"
wait_for_rpc 2

echo "starting devnet authority 3"
start_authority 3 "${node_multiaddrs[1]}" "${node_multiaddrs[2]}"
wait_for_rpc 3

echo "starting devnet authority 4"
start_authority 4 "${node_multiaddrs[1]}" "${node_multiaddrs[2]}" "${node_multiaddrs[3]}"
wait_for_rpc 4

wait_for_shared_finality "$initial_finality_timeout" "$initial_finalized_target" 1 2 3 4
pre_restart_finalized_block="${shared_finalized_blocks[1]}"

printf 'RESILIENCE before_restart hash=%s\n' "$shared_finalized_hash"
for idx in 1 2 3 4; do
  printf 'RESILIENCE node=%s phase=before_restart best=%s finalized=%s peers=%s rpc=%s\n' \
    "$idx" \
    "${shared_best_blocks[$idx]}" \
    "${shared_finalized_blocks[$idx]}" \
    "${shared_peer_counts[$idx]}" \
    "${node_rpc_urls[$idx]}"
done

echo "stopping authority 4"
kill "${node_pids[4]}"
wait "${node_pids[4]}" 2>/dev/null || true
node_pids[4]=""

offline_target=$((pre_restart_finalized_block + 1))
wait_for_shared_finality "$offline_finality_timeout" "$offline_target" 1 2 3
offline_finalized_block="${shared_finalized_blocks[1]}"

printf 'RESILIENCE while_offline hash=%s\n' "$shared_finalized_hash"
for idx in 1 2 3; do
  printf 'RESILIENCE node=%s phase=while_offline best=%s finalized=%s peers=%s rpc=%s\n' \
    "$idx" \
    "${shared_best_blocks[$idx]}" \
    "${shared_finalized_blocks[$idx]}" \
    "${shared_peer_counts[$idx]}" \
    "${node_rpc_urls[$idx]}"
done

echo "restarting authority 4"
start_authority 4 "${node_multiaddrs[1]}" "${node_multiaddrs[2]}" "${node_multiaddrs[3]}"
wait_for_rpc 4

wait_for_shared_block_hash "$restart_catchup_timeout" "$offline_finalized_block" 1 2 3 4
printf 'RESILIENCE catchup block=%s hash=%s\n' "$offline_finalized_block" "$shared_block_hash"
for idx in 1 2 3 4; do
  printf 'RESILIENCE node=%s phase=catchup best=%s peers=%s rpc=%s\n' \
    "$idx" \
    "${shared_best_blocks[$idx]}" \
    "${shared_peer_counts[$idx]}" \
    "${node_rpc_urls[$idx]}"
done

post_restart_target=$((offline_finalized_block + 1))
wait_for_shared_finality "$post_restart_finality_timeout" "$post_restart_target" 1 2 3 4

printf 'RESILIENCE after_restart hash=%s\n' "$shared_finalized_hash"
for idx in 1 2 3 4; do
  printf 'RESILIENCE node=%s phase=after_restart best=%s finalized=%s peers=%s rpc=%s\n' \
    "$idx" \
    "${shared_best_blocks[$idx]}" \
    "${shared_finalized_blocks[$idx]}" \
    "${shared_peer_counts[$idx]}" \
    "${node_rpc_urls[$idx]}"
done

printf 'RESILIENCE consensus_restart ok\n'
