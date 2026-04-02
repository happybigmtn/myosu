#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
runtime_dir="${MYOSU_E2E_DIR:-$repo_root/target/e2e/devnet}"
state_file="$runtime_dir/devnet.env"
pid_file="$runtime_dir/devnet.pid"
log_file="$runtime_dir/devnet.log"
tmp_root="$runtime_dir/tmp"
rpc_host="${MYOSU_E2E_RPC_HOST:-127.0.0.1}"
rpc_port="${MYOSU_E2E_RPC_PORT:-9955}"
p2p_port="${MYOSU_E2E_P2P_PORT:-30444}"
prometheus_port="${MYOSU_E2E_PROM_PORT:-9616}"
rpc_url="http://${rpc_host}:${rpc_port}"

mkdir -p "$runtime_dir"

if [[ -f "$pid_file" ]]; then
  existing_pid="$(<"$pid_file")"
  if kill -0 "$existing_pid" 2>/dev/null; then
    echo "myosu e2e devnet already running with pid $existing_pid" >&2
    exit 1
  fi
  rm -f "$pid_file"
fi

if ! rustup target list --installed | grep -qx 'wasm32v1-none'; then
  echo "missing Rust target wasm32v1-none; run: rustup target add wasm32v1-none" >&2
  exit 1
fi

rm -rf "$tmp_root"
mkdir -p "$tmp_root"
: >"$log_file"

echo "building myosu-chain runtime wasm cache"
cargo build -p myosu-chain-runtime --quiet
echo "building myosu-chain node (fast-runtime)"
SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet

node_pid="$(
  REPO_ROOT="$repo_root" \
  TMP_ROOT="$tmp_root" \
  LOG_FILE="$log_file" \
  RPC_PORT="$rpc_port" \
  P2P_PORT="$p2p_port" \
  PROMETHEUS_PORT="$prometheus_port" \
  python - <<'PY'
import os
import subprocess

repo_root = os.environ["REPO_ROOT"]
tmp_root = os.environ["TMP_ROOT"]
log_file = os.environ["LOG_FILE"]
rpc_port = os.environ["RPC_PORT"]
p2p_port = os.environ["P2P_PORT"]
prometheus_port = os.environ["PROMETHEUS_PORT"]

env = os.environ.copy()
env["TMPDIR"] = tmp_root
with open(log_file, "ab", buffering=0) as log_handle:
    process = subprocess.Popen(
        [
            os.path.join(repo_root, "target/debug/myosu-chain"),
            "--dev",
            "--force-authoring",
            "--rpc-port",
            rpc_port,
            "--port",
            p2p_port,
            "--prometheus-port",
            prometheus_port,
        ],
        cwd=repo_root,
        env=env,
        stdin=subprocess.DEVNULL,
        stdout=log_handle,
        stderr=subprocess.STDOUT,
        start_new_session=True,
    )
print(process.pid)
PY
)"

printf '%s\n' "$node_pid" >"$pid_file"
{
  printf 'REPO_ROOT=%q\n' "$repo_root"
  printf 'RUNTIME_DIR=%q\n' "$runtime_dir"
  printf 'STATE_FILE=%q\n' "$state_file"
  printf 'PID_FILE=%q\n' "$pid_file"
  printf 'LOG_FILE=%q\n' "$log_file"
  printf 'TMP_ROOT=%q\n' "$tmp_root"
  printf 'RPC_HOST=%q\n' "$rpc_host"
  printf 'RPC_PORT=%q\n' "$rpc_port"
  printf 'P2P_PORT=%q\n' "$p2p_port"
  printf 'PROMETHEUS_PORT=%q\n' "$prometheus_port"
  printf 'RPC_URL=%q\n' "$rpc_url"
} >"$state_file"

sleep 1
if ! kill -0 "$node_pid" 2>/dev/null; then
  echo "myosu e2e devnet exited during startup" >&2
  tail -n 80 "$log_file" >&2 || true
  rm -f "$pid_file"
  exit 1
fi

echo "myosu e2e devnet started"
echo "pid=$node_pid"
echo "rpc_url=$rpc_url"
echo "log_file=$log_file"
