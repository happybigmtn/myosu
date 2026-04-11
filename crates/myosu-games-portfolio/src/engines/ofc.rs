use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::PortfolioChallenge;

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    let state = match challenge {
        PortfolioChallenge::OfcChinesePoker(state) => state,
        _ => unreachable!("ofc::answer only accepts OFC challenges"),
    };

    let place_safe = 1.0
        + f32::from(state.free_slots) * 0.32
        + f32::from(state.foul_pressure) * 0.85
        + if state.foul_pressure == 0 { 0.45 } else { 0.90 };
    let draw_to_nuts =
        0.8 + f32::from(state.fantasyland_outs) * 0.45 + f32::from(state.back_strength) * 0.04
            - f32::from(state.foul_pressure) * 0.30;
    let pot_control = 0.75
        + f32::from(state.middle_strength) * 0.07
        + f32::from(state.front_strength) * 0.04
        + if state.free_slots <= 2 { 0.20 } else { 0.0 }
        - f32::from(state.foul_pressure) * 0.15;

    engine_answer(
        ResearchGame::OfcChinesePoker,
        &state.spot.challenge_id,
        "state-aware foul-safe placement heuristic",
        vec![
            (PortfolioAction::PlaceSafe, place_safe),
            (PortfolioAction::DrawToNuts, draw_to_nuts),
            (PortfolioAction::PotControl, pot_control),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::ofc::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{OfcChallenge, PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn safe_reference_state_prefers_safe_placement() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::OfcChinesePoker)
                .unwrap_or_else(|| panic!("ofc challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::PlaceSafe)
        );
    }

    #[test]
    fn foul_risk_makes_safe_placement_more_urgent() {
        let answer = answer(&PortfolioChallenge::OfcChinesePoker(OfcChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::OfcChinesePoker,
                "foul-risk",
                "Unsafe row ordering",
            ),
            front_strength: 6,
            middle_strength: 5,
            back_strength: 7,
            free_slots: 2,
            fantasyland_outs: 0,
            foul_pressure: 1,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::PlaceSafe)
        );
        assert!(
            answer.response.probability_for(&PortfolioAction::PlaceSafe)
                > answer
                    .response
                    .probability_for(&PortfolioAction::DrawToNuts)
        );
    }

    #[test]
    fn fantasyland_pressure_can_overrule_safe_default() {
        let answer = answer(&PortfolioChallenge::OfcChinesePoker(OfcChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::OfcChinesePoker,
                "fantasyland-live",
                "Clean ordering with strong fantasy-land upside",
            ),
            front_strength: 3,
            middle_strength: 8,
            back_strength: 12,
            free_slots: 1,
            fantasyland_outs: 4,
            foul_pressure: 0,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::DrawToNuts)
        );
    }
}
