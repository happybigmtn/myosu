use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use myosu_games::StrategyResponse;
use myosu_games_liars_dice::LiarsDiceEdge;
use myosu_games_liars_dice::LiarsDiceSolver;
use myosu_games_liars_dice::LiarsDiceSolverError;
use myosu_games_liars_dice::decode_strategy_query as decode_liars_dice_strategy_query;
use myosu_games_liars_dice::decode_strategy_response as decode_liars_dice_strategy_response;
use myosu_games_liars_dice::recommended_edge as recommended_liars_dice_edge;
use myosu_games_poker::ArtifactCodecError;
use myosu_games_poker::PokerSolver;
use myosu_games_poker::PokerSolverError;
use myosu_games_poker::RbpNlheEdge;
use myosu_games_poker::WireCodecError;
use myosu_games_poker::decode_strategy_query;
use myosu_games_poker::decode_strategy_response;
use myosu_games_poker::load_encoder_dir;
use myosu_games_poker::recommended_edge;
use thiserror::Error;
use tracing::info;

use crate::cli::Cli;
use crate::cli::GameSelection;

const LIARS_DICE_SOLVER_TREES: usize = 1 << 10;

/// Startup error returned by the bootstrap validator binary.
#[derive(Debug, Error)]
pub enum ValidatorBootstrapError {
    /// Returned when the operator key source cannot be resolved.
    #[error("{0}")]
    Key(#[from] myosu_keys::KeyError),

    /// Returned when the chain connectivity probe fails.
    #[error("{0}")]
    Chain(#[from] crate::chain::ChainProbeError),

    /// Returned when the validator cannot complete an on-chain bootstrap action.
    #[error("{0}")]
    ChainAction(#[from] crate::chain::ChainActionError),

    /// Returned when bounded validation cannot start or complete.
    #[error("{0}")]
    Validation(#[from] ValidationError),
}

/// Configuration for one bounded validator scoring pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationPlan {
    pub game: GameSelection,
    pub encoder_dir: PathBuf,
    pub checkpoint_path: PathBuf,
    pub query_path: PathBuf,
    pub response_path: PathBuf,
}

/// Operator-visible summary of one bounded validator scoring pass.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationReport {
    pub game: GameSelection,
    pub action_count: usize,
    pub exact_match: bool,
    pub l1_distance: f64,
    pub score: f64,
    pub expected_action: String,
    pub observed_action: String,
}

/// Errors returned while preparing or executing the validator scoring pass.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Returned when validation is requested without the required encoder directory.
    #[error("--query-file and --response-file require --encoder-dir when --game poker")]
    MissingEncoderDir,

    /// Returned when validation is requested without the required checkpoint.
    #[error("--query-file and --response-file require --checkpoint")]
    MissingCheckpoint,

    /// Returned when only one of the query or response paths is supplied.
    #[error("--query-file and --response-file must be provided together")]
    IncompleteArtifactPair,

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

    /// Returned when the response input file cannot be read.
    #[error("failed to read strategy response `{path}`: {source}")]
    ReadResponse {
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

    /// Returned when the response input fails to decode from the wire format.
    #[error("failed to decode strategy response `{path}`: {source}")]
    DecodeResponse {
        path: String,
        #[source]
        source: WireCodecError,
    },

    /// Returned when the solver fails to load the checkpoint or answer a query.
    #[error("failed to compute validator expectation: {0}")]
    Solver(#[from] PokerSolverError),

    /// Returned when the Liar's Dice solver fails to load the checkpoint or answer a query.
    #[error("failed to compute liar's dice validator expectation: {0}")]
    LiarsDiceSolver(#[from] LiarsDiceSolverError),

    /// Returned when the miner response does not form a valid probability distribution.
    #[error("strategy response `{path}` is not a valid distribution")]
    InvalidResponse { path: String },
}

/// Builds an optional bounded validation plan from the current CLI flags.
///
/// Args:
///     cli: Parsed validator CLI arguments.
///
/// Returns:
///     `Ok(None)` when no validation batch was requested, otherwise the
///     validated `ValidationPlan`.
pub fn validation_plan_from_cli(cli: &Cli) -> Result<Option<ValidationPlan>, ValidationError> {
    match (&cli.query_file, &cli.response_file) {
        (None, None) => Ok(None),
        (Some(_), None) | (None, Some(_)) => Err(ValidationError::IncompleteArtifactPair),
        (Some(query_path), Some(response_path)) => {
            let encoder_dir = match cli.game {
                GameSelection::Poker => cli
                    .encoder_dir
                    .clone()
                    .ok_or(ValidationError::MissingEncoderDir)?,
                GameSelection::LiarsDice => cli.encoder_dir.clone().unwrap_or_default(),
            };
            let Some(checkpoint_path) = cli.checkpoint.clone() else {
                return Err(ValidationError::MissingCheckpoint);
            };

            Ok(Some(ValidationPlan {
                game: cli.game,
                encoder_dir,
                checkpoint_path,
                query_path: query_path.clone(),
                response_path: response_path.clone(),
            }))
        }
    }
}

/// Loads the validator checkpoint and scores one wire-encoded miner response.
///
/// Args:
///     plan: Validated scoring plan from the validator CLI.
///
/// Returns:
///     A `ValidationReport` describing the comparison against the local expectation.
pub fn score_response(plan: &ValidationPlan) -> Result<ValidationReport, ValidationError> {
    let started_at = Instant::now();
    let query_bytes = fs::read(&plan.query_path).map_err(|source| ValidationError::ReadQuery {
        path: plan.query_path.display().to_string(),
        source,
    })?;
    let response_bytes =
        fs::read(&plan.response_path).map_err(|source| ValidationError::ReadResponse {
            path: plan.response_path.display().to_string(),
            source,
        })?;
    let query_path = plan.query_path.display().to_string();
    let response_path = plan.response_path.display().to_string();
    let report = match plan.game {
        GameSelection::Poker => {
            let encoder =
                load_encoder_dir(&plan.encoder_dir).map_err(|source| ValidationError::Encoder {
                    path: plan.encoder_dir.display().to_string(),
                    source,
                })?;
            let solver = PokerSolver::load(&plan.checkpoint_path, encoder)?;
            score_poker_response_with_solver(
                &solver,
                &query_path,
                &response_path,
                &query_bytes,
                &response_bytes,
            )?
        }
        GameSelection::LiarsDice => {
            let solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::load(&plan.checkpoint_path)?;
            score_liars_dice_response_with_solver(
                &solver,
                &query_path,
                &response_path,
                &query_bytes,
                &response_bytes,
            )?
        }
    };
    info!(
        game = ?report.game,
        query_path = %plan.query_path.display(),
        response_path = %plan.response_path.display(),
        checkpoint_path = %plan.checkpoint_path.display(),
        action_count = report.action_count,
        exact_match = report.exact_match,
        l1_distance = report.l1_distance,
        score = report.score,
        expected_action = %report.expected_action,
        observed_action = %report.observed_action,
        elapsed_ms = started_at.elapsed().as_millis(),
        "scored bounded validator response"
    );
    Ok(report)
}

fn score_poker_response_with_solver(
    solver: &PokerSolver,
    query_path: &str,
    response_path: &str,
    query_bytes: &[u8],
    response_bytes: &[u8],
) -> Result<ValidationReport, ValidationError> {
    let query =
        decode_strategy_query(query_bytes).map_err(|source| ValidationError::DecodeQuery {
            path: query_path.to_string(),
            source,
        })?;
    let observed = decode_strategy_response(response_bytes).map_err(|source| {
        ValidationError::DecodeResponse {
            path: response_path.to_string(),
            source,
        }
    })?;
    if !observed.is_valid() {
        return Err(ValidationError::InvalidResponse {
            path: response_path.to_string(),
        });
    }

    let expected = solver.answer(query);
    let l1_distance = l1_distance(&expected, &observed);
    let score = score_from_l1_distance(l1_distance);
    let exact_match = l1_distance < f64::EPSILON;

    Ok(ValidationReport {
        game: GameSelection::Poker,
        action_count: observed.actions.len(),
        exact_match,
        l1_distance,
        score,
        expected_action: describe_recommendation(&expected),
        observed_action: describe_recommendation(&observed),
    })
}

fn score_liars_dice_response_with_solver(
    solver: &LiarsDiceSolver<LIARS_DICE_SOLVER_TREES>,
    query_path: &str,
    response_path: &str,
    query_bytes: &[u8],
    response_bytes: &[u8],
) -> Result<ValidationReport, ValidationError> {
    let query = decode_liars_dice_strategy_query(query_bytes).map_err(|source| {
        ValidationError::DecodeQuery {
            path: query_path.to_string(),
            source: match source {
                myosu_games_liars_dice::WireCodecError::Decode { source, .. } => {
                    WireCodecError::Decode {
                        context: "liar's dice strategy query",
                        source,
                    }
                }
                myosu_games_liars_dice::WireCodecError::Encode { source, .. } => {
                    WireCodecError::Encode {
                        context: "liar's dice strategy query",
                        source,
                    }
                }
            },
        }
    })?;
    let observed = decode_liars_dice_strategy_response(response_bytes).map_err(|source| {
        ValidationError::DecodeResponse {
            path: response_path.to_string(),
            source: match source {
                myosu_games_liars_dice::WireCodecError::Decode { source, .. } => {
                    WireCodecError::Decode {
                        context: "liar's dice strategy response",
                        source,
                    }
                }
                myosu_games_liars_dice::WireCodecError::Encode { source, .. } => {
                    WireCodecError::Encode {
                        context: "liar's dice strategy response",
                        source,
                    }
                }
            },
        }
    })?;
    if !observed.is_valid() {
        return Err(ValidationError::InvalidResponse {
            path: response_path.to_string(),
        });
    }

    let expected = solver.answer(query);
    let l1_distance = l1_distance_liars_dice(&expected, &observed);
    let score = score_from_l1_distance(l1_distance);
    let exact_match = l1_distance < f64::EPSILON;

    Ok(ValidationReport {
        game: GameSelection::LiarsDice,
        action_count: observed.actions.len(),
        exact_match,
        l1_distance,
        score,
        expected_action: describe_liars_dice_recommendation(&expected),
        observed_action: describe_liars_dice_recommendation(&observed),
    })
}

fn l1_distance(
    expected: &StrategyResponse<RbpNlheEdge>,
    observed: &StrategyResponse<RbpNlheEdge>,
) -> f64 {
    l1_distance_union(&expected.actions, &observed.actions)
}

fn score_from_l1_distance(l1_distance: f64) -> f64 {
    1.0 / (1.0 + l1_distance.max(0.0))
}

fn describe_recommendation(response: &StrategyResponse<RbpNlheEdge>) -> String {
    recommended_edge(response)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string())
}

fn l1_distance_liars_dice(
    expected: &StrategyResponse<LiarsDiceEdge>,
    observed: &StrategyResponse<LiarsDiceEdge>,
) -> f64 {
    l1_distance_union(&expected.actions, &observed.actions)
}

// Compute symmetric L1 over the union of actions so explicit zero-weight
// entries do not get counted twice.
fn l1_distance_union<E>(expected: &[(E, f32)], observed: &[(E, f32)]) -> f64
where
    E: Clone + Ord + PartialEq,
{
    expected
        .iter()
        .map(|(action, _)| action.clone())
        .chain(observed.iter().map(|(action, _)| action.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|action| {
            let expected_probability = probability_for(expected, &action);
            let observed_probability = probability_for(observed, &action);
            f64::from((expected_probability - observed_probability).abs())
        })
        .sum()
}

fn probability_for<E>(actions: &[(E, f32)], needle: &E) -> f32
where
    E: PartialEq,
{
    actions
        .iter()
        .find(|(action, _)| action == needle)
        .map(|(_, probability)| *probability)
        .unwrap_or(0.0)
}

fn describe_liars_dice_recommendation(response: &StrategyResponse<LiarsDiceEdge>) -> String {
    recommended_liars_dice_edge(response)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    use myosu_games::CfrGame;
    use myosu_games_liars_dice::LiarsDiceClaim;
    use myosu_games_poker::encode_strategy_query;
    use myosu_games_poker::encode_strategy_response;
    use myosu_games_poker::encoder_from_lookup;
    use rbp_cards::Isomorphism;
    use rbp_cards::Observation;
    use rbp_gameplay::Abstraction;
    use rbp_gameplay::Edge;
    use rbp_gameplay::Odds;
    use rbp_mccfr::Encounter;
    use rbp_nlhe::NlheEncoder;
    use rbp_nlhe::NlheInfo;
    use rbp_nlhe::NlheProfile;

    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct LiarsDiceBenchmarkPoint {
        iterations: usize,
        exploitability: f32,
    }

    #[test]
    fn validation_plan_requires_both_artifact_paths() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Bob".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            register: false,
            enable_subtoken: false,
            submit_weights: false,
            stake_amount: None,
            weight_hotkey: None,
            game: GameSelection::Poker,
            encoder_dir: None,
            checkpoint: None,
            query_file: Some(PathBuf::from("/tmp/query.bin")),
            response_file: None,
        };

        let error = validation_plan_from_cli(&cli)
            .expect_err("validation should require both query and response");
        assert!(matches!(error, ValidationError::IncompleteArtifactPair));
    }

    #[test]
    fn exact_match_scores_one() {
        let solver = weighted_solver();
        let query = myosu_games_poker::NlheBlueprint::query_for_info(&sample_info());
        let query_bytes = encode_strategy_query(&query).expect("query should encode");
        let response = solver.answer(query);
        let response_bytes = encode_strategy_response(&response).expect("response should encode");

        let report = score_poker_response_with_solver(
            &solver,
            "/tmp/query.bin",
            "/tmp/response.bin",
            &query_bytes,
            &response_bytes,
        )
        .expect("validation should succeed");

        assert_eq!(report.game, GameSelection::Poker);
        assert!(report.exact_match);
        assert_eq!(report.score, 1.0);
        assert_eq!(report.l1_distance, 0.0);
    }

    #[test]
    fn three_action_mismatch_uses_game_agnostic_normalization() {
        let solver = weighted_solver();
        let query = myosu_games_poker::NlheBlueprint::query_for_info(&sample_info());
        let expected = solver.answer(query.clone());
        let query_bytes = encode_strategy_query(&query).expect("query should encode");
        let observed = StrategyResponse::new(vec![
            (RbpNlheEdge::from(Edge::Fold), 0.0),
            (RbpNlheEdge::from(Edge::Call), 0.0),
            (RbpNlheEdge::from(Edge::Raise(Odds::new(1, 1))), 1.0),
        ]);
        let response_bytes = encode_strategy_response(&observed).expect("response should encode");

        let report = score_poker_response_with_solver(
            &solver,
            "/tmp/query.bin",
            "/tmp/response.bin",
            &query_bytes,
            &response_bytes,
        )
        .expect("validation should succeed");

        let expected_l1_distance = l1_distance(&expected, &observed);

        assert!(!report.exact_match);
        assert!(expected_l1_distance > 0.0);
        assert!((report.l1_distance - expected_l1_distance).abs() < 1e-12);
        assert!((report.score - score_from_l1_distance(expected_l1_distance)).abs() < 1e-12);
    }

    #[test]
    fn poker_l1_distance_does_not_double_count_explicit_zero_weight_actions() {
        let expected = StrategyResponse::new(vec![
            (RbpNlheEdge::from(Edge::Call), 1.0),
            (RbpNlheEdge::from(Edge::Fold), 0.0),
        ]);
        let observed = StrategyResponse::new(vec![
            (RbpNlheEdge::from(Edge::Call), 0.0),
            (RbpNlheEdge::from(Edge::Fold), 1.0),
        ]);

        assert!((l1_distance(&expected, &observed) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn inv_003_determinism() {
        let root = unique_temp_root();
        let checkpoint_path = root.join("checkpoint.bin");
        let query_path = root.join("query.bin");
        let response_path = root.join("response.bin");
        let solver = weighted_solver();
        let query = myosu_games_poker::NlheBlueprint::query_for_info(&sample_info());
        let query_bytes = encode_strategy_query(&query).expect("query should encode");
        let response_bytes =
            encode_strategy_response(&solver.answer(query)).expect("response should encode");

        fs::create_dir_all(&root).expect("temp root should exist");
        solver
            .save(&checkpoint_path)
            .expect("checkpoint should save");
        fs::write(&query_path, &query_bytes).expect("query file should write");
        fs::write(&response_path, &response_bytes).expect("response file should write");

        let first = score_poker_response_with_solver(
            &solver,
            &query_path.display().to_string(),
            &response_path.display().to_string(),
            &query_bytes,
            &response_bytes,
        )
        .expect("first validation should succeed");
        let second = score_poker_response_with_solver(
            &solver,
            &query_path.display().to_string(),
            &response_path.display().to_string(),
            &query_bytes,
            &response_bytes,
        )
        .expect("second validation should succeed");

        assert_eq!(first, second);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn liars_dice_validation_plan_does_not_require_encoder_dir() {
        let cli = Cli {
            chain: "ws://127.0.0.1:9944".to_string(),
            subnet: 1,
            key: Some("//Bob".to_string()),
            key_config_dir: None,
            key_password_env: "MYOSU_KEY_PASSWORD".to_string(),
            register: false,
            enable_subtoken: false,
            submit_weights: false,
            stake_amount: None,
            weight_hotkey: None,
            game: GameSelection::LiarsDice,
            encoder_dir: None,
            checkpoint: Some(PathBuf::from("/tmp/checkpoint.bin")),
            query_file: Some(PathBuf::from("/tmp/query.bin")),
            response_file: Some(PathBuf::from("/tmp/response.bin")),
        };

        let plan = validation_plan_from_cli(&cli)
            .expect("liar's dice validation plan should build")
            .expect("validation should be requested");

        assert_eq!(plan.game, GameSelection::LiarsDice);
        assert_eq!(plan.encoder_dir, PathBuf::new());
    }

    #[test]
    fn liars_dice_exact_match_scores_one() {
        let mut solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::new();
        solver.train(8).expect("training should succeed");
        let opening = myosu_games_liars_dice::LiarsDiceGame::root()
            .apply(myosu_games_liars_dice::LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let query = myosu_games_liars_dice::LiarsDiceStrategyQuery::new(
            opening
                .info()
                .expect("opening player turn should expose info"),
        );
        let query_bytes =
            myosu_games_liars_dice::encode_strategy_query(&query).expect("query should encode");
        let response = solver.answer(query);
        let response_bytes = myosu_games_liars_dice::encode_strategy_response(&response)
            .expect("response should encode");

        let report = score_liars_dice_response_with_solver(
            &solver,
            "/tmp/query.bin",
            "/tmp/response.bin",
            &query_bytes,
            &response_bytes,
        )
        .expect("validation should succeed");

        assert_eq!(report.game, GameSelection::LiarsDice);
        assert!(report.exact_match);
        assert_eq!(report.score, 1.0);
        assert_eq!(report.l1_distance, 0.0);
    }

    #[test]
    fn liars_dice_l1_distance_does_not_double_count_explicit_zero_weight_actions() {
        let expected = StrategyResponse::new(vec![
            (
                LiarsDiceEdge::Bid(LiarsDiceClaim::new(1, 2).expect("claim should build")),
                1.0,
            ),
            (LiarsDiceEdge::Challenge, 0.0),
        ]);
        let observed = StrategyResponse::new(vec![
            (
                LiarsDiceEdge::Bid(LiarsDiceClaim::new(1, 2).expect("claim should build")),
                0.0,
            ),
            (LiarsDiceEdge::Challenge, 1.0),
        ]);

        assert!((l1_distance_liars_dice(&expected, &observed) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn liars_dice_inv_003_determinism() {
        let mut solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::new();
        solver.train(8).expect("training should succeed");
        let opening = myosu_games_liars_dice::LiarsDiceGame::root()
            .apply(myosu_games_liars_dice::LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let query = myosu_games_liars_dice::LiarsDiceStrategyQuery::new(
            opening
                .info()
                .expect("opening player turn should expose info"),
        );
        let query_bytes =
            myosu_games_liars_dice::encode_strategy_query(&query).expect("query should encode");
        let response_bytes =
            myosu_games_liars_dice::encode_strategy_response(&solver.answer(query))
                .expect("response should encode");

        let first = score_liars_dice_response_with_solver(
            &solver,
            "/tmp/query.bin",
            "/tmp/response.bin",
            &query_bytes,
            &response_bytes,
        )
        .expect("first validation should succeed");
        let second = score_liars_dice_response_with_solver(
            &solver,
            "/tmp/query.bin",
            "/tmp/response.bin",
            &query_bytes,
            &response_bytes,
        )
        .expect("second validation should succeed");

        assert_eq!(first, second);
    }

    #[test]
    fn cross_game_one_hot_degradation_stays_in_same_score_band() {
        let poker_solver = weighted_solver();
        let poker_query = myosu_games_poker::NlheBlueprint::query_for_info(&sample_info());
        let poker_query_bytes = encode_strategy_query(&poker_query).expect("query should encode");
        let poker_expected = poker_solver.answer(poker_query);
        let poker_observed = poker_one_hot_least_likely_action(&poker_expected);
        let poker_response_bytes =
            encode_strategy_response(&poker_observed).expect("response should encode");
        let poker_report = score_poker_response_with_solver(
            &poker_solver,
            "/tmp/query.bin",
            "/tmp/response.bin",
            &poker_query_bytes,
            &poker_response_bytes,
        )
        .expect("poker validation should succeed");

        let mut liars_dice_solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::new();
        liars_dice_solver.train(8).expect("training should succeed");
        let opening = myosu_games_liars_dice::LiarsDiceGame::root()
            .apply(myosu_games_liars_dice::LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let liars_dice_query = myosu_games_liars_dice::LiarsDiceStrategyQuery::new(
            opening
                .info()
                .expect("opening player turn should expose info"),
        );
        let liars_dice_query_bytes =
            myosu_games_liars_dice::encode_strategy_query(&liars_dice_query)
                .expect("query should encode");
        let liars_dice_expected = liars_dice_solver.answer(liars_dice_query);
        let liars_dice_observed = liars_dice_one_hot_least_likely_action(&liars_dice_expected);
        let liars_dice_response_bytes =
            myosu_games_liars_dice::encode_strategy_response(&liars_dice_observed)
                .expect("response should encode");
        let liars_dice_report = score_liars_dice_response_with_solver(
            &liars_dice_solver,
            "/tmp/query.bin",
            "/tmp/response.bin",
            &liars_dice_query_bytes,
            &liars_dice_response_bytes,
        )
        .expect("liar's dice validation should succeed");

        // Documentation test: the current stage-0 validator score only sees
        // normalized L1 distance. For the same "collapse the policy to one weak
        // legal action" degradation pattern, the sampled poker and liar's-dice
        // states stay in the same rough score band. That is encouraging for
        // stage-0 fairness, but it does not prove full cross-subnet fairness for
        // different game configs or exploitability units.
        let score_gap = (poker_report.score - liars_dice_report.score).abs();

        assert!(!poker_report.exact_match);
        assert!(!liars_dice_report.exact_match);
        assert!(poker_report.score < 1.0);
        assert!(liars_dice_report.score < 1.0);
        assert!(
            score_gap <= 0.1,
            "sampled cross-game scores diverged: poker={:.6} liar's-dice={:.6}",
            poker_report.score,
            liars_dice_report.score
        );
    }

    #[test]
    fn quality_benchmark_liars_dice_exploitability_converges() {
        // This bypasses the validator self-check path and measures the
        // solver's exact exploitability directly.
        let benchmark = liars_dice_benchmark_points(&[0, 128, 256, 512]);

        assert!(
            benchmark
                .windows(2)
                .all(|pair| pair[1].exploitability < pair[0].exploitability),
            "expected exploitability to decrease across benchmark points: {:?}",
            benchmark
        );

        let baseline_drop =
            benchmark[0].exploitability - benchmark[benchmark.len() - 1].exploitability;
        assert!(
            baseline_drop >= 0.15,
            "expected 512 iterations to materially improve exploitability: {:?}",
            benchmark
        );

        let recommended = benchmark
            .iter()
            .find(|point| point.exploitability <= 0.70)
            .map(|point| point.iterations);
        assert_eq!(
            recommended,
            Some(512),
            "expected the benchmark ladder to recommend 512 iterations: {:?}",
            benchmark
        );
    }

    fn weighted_solver() -> PokerSolver {
        PokerSolver::from_parts(weighted_profile(sample_info()), sample_encoder())
    }

    fn liars_dice_benchmark_points(iterations: &[usize]) -> Vec<LiarsDiceBenchmarkPoint> {
        iterations
            .iter()
            .copied()
            .map(|iterations| {
                let mut solver = LiarsDiceSolver::<LIARS_DICE_SOLVER_TREES>::new();
                solver
                    .train(iterations)
                    .expect("benchmark training should succeed");

                LiarsDiceBenchmarkPoint {
                    iterations,
                    exploitability: solver.exact_exploitability(),
                }
            })
            .collect()
    }

    fn poker_one_hot_least_likely_action(
        response: &StrategyResponse<RbpNlheEdge>,
    ) -> StrategyResponse<RbpNlheEdge> {
        let (action, _) = response
            .actions
            .iter()
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .expect("response should contain at least one action");

        StrategyResponse::new(vec![(action.clone(), 1.0)])
    }

    fn liars_dice_one_hot_least_likely_action(
        response: &StrategyResponse<LiarsDiceEdge>,
    ) -> StrategyResponse<LiarsDiceEdge> {
        let (action, _) = response
            .actions
            .iter()
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .expect("response should contain at least one action");

        StrategyResponse::new(vec![(action.clone(), 1.0)])
    }

    fn sample_encoder() -> NlheEncoder {
        encoder_from_lookup(BTreeMap::from([(
            Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
            Abstraction::from(42_i16),
        )]))
        .expect("encoder lookup should build")
    }

    fn sample_info() -> NlheInfo {
        let subgame = vec![Edge::Check, Edge::Raise(Odds::new(1, 2))]
            .into_iter()
            .collect();
        let choices = vec![Edge::Fold, Edge::Call, Edge::Raise(Odds::new(1, 1))]
            .into_iter()
            .collect();
        let bucket = Abstraction::from(42_i16);

        NlheInfo::from((subgame, bucket, choices))
    }

    fn weighted_profile(info: NlheInfo) -> NlheProfile {
        NlheProfile {
            iterations: 12,
            encounters: BTreeMap::from([(
                info,
                BTreeMap::from([
                    (
                        rbp_nlhe::NlheEdge::from(Edge::Fold),
                        Encounter::new(0.05, 0.0, 0.0, 1),
                    ),
                    (
                        rbp_nlhe::NlheEdge::from(Edge::Call),
                        Encounter::new(0.80, 0.0, 0.0, 1),
                    ),
                    (
                        rbp_nlhe::NlheEdge::from(Edge::Raise(Odds::new(1, 1))),
                        Encounter::new(0.15, 0.0, 0.0, 1),
                    ),
                ]),
            )]),
            metrics: rbp_mccfr::Metrics::with_epoch(12),
        }
    }

    fn unique_temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-validator-{nanos}"))
    }
}
