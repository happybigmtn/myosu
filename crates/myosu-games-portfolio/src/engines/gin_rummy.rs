use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::PortfolioChallenge;

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    let state = match challenge {
        PortfolioChallenge::GinRummy(state) => state,
        _ => unreachable!("gin_rummy::answer only accepts gin-rummy challenges"),
    };

    let knock =
        0.8 + if state.gin_available {
            1.35
        } else if state.knock_available {
            0.95
        } else {
            0.0
        } + f32::from(state.meld_count) * 0.22
            - f32::from(state.deadwood) * 0.03;
    let discard_deadwood = 0.85
        + f32::from(state.deadwood) * 0.06
        + f32::from(state.discard_options) * 0.08
        + if state.live_draws <= 1 { 0.20 } else { 0.0 }
        - if state.knock_available { 0.35 } else { 0.0 };
    let pot_control = 0.75
        + f32::from(state.live_draws) * 0.08
        + if !state.knock_available { 0.20 } else { 0.0 }
        + if state.meld_count <= 1 { 0.15 } else { 0.0 };

    engine_answer(
        ResearchGame::GinRummy,
        &state.spot.challenge_id,
        "state-aware meld-distance heuristic",
        vec![
            (PortfolioAction::Knock, knock),
            (PortfolioAction::DiscardDeadwood, discard_deadwood),
            (PortfolioAction::PotControl, pot_control),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::gin_rummy::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{GinRummyChallenge, PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn melded_reference_state_can_knock() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::GinRummy)
                .unwrap_or_else(|| panic!("gin challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::Knock)
        );
    }

    #[test]
    fn high_deadwood_state_prefers_discarding() {
        let challenge = PortfolioChallenge::GinRummy(GinRummyChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::GinRummy,
                "high-deadwood",
                "High-deadwood rebuild spot",
            ),
            deadwood: 18,
            meld_count: 1,
            live_draws: 3,
            knock_available: false,
            gin_available: false,
            discard_options: 4,
        });

        let answer = answer(&challenge);

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::DiscardDeadwood)
        );
    }

    #[test]
    fn gin_ready_state_amplifies_knock_pressure() {
        let challenge = PortfolioChallenge::GinRummy(GinRummyChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::GinRummy,
                "gin-ready",
                "Zero-deadwood gin conversion spot",
            ),
            deadwood: 0,
            meld_count: 3,
            live_draws: 1,
            knock_available: true,
            gin_available: true,
            discard_options: 1,
        });

        let answer = answer(&challenge);

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::Knock)
        );
        assert!(
            answer.response.probability_for(&PortfolioAction::Knock)
                > answer
                    .response
                    .probability_for(&PortfolioAction::DiscardDeadwood)
        );
    }
}
