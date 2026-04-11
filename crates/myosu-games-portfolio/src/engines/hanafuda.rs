use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::{HanafudaChallenge, PortfolioChallenge};

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    match challenge {
        PortfolioChallenge::HanafudaKoiKoi(state) => koi_koi(state),
        PortfolioChallenge::HwatuGoStop(state) => hwatu_go_stop(state),
        _ => unreachable!("hanafuda::answer only accepts hanafuda challenges"),
    }
}

fn koi_koi(state: &HanafudaChallenge) -> EngineAnswer {
    let continuation_pressure = f32::from(state.continuation_calls);
    let window_gain = f32::from(state.points.saturating_sub(state.locked_points));
    let locked_points = f32::from(state.locked_points);
    let upside_gain = f32::from(state.max_upside_gain);
    let upside_options = f32::from(state.upside_capture_options);
    let stop_round = 0.30
        + if state.decision_window { 0.75 } else { 0.0 }
        + window_gain * 0.36
        + locked_points * 0.10
        + f32::from(state.points) * 0.08
        + f32::from(state.bright_count) * 0.35
        + f32::from(state.yaku_count) * 0.25
        + continuation_pressure * 0.22
        + f32::from(state.opponent_pressure) * 0.20
        - upside_gain * 0.24
        - upside_options * 0.14
        - f32::from(state.hand_count) * 0.05;
    let mixed_yaku = f32::from(state.ribbon_yaku) + f32::from(state.animal_yaku);
    let koi_koi = 0.35
        + if state.decision_window { 0.60 } else { 0.0 }
        + upside_gain * 0.40
        + upside_options * 0.24
        + f32::from(state.bright_capture_options) * 0.24
        + f32::from(state.bonus_cards) * 0.18
        + mixed_yaku * 0.08
        + f32::from(state.hand_count) * 0.12
        + if state.locked_points > 0 && state.points.saturating_sub(state.locked_points) <= 1 {
            0.35
        } else {
            0.0
        }
        - window_gain * 0.18
        - continuation_pressure * 0.18
        - f32::from(state.opponent_pressure) * 0.05;
    let call_go = 0.2
        + if state.decision_window { 0.10 } else { 0.0 }
        + upside_gain * 0.05
        + continuation_pressure * 0.08
        + f32::from(state.animal_yaku) * 0.10;

    engine_answer(
        ResearchGame::HanafudaKoiKoi,
        &state.spot.challenge_id,
        "state-aware yaku EV heuristic",
        vec![
            (PortfolioAction::StopRound, stop_round),
            (PortfolioAction::KoiKoi, koi_koi),
            (PortfolioAction::CallGo, call_go),
        ],
    )
}

fn hwatu_go_stop(state: &HanafudaChallenge) -> EngineAnswer {
    let continuation_pressure = f32::from(state.continuation_calls);
    let mixed_yaku = f32::from(state.ribbon_yaku) + f32::from(state.animal_yaku);
    let window_gain = f32::from(state.points.saturating_sub(state.locked_points));
    let locked_points = f32::from(state.locked_points);
    let upside_gain = f32::from(state.max_upside_gain);
    let upside_options = f32::from(state.upside_capture_options);
    let call_go = 0.35
        + if state.decision_window { 0.75 } else { 0.0 }
        + upside_gain * 0.38
        + upside_options * 0.22
        + f32::from(state.bonus_cards) * 0.30
        + f32::from(state.bright_capture_options) * 0.16
        + mixed_yaku * 0.12
        + f32::from(state.hand_count) * 0.08
        + if window_gain <= 2.0 { 0.25 } else { 0.0 }
        - window_gain * 0.14
        - continuation_pressure * 0.12
        - f32::from(state.opponent_pressure) * 0.04;
    let stop_round = 0.35
        + if state.decision_window { 0.75 } else { 0.0 }
        + window_gain * 0.34
        + locked_points * 0.18
        + f32::from(state.points) * 0.06
        + f32::from(state.yaku_count) * 0.22
        + continuation_pressure * 0.24
        + f32::from(state.opponent_pressure) * 0.20
        - upside_gain * 0.20
        - upside_options * 0.10
        - f32::from(state.hand_count) * 0.05;
    let koi_koi = 0.20
        + if state.decision_window { 0.30 } else { 0.0 }
        + upside_gain * 0.12
        + f32::from(state.bonus_cards) * 0.12
        + f32::from(state.hand_count) * 0.10
        - window_gain * 0.08
        - continuation_pressure * 0.10;

    engine_answer(
        ResearchGame::HwatuGoStop,
        &state.spot.challenge_id,
        "state-aware go-stop bonus heuristic",
        vec![
            (PortfolioAction::CallGo, call_go),
            (PortfolioAction::StopRound, stop_round),
            (PortfolioAction::KoiKoi, koi_koi),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::hanafuda::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{HanafudaChallenge, PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn koi_koi_reference_state_prefers_stopping() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::HanafudaKoiKoi)
                .unwrap_or_else(|| panic!("koi-koi challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::StopRound)
        );
    }

    #[test]
    fn hwatu_call_go_grows_with_bonus_pressure() {
        let aggressive = PortfolioChallenge::HwatuGoStop(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HwatuGoStop,
                "bonus-heavy",
                "Bonus-heavy go-stop spot",
            ),
            points: 4,
            bright_count: 1,
            ribbon_yaku: 1,
            animal_yaku: 2,
            bonus_cards: 4,
            yaku_count: 1,
            bright_capture_options: 2,
            opponent_pressure: 1,
            hand_count: 3,
            decision_window: true,
            locked_points: 0,
            continuation_calls: 0,
            upside_capture_options: 3,
            max_upside_gain: 5,
        });
        let cautious = PortfolioChallenge::HwatuGoStop(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HwatuGoStop,
                "cash-out",
                "Cash-out go-stop spot",
            ),
            points: 7,
            bright_count: 1,
            ribbon_yaku: 1,
            animal_yaku: 1,
            bonus_cards: 0,
            yaku_count: 2,
            bright_capture_options: 0,
            opponent_pressure: 3,
            hand_count: 1,
            decision_window: true,
            locked_points: 5,
            continuation_calls: 1,
            upside_capture_options: 0,
            max_upside_gain: 0,
        });

        let aggressive_answer = answer(&aggressive);
        let cautious_answer = answer(&cautious);

        assert_eq!(
            recommended_action(&aggressive_answer.response),
            Some(PortfolioAction::CallGo)
        );
        assert_eq!(
            recommended_action(&cautious_answer.response),
            Some(PortfolioAction::StopRound)
        );
    }

    #[test]
    fn koi_koi_window_can_prefer_continuation_when_live() {
        let answer = answer(&PortfolioChallenge::HanafudaKoiKoi(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HanafudaKoiKoi,
                "live-koi-koi",
                "Low-point koi-koi continuation window",
            ),
            points: 3,
            bright_count: 1,
            ribbon_yaku: 1,
            animal_yaku: 1,
            bonus_cards: 3,
            yaku_count: 1,
            bright_capture_options: 2,
            opponent_pressure: 0,
            hand_count: 3,
            decision_window: true,
            locked_points: 0,
            continuation_calls: 0,
            upside_capture_options: 2,
            max_upside_gain: 4,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::KoiKoi)
        );
    }

    #[test]
    fn bright_capture_window_changes_koi_koi_cashout_preference() {
        let live = PortfolioChallenge::HanafudaKoiKoi(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HanafudaKoiKoi,
                "live-bright-window",
                "Small cashout with bright continuation pressure",
            ),
            points: 4,
            bright_count: 1,
            ribbon_yaku: 1,
            animal_yaku: 0,
            bonus_cards: 1,
            yaku_count: 1,
            bright_capture_options: 2,
            opponent_pressure: 1,
            hand_count: 2,
            decision_window: true,
            locked_points: 0,
            continuation_calls: 0,
            upside_capture_options: 2,
            max_upside_gain: 4,
        });
        let cash_out = PortfolioChallenge::HanafudaKoiKoi(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HanafudaKoiKoi,
                "banked-yaku-window",
                "Cashout with multiple completed yaku and no bright upside",
            ),
            points: 6,
            bright_count: 2,
            ribbon_yaku: 1,
            animal_yaku: 1,
            bonus_cards: 0,
            yaku_count: 2,
            bright_capture_options: 0,
            opponent_pressure: 2,
            hand_count: 1,
            decision_window: true,
            locked_points: 4,
            continuation_calls: 1,
            upside_capture_options: 0,
            max_upside_gain: 0,
        });

        let live_answer = answer(&live);
        let cash_out_answer = answer(&cash_out);

        assert_eq!(
            recommended_action(&live_answer.response),
            Some(PortfolioAction::KoiKoi)
        );
        assert_eq!(
            recommended_action(&cash_out_answer.response),
            Some(PortfolioAction::StopRound)
        );
    }

    #[test]
    fn locked_margin_and_upside_can_flip_hwatu_decision() {
        let thin = PortfolioChallenge::HwatuGoStop(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HwatuGoStop,
                "thin-window",
                "Small banked increase with real continuation upside",
            ),
            points: 6,
            bright_count: 3,
            ribbon_yaku: 1,
            animal_yaku: 1,
            bonus_cards: 2,
            yaku_count: 1,
            bright_capture_options: 1,
            opponent_pressure: 1,
            hand_count: 2,
            decision_window: true,
            locked_points: 5,
            continuation_calls: 1,
            upside_capture_options: 2,
            max_upside_gain: 4,
        });
        let banked = PortfolioChallenge::HwatuGoStop(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HwatuGoStop,
                "banked-window",
                "Large banked score with no immediate continuation upside",
            ),
            points: 7,
            bright_count: 3,
            ribbon_yaku: 1,
            animal_yaku: 1,
            bonus_cards: 0,
            yaku_count: 1,
            bright_capture_options: 0,
            opponent_pressure: 2,
            hand_count: 1,
            decision_window: true,
            locked_points: 3,
            continuation_calls: 1,
            upside_capture_options: 0,
            max_upside_gain: 0,
        });

        let thin_answer = answer(&thin);
        let banked_answer = answer(&banked);

        assert_eq!(
            recommended_action(&thin_answer.response),
            Some(PortfolioAction::CallGo)
        );
        assert_eq!(
            recommended_action(&banked_answer.response),
            Some(PortfolioAction::StopRound)
        );
    }

    #[test]
    fn continuation_count_pushes_koi_koi_window_toward_cash_out() {
        let fresh = PortfolioChallenge::HanafudaKoiKoi(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HanafudaKoiKoi,
                "fresh-window",
                "Fresh scoring window with live continuation upside",
            ),
            points: 4,
            bright_count: 1,
            ribbon_yaku: 1,
            animal_yaku: 1,
            bonus_cards: 2,
            yaku_count: 1,
            bright_capture_options: 1,
            opponent_pressure: 1,
            hand_count: 2,
            decision_window: true,
            locked_points: 0,
            continuation_calls: 0,
            upside_capture_options: 2,
            max_upside_gain: 3,
        });
        let stretched = PortfolioChallenge::HanafudaKoiKoi(HanafudaChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::HanafudaKoiKoi,
                "stretched-window",
                "Later scoring window after multiple continuations",
            ),
            points: 4,
            bright_count: 1,
            ribbon_yaku: 1,
            animal_yaku: 1,
            bonus_cards: 2,
            yaku_count: 1,
            bright_capture_options: 1,
            opponent_pressure: 1,
            hand_count: 2,
            decision_window: true,
            locked_points: 3,
            continuation_calls: 3,
            upside_capture_options: 1,
            max_upside_gain: 1,
        });

        let fresh_answer = answer(&fresh);
        let stretched_answer = answer(&stretched);

        assert_eq!(
            recommended_action(&fresh_answer.response),
            Some(PortfolioAction::KoiKoi)
        );
        assert_eq!(
            recommended_action(&stretched_answer.response),
            Some(PortfolioAction::StopRound)
        );
    }
}
