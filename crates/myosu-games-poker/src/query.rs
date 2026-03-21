//! Query handler for miner-validator communication.
//!
//! `handle_query` bridges the wire format (`WireStrategy`) with the solver's
//! strategy lookup. Stateless over `PokerSolver`; delegates to `strategy()`.

use super::wire::{WireError, WireStrategy};
use super::PokerSolver;
use rbp_nlhe::NlheInfo;
use thiserror::Error;

/// Errors from query handling.
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("wire deserialization failed: {0}")]
    Wire(#[from] WireError),

    #[error("solver error: {0}")]
    Solver(String),

    #[error("invalid info bytes: query must contain valid NlheInfo")]
    InvalidInfo,
}

/// Handle a strategy query received from a validator.
///
/// Takes a `WireStrategy` with `info_bytes` set (serialized `NlheInfo`),
/// looks up the strategy via the solver, and fills in `actions_bytes`.
///
/// Returns the modified `WireStrategy` with both fields populated.
pub fn handle_query(
    query: &WireStrategy,
    solver: &PokerSolver,
) -> Result<WireStrategy, QueryError> {
    // Deserialize the info set
    let info: NlheInfo = query.info().map_err(|_| QueryError::InvalidInfo)?;

    // Query the solver's averaged strategy
    let distribution = solver.strategy(&info);

    // Build response
    let mut response = WireStrategy::new(query.info_bytes.clone());
    response.set_actions(&distribution);

    Ok(response)
}

/// Handle a raw query from bytes.
///
/// Convenience wrapper that deserializes the query, processes it, and
/// returns raw bytes for the response.
pub fn handle_query_bytes(
    query_bytes: &[u8],
    solver: &PokerSolver,
) -> Result<Vec<u8>, QueryError> {
    let query: WireStrategy =
        serde_json::from_slice(query_bytes).map_err(|e| QueryError::Wire(WireError::Decode(e)))?;

    let response = handle_query(&query, solver)?;

    serde_json::to_vec(&response)
        .map_err(|e| QueryError::Solver(format!("failed to serialize response: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::wire::WireSerializable;
    use rbp_nlhe::NlheGame;
    use rbp_mccfr::{CfrGame, Encoder};

    #[test]
    fn handle_valid_query() {
        let mut solver = PokerSolver::new();
        solver.train(20);

        // Get root info and serialize
        let info = solver.encoder().seed(&NlheGame::root());
        let info_bytes = info.to_wire().unwrap();

        let query = WireStrategy::new(info_bytes);
        let response = handle_query(&query, &solver).unwrap();

        // Verify response has actions
        let actions = response.actions().unwrap();
        assert!(!actions.is_empty(), "response should contain actions");

        // Verify probabilities sum to ~1.0
        let sum: f32 = actions.iter().map(|(_, p)| p).sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "action probabilities should sum to ~1.0, got {}",
            sum
        );
    }

    #[test]
    fn handle_invalid_info_bytes() {
        let solver = PokerSolver::new();
        let query = WireStrategy::new(vec![0u8; 4]); // invalid bytes

        let result = handle_query(&query, &solver);
        assert!(
            result.is_err(),
            "invalid info bytes should produce an error"
        );
    }

    #[test]
    fn response_probabilities_sum_to_one() {
        let mut solver = PokerSolver::new();
        solver.train(50);

        let info = solver.encoder().seed(&NlheGame::root());
        let info_bytes = info.to_wire().unwrap();

        let query = WireStrategy::new(info_bytes);
        let response = handle_query(&query, &solver).unwrap();

        let actions = response.actions().unwrap();
        let sum: f32 = actions.iter().map(|(_, p)| p).sum();

        assert!(
            (sum - 1.0).abs() < 0.001,
            "response probabilities must sum to ~1.0, got {}",
            sum
        );
    }
}
