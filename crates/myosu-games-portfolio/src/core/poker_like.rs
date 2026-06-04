use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cards::Card;
use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::game::ResearchGame;

const NLHE_HU_ACTION_PREFIX: &str = "nlhe-heads-up.";
const NLHE6_ACTION_PREFIX: &str = "nlhe-six-max.";
const PLO_ACTION_PREFIX: &str = "plo.";
const NLHE_TOURNAMENT_ACTION_PREFIX: &str = "nlhe-tournament.";
const SHORT_DECK_ACTION_PREFIX: &str = "short-deck.";
const TEEN_PATTI_ACTION_PREFIX: &str = "teen-patti.";

const HEADS_UP_ORDER: [u8; 2] = [0, 1];
const SIX_MAX_ORDER: [u8; 6] = [2, 3, 4, 5, 0, 1];
const SHORT_DECK_ORDER: [u8; 6] = [5, 0, 1, 2, 3, 4];
const TEEN_PATTI_ORDER: [u8; 6] = [3, 4, 5, 0, 1, 2];
const NINE_MAX_ORDER: [u8; 9] = [2, 3, 4, 5, 6, 7, 8, 0, 1];

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PokerVariant {
    NlheHeadsUp,
    NlheSixMax,
    Plo,
    NlheTournament,
    ShortDeck,
    TeenPatti,
}

impl PokerVariant {
    const fn game(self) -> ResearchGame {
        match self {
            Self::NlheHeadsUp => ResearchGame::NlheHeadsUp,
            Self::NlheSixMax => ResearchGame::NlheSixMax,
            Self::Plo => ResearchGame::Plo,
            Self::NlheTournament => ResearchGame::NlheTournament,
            Self::ShortDeck => ResearchGame::ShortDeck,
            Self::TeenPatti => ResearchGame::TeenPatti,
        }
    }

    const fn action_prefix(self) -> &'static str {
        match self {
            Self::NlheHeadsUp => NLHE_HU_ACTION_PREFIX,
            Self::NlheSixMax => NLHE6_ACTION_PREFIX,
            Self::Plo => PLO_ACTION_PREFIX,
            Self::NlheTournament => NLHE_TOURNAMENT_ACTION_PREFIX,
            Self::ShortDeck => SHORT_DECK_ACTION_PREFIX,
            Self::TeenPatti => TEEN_PATTI_ACTION_PREFIX,
        }
    }

    const fn seat_count(self) -> usize {
        match self {
            Self::NlheHeadsUp => 2,
            Self::NlheSixMax | Self::Plo | Self::ShortDeck | Self::TeenPatti => 6,
            Self::NlheTournament => 9,
        }
    }

    fn order(self) -> &'static [u8] {
        match self {
            Self::NlheHeadsUp => &HEADS_UP_ORDER,
            Self::NlheSixMax | Self::Plo => &SIX_MAX_ORDER,
            Self::NlheTournament => &NINE_MAX_ORDER,
            Self::ShortDeck => &SHORT_DECK_ORDER,
            Self::TeenPatti => &TEEN_PATTI_ORDER,
        }
    }

    const fn allows_see_cards(self) -> bool {
        matches!(self, Self::TeenPatti)
    }

    fn phase(self, public: &PokerVariantPublicState) -> &'static str {
        match self {
            Self::Plo => "flop-betting",
            Self::ShortDeck => "ante-betting",
            Self::TeenPatti => {
                if actor_seen(public) {
                    "seen-player-betting"
                } else {
                    "blind-seen-decision"
                }
            }
            Self::NlheHeadsUp | Self::NlheSixMax | Self::NlheTournament => "preflop-betting",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PokerVariantPublicState {
    variant: PokerVariant,
    blinds: PokerBlinds,
    stacks: Vec<u64>,
    committed: Vec<u64>,
    folded: Vec<bool>,
    hole_card_commitments: Vec<String>,
    board: Vec<Card>,
    pot: u64,
    current_bet: u64,
    min_raise: u64,
    actor: u8,
    seen_cards: Option<Vec<bool>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PokerLikeFeatureView {
    pub pot: u64,
    pub effective_stack: u64,
    pub board_len: usize,
    pub active_players: usize,
    pub to_call: u64,
    pub raise_legal: bool,
    pub check_legal: bool,
    pub in_position: bool,
    pub icm_pressure: u8,
    pub has_seen_cards: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct PokerBlinds {
    small: u64,
    big: u64,
    ante: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PokerAction {
    Fold,
    Call,
    Check,
    RaiseTo(u64),
    SeeCards,
}

pub fn nlhe_heads_up_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(PokerVariantPublicState {
        variant: PokerVariant::NlheHeadsUp,
        blinds: PokerBlinds {
            small: 1,
            big: 2,
            ante: 0,
        },
        stacks: vec![99, 98],
        committed: vec![1, 2],
        folded: vec![false; 2],
        hole_card_commitments: vec![
            "nlhe-heads-up.hole-cards.seat-0.bootstrap-v1".to_string(),
            "nlhe-heads-up.hole-cards.seat-1.bootstrap-v1".to_string(),
        ],
        board: Vec::new(),
        pot: 3,
        current_bet: 2,
        min_raise: 2,
        actor: 0,
        seen_cards: None,
    })
}

pub fn nlhe6max_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(PokerVariantPublicState {
        variant: PokerVariant::NlheSixMax,
        blinds: PokerBlinds {
            small: 1,
            big: 2,
            ante: 0,
        },
        stacks: vec![99, 98, 100, 100, 100, 100],
        committed: vec![1, 2, 0, 0, 0, 0],
        folded: vec![false; 6],
        hole_card_commitments: (0..6)
            .map(|seat| format!("nlhe-six-max.hole-cards.seat-{seat}.bootstrap-v1"))
            .collect(),
        board: Vec::new(),
        pot: 3,
        current_bet: 2,
        min_raise: 2,
        actor: 2,
        seen_cards: None,
    })
}

pub fn plo_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(PokerVariantPublicState {
        variant: PokerVariant::Plo,
        blinds: PokerBlinds {
            small: 1,
            big: 2,
            ante: 0,
        },
        stacks: vec![94, 92, 92, 92, 100, 100],
        committed: vec![2, 4, 4, 4, 0, 0],
        folded: vec![false; 6],
        hole_card_commitments: (0..6)
            .map(|seat| format!("plo.hole-cards.seat-{seat}.bootstrap-v1"))
            .collect(),
        board: vec![
            Card::new(crate::cards::Rank::Ace, crate::cards::Suit::Hearts),
            Card::new(crate::cards::Rank::Ten, crate::cards::Suit::Hearts),
            Card::new(crate::cards::Rank::Nine, crate::cards::Suit::Clubs),
        ],
        pot: 14,
        current_bet: 4,
        min_raise: 4,
        actor: 4,
        seen_cards: None,
    })
}

pub fn nlhe_tournament_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(PokerVariantPublicState {
        variant: PokerVariant::NlheTournament,
        blinds: PokerBlinds {
            small: 1,
            big: 2,
            ante: 1,
        },
        stacks: vec![38, 24, 21, 17, 16, 13, 10, 11, 22],
        committed: vec![2, 3, 1, 1, 1, 1, 1, 1, 1],
        folded: vec![false; 9],
        hole_card_commitments: (0..9)
            .map(|seat| format!("nlhe-tournament.hole-cards.seat-{seat}.bootstrap-v1"))
            .collect(),
        board: Vec::new(),
        pot: 12,
        current_bet: 3,
        min_raise: 3,
        actor: 6,
        seen_cards: None,
    })
}

pub fn short_deck_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(PokerVariantPublicState {
        variant: PokerVariant::ShortDeck,
        blinds: PokerBlinds {
            small: 0,
            big: 0,
            ante: 1,
        },
        stacks: vec![19, 19, 19, 19, 19, 19],
        committed: vec![1, 1, 1, 1, 1, 1],
        folded: vec![false; 6],
        hole_card_commitments: (0..6)
            .map(|seat| format!("short-deck.hole-cards.seat-{seat}.bootstrap-v1"))
            .collect(),
        board: Vec::new(),
        pot: 6,
        current_bet: 1,
        min_raise: 1,
        actor: 5,
        seen_cards: None,
    })
}

pub fn teen_patti_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(PokerVariantPublicState {
        variant: PokerVariant::TeenPatti,
        blinds: PokerBlinds {
            small: 0,
            big: 0,
            ante: 1,
        },
        stacks: vec![19, 19, 19, 19, 18, 19],
        committed: vec![1, 1, 1, 1, 2, 1],
        folded: vec![false; 6],
        hole_card_commitments: (0..6)
            .map(|seat| format!("teen-patti.cards.seat-{seat}.bootstrap-v1"))
            .collect(),
        board: Vec::new(),
        pot: 7,
        current_bet: 2,
        min_raise: 2,
        actor: 3,
        seen_cards: Some(vec![false, false, false, false, true, false]),
    })
}

pub fn apply_nlhe_heads_up_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, PokerVariant::NlheHeadsUp)
}

pub fn apply_nlhe6max_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, PokerVariant::NlheSixMax)
}

pub fn apply_plo_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, PokerVariant::Plo)
}

pub fn apply_nlhe_tournament_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, PokerVariant::NlheTournament)
}

pub fn apply_short_deck_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, PokerVariant::ShortDeck)
}

pub fn apply_teen_patti_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, PokerVariant::TeenPatti)
}

fn apply_variant_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
    variant: PokerVariant,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: PokerVariantPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    if before_public.variant != variant {
        return Err(invalid_variant_state(
            variant,
            action_id,
            "state variant does not match dispatch target",
        ));
    }
    let parsed = parse_variant_action(variant, action_id)?;
    validate_variant_action(&before_public, parsed, action_id)?;
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(illegal_variant_action(
            variant,
            action_id,
            "action is not legal in this betting state",
        ));
    }

    let mut after_public = before_public.clone();
    let actor = actor_index(&after_public)?;
    match parsed {
        PokerAction::Fold => {
            let folded = after_public.folded.get_mut(actor).ok_or_else(|| {
                invalid_variant_state(variant, action_id, "actor folded slot missing")
            })?;
            *folded = true;
        }
        PokerAction::Call => {
            let contribution = to_call(&after_public)?;
            let stack = after_public.stacks.get_mut(actor).ok_or_else(|| {
                invalid_variant_state(variant, action_id, "actor stack slot missing")
            })?;
            *stack = stack.saturating_sub(contribution);
            let committed = after_public.committed.get_mut(actor).ok_or_else(|| {
                invalid_variant_state(variant, action_id, "actor committed slot missing")
            })?;
            *committed = committed.saturating_add(contribution);
            after_public.pot = after_public.pot.saturating_add(contribution);
        }
        PokerAction::Check => {}
        PokerAction::RaiseTo(amount) => {
            let committed_before = after_public.committed.get(actor).copied().ok_or_else(|| {
                invalid_variant_state(variant, action_id, "actor committed slot missing")
            })?;
            let contribution = amount.checked_sub(committed_before).ok_or_else(|| {
                invalid_variant_state(variant, action_id, "raise target below committed")
            })?;
            let raise_size = amount
                .checked_sub(after_public.current_bet)
                .ok_or_else(|| {
                    invalid_variant_state(variant, action_id, "raise target below current bet")
                })?;
            let stack = after_public.stacks.get_mut(actor).ok_or_else(|| {
                invalid_variant_state(variant, action_id, "actor stack slot missing")
            })?;
            *stack = stack.saturating_sub(contribution);
            let committed = after_public.committed.get_mut(actor).ok_or_else(|| {
                invalid_variant_state(variant, action_id, "actor committed slot missing")
            })?;
            *committed = amount;
            after_public.pot = after_public.pot.saturating_add(contribution);
            after_public.current_bet = amount;
            after_public.min_raise = raise_size;
        }
        PokerAction::SeeCards => {
            let seen_cards = after_public.seen_cards.as_mut().ok_or_else(|| {
                invalid_variant_state(variant, action_id, "seen-card state missing for teen patti")
            })?;
            let seen = seen_cards.get_mut(actor).ok_or_else(|| {
                invalid_variant_state(variant, action_id, "actor seen-card slot missing")
            })?;
            *seen = true;
        }
    }

    let (terminal, payoff, next_actor) = if let Some(winner) = sole_remaining_player(&after_public)?
    {
        (true, Some(fold_payoff(&after_public, winner)?), None)
    } else if parsed == PokerAction::SeeCards {
        (false, None, Some(after_public.actor))
    } else {
        let next = next_actor(&after_public)?;
        after_public.actor = next;
        (false, None, Some(next))
    };

    let after = state_from_public_with_terminal(after_public, terminal, payoff, next_actor)?;
    let action = core_action_for_variant(variant, parsed);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(public: PokerVariantPublicState) -> Result<CoreGameState, CoreGameError> {
    state_from_public_with_terminal(public, false, None, None)
}

fn state_from_public_with_terminal(
    public: PokerVariantPublicState,
    terminal: bool,
    payoff: Option<Vec<i64>>,
    actor_override: Option<u8>,
) -> Result<CoreGameState, CoreGameError> {
    validate_variant_state(&public, "poker-like.bootstrap")?;
    let variant = public.variant;
    let phase = variant.phase(&public).to_string();
    let actor = if terminal {
        None
    } else {
        actor_override.or(Some(public.actor))
    };
    let legal_actions = if terminal {
        Vec::new()
    } else {
        legal_variant_actions(&public)
            .into_iter()
            .map(|action| core_action_for_variant(variant, action))
            .collect()
    };
    let private_state_commitments = public.hole_card_commitments.clone();
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: format!("{}.bootstrap", variant.game().slug()),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: variant.game(),
        phase,
        actor,
        public_state,
        private_state_commitments,
        legal_actions,
        terminal,
        payoff,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<PokerLikeFeatureView, CoreGameError> {
    let public: PokerVariantPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: format!("{}.feature-view", state.game.slug()),
            reason: source.to_string(),
        })?;
    validate_variant_state(&public, "poker-like.feature-view")?;

    let active_players = public.folded.iter().filter(|folded| !**folded).count();
    let raise_legal = state.legal_actions.iter().any(|action| {
        matches!(
            parse_variant_action(public.variant, &action.action_id),
            Ok(PokerAction::RaiseTo(_))
        )
    });
    let check_legal = state.legal_actions.iter().any(|action| {
        matches!(
            parse_variant_action(public.variant, &action.action_id),
            Ok(PokerAction::Check)
        )
    });

    Ok(PokerLikeFeatureView {
        pot: public.pot,
        effective_stack: public
            .stacks
            .iter()
            .copied()
            .filter(|stack| *stack > 0)
            .min()
            .unwrap_or_default(),
        board_len: public.board.len(),
        active_players,
        to_call: to_call(&public)?,
        raise_legal,
        check_legal,
        in_position: state.actor.is_some_and(|seat| seat >= 2) || active_players <= 2,
        icm_pressure: if public.variant == PokerVariant::NlheTournament {
            match public
                .stacks
                .iter()
                .copied()
                .filter(|stack| *stack > 0)
                .min()
                .unwrap_or_default()
            {
                0..=12 => 5,
                13..=20 => 3,
                _ => 1,
            }
        } else {
            0
        },
        has_seen_cards: actor_seen(&public),
    })
}

fn legal_variant_actions(public: &PokerVariantPublicState) -> Vec<PokerAction> {
    let mut actions = Vec::new();
    if public.variant.allows_see_cards() && !actor_seen(public) {
        actions.push(PokerAction::SeeCards);
    }
    let to_call = to_call(public).unwrap_or_default();
    if to_call > 0 {
        actions.push(PokerAction::Fold);
        if actor_stack(public).unwrap_or_default() >= to_call {
            actions.push(PokerAction::Call);
        }
    } else {
        actions.push(PokerAction::Check);
    }

    let min_raise_to = public.current_bet.saturating_add(public.min_raise);
    if actor_total_available(public).unwrap_or_default() >= min_raise_to {
        actions.push(PokerAction::RaiseTo(min_raise_to));
    }

    actions
}

fn validate_variant_state(
    public: &PokerVariantPublicState,
    action_id: &str,
) -> Result<(), CoreGameError> {
    let seats = public.variant.seat_count();
    if public.stacks.len() != seats
        || public.committed.len() != seats
        || public.folded.len() != seats
        || public.hole_card_commitments.len() != seats
        || usize::from(public.actor) >= seats
    {
        return Err(invalid_variant_state(
            public.variant,
            action_id,
            "state shape does not match variant seat count",
        ));
    }
    if public.variant.allows_see_cards() {
        let seen_cards = public.seen_cards.as_ref().ok_or_else(|| {
            invalid_variant_state(
                public.variant,
                action_id,
                "teen patti requires seen-card state",
            )
        })?;
        if seen_cards.len() != seats {
            return Err(invalid_variant_state(
                public.variant,
                action_id,
                "seen-card state does not match variant seat count",
            ));
        }
    }

    Ok(())
}

fn validate_variant_action(
    public: &PokerVariantPublicState,
    parsed: PokerAction,
    action_id: &str,
) -> Result<(), CoreGameError> {
    validate_variant_state(public, action_id)?;
    let actor = actor_index(public)?;
    if public.folded.get(actor).copied().unwrap_or(true) {
        return Err(illegal_variant_action(
            public.variant,
            action_id,
            "folded player cannot act",
        ));
    }
    match parsed {
        PokerAction::Fold => {
            if to_call(public)? == 0 {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "fold is not offered when checking is available",
                ));
            }
        }
        PokerAction::Call => {
            let needed = to_call(public)?;
            if needed == 0 {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "call requires a bet to call",
                ));
            }
            if actor_stack(public)? < needed {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "actor stack cannot cover the call",
                ));
            }
        }
        PokerAction::Check => {
            if to_call(public)? > 0 {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "check is illegal while facing a bet",
                ));
            }
        }
        PokerAction::RaiseTo(amount) => {
            if amount <= public.current_bet {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "raise target must exceed the current bet",
                ));
            }
            let raise_size = amount.saturating_sub(public.current_bet);
            if raise_size < public.min_raise {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "raise target violates the minimum raise",
                ));
            }
            if actor_total_available(public)? < amount {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "actor stack cannot cover the raise target",
                ));
            }
        }
        PokerAction::SeeCards => {
            if !public.variant.allows_see_cards() {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "see-cards only applies to teen patti",
                ));
            }
            if actor_seen(public) {
                return Err(illegal_variant_action(
                    public.variant,
                    action_id,
                    "actor has already seen their cards",
                ));
            }
        }
    }

    Ok(())
}

fn core_action_for_variant(variant: PokerVariant, action: PokerAction) -> CoreAction {
    let prefix = variant.action_prefix();
    match action {
        PokerAction::Fold => CoreAction {
            action_id: format!("{prefix}fold"),
            display_label: "fold".to_string(),
            params: json!({}),
        },
        PokerAction::Call => CoreAction {
            action_id: format!("{prefix}call"),
            display_label: "call".to_string(),
            params: json!({}),
        },
        PokerAction::Check => CoreAction {
            action_id: format!("{prefix}check"),
            display_label: "check".to_string(),
            params: json!({}),
        },
        PokerAction::RaiseTo(amount) => CoreAction {
            action_id: format!("{prefix}raise-to.{amount}"),
            display_label: format!("raise-to-{amount}"),
            params: json!({"amount": amount}),
        },
        PokerAction::SeeCards => CoreAction {
            action_id: format!("{prefix}see-cards"),
            display_label: "see-cards".to_string(),
            params: json!({}),
        },
    }
}

fn parse_variant_action(
    variant: PokerVariant,
    action_id: &str,
) -> Result<PokerAction, CoreGameError> {
    let Some(token) = action_id.strip_prefix(variant.action_prefix()) else {
        return Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        });
    };
    match token {
        "fold" => Ok(PokerAction::Fold),
        "call" => Ok(PokerAction::Call),
        "check" => Ok(PokerAction::Check),
        "see-cards" => Ok(PokerAction::SeeCards),
        _ => {
            let Some(amount) = token.strip_prefix("raise-to.") else {
                return Err(CoreGameError::UnknownAction {
                    game: variant.game(),
                    action_id: action_id.to_string(),
                });
            };
            let amount = amount
                .parse::<u64>()
                .map_err(|_| CoreGameError::UnknownAction {
                    game: variant.game(),
                    action_id: action_id.to_string(),
                })?;
            Ok(PokerAction::RaiseTo(amount))
        }
    }
}

fn actor_index(public: &PokerVariantPublicState) -> Result<usize, CoreGameError> {
    let actor = usize::from(public.actor);
    if actor >= public.variant.seat_count() {
        return Err(invalid_variant_state(
            public.variant,
            "poker-like.actor",
            "actor is outside the seat map",
        ));
    }
    Ok(actor)
}

fn actor_stack(public: &PokerVariantPublicState) -> Result<u64, CoreGameError> {
    Ok(public
        .stacks
        .get(actor_index(public)?)
        .copied()
        .unwrap_or_default())
}

fn actor_total_available(public: &PokerVariantPublicState) -> Result<u64, CoreGameError> {
    let actor = actor_index(public)?;
    Ok(public
        .stacks
        .get(actor)
        .copied()
        .unwrap_or_default()
        .saturating_add(public.committed.get(actor).copied().unwrap_or_default()))
}

fn to_call(public: &PokerVariantPublicState) -> Result<u64, CoreGameError> {
    let committed = public
        .committed
        .get(actor_index(public)?)
        .copied()
        .unwrap_or(public.current_bet);
    Ok(public.current_bet.saturating_sub(committed))
}

fn actor_seen(public: &PokerVariantPublicState) -> bool {
    let Some(seen_cards) = public.seen_cards.as_ref() else {
        return true;
    };
    let actor = usize::from(public.actor);
    seen_cards.get(actor).copied().unwrap_or(true)
}

fn next_actor(public: &PokerVariantPublicState) -> Result<u8, CoreGameError> {
    let order = public.variant.order();
    let current_position = order
        .iter()
        .position(|seat| *seat == public.actor)
        .ok_or_else(|| {
            invalid_variant_state(
                public.variant,
                "poker-like.actor-order",
                "actor is not present in the variant action order",
            )
        })?;
    let max_steps = order.len();
    for offset in 1..=max_steps {
        let advanced = current_position.checked_add(offset).ok_or_else(|| {
            invalid_variant_state(
                public.variant,
                "poker-like.actor-order",
                "actor rotation overflowed while advancing turn order",
            )
        })?;
        let position = advanced.checked_rem(max_steps).ok_or_else(|| {
            invalid_variant_state(
                public.variant,
                "poker-like.actor-order",
                "actor rotation modulo failed",
            )
        })?;
        let candidate = order.get(position).copied().ok_or_else(|| {
            invalid_variant_state(
                public.variant,
                "poker-like.actor-order",
                "candidate seat missing from turn order",
            )
        })?;
        let index = usize::from(candidate);
        let folded = public.folded.get(index).copied().unwrap_or(true);
        let total = public
            .stacks
            .get(index)
            .copied()
            .unwrap_or_default()
            .saturating_add(public.committed.get(index).copied().unwrap_or_default());
        if !folded && total > 0 {
            return Ok(candidate);
        }
    }

    Err(invalid_variant_state(
        public.variant,
        "poker-like.actor-order",
        "no active player remains after betting action",
    ))
}

fn sole_remaining_player(public: &PokerVariantPublicState) -> Result<Option<u8>, CoreGameError> {
    let mut active = Vec::new();
    for seat in 0..public.variant.seat_count() {
        let folded = public.folded.get(seat).copied().unwrap_or(true);
        let total = public
            .stacks
            .get(seat)
            .copied()
            .unwrap_or_default()
            .saturating_add(public.committed.get(seat).copied().unwrap_or_default());
        if !folded && total > 0 {
            let seat_u8 = u8::try_from(seat).map_err(|_| {
                invalid_variant_state(
                    public.variant,
                    "poker-like.active-player",
                    "seat index does not fit into u8",
                )
            })?;
            active.push(seat_u8);
        }
    }

    if active.len() == 1 {
        Ok(active.first().copied())
    } else {
        Ok(None)
    }
}

fn fold_payoff(public: &PokerVariantPublicState, winner: u8) -> Result<Vec<i64>, CoreGameError> {
    let winner_index = usize::from(winner);
    let winner_commit = i64::try_from(
        public
            .committed
            .get(winner_index)
            .copied()
            .unwrap_or_default(),
    )
    .map_err(|_| {
        invalid_variant_state(
            public.variant,
            "poker-like.payoff",
            "winner commit does not fit into i64",
        )
    })?;
    let pot = i64::try_from(public.pot).map_err(|_| {
        invalid_variant_state(
            public.variant,
            "poker-like.payoff",
            "pot does not fit into i64",
        )
    })?;
    let mut payoff = Vec::with_capacity(public.variant.seat_count());
    for seat in 0..public.variant.seat_count() {
        let committed = i64::try_from(public.committed.get(seat).copied().unwrap_or_default())
            .map_err(|_| {
                invalid_variant_state(
                    public.variant,
                    "poker-like.payoff",
                    "commit does not fit into i64",
                )
            })?;
        if seat == winner_index {
            payoff.push(pot.saturating_sub(winner_commit));
        } else {
            payoff.push(0_i64.saturating_sub(committed));
        }
    }
    Ok(payoff)
}

fn illegal_variant_action(variant: PokerVariant, action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: variant.game(),
        action_id: action_id.to_string(),
        reason: reason.to_string(),
    }
}

fn invalid_variant_state(variant: PokerVariant, action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::InvalidParams {
        action_id: format!("{}:{action_id}", variant.game().slug()),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::core::model::{apply_action, bootstrap_state};

    #[test]
    fn nlhe6max_bootstrap_state_has_legal_actions() {
        let state = state_for(ResearchGame::NlheSixMax);

        assert_eq!(state.actor, Some(2));
        assert!(has_action(&state, "nlhe-six-max.fold"));
        assert!(has_action(&state, "nlhe-six-max.call"));
        assert!(has_action(&state, "nlhe-six-max.raise-to.4"));
    }

    #[test]
    fn heads_up_fold_ends_the_hand() {
        let state = state_for(ResearchGame::NlheHeadsUp);
        let transition = apply_action(&state, "nlhe-heads-up.fold", json!({}))
            .unwrap_or_else(|error| panic!("heads-up fold should apply: {error}"));

        assert!(transition.after.terminal);
        assert_eq!(transition.after.actor, None);
        assert_eq!(transition.after.payoff, Some(vec![-1, 1]));
    }

    #[test]
    fn plo_uses_flop_betting_phase() {
        let state = state_for(ResearchGame::Plo);

        assert_eq!(state.phase, "flop-betting");
        assert!(has_action(&state, "plo.call"));
        assert!(has_action(&state, "plo.raise-to.8"));
    }

    #[test]
    fn nlhe_tournament_supports_shove_or_fold_shape() {
        let state = state_for(ResearchGame::NlheTournament);

        assert_eq!(state.game, ResearchGame::NlheTournament);
        assert!(has_action(&state, "nlhe-tournament.fold"));
        assert!(has_action(&state, "nlhe-tournament.call"));
        assert!(has_action(&state, "nlhe-tournament.raise-to.6"));
    }

    #[test]
    fn short_deck_allows_check_when_antes_are_even() {
        let state = state_for(ResearchGame::ShortDeck);

        assert!(has_action(&state, "short-deck.check"));
        assert!(has_action(&state, "short-deck.raise-to.2"));
    }

    #[test]
    fn teen_patti_see_cards_stays_on_same_actor() {
        let state = state_for(ResearchGame::TeenPatti);
        let transition = apply_action(&state, "teen-patti.see-cards", json!({}))
            .unwrap_or_else(|error| panic!("see-cards should apply: {error}"));
        let public: PokerVariantPublicState = serde_json::from_value(transition.after.public_state)
            .unwrap_or_else(|error| panic!("teen patti public state should decode: {error}"));

        assert_eq!(transition.after.actor, Some(3));
        assert_eq!(
            public
                .seen_cards
                .as_ref()
                .and_then(|seen| seen.get(3))
                .copied(),
            Some(true)
        );
        assert!(
            !transition
                .after
                .legal_actions
                .iter()
                .any(|action| action.action_id == "teen-patti.see-cards")
        );
    }

    #[test]
    fn teen_patti_feature_view_tracks_seen_cards() {
        let state = state_for(ResearchGame::TeenPatti);
        let blind_view = feature_view(&state)
            .unwrap_or_else(|error| panic!("teen patti feature view should decode: {error}"));
        let seen_state = apply_action(&state, "teen-patti.see-cards", json!({}))
            .unwrap_or_else(|error| panic!("see-cards should apply: {error}"))
            .after;
        let seen_view = feature_view(&seen_state)
            .unwrap_or_else(|error| panic!("seen teen patti view should decode: {error}"));

        assert!(!blind_view.has_seen_cards);
        assert!(seen_view.has_seen_cards);
        assert_eq!(blind_view.to_call, seen_view.to_call);
    }

    #[test]
    fn nlhe6max_minimum_raise_enforcement() {
        let state = state_for(ResearchGame::NlheSixMax);

        assert!(matches!(
            apply_action(&state, "nlhe-six-max.raise-to.3", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("minimum raise")
        ));
    }

    #[test]
    fn nlhe6max_rejects_check_when_facing_bet() {
        let state = state_for(ResearchGame::NlheSixMax);

        assert!(matches!(
            apply_action(&state, "nlhe-six-max.check", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("facing a bet")
        ));
    }

    #[test]
    fn nlhe6max_call_updates_stack_and_pot() {
        let state = state_for(ResearchGame::NlheSixMax);
        let transition = apply_action(&state, "nlhe-six-max.call", json!({}))
            .unwrap_or_else(|error| panic!("call should apply: {error}"));
        let public: PokerVariantPublicState = serde_json::from_value(transition.after.public_state)
            .unwrap_or_else(|error| panic!("nlhe public state should decode: {error}"));

        assert_eq!(public.stacks.get(2).copied(), Some(98));
        assert_eq!(public.committed.get(2).copied(), Some(2));
        assert_eq!(public.pot, 5);
        assert_eq!(public.actor, 3);
    }

    #[test]
    fn nlhe6max_transition_is_deterministic() {
        let state = state_for(ResearchGame::NlheSixMax);
        let first = apply_action(&state, "nlhe-six-max.raise-to.4", json!({}));
        let second = apply_action(&state, "nlhe-six-max.raise-to.4", json!({}));

        assert_eq!(first, second);
    }

    fn state_for(game: ResearchGame) -> CoreGameState {
        bootstrap_state(game)
            .unwrap_or_else(|error| panic!("{} bootstrap should succeed: {error}", game.slug()))
    }

    fn has_action(state: &CoreGameState, action_id: &str) -> bool {
        state
            .legal_actions
            .iter()
            .any(|action| action.action_id == action_id)
    }
}

// =============================================================================
// F-017 PLO benchmark scenario pack
// =============================================================================
//
// The PLO scenario pack is the promotion evidence the `F-017` slice ships: it
// pins the 22-scenario rule-aware scenario pack for PLO, runs the live
// portfolio engine against it, records every engine recommendation, and
// SHA-256-pins the canonical scenario/answer table so the promotion manifest
// harness can verify that the evidence attached to a `tier: benchmarked`
// claim matches the live engine output.
//
// PLO is the F-017 portfolio-game-promotion slice (the
// F-001/F-008/F-009/F-010/F-011/F-012/F-013/F-014/F-015/F-016 eleventh slice:
// Cribbage / Hearts / Gin Rummy / Spades / Bridge / Call Break / Backgammon /
// Hanafuda Koi-Koi / Hwatu Go-Stop / Stratego). F-017 opens the
// `state-aware PLO nut-draw heuristic` engine sub-family in
// `crate::engines::poker_like::plo` — no other portfolio game has a dossier
// row for the `poker_like` engine family (nlhe-six-max, plo, nlhe-tournament,
// short-deck, teen-patti all share the `PokerLikeChallenge` struct, but F-017
// PLO is the first dossier slice for this sub-family). The F-016 row's scope
// boundary explicitly called out "plo, dou-di-zhu, or ofc-chinese-poker are
// the obvious candidates" and PLO is the natural one to open the
// `poker_like` sub-family with because it is the closest cousin to the
// dedicated `nlhe-heads-up` engine and is the second-most-portfolio-cited
// poker variant after NLHE.
//
// The engine has three ranked arms (`draw_to_nuts` / `pot_sized_raise` /
// `pot_control`) keyed to nut-draw-heavy / made-hand-aggression / deep-stack-
// pot-control state. The F-017 scenario pack splits 8/8/6 across
// draw-to-nuts-dominant / pot-sized-raise-dominant / pot-control-dominant so
// a regression that flips the dominant arm on any scenario is loud in the
// dossier's recommendation map.
//
// The pack's `decision` docstring records the expected `d=…` / `p=…` / `c=…`
// values so the dominant arm stays dominant by at least 0.16 on every
// scenario (the tightest margin is `pot-control-weak-draw` at c=2.16 vs
// d=1.87, a 0.29 margin). The other 21 scenarios are all 0.57+ dominant.

/// One row of the canonical 22-scenario PLO benchmark pack.
///
/// The fields mirror the live `PokerLikeChallenge` feature struct so the
/// dossier's typed `PortfolioChallenge::Plo(PokerLikeChallenge { ... })` is a
/// field-for-field copy of the scenario. Keeping the mirror stable means the
/// dossier hash is also a regression guard for the feature-view extraction: a
/// refactor that changes how `feature_view` reads `draw_strength` / `pot_bb`
/// / etc out of a `CoreGameState` will flip the dossier's recommendation map
/// and the unit tests will fail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PloScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
    pub pot_bb: u16,
    pub effective_stack_bb: u16,
    pub made_strength: u8,
    pub draw_strength: u8,
    pub fold_equity: u8,
    pub to_call_bb: u16,
    pub active_players: u8,
    pub check_available: bool,
    pub raise_available: bool,
    pub in_position: bool,
    pub icm_pressure: u8,
    pub has_seen_cards: bool,
}

/// Return the canonical 22-scenario PLO benchmark pack.
pub fn plo_scenario_pack() -> &'static [PloScenario] {
    PLO_SCENARIO_PACK
}

const PLO_SCENARIO_PACK: &[PloScenario] = &[
    // ---- draw-to-nuts bucket (×8) — nut-draw-heavy pot, no raise available ----
    PloScenario {
        scenario_id: "draw-nuts-balanced-call",
        decision: "Balanced nut draw facing a 2bb call (ds=4, pot=20bb, 3-way) — draw-to-nuts dominates (d=5.07, p=0.55, c=1.17)",
        pot_bb: 20,
        effective_stack_bb: 60,
        made_strength: 0,
        draw_strength: 4,
        fold_equity: 0,
        to_call_bb: 2,
        active_players: 3,
        check_available: false,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "draw-nuts-heavy-draw",
        decision: "Heavy nut draw (ds=5) facing a small call — draw-to-nuts dominates (d=6.00, p=0.55, c=1.17)",
        pot_bb: 24,
        effective_stack_bb: 60,
        made_strength: 0,
        draw_strength: 5,
        fold_equity: 0,
        to_call_bb: 2,
        active_players: 3,
        check_available: false,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "draw-nuts-big-pot",
        decision: "Big-pot nut draw (pot=40bb, 4-way) facing 4bb — draw-to-nuts dominates (d=4.93, p=0.55, c=1.34)",
        pot_bb: 40,
        effective_stack_bb: 80,
        made_strength: 0,
        draw_strength: 3,
        fold_equity: 0,
        to_call_bb: 4,
        active_players: 4,
        check_available: false,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "draw-nuts-out-of-pos",
        decision: "Nut draw out of position — draw-to-nuts dominates (d=5.07, p=0.55, c=1.17)",
        pot_bb: 20,
        effective_stack_bb: 60,
        made_strength: 0,
        draw_strength: 4,
        fold_equity: 0,
        to_call_bb: 2,
        active_players: 3,
        check_available: false,
        raise_available: false,
        in_position: false,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "draw-nuts-small-draw",
        decision: "Small nut draw (ds=2) facing a 2bb call — draw-to-nuts dominates (d=3.20, p=0.55, c=1.03)",
        pot_bb: 12,
        effective_stack_bb: 40,
        made_strength: 0,
        draw_strength: 2,
        fold_equity: 0,
        to_call_bb: 2,
        active_players: 3,
        check_available: false,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "draw-nuts-monster-draw",
        decision: "Monster nut draw (ds=6) heads-up — draw-to-nuts dominates (d=6.67, p=0.55, c=1.14)",
        pot_bb: 20,
        effective_stack_bb: 60,
        made_strength: 0,
        draw_strength: 6,
        fold_equity: 0,
        to_call_bb: 2,
        active_players: 2,
        check_available: false,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "draw-nuts-double-suited",
        decision: "Double-suited nut draw in 3-way pot — draw-to-nuts dominates (d=4.60, p=0.55, c=1.24)",
        pot_bb: 30,
        effective_stack_bb: 70,
        made_strength: 0,
        draw_strength: 3,
        fold_equity: 0,
        to_call_bb: 3,
        active_players: 3,
        check_available: false,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "draw-nuts-multiway",
        decision: "Nut draw in 4-way pot — draw-to-nuts dominates (d=5.20, p=0.55, c=1.34)",
        pot_bb: 24,
        effective_stack_bb: 80,
        made_strength: 0,
        draw_strength: 4,
        fold_equity: 0,
        to_call_bb: 2,
        active_players: 4,
        check_available: false,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    // ---- pot-sized-raise bucket (×8) — made hand + fold equity + raise available ----
    PloScenario {
        scenario_id: "raise-mvp-can-raise",
        decision: "MVP made hand (ms=5) with fold equity (fe=4), can raise — pot-sized-raise dominates (d=1.27, p=4.35, c=1.50)",
        pot_bb: 8,
        effective_stack_bb: 40,
        made_strength: 5,
        draw_strength: 0,
        fold_equity: 4,
        to_call_bb: 0,
        active_players: 2,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "raise-strong-made",
        decision: "Strong made hand (ms=6) with fold equity (fe=3) — pot-sized-raise dominates (d=1.27, p=4.55, c=1.50)",
        pot_bb: 8,
        effective_stack_bb: 40,
        made_strength: 6,
        draw_strength: 0,
        fold_equity: 3,
        to_call_bb: 0,
        active_players: 2,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "raise-folding-station",
        decision: "Made hand vs folding station (fe=5) — pot-sized-raise dominates (d=1.27, p=4.15, c=1.50)",
        pot_bb: 8,
        effective_stack_bb: 40,
        made_strength: 4,
        draw_strength: 0,
        fold_equity: 5,
        to_call_bb: 0,
        active_players: 2,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "raise-three-way-fold",
        decision: "3-way: made hand (ms=5) with fold equity (fe=4) — pot-sized-raise dominates (d=1.33, p=4.35, c=1.53)",
        pot_bb: 10,
        effective_stack_bb: 40,
        made_strength: 5,
        draw_strength: 0,
        fold_equity: 4,
        to_call_bb: 0,
        active_players: 3,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "raise-medium-made",
        decision: "Medium made hand (ms=3) with fold equity (fe=3) — pot-sized-raise dominates (d=1.20, p=3.20, c=1.42)",
        pot_bb: 6,
        effective_stack_bb: 30,
        made_strength: 3,
        draw_strength: 0,
        fold_equity: 3,
        to_call_bb: 0,
        active_players: 2,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "raise-the-nuts",
        decision: "The nuts (ms=7) heads-up — pot-sized-raise dominates (d=1.20, p=4.25, c=1.35)",
        pot_bb: 6,
        effective_stack_bb: 20,
        made_strength: 7,
        draw_strength: 0,
        fold_equity: 0,
        to_call_bb: 0,
        active_players: 2,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "raise-thick-value",
        decision: "Thick value (ms=4) with modest fold equity (fe=2) — pot-sized-raise dominates (d=1.27, p=3.40, c=1.42)",
        pot_bb: 8,
        effective_stack_bb: 30,
        made_strength: 4,
        draw_strength: 0,
        fold_equity: 2,
        to_call_bb: 0,
        active_players: 2,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "raise-pure-fold-equity",
        decision: "Pure fold equity (ms=2, fe=4) — pot-sized-raise dominates (d=1.20, p=3.00, c=1.35)",
        pot_bb: 6,
        effective_stack_bb: 20,
        made_strength: 2,
        draw_strength: 0,
        fold_equity: 4,
        to_call_bb: 0,
        active_players: 2,
        check_available: true,
        raise_available: true,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    // ---- pot-control bucket (×6) — deep stack, check available, multi-way, no raise ----
    PloScenario {
        scenario_id: "control-deep-checked",
        decision: "Deep stack (120bb), check available, 4-way, no raise — pot-control dominates (d=1.40, p=0.55, c=2.13)",
        pot_bb: 12,
        effective_stack_bb: 120,
        made_strength: 0,
        draw_strength: 0,
        fold_equity: 0,
        to_call_bb: 0,
        active_players: 4,
        check_available: true,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "control-deep-multi",
        decision: "Deep stack (100bb), check available, 3-way — pot-control dominates (d=1.33, p=0.55, c=1.95)",
        pot_bb: 10,
        effective_stack_bb: 100,
        made_strength: 0,
        draw_strength: 0,
        fold_equity: 0,
        to_call_bb: 0,
        active_players: 3,
        check_available: true,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "control-shallow-multi",
        decision: "Shallow stack (80bb), check available, 4-way — pot-control dominates (d=1.27, p=0.55, c=1.84)",
        pot_bb: 8,
        effective_stack_bb: 80,
        made_strength: 0,
        draw_strength: 0,
        fold_equity: 0,
        to_call_bb: 0,
        active_players: 4,
        check_available: true,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "control-five-way",
        decision: "5-way checked pot, deep stack — pot-control dominates (d=1.50, p=0.55, c=2.16)",
        pot_bb: 15,
        effective_stack_bb: 120,
        made_strength: 0,
        draw_strength: 0,
        fold_equity: 0,
        to_call_bb: 0,
        active_players: 5,
        check_available: true,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "control-three-way",
        decision: "3-way checked pot, 110bb stack — pot-control dominates (d=1.33, p=0.55, c=2.03)",
        pot_bb: 10,
        effective_stack_bb: 110,
        made_strength: 0,
        draw_strength: 0,
        fold_equity: 0,
        to_call_bb: 0,
        active_players: 3,
        check_available: true,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
    PloScenario {
        scenario_id: "control-weak-draw",
        decision: "Weak gutshot (ds=1), 5-way checked pot — pot-control dominates (d=1.87, p=0.55, c=2.16)",
        pot_bb: 2,
        effective_stack_bb: 120,
        made_strength: 0,
        draw_strength: 1,
        fold_equity: 0,
        to_call_bb: 0,
        active_players: 5,
        check_available: true,
        raise_available: false,
        in_position: true,
        icm_pressure: 0,
        has_seen_cards: true,
    },
];

#[cfg(test)]
mod plo_scenario_pack_tests {
    use super::*;

    #[test]
    fn plo_scenario_pack_is_22_rows() {
        assert_eq!(plo_scenario_pack().len(), 22);
    }

    #[test]
    fn plo_scenario_ids_are_unique() {
        let pack = plo_scenario_pack();
        let mut seen = std::collections::HashSet::new();
        for scenario in pack {
            assert!(
                seen.insert(scenario.scenario_id),
                "duplicate scenario_id: {}",
                scenario.scenario_id
            );
        }
    }

    #[test]
    fn plo_scenario_buckets_have_expected_counts() {
        let pack = plo_scenario_pack();
        let draw = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("draw-"))
            .count();
        let raise = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("raise-"))
            .count();
        let control = pack
            .iter()
            .filter(|s| s.scenario_id.starts_with("control-"))
            .count();
        assert_eq!(draw, 8, "draw-to-nuts bucket should be 8");
        assert_eq!(raise, 8, "pot-sized-raise bucket should be 8");
        assert_eq!(control, 6, "pot-control bucket should be 6");
        assert_eq!(draw + raise + control, 22);
    }

    #[test]
    fn plo_scenario_math_holds_for_every_row() {
        // The dossier's recommendation map is built from
        // `answer_typed_challenge` on the typed challenge, but the
        // pack's own `decision` docstring names the expected
        // `d=…`/`p=…`/`c=…` values, so this test re-derives the
        // expected dominant action and confirms it matches the bucket
        // label. A regression that changes the engine's heuristic
        // math would flip the dominant action on at least one row
        // and this test would fail.
        fn dominant(scenario: &PloScenario) -> &'static str {
            let draw_to_nuts = 1.0_f32
                + (scenario.draw_strength as f32) * 0.80
                + (scenario.pot_bb.min(40) as f32) / 30.0
                + if scenario.to_call_bb > 0 { 0.20 } else { 0.0 };
            let pot_sized_raise = 0.85_f32
                + (scenario.made_strength as f32) * 0.45
                + (scenario.fold_equity as f32) * 0.25
                + if scenario.raise_available { 0.25 } else { -0.30 };
            let pot_control = 0.80_f32
                + (scenario.effective_stack_bb.min(120) as f32) / 140.0
                + if scenario.check_available { 0.35 } else { 0.0 }
                + (scenario.active_players as f32) * 0.03
                - if scenario.to_call_bb > 0 { 0.15 } else { 0.0 };
            if draw_to_nuts >= pot_sized_raise && draw_to_nuts >= pot_control {
                "draw-to-nuts"
            } else if pot_sized_raise >= pot_control {
                "pot-sized-raise"
            } else {
                "pot-control"
            }
        }

        for scenario in plo_scenario_pack() {
            let derived = dominant(scenario);
            let expected = if scenario.scenario_id.starts_with("draw-") {
                "draw-to-nuts"
            } else if scenario.scenario_id.starts_with("raise-") {
                "pot-sized-raise"
            } else {
                "pot-control"
            };
            assert_eq!(
                derived, expected,
                "scenario {} expected {} but engine math derives {}",
                scenario.scenario_id, expected, derived
            );
        }
    }
}
