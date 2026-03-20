//! Query handler for miner-validator communication.
//!
//! Handles `StrategyQuery` / `StrategyResponse` bridging for NLHE.

use crate::solver::{strategy, PokerSolver};
use crate::wire::WireSerializable;
use rbp_mccfr::{Solver, Encoder, CfrGame};
use rbp_nlhe::NlheInfo;
use myosu_games::StrategyResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("invalid info bytes: {0}")]
    InvalidInfo(String),
    #[error("info deserialization failed: {0}")]
    InfoDecode(String),
    #[error("action deserialization failed: {0}")]
    ActionDecode(String),
    #[error("strategy computation failed: {0}")]
    Strategy(String),
}

/// Wire format for NLHE strategy queries.
///
/// The `info_bytes` field contains bincode-serialized `NlheInfo`.
#[derive(Debug, Clone)]
pub struct WireStrategy {
    pub info_bytes: Vec<u8>,
    pub actions_bytes: Vec<u8>,
}

impl WireStrategy {
    /// Create a new wire strategy from info bytes and action bytes.
    pub fn new(info_bytes: Vec<u8>, actions_bytes: Vec<u8>) -> Self {
        Self { info_bytes, actions_bytes }
    }
}

/// Handle a strategy query from a validator.
///
/// Takes a `WireStrategy` containing bincode-serialized `NlheInfo`,
/// queries the solver for action probabilities, and returns a
/// `WireStrategy` containing the action distribution.
pub fn handle_query(
    solver: &PokerSolver,
    wire: &WireStrategy,
) -> Result<WireStrategy, QueryError> {
    // Deserialize the info set
    let info: NlheInfo = NlheInfo::from_bytes(&wire.info_bytes)
        .map_err(|e| QueryError::InvalidInfo(format!("{:?}", e)))?;

    // Get strategy distribution
    let dist = strategy(solver, &info);

    // Serialize action distribution
    let actions_bytes = crate::wire::serialize_actions(&dist)
        .map_err(|e| QueryError::ActionDecode(format!("{:?}", e)))?;

    Ok(WireStrategy::new(wire.info_bytes.clone(), actions_bytes))
}

/// Validate info bytes and check they deserialize to a valid NlheInfo.
pub fn validate_info_bytes(bytes: &[u8]) -> Result<(), QueryError> {
    let _: NlheInfo = NlheInfo::from_bytes(bytes)
        .map_err(|e| QueryError::InvalidInfo(format!("{:?}", e)))?;
    Ok(())
}

/// Deserialize a wire strategy response and validate probability sums.
pub fn deserialize_response(
    wire: &WireStrategy,
) -> Result<StrategyResponse<rbp_nlhe::NlheEdge>, QueryError> {
    let actions = crate::wire::deserialize_actions(&wire.actions_bytes)
        .map_err(|e| QueryError::ActionDecode(format!("{:?}", e)))?;
    Ok(StrategyResponse::new(actions))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::{create_empty_solver, train};
    use crate::wire::WireSerializable;

    #[test]
    fn handle_valid_query() {
        let mut solver = create_empty_solver();
        train(&mut solver, 50);

        // Create a valid info set
        let info = solver.encoder().seed(&rbp_nlhe::NlheGame::root());
        let info_bytes = info.to_bytes().unwrap();

        let wire_in = WireStrategy::new(info_bytes, vec![]);
        let result = handle_query(&solver, &wire_in);

        assert!(result.is_ok());
        let wire_out = result.unwrap();
        assert!(!wire_out.actions_bytes.is_empty());
    }

    #[test]
    fn handle_invalid_info_bytes() {
        let solver = create_empty_solver();
        let invalid_bytes = vec![0u8, 1, 2, 3, 4, 5, 6, 7];

        let wire_in = WireStrategy::new(invalid_bytes, vec![]);
        let result = handle_query(&solver, &wire_in);

        assert!(result.is_err());
    }

    #[test]
    fn response_probabilities_sum_to_one() {
        let mut solver = create_empty_solver();
        train(&mut solver, 100);

        let info = solver.encoder().seed(&rbp_nlhe::NlheGame::root());
        let info_bytes = info.to_bytes().unwrap();

        let wire_in = WireStrategy::new(info_bytes, vec![]);
        let wire_out = handle_query(&solver, &wire_in).unwrap();

        let response = deserialize_response(&wire_out).unwrap();
        assert!(response.is_valid());
    }
}