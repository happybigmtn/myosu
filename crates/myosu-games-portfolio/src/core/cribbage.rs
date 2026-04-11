use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cards::{Card, Rank, Suit};
use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::eval::meld::is_three_card_run;
use crate::game::ResearchGame;

const CRIBBAGE_ACTION_PREFIX: &str = "cribbage.pegging.play-";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct CribbagePublicState {
    running_count: u8,
    pegging_points: u8,
    actor_hand: Vec<Card>,
    played_cards: Vec<Card>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CribbageFeatureView {
    pub pegging_count: u8,
    pub run_potential: u8,
    pub crib_edge: i8,
    pub pair_trap: bool,
    pub go_window: bool,
    pub fifteen_outs: u8,
    pub max_immediate_points: u8,
}

pub fn cribbage_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    let public = CribbagePublicState {
        running_count: 9,
        pegging_points: 0,
        actor_hand: vec![
            Card::new(Rank::Six, Suit::Clubs),
            Card::new(Rank::King, Suit::Hearts),
            Card::new(Rank::Two, Suit::Spades),
            Card::new(Rank::Ace, Suit::Diamonds),
        ],
        played_cards: vec![
            Card::new(Rank::Four, Suit::Clubs),
            Card::new(Rank::Five, Suit::Clubs),
        ],
    };

    state_from_public(public, Some(0))
}

pub fn apply_cribbage_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: CribbagePublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    let card = parse_cribbage_card_action(action_id)?;
    if !before_public.actor_hand.contains(&card) {
        return Err(CoreGameError::IllegalAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
            reason: "card is not in acting hand".to_string(),
        });
    }

    let card_value = cribbage_card_value(card.rank);
    let Some(next_count) = before_public.running_count.checked_add(card_value) else {
        return Err(over_31(action_id));
    };
    if next_count > 31 {
        return Err(over_31(action_id));
    }

    let mut after_public = before_public.clone();
    after_public
        .actor_hand
        .retain(|candidate| *candidate != card);
    after_public.played_cards.push(card);
    after_public.running_count = next_count;
    let points = immediate_pegging_points(&after_public.played_cards, next_count);
    after_public.pegging_points =
        after_public
            .pegging_points
            .checked_add(points)
            .ok_or_else(|| CoreGameError::InvalidParams {
                action_id: action_id.to_string(),
                reason: "pegging points overflow".to_string(),
            })?;

    let after = state_from_public(after_public, state.actor)?;
    let action = core_action_for_card(card);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: CribbagePublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    let mut legal_actions = Vec::new();
    for card in &public.actor_hand {
        if public
            .running_count
            .checked_add(cribbage_card_value(card.rank))
            .is_some_and(|count| count <= 31)
        {
            legal_actions.push(core_action_for_card(*card));
        }
    }
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: "cribbage.bootstrap".to_string(),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: ResearchGame::Cribbage,
        phase: "pegging".to_string(),
        actor,
        public_state,
        private_state_commitments: vec!["cribbage.opponent-hand.bootstrap-v1".to_string()],
        legal_actions,
        terminal: false,
        payoff: None,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<CribbageFeatureView, CoreGameError> {
    let public: CribbagePublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;
    let low_cards = public
        .actor_hand
        .iter()
        .filter(|card| cribbage_card_value(card.rank) <= 5)
        .count();
    let high_cards = public
        .actor_hand
        .iter()
        .filter(|card| cribbage_card_value(card.rank) >= 10)
        .count();
    let legal_cards = public
        .actor_hand
        .iter()
        .copied()
        .filter(|card| {
            public
                .running_count
                .checked_add(cribbage_card_value(card.rank))
                .is_some_and(|count| count <= 31)
        })
        .collect::<Vec<_>>();

    Ok(CribbageFeatureView {
        pegging_count: public.running_count,
        run_potential: usize_to_u8(
            public
                .actor_hand
                .iter()
                .filter(|card| completes_cribbage_run(&public.played_cards, **card))
                .count(),
        ),
        crib_edge: i8::try_from(low_cards)
            .unwrap_or(i8::MAX)
            .saturating_sub(i8::try_from(high_cards).unwrap_or(i8::MAX))
            .clamp(-2, 2),
        pair_trap: public
            .played_cards
            .last()
            .is_some_and(|last| public.actor_hand.iter().any(|card| card.rank == last.rank)),
        go_window: legal_cards.iter().any(|card| {
            public
                .running_count
                .saturating_add(cribbage_card_value(card.rank))
                >= 27
        }),
        fifteen_outs: usize_to_u8(
            legal_cards
                .iter()
                .filter(|card| {
                    public
                        .running_count
                        .saturating_add(cribbage_card_value(card.rank))
                        == 15
                })
                .count(),
        ),
        max_immediate_points: legal_cards
            .iter()
            .map(|card| {
                let mut next_played = public.played_cards.clone();
                next_played.push(*card);
                let next_count = public
                    .running_count
                    .saturating_add(cribbage_card_value(card.rank));
                immediate_pegging_points(&next_played, next_count)
            })
            .max()
            .unwrap_or(0),
    })
}

fn core_action_for_card(card: Card) -> CoreAction {
    CoreAction {
        action_id: format!(
            "{CRIBBAGE_ACTION_PREFIX}{}-{}",
            crate::core::trick_taking::rank_token(card.rank),
            crate::core::trick_taking::suit_token(card.suit)
        ),
        display_label: format!(
            "play-{}-{}",
            crate::core::trick_taking::rank_token(card.rank),
            crate::core::trick_taking::suit_token(card.suit)
        ),
        params: json!({"card": card}),
    }
}

fn parse_cribbage_card_action(action_id: &str) -> Result<Card, CoreGameError> {
    let Some(card_token) = action_id.strip_prefix(CRIBBAGE_ACTION_PREFIX) else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        });
    };
    let Some((rank, suit)) = card_token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        });
    };
    let rank = crate::core::trick_taking::parse_rank(rank).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        }
    })?;
    let suit = crate::core::trick_taking::parse_suit(suit).ok_or_else(|| {
        CoreGameError::UnknownAction {
            game: ResearchGame::Cribbage,
            action_id: action_id.to_string(),
        }
    })?;

    Ok(Card::new(rank, suit))
}

fn cribbage_card_value(rank: Rank) -> u8 {
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

fn immediate_pegging_points(played_cards: &[Card], running_count: u8) -> u8 {
    let mut points: u8 = 0;
    if running_count == 15 || running_count == 31 {
        points = points.saturating_add(2);
    }
    if last_two_are_pair(played_cards) {
        points = points.saturating_add(2);
    }
    if last_three_are_run(played_cards) {
        points = points.saturating_add(3);
    }

    points
}

fn last_two_are_pair(played_cards: &[Card]) -> bool {
    let mut iter = played_cards.iter().rev();
    let Some(last) = iter.next() else {
        return false;
    };
    let Some(previous) = iter.next() else {
        return false;
    };

    last.rank == previous.rank
}

fn last_three_are_run(played_cards: &[Card]) -> bool {
    let mut cards = played_cards
        .iter()
        .rev()
        .take(3)
        .copied()
        .collect::<Vec<_>>();
    if cards.len() != 3 {
        return false;
    }
    cards.reverse();

    is_three_card_run(&cards)
}

fn completes_cribbage_run(played_cards: &[Card], candidate: Card) -> bool {
    if played_cards.len() < 2 {
        return false;
    }
    let Some(previous) = played_cards.get(played_cards.len().saturating_sub(2)..) else {
        return false;
    };
    let mut ranks = previous
        .iter()
        .map(|card| rank_score(card.rank))
        .collect::<Vec<_>>();
    ranks.push(rank_score(candidate.rank));
    ranks.sort_unstable();
    let Some(first) = ranks.first().copied() else {
        return false;
    };
    let Some(second) = ranks.get(1).copied() else {
        return false;
    };
    let Some(third) = ranks.get(2).copied() else {
        return false;
    };

    first.saturating_add(1) == second && second.saturating_add(1) == third
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

fn over_31(action_id: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: ResearchGame::Cribbage,
        action_id: action_id.to_string(),
        reason: "pegging count cannot exceed 31".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn cribbage_bootstrap_state_has_legal_actions() {
        let state = cribbage_state();

        assert_eq!(state.game, ResearchGame::Cribbage);
        assert!(!state.legal_actions.is_empty());
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "cribbage.pegging.play-six-clubs")
        );
    }

    #[test]
    fn cribbage_rejects_illegal_action() {
        let state = cribbage_state();

        assert!(matches!(
            apply_action(&state, "cribbage.pegging.play-three-clubs", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("not in acting hand")
        ));
    }

    #[test]
    fn cribbage_transition_is_deterministic() {
        let state = cribbage_state();
        let first = apply_action(&state, "cribbage.pegging.play-six-clubs", json!({}));
        let second = apply_action(&state, "cribbage.pegging.play-six-clubs", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn cribbage_pegging_rejects_over_31() {
        let state = over_31_state();

        assert!(matches!(
            apply_action(&state, "cribbage.pegging.play-king-hearts", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("exceed 31")
        ));
    }

    #[test]
    fn cribbage_nonterminal_state_has_no_payoff() {
        let state = cribbage_state();
        let transition = match apply_action(&state, "cribbage.pegging.play-six-clubs", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("cribbage legal action should apply: {error}"),
        };

        assert!(!transition.after.terminal);
        assert_eq!(transition.after.payoff, None);
    }

    #[test]
    fn cribbage_scores_fifteen_and_three_card_run() {
        let state = cribbage_state();
        let transition = match apply_action(&state, "cribbage.pegging.play-six-clubs", json!({})) {
            Ok(transition) => transition,
            Err(error) => panic!("cribbage legal action should apply: {error}"),
        };
        let public: CribbagePublicState =
            match serde_json::from_value(transition.after.public_state) {
                Ok(public) => public,
                Err(error) => panic!("cribbage public state should decode: {error}"),
            };

        assert_eq!(public.running_count, 15);
        assert_eq!(public.pegging_points, 5);
    }

    #[test]
    fn cribbage_feature_view_tracks_run_and_edge() {
        let state = cribbage_state();
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("cribbage view should decode: {error}"));

        assert_eq!(view.pegging_count, 9);
        assert_eq!(view.run_potential, 1);
        assert_eq!(view.crib_edge, 1);
        assert!(!view.pair_trap);
        assert!(!view.go_window);
        assert_eq!(view.fifteen_outs, 1);
        assert_eq!(view.max_immediate_points, 5);
    }

    fn cribbage_state() -> CoreGameState {
        match bootstrap_state(ResearchGame::Cribbage) {
            Ok(state) => state,
            Err(error) => panic!("cribbage bootstrap should succeed: {error}"),
        }
    }

    fn over_31_state() -> CoreGameState {
        let public = CribbagePublicState {
            running_count: 25,
            pegging_points: 0,
            actor_hand: vec![Card::new(Rank::King, Suit::Hearts)],
            played_cards: Vec::new(),
        };

        match state_from_public(public, Some(0)) {
            Ok(state) => state,
            Err(error) => panic!("over-31 state should build: {error}"),
        }
    }
}
