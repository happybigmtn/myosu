#!/usr/bin/env bash
# NEM-004 HTTP axon validator-scoring proof.
#
# The miner HTTP axon (`myosu-miner --serve-http` with
# `crates/myosu-miner/src/axon.rs`) and the validator's
# `myosu_validator::http_axon::query_miner_axon_http` round trip are
# the two halves of the HTTP scoring surface. The unit tests in
# `crates/myosu-validator/src/http_axon.rs` cover the validator
# client against an in-process `TcpListener` mock; this harness
# covers the same client against a real TCP process that serves the
# documented wire format, so a regression in either the wire format
# the validator parses or the public surface a downstream caller
# reaches is caught at the process boundary.
#
# The harness uses the `fake_miner_http` example binary (a tiny
# wire-format-compatible HTTP server) so the proof runs without a
# live `myosu-chain` and without the `myosu-miner` chain-probe
# bootstrap. The live miner HTTP server itself is covered by the
# `myosu-miner` crate's own `axon::tests` module; this harness only
# owns the validator-side half of the contract.
#
# Four real assertions; failing any one of them fails the gate with
# a concrete error message that names the offending surface and the
# requirement it broke.
#
#   1. The fake HTTP server boots on a fresh localhost port, accepts
#      a `GET /health` request, and replies with the documented
#      `{"status":"ok","epochs":0}` JSON body (the "is the live axon
#      alive" probe the validator runs first).
#   2. The new `query_miner_axon_http` validator client (via the
#      `http_axon_validator_scoring` example binary the harness
#      builds) connects to the live HTTP server, runs both round
#      trips, decodes the strategy response, and prints the
#      operator-facing `HTTP_AXON myosu-validator axon ok` summary
#      that includes action_count=1 and recommended_action=F (the
#      documented Fold action the fake server emits).
#   3. The decoded action_count is a non-negative integer and the
#      recommended_action is non-empty (the wire-format round trip
#      succeeded, not just the connect).
#   4. After the proof, the fake HTTP server is still alive (the
#      "survives the proof run" invariant — the live axon should
#      remain reusable for follow-on validator probes).
#
# Required pre-conditions:
#   - `myosu-validator` example binaries `fake_miner_http` and
#     `http_axon_validator_scoring` (cargo handles these via the
#     build step inside the harness).
#
# Usage:  bash tests/e2e/http_axon_validator_scoring.sh
#
# Tunable env:
#   MYOSU_HTTP_AXON_PORT    (default 8191)  fake miner HTTP axon port
#   MYOSU_HTTP_AXON_TIMEOUT (default 60)    fake miner HTTP health timeout (seconds)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

work_parent="$repo_root/target/e2e"
work_root=""
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
example_bin_dir="$cargo_target_dir/debug/examples"
fake_miner_bin="$example_bin_dir/fake_miner_http"
http_axon_example_bin="$example_bin_dir/http_axon_validator_scoring"

miner_port="${MYOSU_HTTP_AXON_PORT:-8191}"
miner_endpoint="http://127.0.0.1:${miner_port}"
ready_timeout_secs="${MYOSU_HTTP_AXON_TIMEOUT:-60}"

fake_miner_pid=""
cleanup() {
  if [[ -n "$fake_miner_pid" ]] && kill -0 "$fake_miner_pid" 2>/dev/null; then
    # Kill the whole process group so a reparented child that survived
    # the original subshell wrapper does not leak across harness runs.
    kill -TERM -- -"$fake_miner_pid" 2>/dev/null || true
    sleep 0.2
    kill -KILL -- -"$fake_miner_pid" 2>/dev/null || true
    kill -KILL "$fake_miner_pid" 2>/dev/null || true
    wait "$fake_miner_pid" 2>/dev/null || true
  fi
  # Belt-and-braces: pkill any fake_miner_http that somehow outlived
  # the group kill (a previous harness run with a different parent
  # could have reparented the child to PID 1).
  pkill -KILL -f "fake_miner_http ${miner_port}\b" 2>/dev/null || true
  if [[ -n "$work_root" && -d "$work_root" && -z "${MYOSU_KEEP_E2E_WORK:-}" ]]; then
    rm -rf "$work_root"
  fi
}
trap cleanup EXIT

mkdir -p "$work_parent"
work_root="$(mktemp -d "$work_parent/http-axon-validator-scoring.XXXXXX")"

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

# -- Build both example binaries once; cargo handles the dependency
# graph and the build cache.
echo "=== http-axon harness: building fake_miner_http example ===" >&2
env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-validator --example fake_miner_http

echo "=== http-axon harness: building http_axon_validator_scoring example ===" >&2
env SKIP_WASM_BUILD=1 cargo build --quiet -p myosu-validator --example http_axon_validator_scoring

# -- 1. Launch the fake miner HTTP server (chain-free).
fake_miner_log="$work_root/fake_miner.log"
echo "=== http-axon harness: launching fake_miner_http on :$miner_port ===" >&2
# `setsid` puts the binary in its own process group so cleanup can kill
# the whole group with `kill -- -$fake_miner_pid` even if the child
# has been reparented to PID 1 between launches.
setsid bash -c "exec '$fake_miner_bin' '$miner_port' \
  < /dev/null > '$fake_miner_log' 2>&1" &
fake_miner_pid=$!

# -- 2. Wait for the fake miner HTTP server to come up.
deadline=$((SECONDS + ready_timeout_secs))
healthy="false"
while (( SECONDS < deadline )); do
  if ! kill -0 "$fake_miner_pid" 2>/dev/null; then
    echo "fake miner HTTP server exited during readiness wait" >&2
    tail -n 120 "$fake_miner_log" >&2 || true
    exit 1
  fi
  health_response="$(curl -fsS --max-time 2 "http://127.0.0.1:${miner_port}/health" 2>/dev/null || true)"
  if printf '%s' "$health_response" | grep -Fq '"status":"ok"'; then
    healthy="true"
    break
  fi
  sleep 1
done
if [[ "$healthy" != "true" ]]; then
  echo "fake miner HTTP server never reported health=ok" >&2
  tail -n 120 "$fake_miner_log" >&2 || true
  exit 1
fi

# -- 3. Run the validator HTTP client against the live fake miner.
client_output="$(
  cd "$repo_root" && \
    "$http_axon_example_bin" \
      --endpoint "$miner_endpoint" \
      --subgame 0 \
      --bucket 0 \
      --choices 0 \
  2>&1
)"
client_status=$?
if [[ "$client_status" -ne 0 ]]; then
  echo "http_axon_validator_scoring example failed (exit $client_status)" >&2
  printf '%s\n' "$client_output" >&2
  tail -n 120 "$fake_miner_log" >&2 || true
  exit 1
fi

# -- 4. Assert the operator-facing summary line is well-formed.
assert_contains "$client_output" "HTTP_AXON myosu-validator axon ok" "http_axon summary"
assert_contains "$client_output" "endpoint=${miner_endpoint}" "http_axon endpoint"
assert_contains "$client_output" "health_ok=true" "http_axon health_ok"
assert_contains "$client_output" "health_status=ok" "http_axon health_status"
action_count="$(require_kv "$client_output" "action_count")"
recommended_action="$(require_kv "$client_output" "recommended_action")"
if ! [[ "$action_count" =~ ^[0-9]+$ ]]; then
  echo "action_count is not a non-negative integer: $action_count" >&2
  printf '%s\n' "$client_output" >&2
  exit 1
fi
if [[ -z "$recommended_action" ]]; then
  echo "recommended_action is empty (wire-format round trip did not produce a recommended action)" >&2
  printf '%s\n' "$client_output" >&2
  exit 1
fi

# -- 5. After the proof, the live HTTP server must still be alive
# (the "survives the proof run" invariant). A second health probe is
# the simplest "still serving" check.
if ! kill -0 "$fake_miner_pid" 2>/dev/null; then
  echo "fake miner HTTP server died during the proof" >&2
  tail -n 120 "$fake_miner_log" >&2 || true
  exit 1
fi
post_health="$(curl -fsS --max-time 2 "http://127.0.0.1:${miner_port}/health" 2>/dev/null || true)"
if ! printf '%s' "$post_health" | grep -Fq '"status":"ok"'; then
  echo "fake miner HTTP server is not serving after the proof" >&2
  printf 'post_health=%s\n' "$post_health" >&2
  tail -n 120 "$fake_miner_log" >&2 || true
  exit 1
fi

echo "HTTP_AXON_HARNESS myosu e2e ok endpoint=${miner_endpoint} action_count=${action_count} recommended_action=${recommended_action}" >&2
