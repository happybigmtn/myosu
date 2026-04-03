use crate::game::{KuhnEdge, KuhnInfo};
use myosu_games::{StrategyQuery, StrategyResponse};

/// Wire-safe Kuhn poker strategy query type.
pub type KuhnStrategyQuery = StrategyQuery<KuhnInfo>;

/// Wire-safe Kuhn poker strategy response type.
pub type KuhnStrategyResponse = StrategyResponse<KuhnEdge>;

/// Pick the highest-probability action from a Kuhn poker strategy response.
pub fn recommended_edge(response: &KuhnStrategyResponse) -> Option<KuhnEdge> {
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
    use crate::game::KuhnEdge;
    use crate::protocol::{KuhnStrategyResponse, recommended_edge};

    #[test]
    fn recommended_edge_prefers_highest_probability_then_edge_order() {
        let response = KuhnStrategyResponse::new(vec![
            (KuhnEdge::Check, 0.4),
            (KuhnEdge::Bet, 0.4),
            (KuhnEdge::Call, 0.7),
        ]);

        assert_eq!(recommended_edge(&response), Some(KuhnEdge::Call));
    }

    #[test]
    fn recommended_edge_handles_empty_response() {
        assert_eq!(
            recommended_edge(&KuhnStrategyResponse::new(Vec::new())),
            None
        );
    }
}
