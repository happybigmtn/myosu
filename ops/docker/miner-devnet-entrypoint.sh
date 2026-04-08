#!/usr/bin/env bash
set -euo pipefail

chain_endpoint="${MYOSU_CHAIN_ENDPOINT:-ws://chain:9944}"
chain_http_endpoint="${MYOSU_CHAIN_HTTP_ENDPOINT:-http://chain:9944}"
subnet="${MYOSU_SUBNET:-7}"
miner_key="${MYOSU_MINER_KEY:-//myosu//devnet//miner-1}"
miner_port="${MYOSU_MINER_PORT:-8080}"
timeout_secs="${MYOSU_RPC_TIMEOUT_SECS:-180}"
workdir="${MYOSU_WORKDIR:-/var/lib/myosu}"
bootstrap_root="/opt/myosu/bootstrap/poker"
encoder_dir="${bootstrap_root}/encoder"
query_file="${bootstrap_root}/query.bin"
miner_data_dir="${workdir}/miner-data"
response_file="${workdir}/response.bin"
checkpoint_path="${miner_data_dir}/checkpoints/latest.bin"

/usr/local/bin/wait-for-rpc.sh "$chain_http_endpoint" "$timeout_secs"

mkdir -p "$miner_data_dir"

echo "starting miner bootstrap"
/usr/local/bin/myosu-miner \
  --chain "$chain_endpoint" \
  --subnet "$subnet" \
  --key "$miner_key" \
  --port "$miner_port" \
  --register \
  --serve-axon \
  --encoder-dir "$encoder_dir" \
  --query-file "$query_file" \
  --response-file "$response_file" \
  --data-dir "$miner_data_dir"

test -s "$checkpoint_path"
test -s "$response_file"

echo "starting live miner HTTP axon"
exec /usr/local/bin/myosu-miner \
  --chain "$chain_endpoint" \
  --subnet "$subnet" \
  --key "$miner_key" \
  --port "$miner_port" \
  --encoder-dir "$encoder_dir" \
  --checkpoint "$checkpoint_path" \
  --serve-http
