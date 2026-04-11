use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cards::{Card, Rank, Suit};
use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::eval::meld::is_simple_meld;
use crate::game::ResearchGame;

const GIN_DISCARD_PREFIX: &str = "gin-rummy.discard.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct GinRummyPublicState {
    hand: Vec<Card>,
    discard_top: Card,
    stock_commitment: String,
    deadwood_count: u8,
    drew_from_discard: Option<Card>,
    turn_step: GinTurnStep,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GinRummyFeatureView {
    pub deadwood: u8,
    pub meld_count: u8,
    pub live_draws: u8,
    pub knock_available: bool,
    pub gin_available: bool,
    pub discard_options: u8,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum GinTurnStep {
    Draw,
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GinAction {
    DrawStock,
    DrawDiscard,
    Discard(Card),
    Knock,
    Gin,
}

pub fn gin_rummy_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    let public = GinRummyPublicState {
        hand: vec![
            Card::new(Rank::Seven, Suit::Hearts),
            Card::new(Rank::Seven, Suit::Clubs),
            Card::new(Rank::Eight, Suit::Clubs),
            Card::new(Rank::Nine, Suit::Clubs),
        ],
        discard_top: Card::new(Rank::Seven, Suit::Hearts),
        stock_commitment: "gin.stock.bootstrap-v1".to_string(),
        deadwood_count: 8,
        drew_from_discard: Some(Card::new(Rank::Seven, Suit::Hearts)),
        turn_step: GinTurnStep::Discard,
    };

    state_from_public(public, Some(0))
}

pub fn apply_gin_rummy_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: GinRummyPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    let action = parse_gin_action(action_id)?;
    if matches!(action, GinAction::Discard(card) if before_public.drew_from_discard == Some(card)) {
        return Err(illegal_gin_action(
            action_id,
            "cannot discard the card drawn from the discard pile in the same turn",
        ));
    }
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(illegal_gin_action(
            action_id,
            "action is not legal in this turn step",
        ));
    }

    let mut after_public = before_public.clone();
    match action {
        GinAction::DrawStock => {
            after_public.turn_step = GinTurnStep::Discard;
            after_public.drew_from_discard = None;
        }
        GinAction::DrawDiscard => {
            after_public.hand.push(after_public.discard_top);
            after_public.turn_step = GinTurnStep::Discard;
            after_public.drew_from_discard = Some(after_public.discard_top);
        }
        GinAction::Discard(card) => {
            if !after_public.hand.contains(&card) {
                return Err(illegal_gin_action(action_id, "card is not in hand"));
            }
            after_public.hand.retain(|candidate| *candidate != card);
            after_public.discard_top = card;
            after_public.drew_from_discard = None;
            after_public.deadwood_count = deadwood_count(&after_public.hand);
            after_public.turn_step = GinTurnStep::Draw;
        }
        GinAction::Knock => {
            if before_public.deadwood_count > 10 {
                return Err(illegal_gin_action(
                    action_id,
                    "knock requires deadwood <= 10",
                ));
            }
        }
        GinAction::Gin => {
            if before_public.deadwood_count != 0 {
                return Err(illegal_gin_action(action_id, "gin requires zero deadwood"));
            }
        }
    }

    let after = state_from_public(after_public, state.actor)?;
    let action = core_action_for_gin_action(action);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: GinRummyPublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    let legal_actions = legal_gin_actions(&public)
        .into_iter()
        .map(core_action_for_gin_action)
        .collect();
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "gin-rummy.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::GinRummy,
        phase: "discard".to_string(),
        actor,
        public_state,
        private_state_commitments: vec!["gin.opponent-hand.bootstrap-v1".to_string()],
        legal_actions,
        terminal: false,
        payoff: None,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<GinRummyFeatureView, CoreGameError> {
    let public: GinRummyPublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;
    let legal_actions = legal_gin_actions(&public);

    Ok(GinRummyFeatureView {
        deadwood: public.deadwood_count,
        meld_count: estimate_meld_count(&public.hand),
        live_draws: live_draw_score(&public.hand, public.discard_top),
        knock_available: legal_actions
            .iter()
            .any(|action| matches!(action, GinAction::Knock | GinAction::Gin)),
        gin_available: legal_actions
            .iter()
            .any(|action| matches!(action, GinAction::Gin)),
        discard_options: usize_to_u8(
            legal_actions
                .iter()
                .filter(|action| matches!(action, GinAction::Discard(_)))
                .count(),
        ),
    })
}

fn legal_gin_actions(public: &GinRummyPublicState) -> Vec<GinAction> {
    match public.turn_step {
        GinTurnStep::Draw => vec![GinAction::DrawStock, GinAction::DrawDiscard],
        GinTurnStep::Discard => {
            let mut actions = Vec::new();
            for card in &public.hand {
                if public.drew_from_discard != Some(*card) {
                    actions.push(GinAction::Discard(*card));
                }
            }
            if public.deadwood_count <= 10 {
                actions.push(GinAction::Knock);
            }
            if public.deadwood_count == 0 {
                actions.push(GinAction::Gin);
            }

            actions
        }
    }
}

fn core_action_for_gin_action(action: GinAction) -> CoreAction {
    match action {
        GinAction::DrawStock => CoreAction {
            action_id: "gin-rummy.draw.stock".to_string(),
            display_label: "draw-stock".to_string(),
            params: json!({}),
        },
        GinAction::DrawDiscard => CoreAction {
            action_id: "gin-rummy.draw.discard".to_string(),
            display_label: "draw-discard".to_string(),
            params: json!({}),
        },
        GinAction::Discard(card) => CoreAction {
            action_id: format!(
                "{GIN_DISCARD_PREFIX}{}-{}",
                crate::core::trick_taking::rank_token(card.rank),
                crate::core::trick_taking::suit_token(card.suit)
            ),
            display_label: format!(
                "discard-{}-{}",
                crate::core::trick_taking::rank_token(card.rank),
                crate::core::trick_taking::suit_token(card.suit)
            ),
            params: json!({"card": card}),
        },
        GinAction::Knock => CoreAction {
            action_id: "gin-rummy.knock".to_string(),
            display_label: "knock".to_string(),
            params: json!({}),
        },
        GinAction::Gin => CoreAction {
            action_id: "gin-rummy.gin".to_string(),
            display_label: "gin".to_string(),
            params: json!({}),
        },
    }
}

fn parse_gin_action(action_id: &str) -> Result<GinAction, CoreGameError> {
    match action_id {
        "gin-rummy.draw.stock" => return Ok(GinAction::DrawStock),
        "gin-rummy.draw.discard" => return Ok(GinAction::DrawDiscard),
        "gin-rummy.knock" => return Ok(GinAction::Knock),
        "gin-rummy.gin" => return Ok(GinAction::Gin),
        _ => {}
    }

    let Some(card_token) = action_id.strip_prefix(GIN_DISCARD_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::GinRummy,
            action_id: action_id.to_string(),
        });
    };
    let Some((rank, suit)) = card_token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::GinRummy,
            action_id: action_id.to_string(),
        });
    };
    let rank = crate::core::trick_taking::parse_rank(rank).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::GinRummy,
            action_id: action_id.to_string(),
        }
    })?;
    let suit = crate::core::trick_taking::parse_suit(suit).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::GinRummy,
            action_id: action_id.to_string(),
        }
    })?;

    Ok(GinAction::Discard(Card::new(rank, suit)))
}

fn deadwood_count(hand: &[Card]) -> u8 {
    let total = hand.iter().fold(0u8, |sum, card| {
        sum.saturating_add(gin_card_value(card.rank))
    });
    let meld_value = hand
        .windows(3)
        .find(|cards| is_simple_meld(cards))
        .map(|cards| {
            cards.iter().fold(0u8, |sum, card| {
                sum.saturating_add(gin_card_value(card.rank))
            })
        })
        .unwrap_or(0);

    total.saturating_sub(meld_value)
}

fn gin_card_value(rank: Rank) -> u8 {
    match rank {
        Rank::Ace => 1,
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => 10,
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

fn estimate_meld_count(hand: &[Card]) -> u8 {
    let rank_pairs = hand
        .iter()
        .map(|card| card.rank)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter(|rank| hand.iter().filter(|card| card.rank == *rank).count() >= 2)
        .count();
    let run_count = hand
        .windows(3)
        .filter(|window| {
            let Some(first) = window.first() else {
                return false;
            };
            let Some(second) = window.get(1) else {
                return false;
            };
            let Some(third) = window.get(2) else {
                return false;
            };

            first.suit == second.suit
                && second.suit == third.suit
                && rank_score(first.rank).saturating_add(1) == rank_score(second.rank)
                && rank_score(second.rank).saturating_add(1) == rank_score(third.rank)
        })
        .count();

    usize_to_u8(rank_pairs.saturating_add(run_count).min(5))
}

fn live_draw_score(hand: &[Card], discard_top: Card) -> u8 {
    let same_rank = hand.iter().any(|card| card.rank == discard_top.rank);
    let same_suit_neighbor = hand.iter().any(|card| {
        card.suit == discard_top.suit
            && rank_score(card.rank).abs_diff(rank_score(discard_top.rank)) == 1
    });
    match (same_rank, same_suit_neighbor) {
        (true, true) => 3,
        (true, false) | (false, true) => 2,
        (false, false) => 1,
    }
}

fn illegal_gin_action(action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: ResearchGame::GinRummy,
        action_id: action_id.to_string(),
        reason: reason.to_string(),
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn gin_rummy_bootstrap_state_has_legal_actions() {
        let state = gin_state();

        assert_eq!(state.game, ResearchGame::GinRummy);
        assert!(!state.legal_actions.is_empty());
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "gin-rummy.knock")
        );
    }

    #[test]
    fn gin_rummy_rejects_draw_discard_same_card() {
        let state = gin_state();

        assert!(matches!(
            apply_action(&state, "gin-rummy.discard.seven-hearts", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("discard pile")
        ));
    }

    #[test]
    fn gin_rummy_knock_eligibility() {
        let state = gin_state();

        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "gin-rummy.knock")
        );
        assert!(
            !state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "gin-rummy.gin")
        );
        assert!(
            gin_state_from_deadwood(0)
                .legal_actions
                .iter()
                .any(|action| action.action_id == "gin-rummy.gin")
        );
    }

    #[test]
    fn gin_rummy_transition_is_deterministic() {
        let state = gin_state();
        let first = apply_action(&state, "gin-rummy.discard.seven-clubs", json!({}));
        let second = apply_action(&state, "gin-rummy.discard.seven-clubs", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn gin_feature_view_tracks_deadwood_and_melds() {
        let state = gin_state();
        let view =
            feature_view(&state).unwrap_or_else(|error| panic!("gin view should decode: {error}"));

        assert_eq!(view.deadwood, 8);
        assert_eq!(view.meld_count, 2);
        assert_eq!(view.live_draws, 2);
        assert!(view.knock_available);
        assert!(!view.gin_available);
        assert_eq!(view.discard_options, 3);
    }

    fn gin_state() -> CoreGameState {
        match bootstrap_state(ResearchGame::GinRummy) {
            Ok(state) => state,
            Err(error) => panic!("gin bootstrap should succeed: {error}"),
        }
    }

    fn gin_state_from_deadwood(deadwood_count: u8) -> CoreGameState {
        let mut public: GinRummyPublicState = match serde_json::from_value(gin_state().public_state)
        {
            Ok(public) => public,
            Err(error) => panic!("gin public state should decode: {error}"),
        };
        public.deadwood_count = deadwood_count;

        match state_from_public(public, Some(0)) {
            Ok(state) => state,
            Err(error) => panic!("gin state should rebuild: {error}"),
        }
    }
}
