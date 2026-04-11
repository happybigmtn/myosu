use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cards::{Card, Rank, Suit};
use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::eval::poker::simple_strength;
use crate::game::ResearchGame;

const OFC_PLACE_PREFIX: &str = "ofc-chinese-poker.place.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct OfcPublicState {
    front: Vec<Card>,
    middle: Vec<Card>,
    back: Vec<Card>,
    dealt: Vec<Card>,
    remaining_draw_commitment: String,
    foul: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OfcFeatureView {
    pub front_strength: u8,
    pub middle_strength: u8,
    pub back_strength: u8,
    pub free_slots: u8,
    pub fantasyland_outs: u8,
    pub foul_pressure: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OfcRow {
    Front,
    Middle,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OfcPlacement {
    row: OfcRow,
    card: Card,
}

pub fn ofc_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    let public = OfcPublicState {
        front: vec![
            Card::new(Rank::Ace, Suit::Clubs),
            Card::new(Rank::Ace, Suit::Spades),
            Card::new(Rank::Queen, Suit::Hearts),
        ],
        middle: vec![
            Card::new(Rank::King, Suit::Clubs),
            Card::new(Rank::Ten, Suit::Diamonds),
            Card::new(Rank::Eight, Suit::Hearts),
            Card::new(Rank::Four, Suit::Spades),
        ],
        back: vec![
            Card::new(Rank::Two, Suit::Clubs),
            Card::new(Rank::Two, Suit::Diamonds),
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::Seven, Suit::Diamonds),
            Card::new(Rank::Nine, Suit::Spades),
        ],
        dealt: vec![Card::new(Rank::Three, Suit::Clubs)],
        remaining_draw_commitment: "ofc.draw.bootstrap-v1".to_string(),
        foul: None,
    };

    state_from_public(public, Some(0))
}

pub fn apply_ofc_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: OfcPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    let placement = parse_ofc_placement(action_id)?;
    if !before_public.dealt.contains(&placement.card) {
        return Err(illegal_ofc_action(action_id, "card is not in dealt cards"));
    }
    if row_cards(&before_public, placement.row).len() >= row_capacity(placement.row) {
        return Err(illegal_ofc_action(action_id, "row is already full"));
    }

    let mut after_public = before_public.clone();
    row_cards_mut(&mut after_public, placement.row).push(placement.card);
    after_public
        .dealt
        .retain(|candidate| *candidate != placement.card);
    if total_placed(&after_public) == 13 && after_public.dealt.is_empty() {
        after_public.foul = Some(detect_foul(&after_public)?);
    }
    let terminal = after_public.foul.is_some();
    let payoff = after_public
        .foul
        .map(|foul| if foul { vec![-6, 6] } else { vec![0, 0] });
    let after = state_from_public_with_terminal(after_public, state.actor, terminal, payoff)?;
    let action = core_action_for_placement(placement);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: OfcPublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    state_from_public_with_terminal(public, actor, false, None)
}

fn state_from_public_with_terminal(
    public: OfcPublicState,
    actor: Option<u8>,
    terminal: bool,
    payoff: Option<Vec<i64>>,
) -> Result<CoreGameState, CoreGameError> {
    let legal_actions = legal_ofc_placements(&public)
        .into_iter()
        .map(core_action_for_placement)
        .collect();
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "ofc.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::OfcChinesePoker,
        phase: "placement".to_string(),
        actor,
        public_state,
        private_state_commitments: vec!["ofc.remaining-draw.bootstrap-v1".to_string()],
        legal_actions,
        terminal,
        payoff,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<OfcFeatureView, CoreGameError> {
    let public: OfcPublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;

    let front_strength = row_strength(&public.front);
    let middle_strength = row_strength(&public.middle);
    let back_strength = row_strength(&public.back);
    let foul_pressure = front_strength
        .saturating_sub(middle_strength)
        .saturating_add(middle_strength.saturating_sub(back_strength))
        .min(15);

    Ok(OfcFeatureView {
        front_strength,
        middle_strength,
        back_strength,
        free_slots: usize_to_u8(
            3usize
                .saturating_sub(public.front.len())
                .saturating_add(5usize.saturating_sub(public.middle.len()))
                .saturating_add(5usize.saturating_sub(public.back.len())),
        ),
        fantasyland_outs: usize_to_u8(
            public
                .dealt
                .iter()
                .filter(|card| rank_score(card.rank) >= 12)
                .count(),
        ),
        foul_pressure,
    })
}

fn legal_ofc_placements(public: &OfcPublicState) -> Vec<OfcPlacement> {
    let mut placements = Vec::new();
    for card in &public.dealt {
        for row in [OfcRow::Front, OfcRow::Middle, OfcRow::Back] {
            if row_cards(public, row).len() < row_capacity(row) {
                placements.push(OfcPlacement { row, card: *card });
            }
        }
    }

    placements
}

fn core_action_for_placement(placement: OfcPlacement) -> CoreAction {
    CoreAction {
        action_id: format!(
            "{OFC_PLACE_PREFIX}{}.{}-{}",
            row_token(placement.row),
            crate::core::trick_taking::rank_token(placement.card.rank),
            crate::core::trick_taking::suit_token(placement.card.suit)
        ),
        display_label: format!(
            "place-{}-{}-{}",
            row_token(placement.row),
            crate::core::trick_taking::rank_token(placement.card.rank),
            crate::core::trick_taking::suit_token(placement.card.suit)
        ),
        params: json!({"row": row_token(placement.row), "card": placement.card}),
    }
}

fn parse_ofc_placement(action_id: &str) -> Result<OfcPlacement, CoreGameError> {
    let Some(place_token) = action_id.strip_prefix(OFC_PLACE_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::OfcChinesePoker,
            action_id: action_id.to_string(),
        });
    };
    let Some((row, card)) = place_token.split_once('.') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::OfcChinesePoker,
            action_id: action_id.to_string(),
        });
    };
    let row = parse_row(action_id, row)?;
    let Some((rank, suit)) = card.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::OfcChinesePoker,
            action_id: action_id.to_string(),
        });
    };
    let rank = crate::core::trick_taking::parse_rank(rank).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::OfcChinesePoker,
            action_id: action_id.to_string(),
        }
    })?;
    let suit = crate::core::trick_taking::parse_suit(suit).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::OfcChinesePoker,
            action_id: action_id.to_string(),
        }
    })?;

    Ok(OfcPlacement {
        row,
        card: Card::new(rank, suit),
    })
}

fn parse_row(action_id: &str, row: &str) -> Result<OfcRow, CoreGameError> {
    match row {
        "front" => Ok(OfcRow::Front),
        "middle" => Ok(OfcRow::Middle),
        "back" => Ok(OfcRow::Back),
        _ => Err(CoreGameError::UnknownAction {
            game: ResearchGame::OfcChinesePoker,
            action_id: action_id.to_string(),
        }),
    }
}

fn detect_foul(public: &OfcPublicState) -> Result<bool, CoreGameError> {
    let front = simple_strength(&public.front).map_err(|source| CoreGameError::InvalidParams {
        action_id: "ofc.detect-foul".to_string(),
        reason: source.to_string(),
    })?;
    let middle =
        simple_strength(&public.middle).map_err(|source| CoreGameError::InvalidParams {
            action_id: "ofc.detect-foul".to_string(),
            reason: source.to_string(),
        })?;
    let back = simple_strength(&public.back).map_err(|source| CoreGameError::InvalidParams {
        action_id: "ofc.detect-foul".to_string(),
        reason: source.to_string(),
    })?;

    Ok(strength_at_least(front, middle) || strength_at_least(middle, back))
}

fn strength_at_least(
    left: crate::eval::poker::SimplePokerStrength,
    right: crate::eval::poker::SimplePokerStrength,
) -> bool {
    (left.class, left.high_rank) >= (right.class, right.high_rank)
}

fn total_placed(public: &OfcPublicState) -> usize {
    public
        .front
        .len()
        .saturating_add(public.middle.len())
        .saturating_add(public.back.len())
}

fn row_cards(public: &OfcPublicState, row: OfcRow) -> &[Card] {
    match row {
        OfcRow::Front => &public.front,
        OfcRow::Middle => &public.middle,
        OfcRow::Back => &public.back,
    }
}

fn row_cards_mut(public: &mut OfcPublicState, row: OfcRow) -> &mut Vec<Card> {
    match row {
        OfcRow::Front => &mut public.front,
        OfcRow::Middle => &mut public.middle,
        OfcRow::Back => &mut public.back,
    }
}

fn row_capacity(row: OfcRow) -> usize {
    match row {
        OfcRow::Front => 3,
        OfcRow::Middle | OfcRow::Back => 5,
    }
}

fn row_token(row: OfcRow) -> &'static str {
    match row {
        OfcRow::Front => "front",
        OfcRow::Middle => "middle",
        OfcRow::Back => "back",
    }
}

fn row_strength(cards: &[Card]) -> u8 {
    match simple_strength(cards) {
        Ok(strength) => {
            let class_score = match strength.class {
                crate::eval::poker::SimplePokerClass::HighCard => 0_u8,
                crate::eval::poker::SimplePokerClass::Pair => 4_u8,
                crate::eval::poker::SimplePokerClass::Trips => 8_u8,
            };
            usize_to_u8(
                usize::from(class_score)
                    .saturating_add(usize::from(rank_score(strength.high_rank) / 4))
                    .saturating_add(cards.len())
                    .min(15),
            )
        }
        Err(_) => usize_to_u8(cards.len()),
    }
}

fn rank_score(rank: Rank) -> u8 {
    match rank {
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        Rank::Ace => 14,
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn illegal_ofc_action(action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: ResearchGame::OfcChinesePoker,
        action_id: action_id.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn ofc_bootstrap_state_has_legal_actions() {
        let state = ofc_state();

        assert_eq!(state.game, ResearchGame::OfcChinesePoker);
        assert_eq!(state.legal_actions.len(), 1);
        assert_eq!(
            state
                .legal_actions
                .first()
                .map(|action| action.action_id.as_str()),
            Some("ofc-chinese-poker.place.middle.three-clubs")
        );
    }

    #[test]
    fn ofc_rejects_full_row_placement() {
        let state = ofc_state();

        assert!(matches!(
            apply_action(&state, "ofc-chinese-poker.place.front.three-clubs", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("full")
        ));
    }

    #[test]
    fn ofc_transition_is_deterministic() {
        let state = ofc_state();
        let first = apply_action(
            &state,
            "ofc-chinese-poker.place.middle.three-clubs",
            json!({}),
        );
        let second = apply_action(
            &state,
            "ofc-chinese-poker.place.middle.three-clubs",
            json!({}),
        );

        assert_eq!(first, second);
    }

    #[test]
    fn ofc_detects_fouling() {
        let state = ofc_state();
        let transition = match apply_action(
            &state,
            "ofc-chinese-poker.place.middle.three-clubs",
            json!({}),
        ) {
            Ok(transition) => transition,
            Err(error) => panic!("OFC legal placement should apply: {error}"),
        };
        let public: OfcPublicState = match serde_json::from_value(transition.after.public_state) {
            Ok(public) => public,
            Err(error) => panic!("OFC public state should decode: {error}"),
        };

        assert!(transition.after.terminal);
        assert_eq!(public.foul, Some(true));
        assert_eq!(transition.after.payoff, Some(vec![-6, 6]));
    }

    #[test]
    fn ofc_feature_view_tracks_slot_pressure() {
        let state = ofc_state();
        let view =
            feature_view(&state).unwrap_or_else(|error| panic!("ofc view should decode: {error}"));

        assert_eq!(view.free_slots, 1);
        assert_eq!(view.fantasyland_outs, 0);
        assert!(view.back_strength > view.middle_strength);
        assert!(view.foul_pressure > 0);
    }

    fn ofc_state() -> CoreGameState {
        match bootstrap_state(ResearchGame::OfcChinesePoker) {
            Ok(state) => state,
            Err(error) => panic!("OFC bootstrap should succeed: {error}"),
        }
    }
}
