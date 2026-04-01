use clap::Parser;
use myosu_miner::axon::axon_plan_from_cli;
use myosu_miner::axon::load_axon_server;
use myosu_miner::axon_report;
use myosu_miner::chain::ensure_registered;
use myosu_miner::chain::ensure_serving;
use myosu_miner::chain::probe_chain;
use myosu_miner::cli::Cli;
use myosu_miner::http_axon_report;
use myosu_miner::registration_report;
use myosu_miner::startup_report;
use myosu_miner::strategy::serve_strategy_once;
use myosu_miner::strategy::strategy_serve_plan_from_cli;
use myosu_miner::strategy_report;
use myosu_miner::training::run_training_batch;
use myosu_miner::training::training_plan_from_cli;
use myosu_miner::training_report;
use subtensor_runtime_common::NetUid;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), myosu_miner::training::MinerBootstrapError> {
    init_tracing();

    let cli = Cli::parse();
    let key_uri = cli.resolve_key_uri()?;
    info!(
        chain = %cli.chain,
        subnet = cli.subnet,
        port = cli.port,
        data_dir = %cli.data_dir.display(),
        key_source = cli.key_source_label(),
        "probing chain for miner bootstrap"
    );

    let report = probe_chain(&cli.chain, NetUid::from(cli.subnet)).await?;
    print!("{}", startup_report(&report));

    if cli.register {
        let report = ensure_registered(&cli.chain, &key_uri, NetUid::from(cli.subnet)).await?;
        print!("{}", registration_report(&report));
    }
    if cli.serve_axon {
        let report =
            ensure_serving(&cli.chain, &key_uri, NetUid::from(cli.subnet), cli.port).await?;
        print!("{}", axon_report(&report));
    }

    let training_plan = training_plan_from_cli(&cli)?;
    let training_report = if let Some(training_plan) = training_plan {
        let training = run_training_batch(&training_plan)?;
        print!("{}", training_report(&training));
        Some(training)
    } else {
        None
    };

    let strategy_plan = strategy_serve_plan_from_cli(
        &cli,
        training_report
            .as_ref()
            .map(|report| report.checkpoint_path.as_path()),
    )?;
    if let Some(strategy_plan) = strategy_plan {
        let strategy = serve_strategy_once(&strategy_plan)?;
        print!("{}", strategy_report(&strategy));
    }

    let axon_plan = axon_plan_from_cli(
        &cli,
        training_report
            .as_ref()
            .map(|report| report.checkpoint_path.as_path()),
    )?;
    if let Some(axon_plan) = axon_plan {
        let server = load_axon_server(&axon_plan).await?;
        print!("{}", http_axon_report(server.report()));
        server.serve().await?;
    }

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "myosu_miner=info".into()),
        )
        .with_target(false)
        .try_init();
}
