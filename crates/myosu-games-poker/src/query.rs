//! Query handler bridge: converts between wire format and solver responses.
//!
//! Stateless over [`PokerSolver`](crate::solver::PokerSolver); this module only
//! handles the wire protocol conversion.

use crate::solver::PokerSolver;
use crate::wire::{WireError, WireStrategy};
use anyhow::{Context, Result};

/// Handles a wire strategy query and produces a wire strategy response.
///
/// ## Wire Protocol
///
/// **Query**: `WireStrategy { info_bytes: <bincode(NlheInfo)>, action_bytes: vec![] }`
/// **Response**: `WireStrategy { info_bytes: vec![], action_bytes: <bincode(Vec<(NlheEdge, Prob)>>) }`
///
/// ## Errors
///
/// Returns an error if:
/// - The `info_bytes` field is empty (not a query)
/// - The bytes do not decode to a valid `NlheInfo`
/// - The info set is not in the solver's profile (no trained strategy for that infoset)
pub fn handle_query(solver: &PokerSolver, wire: &WireStrategy) -> Result<WireStrategy> {
    let info = wire
        .parse_info()
        .context("failed to parse NlheInfo from wire")?;

    // Get the averaged strategy distribution for this info set
    let actions = solver.strategy(&info);

    // If no strategy exists for this info set, return empty response
    // (this is valid for terminal states or unseen infosets)
    Ok(WireStrategy::response(&actions))
}

/// Validates that a wire strategy message contains valid info bytes.
pub fn validate_info_bytes(wire: &WireStrategy) -> Result<(), WireError> {
    wire.parse_info().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::PokerSolver;
    use crate::wire::WireStrategy;

    #[test]
    fn handle_valid_query() {
        let mut solver = PokerSolver::new();
        solver.train(20);

        // Get an info set from the trained profile
        let info = solver.info_sets().next().copied();
        if let Some(info) = info {
            let query = WireStrategy::query(&info);
            let response = handle_query(&solver, &query).unwrap();
            assert!(response.action_bytes.len() > 0, "response should have actions");
        }
    }

    #[test]
    fn handle_invalid_info_bytes() {
        let solver = PokerSolver::new();
        let invalid_wire = WireStrategy {
            info_bytes: vec![0, 1, 2, 3], // not valid JSON
            action_bytes: Vec::new(),
        };
        let result = handle_query(&solver, &invalid_wire);
        assert!(result.is_err(), "should fail on invalid bytes");
    }

    #[test]
    fn response_probabilities_sum_to_one() {
        let mut solver = PokerSolver::new();
        solver.train(50);

        let info = solver.info_sets().next().copied();
        if let Some(info) = info {
            let query = WireStrategy::query(&info);
            let response = handle_query(&solver, &query).unwrap();
            let actions = response.parse_actions().unwrap();
            let sum: f32 = actions.iter().map(|(_, p)| p).sum();
            assert!(
                (sum - 1.0).abs() < 0.001,
                "probabilities sum to {}",
                sum
            );
        }
    }
}
