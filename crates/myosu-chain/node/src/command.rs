use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Read},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::AtomicBool,
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::{
    chain_spec,
    cli::{Cli, Subcommand, SupportedConsensusMechanism},
    consensus::BabeConsensus,
    service,
};

use crate::consensus::AuraConsensus;
use clap::{ArgMatches, CommandFactory, FromArgMatches, parser::ValueSource};
use jsonrpsee::rpc_params;
use myosu_chain_client::{ChainClient, ChainHeader};
use myosu_chain_runtime::Block;
use sc_cli::SubstrateCli;
use sc_service::{
    ChainType, Configuration,
    config::{ExecutorConfiguration, RpcConfiguration},
};
use subtensor_runtime_common::NetUid;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Myosu Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "support.anonymous.an".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::localnet::localnet_config(true)?),
            "local" => Box::new(chain_spec::localnet::localnet_config(false)?),
            "finney" => Box::new(chain_spec::finney::finney_mainnet_config()?),
            "devnet" => Box::new(chain_spec::devnet::devnet_config()?),
            "" | "test_finney" => Box::new(chain_spec::testnet::finney_testnet_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }
}

// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
    let cmd = Cli::command();
    let arg_matches = cmd.get_matches();
    let cli = Cli::from_arg_matches(&arg_matches)?;

    if cli.smoke_test {
        if cli.subcommand.is_some() || cli.stage0_local_loop_smoke || cli.dual_register_smoke {
            return Err(sc_cli::Error::Input(
                "--smoke-test cannot be combined with a subcommand, --stage0-local-loop-smoke, or --dual-register-smoke".to_string(),
            ));
        }
        return run_smoke_test();
    }
    if cli.stage0_local_loop_smoke {
        if cli.subcommand.is_some() || cli.dual_register_smoke {
            return Err(sc_cli::Error::Input(
                "--stage0-local-loop-smoke cannot be combined with a subcommand or --dual-register-smoke".to_string(),
            ));
        }
        return run_stage0_local_loop_smoke();
    }
    if cli.dual_register_smoke {
        if cli.subcommand.is_some() {
            return Err(sc_cli::Error::Input(
                "--dual-register-smoke cannot be combined with a subcommand".to_string(),
            ));
        }
        return run_dual_register_smoke();
    }

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) =
                    cli.initial_consensus.new_chain_ops(&mut config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) =
                    cli.initial_consensus.new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, _, task_manager) =
                    cli.initial_consensus.new_chain_ops(&mut config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, _, import_queue, task_manager) =
                    cli.initial_consensus.new_chain_ops(&mut config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, backend, _, task_manager) =
                    cli.initial_consensus.new_chain_ops(&mut config)?;
                let aux_revert = Box::new(move |client, _, blocks| {
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            use crate::benchmarking::{
                RemarkBuilder, TransferKeepAliveBuilder, inherent_benchmark_data,
            };
            use frame_benchmarking_cli::{
                BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE,
            };
            use myosu_chain_runtime::EXISTENTIAL_DEPOSIT;
            use sc_service::PartialComponents;
            use sp_keyring::Sr25519Keyring;
            use sp_runtime::traits::HashingFor;

            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| {
                let PartialComponents {
                    client, backend, ..
                } = crate::service::new_partial(
                    &config,
                    Box::new(crate::service::build_manual_seal_import_queue),
                )?;

                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => cmd
                        .run_with_spec::<HashingFor<Block>, crate::client::HostFunctions>(Some(
                            config.chain_spec,
                        )),
                    BenchmarkCmd::Block(cmd) => cmd.run(client),
                    BenchmarkCmd::Storage(cmd) => {
                        let db = backend.expose_db();
                        let storage = backend.expose_storage();
                        let shared_cache = backend.expose_shared_trie_cache();

                        cmd.run(config, client, db, storage, shared_cache)
                    }
                    BenchmarkCmd::Overhead(cmd) => {
                        let ext_builder = RemarkBuilder::new(client.clone());

                        cmd.run(
                            config.chain_spec.name().into(),
                            client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_builder,
                            false,
                        )
                    }
                    BenchmarkCmd::Extrinsic(cmd) => {
                        // Register the *Remark* and *TKA* builders.
                        let ext_factory = ExtrinsicFactory(vec![
                            Box::new(RemarkBuilder::new(client.clone())),
                            Box::new(TransferKeepAliveBuilder::new(
                                client.clone(),
                                Sr25519Keyring::Alice.to_account_id(),
                                EXISTENTIAL_DEPOSIT,
                            )),
                        ]);

                        cmd.run(client, inherent_benchmark_data()?, Vec::new(), &ext_factory)
                    }
                    BenchmarkCmd::Machine(cmd) => {
                        cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
                    }
                }
            })
        }
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
        // Start with the initial consensus type asked.
        None => {
            let arg_matches = Cli::command().get_matches();
            let cli = Cli::from_args();
            match cli.initial_consensus {
                SupportedConsensusMechanism::Babe => start_babe_service(&arg_matches),
                SupportedConsensusMechanism::Aura => start_aura_service(&arg_matches),
            }
        }
    }
}

fn run_smoke_test() -> sc_cli::Result<()> {
    const SMOKE_TIMEOUT: Duration = Duration::from_secs(95);
    const REQUIRED_IMPORTED_BLOCK: u64 = 2;
    const REQUIRED_FINALIZED_BLOCK: u64 = 1;

    ensure_smoke_runtime_wasm_is_fresh()?;
    let current_exe = std::env::current_exe().map_err(boxed_application_error)?;
    let mut child = Command::new(current_exe)
        .args([
            "--dev",
            "--tmp",
            "--force-authoring",
            "--execution",
            "native",
            "--name",
            "Alice",
            "--initial-consensus",
            "aura",
        ])
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(boxed_application_error)?;

    let receiver = capture_child_output(&mut child)?;
    let deadline = Instant::now()
        .checked_add(SMOKE_TIMEOUT)
        .ok_or_else(|| sc_cli::Error::Input("smoke-test timeout overflow".to_string()))?;
    let mut recent_lines = VecDeque::with_capacity(40);
    let mut imported_height = 0_u64;
    let mut finalized_height = 0_u64;

    loop {
        if imported_height >= REQUIRED_IMPORTED_BLOCK
            && finalized_height >= REQUIRED_FINALIZED_BLOCK
        {
            stop_child(&mut child);
            println!("SMOKE myosu-chain ok");
            println!("best_imported={imported_height}");
            println!("best_finalized={finalized_height}");
            return Ok(());
        }

        if Instant::now() >= deadline {
            let summary = recent_log_summary(&recent_lines);
            stop_child(&mut child);
            return Err(sc_cli::Error::Input(format!(
                "devnet smoke test timed out after {}s\n{}",
                SMOKE_TIMEOUT.as_secs(),
                summary
            )));
        }

        match receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(line) => {
                push_recent_line(&mut recent_lines, line.clone());
                if let Some(height) = extract_height(&line, "Imported #") {
                    imported_height = imported_height.max(height);
                }
                if let Some(height) = extract_height(&line, "finalized #") {
                    finalized_height = finalized_height.max(height);
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                if let Some(status) = child.try_wait().map_err(boxed_application_error)? {
                    let summary = recent_log_summary(&recent_lines);
                    return Err(sc_cli::Error::Input(format!(
                        "devnet smoke child exited with status {status}\n{}",
                        summary
                    )));
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(status) = child.try_wait().map_err(boxed_application_error)? {
                    let summary = recent_log_summary(&recent_lines);
                    return Err(sc_cli::Error::Input(format!(
                        "devnet smoke child disconnected with status {status}\n{}",
                        summary
                    )));
                }
            }
        }
    }
}

fn run_dual_register_smoke() -> sc_cli::Result<()> {
    const CHAIN_ACTION_TIMEOUT: Duration = Duration::from_secs(120);
    const STAGE0_BOOTSTRAP_READY_TIMEOUT: Duration = Duration::from_secs(60);
    const STAGE0_RPC_ENDPOINT: &str = "ws://127.0.0.1:9955";
    let stage0_rpc_socket = SocketAddr::from(([127, 0, 0, 1], 9955));
    ensure_smoke_runtime_wasm_is_fresh()?;
    let current_exe = std::env::current_exe().map_err(boxed_application_error)?;
    let mut child = Command::new(current_exe)
        .args([
            "--dev",
            "--tmp",
            "--force-authoring",
            "--execution",
            "native",
            "--name",
            "Alice",
            "--initial-consensus",
            "aura",
            "--rpc-port",
            "9955",
            "--port",
            "30444",
            "--prometheus-port",
            "9616",
        ])
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(boxed_application_error)?;

    let receiver = capture_child_output(&mut child)?;
    thread::spawn(move || {
        while let Ok(line) = receiver.recv() {
            if line.contains("register_network_check") {
                println!("DUAL_REGISTER_NODE {line}");
            }
        }
    });

    let result = (|| -> sc_cli::Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(boxed_application_error)?;
        wait_for_tcp_endpoint(
            &mut child,
            stage0_rpc_socket,
            STAGE0_BOOTSTRAP_READY_TIMEOUT,
            "dual register smoke",
        )?;
        runtime
            .block_on(async {
                let client = ChainClient::connect(STAGE0_RPC_ENDPOINT).await?;
                let first = client
                    .register_network("//Alice", CHAIN_ACTION_TIMEOUT)
                    .await?;
                println!(
                    "DUAL_REGISTER first_subnet={} first_inclusion_block={} first_network_last_registered_at_inclusion={} first_network_last_registered_at_head={}",
                    first.subnet,
                    first.inclusion_block,
                    first.network_last_registered_at_inclusion,
                    first.network_last_registered_at_head,
                );

                let second = client
                    .register_network("//Charlie", CHAIN_ACTION_TIMEOUT)
                    .await?;
                println!(
                    "DUAL_REGISTER second_subnet={} second_inclusion_block={} second_network_last_registered_at_inclusion={} second_network_last_registered_at_head={}",
                    second.subnet,
                    second.inclusion_block,
                    second.network_last_registered_at_inclusion,
                    second.network_last_registered_at_head,
                );
                println!("DUAL_REGISTER ok");
                Ok::<(), myosu_chain_client::ChainClientError>(())
            })
            .map_err(boxed_application_error)
    })();

    stop_child(&mut child);
    result
}

fn run_stage0_local_loop_smoke() -> sc_cli::Result<()> {
    const CHAIN_ACTION_TIMEOUT: Duration = Duration::from_secs(120);
    const STAGE0_BOOTSTRAP_READY_TIMEOUT: Duration = Duration::from_secs(60);
    const STAGE0_MINER_HTTP_READY_TIMEOUT: Duration = Duration::from_secs(30);
    const VALIDATOR_STAKE: u64 = 100_000_000_000_000;
    const STAGE0_RPC_ENDPOINT: &str = "ws://127.0.0.1:9955";
    let stage0_rpc_socket = SocketAddr::from(([127, 0, 0, 1], 9955));
    let poker_miner_http_port = select_local_tcp_port()?;
    let poker_miner_http_socket = SocketAddr::from(([127, 0, 0, 1], poker_miner_http_port));
    let liars_dice_axon_port = select_local_tcp_port()?;

    ensure_smoke_runtime_wasm_is_fresh()?;
    let current_exe = std::env::current_exe().map_err(boxed_application_error)?;
    let mut child = Command::new(current_exe)
        .args([
            "--dev",
            "--tmp",
            "--force-authoring",
            "--execution",
            "native",
            "--name",
            "Alice",
            "--initial-consensus",
            "aura",
            "--rpc-port",
            "9955",
            "--port",
            "30444",
            "--prometheus-port",
            "9616",
        ])
        .env("RUST_LOG", "info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(boxed_application_error)?;

    let result = (|| -> sc_cli::Result<Stage0LocalLoopSummary> {
        let receiver = capture_child_output(&mut child)?;
        thread::spawn(move || {
            while let Ok(line) = receiver.recv() {
                if line.contains("register_network_check") {
                    println!("STAGE0_NODE {line}");
                }
            }
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(boxed_application_error)?;
        println!("STAGE0_STEP waiting_for_bootstrap_node");
        wait_for_tcp_endpoint(
            &mut child,
            stage0_rpc_socket,
            STAGE0_BOOTSTRAP_READY_TIMEOUT,
            "stage-0 local loop smoke",
        )?;
        let repo_root = workspace_root()?;
        let poker_harness = Stage0HarnessPaths::new(repo_root.clone(), "poker")?;
        let liars_dice_harness = Stage0HarnessPaths::new(repo_root.clone(), "liars-dice")?;
        println!("STAGE0_STEP registering_poker_subnet");
        let poker_subnet = runtime
            .block_on(async {
                let client = ChainClient::connect(STAGE0_RPC_ENDPOINT).await?;
                let report = client
                    .register_network("//Alice", CHAIN_ACTION_TIMEOUT)
                    .await?;
                println!(
                    "STAGE0_DIAG poker_register inclusion_block={} network_last_registered_at_inclusion={} network_last_registered_at_head={}",
                    report.inclusion_block,
                    report.network_last_registered_at_inclusion,
                    report.network_last_registered_at_head,
                );
                Ok::<NetUid, myosu_chain_client::ChainClientError>(report.subnet)
            })
            .map_err(boxed_application_error)?;
        let poker_subnet_arg = poker_subnet.to_string();
        let validator_stake_arg = VALIDATOR_STAKE.to_string();
        let poker_miner_http_port_arg = poker_miner_http_port.to_string();
        let liars_dice_axon_port_arg = liars_dice_axon_port.to_string();

        println!("STAGE0_STEP writing_poker_bootstrap_artifacts subnet={poker_subnet}");
        run_cargo_command(
            &poker_harness.repo_root,
            "bootstrap_artifacts_poker",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-games-poker",
                "--example",
                "bootstrap_artifacts",
                "--",
                poker_harness.encoder_dir_string()?,
                poker_harness.query_file_string()?,
            ],
        )?;
        println!("STAGE0_STEP running_poker_miner subnet={poker_subnet}");
        run_cargo_command(
            &poker_harness.repo_root,
            "myosu-miner-poker",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-miner",
                "--",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &poker_subnet_arg,
                "--key",
                "//Alice",
                "--port",
                &poker_miner_http_port_arg,
                "--register",
                "--serve-axon",
                "--encoder-dir",
                poker_harness.encoder_dir_string()?,
                "--query-file",
                poker_harness.query_file_string()?,
                "--response-file",
                poker_harness.response_file_string()?,
                "--data-dir",
                poker_harness.miner_data_string()?,
            ],
        )?;
        println!("STAGE0_STEP configuring_poker_owner_validator subnet={poker_subnet}");
        run_cargo_command(
            &poker_harness.repo_root,
            "myosu-validator-owner-poker",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-validator",
                "--",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &poker_subnet_arg,
                "--key",
                "//Alice",
                "--enable-subtoken",
            ],
        )?;
        println!("STAGE0_STEP running_poker_bob_validator subnet={poker_subnet}");
        run_cargo_command(
            &poker_harness.repo_root,
            "myosu-validator-bob-poker",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-validator",
                "--",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &poker_subnet_arg,
                "--key",
                "//Bob",
                "--register",
                "--stake-amount",
                &validator_stake_arg,
                "--submit-weights",
                "--weight-hotkey",
                "//Alice",
                "--encoder-dir",
                poker_harness.encoder_dir_string()?,
                "--checkpoint",
                poker_harness.checkpoint_string()?,
                "--query-file",
                poker_harness.query_file_string()?,
                "--response-file",
                poker_harness.response_file_string()?,
            ],
        )?;
        println!("STAGE0_STEP verifying_poker_chain_state subnet={poker_subnet}");
        let poker_verified = runtime.block_on(verify_stage0_state(
            STAGE0_RPC_ENDPOINT,
            poker_subnet,
            "//Alice",
            "//Bob",
            CHAIN_ACTION_TIMEOUT,
        ))?;
        println!("STAGE0_STEP serving_poker_miner_http subnet={poker_subnet}");
        let mut miner_http = spawn_cargo_process(
            &poker_harness.repo_root,
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-miner",
                "--",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &poker_subnet_arg,
                "--key",
                "//Alice",
                "--port",
                &poker_miner_http_port_arg,
                "--encoder-dir",
                poker_harness.encoder_dir_string()?,
                "--checkpoint",
                poker_harness.checkpoint_string()?,
                "--serve-http",
            ],
        )?;
        wait_for_tcp_endpoint(
            &mut miner_http,
            poker_miner_http_socket,
            STAGE0_MINER_HTTP_READY_TIMEOUT,
            "stage-0 miner HTTP axon",
        )?;

        println!("STAGE0_STEP running_poker_gameplay_smoke subnet={poker_subnet}");
        let poker_gameplay_smoke = run_cargo_command(
            &poker_harness.repo_root,
            "myosu-play-smoke-poker",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-play",
                "--",
                "--smoke-test",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &poker_subnet_arg,
                "--require-discovery",
                "--require-live-query",
                "--require-artifact",
                "--smoke-checkpoint",
                poker_harness.checkpoint_string()?,
                "--smoke-encoder-dir",
                poker_harness.encoder_dir_string()?,
            ],
        );
        stop_child(&mut miner_http);
        let poker_gameplay_smoke = poker_gameplay_smoke?;
        if !poker_gameplay_smoke.contains("SMOKE myosu-play ok") {
            return Err(sc_cli::Error::Input(format!(
                "stage-0 gameplay smoke did not report success\n{}",
                trim_command_output(&poker_gameplay_smoke)
            )));
        }
        if !poker_gameplay_smoke.contains("advice_source=artifact") {
            return Err(sc_cli::Error::Input(format!(
                "stage-0 gameplay smoke did not use artifact advice\n{}",
                trim_command_output(&poker_gameplay_smoke)
            )));
        }
        if !poker_gameplay_smoke.contains("final_state=complete") {
            return Err(sc_cli::Error::Input(format!(
                "stage-0 gameplay smoke did not complete a hand\n{}",
                trim_command_output(&poker_gameplay_smoke)
            )));
        }
        if !poker_gameplay_smoke.contains("miner_discovery=chain_visible") {
            return Err(sc_cli::Error::Input(format!(
                "stage-0 gameplay smoke did not discover a chain-visible miner\n{}",
                trim_command_output(&poker_gameplay_smoke)
            )));
        }
        if !poker_gameplay_smoke.contains("live_miner_query=live_http") {
            return Err(sc_cli::Error::Input(format!(
                "stage-0 gameplay smoke did not execute a live miner query\n{}",
                trim_command_output(&poker_gameplay_smoke)
            )));
        }
        let poker_gameplay_discovered_miner_uid = required_command_kv(
            &poker_gameplay_smoke,
            "discovered_miner_uid",
            "poker gameplay smoke",
        )?
        .parse::<u16>()
        .map_err(boxed_application_error)?;
        let poker_gameplay_discovered_miner_endpoint = required_command_kv(
            &poker_gameplay_smoke,
            "discovered_miner_endpoint",
            "poker gameplay smoke",
        )?
        .to_string();
        let poker_gameplay_live_miner_connect_endpoint = required_command_kv(
            &poker_gameplay_smoke,
            "live_miner_connect_endpoint",
            "poker gameplay smoke",
        )?
        .to_string();
        let poker_summary = Stage0GameLoopSummary {
            subnet: poker_verified.subnet,
            alice_uid: poker_verified.alice_uid,
            bob_uid: poker_verified.bob_uid,
            bob_has_validator_permit: poker_verified.bob_has_validator_permit,
            bob_weights: poker_verified.bob_weights,
            gameplay_advice_source: required_command_kv(
                &poker_gameplay_smoke,
                "advice_source",
                "poker gameplay smoke",
            )?
            .to_string(),
            gameplay_final_state: required_command_kv(
                &poker_gameplay_smoke,
                "final_state",
                "poker gameplay smoke",
            )?
            .to_string(),
            gameplay_discovered_miner_uid: Some(poker_gameplay_discovered_miner_uid),
            gameplay_discovered_miner_endpoint: Some(poker_gameplay_discovered_miner_endpoint),
            gameplay_live_miner_connect_endpoint: Some(poker_gameplay_live_miner_connect_endpoint),
            alice_miner_incentive: poker_verified.alice_miner_incentive,
            bob_validator_dividend: poker_verified.bob_validator_dividend,
            alice_miner_emission: poker_verified.alice_miner_emission,
        };

        const LIARS_DICE_MINER_KEY_URI: &str = "//Charlie";
        const LIARS_DICE_VALIDATOR_KEY_URI: &str = "//Bob";
        println!("STAGE0_STEP registering_liars_dice_subnet");
        let liars_dice_subnet = runtime
            .block_on(async {
                let client = ChainClient::connect(STAGE0_RPC_ENDPOINT).await?;
                let best_block = client.best_block_number().await?;
                let network_rate_limit = client.get_network_rate_limit().await?;
                let network_last_registered =
                    client.get_network_last_registered_block().await?;
                println!(
                    "STAGE0_DIAG liars_dice_pre_register best_block={best_block} network_rate_limit={network_rate_limit} network_last_registered={network_last_registered}"
                );
                let report = client
                    .register_network(LIARS_DICE_MINER_KEY_URI, CHAIN_ACTION_TIMEOUT)
                    .await?;
                Ok::<NetUid, myosu_chain_client::ChainClientError>(report.subnet)
            })
            .map_err(boxed_application_error)?;
        let liars_dice_subnet_arg = liars_dice_subnet.to_string();

        println!("STAGE0_STEP writing_liars_dice_bootstrap_query subnet={liars_dice_subnet}");
        run_cargo_command(
            &liars_dice_harness.repo_root,
            "bootstrap_query_liars_dice",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-games-liars-dice",
                "--example",
                "bootstrap_query",
                "--",
                liars_dice_harness.query_file_string()?,
            ],
        )?;
        println!("STAGE0_STEP running_liars_dice_miner subnet={liars_dice_subnet}");
        run_cargo_command(
            &liars_dice_harness.repo_root,
            "myosu-miner-liars-dice",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-miner",
                "--",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &liars_dice_subnet_arg,
                "--key",
                LIARS_DICE_MINER_KEY_URI,
                "--port",
                &liars_dice_axon_port_arg,
                "--register",
                "--serve-axon",
                "--game",
                "liars-dice",
                "--train-iterations",
                "64",
                "--query-file",
                liars_dice_harness.query_file_string()?,
                "--response-file",
                liars_dice_harness.response_file_string()?,
                "--data-dir",
                liars_dice_harness.miner_data_string()?,
            ],
        )?;
        println!("STAGE0_STEP configuring_liars_dice_owner_validator subnet={liars_dice_subnet}");
        run_cargo_command(
            &liars_dice_harness.repo_root,
            "myosu-validator-owner-liars-dice",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-validator",
                "--",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &liars_dice_subnet_arg,
                "--key",
                LIARS_DICE_MINER_KEY_URI,
                "--enable-subtoken",
            ],
        )?;
        println!("STAGE0_STEP running_liars_dice_bob_validator subnet={liars_dice_subnet}");
        run_cargo_command(
            &liars_dice_harness.repo_root,
            "myosu-validator-bob-liars-dice",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-validator",
                "--",
                "--chain",
                STAGE0_RPC_ENDPOINT,
                "--subnet",
                &liars_dice_subnet_arg,
                "--key",
                LIARS_DICE_VALIDATOR_KEY_URI,
                "--register",
                "--stake-amount",
                &validator_stake_arg,
                "--submit-weights",
                "--weight-hotkey",
                LIARS_DICE_MINER_KEY_URI,
                "--game",
                "liars-dice",
                "--checkpoint",
                liars_dice_harness.checkpoint_string()?,
                "--query-file",
                liars_dice_harness.query_file_string()?,
                "--response-file",
                liars_dice_harness.response_file_string()?,
            ],
        )?;

        println!("STAGE0_STEP verifying_liars_dice_chain_state subnet={liars_dice_subnet}");
        let liars_dice_verified = runtime.block_on(verify_stage0_state(
            STAGE0_RPC_ENDPOINT,
            liars_dice_subnet,
            LIARS_DICE_MINER_KEY_URI,
            LIARS_DICE_VALIDATOR_KEY_URI,
            CHAIN_ACTION_TIMEOUT,
        ))?;
        println!("STAGE0_STEP running_liars_dice_gameplay_smoke subnet={liars_dice_subnet}");
        let liars_dice_gameplay_smoke = run_cargo_command(
            &liars_dice_harness.repo_root,
            "myosu-play-smoke-liars-dice",
            &[
                "run",
                "--quiet",
                "-p",
                "myosu-play",
                "--",
                "--smoke-test",
                "--game",
                "liars-dice",
                "--require-artifact",
                "--smoke-checkpoint",
                liars_dice_harness.checkpoint_string()?,
            ],
        )?;
        if !liars_dice_gameplay_smoke.contains("SMOKE myosu-play ok") {
            return Err(sc_cli::Error::Input(format!(
                "liar's dice gameplay smoke did not report success\n{}",
                trim_command_output(&liars_dice_gameplay_smoke)
            )));
        }
        if !liars_dice_gameplay_smoke.contains("game=liars_dice") {
            return Err(sc_cli::Error::Input(format!(
                "liar's dice gameplay smoke did not report the game label\n{}",
                trim_command_output(&liars_dice_gameplay_smoke)
            )));
        }
        if !liars_dice_gameplay_smoke.contains("advice_source=artifact") {
            return Err(sc_cli::Error::Input(format!(
                "liar's dice gameplay smoke did not use artifact advice\n{}",
                trim_command_output(&liars_dice_gameplay_smoke)
            )));
        }
        if !liars_dice_gameplay_smoke.contains("final_state=static_demo") {
            return Err(sc_cli::Error::Input(format!(
                "liar's dice gameplay smoke did not reach the expected static demo terminal state\n{}",
                trim_command_output(&liars_dice_gameplay_smoke)
            )));
        }
        let liars_dice_summary = Stage0GameLoopSummary {
            subnet: liars_dice_verified.subnet,
            alice_uid: liars_dice_verified.alice_uid,
            bob_uid: liars_dice_verified.bob_uid,
            bob_has_validator_permit: liars_dice_verified.bob_has_validator_permit,
            bob_weights: liars_dice_verified.bob_weights,
            gameplay_advice_source: required_command_kv(
                &liars_dice_gameplay_smoke,
                "advice_source",
                "liar's dice gameplay smoke",
            )?
            .to_string(),
            gameplay_final_state: required_command_kv(
                &liars_dice_gameplay_smoke,
                "final_state",
                "liar's dice gameplay smoke",
            )?
            .to_string(),
            gameplay_discovered_miner_uid: None,
            gameplay_discovered_miner_endpoint: None,
            gameplay_live_miner_connect_endpoint: None,
            alice_miner_incentive: liars_dice_verified.alice_miner_incentive,
            bob_validator_dividend: liars_dice_verified.bob_validator_dividend,
            alice_miner_emission: liars_dice_verified.alice_miner_emission,
        };

        Ok(Stage0LocalLoopSummary {
            poker: poker_summary,
            liars_dice: liars_dice_summary,
            imported_height: liars_dice_verified.imported_height,
            finalized_height: liars_dice_verified.finalized_height,
        })
    })();

    stop_child(&mut child);
    let summary = result?;
    let Some(poker_gameplay_discovered_miner_uid) = summary.poker.gameplay_discovered_miner_uid
    else {
        return Err(sc_cli::Error::Input(
            "poker smoke should emit discovered miner uid".to_string(),
        ));
    };
    let Some(poker_gameplay_discovered_miner_endpoint) =
        summary.poker.gameplay_discovered_miner_endpoint.as_deref()
    else {
        return Err(sc_cli::Error::Input(
            "poker smoke should emit discovered miner endpoint".to_string(),
        ));
    };
    let Some(poker_gameplay_live_miner_connect_endpoint) = summary
        .poker
        .gameplay_live_miner_connect_endpoint
        .as_deref()
    else {
        return Err(sc_cli::Error::Input(
            "poker smoke should emit live miner connect endpoint".to_string(),
        ));
    };
    println!("STAGE0 myosu-chain local-loop ok");
    println!("poker_subnet={}", summary.poker.subnet);
    println!("poker_alice_uid={}", summary.poker.alice_uid);
    println!("poker_bob_uid={}", summary.poker.bob_uid);
    println!(
        "poker_bob_has_validator_permit={}",
        summary.poker.bob_has_validator_permit
    );
    println!("poker_bob_weights={:?}", summary.poker.bob_weights);
    println!(
        "poker_gameplay_advice_source={}",
        summary.poker.gameplay_advice_source
    );
    println!(
        "poker_gameplay_final_state={}",
        summary.poker.gameplay_final_state
    );
    println!(
        "poker_gameplay_discovered_miner_uid={}",
        poker_gameplay_discovered_miner_uid
    );
    println!(
        "poker_gameplay_discovered_miner_endpoint={}",
        poker_gameplay_discovered_miner_endpoint
    );
    println!(
        "poker_gameplay_live_miner_connect_endpoint={}",
        poker_gameplay_live_miner_connect_endpoint
    );
    println!(
        "poker_alice_miner_incentive={}",
        summary.poker.alice_miner_incentive
    );
    println!(
        "poker_bob_validator_dividend={}",
        summary.poker.bob_validator_dividend
    );
    println!(
        "poker_alice_miner_emission={}",
        summary.poker.alice_miner_emission
    );
    println!("liars_dice_subnet={}", summary.liars_dice.subnet);
    println!("liars_dice_alice_uid={}", summary.liars_dice.alice_uid);
    println!("liars_dice_bob_uid={}", summary.liars_dice.bob_uid);
    println!(
        "liars_dice_bob_has_validator_permit={}",
        summary.liars_dice.bob_has_validator_permit
    );
    println!(
        "liars_dice_bob_weights={:?}",
        summary.liars_dice.bob_weights
    );
    println!(
        "liars_dice_gameplay_advice_source={}",
        summary.liars_dice.gameplay_advice_source
    );
    println!(
        "liars_dice_gameplay_final_state={}",
        summary.liars_dice.gameplay_final_state
    );
    println!(
        "liars_dice_alice_miner_incentive={}",
        summary.liars_dice.alice_miner_incentive
    );
    println!(
        "liars_dice_bob_validator_dividend={}",
        summary.liars_dice.bob_validator_dividend
    );
    println!(
        "liars_dice_alice_miner_emission={}",
        summary.liars_dice.alice_miner_emission
    );
    println!("best_imported={}", summary.imported_height);
    println!("best_finalized={}", summary.finalized_height);
    Ok(())
}

fn capture_child_output(child: &mut Child) -> sc_cli::Result<Receiver<String>> {
    let stdout = child.stdout.take().ok_or_else(|| {
        sc_cli::Error::Input("failed to capture smoke-test child stdout".to_string())
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        sc_cli::Error::Input("failed to capture smoke-test child stderr".to_string())
    })?;

    let (sender, receiver) = mpsc::channel();
    spawn_output_thread(stdout, sender.clone());
    spawn_output_thread(stderr, sender);
    Ok(receiver)
}

fn spawn_output_thread<R>(reader: R, sender: Sender<String>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let _ = sender.send(line);
        }
    });
}

#[derive(Debug)]
struct Stage0GameLoopSummary {
    subnet: NetUid,
    alice_uid: u16,
    bob_uid: u16,
    bob_has_validator_permit: bool,
    bob_weights: Vec<(u16, u16)>,
    gameplay_advice_source: String,
    gameplay_final_state: String,
    gameplay_discovered_miner_uid: Option<u16>,
    gameplay_discovered_miner_endpoint: Option<String>,
    gameplay_live_miner_connect_endpoint: Option<String>,
    alice_miner_incentive: u16,
    bob_validator_dividend: u16,
    alice_miner_emission: u64,
}

#[derive(Debug)]
struct Stage0LocalLoopSummary {
    poker: Stage0GameLoopSummary,
    liars_dice: Stage0GameLoopSummary,
    imported_height: u64,
    finalized_height: u64,
}

async fn verify_stage0_state(
    chain_endpoint: &str,
    subnet: NetUid,
    miner_key_uri: &str,
    validator_key_uri: &str,
    timeout: Duration,
) -> sc_cli::Result<VerifiedStage0State> {
    let client = ChainClient::connect(chain_endpoint)
        .await
        .map_err(boxed_application_error)?;
    let alice_hotkey =
        ChainClient::account_id_from_uri(miner_key_uri).map_err(boxed_application_error)?;
    let bob_hotkey =
        ChainClient::account_id_from_uri(validator_key_uri).map_err(boxed_application_error)?;
    let alice_uid = client
        .get_uid_for_net_and_hotkey(subnet, &alice_hotkey)
        .await
        .map_err(boxed_application_error)?
        .ok_or_else(|| {
            sc_cli::Error::Input(format!(
                "missing Alice uid on subnet {subnet} after stage-0 smoke"
            ))
        })?;
    let bob_uid = client
        .get_uid_for_net_and_hotkey(subnet, &bob_hotkey)
        .await
        .map_err(boxed_application_error)?
        .ok_or_else(|| {
            sc_cli::Error::Input(format!(
                "missing Bob uid on subnet {subnet} after stage-0 smoke"
            ))
        })?;
    let bob_has_validator_permit = client
        .has_validator_permit(subnet, bob_uid)
        .await
        .map_err(boxed_application_error)?;
    let bob_weights = client
        .get_weights_for_uid(subnet, bob_uid)
        .await
        .map_err(boxed_application_error)?;
    let expected_weights = vec![(alice_uid, u16::MAX)];
    if !bob_has_validator_permit {
        return Err(sc_cli::Error::Input(format!(
            "Bob validator permit did not converge on subnet {subnet}"
        )));
    }
    if bob_weights != expected_weights {
        return Err(sc_cli::Error::Input(format!(
            "unexpected Bob weights on subnet {subnet}: expected {expected_weights:?}, got {bob_weights:?}"
        )));
    }
    let epoch_outcome = client
        .wait_for_epoch_outcome(subnet, alice_uid, bob_uid, timeout)
        .await
        .map_err(boxed_application_error)?;
    let imported_header = client
        .chain_get_header()
        .await
        .map_err(boxed_application_error)?;
    let finalized_hash: String = client
        .request("chain_getFinalizedHead", rpc_params![])
        .await
        .map_err(boxed_application_error)?;
    let finalized_header: ChainHeader = client
        .request("chain_getHeader", rpc_params![finalized_hash])
        .await
        .map_err(boxed_application_error)?;

    Ok(VerifiedStage0State {
        subnet,
        alice_uid,
        bob_uid,
        bob_has_validator_permit,
        bob_weights,
        alice_miner_incentive: epoch_outcome.miner_incentive,
        bob_validator_dividend: epoch_outcome.validator_dividend,
        alice_miner_emission: epoch_outcome.miner_emission,
        imported_height: parse_chain_header_number(&imported_header)?,
        finalized_height: parse_chain_header_number(&finalized_header)?,
    })
}

fn required_command_kv<'a>(output: &'a str, key: &str, label: &str) -> sc_cli::Result<&'a str> {
    extract_command_kv(output, key).ok_or_else(|| {
        sc_cli::Error::Input(format!(
            "{label} did not emit {key}\n{}",
            trim_command_output(output)
        ))
    })
}

#[derive(Debug)]
struct VerifiedStage0State {
    subnet: NetUid,
    alice_uid: u16,
    bob_uid: u16,
    bob_has_validator_permit: bool,
    bob_weights: Vec<(u16, u16)>,
    alice_miner_incentive: u16,
    bob_validator_dividend: u16,
    alice_miner_emission: u64,
    imported_height: u64,
    finalized_height: u64,
}

#[derive(Debug)]
struct Stage0HarnessPaths {
    repo_root: PathBuf,
    encoder_dir: PathBuf,
    query_file: PathBuf,
    response_file: PathBuf,
    miner_data_dir: PathBuf,
    checkpoint_path: PathBuf,
}

impl Stage0HarnessPaths {
    fn new(repo_root: PathBuf, label: &str) -> sc_cli::Result<Self> {
        let root = unique_stage0_temp_root().join(label);
        let encoder_dir = root.join("encoder");
        let query_file = root.join("query.bin");
        let response_file = root.join("response.bin");
        let miner_data_dir = root.join("miner-data");
        let checkpoint_path = miner_data_dir.join("checkpoints").join("latest.bin");
        fs::create_dir_all(&root).map_err(boxed_application_error)?;
        Ok(Self {
            repo_root,
            encoder_dir,
            query_file,
            response_file,
            miner_data_dir,
            checkpoint_path,
        })
    }

    fn encoder_dir_string(&self) -> sc_cli::Result<&str> {
        path_as_str(&self.encoder_dir)
    }

    fn query_file_string(&self) -> sc_cli::Result<&str> {
        path_as_str(&self.query_file)
    }

    fn response_file_string(&self) -> sc_cli::Result<&str> {
        path_as_str(&self.response_file)
    }

    fn miner_data_string(&self) -> sc_cli::Result<&str> {
        path_as_str(&self.miner_data_dir)
    }

    fn checkpoint_string(&self) -> sc_cli::Result<&str> {
        path_as_str(&self.checkpoint_path)
    }
}

fn parse_chain_header_number(header: &ChainHeader) -> sc_cli::Result<u64> {
    let trimmed = header.number.trim_start_matches("0x");
    u64::from_str_radix(trimmed, 16).map_err(boxed_application_error)
}

fn wait_for_tcp_endpoint(
    child: &mut Child,
    address: SocketAddr,
    timeout: Duration,
    operation: &str,
) -> sc_cli::Result<()> {
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| sc_cli::Error::Input(format!("{operation} timeout overflow")))?;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().map_err(boxed_application_error)? {
            return Err(sc_cli::Error::Input(format!(
                "{operation} child exited with status {status} before RPC became ready"
            )));
        }
        if TcpStream::connect_timeout(&address, Duration::from_millis(500)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    Err(sc_cli::Error::Input(format!(
        "{operation} timed out waiting for RPC endpoint {address} after {}s",
        timeout.as_secs()
    )))
}

fn run_cargo_command(repo_root: &Path, label: &str, args: &[&str]) -> sc_cli::Result<String> {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(boxed_application_error)?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if output.status.success() {
        return Ok(stdout);
    }
    Err(sc_cli::Error::Input(format!(
        "{label} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        trim_command_output(&stdout),
        trim_command_output(&stderr)
    )))
}

fn spawn_cargo_process(repo_root: &Path, args: &[&str]) -> sc_cli::Result<Child> {
    Command::new("cargo")
        .args(args)
        .current_dir(repo_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(boxed_application_error)
}

fn trim_command_output(output: &str) -> String {
    const OUTPUT_LIMIT: usize = 4000;
    if output.len() <= OUTPUT_LIMIT {
        return output.to_string();
    }
    let start = output.len().saturating_sub(OUTPUT_LIMIT);
    format!("...[truncated]...\n{}", &output[start..])
}

fn extract_command_kv<'a>(output: &'a str, key: &str) -> Option<&'a str> {
    output.lines().find_map(|line| {
        let (line_key, value) = line.split_once('=')?;
        (line_key == key).then_some(value)
    })
}

fn workspace_root() -> sc_cli::Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            sc_cli::Error::Input("failed to resolve workspace root from node crate".to_string())
        })
}

fn ensure_smoke_runtime_wasm_is_fresh() -> sc_cli::Result<()> {
    let workspace_root = workspace_root()?;
    let cached_wasm = find_cached_runtime_wasm(&workspace_root).ok_or_else(|| {
        sc_cli::Error::Input(
            "local smoke commands require a cached runtime wasm, but none was found under target/{debug,release}/wbuild/myosu-chain-runtime".to_string(),
        )
    })?;
    let cached_wasm_mtime = fs::metadata(&cached_wasm)
        .map_err(boxed_application_error)?
        .modified()
        .map_err(boxed_application_error)?;
    let source_roots = [
        workspace_root.join("crates/myosu-chain/runtime"),
        workspace_root.join("crates/myosu-chain/pallets"),
        workspace_root.join("crates/myosu-chain/common"),
        workspace_root.join("crates/myosu-chain/runtime-common"),
        workspace_root.join("crates/myosu-chain/support"),
    ];
    let Some((newest_source_path, newest_source_mtime)) = newest_source_file(&source_roots)? else {
        return Ok(());
    };
    if newest_source_mtime <= cached_wasm_mtime {
        return Ok(());
    }

    Err(sc_cli::Error::Input(format!(
        "local smoke commands are blocked because the cached runtime wasm is stale\ncached_wasm={}\ncached_wasm_mtime={}\nnewest_runtime_source={}\nnewest_runtime_source_mtime={}\nrefresh the runtime wasm cache with a wasm-capable toolchain before trusting local chain smoke results",
        cached_wasm.display(),
        system_time_label(cached_wasm_mtime),
        newest_source_path.display(),
        system_time_label(newest_source_mtime),
    )))
}

fn find_cached_runtime_wasm(workspace_root: &Path) -> Option<PathBuf> {
    [
        workspace_root
            .join("target")
            .join("debug")
            .join("wbuild")
            .join("myosu-chain-runtime")
            .join("myosu_chain_runtime.wasm"),
        workspace_root
            .join("target")
            .join("release")
            .join("wbuild")
            .join("myosu-chain-runtime")
            .join("myosu_chain_runtime.wasm"),
    ]
    .into_iter()
    .find(|candidate| candidate.is_file())
}

fn newest_source_file(source_roots: &[PathBuf]) -> sc_cli::Result<Option<(PathBuf, SystemTime)>> {
    let mut newest: Option<(PathBuf, SystemTime)> = None;
    for root in source_roots {
        visit_source_tree(root, &mut newest)?;
    }
    Ok(newest)
}

fn visit_source_tree(
    path: &Path,
    newest: &mut Option<(PathBuf, SystemTime)>,
) -> sc_cli::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        let modified = fs::metadata(path)
            .map_err(boxed_application_error)?
            .modified()
            .map_err(boxed_application_error)?;
        let should_replace = newest
            .as_ref()
            .map(|(_, current)| modified > *current)
            .unwrap_or(true);
        if should_replace {
            *newest = Some((path.to_path_buf(), modified));
        }
        return Ok(());
    }

    for entry in fs::read_dir(path).map_err(boxed_application_error)? {
        let entry = entry.map_err(boxed_application_error)?;
        visit_source_tree(&entry.path(), newest)?;
    }
    Ok(())
}

fn system_time_label(time: SystemTime) -> String {
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{seconds}s_since_unix_epoch")
}

fn unique_stage0_temp_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "myosu-stage0-local-loop-{}-{nanos}",
        std::process::id()
    ))
}

fn path_as_str(path: &Path) -> sc_cli::Result<&str> {
    path.to_str().ok_or_else(|| {
        sc_cli::Error::Input(format!(
            "stage-0 smoke path is not valid UTF-8: {}",
            path.display()
        ))
    })
}

fn select_local_tcp_port() -> sc_cli::Result<u16> {
    std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .map_err(boxed_application_error)?
        .local_addr()
        .map(|address| address.port())
        .map_err(boxed_application_error)
}

fn extract_height(line: &str, marker: &str) -> Option<u64> {
    let start = line.find(marker)?.checked_add(marker.len())?;
    let digits: String = line[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

fn push_recent_line(recent_lines: &mut VecDeque<String>, line: String) {
    const RECENT_LINE_LIMIT: usize = 40;
    if recent_lines.len() == RECENT_LINE_LIMIT {
        recent_lines.pop_front();
    }
    recent_lines.push_back(line);
}

fn recent_log_summary(recent_lines: &VecDeque<String>) -> String {
    if recent_lines.is_empty() {
        return "recent logs: <none>".to_string();
    }

    let joined = recent_lines
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    format!("recent logs:\n{joined}")
}

fn stop_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn boxed_application_error(error: impl std::error::Error + Send + Sync + 'static) -> sc_cli::Error {
    sc_cli::Error::Application(Box::new(error))
}

#[allow(clippy::expect_used)]
fn start_babe_service(arg_matches: &ArgMatches) -> Result<(), sc_cli::Error> {
    let cli = Cli::from_arg_matches(arg_matches).expect("Bad arg_matches");
    let runner = cli.create_runner(&cli.run)?;
    match runner.run_node_until_exit(|config| async move {
        let config = customise_config(arg_matches, config);
        service::build_full::<BabeConsensus>(config, cli.sealing, None).await
    }) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Handle node needs to be in Aura mode.
            if matches!(
                e,
                sc_service::Error::Client(sp_blockchain::Error::VersionInvalid(ref msg))
                    if msg == "Unsupported or invalid BabeApi version"
            ) {
                log::info!(
                    "💡 Chain is using Aura consensus. Switching to Aura service until Babe block is detected.",
                );
                start_aura_service(arg_matches)
            // Handle Aura service still has DB lock. This never has been observed to take more
            // than 1s to drop.
            } else if matches!(e, sc_service::Error::Client(sp_blockchain::Error::Backend(ref msg))
                if msg.starts_with("IO error: lock hold by current process"))
            {
                log::info!("Failed to aquire DB lock, trying again in 1s...");
                std::thread::sleep(std::time::Duration::from_secs(1));
                start_babe_service(arg_matches)
            // Unknown error, return it.
            } else {
                log::error!("Failed to start Babe service: {e:?}");
                Err(e.into())
            }
        }
    }
}

#[allow(clippy::expect_used)]
fn start_aura_service(arg_matches: &ArgMatches) -> Result<(), sc_cli::Error> {
    let cli = Cli::from_arg_matches(arg_matches).expect("Bad arg_matches");
    let runner = cli.create_runner(&cli.run)?;

    // Unlike when the Babe node fails to build due to missing BabeApi in the runtime,
    // there is no way to detect the exit reason for the Aura node when it encounters a Babe block.
    //
    // Passing this atomic bool is a hacky solution, allowing the node to set it to true to indicate
    // a Babe service should be spawned on exit instead of a regular shutdown.
    let custom_service_signal = Arc::new(AtomicBool::new(false));
    let custom_service_signal_clone = custom_service_signal.clone();
    match runner.run_node_until_exit(|config| async move {
        let config = customise_config(arg_matches, config);
        service::build_full::<AuraConsensus>(config, cli.sealing, Some(custom_service_signal_clone))
            .await
    }) {
        Ok(()) => Ok(()),
        Err(e) => {
            if custom_service_signal.load(std::sync::atomic::Ordering::Relaxed) {
                start_babe_service(arg_matches)
            } else {
                Err(e.into())
            }
        }
    }
}

#[allow(clippy::expect_used)]
fn customise_config(arg_matches: &ArgMatches, config: Configuration) -> Configuration {
    let cli = Cli::from_arg_matches(arg_matches).expect("Bad arg_matches");

    let mut config = override_default_heap_pages(config, 60_000);

    // If the operator did **not** supply `--rpc-rate-limit`, disable the limiter.
    if cli.run.rpc_params.rpc_rate_limit.is_none() {
        config.rpc.rate_limit = None;
    }

    // If the operator did **not** supply `--rpc-max-subscriptions-per-connection` set to high value.
    config.rpc.max_subs_per_conn = match arg_matches
        .value_source("rpc_max_subscriptions_per_connection")
    {
        Some(ValueSource::CommandLine) => cli.run.rpc_params.rpc_max_subscriptions_per_connection,
        _ => 10000,
    };

    // If the operator did **not** supply `--rpc-max-connections` set to high value.
    config.rpc.max_connections = match arg_matches.value_source("rpc_max_connections") {
        Some(ValueSource::CommandLine) => cli.run.rpc_params.rpc_max_connections,
        _ => 10000,
    };

    // The fork-aware pool currently tears down the restarted local devnet during startup.
    // Keep honoring explicit operator choice, but default local chains to the stable single-state
    // pool until the fork-aware path is reconciled for this runtime.
    if matches!(config.chain_spec.chain_type(), ChainType::Local)
        && !matches!(
            arg_matches.value_source("pool_type"),
            Some(ValueSource::CommandLine)
        )
    {
        let pool_bytes = cli.run.pool_config.pool_kbytes.saturating_mul(1024);
        config.transaction_pool = sc_transaction_pool::TransactionPoolOptions::new_with_params(
            cli.run.pool_config.pool_limit,
            pool_bytes,
            cli.run.pool_config.tx_ban_seconds,
            sc_transaction_pool::TransactionPoolType::SingleState,
            true,
        );
    }

    config
}

/// Override default heap pages
fn override_default_heap_pages(config: Configuration, pages: u64) -> Configuration {
    Configuration {
        impl_name: config.impl_name,
        impl_version: config.impl_version,
        role: config.role,
        tokio_handle: config.tokio_handle,
        transaction_pool: config.transaction_pool,
        network: config.network,
        keystore: config.keystore,
        database: config.database,
        trie_cache_maximum_size: config.trie_cache_maximum_size,
        warm_up_trie_cache: config.warm_up_trie_cache,
        state_pruning: config.state_pruning,
        blocks_pruning: config.blocks_pruning,
        chain_spec: config.chain_spec,
        wasm_runtime_overrides: config.wasm_runtime_overrides,
        prometheus_config: config.prometheus_config,
        telemetry_endpoints: config.telemetry_endpoints,
        offchain_worker: config.offchain_worker,
        force_authoring: config.force_authoring,
        disable_grandpa: config.disable_grandpa,
        dev_key_seed: config.dev_key_seed,
        tracing_targets: config.tracing_targets,
        tracing_receiver: config.tracing_receiver,
        announce_block: config.announce_block,
        data_path: config.data_path,
        base_path: config.base_path,
        executor: ExecutorConfiguration {
            default_heap_pages: Some(pages),
            wasm_method: config.executor.wasm_method,
            max_runtime_instances: config.executor.max_runtime_instances,
            runtime_cache_size: config.executor.runtime_cache_size,
        },
        rpc: RpcConfiguration {
            addr: config.rpc.addr,
            max_connections: config.rpc.max_connections,
            cors: config.rpc.cors,
            methods: config.rpc.methods,
            max_request_size: config.rpc.max_request_size,
            max_response_size: config.rpc.max_response_size,
            id_provider: config.rpc.id_provider,
            max_subs_per_conn: config.rpc.max_subs_per_conn,
            port: config.rpc.port,
            message_buffer_capacity: config.rpc.message_buffer_capacity,
            batch_config: config.rpc.batch_config,
            rate_limit: config.rpc.rate_limit,
            rate_limit_whitelisted_ips: config.rpc.rate_limit_whitelisted_ips,
            rate_limit_trust_proxy_headers: config.rpc.rate_limit_trust_proxy_headers,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::extract_height;

    #[test]
    fn extracts_imported_height() {
        let line = "2026-03-28 18:49:24 🏆 Imported #2 (0xf014…f7e0 → 0x3e7e…9ace)";
        assert_eq!(extract_height(line, "Imported #"), Some(2));
    }

    #[test]
    fn extracts_finalized_height() {
        let line = "2026-03-28 18:51:39 💤 Idle (0 peers), best: #4, finalized #2, ⬇ 0 ⬆ 0";
        assert_eq!(extract_height(line, "finalized #"), Some(2));
    }

    #[test]
    fn ignores_missing_marker() {
        assert_eq!(
            extract_height("no block information here", "Imported #"),
            None
        );
    }
}
