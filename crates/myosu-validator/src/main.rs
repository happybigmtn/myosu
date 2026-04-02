use clap::Parser;
use myosu_validator::chain::ensure_registered;
use myosu_validator::chain::ensure_subtoken_enabled;
use myosu_validator::chain::ensure_validator_permit_ready;
use myosu_validator::chain::ensure_weights_set;
use myosu_validator::chain::probe_chain;
use myosu_validator::cli::Cli;
use myosu_validator::permit_bootstrap_report;
use myosu_validator::registration_report;
use myosu_validator::startup_report;
use myosu_validator::subtoken_bootstrap_report;
use myosu_validator::validation::ValidatorBootstrapError;
use myosu_validator::validation::score_response;
use myosu_validator::validation::validation_plan_from_cli;
use myosu_validator::validation_report;
use myosu_validator::weight_submission_report;
use std::time::Instant;
use subtensor_runtime_common::NetUid;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), ValidatorBootstrapError> {
    init_tracing();

    let cli = Cli::parse();
    let key_uri = cli.resolve_key_uri()?;
    info!(
        chain = %cli.chain,
        subnet = cli.subnet,
        key_source = cli.key_source_label(),
        "probing chain for validator bootstrap"
    );

    let report = probe_chain(&cli.chain, NetUid::from(cli.subnet)).await?;
    print!("{}", startup_report(&report));

    if cli.register {
        let report = ensure_registered(&cli.chain, &key_uri, NetUid::from(cli.subnet)).await?;
        print!("{}", registration_report(&report));
    }

    if cli.enable_subtoken {
        let report =
            ensure_subtoken_enabled(&cli.chain, &key_uri, NetUid::from(cli.subnet)).await?;
        print!("{}", subtoken_bootstrap_report(&report));
    }

    if let Some(stake_amount) = cli.stake_amount {
        let report = ensure_validator_permit_ready(
            &cli.chain,
            &key_uri,
            NetUid::from(cli.subnet),
            stake_amount,
        )
        .await?;
        print!("{}", permit_bootstrap_report(&report));
    }

    let validation_plan = validation_plan_from_cli(&cli)?;
    if let Some(validation_plan) = validation_plan {
        let report = score_response(&validation_plan)?;
        print!("{}", validation_report(&report));
    }

    if cli.submit_weights {
        let target_hotkey = cli.weight_hotkey.as_deref().unwrap_or(&key_uri);
        let started_at = Instant::now();
        let report = ensure_weights_set(
            &cli.chain,
            &key_uri,
            NetUid::from(cli.subnet),
            target_hotkey,
        )
        .await?;
        info!(
            subnet = cli.subnet,
            target_hotkey = target_hotkey,
            extrinsic_hash = ?report.extrinsic_hash,
            already_submitted = report.already_submitted,
            elapsed_ms = started_at.elapsed().as_millis(),
            "submitted validator bootstrap weights"
        );
        print!("{}", weight_submission_report(&report));
    }

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "myosu_validator=info".into()),
        )
        .with_target(false)
        .try_init();
}
