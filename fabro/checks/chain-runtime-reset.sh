#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

proof_target_dir="${MYOSU_CHAIN_TARGET_DIR:-/tmp/myosu-chain-target}"

export CARGO_TARGET_DIR="$proof_target_dir"
export CARGO_BUILD_TARGET_DIR="$proof_target_dir"
export WASM_BUILD_WORKSPACE_HINT="${WASM_BUILD_WORKSPACE_HINT:-$repo_root}"
export CARGO_NET_OFFLINE="${CARGO_NET_OFFLINE:-true}"

runtime_wbuild_dir="$CARGO_TARGET_DIR/release/wbuild/myosu-runtime"

run() {
    printf '+ %s\n' "$*"
    "$@"
}

run cargo check --offline -p myosu-chain-common
run cargo check --offline -p myosu-chain
run cargo check --offline -p myosu-runtime
run cargo build --offline --release -p myosu-runtime

run test -s "$runtime_wbuild_dir/myosu_runtime.wasm"
run test -s "$runtime_wbuild_dir/myosu_runtime.compact.wasm"
run test -s "$runtime_wbuild_dir/myosu_runtime.compact.compressed.wasm"
