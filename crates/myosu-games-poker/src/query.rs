// query.rs: handle_query bridge for miner-validator communication
//
// Stateless query handler that deserializes NlheInfo, queries the solver strategy,
// and serializes the response as (NlheEdge, Probability) pairs

use rbp_nlhe::{NlheEdge, NlheInfo};
use thiserror::Error;

use crate::wire::{self, WireSerializable};

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("failed to deserialize info bytes: {0}")]
    InvalidInfoBytes(String),
    #[error("failed to serialize response: {0}")]
    SerializationError(String),
    #[error("wire error: {0}")]
    WireError(#[from] wire::WireError),
}

/// Wire format for strategy queries and responses
#[derive(Debug, Clone)]
pub struct WireStrategy {
    /// Serialized NlheInfo (query) or Vec<(NlheEdge, Prob)> (response)
    pub actions: Vec<u8>,
}

/// Handle a strategy query from a validator
///
/// Takes a WireStrategy containing serialized NlheInfo, queries the solver,
/// and returns a WireStrategy containing serialized action distribution
pub fn handle_query(query: &WireStrategy, solver: &crate::PokerSolver) -> Result<WireStrategy, QueryError> {
    // Deserialize the info set
    let info: NlheInfo = NlheInfo::deserialize(&query.actions)
        .map_err(|e| QueryError::InvalidInfoBytes(e.to_string()))?;

    // Query the solver's strategy
    let strategy = solver.strategy(&info);

    // Serialize the response
    let response_bytes = wire::serialize_action_distribution(&strategy)
        .map_err(|e| QueryError::SerializationError(e.to_string()))?;

    Ok(WireStrategy {
        actions: response_bytes,
    })
}

/// Validate that info_bytes can be deserialized as NlheInfo
pub fn validate_info_bytes(info_bytes: &[u8]) -> bool {
    NlheInfo::deserialize(info_bytes).is_ok()
}

/// Parse the response bytes into action distribution
pub fn parse_response(actions_bytes: &[u8]) -> Result<Vec<(NlheEdge, f64)>, QueryError> {
    wire::deserialize_action_distribution(actions_bytes)
        .map_err(|e| QueryError::SerializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_core::Arbitrary;

    #[test]
    fn handle_valid_query() {
        let solver = crate::PokerSolver::new().unwrap();
        let info = NlheInfo::random();
        let info_bytes = info.serialize().unwrap();

        let query = WireStrategy {
            actions: info_bytes,
        };

        let response = handle_query(&query, &solver).unwrap();
        let actions = parse_response(&response.actions).unwrap();

        // Verify probabilities sum to approximately 1.0
        let sum: f64 = actions.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 0.001 || sum == 0.0);
    }

    #[test]
    fn handle_invalid_info_bytes() {
        let solver = crate::PokerSolver::new().unwrap();
        let invalid_bytes = vec![0u8, 1, 2, 3, 4, 5];

        let query = WireStrategy {
            actions: invalid_bytes,
        };

        let result = handle_query(&query, &solver);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::InvalidInfoBytes(_)));
    }

    #[test]
    fn response_probabilities_sum_to_one() {
        let mut solver = crate::PokerSolver::new().unwrap();
        solver.train(100).unwrap();

        let info = NlheInfo::random();
        let info_bytes = info.serialize().unwrap();

        let query = WireStrategy {
            actions: info_bytes,
        };

        let response = handle_query(&query, &solver).unwrap();
        let actions = parse_response(&response.actions).unwrap();

        let sum: f64 = actions.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 0.001, "probabilities should sum to ~1.0, got {}", sum);
    }
}
