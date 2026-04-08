#!/usr/bin/env bash
set -euo pipefail

endpoint="${1:?usage: wait-for-rpc.sh <http-endpoint> [timeout-seconds]}"
timeout_secs="${2:-180}"
deadline=$((SECONDS + timeout_secs))
request='{"jsonrpc":"2.0","id":1,"method":"system_health","params":[]}'

while (( SECONDS < deadline )); do
  if curl --noproxy '*' -fsS \
    -H 'Content-Type: application/json' \
    -d "$request" \
    "$endpoint" >/dev/null; then
    exit 0
  fi

  sleep 1
done

echo "timed out waiting for chain RPC at ${endpoint}" >&2
exit 1
