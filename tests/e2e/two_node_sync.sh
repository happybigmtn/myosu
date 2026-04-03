#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""
node_bin="$repo_root/target/debug/myosu-chain"
runtime_wasm="$repo_root/target/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm"

rpc_ready_timeout="${MYOSU_E2E_RPC_READY_TIMEOUT:-120}"
bootnode_block_timeout="${MYOSU_E2E_BOOTNODE_BLOCK_TIMEOUT:-120}"
sync_timeout="${MYOSU_E2E_SYNC_TIMEOUT:-60}"
sync_target_block="${MYOSU_E2E_SYNC_TARGET_BLOCK:-1}"

bootnode_base=""
peer_base=""
bootnode_key_file=""
bootnode_log=""
peer_log=""
bootnode_rpc_url=""
peer_rpc_url=""
bootnode_multiaddr=""
bootnode_pid=""
peer_pid=""
wait_block_result=""
sync_boot_peers=""
sync_peer_peers=""
sync_boot_block=""
sync_peer_block=""

cleanup() {
  for pid in "$peer_pid" "$bootnode_pid"; do
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

wait_for_rpc() {
  local url="$1"
  local pid="$2"
  local label="$3"
  local log_file="$4"
  local timeout_secs="$5"
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    ensure_node_alive "$pid" "$label" "$log_file"
    if rpc_request "$url" "system_health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "timed out waiting ${timeout_secs}s for ${label} RPC at ${url}" >&2
  tail -n 120 "$log_file" >&2 || true
  exit 1
}

wait_for_best_block() {
  local url="$1"
  local pid="$2"
  local label="$3"
  local log_file="$4"
  local target_block="$5"
  local timeout_secs="$6"
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    ensure_node_alive "$pid" "$label" "$log_file"
    local response
    response="$(rpc_request "$url" "chain_getHeader" 2>/dev/null || true)"
    if [[ -n "$response" ]]; then
      local block
      block="$(printf '%s' "$response" | parse_block_number 2>/dev/null || true)"
      if [[ -n "$block" ]] && (( block >= target_block )); then
        wait_block_result="$block"
        return 0
      fi
    fi
    sleep 1
  done

  echo "timed out waiting ${timeout_secs}s for ${label} to reach block ${target_block}" >&2
  tail -n 120 "$log_file" >&2 || true
  exit 1
}

wait_for_sync() {
  local deadline=$((SECONDS + sync_timeout))

  while (( SECONDS < deadline )); do
    ensure_node_alive "$bootnode_pid" "bootnode" "$bootnode_log"
    ensure_node_alive "$peer_pid" "peer" "$peer_log"

    local boot_health peer_health boot_header peer_header
    boot_health="$(rpc_request "$bootnode_rpc_url" "system_health" 2>/dev/null || true)"
    peer_health="$(rpc_request "$peer_rpc_url" "system_health" 2>/dev/null || true)"
    boot_header="$(rpc_request "$bootnode_rpc_url" "chain_getHeader" 2>/dev/null || true)"
    peer_header="$(rpc_request "$peer_rpc_url" "chain_getHeader" 2>/dev/null || true)"

    if [[ -z "$boot_health" || -z "$peer_health" || -z "$boot_header" || -z "$peer_header" ]]; then
      sleep 1
      continue
    fi

    local boot_peers peer_peers boot_block peer_block
    boot_peers="$(printf '%s' "$boot_health" | parse_peer_count 2>/dev/null || true)"
    peer_peers="$(printf '%s' "$peer_health" | parse_peer_count 2>/dev/null || true)"
    boot_block="$(printf '%s' "$boot_header" | parse_block_number 2>/dev/null || true)"
    peer_block="$(printf '%s' "$peer_header" | parse_block_number 2>/dev/null || true)"

    if [[ -z "$boot_peers" || -z "$peer_peers" || -z "$boot_block" || -z "$peer_block" ]]; then
      sleep 1
      continue
    fi

    if (( boot_peers >= 1 && peer_peers >= 1 && boot_block >= sync_target_block && peer_block == boot_block )); then
      sync_boot_peers="$boot_peers"
      sync_peer_peers="$peer_peers"
      sync_boot_block="$boot_block"
      sync_peer_block="$peer_block"
      return 0
    fi

    sleep 1
  done

  echo "timed out waiting ${sync_timeout}s for two-node sync" >&2
  echo "--- bootnode log ---" >&2
  tail -n 120 "$bootnode_log" >&2 || true
  echo "--- peer log ---" >&2
  tail -n 120 "$peer_log" >&2 || true
  exit 1
}

trap cleanup EXIT

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/two-node-sync.XXXXXX")"
bootnode_base="$work_root/bootnode"
peer_base="$work_root/peer"
bootnode_key_file="$bootnode_base/node-key"
bootnode_log="$work_root/bootnode.log"
peer_log="$work_root/peer.log"
mkdir -p "$bootnode_base" "$peer_base"

bootnode_p2p_port="${MYOSU_E2E_BOOTNODE_P2P_PORT:-$(select_local_port)}"
bootnode_rpc_port="${MYOSU_E2E_BOOTNODE_RPC_PORT:-$(select_local_port)}"
peer_p2p_port="${MYOSU_E2E_PEER_P2P_PORT:-$(select_local_port)}"
peer_rpc_port="${MYOSU_E2E_PEER_RPC_PORT:-$(select_local_port)}"
bootnode_rpc_url="http://127.0.0.1:${bootnode_rpc_port}"
peer_rpc_url="http://127.0.0.1:${peer_rpc_port}"

if [[ ! -f "$runtime_wasm" || ! -x "$node_bin" ]]; then
  if ! rustup target list --installed | grep -qx 'wasm32v1-none'; then
    echo "missing Rust target wasm32v1-none; run: rustup target add wasm32v1-none" >&2
    exit 1
  fi

  if [[ ! -f "$runtime_wasm" ]]; then
    echo "building myosu-chain runtime wasm cache"
    run_logged "build_runtime" cargo build -p myosu-chain-runtime --quiet
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

echo "creating bootnode identity"
"$node_bin" key generate-node-key --file "$bootnode_key_file" >/dev/null
bootnode_peer_id="$("$node_bin" key inspect-node-key --file "$bootnode_key_file")"
bootnode_multiaddr="/ip4/127.0.0.1/tcp/${bootnode_p2p_port}/p2p/${bootnode_peer_id}"

echo "starting devnet authority bootnode"
env MYOSU_NODE_AUTHORITY_SURI=//myosu//devnet//authority-1 "$node_bin" \
  --chain devnet \
  --base-path "$bootnode_base" \
  --node-key-file "$bootnode_key_file" \
  --validator \
  --force-authoring \
  --name myosu-two-node-bootnode \
  --rpc-port "$bootnode_rpc_port" \
  --port "$bootnode_p2p_port" \
  --allow-private-ip \
  >"$bootnode_log" 2>&1 &
bootnode_pid="$!"

wait_for_rpc "$bootnode_rpc_url" "$bootnode_pid" "bootnode" "$bootnode_log" "$rpc_ready_timeout"
wait_for_best_block \
  "$bootnode_rpc_url" \
  "$bootnode_pid" \
  "bootnode" \
  "$bootnode_log" \
  "$sync_target_block" \
  "$bootnode_block_timeout"
bootnode_best_before_peer="$wait_block_result"

echo "starting devnet sync peer"
"$node_bin" \
  --chain devnet \
  --base-path "$peer_base" \
  --name myosu-two-node-peer \
  --rpc-port "$peer_rpc_port" \
  --port "$peer_p2p_port" \
  --allow-private-ip \
  --bootnodes "$bootnode_multiaddr" \
  >"$peer_log" 2>&1 &
peer_pid="$!"

wait_for_rpc "$peer_rpc_url" "$peer_pid" "peer" "$peer_log" "$rpc_ready_timeout"
wait_for_sync

printf 'SYNC bootnode_multiaddr=%s\n' "$bootnode_multiaddr"
printf 'SYNC bootnode_rpc=%s\n' "$bootnode_rpc_url"
printf 'SYNC peer_rpc=%s\n' "$peer_rpc_url"
printf 'SYNC bootnode_best_before_peer=%s\n' "$bootnode_best_before_peer"
printf 'boot_peers=%s\n' "$sync_boot_peers"
printf 'peer_peers=%s\n' "$sync_peer_peers"
printf 'boot_block=%s\n' "$sync_boot_block"
printf 'peer_block=%s\n' "$sync_peer_block"
printf 'SYNC two_node_devnet ok\n'
