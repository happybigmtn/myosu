use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::PortfolioChallenge;

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    let state = match challenge {
        PortfolioChallenge::Stratego(state) => state,
        _ => unreachable!("stratego::answer only accepts stratego challenges"),
    };

    let scout = 0.85
        + f32::from(state.scout_lanes) * 0.55
        + f32::from(state.hidden_targets) * 0.45
        + f32::from(state.bombs_suspected) * 0.05
        + if state.attack_is_forced { -0.40 } else { 0.15 };
    let place_safe = 0.8
        + f32::from(state.bombs_suspected) * 0.30
        + if state.attack_targets == 0 { 0.25 } else { 0.0 }
        + if state.miners_remaining == 0 {
            0.25
        } else {
            0.0
        }
        + if state.attack_is_forced { -0.20 } else { 0.10 };
    let advance_piece = 0.8
        + f32::from(state.attack_targets) * 0.60
        + if state.attack_is_forced { 0.95 } else { 0.0 }
        + if state.bombs_suspected > state.miners_remaining {
            -0.10
        } else {
            0.10
        };

    engine_answer(
        ResearchGame::Stratego,
        &state.spot.challenge_id,
        "state-aware belief-scout heuristic",
        vec![
            (PortfolioAction::Scout, scout),
            (PortfolioAction::PlaceSafe, place_safe),
            (PortfolioAction::AdvancePiece, advance_piece),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::stratego::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, StrategoChallenge};

    #[test]
    fn open_scout_lane_prefers_scouting() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::Stratego)
                .unwrap_or_else(|| panic!("stratego challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::Scout)
        );
    }

    #[test]
    fn forced_attack_state_prefers_advancing() {
        let answer = answer(&PortfolioChallenge::Stratego(StrategoChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Stratego,
                "forced-attack",
                "Forced combat lane",
            ),
            scout_lanes: 0,
            miners_remaining: 2,
            bombs_suspected: 0,
            attack_targets: 1,
            hidden_targets: 0,
            attack_is_forced: true,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::AdvancePiece)
        );
    }

    #[test]
    fn hidden_targets_keep_scouting_live() {
        let answer = answer(&PortfolioChallenge::Stratego(StrategoChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Stratego,
                "hidden-targets",
                "Scout has multiple hidden lanes and no forced combat",
            ),
            scout_lanes: 2,
            miners_remaining: 1,
            bombs_suspected: 2,
            attack_targets: 1,
            hidden_targets: 1,
            attack_is_forced: false,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::Scout)
        );
    }
}
