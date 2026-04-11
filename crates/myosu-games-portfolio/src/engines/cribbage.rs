use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::PortfolioChallenge;

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    let state = match challenge {
        PortfolioChallenge::Cribbage(state) => state,
        _ => unreachable!("cribbage::answer only accepts cribbage challenges"),
    };

    let peg_run = 0.85
        + f32::from(state.run_potential) * 0.50
        + f32::from(state.max_immediate_points) * 0.22
        + f32::from(state.fifteen_outs) * 0.32
        + if state.go_window { 0.15 } else { 0.0 };
    let keep_crib = 0.8
        + f32::from(state.crib_edge.max(0) as u8) * 0.45
        + if state.max_immediate_points <= 1 {
            0.18
        } else {
            0.0
        }
        + if state.pair_trap { 0.12 } else { 0.0 };
    let discard_deadwood = 0.8
        + if state.pair_trap { 0.45 } else { 0.0 }
        + if state.max_immediate_points <= 1 {
            0.25
        } else {
            0.0
        }
        + if state.go_window { 0.0 } else { 0.20 }
        + if state.crib_edge < 0 { 0.20 } else { 0.0 };

    engine_answer(
        ResearchGame::Cribbage,
        &state.spot.challenge_id,
        "state-aware pegging-crib heuristic",
        vec![
            (PortfolioAction::PegRun, peg_run),
            (PortfolioAction::KeepCrib, keep_crib),
            (PortfolioAction::DiscardDeadwood, discard_deadwood),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::cribbage::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{CribbageChallenge, PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn cribbage_reference_state_prefers_pegging_run() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::Cribbage)
                .unwrap_or_else(|| panic!("cribbage challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::PegRun)
        );
    }

    #[test]
    fn discard_pressure_can_overtake_pegging() {
        let answer = answer(&PortfolioChallenge::Cribbage(CribbageChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Cribbage,
                "discard-pressure",
                "Cribbage discard pressure spot",
            ),
            pegging_count: 6,
            run_potential: 0,
            crib_edge: -1,
            pair_trap: true,
            go_window: false,
            fifteen_outs: 0,
            max_immediate_points: 0,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::DiscardDeadwood)
        );
    }

    #[test]
    fn strong_crib_edge_can_beat_live_pegging() {
        let answer = answer(&PortfolioChallenge::Cribbage(CribbageChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Cribbage,
                "keep-crib-window",
                "Dry pegging spot with strong crib equity",
            ),
            pegging_count: 4,
            run_potential: 1,
            crib_edge: 2,
            pair_trap: false,
            go_window: false,
            fifteen_outs: 0,
            max_immediate_points: 1,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::KeepCrib)
        );
    }
}
