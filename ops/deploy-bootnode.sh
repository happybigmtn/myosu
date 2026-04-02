#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
usage: ops/deploy-bootnode.sh [options]

Prepare and optionally launch a persistent Myosu devnet bootnode.

By default this script writes all durable assets under
`target/bootnode/devnet/`, generates a stable libp2p node key if one does not
already exist, renders a launcher script plus a systemd unit file, and starts
the node detached.

`--dry-run` still prepares the durable bootnode assets so the advertised
peer ID and multiaddr are truthful, but it skips launching the node process.

Options:
  --dry-run                 Prepare assets only; do not launch the node
  --base-path PATH          Persistent bootnode root (default: target/bootnode/devnet)
  --chain CHAIN             Chain name or spec path (default: devnet)
  --node-bin PATH           myosu-chain binary path (default: target/debug/myosu-chain)
  --public-host HOST        Public IPv4 or DNS name to advertise (default: 127.0.0.1)
  --p2p-port PORT           P2P TCP port (default: 30333)
  --rpc-port PORT           RPC port (default: 9944)
  --prometheus-port PORT    Prometheus port (default: 9615)
  --rpc-methods MODE        RPC exposure mode: auto|safe|unsafe (default: safe)
  --service-name NAME       Service/unit name (default: myosu-devnet-bootnode)
  --node-name NAME          Human-readable node name (default: Myosu Devnet Bootnode)
  -h, --help                Show this help
EOF
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
repo_root="$(cd "${script_dir}/.." && pwd -P)"

dry_run=0
base_path="${MYOSU_BOOTNODE_BASE_PATH:-${repo_root}/target/bootnode/devnet}"
chain_spec="${MYOSU_BOOTNODE_CHAIN:-devnet}"
node_bin="${MYOSU_BOOTNODE_NODE_BIN:-${repo_root}/target/debug/myosu-chain}"
public_host="${MYOSU_BOOTNODE_PUBLIC_HOST:-127.0.0.1}"
p2p_port="${MYOSU_BOOTNODE_P2P_PORT:-30333}"
rpc_port="${MYOSU_BOOTNODE_RPC_PORT:-9944}"
prometheus_port="${MYOSU_BOOTNODE_PROMETHEUS_PORT:-9615}"
rpc_methods="${MYOSU_BOOTNODE_RPC_METHODS:-safe}"
service_name="${MYOSU_BOOTNODE_SERVICE_NAME:-myosu-devnet-bootnode}"
node_name="${MYOSU_BOOTNODE_NODE_NAME:-Myosu Devnet Bootnode}"
rust_log="${MYOSU_BOOTNODE_RUST_LOG:-info}"
start_timeout="${MYOSU_BOOTNODE_START_TIMEOUT:-120}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      dry_run=1
      shift
      ;;
    --base-path)
      base_path="$2"
      shift 2
      ;;
    --chain)
      chain_spec="$2"
      shift 2
      ;;
    --node-bin)
      node_bin="$2"
      shift 2
      ;;
    --public-host)
      public_host="$2"
      shift 2
      ;;
    --p2p-port)
      p2p_port="$2"
      shift 2
      ;;
    --rpc-port)
      rpc_port="$2"
      shift 2
      ;;
    --prometheus-port)
      prometheus_port="$2"
      shift 2
      ;;
    --rpc-methods)
      rpc_methods="$2"
      shift 2
      ;;
    --service-name)
      service_name="$2"
      shift 2
      ;;
    --node-name)
      node_name="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

abs_path() {
  local path="$1"
  local dir
  if [[ "$path" != /* ]]; then
    path="${PWD}/${path}"
  fi
  dir="$(cd "$(dirname "$path")" && pwd -P)"
  printf '%s/%s\n' "$dir" "$(basename "$path")"
}

resolve_executable() {
  local candidate="$1"
  if [[ "$candidate" == */* ]]; then
    candidate="$(abs_path "$candidate")"
    if [[ ! -x "$candidate" ]]; then
      echo "node binary is not executable: $candidate" >&2
      echo "build it first with: cargo build -p myosu-chain --features fast-runtime" >&2
      exit 1
    fi
    printf '%s\n' "$candidate"
    return
  fi

  if ! candidate="$(command -v "$candidate")"; then
    echo "unable to resolve node binary: $1" >&2
    exit 1
  fi
  printf '%s\n' "$candidate"
}

render_shell_command() {
  local out=()
  local arg
  for arg in "$@"; do
    out+=("$(printf '%q' "$arg")")
  done
  printf '%s\n' "${out[*]}"
}

public_transport_addr() {
  local host="$1"
  local port="$2"
  if [[ "$host" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    printf '/ip4/%s/tcp/%s\n' "$host" "$port"
  elif [[ "$host" == *:* ]]; then
    printf '/ip6/%s/tcp/%s\n' "$host" "$port"
  else
    printf '/dns/%s/tcp/%s\n' "$host" "$port"
  fi
}

mkdir -p "$base_path"
base_path="$(abs_path "$base_path")"
config_dir="${base_path}/config"
data_dir="${base_path}/data"
log_dir="${base_path}/logs"
run_dir="${base_path}/run"
systemd_dir="${base_path}/systemd"
node_key_file="${config_dir}/node-key"
launcher_file="${run_dir}/${service_name}-start.sh"
pid_file="${run_dir}/${service_name}.pid"
log_file="${log_dir}/${service_name}.log"
unit_file="${systemd_dir}/${service_name}.service"
info_file="${config_dir}/${service_name}.env"

node_bin="$(resolve_executable "$node_bin")"
if [[ "$chain_spec" == */* ]]; then
  chain_spec="$(abs_path "$chain_spec")"
fi

mkdir -p "$config_dir" "$data_dir" "$log_dir" "$run_dir" "$systemd_dir"

if [[ ! -f "$node_key_file" ]]; then
  umask 077
  "$node_bin" key generate-node-key --file "$node_key_file" >/dev/null 2>/dev/null
fi

peer_id="$("$node_bin" key inspect-node-key --file "$node_key_file")"
public_addr="$(public_transport_addr "$public_host" "$p2p_port")"
bootnode_multiaddr="${public_addr}/p2p/${peer_id}"
rpc_endpoint="ws://${public_host}:${rpc_port}"
prometheus_endpoint="http://${public_host}:${prometheus_port}/metrics"

node_cmd=(
  "$node_bin"
  "--chain" "$chain_spec"
  "--base-path" "$data_dir"
  "--node-key-file" "$node_key_file"
  "--name" "$node_name"
  "--listen-addr" "/ip4/0.0.0.0/tcp/${p2p_port}"
  "--public-addr" "$public_addr"
  "--allow-private-ip"
  "--rpc-external"
  "--rpc-methods" "$rpc_methods"
  "--rpc-port" "$rpc_port"
  "--rpc-cors" "all"
  "--prometheus-external"
  "--prometheus-port" "$prometheus_port"
  "--execution" "native"
)

cat >"$launcher_file" <<EOF
#!/usr/bin/env bash
set -euo pipefail
export RUST_LOG="\${RUST_LOG:-$(printf '%q' "$rust_log")}"
exec $(render_shell_command "${node_cmd[@]}")
EOF
chmod 755 "$launcher_file"

cat >"$unit_file" <<EOF
[Unit]
Description=Myosu devnet bootnode
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=${repo_root}
ExecStart=${launcher_file}
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=${rust_log}

[Install]
WantedBy=multi-user.target
EOF

cat >"$info_file" <<EOF
service_name=${service_name}
node_name=${node_name}
base_path=${base_path}
data_dir=${data_dir}
node_binary=${node_bin}
chain=${chain_spec}
node_key_file=${node_key_file}
peer_id=${peer_id}
bootnode_multiaddr=${bootnode_multiaddr}
rpc_endpoint=${rpc_endpoint}
prometheus_endpoint=${prometheus_endpoint}
launcher=${launcher_file}
systemd_unit=${unit_file}
log_file=${log_file}
pid_file=${pid_file}
EOF
chmod 600 "$info_file"

print_summary() {
  cat <<EOF
Bootnode prepared:
  service: ${service_name}
  node binary: ${node_bin}
  chain: ${chain_spec}
  base path: ${base_path}
  data dir: ${data_dir}
  node key: ${node_key_file}
  peer id: ${peer_id}
  multiaddr: ${bootnode_multiaddr}
  rpc endpoint: ${rpc_endpoint}
  prometheus: ${prometheus_endpoint}
  launcher: ${launcher_file}
  systemd unit: ${unit_file}
  metadata: ${info_file}
EOF
}

if [[ -f "$pid_file" ]]; then
  existing_pid="$(<"$pid_file")"
  if kill -0 "$existing_pid" 2>/dev/null; then
    print_summary
    echo "Node already running with pid ${existing_pid}" >&2
    echo "Install the generated systemd unit with:" >&2
    echo "  sudo cp ${unit_file} /etc/systemd/system/${service_name}.service" >&2
    echo "  sudo systemctl daemon-reload && sudo systemctl enable --now ${service_name}" >&2
    exit 0
  fi
  rm -f "$pid_file"
fi

print_summary
echo "Install the generated systemd unit with:"
echo "  sudo cp ${unit_file} /etc/systemd/system/${service_name}.service"
echo "  sudo systemctl daemon-reload && sudo systemctl enable --now ${service_name}"

if (( dry_run )); then
  echo "Dry run requested; durable assets were prepared but the node was not started."
  exit 0
fi

nohup "$launcher_file" >"$log_file" 2>&1 &
bootnode_pid=$!
echo "$bootnode_pid" >"$pid_file"

rpc_request='{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}'
deadline=$((SECONDS + start_timeout))
bootnode_ready=0
while (( SECONDS < deadline )); do
  if ! kill -0 "$bootnode_pid" 2>/dev/null; then
    echo "bootnode failed to stay up; recent log output:" >&2
    if [[ -f "$log_file" ]]; then
      tail -n 40 "$log_file" >&2
    fi
    rm -f "$pid_file"
    exit 1
  fi

  if curl -fsS -H 'Content-Type: application/json' -d "$rpc_request" "http://127.0.0.1:${rpc_port}" >/dev/null 2>&1; then
    bootnode_ready=1
    break
  fi

  sleep 1
done

if (( bootnode_ready == 0 )); then
  echo "bootnode did not expose RPC on 127.0.0.1:${rpc_port} within ${start_timeout}s" >&2
  if [[ -f "$log_file" ]]; then
    tail -n 40 "$log_file" >&2
  fi
  rm -f "$pid_file"
  exit 1
fi

echo "Started bootnode pid ${bootnode_pid}"
echo "Log file: ${log_file}"
