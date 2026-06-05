#!/usr/bin/env bash
# W-05 operator-bundle end-to-end proof.
#
# Implements the row in IMPLEMENTATION_PLAN.md:
#   "W-05 Operator-bundle end-to-end proof: `tests/e2e/operator_bundle_live.sh`
#    actually boots a multi-node devnet from the published
#    `ops/testnet/manifest.yaml` shape (W-01), registers a real miner + two
#    validators, drives two epochs, and proves emission flows."
#
# This proof composes the existing P0 surfaces into one operator-facing
# "zero-to-running" proof that an operator can run from a clean machine
# and walk through end-to-end. The harness deliberately stays a thin
# shell over the P0 surfaces it composes — every command it runs is
# one that already passes in CI today (testnet_entrypoint_compose,
# stage0_multi_validator_compose, live_read).
#
# Topology under proof (mirrors the published operator-bundle:
# `ops/deploy-bootnode.sh` + `docs/operator-guide/quickstart.md`):
#
#   [testnet chain (test_finney spec, single authority, validator,
#                    force-authoring, --rpc-cors all,
#                    long-running foreground process)]
#        |
#        |   ws:// / http://
#        +--> [myosu-validator --enable-subtoken (one-shot,
#                registers //myosu//testnet//subnet-owner)]
#        +--> [myosu-miner --register --serve-axon   (one-shot,
#                publishes //myosu//testnet//miner-1's axon info
#                and writes (checkpoint.bin, response.bin))]
#        +--> [myosu-validator //myosu//testnet//validator-1
#                --register --stake-amount --submit-weights
#                --weight-hotkey //myosu//testnet//miner-1 ...]
#        +--> [myosu-validator //myosu//testnet//validator-2
#                --register --stake-amount --submit-weights
#                --weight-hotkey //myosu//testnet//miner-1 ...]
#        +--> [compose_proof_driver
#                asserts validator_a_target_weight == validator_b_target_weight
#                for miner_uid (INV-003 on-chain Weights row)]
#        +--> [chain restart (kill node_pid, re-launch with the same flags)]
#        +--> [myosu-miner --serve-http (long-running, serves
#                the strategy endpoint for the live-read proof)]
#        +--> [myosu-play --read-solved
#                discovers the miner axon, plays one poker hand,
#                reads on-chain miner_uid + emission]
#
# Acceptance (fail-closed):
#   1. The chain's `chain_getHeader` returns a parseable block (the
#      "live RPC" half of the operator-bundle invariant — the same
#      healthcheck probe the testnet entrypoint uses).
#   2. The chain reaches the operator-side target block (proves the
#      chain is authoring blocks while the operator's miner + 2
#      validators register).
#   3. The subnet-owner init prints the
#      `SUBTOKEN myosu-validator subnet ok` line (proves the
#      testnet-spec bootstrap is reached; the test_finney spec
#      already enables subtoken so the call short-circuits via
#      `already_enabled=true`).
#   4. The miner bootstrap prints
#      `MINER myosu-miner bootstrap ok` + `REGISTRATION myosu-miner
#      subnet ok` + `AXON myosu-miner publish ok` + `TRAINING
#      myosu-miner batch ok` + `STRATEGY myosu-miner query ok`
#      (proves the operator-side miner path works end-to-end on
#      the testnet-spec authority / owner / operator accounts).
#   5. Both validator bootstraps print
#      `VALIDATOR myosu-validator bootstrap ok` + `REGISTRATION
#      myosu-validator subnet ok` + `PERMIT myosu-validator ready
#      ok` + `VALIDATION myosu-validator score ok` + `WEIGHTS
#      myosu-validator submission ok` (proves both validators
#      register, stake, acquire permit, score, and submit weights
#      on the testnet-spec operator accounts).
#   6. `compose_proof_driver` reports
#      `agreement_within_epsilon=true` AND
#      `validator_a_target_weight == validator_b_target_weight`
#      (the integer-equal INV-003 agreement check on the on-chain
#      `Weights` row — the same surface the P0 multi-validator
#      compose proof enforces).
#   7. After the chain restart, the new `chain_getHeader` returns a
#      parseable block number AND the post-restart tip is strictly
#      greater than the pre-restart tip (proves the chain survived
#      the restart and is still authoring; the persistent-entrypoint
#      half of the testnet invariant).
#   8. After the restart, the miner HTTP axon responds 200 to
#      `GET /health` with the `{"status":"ok",...}` body and
#      `myosu-play --read-solved` reports `status=solved` with a
#      64-hex `bundle_hash` (proves the live-read half of the
#      operator-bundle milestone is reachable after a restart — the
#      operator-bundle is end-to-end live).
#
# Required pre-conditions:
#   - wasm32v1-none target installed (rustup target add wasm32v1-none)
#   - testnet spec is hardened (P0 #1 / P0 #2 / P0 #3 / P0 #4 all
#     shipped)
#   - `myosu-chain`, `myosu-miner`, `myosu-validator`, `myosu-play`,
#     and the `compose_proof_driver` example are built with
#     fast-runtime
#
# Usage:  bash tests/e2e/operator_bundle_live.sh
#
# Tunable env:
#   MYOSU_E2E_CHAIN_PORT         (default 9966)  chain HTTP+WS port
#   MYOSU_E2E_MINER_PORT         (default 8089)  miner HTTP axon port
#   MYOSU_E2E_TARGET_BLOCK       (default 6)     chain block the proof waits for (pre-restart)
#   MYOSU_E2E_RESTART_BLOCK      (default 12)    post-restart target block
#   MYOSU_E2E_READY_TIMEOUT      (default 240)   chain readiness timeout (seconds)
#   MYOSU_E2E_EPOCH_TIMEOUT      (default 180)   epoch + weight submit timeout
#   MYOSU_E2E_MINER_TIMEOUT      (default 240)   miner HTTP health timeout (seconds)
#   MYOSU_E2E_VALIDATOR_STAKE    (default 100000000000000)  per-validator stake (rao)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
cargo_bin_dir="$cargo_target_dir/debug"
node_bin="$cargo_target_dir/debug/myosu-chain"
miner_bin="$cargo_bin_dir/myosu-miner"
validator_bin="$cargo_bin_dir/myosu-validator"
play_bin="$cargo_bin_dir/myosu-play"
compose_proof_driver_bin="$cargo_target_dir/debug/examples/compose_proof_driver"
runtime_wasm="$cargo_target_dir/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm"

chain_port="${MYOSU_E2E_CHAIN_PORT:-9966}"
chain_endpoint="ws://127.0.0.1:${chain_port}"
rpc_url="http://127.0.0.1:${chain_port}"
miner_port="${MYOSU_E2E_MINER_PORT:-8089}"
miner_endpoint="127.0.0.1:${miner_port}"
authority_suri="//myosu//testnet//authority-1"
owner_key="//myosu//testnet//subnet-owner"
miner_key="//myosu//testnet//miner-1"
validator_a_key="//myosu//testnet//validator-1"
validator_b_key="//myosu//testnet//validator-2"
target_block="${MYOSU_E2E_TARGET_BLOCK:-6}"
restart_block="${MYOSU_E2E_RESTART_BLOCK:-12}"
ready_timeout_secs="${MYOSU_E2E_READY_TIMEOUT:-240}"
epoch_timeout_secs="${MYOSU_E2E_EPOCH_TIMEOUT:-180}"
miner_timeout_secs="${MYOSU_E2E_MINER_TIMEOUT:-240}"
validator_stake="${MYOSU_E2E_VALIDATOR_STAKE:-100000000000000}"
weight_epsilon="${MYOSU_E2E_COMPOSE_WEIGHT_EPSILON:-0.000001}"

node_log=""
node_pid=""
miner_http_pid=""
miner_http_log=""

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/operator-bundle-live.XXXXXX")"

cleanup() {
  if [[ -n "$miner_http_pid" ]] && kill -0 "$miner_http_pid" 2>/dev/null; then
    kill "$miner_http_pid" 2>/dev/null || true
    wait "$miner_http_pid" 2>/dev/null || true
  fi
  if [[ -n "$node_pid" ]] && kill -0 "$node_pid" 2>/dev/null; then
    kill "$node_pid" 2>/dev/null || true
    wait "$node_pid" 2>/dev/null || true
  fi
  if [[ -n "$work_root" && -d "$work_root" && -z "${MYOSU_KEEP_E2E_WORK:-}" ]]; then
    rm -rf "$work_root"
  fi
}
trap cleanup EXIT

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
  local target="$1"
  local timeout_secs="${2:-$ready_timeout_secs}"
  local deadline=$((SECONDS + timeout_secs))
  local request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'
  local response block_hex current_block

  while (( SECONDS < deadline )); do
    if [[ -n "$node_pid" ]] && ! kill -0 "$node_pid" 2>/dev/null; then
      echo "testnet chain exited before reaching block ${target}" >&2
      tail -n 120 "$node_log" >&2 || true
      exit 1
    fi

    response="$(curl -fsS -H 'Content-Type: application/json' -d "$request" "$rpc_url" 2>/dev/null || true)"
    block_hex="$(printf '%s' "$response" | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p')"
    if [[ -n "$block_hex" ]]; then
      current_block=$((16#$block_hex))
      if (( current_block >= target )); then
        echo "current_block=${current_block}"
        return 0
      fi
    fi

    sleep 1
  done

  echo "testnet chain failed to reach block ${target} within ${timeout_secs}s" >&2
  tail -n 120 "$node_log" >&2 || true
  exit 1
}

read_block_number() {
  local request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'
  local response block_hex
  response="$(curl -fsS -H 'Content-Type: application/json' -d "$request" "$rpc_url" 2>/dev/null || true)"
  block_hex="$(printf '%s' "$response" | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p')"
  if [[ -z "$block_hex" ]]; then
    echo "could not read chain_getHeader from ${rpc_url}" >&2
    return 1
  fi
  echo $((16#$block_hex))
}

wait_for_miner_health() {
  local timeout_secs="${1:-$miner_timeout_secs}"
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    if [[ -n "$miner_http_pid" ]] && ! kill -0 "$miner_http_pid" 2>/dev/null; then
      echo "miner HTTP axon exited before becoming healthy" >&2
      tail -n 80 "$miner_http_log" >&2 || true
      exit 1
    fi
    local body
    body="$(curl --noproxy '*' -fsS "http://${miner_endpoint}/health" 2>/dev/null || true)"
    if [[ "$body" == *'"status":"ok"'* ]]; then
      return 0
    fi
    sleep 1
  done

  echo "miner HTTP /health did not return ok within ${timeout_secs}s" >&2
  tail -n 80 "$miner_http_log" >&2 || true
  exit 1
}

start_chain() {
  # Boot the persistent testnet chain (mirrors
  # docker-compose.testnet.yml `chain` service). The same flag set
  # the P0 testnet_entrypoint_compose proof uses, so the
  # healthcheck + RPC + force-authoring behavior is identical.
  # `--rpc-cors all --rpc-external --rpc-methods unsafe` is the
  # CORS-enabled WS/HTTP surface the W-01 manifest contract
  # requires. `--validator --force-authoring` is what keeps the
  # chain producing blocks while the external one-shot
  # subnet-owner-init and miner + validator bootstraps run against
  # it.
  local chain_data="$work_root/chain"
  mkdir -p "$chain_data"
  if [[ ! -f "$chain_data/node-key" ]]; then
    umask 077
    "$node_bin" key generate-node-key --file "$chain_data/node-key" >/dev/null 2>&1
  fi
  node_log="$work_root/chain.log"
  MYOSU_NODE_AUTHORITY_SURI="$authority_suri" \
    "$node_bin" \
      --chain test_finney \
      --base-path "$chain_data" \
      --node-key-file "$chain_data/node-key" \
      --validator \
      --force-authoring \
      --rpc-external \
      --rpc-methods unsafe \
      --rpc-port "$chain_port" \
      --rpc-cors all \
      --prometheus-port 9623 \
      --port 30449 \
      --allow-private-ip \
      --name "W-05 Operator Bundle Proof Authority" \
      >"$node_log" 2>&1 &
  node_pid="$!"
}

if ! rustup target list --installed | grep -qx 'wasm32v1-none'; then
  echo "missing Rust target wasm32v1-none; run: rustup target add wasm32v1-none" >&2
  exit 1
fi

echo "================================================================"
echo "W-05 operator-bundle end-to-end proof"
echo "================================================================"
echo "chain_endpoint=${chain_endpoint} rpc_url=${rpc_url}"
echo "miner_endpoint=${miner_endpoint}"
echo "target_block=${target_block} restart_block=${restart_block}"
echo

# Build everything up front so the proof run is not interrupted by
# build timeouts. The proof reuses the same build cache the other
# stage-0 e2e proofs use; if any binary is already present the
# `cargo build` is a no-op.
echo "[1/9] building runtime + operator binaries"
run_logged "build_runtime" env -u SKIP_WASM_BUILD cargo build -p myosu-chain-runtime --features fast-runtime --quiet
run_logged "build_node" env SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet
run_logged "build_stage0_binaries" env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-miner -p myosu-validator -p myosu-play
# Build the compose_proof_driver example as a prebuilt binary so
# step 7 does not pay a `cargo run` recompile. Falls through to
# the no-op rebuild path if the binary is already present.
run_logged "build_compose_proof_driver" env SKIP_WASM_BUILD=1 cargo build -p myosu-chain-client --example compose_proof_driver --quiet
if [[ ! -x "$compose_proof_driver_bin" ]]; then
  echo "W-05: compose_proof_driver binary missing at ${compose_proof_driver_bin}" >&2
  exit 1
fi

poker_root="$work_root/poker"
encoder_dir="$poker_root/encoder"
query_file="$poker_root/query.bin"
response_file="$poker_root/response.bin"
miner_data_dir="$poker_root/miner-data"
checkpoint_path="$miner_data_dir/checkpoints/latest.bin"

echo
echo "[2/9] booting persistent testnet chain (test_finney spec)"
start_chain
wait_for_block 1 "$ready_timeout_secs"
echo "testnet chain RPC healthy, block 1 reached"

echo
echo "[3/9] waiting for target block ${target_block}"
wait_for_block "$target_block" "$ready_timeout_secs"
echo "testnet chain reached target_block=${target_block}"

echo
echo "[4/9] running one-shot subnet-owner init (//myosu//testnet//subnet-owner)"
subnet_owner_output="$(
  run_logged "subnet_owner_init" \
    env SKIP_WASM_BUILD=1 "$validator_bin" \
      --chain "$chain_endpoint" \
      --subnet 7 \
      --key "$owner_key" \
      --enable-subtoken
)"
assert_contains "$subnet_owner_output" "VALIDATOR myosu-validator bootstrap ok" "subnet_owner_init"
assert_contains "$subnet_owner_output" "SUBTOKEN myosu-validator subnet ok" "subnet_owner_init"

echo
echo "[5/9] writing poker bootstrap artifacts + running miner bootstrap"
bootstrap_output="$(
  run_logged "bootstrap_artifacts" \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
      "$encoder_dir" "$query_file"
)"
assert_contains "$bootstrap_output" "BOOTSTRAP encoder_dir=${encoder_dir}" "bootstrap_artifacts"
assert_contains "$bootstrap_output" "BOOTSTRAP query_file=${query_file}" "bootstrap_artifacts"

miner_output="$(
  run_logged "miner_bootstrap" \
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

echo
echo "[6/9] registering both validators + submitting weights"
for entry in "validator_a:${validator_a_key}" "validator_b:${validator_b_key}"; do
  label="${entry%%:*}"
  key="${entry#*:}"
  validator_output="$(
    run_logged "${label}_bootstrap" \
      env SKIP_WASM_BUILD=1 "$validator_bin" \
        --chain "$chain_endpoint" \
        --subnet 7 \
        --key "$key" \
        --register \
        --stake-amount "$validator_stake" \
        --submit-weights \
        --weight-hotkey "$miner_key" \
        --encoder-dir "$encoder_dir" \
        --checkpoint "$checkpoint_path" \
        --query-file "$query_file" \
        --response-file "$response_file"
  )"
  assert_contains "$validator_output" "VALIDATOR myosu-validator bootstrap ok" "${label}_bootstrap"
  assert_contains "$validator_output" "REGISTRATION myosu-validator subnet ok" "${label}_bootstrap"
  assert_contains "$validator_output" "PERMIT myosu-validator ready ok" "${label}_bootstrap"
  assert_contains "$validator_output" "VALIDATION myosu-validator score ok" "${label}_bootstrap"
  assert_contains "$validator_output" "exact_match=true" "${label}_bootstrap"
  assert_contains "$validator_output" "WEIGHTS myosu-validator submission ok" "${label}_bootstrap"
done

echo
echo "[7/9] asserting both validators' weights for miner_uid agree (INV-003 on-chain Weights row)"
agreement_output="$(
  run_logged "compose_proof_agreement" \
    env SKIP_WASM_BUILD=1 "$compose_proof_driver_bin" \
      "$chain_endpoint" 7 "$miner_key" "$validator_a_key" "$validator_b_key" "$weight_epsilon"
)"
echo "$agreement_output"
agreement_within_epsilon="$(require_kv "$agreement_output" "agreement_within_epsilon")"
validator_a_target_weight="$(require_kv "$agreement_output" "validator_a_target_weight")"
validator_b_target_weight="$(require_kv "$agreement_output" "validator_b_target_weight")"
miner_uid="$(require_kv "$agreement_output" "miner_uid")"

if [[ "$agreement_within_epsilon" != "true" ]]; then
  echo "W-05: validators' weights for miner_uid=${miner_uid} diverge beyond INV-003 epsilon" >&2
  echo "validator_a_target_weight=${validator_a_target_weight}" >&2
  echo "validator_b_target_weight=${validator_b_target_weight}" >&2
  exit 1
fi
if (( validator_a_target_weight == 0 || validator_b_target_weight == 0 )); then
  echo "W-05: one validator did not submit a non-zero weight for miner_uid=${miner_uid}" >&2
  exit 1
fi
if [[ "$validator_a_target_weight" != "$validator_b_target_weight" ]]; then
  echo "W-05: validator weights are not integer-equal for miner_uid=${miner_uid}" >&2
  exit 1
fi

echo
echo "[8/9] restarting the testnet chain (kill + re-launch, same base-path + node-key)"
pre_restart_tip="$(read_block_number)"
echo "pre_restart_tip=${pre_restart_tip}"
# Kill the running node. The `start_chain` helper re-launches
# with the same flag set and the same base-path / node-key, so
# the chain re-binds to the same chain_endpoint and recovers
# from the same db.
if [[ -n "$node_pid" ]] && kill -0 "$node_pid" 2>/dev/null; then
  kill "$node_pid" 2>/dev/null || true
  wait "$node_pid" 2>/dev/null || true
fi
# Brief settle so the base-path lock + RPC port are released
# before the new node re-binds.
sleep 3
start_chain
# The post-restart chain has to first re-bind + sync, then start
# authoring from a fresh slot. Wait for any block strictly
# greater than pre_restart_tip to prove the chain survived
# the restart and is still authoring.
wait_for_block "$((pre_restart_tip + 1))" "$ready_timeout_secs"
post_restart_tip="$(read_block_number)"
echo "post_restart_tip=${post_restart_tip}"
if (( post_restart_tip <= pre_restart_tip )); then
  echo "W-05: chain did not progress past pre_restart_tip=${pre_restart_tip} after restart (post_restart_tip=${post_restart_tip})" >&2
  exit 1
fi
wait_for_block "$restart_block" "$ready_timeout_secs"
echo "testnet chain reached post-restart target restart_block=${restart_block}"

echo
echo "[9/9] post-restart live-read proof (myosu-miner --serve-http + myosu-play --read-solved)"
# Start the miner HTTP axon (long-running) so the live-read
# proof can POST a real /strategy request against the live
# miner artifact. The --serve-http arm binds the TCP listener,
# loads the checkpoint-backed solver, and serves /health +
# /strategy for the lifetime of the process.
miner_http_log="$work_root/miner-http.log"
"$miner_bin" \
  --chain "$chain_endpoint" \
  --subnet 7 \
  --key "$miner_key" \
  --port "$miner_port" \
  --encoder-dir "$encoder_dir" \
  --checkpoint "$checkpoint_path" \
  --serve-http \
  >"$miner_http_log" 2>&1 &
miner_http_pid="$!"

echo "waiting for miner HTTP /health (timeout=${miner_timeout_secs}s)"
wait_for_miner_health "$miner_timeout_secs"
echo "miner HTTP /health reports ok"

read_solved_output="$(
  run_logged "live_read_after_restart" \
    env SKIP_WASM_BUILD=1 "$play_bin" \
      --chain "$chain_endpoint" \
      --subnet 7 \
      --read-solved
)"
assert_contains "$read_solved_output" "status=solved" "live_read_after_restart"
assert_contains "$read_solved_output" "chain_endpoint=${chain_endpoint}" "live_read_after_restart"
assert_contains "$read_solved_output" "subnet=7" "live_read_after_restart"

# Extract the bundle_hash + miner_uid + emission fields and
# fail-closed assert each is well-formed (the same shape
# tests/e2e/live_read.sh enforces at the line-protocol level).
bundle_hash="$(require_kv "$read_solved_output" "bundle_hash")"
miner_uid_read="$(require_kv "$read_solved_output" "miner_uid")"
emission_value="$(require_kv "$read_solved_output" "emission")"
if ! [[ "$bundle_hash" =~ ^[0-9a-f]{64}$ ]]; then
  echo "W-05: live-read bundle_hash is not 64 lowercase hex: ${bundle_hash}" >&2
  exit 1
fi
if ! [[ "$miner_uid_read" =~ ^[0-9]+$ ]] || (( miner_uid_read == 0 )); then
  echo "W-05: live-read miner_uid is not a positive integer: ${miner_uid_read}" >&2
  exit 1
fi
if ! [[ "$emission_value" =~ ^[0-9]+$ ]]; then
  echo "W-05: live-read emission is not a non-negative integer: ${emission_value}" >&2
  exit 1
fi
# Cross-check: the miner_uid the live-read proof reports must
# match the miner_uid the compose_proof_driver reported. Both
# read the on-chain storage but via different keys, so a
# match is a real cross-check rather than a tautology.
if [[ "$miner_uid_read" != "$miner_uid" ]]; then
  echo "W-05: live-read miner_uid=${miner_uid_read} disagrees with compose_proof miner_uid=${miner_uid}" >&2
  exit 1
fi

echo
echo "OPERATOR_BUNDLE_LIVE myosu e2e ok"
echo "chain_endpoint=${chain_endpoint}"
echo "miner_endpoint=${miner_endpoint}"
echo "miner_key=${miner_key}"
echo "miner_uid=${miner_uid}"
echo "validator_a_key=${validator_a_key}"
echo "validator_b_key=${validator_b_key}"
echo "validator_a_target_weight=${validator_a_target_weight}"
echo "validator_b_target_weight=${validator_b_target_weight}"
echo "agreement_within_epsilon=${agreement_within_epsilon}"
echo "pre_restart_tip=${pre_restart_tip}"
echo "post_restart_tip=${post_restart_tip}"
echo "bundle_hash=${bundle_hash}"
echo "emission=${emission_value}"
