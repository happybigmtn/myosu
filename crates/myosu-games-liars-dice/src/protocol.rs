use crate::game::{LiarsDiceEdge, LiarsDiceInfo};
use myosu_games::{StrategyQuery, StrategyResponse};

/// Wire-safe Liar's Dice strategy query type.
pub type LiarsDiceStrategyQuery = StrategyQuery<LiarsDiceInfo>;

/// Wire-safe Liar's Dice strategy response type.
pub type LiarsDiceStrategyResponse = StrategyResponse<LiarsDiceEdge>;

/// Pick the highest-probability action from a Liar's Dice strategy response.
pub fn recommended_edge(response: &LiarsDiceStrategyResponse) -> Option<LiarsDiceEdge> {
    response
        .actions
        .iter()
        .copied()
        .max_by(|(left_edge, left_prob), (right_edge, right_prob)| {
            left_prob
                .total_cmp(right_prob)
                .then_with(|| left_edge.cmp(right_edge))
        })
        .map(|(edge, _)| edge)
}

#[cfg(test)]
mod tests {
    use crate::game::{LiarsDiceClaim, LiarsDiceEdge};
    use crate::protocol::{LiarsDiceStrategyResponse, recommended_edge};

    #[test]
    fn recommended_edge_prefers_highest_probability_then_edge_order() {
        let response = LiarsDiceStrategyResponse::new(vec![
            (
                LiarsDiceEdge::Bid(LiarsDiceClaim::new(1, 3).expect("claim should build")),
                0.4,
            ),
            (LiarsDiceEdge::Challenge, 0.4),
            (
                LiarsDiceEdge::Bid(LiarsDiceClaim::new(2, 1).expect("claim should build")),
                0.7,
            ),
        ]);

        assert_eq!(
            recommended_edge(&response),
            Some(LiarsDiceEdge::Bid(
                LiarsDiceClaim::new(2, 1).expect("claim should build")
            ))
        );
    }

    #[test]
    fn recommended_edge_handles_empty_response() {
        assert_eq!(
            recommended_edge(&LiarsDiceStrategyResponse::new(Vec::new())),
            None
        );
    }
}
