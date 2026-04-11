use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::{PokerLikeChallenge, PortfolioChallenge};

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    match challenge {
        PortfolioChallenge::NlheSixMax(state) => nlhe_six_max(state),
        PortfolioChallenge::Plo(state) => plo(state),
        PortfolioChallenge::NlheTournament(state) => nlhe_tournament(state),
        PortfolioChallenge::ShortDeck(state) => short_deck(state),
        PortfolioChallenge::TeenPatti(state) => teen_patti(state),
        _ => unreachable!("poker_like::answer only accepts poker-like challenges"),
    }
}

fn nlhe_six_max(state: &PokerLikeChallenge) -> EngineAnswer {
    let value_bet = 1.0
        + f32::from(state.made_strength) * 0.75
        + if state.in_position { 0.35 } else { 0.0 }
        + if state.to_call_bb > 0 { 0.10 } else { 0.0 }
        - if !state.raise_available { 0.15 } else { 0.0 };
    let tight_open = 0.85
        + f32::from(state.fold_equity) * 0.45
        + if state.in_position { 0.20 } else { 0.0 }
        + if state.to_call_bb == 0 { 0.35 } else { 0.0 }
        + if state.active_players > 2 { 0.10 } else { 0.0 };
    let pot_control = 0.9
        + f32::from(state.draw_strength) * 0.20
        + f32::from(state.effective_stack_bb.min(120)) / 120.0
        + if state.check_available { 0.30 } else { 0.0 };

    engine_answer(
        ResearchGame::NlheSixMax,
        &state.spot.challenge_id,
        "state-aware poker range heuristic",
        vec![
            (PortfolioAction::ValueBet, value_bet),
            (PortfolioAction::TightOpen, tight_open),
            (PortfolioAction::PotControl, pot_control),
        ],
    )
}

fn plo(state: &PokerLikeChallenge) -> EngineAnswer {
    let draw_to_nuts = 1.0
        + f32::from(state.draw_strength) * 0.80
        + f32::from(state.pot_bb.min(40)) / 30.0
        + if state.to_call_bb > 0 { 0.20 } else { 0.0 };
    let pot_sized_raise = 0.85
        + f32::from(state.made_strength) * 0.45
        + f32::from(state.fold_equity) * 0.25
        + if state.raise_available { 0.25 } else { -0.30 };
    let pot_control = 0.8
        + f32::from(state.effective_stack_bb.min(120)) / 140.0
        + if state.check_available { 0.35 } else { 0.0 }
        + f32::from(state.active_players) * 0.03
        - if state.to_call_bb > 0 { 0.15 } else { 0.0 };

    engine_answer(
        ResearchGame::Plo,
        &state.spot.challenge_id,
        "state-aware PLO nut-draw heuristic",
        vec![
            (PortfolioAction::DrawToNuts, draw_to_nuts),
            (PortfolioAction::PotSizedRaise, pot_sized_raise),
            (PortfolioAction::PotControl, pot_control),
        ],
    )
}

fn nlhe_tournament(state: &PokerLikeChallenge) -> EngineAnswer {
    let short_stack_pressure =
        f32::from(20u16.saturating_sub(state.effective_stack_bb.min(20))) / 6.0;
    let push_fold = 1.0
        + f32::from(state.fold_equity) * 0.55
        + short_stack_pressure
        + f32::from(state.icm_pressure) * 0.15
        + if state.to_call_bb > 0 { 0.20 } else { 0.0 }
        - if !state.raise_available { 0.40 } else { 0.0 };
    let icm_fold = 0.9
        + f32::from(state.icm_pressure) * 0.70
        + if state.made_strength <= 1 { 0.55 } else { 0.0 }
        + if state.to_call_bb > 0 { 0.35 } else { 0.0 };
    let value_bet = 0.8
        + f32::from(state.made_strength) * 0.45
        + if state.check_available { 0.15 } else { 0.0 };

    engine_answer(
        ResearchGame::NlheTournament,
        &state.spot.challenge_id,
        "state-aware ICM pressure heuristic",
        vec![
            (PortfolioAction::PushFold, push_fold),
            (PortfolioAction::IcmFold, icm_fold),
            (PortfolioAction::ValueBet, value_bet),
        ],
    )
}

fn short_deck(state: &PokerLikeChallenge) -> EngineAnswer {
    let value_bet = 1.0
        + f32::from(state.made_strength) * 0.55
        + f32::from(state.draw_strength) * 0.20
        + if state.active_players <= 2 { 0.10 } else { 0.0 };
    let ante_steal = 0.8
        + f32::from(state.fold_equity) * 0.60
        + f32::from(state.pot_bb.min(20)) / 40.0
        + if state.to_call_bb == 0 { 0.45 } else { 0.0 }
        + if state.active_players > 2 { 0.10 } else { 0.0 }
        + if state.raise_available { 0.10 } else { -0.30 };
    let pot_control = 0.8
        + f32::from(state.effective_stack_bb.min(60)) / 80.0
        + if state.check_available { 0.40 } else { 0.0 }
        + if state.to_call_bb > 0 { 0.15 } else { 0.0 };

    engine_answer(
        ResearchGame::ShortDeck,
        &state.spot.challenge_id,
        "state-aware short-deck equity heuristic",
        vec![
            (PortfolioAction::ValueBet, value_bet),
            (PortfolioAction::AnteSteal, ante_steal),
            (PortfolioAction::PotControl, pot_control),
        ],
    )
}

fn teen_patti(state: &PokerLikeChallenge) -> EngineAnswer {
    let see_cards = 0.9
        + if state.has_seen_cards { 0.20 } else { 1.10 }
        + f32::from(state.fold_equity) * 0.20
        + if state.to_call_bb > 0 { 0.10 } else { 0.0 };
    let value_bet = 0.9
        + f32::from(state.made_strength) * 0.45
        + if state.has_seen_cards { 0.40 } else { 0.0 }
        + if state.raise_available { 0.15 } else { 0.0 };
    let pot_control = 0.8
        + f32::from(state.effective_stack_bb.min(40)) / 80.0
        + if state.check_available { 0.30 } else { 0.0 }
        + if state.to_call_bb > 0 { 0.10 } else { 0.0 };

    engine_answer(
        ResearchGame::TeenPatti,
        &state.spot.challenge_id,
        "state-aware blind-seen heuristic",
        vec![
            (PortfolioAction::SeeCards, see_cards),
            (PortfolioAction::ValueBet, value_bet),
            (PortfolioAction::PotControl, pot_control),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engine::EngineTier;
    use crate::engines::poker_like::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{PokerLikeChallenge, PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn nlhe_six_max_uses_typed_overpair_state_to_value_bet() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::NlheSixMax)
                .unwrap_or_else(|| panic!("nlhe six max challenge missing")),
        );

        assert_eq!(answer.engine_tier, EngineTier::RuleAware);
        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::ValueBet)
        );
    }

    #[test]
    fn teen_patti_changes_when_cards_are_already_seen() {
        let blind = PortfolioChallenge::TeenPatti(PokerLikeChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::TeenPatti,
                "blind-check",
                "Blind teen patti spot",
            ),
            pot_bb: 10,
            effective_stack_bb: 20,
            made_strength: 2,
            draw_strength: 0,
            fold_equity: 2,
            to_call_bb: 1,
            active_players: 6,
            check_available: false,
            raise_available: true,
            in_position: false,
            icm_pressure: 0,
            has_seen_cards: false,
        });
        let seen = PortfolioChallenge::TeenPatti(PokerLikeChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::TeenPatti,
                "seen-check",
                "Seen teen patti spot",
            ),
            has_seen_cards: true,
            ..match blind.clone() {
                PortfolioChallenge::TeenPatti(state) => state,
                _ => unreachable!(),
            }
        });

        let blind_answer = answer(&blind);
        let seen_answer = answer(&seen);

        assert_eq!(
            recommended_action(&blind_answer.response),
            Some(PortfolioAction::SeeCards)
        );
        assert_eq!(
            recommended_action(&seen_answer.response),
            Some(PortfolioAction::ValueBet)
        );
    }

    #[test]
    fn short_deck_free_option_prefers_ante_steal() {
        let answer = answer(&PortfolioChallenge::ShortDeck(PokerLikeChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::ShortDeck,
                "free-option",
                "Short deck free-option steal spot",
            ),
            pot_bb: 8,
            effective_stack_bb: 20,
            made_strength: 1,
            draw_strength: 1,
            fold_equity: 4,
            to_call_bb: 0,
            active_players: 5,
            check_available: true,
            raise_available: true,
            in_position: true,
            icm_pressure: 0,
            has_seen_cards: true,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::AnteSteal)
        );
    }
}
