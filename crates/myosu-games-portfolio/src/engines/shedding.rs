use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::{PortfolioChallenge, SheddingChallenge};

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    match challenge {
        PortfolioChallenge::DouDiZhu(state) => dou_di_zhu(state),
        PortfolioChallenge::PusoyDos(state) => pusoy_dos(state),
        PortfolioChallenge::TienLen(state) => tien_len(state),
        _ => unreachable!("shedding::answer only accepts shedding challenges"),
    }
}

fn dou_di_zhu(state: &SheddingChallenge) -> EngineAnswer {
    let preserve_bomb = 0.9
        + f32::from(state.bomb_count) * 0.90
        + if state.danger_opponents > 0 {
            -0.10
        } else {
            0.35
        }
        + if state.lead_rank_pressure >= 10 {
            0.15
        } else {
            0.0
        }
        + if state.play_options <= 1 { 0.25 } else { 0.0 }
        - f32::from(state.finishing_plays) * 0.20
        - if state.bomb_only_escape { 0.60 } else { 0.0 }
        - if state.next_actor_cards <= 2 {
            0.15
        } else {
            0.0
        };
    let landlord_bid = 0.8
        + f32::from(state.control_combos) * 0.45
        + if state.on_lead { 0.30 } else { 0.0 }
        + f32::from(state.play_options) * 0.10
        - if state.forced_pass { 0.40 } else { 0.0 }
        - f32::from(state.danger_opponents) * 0.10
        - if state.next_actor_cards <= 2 {
            0.20
        } else {
            0.0
        };
    let shed_lowest = 0.8
        + f32::from(state.low_singles) * 0.30
        + if state.opponents_min_cards <= 2 {
            0.25
        } else {
            0.0
        }
        + f32::from(state.danger_opponents) * 0.18
        + if state.lead_rank_pressure <= 8 {
            0.10
        } else {
            0.0
        }
        + f32::from(state.finishing_plays) * 0.70
        + if state.next_actor_cards <= 2 {
            0.25
        } else {
            0.0
        }
        - if state.bomb_only_escape { 0.20 } else { 0.0 }
        - if state.forced_pass { 0.50 } else { 0.0 };

    engine_answer(
        ResearchGame::DouDiZhu,
        &state.spot.challenge_id,
        "state-aware bomb-preservation heuristic",
        vec![
            (PortfolioAction::PreserveBomb, preserve_bomb),
            (PortfolioAction::LandlordBid, landlord_bid),
            (PortfolioAction::ShedLowest, shed_lowest),
        ],
    )
}

fn pusoy_dos(state: &SheddingChallenge) -> EngineAnswer {
    let lead_control = 0.9
        + f32::from(state.control_combos) * 0.55
        + if state.on_lead { 0.50 } else { 0.0 }
        + f32::from(state.play_options) * 0.12
        - f32::from(state.finishing_plays) * 0.55
        - if state.next_actor_cards <= 2 {
            0.20
        } else {
            0.0
        };
    let pass_control = 0.8
        + if state.on_lead { 0.0 } else { 0.65 }
        + if state.forced_pass { 0.65 } else { 0.0 }
        + f32::from(state.danger_opponents) * 0.22
        + if state.lead_rank_pressure >= 10 {
            0.20
        } else {
            0.0
        }
        + if state.bomb_only_escape && !state.on_lead {
            0.45
        } else {
            0.0
        };
    let shed_lowest = 0.8
        + f32::from(state.low_singles) * 0.30
        + if state.play_options >= 3 { 0.10 } else { 0.0 }
        + f32::from(state.finishing_plays) * 0.70
        + if state.next_actor_cards <= 2 {
            0.20
        } else {
            0.0
        }
        - if state.bomb_only_escape { 0.10 } else { 0.0 }
        - if state.forced_pass { 0.35 } else { 0.0 };

    engine_answer(
        ResearchGame::PusoyDos,
        &state.spot.challenge_id,
        "state-aware climbing-card control heuristic",
        vec![
            (PortfolioAction::LeadControl, lead_control),
            (PortfolioAction::PassControl, pass_control),
            (PortfolioAction::ShedLowest, shed_lowest),
        ],
    )
}

fn tien_len(state: &SheddingChallenge) -> EngineAnswer {
    let preserve_bomb = 0.9
        + f32::from(state.bomb_count) * 0.75
        + if state.lead_rank_pressure >= 11 {
            0.35
        } else if state.danger_opponents > 0 {
            0.10
        } else {
            0.30
        }
        - f32::from(state.finishing_plays) * 0.45
        - if state.bomb_only_escape { 0.25 } else { 0.0 };
    let shed_lowest = 0.8
        + f32::from(state.low_singles) * 0.35
        + if state.on_lead { 0.15 } else { 0.0 }
        + if state.lead_rank_pressure <= 8 {
            0.10
        } else {
            0.0
        }
        + f32::from(state.finishing_plays) * 0.75
        + if state.next_actor_cards <= 2 {
            0.20
        } else {
            0.0
        }
        - if state.forced_pass { 0.50 } else { 0.0 };
    let lead_control = 0.8
        + f32::from(state.control_combos) * 0.40
        + if state.on_lead { 0.35 } else { 0.0 }
        + f32::from(state.play_options) * 0.10
        + if state.bomb_only_escape { 0.30 } else { 0.0 }
        - if state.forced_pass { 0.25 } else { 0.0 };

    engine_answer(
        ResearchGame::TienLen,
        &state.spot.challenge_id,
        "state-aware tien-len combination heuristic",
        vec![
            (PortfolioAction::PreserveBomb, preserve_bomb),
            (PortfolioAction::ShedLowest, shed_lowest),
            (PortfolioAction::LeadControl, lead_control),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::shedding::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{PortfolioChallenge, PortfolioChallengeSpot, SheddingChallenge};

    #[test]
    fn dou_di_zhu_preserves_bombs_in_reference_state() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::DouDiZhu)
                .unwrap_or_else(|| panic!("dou di zhu challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::PreserveBomb)
        );
    }

    #[test]
    fn pusoy_on_lead_prefers_control_but_off_lead_can_pass() {
        let on_lead = PortfolioChallenge::PusoyDos(SheddingChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::PusoyDos,
                "on-lead",
                "On-lead climbing spot",
            ),
            bomb_count: 0,
            control_combos: 3,
            low_singles: 2,
            opponents_min_cards: 4,
            danger_opponents: 0,
            next_actor_cards: 4,
            on_lead: true,
            play_options: 3,
            finishing_plays: 0,
            bomb_only_escape: false,
            forced_pass: false,
            lead_rank_pressure: 0,
        });
        let off_lead = PortfolioChallenge::PusoyDos(SheddingChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::PusoyDos,
                "off-lead",
                "Off-lead response spot",
            ),
            control_combos: 1,
            low_singles: 1,
            opponents_min_cards: 2,
            danger_opponents: 1,
            next_actor_cards: 2,
            on_lead: false,
            play_options: 0,
            finishing_plays: 0,
            bomb_only_escape: false,
            forced_pass: true,
            lead_rank_pressure: 11,
            ..match on_lead.clone() {
                PortfolioChallenge::PusoyDos(state) => state,
                _ => unreachable!(),
            }
        });

        assert_eq!(
            recommended_action(&answer(&on_lead).response),
            Some(PortfolioAction::LeadControl)
        );
        assert_eq!(
            recommended_action(&answer(&off_lead).response),
            Some(PortfolioAction::PassControl)
        );
    }

    #[test]
    fn tien_len_high_pair_pressure_keeps_bomb_preservation_on_top() {
        let answer = answer(&PortfolioChallenge::TienLen(SheddingChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::TienLen,
                "high-pair-pressure",
                "High pair pressure with a bomb available",
            ),
            bomb_count: 1,
            control_combos: 1,
            low_singles: 1,
            opponents_min_cards: 3,
            danger_opponents: 1,
            next_actor_cards: 2,
            on_lead: false,
            play_options: 1,
            finishing_plays: 0,
            bomb_only_escape: false,
            forced_pass: false,
            lead_rank_pressure: 12,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::PreserveBomb)
        );
    }

    #[test]
    fn pusoy_finishing_window_prefers_shedding_over_control() {
        let answer = answer(&PortfolioChallenge::PusoyDos(SheddingChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::PusoyDos,
                "finishing-window",
                "Lead spot with a clean finishing line before the next actor gets short",
            ),
            bomb_count: 0,
            control_combos: 1,
            low_singles: 2,
            opponents_min_cards: 3,
            danger_opponents: 0,
            next_actor_cards: 2,
            on_lead: true,
            play_options: 1,
            finishing_plays: 2,
            bomb_only_escape: false,
            forced_pass: false,
            lead_rank_pressure: 0,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::ShedLowest)
        );
    }
}
