#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 <bundle-dir> [config-dir]" >&2
  exit 1
fi

bundle_dir="$1"
config_dir="${2:-$bundle_dir/config}"
network="${MYOSU_OPERATOR_NETWORK:-devnet}"
chain_endpoint="${MYOSU_OPERATOR_CHAIN:-ws://127.0.0.1:9944}"
subnet="${MYOSU_OPERATOR_SUBNET:-7}"
password_env="${MYOSU_OPERATOR_PASSWORD_ENV:-MYOSU_KEY_PASSWORD}"
config_file="$config_dir/config.toml"

if [[ -z "${!password_env:-}" ]]; then
  echo "export $password_env before preparing an operator bundle" >&2
  exit 1
fi

mkdir -p "$bundle_dir"
mkdir -p "$config_dir"

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
SKIP_WASM_BUILD=1 cargo run -p myosu-chain --features fast-runtime -- \\
  build-spec --chain devnet >"\$output_path"
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

echo "operator bundle ok"
EOF

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

chmod +x \
  "$bundle_dir/start-miner.sh" \
  "$bundle_dir/start-validator.sh" \
  "$bundle_dir/build-devnet-spec.sh" \
  "$bundle_dir/build-test-finney-spec.sh" \
  "$bundle_dir/verify-bundle.sh"

"$bundle_dir/build-devnet-spec.sh" >/dev/null
"$bundle_dir/build-test-finney-spec.sh" >/dev/null

printf '%s\n' "$bootstrap_output"
printf 'Bundle: %s\n' "$bundle_dir"
