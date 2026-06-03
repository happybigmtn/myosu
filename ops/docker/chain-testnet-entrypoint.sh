#!/usr/bin/env bash
# Testnet chain entrypoint.
#
# Mirrors `chain-devnet-entrypoint.sh` but boots the node from the
# `test_finney` chain spec (see
# `crates/myosu-chain/node/src/chain_spec/testnet.rs`). The testnet spec
# uses the shared `genesis_with_game_solver` builder, so subnet 7, the
# testnet subnet owner + hotkey, the testnet authority set, and the
# testnet operator accounts are all pre-endowed — the persistent entrypoint
# just needs to keep the node alive with a CORS-enabled WS/HTTP RPC
# endpoint that survives the proof run.
#
# Subnet-owner registration (the `enable-subtoken` extrinsic) is a one-shot
# transaction and is handled by a separate `subnet-owner-init` compose
# service that shares this image's `wait-for-rpc.sh` helper, so the chain
# itself never blocks on it. Once the chain is up and producing blocks, the
# external validator or operator loop can register, stake, and submit
# weights against this RPC endpoint.
#
# All defaults are environment-overridable so the same entrypoint works in
# docker compose, in the bash proof harness, and in operator-supplied
# `docker run` invocations.
set -euo pipefail

base_path="${MYOSU_BASE_PATH:-/var/lib/myosu/chain}"
node_key_file="${MYOSU_NODE_KEY_FILE:-${base_path}/config/node-key}"
chain_spec="${MYOSU_CHAIN_SPEC:-test_finney}"
node_name="${MYOSU_NODE_NAME:-Container Testnet Authority}"
rpc_port="${MYOSU_RPC_PORT:-9944}"
rpc_cors="${MYOSU_RPC_CORS:-all}"
rpc_methods="${MYOSU_RPC_METHODS:-unsafe}"
prometheus_port="${MYOSU_PROMETHEUS_PORT:-9615}"
p2p_port="${MYOSU_P2P_PORT:-30333}"

# The named testnet spec is intentionally `ChainType::Local` until a
# persistent live testnet entrypoint exists (see
# `IMPLEMENTATION_PLAN.md`'s P0 #4 row). This entrypoint IS that persistent
# entrypoint, but the chain spec is still sealed as a local network so a
# deployment mistake can't accidentally promote it to a live network
# without an explicit spec change. Operators who want a live testnet should
# produce a fresh `build-spec` artifact and feed it via `--chain <path>`.
if [[ "$chain_spec" != "test_finney" && ! -f "$chain_spec" ]]; then
  echo "testnet entrypoint expects --chain test_finney or an explicit spec file path" >&2
  echo "got: ${chain_spec}" >&2
  exit 1
fi

mkdir -p "$(dirname "$node_key_file")"
if [[ ! -f "$node_key_file" ]]; then
  umask 077
  /usr/local/bin/myosu-chain key generate-node-key --file "$node_key_file" >/dev/null 2>&1
fi

# `--rpc-external` (not `--rpc-internal`) plus `--rpc-cors all` plus
# `--rpc-methods unsafe` is the canonical CORS-enabled WS/HTTP surface the
# P0 milestone calls for. The bash proof harness and any external
# `myosu-chain-client` consumer connects through this surface.
#
# `--validator --force-authoring` is what keeps the chain producing blocks
# while the testnet entrypoint is up. The validator `--enable-subtoken`
# step is intentionally NOT done here — it is a one-shot extrinsic, and
# running it from inside the chain's entrypoint loop would either block
# forever (waiting for RPC) or risk a self-referential race. The
# `subnet-owner-init` compose service does it once after this service's
# healthcheck turns green.
exec /usr/local/bin/myosu-chain \
  --chain "$chain_spec" \
  --base-path "$base_path" \
  --node-key-file "$node_key_file" \
  --validator \
  --force-authoring \
  --rpc-external \
  --rpc-methods "$rpc_methods" \
  --rpc-port "$rpc_port" \
  --rpc-cors "$rpc_cors" \
  --prometheus-port "$prometheus_port" \
  --port "$p2p_port" \
  --allow-private-ip \
  --name "$node_name" \
  "$@"
