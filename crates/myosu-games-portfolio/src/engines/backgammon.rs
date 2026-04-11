use crate::engine::EngineAnswer;
use crate::engines::engine_answer;
use crate::game::ResearchGame;
use crate::protocol::PortfolioAction;
use crate::state::PortfolioChallenge;

pub(super) fn answer(challenge: &PortfolioChallenge) -> EngineAnswer {
    let state = match challenge {
        PortfolioChallenge::Backgammon(state) => state,
        _ => unreachable!("backgammon::answer only accepts backgammon challenges"),
    };

    let move_options = f32::from(state.move_options);
    let off_moves = f32::from(state.off_moves);
    let build_moves = f32::from(state.move_options.saturating_sub(state.off_moves));
    let blot_count = f32::from(state.blot_count);
    let home_board_points = f32::from(state.home_board_points);
    let prime_length = f32::from(state.prime_length);
    let bear_off = 0.8
        + f32::from(state.borne_off) * 0.08
        + if state.has_contact { 0.0 } else { 0.50 }
        + if state.bearoff_ready { 0.55 } else { 0.0 }
        + off_moves * 0.16
        + move_options * 0.02
        + home_board_points * 0.10
        + prime_length * 0.05
        - blot_count * if state.has_contact { 0.05 } else { 0.16 }
        - f32::from(state.bar_count) * 0.35
        - f32::from(state.anchors) * 0.12
        - if state.facing_double { 0.95 } else { 0.0 }
        + (state.race_lead_pips.max(0) as f32) / 40.0;
    let accept_double = -0.10
        + f32::from(state.cube_efficiency) * 0.25
        + if state.facing_double { 1.10 } else { -0.70 }
        + if state.cube_centered { 0.10 } else { -0.10 }
        + if state.cube_owned_by_actor {
            -0.30
        } else {
            0.0
        }
        + if state.has_contact { -0.15 } else { 0.25 }
        + if state.race_lead_pips > 0 {
            0.35
        } else {
            -0.15
        }
        + home_board_points * 0.14
        + prime_length * 0.10
        - blot_count * 0.20
        + off_moves * 0.12
        + move_options * 0.04
        + if state.facing_double && state.off_moves >= 4 {
            0.45
        } else {
            0.0
        }
        + if state.facing_double && state.home_board_points >= 2 {
            0.30
        } else {
            0.0
        }
        - if state.facing_double && state.off_moves == 0 && !state.bearoff_ready {
            0.45
        } else {
            0.0
        }
        - if state.facing_double
            && state.blot_count >= 3
            && state.home_board_points == 0
            && state.off_moves < 6
        {
            0.65
        } else {
            0.0
        }
        - if state.bar_count > 0 { 0.20 } else { 0.0 };
    let advance_piece = 0.8
        + f32::from(state.anchors) * 0.35
        + if state.has_contact { 0.55 } else { 0.0 }
        + f32::from(state.bar_count) * 0.40
        + home_board_points * 0.18
        + prime_length * 0.28
        - blot_count * 0.24
        + build_moves * 0.12
        + if state.has_contact && state.prime_length >= 3 {
            0.50
        } else {
            0.0
        }
        + if state.off_moves == 0 { 0.20 } else { 0.0 }
        + if state.cube_owned_by_actor { 0.10 } else { 0.0 }
        - if state.bearoff_ready { 0.25 } else { 0.0 };

    engine_answer(
        ResearchGame::Backgammon,
        &state.spot.challenge_id,
        "state-aware race-contact heuristic",
        vec![
            (PortfolioAction::BearOff, bear_off),
            (PortfolioAction::AcceptDouble, accept_double),
            (PortfolioAction::AdvancePiece, advance_piece),
        ],
    )
}

#[cfg(test)]
mod tests {
    use crate::engines::backgammon::answer;
    use crate::game::ResearchGame;
    use crate::protocol::{PortfolioAction, recommended_action};
    use crate::state::{BackgammonChallenge, PortfolioChallenge, PortfolioChallengeSpot};

    #[test]
    fn facing_double_race_can_shift_to_accepting() {
        let answer = answer(
            &PortfolioChallenge::bootstrap(ResearchGame::Backgammon)
                .unwrap_or_else(|| panic!("backgammon challenge missing")),
        );

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::AcceptDouble)
        );
    }

    #[test]
    fn contact_state_prefers_advancing() {
        let answer = answer(&PortfolioChallenge::Backgammon(BackgammonChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Backgammon,
                "contact",
                "Contact backgammon spot",
            ),
            race_lead_pips: -6,
            borne_off: 4,
            anchors: 2,
            cube_efficiency: 0,
            has_contact: true,
            bar_count: 1,
            bearoff_ready: false,
            cube_centered: true,
            cube_owned_by_actor: false,
            facing_double: false,
            move_options: 4,
            off_moves: 0,
            blot_count: 1,
            home_board_points: 1,
            prime_length: 2,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::AdvancePiece)
        );
    }

    #[test]
    fn immediate_bearoff_options_support_taking_double() {
        let live_take = answer(&PortfolioChallenge::Backgammon(BackgammonChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Backgammon,
                "live-take",
                "Take point with strong immediate bearoff continuation",
            ),
            race_lead_pips: 10,
            borne_off: 11,
            anchors: 0,
            cube_efficiency: 2,
            has_contact: false,
            bar_count: 0,
            bearoff_ready: true,
            cube_centered: true,
            cube_owned_by_actor: false,
            facing_double: true,
            move_options: 8,
            off_moves: 8,
            blot_count: 0,
            home_board_points: 3,
            prime_length: 2,
        }));
        let thin_take = answer(&PortfolioChallenge::Backgammon(BackgammonChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Backgammon,
                "thin-take",
                "Take point with weak continuation and no immediate off moves",
            ),
            race_lead_pips: -2,
            borne_off: 9,
            anchors: 2,
            cube_efficiency: 0,
            has_contact: false,
            bar_count: 0,
            bearoff_ready: false,
            cube_centered: true,
            cube_owned_by_actor: false,
            facing_double: true,
            move_options: 1,
            off_moves: 0,
            blot_count: 3,
            home_board_points: 0,
            prime_length: 0,
        }));

        assert_eq!(
            recommended_action(&live_take.response),
            Some(PortfolioAction::AcceptDouble)
        );
        assert_ne!(
            recommended_action(&thin_take.response),
            Some(PortfolioAction::AcceptDouble)
        );
    }

    #[test]
    fn centered_race_without_offer_still_prefers_bearing_off() {
        let answer = answer(&PortfolioChallenge::Backgammon(BackgammonChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Backgammon,
                "centered-race",
                "Centered-cube race with no immediate take decision",
            ),
            race_lead_pips: 18,
            borne_off: 8,
            anchors: 0,
            cube_efficiency: 2,
            has_contact: false,
            bar_count: 0,
            bearoff_ready: true,
            cube_centered: true,
            cube_owned_by_actor: false,
            facing_double: false,
            move_options: 8,
            off_moves: 6,
            blot_count: 0,
            home_board_points: 2,
            prime_length: 2,
        }));

        assert_eq!(
            recommended_action(&answer.response),
            Some(PortfolioAction::BearOff)
        );
    }

    #[test]
    fn blotty_take_can_flip_away_from_accepting() {
        let secure_take = answer(&PortfolioChallenge::Backgammon(BackgammonChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Backgammon,
                "secure-take",
                "Strong race with real home-board coverage",
            ),
            race_lead_pips: 8,
            borne_off: 10,
            anchors: 0,
            cube_efficiency: 2,
            has_contact: false,
            bar_count: 0,
            bearoff_ready: true,
            cube_centered: true,
            cube_owned_by_actor: false,
            facing_double: true,
            move_options: 6,
            off_moves: 4,
            blot_count: 0,
            home_board_points: 3,
            prime_length: 2,
        }));
        let blotty_take = answer(&PortfolioChallenge::Backgammon(BackgammonChallenge {
            spot: PortfolioChallengeSpot::scenario(
                ResearchGame::Backgammon,
                "blotty-take",
                "Similar race with exposed blots and no home-board structure",
            ),
            race_lead_pips: 8,
            borne_off: 10,
            anchors: 0,
            cube_efficiency: 2,
            has_contact: false,
            bar_count: 0,
            bearoff_ready: true,
            cube_centered: true,
            cube_owned_by_actor: false,
            facing_double: true,
            move_options: 6,
            off_moves: 4,
            blot_count: 4,
            home_board_points: 0,
            prime_length: 0,
        }));

        assert_eq!(
            recommended_action(&secure_take.response),
            Some(PortfolioAction::AcceptDouble)
        );
        assert_ne!(
            recommended_action(&blotty_take.response),
            Some(PortfolioAction::AcceptDouble)
        );
    }
}
