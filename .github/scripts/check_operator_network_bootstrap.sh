#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

temp_dir="$(mktemp -d)"
trap 'rm -rf "$temp_dir"' EXIT

require_contains() {
  local haystack="$1"
  local needle="$2"
  local context="$3"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "operator bootstrap mismatch: expected $context to contain: $needle"
    exit 1
  fi
}

export MYOSU_KEY_PASSWORD='replace-me'

cargo run -p myosu-keys --quiet -- create --config-dir "$temp_dir" --network devnet >/dev/null
bootstrap_output="$(
  cargo run -p myosu-keys --quiet -- print-bootstrap --config-dir "$temp_dir" --subnet 7
)"

miner_command="$(printf '%s\n' "$bootstrap_output" | sed -n 's/^Miner Command: //p')"
validator_command="$(printf '%s\n' "$bootstrap_output" | sed -n 's/^Validator Command: //p')"

if [[ -z "$miner_command" || -z "$validator_command" ]]; then
  echo "operator bootstrap mismatch: missing printed miner or validator command"
  printf '%s\n' "$bootstrap_output"
  exit 1
fi

require_contains "$bootstrap_output" "Config: $temp_dir/config.toml" "bootstrap output"
require_contains "$miner_command" "--key-config-dir $temp_dir" "miner command"
require_contains "$miner_command" "--key-password-env MYOSU_KEY_PASSWORD" "miner command"
require_contains "$validator_command" "--key-config-dir $temp_dir" "validator command"
require_contains "$validator_command" "--key-password-env MYOSU_KEY_PASSWORD" \
  "validator command"

eval "$miner_command --help" >/dev/null
eval "$validator_command --help" >/dev/null

bundle_dir="$temp_dir/bundle"
export MYOSU_OPERATOR_PASSWORD_ENV='MYOSU_KEY_PASSWORD'
export MYOSU_OPERATOR_CHAIN='ws://127.0.0.1:9944'
export MYOSU_OPERATOR_SUBNET='7'
bash .github/scripts/prepare_operator_network_bundle.sh "$bundle_dir" "$temp_dir" >/dev/null
test -x "$bundle_dir/start-miner.sh"
test -x "$bundle_dir/start-validator.sh"
test -x "$bundle_dir/build-devnet-spec.sh"
test -x "$bundle_dir/build-test-finney-spec.sh"
test -x "$bundle_dir/verify-bundle.sh"
test -s "$bundle_dir/bundle-manifest.toml"
test -s "$bundle_dir/README.md"
test -s "$bundle_dir/devnet-spec.json"
test -s "$bundle_dir/test-finney-spec.json"
"$bundle_dir/verify-bundle.sh" >/dev/null
python - "$bundle_dir/bundle-manifest.toml" "$bundle_dir/devnet-spec.json" <<'PY'
import json
import sys
import tomllib

manifest_path, spec_path = sys.argv[1:3]
with open(manifest_path, "rb") as manifest_file:
    manifest = tomllib.load(manifest_file)
with open(spec_path, "r", encoding="utf-8") as spec_file:
    spec = json.load(spec_file)
bootnode = manifest["bootnode_multiaddr"]
rpc_endpoint = manifest["bootnode_rpc_endpoint"]
if not bootnode or "/p2p/" not in bootnode:
    raise SystemExit("bundle manifest is missing a truthful bootnode_multiaddr")
if rpc_endpoint != "ws://127.0.0.1:9944":
    raise SystemExit(f"unexpected bootnode_rpc_endpoint: {rpc_endpoint}")
if bootnode not in spec.get("bootNodes", []):
    raise SystemExit("devnet-spec.json is missing the manifest bootnode")
PY

env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-chain --features fast-runtime -- \
  build-spec --chain devnet \
  >"$temp_dir/devnet-spec.json"
env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-chain --features fast-runtime -- \
  build-spec --chain test_finney \
  >"$temp_dir/test-finney-spec.json"

test -s "$temp_dir/devnet-spec.json"
test -s "$temp_dir/test-finney-spec.json"

echo "operator network bootstrap ok"
