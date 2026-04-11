use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cards::{Card, Rank, Suit};
use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::eval::trick_taking::legal_follow_suit_cards;
use crate::game::ResearchGame;

const BRIDGE_ACTION_PREFIX: &str = "bridge.play.";
const SPADES_ACTION_PREFIX: &str = "spades.play.";
const CALL_BREAK_ACTION_PREFIX: &str = "call-break.play.";
const HEARTS_ACTION_PREFIX: &str = "hearts.play.";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TrickVariant {
    Bridge,
    Spades,
    CallBreak,
    Hearts,
}

impl TrickVariant {
    const fn game(self) -> ResearchGame {
        match self {
            Self::Bridge => ResearchGame::Bridge,
            Self::Spades => ResearchGame::Spades,
            Self::CallBreak => ResearchGame::CallBreak,
            Self::Hearts => ResearchGame::Hearts,
        }
    }

    const fn action_prefix(self) -> &'static str {
        match self {
            Self::Bridge => BRIDGE_ACTION_PREFIX,
            Self::Spades => SPADES_ACTION_PREFIX,
            Self::CallBreak => CALL_BREAK_ACTION_PREFIX,
            Self::Hearts => HEARTS_ACTION_PREFIX,
        }
    }

    const fn trump(self) -> Option<Suit> {
        match self {
            Self::Bridge => Some(Suit::Spades),
            Self::Spades | Self::CallBreak => Some(Suit::Spades),
            Self::Hearts => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct TrickTakingPublicState {
    variant: TrickVariant,
    led_suit: Option<Suit>,
    trump: Option<Suit>,
    current_trick: Vec<PlayedCard>,
    acting_hand: Vec<Card>,
    tricks_won: Vec<u8>,
    hearts_broken: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PlayedCard {
    seat: u8,
    card: Card,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TrickTakingFeatureView {
    pub trump_count: u8,
    pub winners: u8,
    pub void_suits: u8,
    pub actor_tricks_won: u8,
    pub penalty_pressure: u8,
    pub cards_in_trick: u8,
    pub follow_suit_forced: bool,
    pub nil_viable: bool,
    pub moon_shot_viable: bool,
}

pub fn bridge_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(
        TrickTakingPublicState {
            variant: TrickVariant::Bridge,
            led_suit: Some(Suit::Hearts),
            trump: TrickVariant::Bridge.trump(),
            current_trick: vec![PlayedCard {
                seat: 3,
                card: Card::new(Rank::Ten, Suit::Hearts),
            }],
            acting_hand: vec![
                Card::new(Rank::Ace, Suit::Spades),
                Card::new(Rank::King, Suit::Hearts),
                Card::new(Rank::Queen, Suit::Hearts),
            ],
            tricks_won: vec![0; 4],
            hearts_broken: true,
        },
        Some(0),
    )
}

pub fn spades_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(
        TrickTakingPublicState {
            variant: TrickVariant::Spades,
            led_suit: Some(Suit::Spades),
            trump: TrickVariant::Spades.trump(),
            current_trick: vec![PlayedCard {
                seat: 1,
                card: Card::new(Rank::Ten, Suit::Spades),
            }],
            acting_hand: vec![
                Card::new(Rank::Queen, Suit::Spades),
                Card::new(Rank::Ace, Suit::Clubs),
                Card::new(Rank::Four, Suit::Hearts),
            ],
            tricks_won: vec![1, 0, 0, 0],
            hearts_broken: true,
        },
        Some(2),
    )
}

pub fn call_break_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(
        TrickTakingPublicState {
            variant: TrickVariant::CallBreak,
            led_suit: Some(Suit::Hearts),
            trump: TrickVariant::CallBreak.trump(),
            current_trick: vec![PlayedCard {
                seat: 0,
                card: Card::new(Rank::Nine, Suit::Hearts),
            }],
            acting_hand: vec![
                Card::new(Rank::Ace, Suit::Spades),
                Card::new(Rank::King, Suit::Hearts),
                Card::new(Rank::Three, Suit::Hearts),
            ],
            tricks_won: vec![1, 1, 0, 0],
            hearts_broken: true,
        },
        Some(1),
    )
}

pub fn hearts_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(
        TrickTakingPublicState {
            variant: TrickVariant::Hearts,
            led_suit: Some(Suit::Clubs),
            trump: TrickVariant::Hearts.trump(),
            current_trick: vec![PlayedCard {
                seat: 1,
                card: Card::new(Rank::Ten, Suit::Clubs),
            }],
            acting_hand: vec![
                Card::new(Rank::Two, Suit::Clubs),
                Card::new(Rank::Queen, Suit::Spades),
                Card::new(Rank::Ace, Suit::Hearts),
            ],
            tricks_won: vec![0, 0, 0, 0],
            hearts_broken: false,
        },
        Some(2),
    )
}

pub fn apply_bridge_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, TrickVariant::Bridge)
}

pub fn apply_spades_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, TrickVariant::Spades)
}

pub fn apply_call_break_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, TrickVariant::CallBreak)
}

pub fn apply_hearts_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, TrickVariant::Hearts)
}

fn apply_variant_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
    variant: TrickVariant,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: TrickTakingPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    if before_public.variant != variant {
        return Err(CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: "state variant does not match trick-taking dispatch target".to_string(),
        });
    }
    let card = parse_variant_card_action(variant, action_id)?;
    if !before_public.acting_hand.contains(&card) {
        return Err(CoreGameError::IllegalAction {
            game: variant.game(),
            action_id: action_id.to_string(),
            reason: "card is not in acting hand".to_string(),
        });
    }

    let legal_cards = legal_follow_suit_cards(&before_public.acting_hand, before_public.led_suit);
    if !legal_cards.contains(&card) {
        return Err(CoreGameError::IllegalAction {
            game: variant.game(),
            action_id: action_id.to_string(),
            reason: "must follow led suit while holding that suit".to_string(),
        });
    }
    if variant == TrickVariant::Hearts
        && before_public.led_suit.is_none()
        && !before_public.hearts_broken
        && card.suit == Suit::Hearts
        && before_public
            .acting_hand
            .iter()
            .any(|candidate| candidate.suit != Suit::Hearts)
    {
        return Err(CoreGameError::IllegalAction {
            game: variant.game(),
            action_id: action_id.to_string(),
            reason: "hearts cannot be led before they are broken".to_string(),
        });
    }

    let actor = state.actor.unwrap_or(0);
    let mut after_public = before_public.clone();
    after_public
        .acting_hand
        .retain(|candidate| *candidate != card);
    after_public
        .current_trick
        .push(PlayedCard { seat: actor, card });
    if after_public.led_suit.is_none() {
        after_public.led_suit = Some(card.suit);
    }
    if card.suit == Suit::Hearts {
        after_public.hearts_broken = true;
    }

    let next_actor = if after_public.current_trick.len() == 4 {
        let winner = trick_winner(&after_public).ok_or_else(|| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: "completed trick has no winner".to_string(),
        })?;
        let winner_index = usize::from(winner);
        let won = after_public
            .tricks_won
            .get_mut(winner_index)
            .ok_or_else(|| CoreGameError::InvalidParams {
                action_id: action_id.to_string(),
                reason: "winner index is outside trick score vector".to_string(),
            })?;
        *won = won.saturating_add(1);
        after_public.current_trick.clear();
        after_public.led_suit = None;
        Some(winner)
    } else {
        next_actor(actor)
    };
    let after = state_from_public(after_public, next_actor)?;
    let action = core_action_for_card(variant, card);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: TrickTakingPublicState,
    actor: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    let variant = public.variant;
    let legal_actions = legal_follow_suit_cards(&public.acting_hand, public.led_suit)
        .into_iter()
        .map(|card| core_action_for_card(variant, card))
        .collect();
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: format!("{}.bootstrap", variant.game().slug()),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: variant.game(),
        phase: "play".to_string(),
        actor,
        public_state,
        private_state_commitments: vec![format!(
            "{}.other-hands.bootstrap-v1",
            variant.game().slug()
        )],
        legal_actions,
        terminal: false,
        payoff: None,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<TrickTakingFeatureView, CoreGameError> {
    let public: TrickTakingPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: format!("{}.feature-view", state.game.slug()),
            reason: source.to_string(),
        })?;
    let unique_suits = public
        .acting_hand
        .iter()
        .map(|card| card.suit)
        .collect::<std::collections::BTreeSet<_>>();
    let follow_suit_forced = public
        .led_suit
        .is_some_and(|led| public.acting_hand.iter().any(|card| card.suit == led));
    let trump_count = usize_to_u8(
        public
            .trump
            .map(|trump| {
                public
                    .acting_hand
                    .iter()
                    .filter(|card| card.suit == trump)
                    .count()
            })
            .unwrap_or_default(),
    );
    let winners = usize_to_u8(
        public
            .acting_hand
            .iter()
            .filter(|card| matches!(card.rank, Rank::Queen | Rank::King | Rank::Ace))
            .count()
            .saturating_add(usize::from(trump_count > 0)),
    );
    let penalty_pressure = if public.variant == TrickVariant::Hearts {
        usize_to_u8(
            public
                .acting_hand
                .iter()
                .filter(|card| {
                    card.suit == Suit::Hearts
                        || (card.suit == Suit::Spades && card.rank == Rank::Queen)
                })
                .count(),
        )
    } else {
        0
    };
    let actor = usize::from(state.actor.unwrap_or_default());

    Ok(TrickTakingFeatureView {
        trump_count,
        winners,
        void_suits: usize_to_u8(4usize.saturating_sub(unique_suits.len())),
        actor_tricks_won: public.tricks_won.get(actor).copied().unwrap_or_default(),
        penalty_pressure,
        cards_in_trick: usize_to_u8(public.current_trick.len()),
        follow_suit_forced,
        nil_viable: public.variant != TrickVariant::Hearts && winners == 0 && trump_count == 0,
        moon_shot_viable: public.variant == TrickVariant::Hearts
            && penalty_pressure >= 4
            && winners >= 2,
    })
}

fn core_action_for_card(variant: TrickVariant, card: Card) -> CoreAction {
    CoreAction {
        action_id: format!(
            "{}{}-{}",
            variant.action_prefix(),
            rank_token(card.rank),
            suit_token(card.suit)
        ),
        display_label: format!("play-{}-{}", rank_token(card.rank), suit_token(card.suit)),
        params: json!({"card": card}),
    }
}

fn parse_variant_card_action(
    variant: TrickVariant,
    action_id: &str,
) -> Result<Card, CoreGameError> {
    let Some(card_token) = action_id.strip_prefix(variant.action_prefix()) else {
        return Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        });
    };
    parse_card_token(variant.game(), action_id, card_token)
}

fn parse_card_token(
    game: ResearchGame,
    action_id: &str,
    card_token: &str,
) -> Result<Card, CoreGameError> {
    let Some((rank, suit)) = card_token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game,
            action_id: action_id.to_string(),
        });
    };
    let rank = parse_rank(rank).ok_or_else(|| CoreGameError::UnknownAction {
        game,
        action_id: action_id.to_string(),
    })?;
    let suit = parse_suit(suit).ok_or_else(|| CoreGameError::UnknownAction {
        game,
        action_id: action_id.to_string(),
    })?;

    Ok(Card::new(rank, suit))
}

fn trick_winner(public: &TrickTakingPublicState) -> Option<u8> {
    let led_suit = public
        .led_suit
        .or_else(|| public.current_trick.first().map(|played| played.card.suit))?;
    public
        .current_trick
        .iter()
        .filter(|played| played.card.suit == public.trump.unwrap_or(led_suit))
        .max_by_key(|played| played.card.rank)
        .or_else(|| {
            public
                .current_trick
                .iter()
                .filter(|played| played.card.suit == led_suit)
                .max_by_key(|played| played.card.rank)
        })
        .map(|played| played.seat)
}

fn next_actor(actor: u8) -> Option<u8> {
    let next = actor.checked_add(1)?;
    if next >= 4 { Some(0) } else { Some(next) }
}

pub(crate) fn rank_token(rank: Rank) -> &'static str {
    match rank {
        Rank::Two => "two",
        Rank::Three => "three",
        Rank::Four => "four",
        Rank::Five => "five",
        Rank::Six => "six",
        Rank::Seven => "seven",
        Rank::Eight => "eight",
        Rank::Nine => "nine",
        Rank::Ten => "ten",
        Rank::Jack => "jack",
        Rank::Queen => "queen",
        Rank::King => "king",
        Rank::Ace => "ace",
    }
}

pub(crate) fn suit_token(suit: Suit) -> &'static str {
    match suit {
        Suit::Clubs => "clubs",
        Suit::Diamonds => "diamonds",
        Suit::Hearts => "hearts",
        Suit::Spades => "spades",
    }
}

pub(crate) fn parse_rank(token: &str) -> Option<Rank> {
    match token {
        "two" => Some(Rank::Two),
        "three" => Some(Rank::Three),
        "four" => Some(Rank::Four),
        "five" => Some(Rank::Five),
        "six" => Some(Rank::Six),
        "seven" => Some(Rank::Seven),
        "eight" => Some(Rank::Eight),
        "nine" => Some(Rank::Nine),
        "ten" => Some(Rank::Ten),
        "jack" => Some(Rank::Jack),
        "queen" => Some(Rank::Queen),
        "king" => Some(Rank::King),
        "ace" => Some(Rank::Ace),
        _ => None,
    }
}

pub(crate) fn parse_suit(token: &str) -> Option<Suit> {
    match token {
        "clubs" => Some(Suit::Clubs),
        "diamonds" => Some(Suit::Diamonds),
        "hearts" => Some(Suit::Hearts),
        "spades" => Some(Suit::Spades),
        _ => None,
    }
}

fn usize_to_u8(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use myosu_games::CanonicalStateSnapshot;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn bridge_bootstrap_state_has_legal_actions() {
        let state = trick_state(ResearchGame::Bridge);

        assert_eq!(state.game, ResearchGame::Bridge);
        assert!(
            state
                .legal_actions
                .iter()
                .all(|action| action.action_id.contains("hearts"))
        );
    }

    #[test]
    fn spades_bootstrap_state_has_spades_action() {
        let state = trick_state(ResearchGame::Spades);

        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "spades.play.queen-spades")
        );
    }

    #[test]
    fn bridge_rejects_illegal_action() {
        let state = trick_state(ResearchGame::Bridge);

        assert!(matches!(
            apply_action(&state, "bridge.play.ace-clubs", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("not in acting hand")
        ));
    }

    #[test]
    fn bridge_rejects_not_following_suit() {
        let state = trick_state(ResearchGame::Bridge);

        assert!(matches!(
            apply_action(&state, "bridge.play.ace-spades", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("follow led suit")
        ));
    }

    #[test]
    fn hearts_cannot_lead_hearts_before_break() {
        let mut public: TrickTakingPublicState =
            serde_json::from_value(trick_state(ResearchGame::Hearts).public_state)
                .unwrap_or_else(|error| panic!("hearts public state should decode: {error}"));
        public.led_suit = None;
        public.current_trick.clear();
        public.acting_hand = vec![
            Card::new(Rank::Ace, Suit::Hearts),
            Card::new(Rank::King, Suit::Clubs),
        ];
        let state = state_from_public(public, Some(0))
            .unwrap_or_else(|error| panic!("hearts lead state should build: {error}"));

        assert!(matches!(
            apply_action(&state, "hearts.play.ace-hearts", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("broken")
        ));
    }

    #[test]
    fn call_break_transition_is_deterministic() {
        let state = trick_state(ResearchGame::CallBreak);
        let first = apply_action(&state, "call-break.play.king-hearts", json!({}));
        let second = apply_action(&state, "call-break.play.king-hearts", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn bridge_nonterminal_state_has_no_payoff() {
        let state = trick_state(ResearchGame::Bridge);
        let transition = apply_action(&state, "bridge.play.king-hearts", json!({}))
            .unwrap_or_else(|error| panic!("bridge legal action should apply: {error}"));

        assert!(!transition.after.terminal);
        assert_eq!(transition.after.payoff, None);
        let snapshot = CanonicalStateSnapshot::from(transition.after);
        assert_eq!(snapshot.game_id, "bridge");
    }

    #[test]
    fn hearts_feature_view_tracks_penalty_pressure() {
        let state = trick_state(ResearchGame::Hearts);
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("hearts view should decode: {error}"));

        assert_eq!(view.penalty_pressure, 2);
        assert_eq!(view.trump_count, 0);
        assert_eq!(view.void_suits, 1);
        assert_eq!(view.cards_in_trick, 1);
        assert!(view.follow_suit_forced);
    }

    fn trick_state(game: ResearchGame) -> CoreGameState {
        bootstrap_state(game)
            .unwrap_or_else(|error| panic!("{} bootstrap should succeed: {error}", game.slug()))
    }
}
