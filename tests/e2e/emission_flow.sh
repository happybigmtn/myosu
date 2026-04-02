#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
helpers_dir="$repo_root/tests/e2e/helpers"
runtime_dir="${MYOSU_E2E_DIR:-$repo_root/target/e2e/devnet}"
state_file="$runtime_dir/devnet.env"
driver_root="$repo_root/target/e2e"
driver_target_dir="${MYOSU_E2E_CARGO_TARGET_DIR:-$driver_root/emission-flow-target}"
driver_examples_dir="$repo_root/crates/myosu-chain-client/examples"
driver_source=""
driver_example_name=""
started_devnet=0

cleanup() {
  if [[ -n "$driver_source" && -f "$driver_source" ]]; then
    rm -f "$driver_source"
  fi
  if (( started_devnet )); then
    bash "$helpers_dir/stop_devnet.sh" >/dev/null 2>&1 || true
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

extract_summary_field() {
  local line="$1"
  local field="$2"
  local value
  value="$(printf '%s\n' "$line" | tr ' ' '\n' | sed -n "s/^${field}=//p" | tail -n1)"
  if [[ -z "$value" ]]; then
    echo "missing ${field} in block_step_summary line" >&2
    printf '%s\n' "$line" >&2
    exit 1
  fi
  printf '%s\n' "$value"
}

wait_for_summary_line() {
  local block="$1"
  local timeout="${2:-60}"
  local deadline=$((SECONDS + timeout))
  local line=""

  while (( SECONDS < deadline )); do
    if [[ ! -f "$state_file" ]]; then
      echo "missing devnet state file while waiting for block_step_summary" >&2
      exit 1
    fi

    # shellcheck disable=SC1090
    source "$state_file"

    if [[ -f "$PID_FILE" ]]; then
      local node_pid
      node_pid="$(<"$PID_FILE")"
      if ! kill -0 "$node_pid" 2>/dev/null; then
        echo "devnet exited while waiting for block_step_summary for block ${block}" >&2
        tail -n 120 "$LOG_FILE" >&2 || true
        exit 1
      fi
    fi

    line="$(
      grep "block_step_summary block=${block} " "$LOG_FILE" 2>/dev/null \
        | grep 'drained_epoch_count=1' \
        | tail -n1 \
        || true
    )"
    if [[ -n "$line" ]]; then
      printf '%s\n' "$line"
      return 0
    fi
    sleep 1
  done

  echo "timed out waiting for block_step_summary for block ${block}" >&2
  tail -n 120 "$LOG_FILE" >&2 || true
  exit 1
}

trap cleanup EXIT

mkdir -p "$driver_root"
mkdir -p "$driver_examples_dir"
driver_example_name="emission_flow_driver_$$"
driver_source="$driver_examples_dir/${driver_example_name}.rs"

cat >"$driver_source" <<'EOF'
use std::error::Error;
use std::time::Duration;
use std::time::Instant;

use myosu_chain_client::ChainClient;

const CHAIN_ENDPOINT: &str = "ws://127.0.0.1:9955";
const OWNER_KEY: &str = "//Charlie";
const MINER_KEY: &str = "//Alice";
const VALIDATOR_KEY: &str = "//Bob";
const VALIDATOR_STAKE: u64 = 100_000_000_000_000;
const CHAIN_ACTION_TIMEOUT: Duration = Duration::from_secs(180);
const POLL_INTERVAL: Duration = Duration::from_millis(500);

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

async fn wait_for_block(client: &ChainClient, target_block: u64) -> Result<u64, Box<dyn Error>> {
    let deadline = Instant::now()
        .checked_add(CHAIN_ACTION_TIMEOUT)
        .ok_or("wait_for_block timeout overflow")?;
    loop {
        let best_block = client.best_block_number().await?;
        if best_block >= target_block {
            return Ok(best_block);
        }
        if Instant::now() >= deadline {
            return Err(
                format!("timed out waiting for best block {target_block}, head is {best_block}")
                    .into(),
            );
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = ChainClient::connect(CHAIN_ENDPOINT).await?;
    let owner_hotkey = ChainClient::account_id_from_uri(OWNER_KEY)?;

    let subnet_report = client.register_network(OWNER_KEY, CHAIN_ACTION_TIMEOUT).await?;
    let netuid = subnet_report.subnet;
    let netuid_u16 = u16::from(netuid);
    let owner_uid = client
        .get_uid_for_net_and_hotkey(netuid, &owner_hotkey)
        .await?
        .ok_or("missing subnet owner uid after register_network")?;

    let commit_reveal_enabled = client.get_commit_reveal_weights_enabled(netuid).await?;
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
    let weight_submission = client
        .ensure_weights_set(VALIDATOR_KEY, netuid, MINER_KEY, CHAIN_ACTION_TIMEOUT)
        .await?;

    let tempo = client.get_subnet_tempo(netuid).await?;
    let current_block_after_weights = client.best_block_number().await?;
    let blocks_until_epoch =
        blocks_until_next_epoch(netuid_u16, tempo, current_block_after_weights);
    if blocks_until_epoch == u64::MAX {
        return Err("tempo must be non-zero for emission flow proof".into());
    }
    let target_epoch_block = if blocks_until_epoch == 0 {
        current_block_after_weights.saturating_add(u64::from(tempo).saturating_add(1))
    } else {
        current_block_after_weights.saturating_add(blocks_until_epoch)
    };

    let owner_stake_before = client.get_hotkey_subnet_stake(netuid, &owner_hotkey).await?;
    let observed_block_after_epoch = wait_for_block(&client, target_epoch_block).await?;
    let owner_stake_after = client.get_hotkey_subnet_stake(netuid, &owner_hotkey).await?;
    let owner_stake_delta = owner_stake_after.saturating_sub(owner_stake_before);

    let incentives = client.get_incentives(netuid).await?;
    let dividends = client.get_dividends(netuid).await?;
    let emissions = client.get_emissions(netuid).await?;

    let miner_incentive = incentives
        .get(usize::from(miner_registration.uid))
        .copied()
        .unwrap_or_default();
    let validator_dividend = dividends
        .get(usize::from(validator_uid))
        .copied()
        .unwrap_or_default();
    let miner_emission = emissions
        .get(usize::from(miner_registration.uid))
        .copied()
        .map(u64::from)
        .unwrap_or_default();
    let emission_sum = emissions.iter().copied().map(u64::from).sum::<u64>();

    if miner_incentive == 0 {
        return Err(format!(
            "expected positive miner incentive for uid {}, got 0",
            miner_registration.uid
        )
        .into());
    }
    if validator_dividend == 0 {
        return Err(format!(
            "expected positive validator dividend for uid {validator_uid}, got 0"
        )
        .into());
    }
    if miner_emission == 0 {
        return Err(format!(
            "expected positive miner emission for uid {}, got 0",
            miner_registration.uid
        )
        .into());
    }
    if emission_sum == 0 {
        return Err("expected positive emission sum after target epoch".into());
    }
    if owner_stake_after < owner_stake_before {
        return Err(format!(
            "owner stake regressed: before={owner_stake_before} after={owner_stake_after}"
        )
        .into());
    }

    println!("subnet={netuid_u16}");
    println!("tempo={tempo}");
    println!("owner_uid={owner_uid}");
    println!("miner_uid={}", miner_registration.uid);
    println!("validator_uid={validator_uid}");
    println!("commit_reveal_enabled={commit_reveal_enabled}");
    println!("weight_submission_mode={}", weight_submission.mode);
    println!("current_block_after_weights={current_block_after_weights}");
    println!("target_epoch_block={target_epoch_block}");
    println!("observed_block_after_epoch={observed_block_after_epoch}");
    println!("owner_stake_before={owner_stake_before}");
    println!("owner_stake_after={owner_stake_after}");
    println!("owner_stake_delta={owner_stake_delta}");
    println!("emission_sum={emission_sum}");
    println!("miner_incentive={miner_incentive}");
    println!("validator_dividend={validator_dividend}");
    println!("miner_emission={miner_emission}");

    Ok(())
}
EOF

echo "EMISSION_FLOW starting devnet"
bash "$helpers_dir/start_devnet.sh"
started_devnet=1
MYOSU_E2E_WAIT_TIMEOUT="${MYOSU_E2E_WAIT_TIMEOUT:-120}" \
  bash "$helpers_dir/wait_for_block.sh" 1

driver_output="$(
  SKIP_WASM_BUILD=1 \
  CARGO_TARGET_DIR="$driver_target_dir" \
  cargo run --quiet -p myosu-chain-client --example "$driver_example_name"
)"

if [[ ! -f "$state_file" ]]; then
  echo "missing devnet state file after driver run" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "$state_file"

subnet="$(require_kv "$driver_output" "subnet")"
tempo="$(require_kv "$driver_output" "tempo")"
owner_uid="$(require_kv "$driver_output" "owner_uid")"
miner_uid="$(require_kv "$driver_output" "miner_uid")"
validator_uid="$(require_kv "$driver_output" "validator_uid")"
commit_reveal_enabled="$(require_kv "$driver_output" "commit_reveal_enabled")"
weight_submission_mode="$(require_kv "$driver_output" "weight_submission_mode")"
current_block_after_weights="$(require_kv "$driver_output" "current_block_after_weights")"
target_epoch_block="$(require_kv "$driver_output" "target_epoch_block")"
observed_block_after_epoch="$(require_kv "$driver_output" "observed_block_after_epoch")"
owner_stake_before="$(require_kv "$driver_output" "owner_stake_before")"
owner_stake_after="$(require_kv "$driver_output" "owner_stake_after")"
owner_stake_delta="$(require_kv "$driver_output" "owner_stake_delta")"
emission_sum="$(require_kv "$driver_output" "emission_sum")"
miner_incentive="$(require_kv "$driver_output" "miner_incentive")"
validator_dividend="$(require_kv "$driver_output" "validator_dividend")"
miner_emission="$(require_kv "$driver_output" "miner_emission")"

summary_line="$(wait_for_summary_line "$target_epoch_block" 60)"
drained_epoch_count="$(extract_summary_field "$summary_line" "drained_epoch_count")"
block_emission="$(extract_summary_field "$summary_line" "block_emission")"
server_alpha_distributed="$(extract_summary_field "$summary_line" "server_alpha_distributed")"
validator_alpha_distributed="$(extract_summary_field "$summary_line" "validator_alpha_distributed")"
root_alpha_distributed="$(extract_summary_field "$summary_line" "root_alpha_distributed")"
owner_cut_distributed="$(extract_summary_field "$summary_line" "owner_cut_distributed")"

if [[ "$drained_epoch_count" != "1" ]]; then
  echo "expected drained_epoch_count=1, got ${drained_epoch_count}" >&2
  exit 1
fi

if (( root_alpha_distributed != 0 )); then
  echo "expected stage-0 root alpha distribution to remain zero, got ${root_alpha_distributed}" >&2
  exit 1
fi

expected_emission_sum=$((server_alpha_distributed + validator_alpha_distributed))
actual_total_distribution=$((expected_emission_sum + owner_cut_distributed))
accrual_blocks=$((tempo + 1))
expected_epoch_distribution=$((block_emission * accrual_blocks))
rounding_tolerance=$((accrual_blocks * 8))

if (( emission_sum != expected_emission_sum )); then
  echo "emission sum mismatch: storage=${emission_sum} log=${expected_emission_sum}" >&2
  printf '%s\n' "$summary_line" >&2
  exit 1
fi

if (( owner_stake_delta != owner_cut_distributed )); then
  echo "owner cut mismatch: owner stake delta=${owner_stake_delta} log=${owner_cut_distributed}" >&2
  printf '%s\n' "$summary_line" >&2
  exit 1
fi

if (( actual_total_distribution > expected_epoch_distribution )); then
  echo "distribution total exceeded accrued epoch emission: actual=${actual_total_distribution} expected=${expected_epoch_distribution}" >&2
  printf '%s\n' "$summary_line" >&2
  exit 1
fi

distribution_rounding_loss=$((expected_epoch_distribution - actual_total_distribution))
if (( distribution_rounding_loss > rounding_tolerance )); then
  echo "distribution total mismatch: actual=${actual_total_distribution} expected=${expected_epoch_distribution} rounding_loss=${distribution_rounding_loss} tolerance=${rounding_tolerance}" >&2
  printf '%s\n' "$summary_line" >&2
  exit 1
fi

if (( miner_incentive <= 0 )); then
  echo "expected positive miner incentive, got ${miner_incentive}" >&2
  exit 1
fi

if (( validator_dividend <= 0 )); then
  echo "expected positive validator dividend, got ${validator_dividend}" >&2
  exit 1
fi

if (( miner_emission <= 0 )); then
  echo "expected positive miner emission, got ${miner_emission}" >&2
  exit 1
fi

echo "EMISSION_FLOW ok"
echo "subnet=${subnet}"
echo "tempo=${tempo}"
echo "owner_uid=${owner_uid}"
echo "miner_uid=${miner_uid}"
echo "validator_uid=${validator_uid}"
echo "commit_reveal_enabled=${commit_reveal_enabled}"
echo "weight_submission_mode=${weight_submission_mode}"
echo "current_block_after_weights=${current_block_after_weights}"
echo "target_epoch_block=${target_epoch_block}"
echo "observed_block_after_epoch=${observed_block_after_epoch}"
echo "block_emission=${block_emission}"
echo "accrual_blocks=${accrual_blocks}"
echo "server_alpha_distributed=${server_alpha_distributed}"
echo "validator_alpha_distributed=${validator_alpha_distributed}"
echo "owner_cut_distributed=${owner_cut_distributed}"
echo "actual_total_distribution=${actual_total_distribution}"
echo "expected_epoch_distribution=${expected_epoch_distribution}"
echo "distribution_rounding_loss=${distribution_rounding_loss}"
echo "emission_sum=${emission_sum}"
echo "owner_stake_before=${owner_stake_before}"
echo "owner_stake_after=${owner_stake_after}"
echo "owner_stake_delta=${owner_stake_delta}"
echo "miner_incentive=${miner_incentive}"
echo "validator_dividend=${validator_dividend}"
echo "miner_emission=${miner_emission}"
