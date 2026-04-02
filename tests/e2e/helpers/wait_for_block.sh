#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <target-block>" >&2
  exit 1
fi

target_block="$1"
if [[ ! "$target_block" =~ ^[0-9]+$ ]]; then
  echo "target block must be a non-negative integer" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
runtime_dir="${MYOSU_E2E_DIR:-$repo_root/target/e2e/devnet}"
state_file="$runtime_dir/devnet.env"
wait_timeout="${MYOSU_E2E_WAIT_TIMEOUT:-60}"

if [[ ! -f "$state_file" ]]; then
  echo "missing devnet state file: $state_file" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "$state_file"

if [[ ! -f "$PID_FILE" ]]; then
  echo "missing devnet pid file: $PID_FILE" >&2
  exit 1
fi

node_pid="$(<"$PID_FILE")"
deadline=$((SECONDS + wait_timeout))
request='{"jsonrpc":"2.0","id":1,"method":"chain_getHeader","params":[]}'

while (( SECONDS < deadline )); do
  if ! kill -0 "$node_pid" 2>/dev/null; then
    echo "myosu e2e devnet pid $node_pid exited before reaching block $target_block" >&2
    tail -n 120 "$LOG_FILE" >&2 || true
    exit 1
  fi

  response="$(curl -fsS -H 'Content-Type: application/json' -d "$request" "$RPC_URL" 2>/dev/null || true)"
  block_hex="$(printf '%s' "$response" | sed -n 's/.*"number":"0x\([0-9a-fA-F]\+\)".*/\1/p')"
  if [[ -n "$block_hex" ]]; then
    current_block=$((16#$block_hex))
    if (( current_block >= target_block )); then
      echo "myosu e2e devnet reached block $current_block"
      exit 0
    fi
  fi

  sleep 1
done

echo "timed out waiting ${wait_timeout}s for block ${target_block}" >&2
tail -n 120 "$LOG_FILE" >&2 || true
exit 1

