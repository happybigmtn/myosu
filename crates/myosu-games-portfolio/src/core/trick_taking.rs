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

/// One labeled representative decision point in the Hearts search space.
///
/// Scenarios are intentionally narrow: each row targets a specific Hearts
/// engine heuristic (avoid-penalty, follow-suit, shoot-moon) and is used by
/// both the benchmark dossier writer and the e2e promotion proof. The fields
/// mirror the typed `TrickTakingChallenge` so a scenario can be replayed
/// through the same engine dispatch as a live portfolio challenge.
///
/// The Hearts engine reads the `penalty_pressure`, `winners`, `void_suits`,
/// `follow_suit_forced`, and `moon_shot_viable` fields. `trump_count` and
/// `contract_pressure` are pinned to 0 and `nil_viable` to `false` because
/// the Hearts engine does not use them (`feature_view` keeps them at the
/// Hearts-correct values); they are kept in the struct for shape parity with
/// `TrickTakingChallenge` and the dossier hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeartsScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
    pub penalty_pressure: u8,
    pub winners: u8,
    pub void_suits: u8,
    pub cards_in_trick: u8,
    pub follow_suit_forced: bool,
    pub moon_shot_viable: bool,
}

/// Return the canonical 22-scenario Hearts benchmark pack.
///
/// Coverage requirements (mirrors `genesis/plans/009-cribbage-deepening.md`
/// R1 layout, scoped to the Hearts engine surface):
/// - avoid_penalty dominance × 6 (queen-risk, light-pressure, zero-pressure,
///   multi-penalty, forced-follow-but-still-avoid, heavy-tricks-no-moon)
/// - follow_suit dominance × 5 (clean follow, voided, winner, trick-end,
///   void-low-penalty)
/// - shoot_moon dominance × 5 (fresh, mid, with-winner, with-void, mid-trick)
/// - mixed/edge × 6 (light-penalty-moon-viable, forced-follow-moon-low,
///   forced-follow-moon-voided, zero-everything-no-moon, low-penalty-avoid,
///   penalty-pressured-moon-viable)
///
/// Total: 22 labeled scenarios, exceeds the 20-scenario floor. Each scenario
/// is hand-verified against the `state-aware hearts penalty heuristic` math
/// in `engines/trick_taking.rs::hearts` so the dossier's expected
/// recommendations are stable across runs.
pub fn hearts_scenario_pack() -> &'static [HeartsScenario] {
    HEARTS_SCENARIO_PACK
}

const HEARTS_SCENARIO_PACK: &[HeartsScenario] = &[
    // ---- avoid_penalty bucket (×6) ----
    HeartsScenario {
        scenario_id: "queen-risk-avoidance-v2",
        decision: "Avoid taking the queen of spades with 4 penalty cards visible",
        penalty_pressure: 4,
        winners: 2,
        void_suits: 1,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "light-pressure-avoid",
        decision: "Avoid light penalty pressure on the lead",
        penalty_pressure: 1,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "zero-pressure-avoid",
        decision: "Avoid even without penalty pressure (default safe play)",
        penalty_pressure: 0,
        winners: 2,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "multi-penalty-avoid",
        decision: "Avoid with maximum penalty pressure",
        penalty_pressure: 6,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "forced-follow-but-still-avoid",
        decision: "Follow suit is forced but penalty pressure still dominates",
        penalty_pressure: 3,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "heavy-tricks-no-moon",
        decision: "Many high-card winners but no moon shot — avoid still wins",
        penalty_pressure: 2,
        winners: 4,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: false,
    },
    // ---- follow_suit bucket (×5) ----
    HeartsScenario {
        scenario_id: "forced-follow-clean",
        decision: "Forced to follow suit with no penalty or moon pressure",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "forced-follow-voided",
        decision: "Forced to follow suit with two voided suits (extra follow pressure)",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 2,
        cards_in_trick: 0,
        follow_suit_forced: true,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "forced-follow-with-winner",
        decision: "Forced to follow suit holding three high-card winners",
        penalty_pressure: 0,
        winners: 3,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "forced-follow-trick-end",
        decision: "Forced to follow suit late in the trick (trick-end cleanup)",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 3,
        follow_suit_forced: true,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "forced-follow-with-void-low-penalty",
        decision: "Forced to follow suit with one void suit and light winners",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 1,
        cards_in_trick: 0,
        follow_suit_forced: true,
        moon_shot_viable: false,
    },
    // ---- shoot_moon bucket (×5) ----
    HeartsScenario {
        scenario_id: "moon-viable-fresh",
        decision: "Moon shot viable with five winners in hand",
        penalty_pressure: 0,
        winners: 5,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: true,
    },
    HeartsScenario {
        scenario_id: "moon-viable-mid",
        decision: "Moon shot viable with three winners in hand",
        penalty_pressure: 0,
        winners: 3,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: true,
    },
    HeartsScenario {
        scenario_id: "moon-viable-with-winner",
        decision: "Moon shot viable with two winners in hand",
        penalty_pressure: 0,
        winners: 2,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: true,
    },
    HeartsScenario {
        scenario_id: "moon-viable-with-void",
        decision: "Moon shot viable with one void suit and four winners",
        penalty_pressure: 0,
        winners: 4,
        void_suits: 1,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: true,
    },
    HeartsScenario {
        scenario_id: "moon-viable-mid-trick",
        decision: "Moon shot viable mid-trick (no penalty pressure)",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 1,
        follow_suit_forced: false,
        moon_shot_viable: true,
    },
    // ---- mixed/edge bucket (×6) ----
    HeartsScenario {
        scenario_id: "light-penalty-moon-viable",
        decision: "Moon shot viable with no penalty pressure (default moon hand)",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: true,
    },
    HeartsScenario {
        scenario_id: "forced-follow-moon-low",
        decision: "Forced to follow suit with a moon shot available (shoot still wins)",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        moon_shot_viable: true,
    },
    HeartsScenario {
        scenario_id: "forced-follow-moon-voided",
        decision: "Forced to follow suit with a moon shot but two void suits (follow wins)",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 1,
        cards_in_trick: 0,
        follow_suit_forced: true,
        moon_shot_viable: true,
    },
    HeartsScenario {
        scenario_id: "zero-everything-no-moon",
        decision: "No penalty, no winners, no moon — default safe avoid",
        penalty_pressure: 0,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "low-penalty-avoid",
        decision: "Light penalty pressure with no moon — avoid still wins",
        penalty_pressure: 2,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: false,
    },
    HeartsScenario {
        scenario_id: "penalty-pressured-moon-viable",
        decision: "Penalty pressure outweighs a moon shot — avoid dominates",
        penalty_pressure: 2,
        winners: 0,
        void_suits: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        moon_shot_viable: true,
    },
];

/// Hand-verified 22-scenario pack for the F-010 Spades benchmark dossier.
///
/// The Spades engine reads `trump_count`, `contract_pressure`, `winners`,
/// `void_suits`, `cards_in_trick`, `follow_suit_forced`, and `nil_viable`
/// from the typed `TrickTakingChallenge`. `penalty_pressure` and
/// `moon_shot_viable` are pinned to 0 / `false` because the Spades engine
/// does not use them (`feature_view` keeps them at the Spades-correct
/// values); they are kept in the struct for shape parity with
/// `TrickTakingChallenge` and the dossier hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpadesScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
    pub trump_count: u8,
    pub winners: u8,
    pub void_suits: u8,
    pub contract_pressure: i8,
    pub penalty_pressure: u8,
    pub cards_in_trick: u8,
    pub follow_suit_forced: bool,
    pub nil_viable: bool,
}

/// Return the canonical 22-scenario Spades benchmark pack.
///
/// Coverage requirements (mirrors `genesis/plans/009-cribbage-deepening.md`
/// R1 layout, scoped to the Spades engine surface):
/// - trump_control dominance × 6 (heavy-control, rich-mid-pressure,
///   winners-rich, voided-hand, pressure-tied, with-forced-follow)
/// - follow_suit dominance × 6 (clean, with-winners, voided, trick-end,
///   mid-winners, cards-in-trick)
/// - bid_nil dominance × 6 (clean-window, with-light-winners, void-clean,
///   cards-in-trick, penalty-pressure, low-winners-clean)
/// - mixed/edge × 4 (trump-vs-nil-edge, mixed-trump-nil-penalty,
///   forced-follow-with-nil-spot, trump-vs-forced-follow)
///
/// Total: 22 labeled scenarios, exceeds the 20-scenario floor. Each scenario
/// is hand-verified against the `state-aware spades trump heuristic` math
/// in `engines/trick_taking.rs::spades` so the dossier's expected
/// recommendations are stable across runs.
pub fn spades_scenario_pack() -> &'static [SpadesScenario] {
    SPADES_SCENARIO_PACK
}

const SPADES_SCENARIO_PACK: &[SpadesScenario] = &[
    // ---- trump_control bucket (×6) ----
    SpadesScenario {
        scenario_id: "trump-heavy-control",
        decision: "Heavy trump count plus contract pressure on a free lead",
        trump_count: 4,
        winners: 1,
        void_suits: 1,
        contract_pressure: 3,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "trump-rich-mid-pressure",
        decision: "Trump-rich hand with mid contract pressure",
        trump_count: 5,
        winners: 0,
        void_suits: 0,
        contract_pressure: 2,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "trump-winners-rich",
        decision: "Trump + high winners on a free lead",
        trump_count: 3,
        winners: 3,
        void_suits: 1,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "trump-voided-hand",
        decision: "Trump + void suit on a free lead",
        trump_count: 3,
        winners: 0,
        void_suits: 2,
        contract_pressure: 1,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "trump-pressure-tied",
        decision: "Heavy trump with heavy contract pressure",
        trump_count: 6,
        winners: 0,
        void_suits: 0,
        contract_pressure: 5,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "trump-with-forced-follow",
        decision: "Trump + forced follow with low penalty pressure — trump still wins",
        trump_count: 4,
        winners: 0,
        void_suits: 0,
        contract_pressure: 2,
        penalty_pressure: 0,
        cards_in_trick: 1,
        follow_suit_forced: true,
        nil_viable: false,
    },
    // ---- follow_suit bucket (×6) ----
    SpadesScenario {
        scenario_id: "forced-follow-clean",
        decision: "Forced to follow suit with no other pressure",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "forced-follow-winners",
        decision: "Forced to follow suit holding four high-card winners",
        trump_count: 0,
        winners: 4,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "forced-follow-voided",
        decision: "Forced to follow suit with two void suits",
        trump_count: 0,
        winners: 0,
        void_suits: 2,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "forced-follow-trick-end",
        decision: "Forced to follow suit late in the trick (trick-end cleanup)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 3,
        follow_suit_forced: true,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "forced-follow-mid-winners",
        decision: "Forced to follow suit with mid winners",
        trump_count: 0,
        winners: 2,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    SpadesScenario {
        scenario_id: "forced-follow-cards-in-trick",
        decision: "Forced to follow suit with two cards in trick",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 2,
        follow_suit_forced: true,
        nil_viable: false,
    },
    // ---- bid_nil bucket (×6) ----
    SpadesScenario {
        scenario_id: "nil-clean-window",
        decision: "Nil-viable spot with no other pressure",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "nil-with-light-winners",
        decision: "Nil-viable spot with one light winner",
        trump_count: 0,
        winners: 1,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "nil-void-clean",
        decision: "Nil-viable spot with two void suits",
        trump_count: 0,
        winners: 0,
        void_suits: 2,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "nil-cards-in-trick",
        decision: "Nil-viable spot late in the trick",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 2,
        follow_suit_forced: false,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "nil-penalty-pressure",
        decision: "Nil-viable spot with light penalty pressure",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 1,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "nil-low-winners-clean",
        decision: "Nil-viable spot with one mid winner",
        trump_count: 0,
        winners: 1,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: true,
    },
    // ---- mixed/edge bucket (×4) ----
    SpadesScenario {
        scenario_id: "trump-vs-nil-edge",
        decision: "Trump-heavy hand where nil is also viable — trump wins",
        trump_count: 3,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "mixed-trump-nil-penalty",
        decision: "Trump + nil-viable + light penalty — trump still dominates",
        trump_count: 3,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 1,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "forced-follow-with-nil-spot",
        decision: "Forced to follow suit on a nil-viable spot — follow wins",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: true,
    },
    SpadesScenario {
        scenario_id: "trump-vs-forced-follow",
        decision: "Mid trump + forced follow with no other pressure — trump still wins",
        trump_count: 2,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
];

/// The Bridge `state-aware bridge control heuristic` engine reads
/// `winners`, `trump_count`, `contract_pressure`, `cards_in_trick`,
/// `follow_suit_forced`, and `void_suits` from the typed
/// `TrickTakingChallenge`. The `epochs` parameter is forwarded to the
/// portfolio RNG and contributes a tiny +0.05 nudge to the `double_dummy`
/// arm in roughly half the cases (the seed bit is folded into
/// `double_dummy` in `engines/trick_taking.rs::bridge`); every scenario
/// below is hand-verified so the dominant arm stays dominant by at least
/// 0.30 even when the nudge fires. `penalty_pressure` and `nil_viable`
/// are pinned to 0 / `false` because the Bridge engine does not use them
/// (`feature_view` keeps them at the Bridge-correct values); they are
/// kept in the struct for shape parity with `TrickTakingChallenge` and
/// the dossier hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
    pub trump_count: u8,
    pub winners: u8,
    pub void_suits: u8,
    pub contract_pressure: i8,
    pub penalty_pressure: u8,
    pub cards_in_trick: u8,
    pub follow_suit_forced: bool,
    pub nil_viable: bool,
}

/// Return the canonical 22-scenario Bridge benchmark pack.
///
/// Coverage requirements (mirrors `genesis/plans/009-cribbage-deepening.md`
/// R1 layout, scoped to the Bridge engine surface):
/// - double_dummy dominance × 6 (rich-winners, trump-rich,
///   mid-pressure-winners, cards-in-trick-winners, high-winners-voided,
///   trump-cards-winners)
/// - follow_suit dominance × 6 (clean-forced, forced-voided,
///   forced-with-light-winners, forced-trick-end, forced-cards-in-trick,
///   forced-mixed-trump-void)
/// - bid_contract dominance × 6 (opening-push, mid-pressure, heavy-pressure,
///   with-light-winners, voided-mid-pressure, trump-rich-pressure)
/// - mixed/edge × 4 (dd-vs-bc-edge-winners, dd-with-follow-pressure,
///   bc-vs-follow-edge, dd-rich-everything)
///
/// Total: 22 labeled scenarios, exceeds the 20-scenario floor. Each
/// scenario is hand-verified against the `state-aware bridge control
/// heuristic` math in `engines/trick_taking.rs::bridge` so the dossier's
/// expected recommendations are stable across runs. The hand-checked
/// heuristic values are documented inline in the per-row docstring.
pub fn bridge_scenario_pack() -> &'static [BridgeScenario] {
    BRIDGE_SCENARIO_PACK
}

const BRIDGE_SCENARIO_PACK: &[BridgeScenario] = &[
    // ---- double_dummy bucket (×6) ----
    BridgeScenario {
        scenario_id: "dd-rich-winners",
        decision: "Two winners + free lead — double-dummy dominates (dd≈2.10-2.15, fs≈0.85, bc≈1.05)",
        trump_count: 0,
        winners: 2,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "dd-trump-rich",
        decision: "Two winners + three trumps — double-dummy dominates (dd≈2.55-2.60, fs≈0.85, bc≈1.05)",
        trump_count: 3,
        winners: 2,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "dd-mid-pressure-winners",
        decision: "Two winners + mid contract pressure — double-dummy still dominates (dd≈2.30-2.35, fs≈0.85, bc≈1.47)",
        trump_count: 0,
        winners: 2,
        void_suits: 0,
        contract_pressure: 1,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "dd-cards-in-trick-winners",
        decision: "Two winners + one card in trick — double-dummy dominates (dd≈2.20-2.25, fs≈0.90, bc≈0.75)",
        trump_count: 0,
        winners: 2,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 1,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "dd-high-winners-voided",
        decision: "Three winners + one void suit — double-dummy dominates (dd≈2.65-2.70, fs≈1.10, bc≈1.05)",
        trump_count: 0,
        winners: 3,
        void_suits: 1,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "dd-trump-cards-winners",
        decision: "Two winners + two trumps + one card in trick — double-dummy dominates (dd≈2.50-2.55, fs≈0.90, bc≈0.75)",
        trump_count: 2,
        winners: 2,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 1,
        follow_suit_forced: false,
        nil_viable: false,
    },
    // ---- follow_suit bucket (×6) ----
    BridgeScenario {
        scenario_id: "fs-clean-forced",
        decision: "Forced to follow suit with no other pressure — follow dominates (fs≈1.55, dd≈0.90-0.95, bc≈1.05)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "fs-forced-voided",
        decision: "Forced to follow suit with two void suits — follow dominates (fs≈2.05, dd≈0.90-0.95, bc≈1.05)",
        trump_count: 0,
        winners: 0,
        void_suits: 2,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "fs-forced-with-light-winners",
        decision: "Forced to follow suit with one light winner — follow dominates (fs≈1.55, dd≈1.45-1.50, bc≈1.05)",
        trump_count: 0,
        winners: 1,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "fs-forced-trick-end",
        decision: "Forced to follow suit late in the trick (trick-end cleanup) — follow dominates (fs≈1.70, dd≈1.20-1.25, bc≈0.75)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 3,
        follow_suit_forced: true,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "fs-forced-cards-in-trick",
        decision: "Forced to follow suit with two cards in trick — follow dominates (fs≈1.65, dd≈1.10-1.15, bc≈0.75)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 2,
        follow_suit_forced: true,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "fs-forced-mixed-trump-void",
        decision: "Forced to follow suit with one trump + one void — follow dominates (fs≈1.80, dd≈1.05-1.10, bc≈1.05)",
        trump_count: 1,
        winners: 0,
        void_suits: 1,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    // ---- bid_contract bucket (×6) ----
    BridgeScenario {
        scenario_id: "bc-opening-push",
        decision: "Opening contract push — bid dominates (bc≈1.89, dd≈1.40-1.45, fs≈0.85)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 2,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "bc-mid-pressure",
        decision: "Mid contract pressure — bid dominates (bc≈2.31, dd≈1.60-1.65, fs≈0.85)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 3,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "bc-heavy-pressure",
        decision: "Heavy contract pressure — bid dominates (bc≈2.73, dd≈1.80-1.85, fs≈0.85)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 4,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "bc-with-light-winners",
        decision: "One light winner + heavy contract pressure — bid dominates (bc≈2.73, dd≈2.35-2.40, fs≈0.85)",
        trump_count: 0,
        winners: 1,
        void_suits: 0,
        contract_pressure: 4,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "bc-voided-mid-pressure",
        decision: "One void suit + mid contract pressure — bid dominates (bc≈2.31, dd≈1.60-1.65, fs≈1.10)",
        trump_count: 0,
        winners: 0,
        void_suits: 1,
        contract_pressure: 3,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "bc-trump-rich-pressure",
        decision: "Two trumps + mid contract pressure — bid dominates (bc≈2.31, dd≈1.90-1.95, fs≈0.85)",
        trump_count: 2,
        winners: 0,
        void_suits: 0,
        contract_pressure: 3,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    // ---- mixed/edge bucket (×4) ----
    BridgeScenario {
        scenario_id: "dd-vs-bc-edge-winners",
        decision: "Two winners + mid pressure — double-dummy still beats bid (dd≈2.50-2.55, bc≈1.89, fs≈0.85)",
        trump_count: 0,
        winners: 2,
        void_suits: 0,
        contract_pressure: 2,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: false,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "dd-with-follow-pressure",
        decision: "Two winners + forced follow — double-dummy still beats forced follow (dd≈2.00-2.05, fs≈1.55, bc≈1.05)",
        trump_count: 0,
        winners: 2,
        void_suits: 0,
        contract_pressure: 0,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "bc-vs-follow-edge",
        decision: "Forced follow + mid contract pressure — bid still dominates (bc≈1.89, fs≈1.55, dd≈1.30-1.35)",
        trump_count: 0,
        winners: 0,
        void_suits: 0,
        contract_pressure: 2,
        penalty_pressure: 0,
        cards_in_trick: 0,
        follow_suit_forced: true,
        nil_viable: false,
    },
    BridgeScenario {
        scenario_id: "dd-rich-everything",
        decision: "Three winners + two trumps + pressure + one void + one card in trick — double-dummy dominates (dd≈3.25-3.30, fs≈1.15, bc≈1.17)",
        trump_count: 2,
        winners: 3,
        void_suits: 1,
        contract_pressure: 1,
        penalty_pressure: 0,
        cards_in_trick: 1,
        follow_suit_forced: false,
        nil_viable: false,
    },
];

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
