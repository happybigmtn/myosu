use std::fs;
use std::path::Path;

use bincode::Options;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::game::ResearchGame;
use crate::protocol::{
    PortfolioAction, PortfolioStrategyQuery, PortfolioStrategyResponse, PortfolioStrengthInfo,
    PortfolioStrengthQuery, STRENGTH_WIRE_VERSION, recommended_action,
};
use crate::quality::EngineQualityReport;

const MAX_DECODE_BYTES: u64 = 1_048_576;
const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOP";
const CHECKPOINT_VERSION: u32 = 2;
const CHECKPOINT_HEADER_LEN: usize = 8;

/// Checkpointable rule-aware solver adapter for the research-game portfolio.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioSolver {
    game: Option<ResearchGame>,
    epochs: usize,
}

impl PortfolioSolver {
    /// Create a fresh unscoped portfolio solver.
    ///
    /// Unscoped solvers are useful for demo and test surfaces that need to
    /// answer multiple research games. Persisted miner checkpoints should use
    /// `for_game` so validators can reject mismatched query artifacts.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a fresh portfolio solver scoped to one research game.
    pub fn for_game(game: ResearchGame) -> Self {
        Self {
            game: Some(game),
            epochs: 0,
        }
    }

    /// Create a representative query for a research game.
    pub fn bootstrap_query(game: ResearchGame) -> PortfolioStrategyQuery {
        PortfolioStrategyQuery::new(crate::protocol::PortfolioInfo::bootstrap(game))
    }

    /// Create a typed strength query for a portfolio-routed research game.
    pub fn strength_query(
        game: ResearchGame,
    ) -> Result<PortfolioStrengthQuery, PortfolioSolverError> {
        let info = PortfolioStrengthInfo::bootstrap(game)
            .ok_or(PortfolioSolverError::UnsupportedPortfolioGame { game })?;
        Ok(PortfolioStrengthQuery::new(info))
    }

    /// Load a solver checkpoint from disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PortfolioSolverError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| PortfolioSolverError::Read {
            path: path.display().to_string(),
            source,
        })?;

        Self::from_checkpoint_bytes(&bytes)
    }

    /// Decode a solver checkpoint.
    pub fn from_checkpoint_bytes(bytes: &[u8]) -> Result<Self, PortfolioSolverError> {
        if bytes.len() < CHECKPOINT_HEADER_LEN {
            return Err(PortfolioSolverError::CheckpointTooShort { bytes: bytes.len() });
        }

        let header = bytes
            .get(..CHECKPOINT_HEADER_LEN)
            .ok_or(PortfolioSolverError::CheckpointTooShort { bytes: bytes.len() })?;
        let (magic, version) = header.split_at(4);
        let found_magic: [u8; 4] = magic
            .try_into()
            .map_err(|_| PortfolioSolverError::CheckpointTooShort { bytes: bytes.len() })?;
        if found_magic != CHECKPOINT_MAGIC {
            return Err(PortfolioSolverError::CheckpointMagic {
                found: String::from_utf8_lossy(&found_magic).into_owned(),
            });
        }

        let found_version = u32::from_le_bytes(
            version
                .try_into()
                .map_err(|_| PortfolioSolverError::CheckpointTooShort { bytes: bytes.len() })?,
        );
        if found_version != CHECKPOINT_VERSION {
            return Err(PortfolioSolverError::CheckpointVersion {
                found: found_version,
                expected: CHECKPOINT_VERSION,
            });
        }

        decode_codec(MAX_DECODE_BYTES)
            .deserialize(
                bytes
                    .get(CHECKPOINT_HEADER_LEN..)
                    .ok_or(PortfolioSolverError::CheckpointTooShort { bytes: bytes.len() })?,
            )
            .map_err(|source| PortfolioSolverError::Decode { source })
    }

    /// Save a solver checkpoint to disk.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), PortfolioSolverError> {
        let path = path.as_ref();
        fs::write(path, self.checkpoint_bytes()?).map_err(|source| PortfolioSolverError::Write {
            path: path.display().to_string(),
            source,
        })
    }

    /// Encode a solver checkpoint.
    pub fn checkpoint_bytes(&self) -> Result<Vec<u8>, PortfolioSolverError> {
        let payload = encode_codec()
            .serialize(self)
            .map_err(|source| PortfolioSolverError::Encode { source })?;
        let mut bytes = Vec::with_capacity(CHECKPOINT_HEADER_LEN.saturating_add(payload.len()));
        bytes.extend_from_slice(&CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&payload);
        Ok(bytes)
    }

    /// Record a requested training batch.
    ///
    /// Current reference engines are deterministic, so training records the
    /// accepted iteration count for checkpoint parity and future engine upgrades.
    pub fn train(&mut self, iterations: usize) {
        self.epochs = self.epochs.saturating_add(iterations);
    }

    /// Number of accepted portfolio training iterations.
    pub const fn epochs(&self) -> usize {
        self.epochs
    }

    /// Research game this checkpoint is scoped to, if any.
    pub const fn game(&self) -> Option<ResearchGame> {
        self.game
    }

    /// Verify that this checkpoint can answer the requested research game.
    pub fn ensure_supports(&self, game: ResearchGame) -> Result<(), PortfolioSolverError> {
        match self.game {
            Some(checkpoint_game) if checkpoint_game != game => {
                Err(PortfolioSolverError::GameMismatch {
                    checkpoint_game,
                    query_game: game,
                })
            }
            _ => Ok(()),
        }
    }

    /// Answer a portfolio strategy query.
    pub fn answer(&self, query: PortfolioStrategyQuery) -> PortfolioStrategyResponse {
        crate::engine::answer_game(query.info.game, self.epochs).response
    }

    /// Answer a portfolio strategy query after checking checkpoint scope.
    pub fn answer_checked(
        &self,
        query: PortfolioStrategyQuery,
    ) -> Result<PortfolioStrategyResponse, PortfolioSolverError> {
        self.ensure_supports(query.info.game)?;
        Ok(self.answer(query))
    }

    /// Answer a typed strength query after checking checkpoint and query scope.
    pub fn answer_strength_checked(
        &self,
        query: PortfolioStrengthQuery,
    ) -> Result<PortfolioStrategyResponse, PortfolioSolverError> {
        let answer = self.answer_strength(query)?;

        Ok(answer.response)
    }

    /// Return a deterministic quality report for a typed strength query.
    pub fn strength_quality(
        &self,
        query: PortfolioStrengthQuery,
    ) -> Result<EngineQualityReport, PortfolioSolverError> {
        let answer = self.answer_strength(query)?;
        let baseline = crate::engines::baseline_answer(answer.game, answer.challenge_id.as_str());
        let seed = crate::rng::seed_for(answer.game, answer.challenge_id.as_str(), self.epochs, 0);

        Ok(EngineQualityReport::from_answer(
            &answer,
            &baseline,
            self.epochs,
            seed,
        ))
    }

    /// Return the highest-probability action for a query.
    pub fn recommend_query(&self, query: PortfolioStrategyQuery) -> Option<PortfolioAction> {
        recommended_action(&self.answer(query))
    }

    fn answer_strength(
        &self,
        query: PortfolioStrengthQuery,
    ) -> Result<crate::engine::EngineAnswer, PortfolioSolverError> {
        self.ensure_supports(query.info.game)?;
        if query.info.wire_version != STRENGTH_WIRE_VERSION {
            return Err(PortfolioSolverError::StrengthWireVersionMismatch {
                found: query.info.wire_version,
                expected: STRENGTH_WIRE_VERSION,
            });
        }

        let challenge_game = query.info.challenge.game();
        if query.info.game != challenge_game {
            return Err(PortfolioSolverError::StrengthQueryMismatch {
                query_game: query.info.game,
                challenge_game,
            });
        }

        Ok(crate::engine::answer_typed_challenge(
            &query.info.challenge,
            self.epochs,
        )?)
    }
}

/// Errors returned by the portfolio solver checkpoint surface.
#[derive(Debug, Error)]
pub enum PortfolioSolverError {
    #[error("failed to read checkpoint `{path}`: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write checkpoint `{path}`: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to encode portfolio checkpoint: {source}")]
    Encode { source: bincode::Error },
    #[error("failed to decode portfolio checkpoint: {source}")]
    Decode { source: bincode::Error },
    #[error("checkpoint too short: {bytes} bytes")]
    CheckpointTooShort { bytes: usize },
    #[error("checkpoint magic mismatch: found `{found}`, expected `MYOP`")]
    CheckpointMagic { found: String },
    #[error("checkpoint version {found} does not match expected version {expected}")]
    CheckpointVersion { found: u32, expected: u32 },
    #[error(
        "portfolio checkpoint game `{checkpoint_game}` cannot answer query game `{query_game}`"
    )]
    GameMismatch {
        checkpoint_game: ResearchGame,
        query_game: ResearchGame,
    },
    #[error("game `{game}` is not routed through the research portfolio engine")]
    UnsupportedPortfolioGame { game: ResearchGame },
    #[error(
        "portfolio strength query game `{query_game}` does not match challenge game `{challenge_game}`"
    )]
    StrengthQueryMismatch {
        query_game: ResearchGame,
        challenge_game: ResearchGame,
    },
    #[error("portfolio strength query version {found} does not match expected version {expected}")]
    StrengthWireVersionMismatch { found: u32, expected: u32 },
    #[error(transparent)]
    Engine(#[from] crate::engine::PortfolioEngineError),
}

fn encode_codec() -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .reject_trailing_bytes()
}

fn decode_codec(limit: u64) -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(limit)
        .reject_trailing_bytes()
}

#[cfg(test)]
mod tests {
    use crate::game::{ALL_PORTFOLIO_ROUTED_GAMES, ALL_RESEARCH_GAMES, ResearchGame};
    use crate::protocol::PortfolioAction;

    use super::PortfolioSolver;

    #[test]
    fn checkpoint_roundtrips() {
        let mut solver = PortfolioSolver::for_game(ResearchGame::Bridge);
        solver.train(7);
        let bytes = match solver.checkpoint_bytes() {
            Ok(bytes) => bytes,
            Err(error) => panic!("checkpoint should encode: {error}"),
        };
        let decoded = match PortfolioSolver::from_checkpoint_bytes(&bytes) {
            Ok(decoded) => decoded,
            Err(error) => panic!("checkpoint should decode: {error}"),
        };

        assert_eq!(decoded, solver);
        assert_eq!(decoded.epochs(), 7);
        assert_eq!(decoded.game(), Some(ResearchGame::Bridge));
    }

    #[test]
    fn scoped_checkpoint_rejects_query_for_another_game() {
        let solver = PortfolioSolver::for_game(ResearchGame::Bridge);
        let query = PortfolioSolver::bootstrap_query(ResearchGame::Cribbage);

        let error = solver.answer_checked(query).err();

        assert!(matches!(
            error,
            Some(super::PortfolioSolverError::GameMismatch {
                checkpoint_game: ResearchGame::Bridge,
                query_game: ResearchGame::Cribbage
            })
        ));
    }

    #[test]
    fn all_games_have_distinct_bootstrap_queries() {
        let mut seen = std::collections::BTreeSet::new();

        for game in ALL_RESEARCH_GAMES {
            let query = PortfolioSolver::bootstrap_query(game);

            assert!(seen.insert(query.info.game.slug()));
            assert_eq!(query.info.rule_file, game.rule_file());
        }
    }

    #[test]
    fn portfolio_routed_games_exclude_dedicated_solver_games() {
        assert_eq!(ALL_PORTFOLIO_ROUTED_GAMES.len(), 20);
        assert!(!ALL_PORTFOLIO_ROUTED_GAMES.contains(&ResearchGame::NlheHeadsUp));
        assert!(!ALL_PORTFOLIO_ROUTED_GAMES.contains(&ResearchGame::LiarsDice));
        assert!(
            ALL_PORTFOLIO_ROUTED_GAMES
                .iter()
                .all(|game| game.is_portfolio_routed())
        );
    }

    #[test]
    fn bridge_policy_uses_search_compatible_action() {
        let solver = PortfolioSolver::new();
        let response = solver.answer(PortfolioSolver::bootstrap_query(ResearchGame::Bridge));

        assert!(
            response
                .actions
                .iter()
                .any(|(action, _)| { matches!(action, PortfolioAction::DoubleDummyPlay) })
        );
    }

    #[test]
    fn strength_query_uses_rule_aware_bridge_engine() {
        let mut solver = PortfolioSolver::for_game(ResearchGame::Bridge);
        solver.train(4);
        let query = match PortfolioSolver::strength_query(ResearchGame::Bridge) {
            Ok(query) => query,
            Err(error) => panic!("bridge should have strength query: {error}"),
        };

        let response = match solver.answer_strength_checked(query.clone()) {
            Ok(response) => response,
            Err(error) => panic!("bridge strength query should answer: {error}"),
        };
        let report = match solver.strength_quality(query) {
            Ok(report) => report,
            Err(error) => panic!("bridge strength quality should report: {error}"),
        };

        assert!(response.is_valid());
        assert_eq!(report.game, ResearchGame::Bridge);
        assert_eq!(report.engine_tier.as_str(), "rule-aware");
        assert_eq!(report.iterations, 4);
    }

    #[test]
    fn strength_query_rejects_dedicated_solver_games() {
        let error = PortfolioSolver::strength_query(ResearchGame::LiarsDice).err();

        assert!(matches!(
            error,
            Some(super::PortfolioSolverError::UnsupportedPortfolioGame {
                game: ResearchGame::LiarsDice
            })
        ));
    }
}
