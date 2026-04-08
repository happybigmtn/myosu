#!/usr/bin/env bash
set -euo pipefail

chain_endpoint="${MYOSU_CHAIN_ENDPOINT:-ws://chain:9944}"
chain_http_endpoint="${MYOSU_CHAIN_HTTP_ENDPOINT:-http://chain:9944}"
subnet="${MYOSU_SUBNET:-7}"
owner_key="${MYOSU_OWNER_KEY:-//myosu//devnet//subnet-owner}"
validator_key="${MYOSU_VALIDATOR_KEY:-//myosu//devnet//validator-1}"
miner_hotkey="${MYOSU_MINER_HOTKEY:-//myosu//devnet//miner-1}"
validator_stake="${MYOSU_VALIDATOR_STAKE:-100000000000000}"
miner_http_endpoint="${MYOSU_MINER_HTTP_ENDPOINT:-http://miner:8080/health}"
timeout_secs="${MYOSU_RPC_TIMEOUT_SECS:-180}"
workdir="${MYOSU_WORKDIR:-/var/lib/myosu}"
bootstrap_root="/opt/myosu/bootstrap/poker"
encoder_dir="${bootstrap_root}/encoder"
query_file="${bootstrap_root}/query.bin"
response_file="${workdir}/response.bin"
checkpoint_path="${workdir}/miner-data/checkpoints/latest.bin"
deadline=$((SECONDS + timeout_secs))

/usr/local/bin/wait-for-rpc.sh "$chain_http_endpoint" "$timeout_secs"

while (( SECONDS < deadline )); do
  if [[ -s "$checkpoint_path" && -s "$response_file" ]] \
    && curl --noproxy '*' -fsS "$miner_http_endpoint" >/dev/null; then
    break
  fi

  sleep 1
done

if [[ ! -s "$checkpoint_path" || ! -s "$response_file" ]]; then
  echo "validator bootstrap missing miner artifacts in ${workdir}" >&2
  exit 1
fi

echo "enabling subnet staking"
/usr/local/bin/myosu-validator \
  --chain "$chain_endpoint" \
  --subnet "$subnet" \
  --key "$owner_key" \
  --enable-subtoken

echo "running validator bootstrap"
exec /usr/local/bin/myosu-validator \
  --chain "$chain_endpoint" \
  --subnet "$subnet" \
  --key "$validator_key" \
  --register \
  --stake-amount "$validator_stake" \
  --submit-weights \
  --weight-hotkey "$miner_hotkey" \
  --encoder-dir "$encoder_dir" \
  --checkpoint "$checkpoint_path" \
  --query-file "$query_file" \
  --response-file "$response_file"
