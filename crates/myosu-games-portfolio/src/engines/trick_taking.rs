use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::rng::seed_for;
use crate::state::{PortfolioChallenge, TrickTakingChallenge};

pub(super) fn answer(challenge: &PortfolioChallenge, epochs: usize) -> EngineAnswer {
    match challenge {
        PortfolioChallenge::Bridge(state) => bridge(state, epochs),
        PortfolioChallenge::Spades(state) => spades(state),
        PortfolioChallenge::CallBreak(state) => call_break(state),
        PortfolioChallenge::Hearts(state) => hearts(state),
        _ => unreachable!("trick_taking::answer only accepts trick-taking challenges"),
    }
}

fn bridge(state: &TrickTakingChallenge, epochs: usize) -> EngineAnswer {
    let seed = seed_for(ResearchGame::Bridge, &state.spot.challenge_id, epochs, 0);
    let double_dummy = 1.0
        + f32::from(state.winners) * 0.55
        + f32::from(state.trump_count) * 0.15
        + f32::from(state.contract_pressure.max(0)) * 0.20
        + f32::from(state.cards_in_trick) * 0.10
        - if state.follow_suit_forced { 0.10 } else { 0.0 }
        + if seed & 1 == 0 { 0.05 } else { 0.0 };
    let follow_suit = 0.75
        + if state.follow_suit_forced { 0.80 } else { 0.10 }
        + f32::from(state.void_suits) * 0.25
        + f32::from(state.cards_in_trick) * 0.05;
    let bid_contract = 0.75
        + f32::from(state.contract_pressure.max(0)) * 0.42
        + if state.cards_in_trick == 0 { 0.30 } else { 0.0 };

    engine_answer(
        ResearchGame::Bridge,
        &state.spot.challenge_id,
        "state-aware bridge control heuristic",
        vec![
            (PortfolioAction::DoubleDummyPlay, double_dummy),
            (PortfolioAction::FollowSuit, follow_suit),
            (PortfolioAction::BidContract, bid_contract),
        ],
    )
}

fn spades(state: &TrickTakingChallenge) -> EngineAnswer {
    let trump_control = 1.0
        + f32::from(state.trump_count) * 0.55
        + f32::from(state.contract_pressure.max(0)) * 0.20
        + if !state.follow_suit_forced { 0.15 } else { 0.0 };
    let follow_suit = 0.7
        + if state.follow_suit_forced { 0.85 } else { 0.10 }
        + f32::from(state.winners) * 0.15
        + f32::from(state.cards_in_trick) * 0.04;
    let bid_nil =
        0.6 + if state.nil_viable { 1.00 } else { 0.0 } + f32::from(state.penalty_pressure) * 0.10
            - f32::from(state.trump_count) * 0.20
            - if state.follow_suit_forced { 0.25 } else { 0.0 };

    engine_answer(
        ResearchGame::Spades,
        &state.spot.challenge_id,
        "state-aware spades trump heuristic",
        vec![
            (PortfolioAction::TrumpControl, trump_control),
            (PortfolioAction::FollowSuit, follow_suit),
            (PortfolioAction::BidNil, bid_nil),
        ],
    )
}

fn call_break(state: &TrickTakingChallenge) -> EngineAnswer {
    let call_trump = 0.8
        + f32::from(state.trump_count) * 0.55
        + f32::from(state.contract_pressure.max(0)) * 0.30
        + if state.cards_in_trick == 0 { 0.35 } else { 0.0 };
    let trump_control = 0.9
        + f32::from(state.winners) * 0.35
        + f32::from(state.void_suits) * 0.15
        + if !state.follow_suit_forced { 0.20 } else { 0.0 };
    let follow_suit =
        0.7 + if state.follow_suit_forced { 0.70 } else { 0.0 } + f32::from(state.winners) * 0.12;

    engine_answer(
        ResearchGame::CallBreak,
        &state.spot.challenge_id,
        "state-aware call-break trick heuristic",
        vec![
            (PortfolioAction::CallTrump, call_trump),
            (PortfolioAction::TrumpControl, trump_control),
            (PortfolioAction::FollowSuit, follow_suit),
        ],
    )
}

fn hearts(state: &TrickTakingChallenge) -> EngineAnswer {
    let avoid_penalty = 0.9
        + f32::from(state.penalty_pressure) * 0.65
        + if state.moon_shot_viable { 0.0 } else { 0.35 }
        + if state.follow_suit_forced { 0.15 } else { 0.0 };
    let follow_suit = 0.7
        + if state.follow_suit_forced { 0.75 } else { 0.10 }
        + f32::from(state.winners) * 0.08
        + f32::from(state.void_suits) * 0.10;
    let shoot_moon =
        0.55 + if state.moon_shot_viable { 1.15 } else { 0.0 } + f32::from(state.winners) * 0.20
            - if state.follow_suit_forced { 0.20 } else { 0.0 };

    engine_answer(
        ResearchGame::Hearts,
        &state.spot.challenge_id,
        "state-aware hearts penalty heuristic",
        vec![
            (PortfolioAction::AvoidPenalty, avoid_penalty),
            (PortfolioAction::FollowSuit, follow_suit),
            (PortfolioAction::ShootMoon, shoot_moon),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::trick_taking::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, TrickTakingChallenge};

    #[test]
    fn bridge_prefers_double_dummy_play() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::Bridge)
                .unwrap_or_else(|| panic!("bridge challenge missing")),
            2,
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::DoubleDummyPlay)
        );
    }

    #[test]
    fn spades_nil_window_changes_top_action() {
        let aggressive = PortfolioChallenge::Spades(TrickTakingChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Spades,
                "trump-heavy",
                "Trump-heavy spades spot",
            ),
            trump_count: 4,
            winners: 3,
            void_suits: 1,
            contract_pressure: 1,
            penalty_pressure: 0,
            cards_in_trick: 1,
            follow_suit_forced: false,
            nil_viable: false,
            moon_shot_viable: false,
        });
        let nil = PortfolioChallenge::Spades(TrickTakingChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Spades,
                "nil-window",
                "Nil-viable spades spot",
            ),
            trump_count: 0,
            winners: 0,
            void_suits: 0,
            contract_pressure: 0,
            penalty_pressure: 2,
            cards_in_trick: 0,
            follow_suit_forced: false,
            nil_viable: true,
            moon_shot_viable: false,
        });

        assert_eq!(
            recommended_action(&answer(&aggressive, 0).response),
            Some(PortfolioAction::TrumpControl)
        );
        assert_eq!(
            recommended_action(&answer(&nil, 0).response),
            Some(PortfolioAction::BidNil)
        );
    }

    #[test]
    fn bridge_opening_contract_pressure_can_shift_to_bidding() {
        let answer = answer(
            &PortfolioChallenge::Bridge(TrickTakingChallenge {
                spot: PortfolioChallengeSpot::scenario(
                    ResearchGame::Bridge,
                    "opening-contract-push",
                    "Lead-open bridge spot with contract pressure",
                ),
                trump_count: 0,
                winners: 1,
                void_suits: 0,
                contract_pressure: 5,
                penalty_pressure: 0,
                cards_in_trick: 0,
                follow_suit_forced: false,
                nil_viable: false,
                moon_shot_viable: false,
            }),
            0,
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::BidContract)
        );
    }
}
