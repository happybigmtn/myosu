use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use myosu_games_kuhn::{KuhnSolver, KuhnSolverError};
use myosu_games_liars_dice::{LiarsDiceSolver, LiarsDiceSolverError};
use myosu_games_poker::ArtifactCodecError;
use myosu_games_poker::NlheEncoderArtifactSummary;
use myosu_games_poker::NlheScenarioBenchmarkError;
use myosu_games_poker::PokerSolver;
use myosu_games_poker::PokerSolverError;
use myosu_games_poker::benchmark_against_bootstrap_reference;
use myosu_games_poker::load_encoder_bundle;
use myosu_games_portfolio::{PortfolioSolver, PortfolioSolverError};
use thiserror::Error;
use tracing::info;

use crate::axon::AxonServeError;
use crate::chain::ChainProbeError;
use crate::cli::Cli;
use crate::cli::GameSelection;
use crate::strategy::StrategyServeError;

const LIARS_DICE_SOLVER_TREES: usize = 1 << 10;

/// Startup error returned by the bootstrap miner binary.
#[derive(Debug, Error)]
pub enum MinerBootstrapError {
    /// Returned when the operator key source cannot be resolved.
    #[error("{0}")]
    Key(#[from] myosu_keys::KeyError),

    /// Returned when the chain connectivity probe fails.
    #[error("{0}")]
    Chain(#[from] ChainProbeError),

    /// Returned when the miner cannot complete an on-chain bootstrap action.
    #[error("{0}")]
    ChainAction(#[from] crate::chain::ChainActionError),

    /// Returned when the bounded training batch cannot start or complete.
    #[error("{0}")]
    Training(#[from] TrainingBootstrapError),

    /// Returned when the bounded strategy-response batch cannot start or complete.
    #[error("{0}")]
    Strategy(#[from] StrategyServeError),

    /// Returned when the live HTTP axon cannot start or serve requests.
    #[error("{0}")]
    Axon(#[from] AxonServeError),
}

/// Configuration for one bounded solver-training batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingPlan {
    pub game: GameSelection,
    pub encoder_dir: PathBuf,
    pub checkpoint: Option<PathBuf>,
    pub checkpoint_output: PathBuf,
    pub iterations: usize,
}

/// Operator-visible summary of a bounded training batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingRunReport {
    pub game: GameSelection,
    pub checkpoint_path: PathBuf,
    pub epochs: usize,
    pub exploitability: String,
    pub quality_summary: String,
}

/// Errors returned while preparing or executing the bounded training batch.
#[derive(Debug, Error)]
pub enum TrainingBootstrapError {
    /// Returned when the CLI requests training without the required encoder directory.
    #[error("--train-iterations requires --encoder-dir when --game poker")]
    MissingEncoderDir,

    /// Returned when a checkpoint is requested without an encoder directory.
    #[error("--checkpoint requires --encoder-dir when --game poker")]
    CheckpointWithoutEncoderDir,

    /// Returned when the encoder artifact directory fails to load.
    #[error("failed to load encoder directory `{path}`: {source}")]
    Encoder {
        path: String,
        #[source]
        source: ArtifactCodecError,
    },

    /// Returned when positive-iteration poker training is requested on incomplete artifacts.
    #[error(
        "poker encoder directory `{path}` is not postflop-complete: complete={complete_streets} sampled={sampled_streets} missing={missing_streets} coverage={coverage}; `--train-iterations {requested_iterations}` requires full flop/turn/river coverage"
    )]
    IncompletePokerArtifacts {
        path: String,
        requested_iterations: usize,
        complete_streets: String,
        sampled_streets: String,
        missing_streets: String,
        coverage: String,
    },

    /// Returned when the solver fails to load, train, or save a checkpoint.
    #[error("failed to run bounded training batch: {0}")]
    Solver(#[from] PokerSolverError),

    /// Returned when the Liar's Dice solver fails to load, train, or save a checkpoint.
    #[error("failed to run bounded liar's dice training batch: {0}")]
    LiarsDiceSolver(#[from] LiarsDiceSolverError),

    /// Returned when the Kuhn exact solver fails to load or save a checkpoint.
    #[error("failed to run bounded Kuhn training batch: {0}")]
    KuhnSolver(#[from] KuhnSolverError),

    /// Returned when a research portfolio solver fails to load or save a checkpoint.
    #[error("failed to run bounded research portfolio training batch: {0}")]
    PortfolioSolver(#[from] PortfolioSolverError),

    /// Returned when portfolio training is reached for a non-portfolio game.
    #[error("game `{game:?}` is not a research portfolio game")]
    UnsupportedPortfolioGame { game: GameSelection },

    /// Returned when the miner cannot prepare the checkpoint output directory.
    #[error("failed to prepare checkpoint directory `{path}`: {source}")]
    CreateCheckpointDir {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Builds an optional bounded-training plan from the current CLI flags.
///
/// Args:
///     cli: Parsed miner CLI arguments.
///
/// Returns:
///     `Ok(None)` when no training batch was requested, otherwise the
///     validated `TrainingPlan`.
pub fn training_plan_from_cli(cli: &Cli) -> Result<Option<TrainingPlan>, TrainingBootstrapError> {
    if cli.game == GameSelection::Poker && cli.checkpoint.is_some() && cli.encoder_dir.is_none() {
        return Err(TrainingBootstrapError::CheckpointWithoutEncoderDir);
    }
    let should_bootstrap_checkpoint = cli.query_file.is_some() && cli.checkpoint.is_none();
    if cli.train_iterations == 0 && !should_bootstrap_checkpoint {
        return Ok(None);
    }

    let encoder_dir = if cli.game.portfolio_game().is_some() {
        cli.encoder_dir.clone().unwrap_or_default()
    } else {
        match cli.game {
            GameSelection::Poker => cli
                .encoder_dir
                .clone()
                .ok_or(TrainingBootstrapError::MissingEncoderDir)?,
            GameSelection::Kuhn => cli.encoder_dir.clone().unwrap_or_default(),
            GameSelection::LiarsDice => cli.encoder_dir.clone().unwrap_or_default(),
            _ => cli.encoder_dir.clone().unwrap_or_default(),
        }
    };

    let checkpoint_output = cli
        .checkpoint
        .clone()
        .unwrap_or_else(|| cli.data_dir.join("checkpoints").join("latest.bin"));

    Ok(Some(TrainingPlan {
        game: cli.game,
        encoder_dir,
        checkpoint: cli.checkpoint.clone(),
        checkpoint_output,
        iterations: cli.train_iterations,
    }))
}

/// Loads a solver from the requested artifact set and runs a bounded batch.
///
/// Args:
///     plan: Validated training plan from the miner CLI.
///
/// Returns:
///     A `TrainingRunReport` describing the saved checkpoint and final epoch.
pub fn run_training_batch(
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let started_at = Instant::now();
    let report = if plan.game.portfolio_game().is_some() {
        run_portfolio_training_batch(plan)?
    } else {
        match plan.game {
            GameSelection::Poker => run_poker_training_batch(plan)?,
            GameSelection::Kuhn => run_kuhn_training_batch(plan)?,
            GameSelection::LiarsDice => run_liars_dice_training_batch(plan)?,
            _ => run_portfolio_training_batch(plan)?,
        }
    };
    let elapsed = started_at.elapsed();
    let iterations_per_second = if elapsed.is_zero() {
        0.0
    } else {
        plan.iterations as f64 / elapsed.as_secs_f64()
    };
    info!(
        game = ?report.game,
        checkpoint_path = %report.checkpoint_path.display(),
        iterations = plan.iterations,
        epochs = report.epochs,
        exploitability = %report.exploitability,
        quality_summary = %report.quality_summary,
        elapsed_ms = elapsed.as_millis(),
        iterations_per_second,
        "completed bounded miner training batch"
    );

    Ok(report)
}

fn run_poker_training_batch(
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let artifact_bundle = load_encoder_bundle(&plan.encoder_dir).map_err(|source| {
        TrainingBootstrapError::Encoder {
            path: plan.encoder_dir.display().to_string(),
            source,
        }
    })?;
    let artifact_summary = artifact_bundle.summary();
    if plan.iterations > 0 && !artifact_summary.postflop_complete {
        return Err(TrainingBootstrapError::IncompletePokerArtifacts {
            path: plan.encoder_dir.display().to_string(),
            requested_iterations: plan.iterations,
            complete_streets: artifact_summary.complete_streets_token(),
            sampled_streets: artifact_summary.sampled_streets_token(),
            missing_streets: artifact_summary.missing_streets_token(),
            coverage: artifact_summary.coverage_token(),
        });
    }

    let mut solver = if let Some(checkpoint) = plan.checkpoint.as_ref() {
        PokerSolver::load(checkpoint, artifact_bundle.encoder)?
    } else {
        PokerSolver::new(artifact_bundle.encoder)
    };

    let mut report = run_training_batch_with_poker_solver(&mut solver, plan)?;
    append_poker_artifact_summary(&mut report, &artifact_summary);
    Ok(report)
}

fn run_liars_dice_training_batch(
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let mut solver = if let Some(checkpoint) = plan.checkpoint.as_ref() {
        LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::load(checkpoint)?
    } else {
        LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::new()
    };

    run_training_batch_with_liars_dice_solver(&mut solver, plan)
}

fn run_kuhn_training_batch(
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let mut solver = if let Some(checkpoint) = plan.checkpoint.as_ref() {
        KuhnSolver::load(checkpoint)?
    } else {
        KuhnSolver::new()
    };

    run_training_batch_with_kuhn_solver(&mut solver, plan)
}

fn run_portfolio_training_batch(
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let game = plan
        .game
        .portfolio_game()
        .ok_or(TrainingBootstrapError::UnsupportedPortfolioGame { game: plan.game })?;
    let mut solver = if let Some(checkpoint) = plan.checkpoint.as_ref() {
        PortfolioSolver::load(checkpoint)?
    } else {
        PortfolioSolver::for_game(game)
    };
    solver.ensure_supports(game)?;

    run_training_batch_with_portfolio_solver(&mut solver, plan)
}

fn run_training_batch_with_poker_solver(
    solver: &mut PokerSolver,
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let training = solver.train_select_best(plan.iterations)?;

    let parent = plan
        .checkpoint_output
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent).map_err(|source| TrainingBootstrapError::CreateCheckpointDir {
        path: parent.display().to_string(),
        source,
    })?;
    solver.save(&plan.checkpoint_output)?;

    let mut report = TrainingRunReport {
        game: GameSelection::Poker,
        checkpoint_path: plan.checkpoint_output.clone(),
        epochs: solver.epochs(),
        exploitability: training
            .selected_exploitability
            .map(|value| format!("{value:.6}"))
            .unwrap_or_else(|| match solver.exploitability() {
                Ok(value) => value.to_string(),
                Err(error) => format!("unavailable: {error}"),
            }),
        quality_summary: format!(
            "engine_tier=dedicated-mccfr engine_family=robopoker-nlhe checkpoint_selection={} trained_epochs={} selected_epochs={}",
            training.checkpoint_selection, training.end_epochs, training.selected_epochs,
        ),
    };
    append_poker_benchmark_summary(&mut report, solver);
    Ok(report)
}

fn append_poker_artifact_summary(
    report: &mut TrainingRunReport,
    artifact_summary: &NlheEncoderArtifactSummary,
) {
    report.quality_summary = format!(
        "{} artifact_streets={} complete_streets={} sampled_streets={} missing_streets={} coverage={} postflop_complete={} preflop_entries={} total_entries={}",
        report.quality_summary,
        artifact_summary.available_streets_token(),
        artifact_summary.complete_streets_token(),
        artifact_summary.sampled_streets_token(),
        artifact_summary.missing_streets_token(),
        artifact_summary.coverage_token(),
        artifact_summary.postflop_complete,
        artifact_summary.preflop_entries(),
        artifact_summary.total_entries,
    );
}

fn append_poker_benchmark_summary(report: &mut TrainingRunReport, solver: &PokerSolver) {
    match benchmark_against_bootstrap_reference(solver) {
        Ok(benchmark) => {
            report.quality_summary = format!(
                "{} benchmark_surface=repo-owned-reference-pack benchmark_scenarios={} benchmark_unique_queries={} benchmark_mean_l1_distance={:.6} benchmark_action_agreement={:.6}",
                report.quality_summary,
                benchmark.scenario_count,
                benchmark.unique_query_count,
                benchmark.mean_l1_distance,
                benchmark.recommendation_agreement(),
            );
        }
        Err(error) => {
            report.quality_summary = format!(
                "{} benchmark_surface=unavailable benchmark_reason={} benchmark_scenarios=0 benchmark_unique_queries=0 benchmark_mean_l1_distance=unavailable benchmark_action_agreement=unavailable",
                report.quality_summary,
                poker_benchmark_reason(&error),
            );
        }
    }
}

fn poker_benchmark_reason(error: &NlheScenarioBenchmarkError) -> &'static str {
    match error {
        NlheScenarioBenchmarkError::Request(_) => "scenario-pack-query-unavailable",
        NlheScenarioBenchmarkError::Solver(_) => "scenario-pack-answer-unavailable",
        NlheScenarioBenchmarkError::EmptyChoices { .. } => "scenario-pack-empty-choices",
    }
}

fn run_training_batch_with_liars_dice_solver(
    solver: &mut LiarsDiceSolver<LIARS_DICE_SOLVER_TREES>,
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let training = solver.train_select_best(plan.iterations)?;

    let parent = plan
        .checkpoint_output
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent).map_err(|source| TrainingBootstrapError::CreateCheckpointDir {
        path: parent.display().to_string(),
        source,
    })?;
    solver.save(&plan.checkpoint_output)?;

    Ok(TrainingRunReport {
        game: GameSelection::LiarsDice,
        checkpoint_path: plan.checkpoint_output.clone(),
        epochs: solver.epochs(),
        exploitability: format!("{:.6}", training.selected_exploitability),
        quality_summary: format!(
            "engine_tier=dedicated-mccfr engine_family=liars-dice-cfr checkpoint_selection=exact-exploitability trained_epochs={} selected_epochs={}",
            training.end_epochs, training.selected_epochs,
        ),
    })
}

fn run_training_batch_with_kuhn_solver(
    solver: &mut KuhnSolver,
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    solver.train(plan.iterations);

    let parent = plan
        .checkpoint_output
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent).map_err(|source| TrainingBootstrapError::CreateCheckpointDir {
        path: parent.display().to_string(),
        source,
    })?;
    solver.save(&plan.checkpoint_output)?;

    Ok(TrainingRunReport {
        game: GameSelection::Kuhn,
        checkpoint_path: plan.checkpoint_output.clone(),
        epochs: solver.epochs(),
        exploitability: "0".to_string(),
        quality_summary: "engine_tier=exact engine_family=kuhn-poker-exact".to_string(),
    })
}

fn run_training_batch_with_portfolio_solver(
    solver: &mut PortfolioSolver,
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let game = plan
        .game
        .portfolio_game()
        .ok_or(TrainingBootstrapError::UnsupportedPortfolioGame { game: plan.game })?;
    solver.ensure_supports(game)?;
    solver.train(plan.iterations);

    let parent = plan
        .checkpoint_output
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent).map_err(|source| TrainingBootstrapError::CreateCheckpointDir {
        path: parent.display().to_string(),
        source,
    })?;
    solver.save(&plan.checkpoint_output)?;

    let quality_summary = portfolio_quality_summary(solver, game)?;
    Ok(TrainingRunReport {
        game: plan.game,
        checkpoint_path: plan.checkpoint_output.clone(),
        epochs: solver.epochs(),
        exploitability: "not-applicable: portfolio quality summary".to_string(),
        quality_summary,
    })
}

fn portfolio_quality_summary(
    solver: &PortfolioSolver,
    game: myosu_games_portfolio::ResearchGame,
) -> Result<String, PortfolioSolverError> {
    let query = PortfolioSolver::strength_query(game)?;
    let report = solver.strength_quality(query)?;

    Ok(format!(
        "engine_tier={} engine_family={} challenge_id={} score={:.6} legal_actions={} deterministic={}",
        report.engine_tier.as_str(),
        shell_token(&report.engine_family),
        report.challenge_id,
        report.score,
        report.legal_action_count,
        report.deterministic,
    ))
}

fn shell_token(value: &str) -> String {
    value.replace(' ', "_")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::collections::BTreeMap;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use rbp_cards::Isomorphism;
    use rbp_cards::Observation;
    use rbp_gameplay::Abstraction;

    use super::*;
    use crate::cli::Cli;
    use crate::cli::GameSelection;

    #[test]
    fn training_plan_requires_encoder_dir_when_iterations_requested() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Alice".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            port: 8080,
            register: false,
            serve_axon: false,
            serve_http: false,
            data_dir: PathBuf::from("/tmp/miner-data"),
            game: GameSelection::Poker,
            encoder_dir: None,
            checkpoint: None,
            train_iterations: 128,
            query_file: None,
            response_file: None,
        };

        let error = training_plan_from_cli(&cli).expect_err("training should require encoder dir");
        assert!(matches!(error, TrainingBootstrapError::MissingEncoderDir));
    }

    #[test]
    fn training_plan_bootstraps_checkpoint_for_query_without_iterations() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Alice".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            port: 8080,
            register: false,
            serve_axon: false,
            serve_http: false,
            data_dir: PathBuf::from("/tmp/miner-data"),
            game: GameSelection::Poker,
            encoder_dir: Some(PathBuf::from("/tmp/encoder")),
            checkpoint: None,
            train_iterations: 0,
            query_file: Some(PathBuf::from("/tmp/query.bin")),
            response_file: Some(PathBuf::from("/tmp/response.bin")),
        };

        let plan = training_plan_from_cli(&cli)
            .expect("query bootstrap should build training plan")
            .expect("query bootstrap should emit zero-epoch checkpoint");

        assert_eq!(plan.iterations, 0);
        assert_eq!(plan.game, GameSelection::Poker);
        assert_eq!(plan.checkpoint, None);
        assert_eq!(
            plan.checkpoint_output,
            PathBuf::from("/tmp/miner-data/checkpoints/latest.bin")
        );
    }

    #[test]
    fn run_training_batch_saves_checkpoint_without_iterations() {
        let root = unique_temp_root();
        let checkpoint = root.join("checkpoints").join("latest.bin");
        let mut solver = PokerSolver::new(sample_encoder());
        let plan = TrainingPlan {
            game: GameSelection::Poker,
            encoder_dir: PathBuf::new(),
            checkpoint: None,
            checkpoint_output: checkpoint.clone(),
            iterations: 0,
        };

        let report = run_training_batch_with_poker_solver(&mut solver, &plan)
            .expect("training batch should succeed");

        assert_eq!(report.game, GameSelection::Poker);
        assert_eq!(report.checkpoint_path, checkpoint);
        assert_eq!(report.epochs, 0);
        assert!(report.exploitability.starts_with("unavailable: "));
        assert!(report.checkpoint_path.is_file());
        assert!(
            report
                .quality_summary
                .contains("checkpoint_selection=last-iterate")
        );
        assert!(
            report
                .quality_summary
                .contains("benchmark_surface=unavailable")
        );
        assert!(
            report
                .quality_summary
                .contains("benchmark_reason=scenario-pack-query-unavailable")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_training_batch_reports_sparse_encoder_failure_cleanly() {
        let root = unique_temp_root();
        let checkpoint = root.join("checkpoints").join("latest.bin");
        let mut solver = PokerSolver::new(sample_encoder());
        let plan = TrainingPlan {
            game: GameSelection::Poker,
            encoder_dir: PathBuf::new(),
            checkpoint: None,
            checkpoint_output: checkpoint,
            iterations: 1,
        };

        let error = run_training_batch_with_poker_solver(&mut solver, &plan)
            .expect_err("sparse encoder should fail cleanly");

        match error {
            TrainingBootstrapError::Solver(PokerSolverError::UpstreamPanic {
                operation,
                message,
            }) => {
                assert_eq!(operation, "solver step");
                assert!(message.contains("isomorphism not found"));
            }
            other => panic!("unexpected training error: {other}"),
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_poker_training_batch_rejects_incomplete_artifacts_before_training() {
        let root = unique_temp_root();
        let encoder_dir = root.join("encoder");
        myosu_games_poker::write_encoder_dir(
            &encoder_dir,
            BTreeMap::from([(
                myosu_games_poker::NlheAbstractionStreet::Preflop,
                BTreeMap::from([(
                    Isomorphism::from(
                        Observation::try_from("AcKh").expect("observation should parse"),
                    ),
                    Abstraction::from(42_i16),
                )]),
            )]),
        )
        .expect("encoder dir should write");

        let plan = TrainingPlan {
            game: GameSelection::Poker,
            encoder_dir: encoder_dir.clone(),
            checkpoint: None,
            checkpoint_output: root.join("checkpoints").join("latest.bin"),
            iterations: 1,
        };

        let error = run_poker_training_batch(&plan)
            .expect_err("incomplete artifacts should be rejected before training");

        match error {
            TrainingBootstrapError::IncompletePokerArtifacts {
                path,
                requested_iterations,
                complete_streets,
                sampled_streets,
                missing_streets,
                coverage,
            } => {
                assert_eq!(path, encoder_dir.display().to_string());
                assert_eq!(requested_iterations, 1);
                assert_eq!(complete_streets, "none");
                assert_eq!(sampled_streets, "preflop");
                assert_eq!(missing_streets, "flop,turn,river");
                assert_eq!(
                    coverage,
                    "preflop=1/169,flop=0/1286792,turn=0/13960050,river=0/123156254"
                );
            }
            other => panic!("unexpected training error: {other}"),
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_poker_training_batch_reports_artifact_metadata_in_quality_summary() {
        let root = unique_temp_root();
        let encoder_dir = root.join("encoder");
        myosu_games_poker::write_encoder_dir(
            &encoder_dir,
            BTreeMap::from([(
                myosu_games_poker::NlheAbstractionStreet::Preflop,
                BTreeMap::from([(
                    Isomorphism::from(
                        Observation::try_from("AcKh").expect("observation should parse"),
                    ),
                    Abstraction::from(42_i16),
                )]),
            )]),
        )
        .expect("encoder dir should write");

        let plan = TrainingPlan {
            game: GameSelection::Poker,
            encoder_dir: encoder_dir.clone(),
            checkpoint: None,
            checkpoint_output: root.join("checkpoints").join("latest.bin"),
            iterations: 0,
        };

        let report =
            run_poker_training_batch(&plan).expect("zero-iteration training should succeed");

        assert!(report.quality_summary.contains("artifact_streets=preflop"));
        assert!(report.quality_summary.contains("complete_streets=none"));
        assert!(report.quality_summary.contains("sampled_streets=preflop"));
        assert!(
            report
                .quality_summary
                .contains("missing_streets=flop,turn,river")
        );
        assert!(
            report.quality_summary.contains(
                "coverage=preflop=1/169,flop=0/1286792,turn=0/13960050,river=0/123156254"
            )
        );
        assert!(report.quality_summary.contains("postflop_complete=false"));
        assert!(report.quality_summary.contains("preflop_entries=1"));
        assert!(report.quality_summary.contains("total_entries=1"));
        assert!(
            report
                .quality_summary
                .contains("benchmark_surface=unavailable")
        );
        assert!(
            report
                .quality_summary
                .contains("benchmark_reason=scenario-pack-query-unavailable")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_poker_training_batch_reports_reference_pack_benchmark_for_bootstrap_artifacts() {
        let root = unique_temp_root();
        let encoder_dir = root.join("encoder");
        myosu_games_poker::write_encoder_dir(
            &encoder_dir,
            myosu_games_poker::bootstrap_encoder_streets(),
        )
        .expect("encoder dir should write");

        let plan = TrainingPlan {
            game: GameSelection::Poker,
            encoder_dir: encoder_dir.clone(),
            checkpoint: None,
            checkpoint_output: root.join("checkpoints").join("latest.bin"),
            iterations: 0,
        };

        let report =
            run_poker_training_batch(&plan).expect("zero-iteration training should succeed");

        assert!(
            report
                .quality_summary
                .contains("benchmark_surface=repo-owned-reference-pack")
        );
        assert!(report.quality_summary.contains("benchmark_scenarios=80"));
        assert!(
            report
                .quality_summary
                .contains("benchmark_unique_queries=73")
        );
        assert!(
            report
                .quality_summary
                .contains("benchmark_mean_l1_distance=")
        );
        assert!(
            report
                .quality_summary
                .contains("benchmark_action_agreement=")
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn liars_dice_training_plan_does_not_require_encoder_dir() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Alice".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            port: 8080,
            register: false,
            serve_axon: false,
            serve_http: false,
            data_dir: PathBuf::from("/tmp/miner-data"),
            game: GameSelection::LiarsDice,
            encoder_dir: None,
            checkpoint: None,
            train_iterations: 8,
            query_file: None,
            response_file: None,
        };

        let plan = training_plan_from_cli(&cli)
            .expect("liar's dice training should build without encoder dir")
            .expect("training should be requested");

        assert_eq!(plan.game, GameSelection::LiarsDice);
        assert_eq!(plan.encoder_dir, PathBuf::new());
    }

    #[test]
    fn kuhn_training_plan_does_not_require_encoder_dir() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Alice".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            port: 8080,
            register: false,
            serve_axon: false,
            serve_http: false,
            data_dir: PathBuf::from("/tmp/miner-data"),
            game: GameSelection::Kuhn,
            encoder_dir: None,
            checkpoint: None,
            train_iterations: 8,
            query_file: None,
            response_file: None,
        };

        let plan = training_plan_from_cli(&cli)
            .expect("kuhn training should build without encoder dir")
            .expect("training should be requested");

        assert_eq!(plan.game, GameSelection::Kuhn);
        assert_eq!(plan.encoder_dir, PathBuf::new());
    }

    #[test]
    fn portfolio_training_plan_does_not_require_encoder_dir() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Alice".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            port: 8080,
            register: false,
            serve_axon: false,
            serve_http: false,
            data_dir: PathBuf::from("/tmp/miner-data"),
            game: GameSelection::Bridge,
            encoder_dir: None,
            checkpoint: None,
            train_iterations: 8,
            query_file: None,
            response_file: None,
        };

        let plan = training_plan_from_cli(&cli)
            .expect("portfolio training should build without encoder dir")
            .expect("training should be requested");

        assert_eq!(plan.game, GameSelection::Bridge);
        assert_eq!(plan.encoder_dir, PathBuf::new());
    }

    #[test]
    fn run_liars_dice_training_batch_saves_checkpoint() {
        let root = unique_temp_root();
        let checkpoint = root.join("checkpoints").join("latest.bin");
        let mut solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::new();
        let plan = TrainingPlan {
            game: GameSelection::LiarsDice,
            encoder_dir: PathBuf::new(),
            checkpoint: None,
            checkpoint_output: checkpoint.clone(),
            iterations: 8,
        };

        let report = run_training_batch_with_liars_dice_solver(&mut solver, &plan)
            .expect("liar's dice training batch should succeed");

        assert_eq!(report.game, GameSelection::LiarsDice);
        assert_eq!(report.checkpoint_path, checkpoint);
        assert!(report.epochs <= 8);
        assert!(report.checkpoint_path.is_file());
        assert!(
            report
                .quality_summary
                .contains("checkpoint_selection=exact-exploitability")
        );

        let restored = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::load(&report.checkpoint_path)
            .expect("checkpoint should load");
        assert_eq!(restored.epochs(), report.epochs);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_kuhn_training_batch_saves_checkpoint() {
        let root = unique_temp_root();
        let checkpoint = root.join("checkpoints").join("latest.bin");
        let mut solver = KuhnSolver::new();
        let plan = TrainingPlan {
            game: GameSelection::Kuhn,
            encoder_dir: PathBuf::new(),
            checkpoint: None,
            checkpoint_output: checkpoint.clone(),
            iterations: 8,
        };

        let report = run_training_batch_with_kuhn_solver(&mut solver, &plan)
            .expect("kuhn training batch should succeed");

        assert_eq!(report.game, GameSelection::Kuhn);
        assert_eq!(report.checkpoint_path, checkpoint);
        assert_eq!(report.epochs, 0);
        assert_eq!(report.exploitability, "0");
        assert!(report.checkpoint_path.is_file());

        let restored = KuhnSolver::load(&report.checkpoint_path).expect("checkpoint should load");
        assert_eq!(restored.profile(), solver.profile());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_portfolio_training_batch_saves_checkpoint_for_every_portfolio_game() {
        let root = unique_temp_root();

        for game_selection in GameSelection::PORTFOLIO_SELECTIONS {
            let research_game = game_selection
                .portfolio_game()
                .expect("portfolio selection should map to a research game");
            let checkpoint = root
                .join(research_game.slug())
                .join("checkpoints")
                .join("latest.bin");
            let mut solver = PortfolioSolver::for_game(research_game);
            let plan = TrainingPlan {
                game: game_selection,
                encoder_dir: PathBuf::new(),
                checkpoint: None,
                checkpoint_output: checkpoint.clone(),
                iterations: 8,
            };

            let report = run_training_batch_with_portfolio_solver(&mut solver, &plan)
                .expect("portfolio training batch should succeed");

            assert_eq!(report.game, game_selection);
            assert_eq!(report.checkpoint_path, checkpoint);
            assert_eq!(report.epochs, 8);
            assert_eq!(
                report.exploitability,
                "not-applicable: portfolio quality summary"
            );
            assert!(report.quality_summary.contains("engine_tier=rule-aware"));
            let challenge_id =
                myosu_games_portfolio::PortfolioSolver::strength_query(research_game)
                    .expect("portfolio game should expose typed challenge")
                    .info
                    .challenge
                    .spot()
                    .challenge_id
                    .clone();
            assert!(
                report
                    .quality_summary
                    .contains(&format!("challenge_id={challenge_id}"))
            );
            assert!(report.checkpoint_path.is_file());

            let restored =
                PortfolioSolver::load(&report.checkpoint_path).expect("checkpoint should load");
            assert_eq!(restored.epochs(), 8);
            assert_eq!(restored.game(), Some(research_game));
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn run_portfolio_training_batch_rejects_mismatched_checkpoint_scope() {
        let root = unique_temp_root();
        let checkpoint = root.join("bridge.bin");
        fs::create_dir_all(&root).expect("temp root should exist");
        PortfolioSolver::for_game(myosu_games_portfolio::ResearchGame::Bridge)
            .save(&checkpoint)
            .expect("checkpoint should save");
        let plan = TrainingPlan {
            game: GameSelection::Cribbage,
            encoder_dir: PathBuf::new(),
            checkpoint: Some(checkpoint),
            checkpoint_output: root.join("cribbage.bin"),
            iterations: 8,
        };

        let error = run_portfolio_training_batch(&plan)
            .expect_err("mismatched portfolio checkpoint should fail");

        assert!(matches!(
            error,
            TrainingBootstrapError::PortfolioSolver(PortfolioSolverError::GameMismatch {
                checkpoint_game: myosu_games_portfolio::ResearchGame::Bridge,
                query_game: myosu_games_portfolio::ResearchGame::Cribbage,
            })
        ));

        let _ = fs::remove_dir_all(root);
    }

    fn sample_encoder() -> rbp_nlhe::NlheEncoder {
        myosu_games_poker::encoder_from_lookup(BTreeMap::from([(
            Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
            Abstraction::from(42_i16),
        )]))
        .expect("encoder lookup should build")
    }

    fn unique_temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-miner-training-{nanos}"))
    }
}
