use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use myosu_games_liars_dice::LiarsDiceSolver;
use myosu_games_liars_dice::LiarsDiceSolverError;
use myosu_games_poker::ArtifactCodecError;
use myosu_games_poker::PokerSolver;
use myosu_games_poker::PokerSolverError;
use myosu_games_poker::load_encoder_dir;
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

    /// Returned when the solver fails to load, train, or save a checkpoint.
    #[error("failed to run bounded training batch: {0}")]
    Solver(#[from] PokerSolverError),

    /// Returned when the Liar's Dice solver fails to load, train, or save a checkpoint.
    #[error("failed to run bounded liar's dice training batch: {0}")]
    LiarsDiceSolver(#[from] LiarsDiceSolverError),

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

    let encoder_dir = match cli.game {
        GameSelection::Poker => cli
            .encoder_dir
            .clone()
            .ok_or(TrainingBootstrapError::MissingEncoderDir)?,
        GameSelection::LiarsDice => cli.encoder_dir.clone().unwrap_or_default(),
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
    let report = match plan.game {
        GameSelection::Poker => run_poker_training_batch(plan)?,
        GameSelection::LiarsDice => run_liars_dice_training_batch(plan)?,
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
        elapsed_ms = elapsed.as_millis(),
        iterations_per_second,
        "completed bounded miner training batch"
    );

    Ok(report)
}

fn run_poker_training_batch(
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    let encoder =
        load_encoder_dir(&plan.encoder_dir).map_err(|source| TrainingBootstrapError::Encoder {
            path: plan.encoder_dir.display().to_string(),
            source,
        })?;
    let mut solver = if let Some(checkpoint) = plan.checkpoint.as_ref() {
        PokerSolver::load(checkpoint, encoder)?
    } else {
        PokerSolver::new(encoder)
    };

    run_training_batch_with_poker_solver(&mut solver, plan)
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

fn run_training_batch_with_poker_solver(
    solver: &mut PokerSolver,
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    solver.train(plan.iterations)?;

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
        game: GameSelection::Poker,
        checkpoint_path: plan.checkpoint_output.clone(),
        epochs: solver.epochs(),
        exploitability: match solver.exploitability() {
            Ok(value) => value.to_string(),
            Err(error) => format!("unavailable: {error}"),
        },
    })
}

fn run_training_batch_with_liars_dice_solver(
    solver: &mut LiarsDiceSolver<LIARS_DICE_SOLVER_TREES>,
    plan: &TrainingPlan,
) -> Result<TrainingRunReport, TrainingBootstrapError> {
    solver.train(plan.iterations)?;

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
        exploitability: solver.exact_exploitability().to_string(),
    })
}

#[cfg(test)]
mod tests {
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
            key: "//Alice".to_string(),
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
            key: "//Alice".to_string(),
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
        assert!(report.checkpoint_path.is_file());

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
    fn liars_dice_training_plan_does_not_require_encoder_dir() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: "//Alice".to_string(),
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
        assert_eq!(report.epochs, 8);
        assert!(report.checkpoint_path.is_file());

        let restored = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::load(&report.checkpoint_path)
            .expect("checkpoint should load");
        assert_eq!(restored.epochs(), 8);

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
