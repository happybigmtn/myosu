//! Query handler for miner-validator communication.
//!
//! Provides `handle_query`, a stateless function that:
//! 1. Deserializes the incoming `NlheInfo` from `info_bytes`
//! 2. Queries the solver's average strategy at that information set
//! 3. Serializes the action distribution into `action_bytes`
//!
//! The wire format uses bincode for `NlheInfo` and `NlheEdge` serialization.

use crate::solver::PokerSolver;
use crate::wire::WireSerializable;
use rbp_core::Probability;
use rbp_nlhe::{NlheEdge, NlheInfo};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during query handling.
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("failed to deserialize info bytes: {0}")]
    InfoDeserialization(#[source] bincode::Error),
    #[error("failed to deserialize action bytes: {0}")]
    ActionDeserialization(#[source] bincode::Error),
    #[error("info bytes are empty")]
    EmptyInfoBytes,
}

/// Wire format for strategy queries and responses.
///
/// Serialized as JSON containing `info_bytes` and `action_bytes`
/// which hold bincode-serialized `NlheInfo` and `Vec<(NlheEdge, Probability)>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireStrategy {
    /// Bincode-serialized `NlheInfo` identifying the information set.
    pub info_bytes: Vec<u8>,
    /// Bincode-serialized `Vec<(NlheEdge, Probability)>` action distribution.
    pub action_bytes: Vec<u8>,
}

impl WireStrategy {
    /// Creates a new wire strategy from info and action distribution.
    pub fn new(info: &NlheInfo, actions: &[(NlheEdge, Probability)]) -> Self {
        Self {
            info_bytes: info.to_bytes(),
            action_bytes: bincode::serialize(actions).expect("bincode serialization should not fail"),
        }
    }

    /// Deserializes the info from `info_bytes`.
    pub fn into_info(&self) -> Result<NlheInfo, QueryError> {
        if self.info_bytes.is_empty() {
            return Err(QueryError::EmptyInfoBytes);
        }
        NlheInfo::from_bytes(&self.info_bytes).map_err(QueryError::InfoDeserialization)
    }

    /// Deserializes the actions from `action_bytes`.
    pub fn into_actions(&self) -> Result<Vec<(NlheEdge, Probability)>, QueryError> {
        bincode::deserialize(&self.action_bytes).map_err(QueryError::ActionDeserialization)
    }
}

/// Handles a strategy query against the given solver.
///
/// This is a stateless function — it does not modify the solver.
/// The solver should be shared across queries via interior mutability
/// (e.g., `Arc<Mutex<PokerSolver>>`) in the calling binary.
///
/// # Arguments
///
/// * `query` — The wire-encoded strategy query containing `info_bytes`
/// * `solver` — Reference to the poker solver (typically shared)
///
/// # Returns
///
/// `Ok(WireStrategy)` containing the action distribution, or an error.
pub fn handle_query(
    query: &WireStrategy,
    solver: &PokerSolver,
) -> Result<WireStrategy, QueryError> {
    let info = query.into_info()?;
    let actions = solver.strategy(&info);
    Ok(WireStrategy::new(&info, &actions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::PokerSolver;
    use rbp_mccfr::{CfrGame, Encoder, Solver};

    #[test]
    fn handle_valid_query() {
        let solver = PokerSolver::new();
        let info = solver.inner().encoder().seed(&rbp_nlhe::NlheGame::root());
        let query_bytes = info.to_bytes();

        let query = WireStrategy {
            info_bytes: query_bytes,
            action_bytes: vec![],
        };

        let response = handle_query(&query, &solver).unwrap();

        // Verify action distribution is valid
        let actions = response.into_actions().unwrap();
        let sum: Probability = actions.iter().map(|(_, p)| *p).sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "action probabilities should sum to ~1.0, got {}",
            sum
        );
    }

    #[test]
    fn handle_invalid_info_bytes() {
        let solver = PokerSolver::new();
        let query = WireStrategy {
            info_bytes: vec![0xFF, 0xFE], // Invalid bytes
            action_bytes: vec![],
        };

        let result = handle_query(&query, &solver);
        assert!(result.is_err());
    }

    #[test]
    fn response_probabilities_sum_to_one() {
        let solver = PokerSolver::new();
        let info = solver.inner().encoder().seed(&rbp_nlhe::NlheGame::root());
        let query_bytes = info.to_bytes();

        let query = WireStrategy {
            info_bytes: query_bytes,
            action_bytes: vec![],
        };

        let response = handle_query(&query, &solver).unwrap();
        let actions = response.into_actions().unwrap();

        // Sum of probabilities should be approximately 1.0
        let sum: f32 = actions.iter().map(|(_, p)| *p).sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "probabilities sum to {}",
            sum
        );
    }
}
