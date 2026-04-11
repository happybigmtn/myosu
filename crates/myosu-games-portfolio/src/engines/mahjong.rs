use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::PortfolioChallenge;

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    let state = match challenge {
        PortfolioChallenge::RiichiMahjong(state) => state,
        _ => unreachable!("mahjong::answer only accepts riichi-mahjong challenges"),
    };
    let discard_options = state.discard_options.max(1);
    let safety_ratio = f32::from(state.safe_discards) / f32::from(discard_options);

    let fold_danger = 0.9
        + f32::from(state.riichi_threats) * 0.72
        + (1.0 - safety_ratio) * 0.90
        + if state.shanten > 0 { 0.40 } else { 0.0 }
        + if !state.riichi_available { 0.25 } else { 0.0 }
        + if state.push_pressure <= 1 { 0.40 } else { 0.0 }
        - f32::from(state.dora_count) * 0.08
        - f32::from(state.yakuhai_pairs) * 0.18
        - f32::from(state.push_pressure) * 0.16;
    let declare_riichi = 0.2
        + if state.riichi_available { 1.00 } else { 0.0 }
        + f32::from(state.ukeire) * 0.07
        + f32::from(state.run_bases) * 0.10
        + f32::from(state.dora_count) * 0.18
        + f32::from(state.yakuhai_pairs) * 0.30
        + f32::from(state.push_pressure) * 0.24
        - if state.push_pressure <= 1 { 0.55 } else { 0.0 }
        - f32::from(state.pair_count) * 0.12
        - f32::from(state.riichi_threats) * 0.24
        - safety_ratio * 0.08;
    let pot_control = 0.8
        + safety_ratio * 0.55
        + f32::from(discard_options) * 0.02
        + f32::from(state.pair_count) * 0.18
        + if state.run_bases <= 1 { 0.30 } else { 0.0 }
        + if state.shanten == 0 && !state.riichi_available {
            0.35
        } else {
            0.0
        }
        + if state.push_pressure <= 2 { 0.28 } else { 0.0 };

    engine_answer(
        ResearchGame::RiichiMahjong,
        &state.spot.challenge_id,
        "state-aware shanten safety heuristic",
        vec![
            (PortfolioAction::FoldDanger, fold_danger),
            (PortfolioAction::DeclareRiichi, declare_riichi),
            (PortfolioAction::PotControl, pot_control),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::mahjong::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{MahjongChallenge, PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn threat_board_prefers_folding() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::RiichiMahjong)
                .unwrap_or_else(|| panic!("mahjong challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::FoldDanger)
        );
    }

    #[test]
    fn tenpai_board_prefers_riichi() {
        let challenge = PortfolioChallenge::RiichiMahjong(MahjongChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::RiichiMahjong,
                "tenpai-race",
                "Tenpai hand with live riichi window",
            ),
            shanten: 0,
            ukeire: 10,
            safe_discards: 1,
            discard_options: 4,
            pair_count: 1,
            run_bases: 5,
            dora_count: 2,
            yakuhai_pairs: 1,
            push_pressure: 5,
            riichi_threats: 0,
            riichi_available: true,
        });

        let answer = answer(&challenge);

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::DeclareRiichi)
        );
    }

    #[test]
    fn tenpai_without_riichi_window_prefers_control() {
        let challenge = PortfolioChallenge::RiichiMahjong(MahjongChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::RiichiMahjong,
                "tenpai-no-riichi",
                "Tenpai hand without a riichi declaration window",
            ),
            shanten: 0,
            ukeire: 7,
            safe_discards: 3,
            discard_options: 4,
            pair_count: 2,
            run_bases: 3,
            dora_count: 1,
            yakuhai_pairs: 0,
            push_pressure: 2,
            riichi_threats: 0,
            riichi_available: false,
        });

        let answer = answer(&challenge);

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::PotControl)
        );
    }

    #[test]
    fn rigid_tenpai_can_prefer_control_over_riichi() {
        let challenge = PortfolioChallenge::RiichiMahjong(MahjongChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::RiichiMahjong,
                "rigid-tenpai",
                "Rigid pair-heavy tenpai with a safe control line",
            ),
            shanten: 0,
            ukeire: 8,
            safe_discards: 4,
            discard_options: 4,
            pair_count: 4,
            run_bases: 1,
            dora_count: 1,
            yakuhai_pairs: 0,
            push_pressure: 2,
            riichi_threats: 0,
            riichi_available: true,
        });

        let answer = answer(&challenge);

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::PotControl)
        );
    }

    #[test]
    fn yakuhai_and_push_pressure_can_flip_same_tenpai_spot() {
        let push = PortfolioChallenge::RiichiMahjong(MahjongChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::RiichiMahjong,
                "push-tenpai",
                "Valuable live tenpai worth pushing through light pressure",
            ),
            shanten: 0,
            ukeire: 8,
            safe_discards: 2,
            discard_options: 4,
            pair_count: 2,
            run_bases: 3,
            dora_count: 1,
            yakuhai_pairs: 1,
            push_pressure: 4,
            riichi_threats: 1,
            riichi_available: true,
        });
        let pull = PortfolioChallenge::RiichiMahjong(MahjongChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::RiichiMahjong,
                "pull-tenpai",
                "Thin low-value tenpai that should not force a push",
            ),
            shanten: 0,
            ukeire: 8,
            safe_discards: 2,
            discard_options: 4,
            pair_count: 2,
            run_bases: 3,
            dora_count: 1,
            yakuhai_pairs: 0,
            push_pressure: 1,
            riichi_threats: 1,
            riichi_available: true,
        });

        let push_answer = answer(&push);
        let pull_answer = answer(&pull);

        assert_eq!(
            recommended_action(&push_answer.response),
            Some(PortfolioAction::DeclareRiichi)
        );
        assert_eq!(
            recommended_action(&pull_answer.response),
            Some(PortfolioAction::FoldDanger)
        );
    }
}
