//! Query handler bridge for miner-validator communication.
//!
//! Provides `handle_query` which converts a wire-format strategy query
//! (containing serialized `NlheInfo`) into a `StrategyResponse` with
//! action probabilities.

use crate::wire::WireSerializable;
use crate::{PokerSolver, StrategyQuery, StrategyResponse};
use anyhow::{Context, Result};
use rbp_core::Probability;
use rbp_mccfr::Encoder;
use rbp_nlhe::NlheInfo;

/// Errors that can occur during query handling.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("failed to deserialize NlheInfo from info_bytes")]
    InvalidInfoBytes(#[source] anyhow::Error),

    #[error("query response has invalid probability distribution: {0}")]
    InvalidDistribution(String),

    #[error("no actions available for this information set")]
    NoActionsAvailable,
}

/// Handle a strategy query from a validator or client.
///
/// Takes a `StrategyQuery` containing serialized `NlheInfo` bytes,
/// queries the solver for the strategy distribution, and returns
/// a `StrategyResponse` with action probabilities.
///
/// # Arguments
/// - `query` — The strategy query containing wire-serialized `NlheInfo`
/// - `solver` — The poker solver to query
///
/// # Returns
/// A `StrategyResponse` with (edge, probability) pairs, or an error if
/// the query cannot be processed.
pub fn handle_query(
    query: &StrategyQuery<Vec<u8>>,
    solver: &PokerSolver,
) -> Result<StrategyResponse<u64>> {
    // Deserialize the info set from wire format
    let info = NlheInfo::from_bytes(&query.info)
        .map_err(|e| QueryError::InvalidInfoBytes(e.into()))
        .context("failed to deserialize NlheInfo from query")?;

    // Get strategy distribution from solver
    let distribution = solver.strategy(&info);

    if distribution.is_empty() {
        return Err(QueryError::NoActionsAvailable.into());
    }

    // Convert to response format with (encoded_edge, probability) pairs
    let actions: Vec<(u64, Probability)> = distribution
        .iter()
        .map(|(edge, prob)| {
            let encoded = edge.to_bytes().expect("edge serialization should not fail");
            let key = u64::from_le_bytes(encoded.try_into().expect("edge bytes should be 8 or less"));
            (key, *prob)
        })
        .collect();

    let response = StrategyResponse::new(actions);

    // Validate distribution
    if !response.is_valid() {
        let sum: Probability = response.actions.iter().map(|(_, p)| p).sum();
        return Err(QueryError::InvalidDistribution(format!(
            "probabilities sum to {} instead of 1.0",
            sum
        ))
        .into());
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::WireSerializable;
    use rbp_mccfr::CfrGame;

    #[test]
    fn handle_valid_query() {
        let mut solver = PokerSolver::new();
        solver.train(50);

        // Create a query at the root info set
        let encoder = rbp_nlhe::NlheEncoder::default();
        let root_info = encoder.seed(&rbp_nlhe::NlheGame::root());
        let info_bytes = root_info.to_bytes().unwrap();

        let query = StrategyQuery::new(info_bytes);
        let response = handle_query(&query, &solver).unwrap();

        assert!(!response.actions.is_empty());
        assert!(response.is_valid());
    }

    #[test]
    fn handle_invalid_info_bytes() {
        let solver = PokerSolver::new();
        let query = StrategyQuery::new(vec![0xFF, 0xFE, 0xFD]); // Invalid bytes

        let result = handle_query(&query, &solver);
        assert!(result.is_err());
    }

    #[test]
    fn response_probabilities_sum_to_one() {
        let mut solver = PokerSolver::new();
        solver.train(100);

        let encoder = rbp_nlhe::NlheEncoder::default();
        let root_info = encoder.seed(&rbp_nlhe::NlheGame::root());
        let info_bytes = root_info.to_bytes().unwrap();

        let query = StrategyQuery::new(info_bytes);
        let response = handle_query(&query, &solver).unwrap();

        let sum: Probability = response.actions.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 0.001);
    }
}
