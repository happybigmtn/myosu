#!/usr/bin/env bash
# Stage0 multi-validator compose proof.
#
# Boots the docker-compose stage0 stack (`chain` + `miner` + `validator` +
# `validator-2`), waits for both validators to finish their on-chain
# `set_weights` submission against `//myosu//devnet//miner-1`, then runs the
# `compose_proof_driver` example over the chain RPC to fetch the on-chain
# `Weights` row each validator submitted and assert the two values agree
# within the INV-003 epsilon window.
#
# This is the fail-closed proof the IMPLEMENTATION_PLAN P0 row requires:
#   "Extend the stage0 multi-node compose path to a second validator: add a
#    `validator-2` service using `//myosu//devnet//validator-2` (already
#    endowed in devnet genesis) and assert both validators' submitted
#    weights for `miner-1` agree within INV-003 epsilon in the compose
#    proof's exit check."
#
# Usage:
#   tests/e2e/compose_proof.sh
#   MYOSU_COMPOSE_NO_BUILD=1 tests/e2e/compose_proof.sh  # reuse prebuilt images
#   MYOSU_COMPOSE_EPSILON=0.000001 tests/e2e/compose_proof.sh
#   MYOSU_COMPOSE_PROJECT=myosu-stage0 tests/e2e/compose_proof.sh
#
# Required env:
#   docker + docker compose v2
#
# Exit codes:
#   0  proof passed: both validators' weights agree within INV-003 epsilon
#      and target the miner UID on subnet 7.
#   1  a validator did not finish, the driver failed, weights diverged, or
#      the on-chain row was missing the miner entry.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
compose_file="$repo_root/docker-compose.yml"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/myosu-compose-proof.XXXXXX")"
project_name="${MYOSU_COMPOSE_PROJECT:-myosu-stage0-proof}"
epsilon="${MYOSU_COMPOSE_EPSILON:-0.000001}"
chain_http_endpoint="http://127.0.0.1:9944"
chain_ws_endpoint="ws://127.0.0.1:9944"
subnet="${MYOSU_COMPOSE_SUBNET:-7}"
miner_uri="${MYOSU_COMPOSE_MINER_URI:-//myosu//devnet//miner-1}"
validator_a_uri="${MYOSU_COMPOSE_VALIDATOR_A_URI:-//myosu//devnet//validator-1}"
validator_b_uri="${MYOSU_COMPOSE_VALIDATOR_B_URI:-//myosu//devnet//validator-2}"
build_images="${MYOSU_COMPOSE_NO_BUILD:+0}"
build_images="${build_images:-1}"
health_timeout_secs="${MYOSU_COMPOSE_HEALTH_TIMEOUT_SECS:-600}"
validator_finish_timeout_secs="${MYOSU_COMPOSE_VALIDATOR_FINISH_TIMEOUT_SECS:-900}"

log() {
  printf '[compose_proof] %s\n' "$*" >&2
}

require_command() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd" >&2
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

cleanup() {
  log "bringing compose stack down (project=${project_name})"
  docker compose -p "$project_name" -f "$compose_file" down -v --remove-orphans \
    >"$work_root/compose-down.stdout" 2>"$work_root/compose-down.stderr" || true
  log "compose down logs at $work_root/compose-down.{stdout,stderr}"
}

trap cleanup EXIT

require_command docker
if ! docker compose version >/dev/null 2>&1; then
  echo "docker compose v2 plugin required" >&2
  exit 1
fi

if [[ ! -f "$compose_file" ]]; then
  echo "compose file not found: $compose_file" >&2
  exit 1
fi

if [[ -n "${MYOSU_COMPOSE_EPSILON:-}" ]]; then
  if ! python -c "import math, sys; sys.exit(0 if math.isfinite(float(sys.argv[1])) and float(sys.argv[1]) >= 0.0 else 1)" "$epsilon"; then
    echo "invalid MYOSU_COMPOSE_EPSILON: $epsilon (must be finite and non-negative)" >&2
    exit 1
  fi
fi

log "work root: $work_root"
log "compose file: $compose_file"
log "project: $project_name"
log "epsilon: $epsilon"

if [[ "$build_images" == "1" ]]; then
  log "building compose images (chain-runtime, miner-runtime, validator-runtime)"
  docker compose -p "$project_name" -f "$compose_file" build \
    >"$work_root/compose-build.stdout" 2>"$work_root/compose-build.stderr"
fi

log "starting compose stack (chain + miner + validator + validator-2)"
docker compose -p "$project_name" -f "$compose_file" up -d chain miner validator validator-2 \
  >"$work_root/compose-up.stdout" 2>"$work_root/compose-up.stderr"

log "waiting for chain health"
chain_health_deadline=$(( $(date +%s) + health_timeout_secs ))
while (( $(date +%s) < chain_health_deadline )); do
  if curl -fsS -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
      "$chain_http_endpoint" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
if ! curl -fsS -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}' \
    "$chain_http_endpoint" >/dev/null 2>&1; then
  echo "chain RPC never became reachable at $chain_http_endpoint" >&2
  docker compose -p "$project_name" -f "$compose_file" logs --tail=200 \
    >"$work_root/compose-logs-chain-timeout.stdout" 2>&1 || true
  exit 1
fi

log "waiting for miner health (this also waits for the miner to produce a checkpoint and strategy response)"
miner_health_deadline=$(( $(date +%s) + health_timeout_secs ))
while (( $(date +%s) < miner_health_deadline )); do
  if curl --noproxy '*' -fsS "http://127.0.0.1:8080/health" 2>/dev/null \
       | grep -q '"status":"ok"'; then
    break
  fi
  sleep 3
done
if ! curl --noproxy '*' -fsS "http://127.0.0.1:8080/health" 2>/dev/null \
     | grep -q '"status":"ok"'; then
  echo "miner HTTP health never became reachable at http://127.0.0.1:8080/health" >&2
  docker compose -p "$project_name" -f "$compose_file" logs --tail=200 \
    >"$work_root/compose-logs-miner-timeout.stdout" 2>&1 || true
  exit 1
fi

log "waiting for both validators to finish on-chain weight submission"
validator_finish_deadline=$(( $(date +%s) + validator_finish_timeout_secs ))
while (( $(date +%s) < validator_finish_deadline )); do
  a_done=0
  b_done=0
  a_status="$(docker compose -p "$project_name" -f "$compose_file" ps -a --format json validator 2>/dev/null || true)"
  b_status="$(docker compose -p "$project_name" -f "$compose_file" ps -a --format json validator-2 2>/dev/null || true)"

  if printf '%s' "$a_status" | grep -q '"State":"exited"'; then
    a_done=1
  fi
  if printf '%s' "$b_status" | grep -q '"State":"exited"'; then
    b_done=1
  fi

  a_logs="$(docker compose -p "$project_name" -f "$compose_file" logs --no-color validator 2>/dev/null || true)"
  b_logs="$(docker compose -p "$project_name" -f "$compose_file" logs --no-color validator-2 2>/dev/null || true)"

  if (( a_done == 0 )) && printf '%s' "$a_logs" | grep -q 'WEIGHTS myosu-validator submission ok'; then
    a_done=1
  fi
  if (( b_done == 0 )) && printf '%s' "$b_logs" | grep -q 'WEIGHTS myosu-validator submission ok'; then
    b_done=1
  fi

  if (( a_done == 1 && b_done == 1 )); then
    break
  fi

  sleep 5
done

a_logs="$(docker compose -p "$project_name" -f "$compose_file" logs --no-color validator 2>/dev/null || true)"
b_logs="$(docker compose -p "$project_name" -f "$compose_file" logs --no-color validator-2 2>/dev/null || true)"
printf '%s\n' "$a_logs" >"$work_root/validator.stdout"
printf '%s\n' "$b_logs" >"$work_root/validator-2.stdout"

if ! printf '%s' "$a_logs" | grep -q 'WEIGHTS myosu-validator submission ok'; then
  echo "validator never printed 'WEIGHTS myosu-validator submission ok'" >&2
  printf '%s\n' "$a_logs" >&2
  exit 1
fi
if ! printf '%s' "$b_logs" | grep -q 'WEIGHTS myosu-validator submission ok'; then
  echo "validator-2 never printed 'WEIGHTS myosu-validator submission ok'" >&2
  printf '%s\n' "$b_logs" >&2
  exit 1
fi

log "building compose_proof_driver"
SKIP_WASM_BUILD=1 cargo build --quiet \
  -p myosu-chain-client --example compose_proof_driver

log "running compose_proof_driver"
driver_output="$(
  SKIP_WASM_BUILD=1 cargo run --quiet \
    -p myosu-chain-client --example compose_proof_driver -- \
    "$chain_ws_endpoint" "$subnet" "$miner_uri" "$validator_a_uri" "$validator_b_uri" "$epsilon" \
    2>"$work_root/compose_proof_driver.stderr"
)"
printf '%s\n' "$driver_output" >"$work_root/compose_proof_driver.stdout"

if ! printf '%s' "$driver_output" | grep -q '^agreement_within_epsilon=true$'; then
  echo "compose proof agreement assertion FAILED" >&2
  printf '%s\n' "$driver_output" >&2
  exit 1
fi

log "compose_proof_driver output:"
printf '%s\n' "$driver_output"

miner_uid="$(require_kv "$driver_output" "miner_uid")"
validator_a_uid="$(require_kv "$driver_output" "validator_a_uid")"
validator_b_uid="$(require_kv "$driver_output" "validator_b_uid")"
validator_a_target_weight="$(require_kv "$driver_output" "validator_a_target_weight")"
validator_b_target_weight="$(require_kv "$driver_output" "validator_b_target_weight")"
agreement="$(require_kv "$driver_output" "agreement_within_epsilon")"

assert_equal "$agreement" "true" "agreement_within_epsilon"
if (( validator_a_target_weight == 0 || validator_b_target_weight == 0 )); then
  echo "validator target weight must be non-zero (got a=${validator_a_target_weight} b=${validator_b_target_weight})" >&2
  exit 1
fi
if (( validator_a_target_weight != validator_b_target_weight )); then
  echo "validator weights diverge beyond INV-003 epsilon: a=${validator_a_target_weight} b=${validator_b_target_weight}" >&2
  exit 1
fi

echo "COMPOSE_PROOF myosu stage0 multi-validator ok"
echo "project=${project_name}"
echo "subnet=${subnet}"
echo "miner_uri=${miner_uri}"
echo "validator_a_uri=${validator_a_uri}"
echo "validator_b_uri=${validator_b_uri}"
echo "miner_uid=${miner_uid}"
echo "validator_a_uid=${validator_a_uid}"
echo "validator_b_uid=${validator_b_uid}"
echo "validator_a_target_weight=${validator_a_target_weight}"
echo "validator_b_target_weight=${validator_b_target_weight}"
echo "epsilon=${epsilon}"
echo "agreement_within_epsilon=${agreement}"
echo "work_root=${work_root}"
