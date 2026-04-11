use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use myosu_games_kuhn::KuhnSolver;
use myosu_games_kuhn::KuhnSolverError;
use myosu_games_kuhn::decode_strategy_query as decode_kuhn_strategy_query;
use myosu_games_kuhn::encode_strategy_response as encode_kuhn_strategy_response;
use myosu_games_kuhn::recommended_edge as recommended_kuhn_edge;
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
use myosu_games_portfolio::PortfolioSolver;
use myosu_games_portfolio::PortfolioSolverError;
use myosu_games_portfolio::PortfolioStrategyQuery;
use myosu_games_portfolio::PortfolioStrengthQuery;
use myosu_games_portfolio::decode_strategy_query as decode_portfolio_strategy_query;
use myosu_games_portfolio::decode_strength_query as decode_portfolio_strength_query;
use myosu_games_portfolio::encode_strategy_response as encode_portfolio_strategy_response;
use myosu_games_portfolio::recommended_action as recommended_portfolio_action;
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
    pub quality_summary: String,
}

#[derive(Clone, Debug, PartialEq)]
enum PortfolioQuery {
    Bootstrap(PortfolioStrategyQuery),
    Strength(PortfolioStrengthQuery),
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

    /// Returned when the Kuhn solver fails to load a checkpoint or answer a query.
    #[error("failed to answer Kuhn strategy query: {0}")]
    KuhnSolver(#[from] KuhnSolverError),

    /// Returned when a research portfolio solver fails to load a checkpoint or answer a query.
    #[error("failed to answer research portfolio strategy query: {0}")]
    PortfolioSolver(#[from] PortfolioSolverError),

    /// Returned when the query belongs to a different portfolio game than the CLI route.
    #[error("portfolio query game `{query_game}` does not match requested game `{requested_game}`")]
    PortfolioGameMismatch {
        requested_game: String,
        query_game: String,
    },
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

    let encoder_dir = if cli.game.portfolio_game().is_some() {
        cli.encoder_dir.clone().unwrap_or_default()
    } else {
        match cli.game {
            GameSelection::Poker => cli
                .encoder_dir
                .clone()
                .ok_or(StrategyServeError::MissingEncoderDir)?,
            GameSelection::Kuhn => cli.encoder_dir.clone().unwrap_or_default(),
            GameSelection::LiarsDice => cli.encoder_dir.clone().unwrap_or_default(),
            _ => cli.encoder_dir.clone().unwrap_or_default(),
        }
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
    let report = if plan.game.portfolio_game().is_some() {
        let solver = PortfolioSolver::load(&plan.checkpoint_path)?;
        serve_portfolio_strategy_once_with_solver(&solver, plan, &query_bytes)?
    } else {
        match plan.game {
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
            GameSelection::Kuhn => {
                let solver = KuhnSolver::load(&plan.checkpoint_path)?;
                serve_kuhn_strategy_once_with_solver(&solver, plan, &query_bytes)?
            }
            GameSelection::LiarsDice => {
                let solver =
                    LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::load(&plan.checkpoint_path)?;
                serve_liars_dice_strategy_once_with_solver(&solver, plan, &query_bytes)?
            }
            _ => {
                let solver = PortfolioSolver::load(&plan.checkpoint_path)?;
                serve_portfolio_strategy_once_with_solver(&solver, plan, &query_bytes)?
            }
        }
    };
    info!(
        game = ?report.game,
        checkpoint_path = %plan.checkpoint_path.display(),
        query_path = %plan.query_path.display(),
        response_path = %report.response_path.display(),
        action_count = report.action_count,
        recommended_action = %report.recommended_action,
        quality_summary = %report.quality_summary,
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
        quality_summary: "engine_tier=dedicated-mccfr engine_family=robopoker-nlhe".to_string(),
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
        quality_summary: "engine_tier=dedicated-mccfr engine_family=liars-dice-cfr".to_string(),
    })
}

fn serve_kuhn_strategy_once_with_solver(
    solver: &KuhnSolver,
    plan: &StrategyServePlan,
    query_bytes: &[u8],
) -> Result<StrategyServeReport, StrategyServeError> {
    let query = decode_kuhn_strategy_query(query_bytes).map_err(|source| {
        StrategyServeError::DecodeQuery {
            path: plan.query_path.display().to_string(),
            source: match source {
                myosu_games_kuhn::WireCodecError::Decode { source, .. } => WireCodecError::Decode {
                    context: "kuhn strategy query",
                    source,
                },
                myosu_games_kuhn::WireCodecError::Encode { source, .. } => WireCodecError::Encode {
                    context: "kuhn strategy query",
                    source,
                },
            },
        }
    })?;
    let response = solver.answer(query);
    let response_bytes = encode_kuhn_strategy_response(&response).map_err(|source| {
        StrategyServeError::EncodeResponse(match source {
            myosu_games_kuhn::WireCodecError::Encode { source, .. } => WireCodecError::Encode {
                context: "kuhn strategy response",
                source,
            },
            myosu_games_kuhn::WireCodecError::Decode { source, .. } => WireCodecError::Decode {
                context: "kuhn strategy response",
                source,
            },
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

    let recommended_action = recommended_kuhn_edge(&response)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string());

    Ok(StrategyServeReport {
        game: GameSelection::Kuhn,
        response_path: plan.response_path.clone(),
        action_count: response.actions.len(),
        recommended_action,
        quality_summary: "engine_tier=exact engine_family=kuhn-poker-exact".to_string(),
    })
}

fn serve_portfolio_strategy_once_with_solver(
    solver: &PortfolioSolver,
    plan: &StrategyServePlan,
    query_bytes: &[u8],
) -> Result<StrategyServeReport, StrategyServeError> {
    let query = decode_portfolio_query(plan, query_bytes)?;
    let requested_game =
        plan.game
            .portfolio_game()
            .ok_or_else(|| StrategyServeError::PortfolioGameMismatch {
                requested_game: format!("{:?}", plan.game),
                query_game: "unknown".to_string(),
            })?;
    let (response, quality_summary) = match query {
        PortfolioQuery::Bootstrap(query) => {
            ensure_portfolio_query_game(requested_game, query.info.game)?;
            let response = solver.answer_checked(query)?;
            let quality_summary = portfolio_quality_summary_for_game(solver, requested_game)?;
            (response, quality_summary)
        }
        PortfolioQuery::Strength(query) => {
            ensure_portfolio_query_game(requested_game, query.info.game)?;
            let response = solver.answer_strength_checked(query.clone())?;
            let quality = solver.strength_quality(query)?;
            (response, portfolio_quality_summary(&quality))
        }
    };
    let response_bytes = encode_portfolio_strategy_response(&response).map_err(|source| {
        StrategyServeError::EncodeResponse(portfolio_wire_error(
            "portfolio strategy response",
            source,
        ))
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

    let recommended_action = recommended_portfolio_action(&response)
        .map(|action| format!("{action:?}"))
        .unwrap_or_else(|| "none".to_string());

    Ok(StrategyServeReport {
        game: plan.game,
        response_path: plan.response_path.clone(),
        action_count: response.actions.len(),
        recommended_action,
        quality_summary,
    })
}

fn decode_portfolio_query(
    plan: &StrategyServePlan,
    query_bytes: &[u8],
) -> Result<PortfolioQuery, StrategyServeError> {
    match decode_portfolio_strength_query(query_bytes) {
        Ok(query) => Ok(PortfolioQuery::Strength(query)),
        Err(strength_error) => match decode_portfolio_strategy_query(query_bytes) {
            Ok(query) => Ok(PortfolioQuery::Bootstrap(query)),
            Err(_) => Err(StrategyServeError::DecodeQuery {
                path: plan.query_path.display().to_string(),
                source: portfolio_wire_error(
                    "portfolio strategy or strength query",
                    strength_error,
                ),
            }),
        },
    }
}

fn ensure_portfolio_query_game(
    requested_game: myosu_games_portfolio::ResearchGame,
    query_game: myosu_games_portfolio::ResearchGame,
) -> Result<(), StrategyServeError> {
    if query_game == requested_game {
        return Ok(());
    }

    Err(StrategyServeError::PortfolioGameMismatch {
        requested_game: requested_game.to_string(),
        query_game: query_game.to_string(),
    })
}

fn portfolio_quality_summary_for_game(
    solver: &PortfolioSolver,
    game: myosu_games_portfolio::ResearchGame,
) -> Result<String, PortfolioSolverError> {
    let query = PortfolioSolver::strength_query(game)?;
    let quality = solver.strength_quality(query)?;

    Ok(portfolio_quality_summary(&quality))
}

fn portfolio_quality_summary(report: &myosu_games_portfolio::EngineQualityReport) -> String {
    format!(
        "engine_tier={} engine_family={} challenge_id={} score={:.6} baseline_l1_distance={:.6} legal_actions={} deterministic={}",
        report.engine_tier.as_str(),
        shell_token(&report.engine_family),
        report.challenge_id,
        report.score,
        report.baseline_l1_distance,
        report.legal_action_count,
        report.deterministic,
    )
}

fn shell_token(value: &str) -> String {
    value.replace(' ', "_")
}

fn portfolio_wire_error(
    context: &'static str,
    source: myosu_games_portfolio::WireCodecError,
) -> WireCodecError {
    match source {
        myosu_games_portfolio::WireCodecError::Encode { source, .. } => {
            WireCodecError::Encode { context, source }
        }
        myosu_games_portfolio::WireCodecError::Decode { source, .. } => {
            WireCodecError::Decode { context, source }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

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
    fn kuhn_serve_plan_does_not_require_encoder_dir() {
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
            checkpoint: Some(PathBuf::from("/tmp/checkpoint.bin")),
            train_iterations: 0,
            query_file: Some(PathBuf::from("/tmp/query.bin")),
            response_file: None,
        };

        let plan = strategy_serve_plan_from_cli(&cli, None)
            .expect("kuhn serve plan should build")
            .expect("query serving should be requested");

        assert_eq!(plan.game, GameSelection::Kuhn);
        assert_eq!(plan.encoder_dir, PathBuf::new());
    }

    #[test]
    fn portfolio_serve_plan_does_not_require_encoder_dir() {
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
            checkpoint: Some(PathBuf::from("/tmp/checkpoint.bin")),
            train_iterations: 0,
            query_file: Some(PathBuf::from("/tmp/query.bin")),
            response_file: None,
        };

        let plan = strategy_serve_plan_from_cli(&cli, None)
            .expect("portfolio serve plan should build")
            .expect("query serving should be requested");

        assert_eq!(plan.game, GameSelection::Bridge);
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

    #[test]
    fn serve_kuhn_strategy_once_writes_wire_response() {
        let root = unique_temp_root();
        let query_path = root.join("query.bin");
        let response_path = root.join("responses").join("latest.bin");
        let opening = myosu_games_kuhn::KuhnGame::root().apply(myosu_games_kuhn::KuhnEdge::Deal {
            p1: myosu_games_kuhn::KuhnCard::King,
            p2: myosu_games_kuhn::KuhnCard::Queen,
        });
        let query = myosu_games_kuhn::KuhnStrategyQuery::new(
            opening
                .info()
                .expect("opening player turn should expose info"),
        );
        let query_bytes =
            myosu_games_kuhn::encode_strategy_query(&query).expect("query should encode");
        let solver = KuhnSolver::new();
        let plan = StrategyServePlan {
            game: GameSelection::Kuhn,
            encoder_dir: PathBuf::new(),
            checkpoint_path: root.join("checkpoint.bin"),
            query_path: query_path.clone(),
            response_path: response_path.clone(),
        };

        fs::create_dir_all(&root).expect("temp root should exist");
        fs::write(&query_path, &query_bytes).expect("query file should write");

        let report = serve_kuhn_strategy_once_with_solver(&solver, &plan, &query_bytes)
            .expect("serve batch should succeed");
        let response_bytes = fs::read(&response_path).expect("response file should be written");
        let response = myosu_games_kuhn::decode_strategy_response(&response_bytes)
            .expect("response should decode");

        assert_eq!(report.game, GameSelection::Kuhn);
        assert_eq!(report.response_path, response_path);
        assert_eq!(report.action_count, response.actions.len());
        assert!(report.response_path.is_file());
        assert!(response.is_valid());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serve_portfolio_strategy_once_writes_wire_response_for_every_portfolio_game() {
        let root = unique_temp_root();

        for game_selection in GameSelection::PORTFOLIO_SELECTIONS {
            let research_game = game_selection
                .portfolio_game()
                .expect("portfolio selection should map to a research game");
            let game_root = root.join(research_game.slug());
            let query_path = game_root.join("query.bin");
            let response_path = game_root.join("responses").join("latest.bin");
            let query = myosu_games_portfolio::PortfolioSolver::bootstrap_query(research_game);
            let query_bytes =
                myosu_games_portfolio::encode_strategy_query(&query).expect("query should encode");
            let solver = PortfolioSolver::for_game(research_game);
            let plan = StrategyServePlan {
                game: game_selection,
                encoder_dir: PathBuf::new(),
                checkpoint_path: game_root.join("checkpoint.bin"),
                query_path: query_path.clone(),
                response_path: response_path.clone(),
            };

            fs::create_dir_all(&game_root).expect("temp game root should exist");
            fs::write(&query_path, &query_bytes).expect("query file should write");

            let report = serve_portfolio_strategy_once_with_solver(&solver, &plan, &query_bytes)
                .expect("serve batch should succeed");
            let response_bytes = fs::read(&response_path).expect("response file should be written");
            let response = myosu_games_portfolio::decode_strategy_response(&response_bytes)
                .expect("response should decode");

            assert_eq!(report.game, game_selection);
            assert_eq!(report.response_path, response_path);
            assert_eq!(report.action_count, response.actions.len());
            assert!(report.quality_summary.contains("engine_tier=rule-aware"));
            assert!(report.quality_summary.contains("baseline_l1_distance="));
            assert!(report.response_path.is_file());
            assert!(response.is_valid());
        }

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serve_portfolio_strategy_once_rejects_mismatched_query_game() {
        let root = unique_temp_root();
        let query_path = root.join("query.bin");
        let response_path = root.join("responses").join("latest.bin");
        let query = myosu_games_portfolio::PortfolioSolver::bootstrap_query(
            myosu_games_portfolio::ResearchGame::Cribbage,
        );
        let query_bytes =
            myosu_games_portfolio::encode_strategy_query(&query).expect("query should encode");
        let solver = PortfolioSolver::for_game(myosu_games_portfolio::ResearchGame::Bridge);
        let plan = StrategyServePlan {
            game: GameSelection::Bridge,
            encoder_dir: PathBuf::new(),
            checkpoint_path: root.join("checkpoint.bin"),
            query_path: query_path.clone(),
            response_path,
        };

        let error = serve_portfolio_strategy_once_with_solver(&solver, &plan, &query_bytes)
            .expect_err("mismatched portfolio query should fail");

        assert!(matches!(
            error,
            StrategyServeError::PortfolioGameMismatch { .. }
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serve_portfolio_strategy_once_accepts_typed_strength_query() {
        let root = unique_temp_root();
        let query_path = root.join("strength-query.bin");
        let response_path = root.join("responses").join("latest.bin");
        let query = myosu_games_portfolio::PortfolioSolver::strength_query(
            myosu_games_portfolio::ResearchGame::Bridge,
        )
        .expect("bridge should have strength query");
        let solver = PortfolioSolver::for_game(myosu_games_portfolio::ResearchGame::Bridge);
        let expected_quality = solver
            .strength_quality(query.clone())
            .expect("bridge quality should compute");
        let query_bytes =
            myosu_games_portfolio::encode_strength_query(&query).expect("query should encode");
        let plan = StrategyServePlan {
            game: GameSelection::Bridge,
            encoder_dir: PathBuf::new(),
            checkpoint_path: root.join("checkpoint.bin"),
            query_path: query_path.clone(),
            response_path: response_path.clone(),
        };

        fs::create_dir_all(&root).expect("temp root should exist");
        fs::write(&query_path, &query_bytes).expect("query file should write");

        let report = serve_portfolio_strategy_once_with_solver(&solver, &plan, &query_bytes)
            .expect("typed serve batch should succeed");
        let response_bytes = fs::read(&response_path).expect("response file should be written");
        let response = myosu_games_portfolio::decode_strategy_response(&response_bytes)
            .expect("response should decode");
        let challenge_id = query.info.challenge.spot().challenge_id.clone();

        assert_eq!(report.game, GameSelection::Bridge);
        assert_eq!(report.response_path, response_path);
        assert_eq!(report.action_count, response.actions.len());
        assert!(
            report
                .quality_summary
                .contains(&format!("challenge_id={challenge_id}"))
        );
        assert!(report.quality_summary.contains(&format!(
            "baseline_l1_distance={:.6}",
            expected_quality.baseline_l1_distance
        )));
        assert!(report.response_path.is_file());
        assert!(response.is_valid());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn serve_portfolio_strategy_once_rejects_typed_query_game_mismatch() {
        let root = unique_temp_root();
        let query_path = root.join("strength-query.bin");
        let query = myosu_games_portfolio::PortfolioSolver::strength_query(
            myosu_games_portfolio::ResearchGame::Cribbage,
        )
        .expect("cribbage should have strength query");
        let query_bytes =
            myosu_games_portfolio::encode_strength_query(&query).expect("query should encode");
        let solver = PortfolioSolver::for_game(myosu_games_portfolio::ResearchGame::Bridge);
        let plan = StrategyServePlan {
            game: GameSelection::Bridge,
            encoder_dir: PathBuf::new(),
            checkpoint_path: root.join("checkpoint.bin"),
            query_path,
            response_path: root.join("responses").join("latest.bin"),
        };

        let error = serve_portfolio_strategy_once_with_solver(&solver, &plan, &query_bytes)
            .expect_err("mismatched typed portfolio query should fail");

        assert!(matches!(
            error,
            StrategyServeError::PortfolioGameMismatch { .. }
        ));
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
