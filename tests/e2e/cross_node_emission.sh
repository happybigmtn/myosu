#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work_parent="$repo_root/target/e2e"
work_root=""
cargo_target_dir="${CARGO_TARGET_DIR:-$repo_root/target}"
node_bin="$cargo_target_dir/debug/myosu-chain"
runtime_wasm="$cargo_target_dir/debug/wbuild/myosu-chain-runtime/myosu_chain_runtime.wasm"
driver_root="$repo_root/target/e2e"
driver_target_dir="${MYOSU_E2E_CARGO_TARGET_DIR:-$driver_root/cross-node-emission-target}"
driver_examples_dir="$repo_root/crates/myosu-chain-client/examples"
driver_source=""
driver_example_name=""

rpc_ready_timeout="${MYOSU_E2E_RPC_READY_TIMEOUT:-120}"
initial_finality_timeout="${MYOSU_E2E_INITIAL_FINALITY_TIMEOUT:-120}"
epoch_finality_timeout="${MYOSU_E2E_EPOCH_FINALITY_TIMEOUT:-180}"
initial_finalized_target="${MYOSU_E2E_INITIAL_FINALIZED_TARGET:-2}"
network_backend="${MYOSU_E2E_NETWORK_BACKEND:-}"

declare -a node_base_dirs node_key_files node_logs node_pids node_p2p_ports node_rpc_ports
declare -a node_prometheus_ports node_rpc_urls node_ws_urls node_multiaddrs
declare -a shared_best_blocks shared_finalized_blocks shared_peer_counts
shared_finalized_hash=""
shared_block_hash=""

cleanup() {
  if [[ -n "$driver_source" && -f "$driver_source" ]]; then
    rm -f "$driver_source"
  fi

  for idx in 4 3 2 1; do
    local pid="${node_pids[$idx]:-}"
    if [[ -n "$pid" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" 2>/dev/null || true
      wait "$pid" 2>/dev/null || true
    fi
  done
}

select_local_port() {
  python - <<'PY'
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

ensure_node_alive() {
  local pid="$1"
  local label="$2"
  local log_file="$3"

  if kill -0 "$pid" 2>/dev/null; then
    return 0
  fi

  echo "${label} exited unexpectedly" >&2
  tail -n 120 "$log_file" >&2 || true
  exit 1
}

rpc_request() {
  local url="$1"
  local method="$2"
  local params="${3:-[]}"

  curl -fsS \
    -H 'Content-Type: application/json' \
    -d "$(printf '{"jsonrpc":"2.0","id":1,"method":"%s","params":%s}' "$method" "$params")" \
    "$url"
}

parse_block_number() {
  python -c 'import json, sys; payload = json.load(sys.stdin); print(int(payload["result"]["number"], 16))'
}

parse_peer_count() {
  python -c 'import json, sys; payload = json.load(sys.stdin); print(int(payload["result"]["peers"]))'
}

parse_hash() {
  python -c 'import json, sys; payload = json.load(sys.stdin); print(payload["result"])'
}

wait_for_rpc() {
  local idx="$1"
  local url="${node_rpc_urls[$idx]}"
  local pid="${node_pids[$idx]}"
  local label="authority-${idx}"
  local log_file="${node_logs[$idx]}"
  local deadline=$((SECONDS + rpc_ready_timeout))

  while (( SECONDS < deadline )); do
    ensure_node_alive "$pid" "$label" "$log_file"
    if rpc_request "$url" "system_health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "timed out waiting ${rpc_ready_timeout}s for ${label} RPC at ${url}" >&2
  tail -n 120 "$log_file" >&2 || true
  exit 1
}

wait_for_shared_finality() {
  local timeout_secs="$1"
  local target_finalized_block="$2"
  shift 2
  local active_indices=("$@")
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    local all_ready=1
    local expected_hash=""
    shared_best_blocks=()
    shared_finalized_blocks=()
    shared_peer_counts=()
    shared_finalized_hash=""

    for idx in "${active_indices[@]}"; do
      local pid="${node_pids[$idx]}"
      local label="authority-${idx}"
      local log_file="${node_logs[$idx]}"
      local url="${node_rpc_urls[$idx]}"

      ensure_node_alive "$pid" "$label" "$log_file"

      local best_header finalized_head finalized_header health
      best_header="$(rpc_request "$url" "chain_getHeader" 2>/dev/null || true)"
      finalized_head="$(rpc_request "$url" "chain_getFinalizedHead" 2>/dev/null || true)"
      health="$(rpc_request "$url" "system_health" 2>/dev/null || true)"
      if [[ -z "$best_header" || -z "$finalized_head" || -z "$health" ]]; then
        all_ready=0
        break
      fi

      local finalized_hash best_block finalized_block peer_count
      finalized_hash="$(printf '%s' "$finalized_head" | parse_hash 2>/dev/null || true)"
      if [[ -z "$finalized_hash" ]]; then
        all_ready=0
        break
      fi

      finalized_header="$(rpc_request "$url" "chain_getHeader" "[\"$finalized_hash\"]" 2>/dev/null || true)"
      if [[ -z "$finalized_header" ]]; then
        all_ready=0
        break
      fi

      best_block="$(printf '%s' "$best_header" | parse_block_number 2>/dev/null || true)"
      finalized_block="$(printf '%s' "$finalized_header" | parse_block_number 2>/dev/null || true)"
      peer_count="$(printf '%s' "$health" | parse_peer_count 2>/dev/null || true)"
      if [[ -z "$best_block" || -z "$finalized_block" || -z "$peer_count" ]]; then
        all_ready=0
        break
      fi

      shared_best_blocks[$idx]="$best_block"
      shared_finalized_blocks[$idx]="$finalized_block"
      shared_peer_counts[$idx]="$peer_count"

      if (( finalized_block < target_finalized_block )); then
        all_ready=0
      fi

      if [[ -z "$expected_hash" ]]; then
        expected_hash="$finalized_hash"
      elif [[ "$expected_hash" != "$finalized_hash" ]]; then
        all_ready=0
      fi
    done

    if (( all_ready )) && [[ -n "$expected_hash" ]]; then
      shared_finalized_hash="$expected_hash"
      return 0
    fi

    sleep 1
  done

  echo "timed out waiting ${timeout_secs}s for shared finality at block ${target_finalized_block}" >&2
  for idx in "${active_indices[@]}"; do
    echo "--- authority-${idx} log ---" >&2
    tail -n 120 "${node_logs[$idx]}" >&2 || true
  done
  exit 1
}

wait_for_shared_block_hash() {
  local timeout_secs="$1"
  local target_block="$2"
  shift 2
  local active_indices=("$@")
  local deadline=$((SECONDS + timeout_secs))

  while (( SECONDS < deadline )); do
    local all_ready=1
    local expected_hash=""
    shared_best_blocks=()
    shared_peer_counts=()
    shared_block_hash=""

    for idx in "${active_indices[@]}"; do
      local pid="${node_pids[$idx]}"
      local label="authority-${idx}"
      local log_file="${node_logs[$idx]}"
      local url="${node_rpc_urls[$idx]}"

      ensure_node_alive "$pid" "$label" "$log_file"

      local best_header target_block_hash health
      best_header="$(rpc_request "$url" "chain_getHeader" 2>/dev/null || true)"
      target_block_hash="$(rpc_request "$url" "chain_getBlockHash" "[${target_block}]" 2>/dev/null || true)"
      health="$(rpc_request "$url" "system_health" 2>/dev/null || true)"
      if [[ -z "$best_header" || -z "$target_block_hash" || -z "$health" ]]; then
        all_ready=0
        break
      fi

      local best_block peer_count block_hash
      best_block="$(printf '%s' "$best_header" | parse_block_number 2>/dev/null || true)"
      peer_count="$(printf '%s' "$health" | parse_peer_count 2>/dev/null || true)"
      block_hash="$(printf '%s' "$target_block_hash" | parse_hash 2>/dev/null || true)"
      if [[ -z "$best_block" || -z "$peer_count" || -z "$block_hash" || "$block_hash" == "null" ]]; then
        all_ready=0
        break
      fi

      shared_best_blocks[$idx]="$best_block"
      shared_peer_counts[$idx]="$peer_count"

      if (( best_block < target_block )); then
        all_ready=0
      fi

      if [[ -z "$expected_hash" ]]; then
        expected_hash="$block_hash"
      elif [[ "$expected_hash" != "$block_hash" ]]; then
        all_ready=0
      fi
    done

    if (( all_ready )) && [[ -n "$expected_hash" ]]; then
      shared_block_hash="$expected_hash"
      return 0
    fi

    sleep 1
  done

  echo "timed out waiting ${timeout_secs}s for shared block hash at block ${target_block}" >&2
  for idx in "${active_indices[@]}"; do
    echo "--- authority-${idx} log ---" >&2
    tail -n 120 "${node_logs[$idx]}" >&2 || true
  done
  exit 1
}

start_authority() {
  local idx="$1"
  shift

  local cmd=(
    "$node_bin"
    --chain devnet
    --base-path "${node_base_dirs[$idx]}"
    --node-key-file "${node_key_files[$idx]}"
    --validator
    --force-authoring
    --name "myosu-cross-node-emission-authority-${idx}"
    --rpc-port "${node_rpc_ports[$idx]}"
    --port "${node_p2p_ports[$idx]}"
    --prometheus-port "${node_prometheus_ports[$idx]}"
    --allow-private-ip
  )

  if [[ -n "$network_backend" ]]; then
    cmd+=(--network-backend "$network_backend")
  fi

  while [[ $# -gt 0 ]]; do
    cmd+=(--bootnodes "$1")
    shift
  done

  env MYOSU_NODE_AUTHORITY_SURI="//myosu//devnet//authority-${idx}" "${cmd[@]}" \
    >"${node_logs[$idx]}" 2>&1 &
  node_pids[$idx]="$!"
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

trap cleanup EXIT

mkdir -p "$work_parent"
mkdir -p "$driver_root"
mkdir -p "$driver_examples_dir"
work_root="$(mktemp -d "$work_parent/cross-node-emission.XXXXXX")"
driver_example_name="cross_node_emission_driver_$$"
driver_source="$driver_examples_dir/${driver_example_name}.rs"

if [[ ! -f "$runtime_wasm" || ! -x "$node_bin" ]]; then
  if ! rustup target list --installed | grep -qx 'wasm32v1-none'; then
    echo "missing Rust target wasm32v1-none; run: rustup target add wasm32v1-none" >&2
    exit 1
  fi

  if [[ ! -f "$runtime_wasm" ]]; then
    echo "building myosu-chain runtime wasm cache"
    run_logged "build_runtime" env -u SKIP_WASM_BUILD cargo build -p myosu-chain-runtime --quiet
  else
    echo "reusing existing myosu-chain runtime wasm cache"
  fi

  if [[ ! -x "$node_bin" ]]; then
    echo "building myosu-chain node (fast-runtime)"
    run_logged "build_node" env SKIP_WASM_BUILD=1 cargo build -p myosu-chain --features fast-runtime --quiet
  else
    echo "reusing existing myosu-chain node binary"
  fi
else
  echo "reusing existing myosu-chain runtime wasm cache and node binary"
fi

cat >"$driver_source" <<'EOF'
use std::env;
use std::error::Error;
use std::time::Duration;

use codec::{Decode, DecodeAll, Encode};
use jsonrpsee::rpc_params;
use myosu_chain_client::ChainClient;
use serde::Deserialize;
use sp_core::{H256, blake2_128, twox_128};
use sp_runtime::AccountId32;
use subtensor_runtime_common::{AlphaCurrency, NetUid, NetUidStorageIndex, TaoCurrency};

const OWNER_KEY: &str = "//myosu//devnet//subnet-owner";
const MINER_KEY: &str = "//myosu//devnet//miner-1";
const VALIDATOR_KEY: &str = "//myosu//devnet//validator-1";
const VALIDATOR_STAKE: u64 = 100_000_000_000_000;
const CHAIN_ACTION_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Debug, Clone, Deserialize)]
struct ChainHeader {
    number: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeSnapshot {
    total_issuance: u64,
    total_stake: u64,
    subnets: Vec<SubnetSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubnetSnapshot {
    netuid: u16,
    pending_server: u64,
    pending_validator: u64,
    pending_root: u64,
    pending_owner_cut: u64,
    incentives: Vec<u16>,
    dividends: Vec<u16>,
    emissions: Vec<u64>,
    members: Vec<(u16, String, u64)>,
}

fn blocks_until_next_epoch(netuid: u16, tempo: u16, block_number: u64) -> u64 {
    if tempo == 0 {
        return u64::MAX;
    }
    let netuid_plus_one = u64::from(netuid).saturating_add(1);
    let tempo_plus_one = u64::from(tempo).saturating_add(1);
    let adjusted_block = block_number.wrapping_add(netuid_plus_one);
    let remainder = adjusted_block % tempo_plus_one;
    u64::from(tempo).saturating_sub(remainder)
}

fn h256_hex(hash: &H256) -> String {
    format!("0x{}", hex::encode(hash.as_bytes()))
}

fn parse_h256(hex_value: &str) -> Result<H256, Box<dyn Error>> {
    let trimmed = hex_value.trim_start_matches("0x");
    let bytes = hex::decode(trimmed)?;
    if bytes.len() != 32 {
        return Err(format!("expected 32-byte hash, got {} bytes", bytes.len()).into());
    }
    Ok(H256::from_slice(&bytes))
}

fn parse_block_number(hex_value: &str) -> Result<u64, Box<dyn Error>> {
    Ok(u64::from_str_radix(hex_value.trim_start_matches("0x"), 16)?)
}

fn storage_prefix(pallet: &str, storage: &str) -> Vec<u8> {
    let mut key = Vec::new();
    key.extend_from_slice(&twox_128(pallet.as_bytes()));
    key.extend_from_slice(&twox_128(storage.as_bytes()));
    key
}

fn map_identity_storage_key<K: Encode>(pallet: &str, storage: &str, key1: &K) -> Vec<u8> {
    let mut key = storage_prefix(pallet, storage);
    key.extend_from_slice(&key1.encode());
    key
}

fn double_map_identity_identity_storage_key<K1: Encode, K2: Encode>(
    pallet: &str,
    storage: &str,
    key1: &K1,
    key2: &K2,
) -> Vec<u8> {
    let mut key = storage_prefix(pallet, storage);
    key.extend_from_slice(&key1.encode());
    key.extend_from_slice(&key2.encode());
    key
}

fn double_map_blake2_identity_storage_key<K1: Encode, K2: Encode>(
    pallet: &str,
    storage: &str,
    key1: &K1,
    key2: &K2,
) -> Vec<u8> {
    let mut key = storage_prefix(pallet, storage);
    let key1_bytes = key1.encode();
    key.extend_from_slice(&blake2_128(&key1_bytes));
    key.extend_from_slice(&key1_bytes);
    key.extend_from_slice(&key2.encode());
    key
}

async fn storage_at<T: Decode>(
    client: &ChainClient,
    key: &[u8],
    block_hash: H256,
) -> Result<Option<T>, Box<dyn Error>> {
    let key_hex = format!("0x{}", hex::encode(key));
    let value: Option<String> = client
        .request("state_getStorage", rpc_params![key_hex, h256_hex(&block_hash)])
        .await?;
    let Some(value) = value else {
        return Ok(None);
    };
    let bytes = hex::decode(value.trim_start_matches("0x"))?;
    let decoded = T::decode_all(&mut bytes.as_slice())?;
    Ok(Some(decoded))
}

async fn finalized_block_number(
    client: &ChainClient,
    finalized_hash: H256,
) -> Result<u64, Box<dyn Error>> {
    let header: ChainHeader = client
        .request("chain_getHeader", rpc_params![h256_hex(&finalized_hash)])
        .await?;
    parse_block_number(&header.number)
}

async fn snapshot_subnet(
    client: &ChainClient,
    netuid: NetUid,
    finalized_hash: H256,
) -> Result<SubnetSnapshot, Box<dyn Error>> {
    let pending_server = storage_at::<AlphaCurrency>(
        client,
        &map_identity_storage_key("GameSolver", "PendingServerEmission", &netuid),
        finalized_hash,
    )
    .await?
    .map(u64::from)
    .unwrap_or_default();
    let pending_validator = storage_at::<AlphaCurrency>(
        client,
        &map_identity_storage_key("GameSolver", "PendingValidatorEmission", &netuid),
        finalized_hash,
    )
    .await?
    .map(u64::from)
    .unwrap_or_default();
    let pending_root = storage_at::<AlphaCurrency>(
        client,
        &map_identity_storage_key("GameSolver", "PendingRootAlphaDivs", &netuid),
        finalized_hash,
    )
    .await?
    .map(u64::from)
    .unwrap_or_default();
    let pending_owner_cut = storage_at::<AlphaCurrency>(
        client,
        &map_identity_storage_key("GameSolver", "PendingOwnerCut", &netuid),
        finalized_hash,
    )
    .await?
    .map(u64::from)
    .unwrap_or_default();

    let incentives = storage_at::<Vec<u16>>(
        client,
        &map_identity_storage_key(
            "GameSolver",
            "Incentive",
            &NetUidStorageIndex::from(netuid),
        ),
        finalized_hash,
    )
    .await?
    .unwrap_or_default();
    let dividends = storage_at::<Vec<u16>>(
        client,
        &map_identity_storage_key("GameSolver", "Dividends", &netuid),
        finalized_hash,
    )
    .await?
    .unwrap_or_default();
    let emissions = storage_at::<Vec<AlphaCurrency>>(
        client,
        &map_identity_storage_key("GameSolver", "Emission", &netuid),
        finalized_hash,
    )
    .await?
    .unwrap_or_default()
    .into_iter()
    .map(u64::from)
    .collect::<Vec<_>>();

    let member_count = storage_at::<u16>(
        client,
        &map_identity_storage_key("GameSolver", "SubnetworkN", &netuid),
        finalized_hash,
    )
    .await?
    .unwrap_or_default();
    let mut members = Vec::new();
    for uid in 0..member_count {
        let hotkey = storage_at::<AccountId32>(
            client,
            &double_map_identity_identity_storage_key("GameSolver", "Keys", &netuid, &uid),
            finalized_hash,
        )
        .await?
        .ok_or_else(|| format!("missing hotkey for subnet {} uid {}", u16::from(netuid), uid))?;
        let stake = storage_at::<AlphaCurrency>(
            client,
            &double_map_blake2_identity_storage_key(
                "GameSolver",
                "TotalHotkeyAlpha",
                &hotkey,
                &netuid,
            ),
            finalized_hash,
        )
        .await?
        .map(u64::from)
        .unwrap_or_default();
        members.push((uid, hotkey.to_string(), stake));
    }

    Ok(SubnetSnapshot {
        netuid: u16::from(netuid),
        pending_server,
        pending_validator,
        pending_root,
        pending_owner_cut,
        incentives,
        dividends,
        emissions,
        members,
    })
}

async fn collect_snapshot(
    client: &ChainClient,
    subnets: &[NetUid],
    finalized_hash: H256,
) -> Result<NodeSnapshot, Box<dyn Error>> {
    let total_issuance = storage_at::<TaoCurrency>(
        client,
        &storage_prefix("GameSolver", "TotalIssuance"),
        finalized_hash,
    )
    .await?
    .map(u64::from)
    .unwrap_or_default();
    let total_stake = storage_at::<TaoCurrency>(
        client,
        &storage_prefix("GameSolver", "TotalStake"),
        finalized_hash,
    )
    .await?
    .map(u64::from)
    .unwrap_or_default();

    let mut subnet_snapshots = Vec::new();
    for netuid in subnets {
        subnet_snapshots.push(snapshot_subnet(client, *netuid, finalized_hash).await?);
    }

    Ok(NodeSnapshot {
        total_issuance,
        total_stake,
        subnets: subnet_snapshots,
    })
}

fn explain_snapshot_mismatch(
    baseline: &NodeSnapshot,
    candidate: &NodeSnapshot,
    node_label: &str,
) -> String {
    if baseline.total_issuance != candidate.total_issuance {
        return format!(
            "{node_label} total issuance mismatch: baseline={} candidate={}",
            baseline.total_issuance, candidate.total_issuance
        );
    }
    if baseline.total_stake != candidate.total_stake {
        return format!(
            "{node_label} total stake mismatch: baseline={} candidate={}",
            baseline.total_stake, candidate.total_stake
        );
    }
    if baseline.subnets.len() != candidate.subnets.len() {
        return format!(
            "{node_label} subnet count mismatch: baseline={} candidate={}",
            baseline.subnets.len(),
            candidate.subnets.len()
        );
    }

    for (baseline_subnet, candidate_subnet) in baseline.subnets.iter().zip(candidate.subnets.iter()) {
        if baseline_subnet != candidate_subnet {
            return format!(
                "{node_label} subnet {} mismatch: baseline={baseline_subnet:?} candidate={candidate_subnet:?}",
                baseline_subnet.netuid
            );
        }
    }

    format!("{node_label} snapshot mismatch: baseline={baseline:?} candidate={candidate:?}")
}

async fn run_setup() -> Result<(), Box<dyn Error>> {
    let client = ChainClient::connect(&env::var("MYOSU_E2E_WS_1")?).await?;
    let owner_hotkey = ChainClient::account_id_from_uri(OWNER_KEY)?;

    let subnet_report = client.register_network(OWNER_KEY, CHAIN_ACTION_TIMEOUT).await?;
    let netuid = subnet_report.subnet;
    let netuid_u16 = u16::from(netuid);
    let owner_uid = client
        .get_uid_for_net_and_hotkey(netuid, &owner_hotkey)
        .await?
        .ok_or("missing subnet owner uid after register_network")?;

    client
        .ensure_subtoken_enabled(OWNER_KEY, netuid, CHAIN_ACTION_TIMEOUT)
        .await?;
    let miner_registration = client
        .ensure_burned_registration(MINER_KEY, netuid, CHAIN_ACTION_TIMEOUT)
        .await?;
    let validator_registration = client
        .ensure_burned_registration(VALIDATOR_KEY, netuid, CHAIN_ACTION_TIMEOUT)
        .await?;
    client
        .ensure_stake_added(
            VALIDATOR_KEY,
            netuid,
            VALIDATOR_STAKE,
            CHAIN_ACTION_TIMEOUT,
        )
        .await?;
    let validator_uid = client
        .wait_for_validator_permit(netuid, &validator_registration.hotkey, CHAIN_ACTION_TIMEOUT)
        .await?;
    client
        .ensure_weights_set(VALIDATOR_KEY, netuid, MINER_KEY, CHAIN_ACTION_TIMEOUT)
        .await?;

    let tempo = client.get_subnet_tempo(netuid).await?;
    let current_block_after_weights = client.best_block_number().await?;
    let blocks_until_epoch = blocks_until_next_epoch(netuid_u16, tempo, current_block_after_weights);
    if blocks_until_epoch == u64::MAX {
        return Err("tempo must be non-zero for cross-node emission proof".into());
    }
    let target_epoch_block = if blocks_until_epoch == 0 {
        current_block_after_weights.saturating_add(u64::from(tempo).saturating_add(1))
    } else {
        current_block_after_weights.saturating_add(blocks_until_epoch)
    };
    let target_snapshot_block =
        target_epoch_block.saturating_add(u64::from(tempo).saturating_add(1).saturating_mul(2));

    println!("subnet={netuid_u16}");
    println!("tempo={tempo}");
    println!("owner_uid={owner_uid}");
    println!("miner_uid={}", miner_registration.uid);
    println!("validator_uid={validator_uid}");
    println!("current_block_after_weights={current_block_after_weights}");
    println!("target_epoch_block={target_epoch_block}");
    println!("target_snapshot_block={target_snapshot_block}");

    Ok(())
}

async fn run_compare() -> Result<(), Box<dyn Error>> {
    let snapshot_hash = parse_h256(&env::var("MYOSU_E2E_SNAPSHOT_HASH")?)?;
    let target_subnet = NetUid::from(env::var("MYOSU_E2E_TARGET_SUBNET")?.parse::<u16>()?);
    let target_snapshot_block = env::var("MYOSU_E2E_TARGET_SNAPSHOT_BLOCK")?.parse::<u64>()?;

    let endpoints = [
        env::var("MYOSU_E2E_WS_1")?,
        env::var("MYOSU_E2E_WS_2")?,
        env::var("MYOSU_E2E_WS_3")?,
        env::var("MYOSU_E2E_WS_4")?,
    ];
    let mut clients = Vec::new();
    for endpoint in &endpoints {
        clients.push(ChainClient::connect(endpoint).await?);
    }

    for (index, client) in clients.iter().enumerate() {
        let snapshot_block = finalized_block_number(client, snapshot_hash).await?;
        if snapshot_block != target_snapshot_block {
            return Err(format!(
                "node {} reports block {} for snapshot hash instead of expected block {}",
                index + 1,
                snapshot_block,
                target_snapshot_block
            )
            .into());
        }
    }

    let mut subnets = clients[0].get_existing_subnets().await?;
    subnets.sort_unstable();
    if !subnets.contains(&target_subnet) {
        return Err(format!("target subnet {} missing from live subnet set", u16::from(target_subnet)).into());
    }

    let baseline = collect_snapshot(&clients[0], &subnets, snapshot_hash).await?;
    let target_snapshot = baseline
        .subnets
        .iter()
        .find(|subnet| subnet.netuid == u16::from(target_subnet))
        .ok_or("target subnet snapshot missing from baseline")?;
    let target_emission_sum = target_snapshot.emissions.iter().copied().sum::<u64>();
    let target_member_stake_sum = target_snapshot
        .members
        .iter()
        .map(|(_, _, stake)| *stake)
        .sum::<u64>();

    if target_emission_sum == 0 {
        return Err(format!("target subnet {} has zero emission sum at finalized snapshot", target_snapshot.netuid).into());
    }
    if target_member_stake_sum == 0 {
        return Err(format!("target subnet {} has zero aggregate member stake at finalized snapshot", target_snapshot.netuid).into());
    }

    for (index, client) in clients.iter().enumerate().skip(1) {
        let snapshot = collect_snapshot(client, &subnets, snapshot_hash).await?;
        if snapshot != baseline {
            return Err(explain_snapshot_mismatch(
                &baseline,
                &snapshot,
                &format!("node {}", index + 1),
            )
            .into());
        }
    }

    println!("snapshot_hash={}", h256_hex(&snapshot_hash));
    println!("snapshot_block={target_snapshot_block}");
    println!("subnets_compared={}", subnets.len());
    println!("target_subnet={}", u16::from(target_subnet));
    println!("target_subnet_emission_sum={target_emission_sum}");
    println!("target_subnet_member_stake_sum={target_member_stake_sum}");
    println!("target_subnet_member_count={}", target_snapshot.members.len());

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    match env::var("MYOSU_E2E_MODE").unwrap_or_else(|_| "setup".to_string()).as_str() {
        "setup" => run_setup().await,
        "compare" => run_compare().await,
        other => Err(format!("unsupported MYOSU_E2E_MODE `{other}`").into()),
    }
}
EOF

for idx in 1 2 3 4; do
  node_base_dirs[$idx]="$work_root/node${idx}"
  node_key_files[$idx]="${node_base_dirs[$idx]}/node-key"
  node_logs[$idx]="$work_root/node${idx}.log"
  mkdir -p "${node_base_dirs[$idx]}"

  "$node_bin" key generate-node-key --file "${node_key_files[$idx]}" >/dev/null
  local_peer_id="$("$node_bin" key inspect-node-key --file "${node_key_files[$idx]}")"
  node_p2p_ports[$idx]="$(select_local_port)"
  node_rpc_ports[$idx]="$(select_local_port)"
  node_prometheus_ports[$idx]="$(select_local_port)"
  node_rpc_urls[$idx]="http://127.0.0.1:${node_rpc_ports[$idx]}"
  node_ws_urls[$idx]="ws://127.0.0.1:${node_rpc_ports[$idx]}"
  node_multiaddrs[$idx]="/ip4/127.0.0.1/tcp/${node_p2p_ports[$idx]}/p2p/${local_peer_id}"
done

echo "starting devnet authority 1"
start_authority 1
wait_for_rpc 1

echo "starting devnet authority 2"
start_authority 2 "${node_multiaddrs[1]}"
wait_for_rpc 2

echo "starting devnet authority 3"
start_authority 3 "${node_multiaddrs[1]}" "${node_multiaddrs[2]}"
wait_for_rpc 3

echo "starting devnet authority 4"
start_authority 4 "${node_multiaddrs[1]}" "${node_multiaddrs[2]}" "${node_multiaddrs[3]}"
wait_for_rpc 4

wait_for_shared_finality "$initial_finality_timeout" "$initial_finalized_target" 1 2 3 4

setup_output="$(
  MYOSU_E2E_MODE=setup \
  MYOSU_E2E_WS_1="${node_ws_urls[1]}" \
  SKIP_WASM_BUILD=1 \
  CARGO_TARGET_DIR="$driver_target_dir" \
  cargo run --quiet -p myosu-chain-client --example "$driver_example_name"
)"

target_subnet="$(require_kv "$setup_output" "subnet")"
tempo="$(require_kv "$setup_output" "tempo")"
target_epoch_block="$(require_kv "$setup_output" "target_epoch_block")"
target_snapshot_block="$(require_kv "$setup_output" "target_snapshot_block")"

wait_for_shared_block_hash "$epoch_finality_timeout" "$target_snapshot_block" 1 2 3 4

compare_output="$(
  MYOSU_E2E_MODE=compare \
  MYOSU_E2E_SNAPSHOT_HASH="$shared_block_hash" \
  MYOSU_E2E_TARGET_SUBNET="$target_subnet" \
  MYOSU_E2E_TARGET_SNAPSHOT_BLOCK="$target_snapshot_block" \
  MYOSU_E2E_WS_1="${node_ws_urls[1]}" \
  MYOSU_E2E_WS_2="${node_ws_urls[2]}" \
  MYOSU_E2E_WS_3="${node_ws_urls[3]}" \
  MYOSU_E2E_WS_4="${node_ws_urls[4]}" \
  SKIP_WASM_BUILD=1 \
  CARGO_TARGET_DIR="$driver_target_dir" \
  cargo run --quiet -p myosu-chain-client --example "$driver_example_name"
)"

echo "CROSS_NODE_EMISSION ok"
echo "tempo=${tempo}"
echo "target_epoch_block=${target_epoch_block}"
printf 'shared_snapshot_hash=%s\n' "$shared_block_hash"
printf 'shared_snapshot_block=%s\n' "$target_snapshot_block"
printf '%s\n' "$setup_output"
printf '%s\n' "$compare_output"
