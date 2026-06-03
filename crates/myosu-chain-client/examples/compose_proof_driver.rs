// Stage0 multi-validator compose proof driver.
//
// Used by `tests/e2e/compose_proof.sh` after the stage0 docker-compose stack
// boots `chain`, `miner`, `validator`, and `validator-2`. Reads the on-chain
// `Weights` rows each validator submitted for `//myosu//devnet//miner-1` and
// asserts they agree within the INV-003 epsilon window — that is the
// fail-closed assertion the IMPLEMENTATION_PLAN P0 row requires in the
// compose proof's exit check.
//
// Usage:
//   cargo run -p myosu-chain-client --example compose_proof_driver -- \
//     <ws-endpoint> <subnet> <miner-uri> <validator-a-uri> <validator-b-uri> <epsilon>
//
// Output key/value lines (consumed by the bash proof):
//   miner_uid=<u16>
//   validator_a_uid=<u16>
//   validator_b_uid=<u16>
//   validator_a_weights=<"[(u16,u16); N]">
//   validator_b_weights=<"[(u16,u16); N]">
//   validator_a_target_weight=<u16>      // 0 if miner not in the row
//   validator_b_target_weight=<u16>      // 0 if miner not in the row
//   agreement_within_epsilon=<true|false>

use std::env;
use std::error::Error;
use std::process::ExitCode;
use std::time::Duration;

use myosu_chain_client::ChainClient;
use myosu_chain_client::evaluate_validator_agreement;
use myosu_chain_client::target_weight_in_row;
use subtensor_runtime_common::NetUid;

const ACTION_TIMEOUT: Duration = Duration::from_secs(180);
const DEFAULT_EPSILON: f64 = 1.0e-6;

fn require_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing required argument: {name}"))
}

fn usage() -> String {
    [
        "usage: cargo run -p myosu-chain-client --example compose_proof_driver -- \\",
        "  <ws-endpoint> <subnet> <miner-uri> <validator-a-uri> <validator-b-uri> [epsilon]",
    ]
    .join("\n")
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("compose_proof_driver failed: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let endpoint = require_arg(&mut args, "ws-endpoint")?;
    let subnet: u16 = require_arg(&mut args, "subnet")?
        .parse()
        .map_err(|err| format!("subnet must be u16: {err}"))?;
    let miner_uri = require_arg(&mut args, "miner-uri")?;
    let validator_a_uri = require_arg(&mut args, "validator-a-uri")?;
    let validator_b_uri = require_arg(&mut args, "validator-b-uri")?;
    let epsilon: f64 = match args.next() {
        Some(value) => value
            .parse()
            .map_err(|err| format!("epsilon must be f64: {err}"))?,
        None => DEFAULT_EPSILON,
    };
    if args.next().is_some() {
        return Err(usage().into());
    }
    if !epsilon.is_finite() || epsilon < 0.0 {
        return Err(format!("epsilon must be finite and non-negative: {epsilon}").into());
    }

    let netuid = NetUid::from(subnet);
    let client = ChainClient::connect(&endpoint).await?;

    let miner_hotkey = ChainClient::account_id_from_uri(&miner_uri)?;
    let validator_a_hotkey = ChainClient::account_id_from_uri(&validator_a_uri)?;
    let validator_b_hotkey = ChainClient::account_id_from_uri(&validator_b_uri)?;

    let miner_uid = client
        .get_uid_for_net_and_hotkey(netuid, &miner_hotkey)
        .await?
        .ok_or_else(|| {
            format!(
                "miner hotkey {} is not yet registered on subnet {subnet}",
                miner_hotkey
            )
        })?;
    let validator_a_uid = client
        .wait_for_validator_permit(netuid, &validator_a_hotkey, ACTION_TIMEOUT)
        .await?;
    let validator_b_uid = client
        .wait_for_validator_permit(netuid, &validator_b_hotkey, ACTION_TIMEOUT)
        .await?;

    let validator_a_weights = client.get_weights_for_uid(netuid, validator_a_uid).await?;
    let validator_b_weights = client.get_weights_for_uid(netuid, validator_b_uid).await?;

    let validator_a_target_weight = target_weight_in_row(&validator_a_weights, miner_uid);
    let validator_b_target_weight = target_weight_in_row(&validator_b_weights, miner_uid);

    // INV-003: the integer weight values are the post-quantization agreement
    // signal. u16 weights cannot be non-finite; the epsilon is the score-domain
    // tolerance floored to 0 weight units here. The shared
    // `evaluate_validator_agreement` helper is the fail-closed gate (errors
    // on `InvalidEpsilon`, `MissingTargetWeight`, and `WeightsDiverge`).
    let agreement = evaluate_validator_agreement(
        &validator_a_weights,
        &validator_b_weights,
        miner_uid,
        epsilon,
    );

    let agreement_within_epsilon = agreement
        .as_ref()
        .map(|report| report.agreement_within_epsilon)
        .unwrap_or(false);
    let agreement_within_epsilon_str = if agreement_within_epsilon {
        "true"
    } else {
        "false"
    };

    println!("miner_uid={miner_uid}");
    println!("validator_a_uid={validator_a_uid}");
    println!("validator_b_uid={validator_b_uid}");
    println!("validator_a_uri={validator_a_uri}");
    println!("validator_b_uri={validator_b_uri}");
    println!("validator_a_weights={validator_a_weights:?}");
    println!("validator_b_weights={validator_b_weights:?}");
    println!("validator_a_target_weight={validator_a_target_weight}");
    println!("validator_b_target_weight={validator_b_target_weight}");
    println!("epsilon={epsilon}");
    println!("agreement_within_epsilon={agreement_within_epsilon_str}");

    if let Err(error) = agreement {
        return Err(error.into());
    }

    Ok(())
}
