#!/usr/bin/env bash
set -euo pipefail

base_path="${MYOSU_BASE_PATH:-/var/lib/myosu/chain}"
node_key_file="${MYOSU_NODE_KEY_FILE:-${base_path}/config/node-key}"
chain_spec="${MYOSU_CHAIN_SPEC:-devnet}"
node_name="${MYOSU_NODE_NAME:-Container Devnet Authority}"
rpc_port="${MYOSU_RPC_PORT:-9944}"

mkdir -p "$(dirname "$node_key_file")"
if [[ ! -f "$node_key_file" ]]; then
  umask 077
  /usr/local/bin/myosu-chain key generate-node-key --file "$node_key_file" >/dev/null 2>&1
fi

exec /usr/local/bin/myosu-chain \
  --chain "$chain_spec" \
  --base-path "$base_path" \
  --node-key-file "$node_key_file" \
  --validator \
  --force-authoring \
  --rpc-external \
  --rpc-methods unsafe \
  --rpc-port "$rpc_port" \
  --allow-private-ip \
  --name "$node_name" \
  "$@"
