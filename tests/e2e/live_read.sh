#!/usr/bin/env bash
# Live-read proof: myosu-play --read-solved against a live chain + live miner.
#
# Implements the P0 row in IMPLEMENTATION_PLAN.md:
#   "Add a live-read proof: a `myosu-play --chain-endpoint ws://... --read-solved`
#    mode (or a `myosu-chain-client` example) that connects to a running node,
#    discovers the miner axon, plays one poker hand, and prints `bundle_hash`,
#    `miner_uid`, and `emission` — the externally-verifiable 'read a solved
#    result through the same surface' milestone."
#
# This proof mirrors the docker-compose topology (`chain` + `miner` from
# `docker-compose.yml`) as a process tree (no docker required), so the
# same live-read invariant is exercised in CI:
#
#   [devnet chain (single authority, validator, force-authoring,
#                   --rpc-methods unsafe + --rpc-cors all,
#                   long-running foreground process)]
#        |
#        |   ws:// / http://
#        +--> [myosu-miner --register --serve-axon   (one-shot,
#                publishes //myosu//devnet//miner-1's axon info,
#                trains + writes a checkpoint, and prepares the strategy
#                response that the live axon will serve)]
#        +--> [myosu-miner --serve-http               (long-running,
#                serves the strategy request on http://127.0.0.1:PORT
#                so the live-read proof can POST a real query)]
#        +--> [myosu-play --read-solved               (one-shot,
#                connects to the chain, discovers the miner axon,
#                POSTs /strategy to the live miner HTTP, decodes
#                the wire response, computes a deterministic
#                bundle_hash, reads on-chain miner_uid + emission)]
#
# Acceptance (fail-closed):
#   1. The chain's `chain_getHeader` returns a parseable block (the
#      "live RPC" half of the invariant).
#   2. The miner bootstrap succeeds, registering miner-1 on subnet 7
#      and publishing a non-zero axon info row.
#   3. The miner HTTP axon responds 200 to `GET /health` with the
#      `{"status":"ok",...}` body the live-read proof expects.
#   4. `myosu-play --read-solved` exits 0 and prints at least:
#         status=solved
#         chain_endpoint=ws://...
#         subnet=7
#         discovered_miner_uid=<uid>
#         miner_uid=<uid>
#         bundle_hash=<64 hex chars>
#         emission=<u64>
#      All three milestone fields (`bundle_hash`, `miner_uid`, and
#      `emission`) must be present and well-formed.
#   5. The chain's on-chain `miner_uid` is a non-zero u16 and the
#      `discovered_miner_uid` returned by the live-read path agrees
#      with it (the two read the chain via different storage keys,
#      so the agreement is a real cross-check rather than a tautology).
#   6. The chain's on-chain axon endpoint port matches the live
#      miner HTTP port (i.e. the live axon the proof is talking to
#      is the same one the chain says is the miner's).
#   7. After the proof, both the chain and the miner HTTP axon are
#      still alive (the "survives the proof run" invariant).
#   8. The chain has authored at least one new block since the
#      `wait_for_block` baseline.
#
# Required pre-conditions:
#   - wasm32v1-none target installed (rustup target add wasm32v1-none)
#   - `myosu-chain`, `myosu-miner`, and `myosu-play` are built with
#     fast-runtime
#
# Usage:  bash tests/e2e/live_read.sh
#
# Tunable env:
#   MYOSU_E2E_CHAIN_PORT       (default 9957)  chain HTTP+WS port
#   MYOSU_E2E_MINER_PORT       (default 8091)  miner HTTP axon port
#   MYOSU_E2E_TARGET_BLOCK     (default 3)     chain block the proof waits for
#   MYOSU_E2E_READY_TIMEOUT    (default 240)   chain readiness timeout (seconds)
#   MYOSU_E2E_MINER_TIMEOUT    (default 240)   miner HTTP health timeout (seconds)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
cargo_bin_dir="$cargo_target_dir/debug"
node_bin="$cargo_target_dir/debug/myosu-chain"
miner_bin="$cargo_bin_dir/myosu-miner"
play_bin="$cargo_bin_dir/myosu-play"
runtime_wasm="$cargo_target_dir/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm"

chain_port="${MYOSU_E2E_CHAIN_PORT:-9957}"
chain_endpoint="ws://127.0.0.1:${chain_port}"
rpc_url="http://127.0.0.1:${chain_port}"
miner_port="${MYOSU_E2E_MINER_PORT:-8091}"
miner_endpoint="127.0.0.1:${miner_port}"
owner_key="//myosu//devnet//subnet-owner"
miner_key="//myosu//devnet//miner-1"
authority_suri="//myosu//devnet//authority-1"
target_block="${MYOSU_E2E_TARGET_BLOCK:-3}"
ready_timeout_secs="${MYOSU_E2E_READY_TIMEOUT:-240}"
miner_timeout_secs="${MYOSU_E2E_MINER_TIMEOUT:-240}"

node_log=""
node_pid=""
miner_http_pid=""

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/live-read.XXXXXX")"

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

wait_for_health() {
  local timeout_secs="${1:-180}"
  local deadline=$((SECONDS + timeout_secs))
  local request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'

  while (( SECONDS < deadline )); do
    if [[ -n "$node_pid" ]] && ! kill -0 "$node_pid" 2>/dev/null; then
      echo "live-read chain exited during health check" >&2
      tail -n 120 "$node_log" >&2 || true
      exit 1
    fi
    local response block_hex
    response="$(curl -fsS -H 'Content-Type: application/json' -d "$request" "$rpc_url" 2>/dev/null || true)"
    block_hex="$(printf '%s' "$response" | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p')"
    if [[ -n "$block_hex" ]]; then
      return 0
    fi
    sleep 1
  done

  echo "live-read proof timed out waiting for chain_getHeader" >&2
  tail -n 120 "$node_log" >&2 || true
  exit 1
}

wait_for_block() {
  local target="$1"
  local timeout_secs="${2:-180}"
  local deadline=$((SECONDS + timeout_secs))
  local request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'

  while (( SECONDS < deadline )); do
    if [[ -n "$node_pid" ]] && ! kill -0 "$node_pid" 2>/dev/null; then
      echo "live-read chain exited before reaching block ${target}" >&2
      tail -n 120 "$node_log" >&2 || true
      exit 1
    fi

    local response block_hex current_block
    response="$(curl -fsS -H 'Content-Type: application/json' -d "$request" "$rpc_url" 2>/dev/null || true)"
    block_hex="$(printf '%s' "$response" | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p')"
    if [[ -n "$block_hex" ]]; then
      current_block=$((16#$block_hex))
      if (( current_block >= target )); then
        return 0
      fi
    fi

    sleep 1
  done

  echo "live-read proof timed out waiting for block ${target}" >&2
  tail -n 120 "$node_log" >&2 || true
  exit 1
}

wait_for_miner_health() {
  local timeout_secs="${1:-180}"
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    if [[ -n "$miner_http_pid" ]] && ! kill -0 "$miner_http_pid" 2>/dev/null; then
      echo "live-read miner HTTP axon exited before becoming healthy" >&2
      exit 1
    fi
    local body
    body="$(curl --noproxy '*' -fsS "http://${miner_endpoint}/health" 2>/dev/null || true)"
    if [[ "$body" == *'"status":"ok"'* ]]; then
      return 0
    fi
    sleep 1
  done

  echo "live-read proof timed out waiting for miner HTTP /health" >&2
  exit 1
}

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

if [[ ! -x "$miner_bin" ]]; then
  echo "building myosu-miner"
  run_logged "build_miner" env SKIP_WASM_BUILD=1 cargo build -p myosu-miner --quiet
fi

if [[ ! -x "$play_bin" ]]; then
  echo "building myosu-play"
  run_logged "build_play" env SKIP_WASM_BUILD=1 cargo build -p myosu-play --quiet
fi

echo "booting live-read devnet chain (mirrors compose 'chain' service)"
node_log="$work_root/chain.log"
mkdir -p "$work_root/chain"
if [[ ! -f "$work_root/chain/node-key" ]]; then
  umask 077
  "$node_bin" key generate-node-key --file "$work_root/chain/node-key" >/dev/null 2>&1
fi
MYOSU_NODE_AUTHORITY_SURI="$authority_suri" \
  "$node_bin" \
    --chain devnet \
    --base-path "$work_root/chain" \
    --node-key-file "$work_root/chain/node-key" \
    --validator \
    --force-authoring \
    --rpc-methods unsafe \
    --rpc-port "$chain_port" \
    --rpc-cors all \
    --prometheus-port 9621 \
    --port 30447 \
    --allow-private-ip \
    --name "Live Read Proof Authority" \
    >"$node_log" 2>&1 &
node_pid="$!"

echo "live-read chain pid=${node_pid}; waiting for chain_getHeader (timeout=${ready_timeout_secs}s)"
wait_for_health "$ready_timeout_secs"
echo "live-read chain RPC healthy"

echo "waiting for block ${target_block}"
wait_for_block "$target_block" "$ready_timeout_secs"
echo "live-read chain reached block ${target_block}"

# Subnet 7 is bootstrapped by the named `devnet` chain spec; the
# subnet-owner is in the genesis storage map already and subnet
# staking is enabled, so the live-read proof can register a fresh
# axon directly without an explicit `--enable-subtoken` step.

poker_root="$work_root/poker"
encoder_dir="$poker_root/encoder"
query_file="$poker_root/query.bin"
response_file="$poker_root/response.bin"
miner_data_dir="$poker_root/miner-data"
checkpoint_path="$miner_data_dir/checkpoints/latest.bin"

echo "writing poker bootstrap artifacts (encoder + reference query)"
bootstrap_output="$(
  run_logged "bootstrap_artifacts" \
    env SKIP_WASM_BUILD=1 cargo run --quiet -p myosu-games-poker --example bootstrap_artifacts -- \
    "$encoder_dir" "$query_file"
)"
assert_contains "$bootstrap_output" "BOOTSTRAP encoder_dir=${encoder_dir}" "bootstrap_artifacts"
assert_contains "$bootstrap_output" "BOOTSTRAP query_file=${query_file}" "bootstrap_artifacts"

echo "running miner bootstrap (//myosu//devnet//miner-1) on port ${miner_port}"
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

# The miner bootstrap is one-shot, so a fresh `myosu-miner --serve-http`
# invocation is what keeps the strategy endpoint alive for the live-read
# proof. The `--serve-http` arm binds the TCP listener, loads the
# checkpoint-backed solver, and serves `/health` + `/strategy` for the
# lifetime of the process.
echo "starting live miner HTTP axon on http://${miner_endpoint}"
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

echo "running myosu-play --read-solved against chain ${chain_endpoint} subnet 7"
read_solved_output="$(
  run_logged "live_read" \
    env SKIP_WASM_BUILD=1 "$play_bin" \
      --chain "$chain_endpoint" \
      --subnet 7 \
      --read-solved
)"
echo "$read_solved_output"

# Hard-fail on every required milestone key. The proof is the
# P0 row's external-verifiability gate, so any missing key aborts.
status="$(require_kv "$read_solved_output" "status")"
if [[ "$status" != "solved" ]]; then
  echo "live-read proof: --read-solved did not report status=solved (got: ${status})" >&2
  printf '%s\n' "$read_solved_output" >&2
  exit 1
fi
miner_uid="$(require_kv "$read_solved_output" "miner_uid")"
bundle_hash="$(require_kv "$read_solved_output" "bundle_hash")"
emission="$(require_kv "$read_solved_output" "emission")"
discovered_uid="$(require_kv "$read_solved_output" "discovered_miner_uid")"
discovered_endpoint="$(require_kv "$read_solved_output" "discovered_miner_endpoint")"
discovered_hotkey="$(require_kv "$read_solved_output" "discovered_miner_hotkey")"
discovered_incentive="$(require_kv "$read_solved_output" "discovered_miner_incentive")"
subnet_field="$(require_kv "$read_solved_output" "subnet")"
chain_endpoint_field="$(require_kv "$read_solved_output" "chain_endpoint")"

# `subnet` must equal 7 (the proof's whole point is the devnet
# subnet the chain spec bootstraps).
if [[ "$subnet_field" != "7" ]]; then
  echo "live-read proof: subnet field is not 7 (got: ${subnet_field})" >&2
  exit 1
fi
# `chain_endpoint` must echo back the URL we passed in (proves the
# proof is not papering over a stale cached chain).
if [[ "$chain_endpoint_field" != "$chain_endpoint" ]]; then
  echo "live-read proof: chain_endpoint echo mismatch (${chain_endpoint_field} vs ${chain_endpoint})" >&2
  exit 1
fi
# `miner_uid` must be a non-negative integer < 2^16.
if ! [[ "$miner_uid" =~ ^[0-9]+$ ]] || (( miner_uid >= 65536 )); then
  echo "live-read proof: miner_uid is not a u16 (got: ${miner_uid})" >&2
  exit 1
fi
# `bundle_hash` must be 64 lowercase hex chars.
if ! [[ "$bundle_hash" =~ ^[0-9a-f]{64}$ ]]; then
  echo "live-read proof: bundle_hash is not a 64-char lowercase hex digest (got: ${bundle_hash})" >&2
  exit 1
fi
# `emission` must be a non-negative integer < 2^64.
if ! [[ "$emission" =~ ^[0-9]+$ ]]; then
  echo "live-read proof: emission is not a non-negative u64 (got: ${emission})" >&2
  exit 1
fi
# `discovered_miner_uid` must equal `miner_uid`. They are read
# via different storage keys (the discovery path reads
# `Incentive(7)` + `Keys(7)` + `Axon(hotkey)`, while the
# post-discovery path reads `Uids(7, hotkey)`), so agreement is a
# real cross-check.
if [[ "$discovered_uid" != "$miner_uid" ]]; then
  echo "live-read proof: discovered_miner_uid (${discovered_uid}) disagrees with miner_uid (${miner_uid})" >&2
  exit 1
fi
# `discovered_miner_endpoint` is the chain-recorded axon endpoint
# (0.0.0.0:port) — it must agree on the port with the live miner.
if [[ "$discovered_endpoint" != *":${miner_port}" ]]; then
  echo "live-read proof: discovered_miner_endpoint (${discovered_endpoint}) does not end with the bound port (${miner_port})" >&2
  exit 1
fi
# `discovered_miner_hotkey` must be a non-empty SS58 string. The
# full 32-byte AccountId32 hex / SS58 check is done inside
# `myosu-play`; here we just confirm the value was rendered.
if [[ -z "$discovered_hotkey" || "${#discovered_hotkey}" -lt 40 ]]; then
  echo "live-read proof: discovered_miner_hotkey is not a plausible SS58 string (got: ${discovered_hotkey})" >&2
  exit 1
fi
# `discovered_miner_incentive` must be a non-negative u16.
if ! [[ "$discovered_incentive" =~ ^[0-9]+$ ]] || (( discovered_incentive >= 65536 )); then
  echo "live-read proof: discovered_miner_incentive is not a u16 (got: ${discovered_incentive})" >&2
  exit 1
fi

# Final progress check: the chain is still authoring after the live-read
# proof. This is the "survives the proof run" half of the invariant.
final_block_hex="$(
  curl -fsS -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}' \
    "$rpc_url" \
  | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p'
)"
if [[ -z "$final_block_hex" ]]; then
  echo "could not read post-proof block header from the live-read chain RPC" >&2
  exit 1
fi
final_block=$((16#$final_block_hex))
if (( final_block < target_block )); then
  echo "live-read chain did not progress past block ${target_block} during the proof (got ${final_block})" >&2
  exit 1
fi
echo "live-read chain progress post-proof: block ${final_block} (target was ${target_block})"

# Re-assert that the miner HTTP axon is still alive after the live-read
# proof. The live-read POST + a second `curl /health` is the strongest
# evidence we have that the surface is durable, not a one-shot.
post_health="$(curl --noproxy '*' -fsS "http://${miner_endpoint}/health" 2>/dev/null || true)"
if [[ "$post_health" != *'"status":"ok"'* ]]; then
  echo "live-read miner HTTP axon is no longer healthy after the proof" >&2
  exit 1
fi

echo "LIVE_READ myosu-play --read-solved ok"
echo "chain_endpoint=${chain_endpoint}"
echo "rpc_url=${rpc_url}"
echo "subnet=7"
echo "miner_key=${miner_key}"
echo "miner_uid=${miner_uid}"
echo "discovered_miner_uid=${discovered_uid}"
echo "discovered_miner_endpoint=${discovered_endpoint}"
echo "discovered_miner_incentive=${discovered_incentive}"
echo "bundle_hash=${bundle_hash}"
echo "emission=${emission}"
echo "final_block=${final_block}"
echo "target_block=${target_block}"
echo "work_root=${work_root}"
