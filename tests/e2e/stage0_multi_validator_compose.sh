#!/usr/bin/env bash
# Stage0 multi-validator compose proof.
#
# Implements the P0 row in IMPLEMENTATION_PLAN.md:
#   "Extend the stage0 multi-node compose path to a second validator: add a
#    `validator-2` service using `//myosu//devnet//validator-2` (already
#    endowed in devnet genesis) and assert both validators' submitted
#    weights for `miner-1` agree within INV-003 epsilon in the compose
#    proof's exit check."
#
# This proof mirrors the docker-compose topology in `docker-compose.yml`
# (one chain, one miner, `validator`, `validator-2`) as a process tree,
# so the same multi-validator agreement invariant is exercised in CI
# without requiring a docker daemon. Both validators are keyed to the
# dedicated operator URIs `//myosu//devnet//validator-1` and
# `//myosu//devnet//validator-2`, which `devnet.rs` already endows in
# the named `devnet` chain spec. Both score the same miner checkpoint
# + response that miner-1 publishes, and submit weights. The proof
# then reads the on-chain `Weights` rows for each validator and
# asserts the per-miner weight value agrees within the INV-003 epsilon
# (< 1e-6). A mismatch fails closed with a S0 INV-003 violation message.
#
# Required pre-conditions:
#   - wasm32v1-none target installed (rustup target add wasm32v1-none)
#   - docker-compose.yml is the topology under proof (both
#     `validator` and `validator-2` services use
#     `validator-runtime` and target `//myosu//devnet//validator-{1,2}`)
#
# Topology under proof (mirrors docker-compose.yml):
#
#   [authority-1 chain]  <-  [miner (//myosu//devnet//miner-1)]
#        |
#        |   ws://
#        +--> [validator (//myosu//devnet//validator-1)]
#        +--> [validator-2 (//myosu//devnet//validator-2)]
#
# Usage:  bash tests/e2e/stage0_multi_validator_compose.sh

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
cargo_bin_dir="$cargo_target_dir/debug"
node_bin="$cargo_target_dir/debug/myosu-chain"
runtime_wasm="$cargo_target_dir/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm"

chain_endpoint="${MYOSU_E2E_CHAIN_ENDPOINT:-ws://127.0.0.1:9955}"
rpc_url="http://127.0.0.1:9955"
owner_key="//myosu//devnet//subnet-owner"
miner_key="//myosu//devnet//miner-1"
validator_a_key="//myosu//devnet//validator-1"
validator_b_key="//myosu//devnet//validator-2"
validator_stake="${MYOSU_E2E_VALIDATOR_STAKE:-100000000000000}"
weight_epsilon="${MYOSU_E2E_COMPOSE_WEIGHT_EPSILON:-0.000001}"

node_log=""
node_pid=""
miner_port=""

cleanup() {
  if [[ -n "$node_pid" ]] && kill -0 "$node_pid" 2>/dev/null; then
    kill "$node_pid" 2>/dev/null || true
    wait "$node_pid" 2>/dev/null || true
  fi
  if [[ -n "$work_root" && -d "$work_root" && -z "${MYOSU_KEEP_E2E_WORK:-}" ]]; then
    rm -rf "$work_root"
  fi
}

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

select_local_port() {
  command -v python3 >/dev/null 2>&1 && PY=python3 || PY=python
  "$PY" - <<'PY'
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

wait_for_block() {
  local target_block="$1"
  local timeout_secs="${2:-180}"
  local deadline=$((SECONDS + timeout_secs))
  local request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'

  while (( SECONDS < deadline )); do
    if [[ -n "$node_pid" ]] && ! kill -0 "$node_pid" 2>/dev/null; then
      echo "compose chain exited before reaching block ${target_block}" >&2
      tail -n 120 "$node_log" >&2 || true
      exit 1
    fi

    local response block_hex current_block
    response="$(curl -fsS -H 'Content-Type: application/json' -d "$request" "$rpc_url" 2>/dev/null || true)"
    block_hex="$(printf '%s' "$response" | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p')"
    if [[ -n "$block_hex" ]]; then
      current_block=$((16#$block_hex))
      if (( current_block >= target_block )); then
        return 0
      fi
    fi

    sleep 1
  done

  echo "compose proof timed out waiting for block ${target_block}" >&2
  tail -n 120 "$node_log" >&2 || true
  exit 1
}

trap cleanup EXIT

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/stage0-multi-validator-compose.XXXXXX")"

if ! rustup target list --installed | grep -qx 'wasm32v1-none'; then
  echo "missing Rust target wasm32v1-none; run: rustup target add wasm32v1-none" >&2
  exit 1
fi

if [[ ! -f "$runtime_wasm" || ! -x "$node_bin" ]]; then
  echo "building myosu-chain runtime wasm cache (fast-runtime = 250ms slot)"
  run_logged "build_runtime" env -u SKIP_WASM_BUILD cargo build -p myosu-chain-runtime --features fast-runtime --quiet
  echo "building myosu-chain node (fast-runtime)"
  run_logged "build_node" env SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet
else
  echo "reusing existing myosu-chain runtime wasm cache and node binary"
fi

echo "booting single-authority named devnet (mirrors compose 'chain' service)"
node_log="$work_root/chain.log"
mkdir -p "$work_root/chain"
if [[ ! -f "$work_root/chain/node-key" ]]; then
  umask 077
  "$node_bin" key generate-node-key --file "$work_root/chain/node-key" >/dev/null 2>&1
fi
MYOSU_NODE_AUTHORITY_SURI="//myosu//devnet//authority-1" \
  "$node_bin" \
    --chain devnet \
    --base-path "$work_root/chain" \
    --node-key-file "$work_root/chain/node-key" \
    --validator \
    --force-authoring \
    --name "Stage0 Multi-Validator Compose Authority" \
    --rpc-port 9955 \
    --port 30444 \
    --prometheus-port 9616 \
    --allow-private-ip \
    >"$node_log" 2>&1 &
node_pid="$!"
wait_for_block 1 180
echo "compose chain up: pid=${node_pid}"

echo "building stage-0 operator binaries"
run_logged \
  "build_stage0_binaries" \
  env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-miner -p myosu-validator

poker_root="$work_root/poker"
encoder_dir="$poker_root/encoder"
query_file="$poker_root/query.bin"
response_file="$poker_root/response.bin"
miner_data_dir="$poker_root/miner-data"
checkpoint_path="$miner_data_dir/checkpoints/latest.bin"
miner_port="$(select_local_port)"

echo "subnet 7 is bootstrapped by the named devnet chain spec genesis"
echo "(see crates/myosu-chain/node/src/chain_spec/devnet.rs:DEVNET_SUBNET_UID = 7)"
echo "owner enables subnet staking so validators can submit weights"
owner_subtoken_output="$(
  run_logged \
    "owner_enable_subtoken" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-validator" \
      --chain "$chain_endpoint" \
      --subnet 7 \
      --key "$owner_key" \
      --enable-subtoken
)"
assert_contains "$owner_subtoken_output" "VALIDATOR myosu-validator bootstrap ok" "owner_enable_subtoken"
assert_contains "$owner_subtoken_output" "SUBTOKEN myosu-validator subnet ok" "owner_enable_subtoken"

echo "writing poker bootstrap artifacts"
bootstrap_output="$(
  run_logged \
    "bootstrap_artifacts" \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
    "$encoder_dir" "$query_file"
)"
assert_contains "$bootstrap_output" "BOOTSTRAP encoder_dir=${encoder_dir}" "bootstrap_artifacts"
assert_contains "$bootstrap_output" "BOOTSTRAP query_file=${query_file}" "bootstrap_artifacts"

echo "running miner bootstrap (//myosu//devnet//miner-1)"
miner_output="$(
  run_logged \
    "miner_bootstrap" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-miner" \
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
assert_contains "$miner_output" "MINER myosu-miner bootstrap ok" "miner_bootstrap"
assert_contains "$miner_output" "REGISTRATION myosu-miner subnet ok" "miner_bootstrap"
assert_contains "$miner_output" "AXON myosu-miner publish ok" "miner_bootstrap"
assert_contains "$miner_output" "TRAINING myosu-miner batch ok" "miner_bootstrap"
assert_contains "$miner_output" "STRATEGY myosu-miner query ok" "miner_bootstrap"
if [[ ! -s "$checkpoint_path" ]]; then
  echo "expected miner checkpoint at ${checkpoint_path}" >&2
  exit 1
fi
if [[ ! -s "$response_file" ]]; then
  echo "expected miner response file at ${response_file}" >&2
  exit 1
fi

echo "running validator-1 (//myosu//devnet//validator-1) bootstrap + weight submission"
validator_a_output="$(
  run_logged \
    "validator_a_bootstrap" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-validator" \
      --chain "$chain_endpoint" \
      --subnet 7 \
      --key "$validator_a_key" \
      --register \
      --stake-amount "$validator_stake" \
      --submit-weights \
      --weight-hotkey "$miner_key" \
      --encoder-dir "$encoder_dir" \
      --checkpoint "$checkpoint_path" \
      --query-file "$query_file" \
      --response-file "$response_file"
)"
assert_contains "$validator_a_output" "VALIDATOR myosu-validator bootstrap ok" "validator_a_bootstrap"
assert_contains "$validator_a_output" "REGISTRATION myosu-validator subnet ok" "validator_a_bootstrap"
assert_contains "$validator_a_output" "PERMIT myosu-validator ready ok" "validator_a_bootstrap"
assert_contains "$validator_a_output" "VALIDATION myosu-validator score ok" "validator_a_bootstrap"
assert_contains "$validator_a_output" "exact_match=true" "validator_a_bootstrap"
assert_contains "$validator_a_output" "WEIGHTS myosu-validator submission ok" "validator_a_bootstrap"

echo "running validator-2 (//myosu//devnet//validator-2) bootstrap + weight submission"
validator_b_output="$(
  run_logged \
    "validator_b_bootstrap" \
    env SKIP_WASM_BUILD=1 "$cargo_bin_dir/myosu-validator" \
      --chain "$chain_endpoint" \
      --subnet 7 \
      --key "$validator_b_key" \
      --register \
      --stake-amount "$validator_stake" \
      --submit-weights \
      --weight-hotkey "$miner_key" \
      --encoder-dir "$encoder_dir" \
      --checkpoint "$checkpoint_path" \
      --query-file "$query_file" \
      --response-file "$response_file"
)"
assert_contains "$validator_b_output" "VALIDATOR myosu-validator bootstrap ok" "validator_b_bootstrap"
assert_contains "$validator_b_output" "REGISTRATION myosu-validator subnet ok" "validator_b_bootstrap"
assert_contains "$validator_b_output" "PERMIT myosu-validator ready ok" "validator_b_bootstrap"
assert_contains "$validator_b_output" "VALIDATION myosu-validator score ok" "validator_b_bootstrap"
assert_contains "$validator_b_output" "exact_match=true" "validator_b_bootstrap"
assert_contains "$validator_b_output" "WEIGHTS myosu-validator submission ok" "validator_b_bootstrap"

echo "asserting both validators' submitted weights for miner-1 agree within INV-003 epsilon"
agreement_output="$(
  run_logged \
    "compose_proof_agreement" \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-chain-client --example compose_proof_driver -- \
      "$chain_endpoint" 7 "$miner_key" "$validator_a_key" "$validator_b_key" "$weight_epsilon"
)"
echo "$agreement_output"

miner_uid="$(require_kv "$agreement_output" "miner_uid")"
validator_a_uid="$(require_kv "$agreement_output" "validator_a_uid")"
validator_b_uid="$(require_kv "$agreement_output" "validator_b_uid")"
validator_a_target_weight="$(require_kv "$agreement_output" "validator_a_target_weight")"
validator_b_target_weight="$(require_kv "$agreement_output" "validator_b_target_weight")"
agreement_within_epsilon="$(require_kv "$agreement_output" "agreement_within_epsilon")"

if [[ "$agreement_within_epsilon" != "true" ]]; then
  echo "compose proof: validators' weights for miner_uid=${miner_uid} diverge beyond INV-003 epsilon" >&2
  echo "validator_a_target_weight=${validator_a_target_weight}" >&2
  echo "validator_b_target_weight=${validator_b_target_weight}" >&2
  exit 1
fi

if (( validator_a_target_weight == 0 || validator_b_target_weight == 0 )); then
  echo "compose proof: one validator did not submit a non-zero weight for miner_uid=${miner_uid}" >&2
  echo "validator_a_target_weight=${validator_a_target_weight}" >&2
  echo "validator_b_target_weight=${validator_b_target_weight}" >&2
  exit 1
fi

# Also assert exact integer equality for the integer-typed weights: the
# epsilon is the score-domain tolerance, but the on-chain Weights row
# is a u16 vector so two validators scoring the same response must
# produce the same exact weight value (otherwise an exact_match=true
# validator pass produced a non-deterministic quantization).
if [[ "$validator_a_target_weight" != "$validator_b_target_weight" ]]; then
  echo "compose proof: validator weights are not integer-equal for miner_uid=${miner_uid}" >&2
  echo "validator_a_target_weight=${validator_a_target_weight}" >&2
  echo "validator_b_target_weight=${validator_b_target_weight}" >&2
  exit 1
fi

echo "STAGE0_MULTI_VALIDATOR_COMPOSE myosu e2e ok"
echo "subnet=7"
echo "miner_key=${miner_key}"
echo "miner_uid=${miner_uid}"
echo "validator_a_key=${validator_a_key}"
echo "validator_a_uid=${validator_a_uid}"
echo "validator_b_key=${validator_b_key}"
echo "validator_b_uid=${validator_b_uid}"
echo "validator_a_target_weight=${validator_a_target_weight}"
echo "validator_b_target_weight=${validator_b_target_weight}"
echo "weight_epsilon=${weight_epsilon}"
echo "agreement_within_epsilon=${agreement_within_epsilon}"
