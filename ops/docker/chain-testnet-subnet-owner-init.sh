#!/usr/bin/env bash
# Testnet subnet-owner registration entrypoint.
#
# One-shot helper that, after the testnet chain service is healthy, runs
# `myosu-validator --enable-subtoken` to register the testnet subnet owner
# and enable subnet 7 staking. This is the "registers the subnet owner"
# half of the P0 #4 milestone. The chain itself does not need to wait on
# this — the entrypoint's `wait-for-rpc.sh` blocks until the chain RPC is
# up, then submits the subnet-owner extrinsics, then exits.
#
# This deliberately uses `exec` so the compose service's exit status
# reflects the bootstrap's success or failure (a failed `enable-subtoken`
# submission kills the container, and the testnet operator can see the
# failure in `docker compose logs`).
set -euo pipefail

chain_endpoint="${MYOSU_CHAIN_ENDPOINT:-ws://chain:9944}"
chain_http_endpoint="${MYOSU_CHAIN_HTTP_ENDPOINT:-http://chain:9944}"
subnet="${MYOSU_SUBNET:-7}"
owner_key="${MYOSU_OWNER_KEY:-//myosu//testnet//subnet-owner}"
timeout_secs="${MYOSU_RPC_TIMEOUT_SECS:-180}"

/usr/local/bin/wait-for-rpc.sh "$chain_http_endpoint" "$timeout_secs"

echo "enabling subnet ${subnet} staking on the testnet chain (owner=${owner_key})"
exec /usr/local/bin/myosu-validator \
  --chain "$chain_endpoint" \
  --subnet "$subnet" \
  --key "$owner_key" \
  --enable-subtoken
