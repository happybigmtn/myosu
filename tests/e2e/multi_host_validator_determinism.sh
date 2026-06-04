#!/usr/bin/env bash
# W-04 multi-host validator determinism proof.
#
# Implements the row in IMPLEMENTATION_PLAN.md:
#   "W-04 Multi-host validator determinism proof: two physical hosts (or
#    two container namespaces with distinct `MYOSU_OPERATOR_CHAIN` data
#    dirs and distinct SURI) score the same miner artifact and produce
#    weights that agree within INV-003 epsilon (Eng lens: INV-003 is
#    the chain's economic spine; the proof today is single-host)."
#
# The on-host base surface is `tests/e2e/validator_determinism.sh`
# (single-host, two SURI keys). This script is the multi-host-equivalent
# extension: two distinct SURI keys, two distinct `MYOSU_OPERATOR_CHAIN`
# data dirs, and the per-operator `multi_host_validator_determinism`
# example binary that ships with this row. The agreement check is the
# integer weight value the example emits — the same domain the on-chain
# `Weights` row stores, so `delta == 0` IS the INV-003 epsilon window
# (the helper `myosu-chain-client::evaluate_validator_agreement` already
# documents this).
#
# This proof mirrors the operator-bundle topology the W-04 spec calls
# out as the CI-grade composable surface, but it deliberately keeps the
# multi-host proof on the file-based query/response path (the path the
# live `myosu-validator score` binary uses) instead of the chain RPC
# path. The on-chain `Weights` row comparison is the on-host base
# surface's job; this row is the operator-facing multi-host scoring
# determinism contract.
#
# Topology under proof:
#
#   [bootstrap_artifacts] -> [encoder_dir, query.bin]
#                               |
#                               v
#                       [myosu-miner --register --serve-axon]
#                               |
#                               v
#                       [checkpoint.bin, response.bin]
#                               |
#                               +--> [multi_host_validator_determinism <validator-a, data-a>]
#                               +--> [multi_host_validator_determinism <validator-b, data-b>]
#                                       |
#                                       v
#                               assert weight_a == weight_b
#
# Required pre-conditions:
#   - wasm32v1-none target installed (rustup target add wasm32v1-none)
#   - myosu-miner and myosu-validator binaries built (cargo handles
#     this via the build step inside the harness).
#
# Usage:  bash tests/e2e/multi_host_validator_determinism.sh
#
# Tunable env:
#   MYOSU_E2E_CHAIN_ENDPOINT   (default ws://127.0.0.1:9955)
#   MYOSU_E2E_MINER_KEY        (default //Alice)
#   MYOSU_E2E_OWNER_KEY        (default //Alice)
#   MYOSU_E2E_VALIDATOR_A_KEY  (default //myosu//devnet//validator-1)
#   MYOSU_E2E_VALIDATOR_B_KEY  (default //myosu//devnet//validator-2)
#   MYOSU_E2E_MINER_PORT       (default 8091)
#   MYOSU_E2E_RPC_READY_TIMEOUT (default 60 seconds)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

helpers_dir="$repo_root/tests/e2e/helpers"
work_parent="$repo_root/target/e2e"
work_root=""
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
cargo_bin_dir="$cargo_target_dir/debug"
node_bin="$cargo_bin_dir/myosu-chain"
miner_bin="$cargo_bin_dir/myosu-miner"
example_bin_dir="$cargo_target_dir/debug/examples"
example_bin="$example_bin_dir/multi_host_validator_determinism"

chain_endpoint="${MYOSU_E2E_CHAIN_ENDPOINT:-ws://127.0.0.1:9955}"
rpc_url="http://127.0.0.1:9955"
owner_key="${MYOSU_E2E_OWNER_KEY:-//Alice}"
miner_key="${MYOSU_E2E_MINER_KEY:-//Alice}"
validator_a_key="${MYOSU_E2E_VALIDATOR_A_KEY:-//myosu//devnet//validator-1}"
validator_b_key="${MYOSU_E2E_VALIDATOR_B_KEY:-//myosu//devnet//validator-2}"
miner_port="${MYOSU_E2E_MINER_PORT:-8091}"
rpc_ready_timeout_secs="${MYOSU_E2E_RPC_READY_TIMEOUT:-60}"

started_devnet=0
node_pid=""

cleanup() {
  if [[ -n "$node_pid" ]] && kill -0 "$node_pid" 2>/dev/null; then
    kill "$node_pid" 2>/dev/null || true
    wait "$node_pid" 2>/dev/null || true
  fi
  if ((started_devnet)); then
    bash "$helpers_dir/stop_devnet.sh" >/dev/null 2>&1 || true
  fi
  if [[ -n "$work_root" && -d "$work_root" && -z "${MYOSU_KEEP_E2E_WORK:-}" ]]; then
    rm -rf "$work_root"
  fi
}

trap cleanup EXIT

require_kv() {
  local blob="$1"
  local key="$2"
  local value
  value="$(printf '%s\n' "$blob" | sed -n "s/^${key}=//p" | tail -n1)"
  if [[ -z "$value" ]]; then
    echo "missing output key ${key}" >&2
    printf '%s\n' "$blob" >&2
    exit 1
  fi
  printf '%s\n' "$value"
}

assert_contains() {
  local blob="$1"
  local needle="$2"
  local label="$3"
  if ! printf '%s\n' "$blob" | grep -Fq "$needle"; then
    echo "${label} missing expected text: ${needle}" >&2
    printf '%s\n' "$blob" >&2
    exit 1
  fi
}

assert_equal() {
  local left="$1"
  local right="$2"
  local label="$3"
  if [[ "$left" != "$right" ]]; then
    echo "${label} mismatch" >&2
    echo "left=${left}" >&2
    echo "right=${right}" >&2
    exit 1
  fi
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

select_local_port() {
  python - <<'PY'
import socket

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
    listener.bind(("127.0.0.1", 0))
    print(listener.getsockname()[1])
PY
}

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/multi-host-validator-determinism.XXXXXX")"

# -- 1. Build the binaries we need. cargo handles the dependency
# graph and the build cache.
echo "=== W-04 harness: building myosu-chain runtime wasm cache ===" >&2
if ! rustup target list --installed | grep -qx 'wasm32v1-none'; then
  echo "missing Rust target wasm32v1-none; run: rustup target add wasm32v1-none" >&2
  exit 1
fi
run_logged "build_runtime" env -u SKIP_WASM_BUILD cargo build -p myosu-chain-runtime --quiet

echo "=== W-04 harness: building myosu-chain node (fast-runtime) ===" >&2
run_logged "build_node" env SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet

echo "=== W-04 harness: building myosu-miner and myosu-validator ===" >&2
run_logged "build_binaries" env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-miner -p myosu-validator

echo "=== W-04 harness: building multi_host_validator_determinism example ===" >&2
run_logged "build_example" env SKIP_WASM_BUILD=1 cargo build -p myosu-validator --example multi_host_validator_determinism --quiet

# -- 2. Boot the local devnet via the shared helper. The helper
# persists a devnet.env file under `target/e2e/devnet/` that the
# `wait_for_block.sh` helper consumes, so the W-04 proof runs
# against the same devnet shape every other stage0 e2e proof uses.
echo "=== W-04 harness: starting local devnet ===" >&2
bash "$helpers_dir/start_devnet.sh" >&2
started_devnet=1

# -- 3. Wait for block 1 (the validator-permit-acquire path needs at
# least one block to exist before the miner can register its axon).
echo "=== W-04 harness: waiting for block 1 ===" >&2
MYOSU_E2E_WAIT_TIMEOUT="${MYOSU_E2E_WAIT_TIMEOUT:-60}" \
  bash "$helpers_dir/wait_for_block.sh" 1

# -- 5. Run the validator-determinism-style bootstrap to produce the
# shared artifacts the W-04 example consumes: encoder_dir, query.bin,
# checkpoint.bin, response.bin. This is the exact same artifact
# pipeline the on-host `tests/e2e/validator_determinism.sh` proof
# uses — the W-04 multi-host extension runs the per-operator scoring
# example on top of these artifacts.
echo "=== W-04 harness: writing poker bootstrap artifacts ===" >&2
encoder_dir="$work_root/encoder"
query_file="$work_root/query.bin"
bootstrap_output="$(
  run_logged "poker_bootstrap_artifacts" \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
    "$encoder_dir" "$query_file"
)"
assert_contains "$bootstrap_output" "BOOTSTRAP encoder_dir=${encoder_dir}" "poker_bootstrap_artifacts"
assert_contains "$bootstrap_output" "BOOTSTRAP query_file=${query_file}" "poker_bootstrap_artifacts"

response_file="$work_root/response.bin"
miner_data_dir="$work_root/miner-data"
checkpoint_path="$miner_data_dir/checkpoints/latest.bin"
echo "=== W-04 harness: running myosu-miner bootstrap ===" >&2
miner_output="$(
  run_logged "poker_miner_bootstrap" \
    env SKIP_WASM_BUILD=1 "$miner_bin" \
    --chain "$chain_endpoint" \
    --subnet 7 \
    --key "$miner_key" \
    --port "$miner_port" \
    --register \
    --serve-axon \
    --encoder-dir "$encoder_dir" \
    --query-file "$query_file" \
    --response-file "$response_file" \
    --data-dir "$miner_data_dir"
)"
assert_contains "$miner_output" "MINER myosu-miner bootstrap ok" "poker_miner_bootstrap"
assert_contains "$miner_output" "STRATEGY myosu-miner query ok" "poker_miner_bootstrap"
if [[ ! -f "$checkpoint_path" ]]; then
  echo "expected miner checkpoint at ${checkpoint_path}" >&2
  exit 1
fi
if [[ ! -f "$response_file" ]]; then
  echo "expected miner response file at ${response_file}" >&2
  exit 1
fi

# -- 6. Run the W-04 example twice with two distinct SURI-anchored
# data dirs. The "multi-host-equivalent" surface that fits in CI
# without provisioning two physical machines: each per-operator
# invocation runs in its own operator-data-dir, the operator's SURI
# is the seed the validator uses to derive the scoring key, and the
# two invocations score the same (query.bin, response.bin,
# checkpoint.bin) triple. INV-003 requires both runs to produce the
# SAME integer weight (the on-chain `Weights` row domain).
echo "=== W-04 harness: running validator-a scoring pass ===" >&2
validator_a_data_dir="$work_root/validator-a"
validator_a_output="$(
  run_logged "validator_a_scoring" \
    "$example_bin" \
      --miner-endpoint "http://127.0.0.1:${miner_port}" \
      --operator-data-dir "$validator_a_data_dir" \
      --suri "$validator_a_key" \
      --query-path "$query_file" \
      --response-path "$response_file" \
      --checkpoint-path "$checkpoint_path" \
      --encoder-dir "$encoder_dir" \
      --game poker
)"
assert_contains "$validator_a_output" "MULTI_HOST_VALIDATOR_DETERMINISM operator=${validator_a_key}" "validator_a header"
assert_contains "$validator_a_output" "MULTI_HOST_VALIDATOR_DETERMINISM_DATA_DIR operator=${validator_a_key} operator_data_dir=${validator_a_data_dir}" "validator_a data dir line"

echo "=== W-04 harness: running validator-b scoring pass ===" >&2
validator_b_data_dir="$work_root/validator-b"
validator_b_output="$(
  run_logged "validator_b_scoring" \
    "$example_bin" \
      --miner-endpoint "http://127.0.0.1:${miner_port}" \
      --operator-data-dir "$validator_b_data_dir" \
      --suri "$validator_b_key" \
      --query-path "$query_file" \
      --response-path "$response_file" \
      --checkpoint-path "$checkpoint_path" \
      --encoder-dir "$encoder_dir" \
      --game poker
)"
assert_contains "$validator_b_output" "MULTI_HOST_VALIDATOR_DETERMINISM operator=${validator_b_key}" "validator_b header"
assert_contains "$validator_b_output" "MULTI_HOST_VALIDATOR_DETERMINISM_DATA_DIR operator=${validator_b_key} operator_data_dir=${validator_b_data_dir}" "validator_b data dir line"

# -- 7. Extract the weight each operator produced. The integer-weight
# agreement check IS the INV-003 epsilon check (the on-chain `Weights`
# row domain is u16, so the floating-point tolerance window collapses
# to `delta == 0` here — the same `evaluate_validator_agreement`
# invariant the P0 multi-validator compose proof uses, applied at the
# per-operator scoring surface instead of the on-chain row).
validator_a_weight="$(require_kv "$validator_a_output" "weight")"
validator_b_weight="$(require_kv "$validator_b_output" "weight")"
validator_a_suri="$(require_kv "$validator_a_output" "operator")"
validator_b_suri="$(require_kv "$validator_b_output" "operator")"
validator_a_l1_distance="$(require_kv "$validator_a_output" "l1_distance")"
validator_b_l1_distance="$(require_kv "$validator_b_output" "l1_distance")"
validator_a_score="$(require_kv "$validator_a_output" "score")"
validator_b_score="$(require_kv "$validator_b_output" "score")"
validator_a_exact_match="$(require_kv "$validator_a_output" "exact_match")"
validator_b_exact_match="$(require_kv "$validator_b_output" "exact_match")"

# Both operators must report the integer weight they each computed for
# the same miner artifact. Agreement collapses to `==` because u16
# weights are integers (no floating-point tolerance window — the
# INV-003 epsilon on this domain is `delta == 0`).
assert_equal "$validator_a_weight" "$validator_b_weight" "INV-003 weight (validator_a vs validator_b)"
assert_equal "$validator_a_l1_distance" "$validator_b_l1_distance" "INV-003 l1_distance"
assert_equal "$validator_a_score" "$validator_b_score" "INV-003 score"
assert_equal "$validator_a_exact_match" "$validator_b_exact_match" "INV-003 exact_match"
assert_equal "$validator_a_suri" "$validator_a_key" "validator_a_suri"
assert_equal "$validator_b_suri" "$validator_b_key" "validator_b_suri"

# The two operator-data-dirs must NOT have leaked into each other. A
# regression in the example's --operator-data-dir contract that
# caused a hardlink or shared temp path would defeat the whole point
# of the multi-host extension.
if [[ -d "$validator_b_data_dir" ]] && \
   [[ -e "$validator_b_data_dir/$(basename "$validator_a_data_dir")" ]]; then
  echo "validator-b data dir contains a leaked validator-a path" >&2
  exit 1
fi
if [[ -d "$validator_a_data_dir" ]] && \
   [[ -e "$validator_a_data_dir/$(basename "$validator_b_data_dir")" ]]; then
  echo "validator-a data dir contains a leaked validator-b path" >&2
  exit 1
fi

echo "MULTI_HOST_VALIDATOR_DETERMINISM_HARNESS myosu e2e ok"
echo "validator_a_key=${validator_a_key}"
echo "validator_b_key=${validator_b_key}"
echo "validator_a_data_dir=${validator_a_data_dir}"
echo "validator_b_data_dir=${validator_b_data_dir}"
echo "validator_a_weight=${validator_a_weight}"
echo "validator_b_weight=${validator_b_weight}"
echo "validator_a_l1_distance=${validator_a_l1_distance}"
echo "validator_b_l1_distance=${validator_b_l1_distance}"
echo "validator_a_score=${validator_a_score}"
echo "validator_b_score=${validator_b_score}"
echo "validator_a_exact_match=${validator_a_exact_match}"
echo "validator_b_exact_match=${validator_b_exact_match}"
echo "weight_agreement=integer_equal"
echo "inv_003_epsilon=delta_eq_0"
