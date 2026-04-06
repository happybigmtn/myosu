#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if [[ $# -gt 2 ]]; then
  echo "usage: $0 [bundle-dir] [config-dir]" >&2
  exit 1
fi

bundle_dir="${1:-${MYOSU_OPERATOR_BUNDLE_DIR:-$repo_root/target/operator-network-bundle}}"
config_dir="${2:-${MYOSU_OPERATOR_CONFIG_DIR:-$bundle_dir/config}}"
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
network="${MYOSU_OPERATOR_NETWORK:-devnet}"
subnet="${MYOSU_OPERATOR_SUBNET:-7}"
password_env="${MYOSU_OPERATOR_PASSWORD_ENV:-MYOSU_KEY_PASSWORD}"
bootnode_base_path="${MYOSU_OPERATOR_BOOTNODE_BASE_PATH:-${repo_root}/target/bootnode/devnet}"
bootnode_node_bin="${MYOSU_OPERATOR_BOOTNODE_NODE_BIN:-${cargo_target_dir}/debug/myosu-chain}"
bootnode_chain="${MYOSU_OPERATOR_BOOTNODE_CHAIN:-devnet}"
bootnode_public_host="${MYOSU_OPERATOR_BOOTNODE_PUBLIC_HOST:-127.0.0.1}"
bootnode_p2p_port="${MYOSU_OPERATOR_BOOTNODE_P2P_PORT:-30333}"
bootnode_rpc_port="${MYOSU_OPERATOR_BOOTNODE_RPC_PORT:-9944}"
bootnode_prometheus_port="${MYOSU_OPERATOR_BOOTNODE_PROMETHEUS_PORT:-9615}"
bootnode_service_name="${MYOSU_OPERATOR_BOOTNODE_SERVICE_NAME:-myosu-devnet-bootnode}"
bootnode_info_file="${bootnode_base_path}/config/${bootnode_service_name}.env"
config_file="$config_dir/config.toml"

if [[ -z "${!password_env:-}" ]]; then
  echo "export $password_env before preparing an operator bundle" >&2
  exit 1
fi

mkdir -p "$bundle_dir"
mkdir -p "$config_dir"

read_bootnode_field() {
  local field="$1"
  sed -n "s/^${field}=//p" "$bootnode_info_file"
}

SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet >/dev/null

bash ops/deploy-bootnode.sh \
  --dry-run \
  --base-path "$bootnode_base_path" \
  --chain "$bootnode_chain" \
  --node-bin "$bootnode_node_bin" \
  --public-host "$bootnode_public_host" \
  --p2p-port "$bootnode_p2p_port" \
  --rpc-port "$bootnode_rpc_port" \
  --prometheus-port "$bootnode_prometheus_port" \
  --service-name "$bootnode_service_name" \
  >/dev/null

bootnode_multiaddr="$(read_bootnode_field bootnode_multiaddr)"
bootnode_rpc_endpoint="$(read_bootnode_field rpc_endpoint)"
bootnode_peer_id="$(read_bootnode_field peer_id)"

if [[ -z "$bootnode_multiaddr" || -z "$bootnode_rpc_endpoint" || -z "$bootnode_peer_id" ]]; then
  echo "failed to read bootnode metadata from $bootnode_info_file" >&2
  exit 1
fi

chain_endpoint="${MYOSU_OPERATOR_CHAIN:-$bootnode_rpc_endpoint}"

if [[ ! -f "$config_file" ]]; then
  cargo run -p myosu-keys --quiet -- create \
    --config-dir "$config_dir" \
    --network "$network" \
    >/dev/null
fi

bootstrap_output="$(
  cargo run -p myosu-keys --quiet -- print-bootstrap \
    --config-dir "$config_dir" \
    --password-env "$password_env" \
    --chain "$chain_endpoint" \
    --subnet "$subnet"
)"

active_address="$(printf '%s\n' "$bootstrap_output" | sed -n 's/^Active Address: //p')"
active_network="$(printf '%s\n' "$bootstrap_output" | sed -n 's/^Network: //p')"

cat >"$bundle_dir/start-miner.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd $(printf '%q' "$repo_root")
: "\${$password_env:?export $password_env before running start-miner.sh}"
cargo run -p myosu-miner -- \\
  --chain $(printf '%q' "$chain_endpoint") \\
  --subnet $(printf '%q' "$subnet") \\
  --key-config-dir $(printf '%q' "$config_dir") \\
  --key-password-env $(printf '%q' "$password_env") \\
  "\$@"
EOF

cat >"$bundle_dir/start-validator.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd $(printf '%q' "$repo_root")
: "\${$password_env:?export $password_env before running start-validator.sh}"
cargo run -p myosu-validator -- \\
  --chain $(printf '%q' "$chain_endpoint") \\
  --subnet $(printf '%q' "$subnet") \\
  --key-config-dir $(printf '%q' "$config_dir") \\
  --key-password-env $(printf '%q' "$password_env") \\
  "\$@"
EOF

cat >"$bundle_dir/build-devnet-spec.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd $(printf '%q' "$repo_root")
output_path="\${1:-$(printf '%q' "$bundle_dir/devnet-spec.json")}"
temp_path="\$(mktemp)"
bootnode_info_file=$(printf '%q' "$bootnode_info_file")
bootnode_multiaddr=
trap 'rm -f "\$temp_path"' EXIT
SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- \\
  build-spec --chain devnet >"\$temp_path"
bootnode_multiaddr="\$(sed -n 's/^bootnode_multiaddr=//p' "\$bootnode_info_file")"
if [[ -z "\$bootnode_multiaddr" ]]; then
  echo "missing bootnode metadata in \$bootnode_info_file" >&2
  exit 1
fi
python3 - "\$temp_path" "\$output_path" "\$bootnode_multiaddr" <<'PY'
import json
import sys

source_path, output_path, bootnode = sys.argv[1:4]
with open(source_path, "r", encoding="utf-8") as source_file:
    spec = json.load(source_file)
spec["bootNodes"] = [bootnode]
with open(output_path, "w", encoding="utf-8") as output_file:
    json.dump(spec, output_file, indent=2)
    output_file.write("\n")
PY
printf 'Wrote: %s\n' "\$output_path"
EOF

cat >"$bundle_dir/build-test-finney-spec.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd $(printf '%q' "$repo_root")
output_path="\${1:-$(printf '%q' "$bundle_dir/test-finney-spec.json")}"
SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- \\
  build-spec --chain test_finney >"\$output_path"
printf 'Wrote: %s\n' "\$output_path"
EOF

bash "$bundle_dir/build-devnet-spec.sh" >/dev/null
bash "$bundle_dir/build-test-finney-spec.sh" >/dev/null

cat >"$bundle_dir/bundle-manifest.toml" <<EOF
format_version = 1
bundle_dir = "$bundle_dir"
repo_root = "$repo_root"
config_dir = "$config_dir"
active_address = "$active_address"
network = "$active_network"
chain_endpoint = "$chain_endpoint"
subnet = $subnet
password_env = "$password_env"
bootnode_multiaddr = "$bootnode_multiaddr"
bootnode_rpc_endpoint = "$bootnode_rpc_endpoint"
bootnode_peer_id = "$bootnode_peer_id"

[scripts]
start_miner = "$bundle_dir/start-miner.sh"
start_validator = "$bundle_dir/start-validator.sh"
build_devnet_spec = "$bundle_dir/build-devnet-spec.sh"
build_test_finney_spec = "$bundle_dir/build-test-finney-spec.sh"
verify_bundle = "$bundle_dir/verify-bundle.sh"

[artifacts]
devnet_spec = "$bundle_dir/devnet-spec.json"
test_finney_spec = "$bundle_dir/test-finney-spec.json"
readme = "$bundle_dir/README.md"
EOF

cat >"$bundle_dir/README.md" <<EOF
# Operator Network Bundle

Prepared from:
- repo: $repo_root
- config dir: $config_dir
- chain: $chain_endpoint
- bootnode: $bootnode_multiaddr
- bootnode rpc: $bootnode_rpc_endpoint
- subnet: $subnet
- password env: $password_env

Use:

\`\`\`bash
export $password_env='replace-me'
$bundle_dir/verify-bundle.sh
\`\`\`

Materialized artifacts:

- $bundle_dir/devnet-spec.json
- $bundle_dir/test-finney-spec.json
- $bundle_dir/bundle-manifest.toml

Join the devnet with the bundled bootnode metadata:

\`\`\`bash
cargo run -p myosu-chain -- \\
  --chain $(printf '%q' "$bundle_dir/devnet-spec.json") \\
  --bootnodes $(printf '%q' "$bootnode_multiaddr") \\
  --rpc-port 9955 \\
  --prometheus-port 9616 \\
  --base-path /tmp/myosu-devnet-node
\`\`\`

The bundled \`devnet-spec.json\` already carries the same \`bootNodes\` entry,
so the explicit \`--bootnodes\` flag above is for operator clarity rather than
manual chain-spec editing.

Refresh the named-network specs:

\`\`\`bash
$bundle_dir/build-devnet-spec.sh
$bundle_dir/build-test-finney-spec.sh
\`\`\`

Current bootstrap summary:

\`\`\`text
$bootstrap_output
\`\`\`
EOF

cat >"$bundle_dir/verify-bundle.sh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
bundle_dir="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
temp_dir="\$(mktemp -d)"
trap 'rm -rf "\$temp_dir"' EXIT

test -s "\$bundle_dir/bundle-manifest.toml"
"\$bundle_dir/start-miner.sh" --help >/dev/null
"\$bundle_dir/start-validator.sh" --help >/dev/null
"\$bundle_dir/build-devnet-spec.sh" "\$temp_dir/devnet.json" >/dev/null
"\$bundle_dir/build-test-finney-spec.sh" "\$temp_dir/test-finney.json" >/dev/null

test -s "\$temp_dir/devnet.json"
test -s "\$temp_dir/test-finney.json"
grep -q 'bootnode_multiaddr = ' "\$bundle_dir/bundle-manifest.toml"
grep -Fq -- $(printf '%q' "$bootnode_multiaddr") "\$bundle_dir/README.md"
manifest_bootnode="\$(sed -n 's/^bootnode_multiaddr = \"\\(.*\\)\"$/\\1/p' "\$bundle_dir/bundle-manifest.toml")"
if [[ -z "\$manifest_bootnode" ]]; then
  echo "bundle-manifest.toml is missing bootnode_multiaddr" >&2
  exit 1
fi
python3 - "\$manifest_bootnode" "\$temp_dir/devnet.json" <<'PY'
import json
import sys

bootnode, spec_path = sys.argv[1:3]
with open(spec_path, "r", encoding="utf-8") as spec_file:
    spec = json.load(spec_file)
if not bootnode or "/p2p/" not in bootnode:
    raise SystemExit("invalid bootnode_multiaddr in bundle-manifest.toml")
if bootnode not in spec.get("bootNodes", []):
    raise SystemExit("devnet.json missing bootnode from bundle-manifest.toml")
PY

echo "operator bundle ok"
EOF

chmod +x \
  "$bundle_dir/start-miner.sh" \
  "$bundle_dir/start-validator.sh" \
  "$bundle_dir/build-devnet-spec.sh" \
  "$bundle_dir/build-test-finney-spec.sh" \
  "$bundle_dir/verify-bundle.sh"

printf '%s\n' "$bootstrap_output"
printf 'Bundle: %s\n' "$bundle_dir"
