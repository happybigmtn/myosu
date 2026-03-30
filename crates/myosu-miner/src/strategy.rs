use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use myosu_games_liars_dice::LiarsDiceSolver;
use myosu_games_liars_dice::LiarsDiceSolverError;
use myosu_games_liars_dice::decode_strategy_query as decode_liars_dice_strategy_query;
use myosu_games_liars_dice::encode_strategy_response as encode_liars_dice_strategy_response;
use myosu_games_liars_dice::recommended_edge as recommended_liars_dice_edge;
use myosu_games_poker::ArtifactCodecError;
use myosu_games_poker::PokerSolver;
use myosu_games_poker::PokerSolverError;
use myosu_games_poker::WireCodecError;
use myosu_games_poker::decode_strategy_query;
use myosu_games_poker::encode_strategy_response;
use myosu_games_poker::load_encoder_dir;
use myosu_games_poker::recommended_edge;
use thiserror::Error;
use tracing::info;

use crate::cli::Cli;
use crate::cli::GameSelection;

const LIARS_DICE_SOLVER_TREES: usize = 1 << 10;

/// Configuration for one bounded strategy-response batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrategyServePlan {
    pub game: GameSelection,
    pub encoder_dir: PathBuf,
    pub checkpoint_path: PathBuf,
    pub query_path: PathBuf,
    pub response_path: PathBuf,
}

/// Operator-visible summary of one bounded strategy-response batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrategyServeReport {
    pub game: GameSelection,
    pub response_path: PathBuf,
    pub action_count: usize,
    pub recommended_action: String,
}

/// Errors returned while preparing or executing a strategy-response batch.
#[derive(Debug, Error)]
pub enum StrategyServeError {
    /// Returned when a query batch is requested without the required encoder directory.
    #[error("--query-file requires --encoder-dir when --game poker")]
    MissingEncoderDir,

    /// Returned when a query batch has no checkpoint to load.
    #[error("--query-file requires --checkpoint or --train-iterations")]
    MissingCheckpoint,

    /// Returned when the encoder artifact directory fails to load.
    #[error("failed to load encoder directory `{path}`: {source}")]
    Encoder {
        path: String,
        #[source]
        source: ArtifactCodecError,
    },

    /// Returned when the query input file cannot be read.
    #[error("failed to read strategy query `{path}`: {source}")]
    ReadQuery {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Returned when the query input fails to decode from the wire format.
    #[error("failed to decode strategy query `{path}`: {source}")]
    DecodeQuery {
        path: String,
        #[source]
        source: WireCodecError,
    },

    /// Returned when the response output directory cannot be prepared.
    #[error("failed to prepare response directory `{path}`: {source}")]
    CreateResponseDir {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Returned when the response payload fails to encode.
    #[error("failed to encode strategy response: {0}")]
    EncodeResponse(#[source] WireCodecError),

    /// Returned when the response payload cannot be written.
    #[error("failed to write strategy response `{path}`: {source}")]
    WriteResponse {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Returned when the solver fails to load a checkpoint or answer a query.
    #[error("failed to answer strategy query: {0}")]
    Solver(#[from] PokerSolverError),

    /// Returned when the Liar's Dice solver fails to load a checkpoint or answer a query.
    #[error("failed to answer liar's dice strategy query: {0}")]
    LiarsDiceSolver(#[from] LiarsDiceSolverError),
}

/// Builds an optional single-query serving plan from the current CLI flags.
///
/// Args:
///     cli: Parsed miner CLI arguments.
///     checkpoint_hint: Fresh checkpoint path produced earlier in the same run.
///
/// Returns:
///     `Ok(None)` when no query batch was requested, otherwise the validated
///     `StrategyServePlan`.
pub fn strategy_serve_plan_from_cli(
    cli: &Cli,
    checkpoint_hint: Option<&Path>,
) -> Result<Option<StrategyServePlan>, StrategyServeError> {
    let Some(query_path) = cli.query_file.clone() else {
        return Ok(None);
    };

    let encoder_dir = match cli.game {
        GameSelection::Poker => cli
            .encoder_dir
            .clone()
            .ok_or(StrategyServeError::MissingEncoderDir)?,
        GameSelection::LiarsDice => cli.encoder_dir.clone().unwrap_or_default(),
    };

    let checkpoint_path = cli
        .checkpoint
        .clone()
        .or_else(|| checkpoint_hint.map(Path::to_path_buf))
        .ok_or(StrategyServeError::MissingCheckpoint)?;
    let response_path = cli
        .response_file
        .clone()
        .unwrap_or_else(|| cli.data_dir.join("responses").join("latest.bin"));

    Ok(Some(StrategyServePlan {
        game: cli.game,
        encoder_dir,
        checkpoint_path,
        query_path,
        response_path,
    }))
}

/// Loads a solver checkpoint and answers one wire-encoded strategy query.
///
/// Args:
///     plan: Validated serving plan from the miner CLI.
///
/// Returns:
///     A `StrategyServeReport` describing the saved response payload.
pub fn serve_strategy_once(
    plan: &StrategyServePlan,
) -> Result<StrategyServeReport, StrategyServeError> {
    let started_at = Instant::now();
    let query_bytes =
        fs::read(&plan.query_path).map_err(|source| StrategyServeError::ReadQuery {
            path: plan.query_path.display().to_string(),
            source,
        })?;
    let report = match plan.game {
        GameSelection::Poker => {
            let encoder = load_encoder_dir(&plan.encoder_dir).map_err(|source| {
                StrategyServeError::Encoder {
                    path: plan.encoder_dir.display().to_string(),
                    source,
                }
            })?;
            let solver = PokerSolver::load(&plan.checkpoint_path, encoder)?;
            serve_poker_strategy_once_with_solver(&solver, plan, &query_bytes)?
        }
        GameSelection::LiarsDice => {
            let solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::load(&plan.checkpoint_path)?;
            serve_liars_dice_strategy_once_with_solver(&solver, plan, &query_bytes)?
        }
    };
    info!(
        game = ?report.game,
        checkpoint_path = %plan.checkpoint_path.display(),
        query_path = %plan.query_path.display(),
        response_path = %report.response_path.display(),
        action_count = report.action_count,
        recommended_action = %report.recommended_action,
        elapsed_ms = started_at.elapsed().as_millis(),
        "served bounded miner strategy query"
    );
    Ok(report)
}

fn serve_poker_strategy_once_with_solver(
    solver: &PokerSolver,
    plan: &StrategyServePlan,
    query_bytes: &[u8],
) -> Result<StrategyServeReport, StrategyServeError> {
    let query =
        decode_strategy_query(query_bytes).map_err(|source| StrategyServeError::DecodeQuery {
            path: plan.query_path.display().to_string(),
            source,
        })?;
    let response = solver.answer(query);
    let response_bytes =
        encode_strategy_response(&response).map_err(StrategyServeError::EncodeResponse)?;

    let parent = plan
        .response_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent).map_err(|source| StrategyServeError::CreateResponseDir {
        path: parent.display().to_string(),
        source,
    })?;
    fs::write(&plan.response_path, response_bytes).map_err(|source| {
        StrategyServeError::WriteResponse {
            path: plan.response_path.display().to_string(),
            source,
        }
    })?;

    let recommended_action = recommended_edge(&response)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string());

    Ok(StrategyServeReport {
        game: GameSelection::Poker,
        response_path: plan.response_path.clone(),
        action_count: response.actions.len(),
        recommended_action,
    })
}

fn serve_liars_dice_strategy_once_with_solver(
    solver: &LiarsDiceSolver<LIARS_DICE_SOLVER_TREES>,
    plan: &StrategyServePlan,
    query_bytes: &[u8],
) -> Result<StrategyServeReport, StrategyServeError> {
    let query = decode_liars_dice_strategy_query(query_bytes).map_err(|source| {
        StrategyServeError::DecodeQuery {
            path: plan.query_path.display().to_string(),
            source: WireCodecError::Decode {
                context: "liar's dice strategy query",
                source: match source {
                    myosu_games_liars_dice::WireCodecError::Decode { source, .. } => source,
                    myosu_games_liars_dice::WireCodecError::Encode { source, .. } => source,
                },
            },
        }
    })?;
    let response = solver.answer(query);
    let response_bytes = encode_liars_dice_strategy_response(&response).map_err(|source| {
        StrategyServeError::EncodeResponse(match source {
            myosu_games_liars_dice::WireCodecError::Encode { source, .. } => {
                WireCodecError::Encode {
                    context: "liar's dice strategy response",
                    source,
                }
            }
            myosu_games_liars_dice::WireCodecError::Decode { source, .. } => {
                WireCodecError::Decode {
                    context: "liar's dice strategy response",
                    source,
                }
            }
        })
    })?;

    let parent = plan
        .response_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&parent).map_err(|source| StrategyServeError::CreateResponseDir {
        path: parent.display().to_string(),
        source,
    })?;
    fs::write(&plan.response_path, response_bytes).map_err(|source| {
        StrategyServeError::WriteResponse {
            path: plan.response_path.display().to_string(),
            source,
        }
    })?;

    let recommended_action = recommended_liars_dice_edge(&response)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string());

    Ok(StrategyServeReport {
        game: GameSelection::LiarsDice,
        response_path: plan.response_path.clone(),
        action_count: response.actions.len(),
        recommended_action,
    })
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use myosu_games::CfrGame;
    use myosu_games_poker::NlheInfoKey;
    use myosu_games_poker::NlheStrategyQuery;
    use myosu_games_poker::RbpNlheEncoder;
    use myosu_games_poker::decode_strategy_response;
    use myosu_games_poker::encode_strategy_query;

    use super::*;

    #[test]
    fn serve_plan_requires_checkpoint_or_training_output() {
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
            response_file: None,
        };

        let error = strategy_serve_plan_from_cli(&cli, None)
            .expect_err("query batch should require checkpoint or fresh training output");
        assert!(matches!(error, StrategyServeError::MissingCheckpoint));
    }

    #[test]
    fn serve_strategy_once_writes_wire_response() {
        let root = unique_temp_root();
        let query_path = root.join("query.bin");
        let response_path = root.join("responses").join("latest.bin");
        let query = NlheStrategyQuery::new(NlheInfoKey {
            subgame: 0,
            bucket: 0,
            choices: 0,
        });
        let query_bytes = encode_strategy_query(&query).expect("query should encode");
        let solver = PokerSolver::new(RbpNlheEncoder::default());
        let plan = StrategyServePlan {
            game: GameSelection::Poker,
            encoder_dir: PathBuf::new(),
            checkpoint_path: root.join("checkpoint.bin"),
            query_path: query_path.clone(),
            response_path: response_path.clone(),
        };

        fs::create_dir_all(&root).expect("temp root should exist");
        fs::write(&query_path, &query_bytes).expect("query file should write");

        let report = serve_poker_strategy_once_with_solver(&solver, &plan, &query_bytes)
            .expect("serve batch should succeed");
        let response_bytes = fs::read(&response_path).expect("response file should be written");
        let response = decode_strategy_response(&response_bytes).expect("response should decode");

        assert_eq!(report.response_path, response_path);
        assert_eq!(report.game, GameSelection::Poker);
        assert_eq!(report.action_count, response.actions.len());
        assert!(report.response_path.is_file());
        assert!(response.is_valid());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serve_strategy_once_reports_bad_query_bytes_cleanly() {
        let root = unique_temp_root();
        let query_path = root.join("query.bin");
        let solver = PokerSolver::new(RbpNlheEncoder::default());
        let plan = StrategyServePlan {
            game: GameSelection::Poker,
            encoder_dir: PathBuf::new(),
            checkpoint_path: root.join("checkpoint.bin"),
            query_path: query_path.clone(),
            response_path: root.join("responses").join("latest.bin"),
        };

        let error = serve_poker_strategy_once_with_solver(&solver, &plan, &[0xff, 0x00, 0x01])
            .expect_err("invalid query should fail");

        match error {
            StrategyServeError::DecodeQuery { path, .. } => {
                assert_eq!(path, query_path.display().to_string());
            }
            other => panic!("unexpected serve error: {other}"),
        }
    }

    #[test]
    fn liars_dice_serve_plan_does_not_require_encoder_dir() {
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
            checkpoint: Some(PathBuf::from("/tmp/checkpoint.bin")),
            train_iterations: 0,
            query_file: Some(PathBuf::from("/tmp/query.bin")),
            response_file: None,
        };

        let plan = strategy_serve_plan_from_cli(&cli, None)
            .expect("liar's dice serve plan should build")
            .expect("query serving should be requested");

        assert_eq!(plan.game, GameSelection::LiarsDice);
        assert_eq!(plan.encoder_dir, PathBuf::new());
    }

    #[test]
    fn serve_liars_dice_strategy_once_writes_wire_response() {
        let root = unique_temp_root();
        let query_path = root.join("query.bin");
        let response_path = root.join("responses").join("latest.bin");
        let opening = myosu_games_liars_dice::LiarsDiceGame::root()
            .apply(myosu_games_liars_dice::LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let query = myosu_games_liars_dice::LiarsDiceStrategyQuery::new(
            opening
                .info()
                .expect("opening player turn should expose info"),
        );
        let query_bytes =
            myosu_games_liars_dice::encode_strategy_query(&query).expect("query should encode");
        let mut solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::new();
        solver.train(8).expect("training should succeed");
        let plan = StrategyServePlan {
            game: GameSelection::LiarsDice,
            encoder_dir: PathBuf::new(),
            checkpoint_path: root.join("checkpoint.bin"),
            query_path: query_path.clone(),
            response_path: response_path.clone(),
        };

        fs::create_dir_all(&root).expect("temp root should exist");
        fs::write(&query_path, &query_bytes).expect("query file should write");

        let report = serve_liars_dice_strategy_once_with_solver(&solver, &plan, &query_bytes)
            .expect("serve batch should succeed");
        let response_bytes = fs::read(&response_path).expect("response file should be written");
        let response = myosu_games_liars_dice::decode_strategy_response(&response_bytes)
            .expect("response should decode");

        assert_eq!(report.game, GameSelection::LiarsDice);
        assert_eq!(report.response_path, response_path);
        assert_eq!(report.action_count, response.actions.len());
        assert!(report.response_path.is_file());
        assert!(response.is_valid());

        let _ = fs::remove_dir_all(root);
    }

    fn unique_temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-miner-serve-{nanos}"))
    }
}
