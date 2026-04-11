use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::cards::{HanafudaCard, HanafudaKind, HanafudaMonth};
use crate::core::model::{CoreAction, CoreGameError, CoreGameState, CoreTransition};
use crate::game::ResearchGame;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FlowerVariant {
    HanafudaKoiKoi,
    HwatuGoStop,
}

impl FlowerVariant {
    const fn game(self) -> ResearchGame {
        match self {
            Self::HanafudaKoiKoi => ResearchGame::HanafudaKoiKoi,
            Self::HwatuGoStop => ResearchGame::HwatuGoStop,
        }
    }

    const fn capture_prefix(self) -> &'static str {
        match self {
            Self::HanafudaKoiKoi => "hanafuda-koi-koi.capture.",
            Self::HwatuGoStop => "hwatu-go-stop.capture.",
        }
    }

    const fn discard_prefix(self) -> &'static str {
        match self {
            Self::HanafudaKoiKoi => "hanafuda-koi-koi.discard.",
            Self::HwatuGoStop => "hwatu-go-stop.discard.",
        }
    }

    const fn koi_koi_action(self) -> &'static str {
        match self {
            Self::HanafudaKoiKoi => "hanafuda-koi-koi.koi-koi",
            Self::HwatuGoStop => "hwatu-go-stop.koi-koi",
        }
    }

    const fn stop_round_action(self) -> &'static str {
        match self {
            Self::HanafudaKoiKoi => "hanafuda-koi-koi.stop-round",
            Self::HwatuGoStop => "hwatu-go-stop.stop-round",
        }
    }

    const fn call_go_action(self) -> &'static str {
        match self {
            Self::HanafudaKoiKoi => "hanafuda-koi-koi.call-go",
            Self::HwatuGoStop => "hwatu-go-stop.call-go",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct HanafudaPublicState {
    variant: FlowerVariant,
    hand: Vec<HanafudaCard>,
    field: Vec<HanafudaCard>,
    captured: Vec<HanafudaCard>,
    draw_pile_commitment: String,
    yaku: Vec<SupportedYaku>,
    decision_window_open: bool,
    continuation_calls: u8,
    locked_points: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HanafudaFeatureView {
    pub points: u8,
    pub bright_count: u8,
    pub ribbon_yaku: u8,
    pub animal_yaku: u8,
    pub bonus_cards: u8,
    pub yaku_count: u8,
    pub bright_capture_options: u8,
    pub opponent_pressure: u8,
    pub hand_count: u8,
    pub decision_window: bool,
    pub locked_points: u8,
    pub continuation_calls: u8,
    pub upside_capture_options: u8,
    pub max_upside_gain: u8,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SupportedYaku {
    ThreeBrights,
    RibbonSet,
    AnimalSet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HanafudaAction {
    Capture(HanafudaCard),
    Discard(HanafudaCard),
    KoiKoi,
    CallGo,
    StopRound,
}

pub fn hanafuda_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(
        HanafudaPublicState {
            variant: FlowerVariant::HanafudaKoiKoi,
            hand: vec![
                HanafudaCard {
                    month: HanafudaMonth::April,
                    kind: HanafudaKind::Ribbon,
                },
                HanafudaCard {
                    month: HanafudaMonth::October,
                    kind: HanafudaKind::Chaff,
                },
            ],
            field: vec![
                HanafudaCard {
                    month: HanafudaMonth::April,
                    kind: HanafudaKind::Chaff,
                },
                HanafudaCard {
                    month: HanafudaMonth::February,
                    kind: HanafudaKind::Chaff,
                },
                HanafudaCard {
                    month: HanafudaMonth::December,
                    kind: HanafudaKind::Chaff,
                },
            ],
            captured: vec![
                HanafudaCard {
                    month: HanafudaMonth::January,
                    kind: HanafudaKind::Bright,
                },
                HanafudaCard {
                    month: HanafudaMonth::March,
                    kind: HanafudaKind::Bright,
                },
                HanafudaCard {
                    month: HanafudaMonth::November,
                    kind: HanafudaKind::Bright,
                },
                HanafudaCard {
                    month: HanafudaMonth::May,
                    kind: HanafudaKind::Animal,
                },
                HanafudaCard {
                    month: HanafudaMonth::June,
                    kind: HanafudaKind::Animal,
                },
                HanafudaCard {
                    month: HanafudaMonth::July,
                    kind: HanafudaKind::Ribbon,
                },
                HanafudaCard {
                    month: HanafudaMonth::September,
                    kind: HanafudaKind::Ribbon,
                },
            ],
            draw_pile_commitment: "hanafuda.draw.bootstrap-v1".to_string(),
            yaku: vec![
                SupportedYaku::ThreeBrights,
                SupportedYaku::RibbonSet,
                SupportedYaku::AnimalSet,
            ],
            decision_window_open: true,
            continuation_calls: 0,
            locked_points: 0,
        },
        Some(0),
        false,
        None,
    )
}

pub fn hwatu_bootstrap_state() -> Result<CoreGameState, CoreGameError> {
    state_from_public(
        HanafudaPublicState {
            variant: FlowerVariant::HwatuGoStop,
            hand: vec![
                HanafudaCard {
                    month: HanafudaMonth::January,
                    kind: HanafudaKind::Bright,
                },
                HanafudaCard {
                    month: HanafudaMonth::March,
                    kind: HanafudaKind::Animal,
                },
            ],
            field: vec![
                HanafudaCard {
                    month: HanafudaMonth::January,
                    kind: HanafudaKind::Chaff,
                },
                HanafudaCard {
                    month: HanafudaMonth::March,
                    kind: HanafudaKind::Ribbon,
                },
                HanafudaCard {
                    month: HanafudaMonth::December,
                    kind: HanafudaKind::Chaff,
                },
            ],
            captured: vec![
                HanafudaCard {
                    month: HanafudaMonth::August,
                    kind: HanafudaKind::Bright,
                },
                HanafudaCard {
                    month: HanafudaMonth::November,
                    kind: HanafudaKind::Bright,
                },
                HanafudaCard {
                    month: HanafudaMonth::December,
                    kind: HanafudaKind::Bright,
                },
                HanafudaCard {
                    month: HanafudaMonth::April,
                    kind: HanafudaKind::Ribbon,
                },
                HanafudaCard {
                    month: HanafudaMonth::May,
                    kind: HanafudaKind::Animal,
                },
            ],
            draw_pile_commitment: "hwatu-go-stop.draw.bootstrap-v1".to_string(),
            yaku: vec![SupportedYaku::ThreeBrights],
            decision_window_open: true,
            continuation_calls: 0,
            locked_points: 0,
        },
        Some(0),
        false,
        None,
    )
}

pub fn apply_hanafuda_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, FlowerVariant::HanafudaKoiKoi)
}

pub fn apply_hwatu_action(
    state: &CoreGameState,
    action_id: &str,
    params: serde_json::Value,
) -> Result<CoreTransition, CoreGameError> {
    apply_variant_action(state, action_id, params, FlowerVariant::HwatuGoStop)
}

fn apply_variant_action(
    state: &CoreGameState,
    action_id: &str,
    _params: serde_json::Value,
    variant: FlowerVariant,
) -> Result<CoreTransition, CoreGameError> {
    let before_public: HanafudaPublicState = serde_json::from_value(state.public_state.clone())
        .map_err(|source| CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: source.to_string(),
        })?;
    if before_public.variant != variant {
        return Err(CoreGameError::InvalidParams {
            action_id: action_id.to_string(),
            reason: "state variant does not match flower-card dispatch target".to_string(),
        });
    }
    let action = parse_hanafuda_action(variant, action_id)?;
    validate_hanafuda_action(&before_public, action, action_id)?;
    if !state
        .legal_actions
        .iter()
        .any(|candidate| candidate.action_id == action_id)
    {
        return Err(illegal_hanafuda_action(
            variant,
            action_id,
            "action is not legal in this flower-card turn",
        ));
    }

    let mut after_public = before_public.clone();
    match action {
        HanafudaAction::Capture(card) => {
            remove_hanafuda_card(&mut after_public.hand, card);
            if let Some(captured_field) = remove_first_month(&mut after_public.field, card.month) {
                after_public.captured.push(card);
                after_public.captured.push(captured_field);
            }
            after_public.yaku = supported_yaku(after_public.variant, &after_public.captured);
            after_public.decision_window_open =
                estimated_points(&after_public) > after_public.locked_points;
        }
        HanafudaAction::Discard(card) => {
            remove_hanafuda_card(&mut after_public.hand, card);
            after_public.field.push(card);
            after_public.yaku = supported_yaku(after_public.variant, &after_public.captured);
            after_public.decision_window_open = false;
        }
        HanafudaAction::KoiKoi | HanafudaAction::CallGo => {
            after_public.decision_window_open = false;
            after_public.locked_points = estimated_points(&after_public);
            after_public.continuation_calls = after_public.continuation_calls.saturating_add(1);
        }
        HanafudaAction::StopRound => {}
    }
    let terminal = action == HanafudaAction::StopRound;
    let payoff = terminal.then_some(vec![1, -1]);
    let after = state_from_public(after_public, state.actor, terminal, payoff)?;
    let action = core_action_for_hanafuda(variant, action);

    Ok(CoreTransition {
        before: state.clone(),
        action,
        after,
    })
}

fn state_from_public(
    public: HanafudaPublicState,
    actor: Option<u8>,
    terminal: bool,
    payoff: Option<Vec<i64>>,
) -> Result<CoreGameState, CoreGameError> {
    let variant = public.variant;
    let phase = hanafuda_phase(&public);
    let legal_actions = if terminal {
        Vec::new()
    } else {
        legal_hanafuda_actions(&public)
            .into_iter()
            .map(|action| core_action_for_hanafuda(variant, action))
            .collect()
    };
    let public_state =
        serde_json::to_value(public).map_err(|source| CoreGameError::InvalidParams {
            action_id: format!("{}.bootstrap", variant.game().slug()),
            reason: source.to_string(),
        })?;

    Ok(CoreGameState {
        game: variant.game(),
        phase: phase.to_string(),
        actor,
        public_state,
        private_state_commitments: vec![format!("{}.draw.bootstrap-v1", variant.game().slug())],
        legal_actions,
        terminal,
        payoff,
    })
}

pub(crate) fn feature_view(state: &CoreGameState) -> Result<HanafudaFeatureView, CoreGameError> {
    let public: HanafudaPublicState =
        serde_json::from_value(state.public_state.clone()).map_err(|source| {
            CoreGameError::InvalidParams {
                action_id: format!("{}.feature-view", state.game.slug()),
                reason: source.to_string(),
            }
        })?;
    let capture_options = capture_options(&public);
    let current_points = estimated_points(&public);
    let upside_gains: Vec<u8> = capture_options
        .iter()
        .copied()
        .map(|card| capture_upside_gain(&public, card))
        .filter(|gain| *gain > 0)
        .collect();

    Ok(HanafudaFeatureView {
        points: current_points,
        bright_count: usize_to_u8(
            public
                .captured
                .iter()
                .filter(|card| card.kind == HanafudaKind::Bright)
                .count(),
        ),
        ribbon_yaku: usize_to_u8(
            public
                .captured
                .iter()
                .filter(|card| card.kind == HanafudaKind::Ribbon)
                .count(),
        ),
        animal_yaku: usize_to_u8(
            public
                .captured
                .iter()
                .filter(|card| card.kind == HanafudaKind::Animal)
                .count(),
        ),
        bonus_cards: usize_to_u8(capture_options.len()),
        yaku_count: usize_to_u8(public.yaku.len()),
        bright_capture_options: usize_to_u8(
            capture_options
                .iter()
                .copied()
                .filter(|card| capture_includes_bright(&public, *card))
                .count(),
        ),
        opponent_pressure: usize_to_u8(
            public
                .field
                .len()
                .saturating_sub(public.hand.len())
                .saturating_add(1),
        ),
        hand_count: usize_to_u8(public.hand.len()),
        decision_window: public.decision_window_open,
        locked_points: public.locked_points,
        continuation_calls: public.continuation_calls,
        upside_capture_options: usize_to_u8(upside_gains.len()),
        max_upside_gain: upside_gains.into_iter().max().unwrap_or(0),
    })
}

fn legal_hanafuda_actions(public: &HanafudaPublicState) -> Vec<HanafudaAction> {
    if public.decision_window_open {
        let mut actions = Vec::new();
        if public.variant == FlowerVariant::HwatuGoStop {
            actions.push(HanafudaAction::CallGo);
        }
        actions.push(HanafudaAction::KoiKoi);
        actions.push(HanafudaAction::StopRound);
        return actions;
    }
    if public.hand.is_empty() {
        return vec![HanafudaAction::StopRound];
    }
    let mut actions = Vec::new();
    let captures = capture_options(public);
    if captures.is_empty() {
        actions.extend(public.hand.iter().copied().map(HanafudaAction::Discard));
    } else {
        actions.extend(captures.into_iter().map(HanafudaAction::Capture));
    }

    actions
}

fn validate_hanafuda_action(
    public: &HanafudaPublicState,
    action: HanafudaAction,
    action_id: &str,
) -> Result<(), CoreGameError> {
    match action {
        HanafudaAction::Capture(card) => {
            if !public.hand.contains(&card) {
                return Err(illegal_hanafuda_action(
                    public.variant,
                    action_id,
                    "card is not in hand",
                ));
            }
            if !public.field.iter().any(|field| field.month == card.month) {
                return Err(illegal_hanafuda_action(
                    public.variant,
                    action_id,
                    "capture requires matching month on the field",
                ));
            }
        }
        HanafudaAction::Discard(card) => {
            if !public.hand.contains(&card) {
                return Err(illegal_hanafuda_action(
                    public.variant,
                    action_id,
                    "card is not in hand",
                ));
            }
            if public.field.iter().any(|field| field.month == card.month) {
                return Err(illegal_hanafuda_action(
                    public.variant,
                    action_id,
                    "discard is illegal when a capture is available for that month",
                ));
            }
        }
        HanafudaAction::KoiKoi | HanafudaAction::StopRound | HanafudaAction::CallGo => {
            if action == HanafudaAction::StopRound && public.hand.is_empty() {
                return Ok(());
            }
            if !public.decision_window_open {
                return Err(illegal_hanafuda_action(
                    public.variant,
                    action_id,
                    "go/stop decisions require an open scoring window",
                ));
            }
            if action == HanafudaAction::CallGo && public.variant != FlowerVariant::HwatuGoStop {
                return Err(illegal_hanafuda_action(
                    public.variant,
                    action_id,
                    "call-go only applies to hwatu go-stop",
                ));
            }
        }
    }

    Ok(())
}

fn core_action_for_hanafuda(variant: FlowerVariant, action: HanafudaAction) -> CoreAction {
    match action {
        HanafudaAction::Capture(card) => CoreAction {
            action_id: format!("{}{}", variant.capture_prefix(), hanafuda_card_token(card)),
            display_label: format!("capture-{}", hanafuda_card_token(card)),
            params: json!({"card": card}),
        },
        HanafudaAction::Discard(card) => CoreAction {
            action_id: format!("{}{}", variant.discard_prefix(), hanafuda_card_token(card)),
            display_label: format!("discard-{}", hanafuda_card_token(card)),
            params: json!({"card": card}),
        },
        HanafudaAction::KoiKoi => CoreAction {
            action_id: variant.koi_koi_action().to_string(),
            display_label: "koi-koi".to_string(),
            params: json!({}),
        },
        HanafudaAction::CallGo => CoreAction {
            action_id: variant.call_go_action().to_string(),
            display_label: "call-go".to_string(),
            params: json!({}),
        },
        HanafudaAction::StopRound => CoreAction {
            action_id: variant.stop_round_action().to_string(),
            display_label: "stop-round".to_string(),
            params: json!({}),
        },
    }
}

fn parse_hanafuda_action(
    variant: FlowerVariant,
    action_id: &str,
) -> Result<HanafudaAction, CoreGameError> {
    if action_id == variant.koi_koi_action() {
        return Ok(HanafudaAction::KoiKoi);
    }
    if action_id == variant.stop_round_action() {
        return Ok(HanafudaAction::StopRound);
    }
    if action_id == variant.call_go_action() {
        return Ok(HanafudaAction::CallGo);
    }
    if let Some(token) = action_id.strip_prefix(variant.capture_prefix()) {
        return Ok(HanafudaAction::Capture(parse_hanafuda_card(
            variant, action_id, token,
        )?));
    }
    if let Some(token) = action_id.strip_prefix(variant.discard_prefix()) {
        return Ok(HanafudaAction::Discard(parse_hanafuda_card(
            variant, action_id, token,
        )?));
    }

    Err(CoreGameError::UnknownAction {
        game: variant.game(),
        action_id: action_id.to_string(),
    })
}

fn parse_hanafuda_card(
    variant: FlowerVariant,
    action_id: &str,
    token: &str,
) -> Result<HanafudaCard, CoreGameError> {
    let Some((month, kind)) = token.split_once('-') else {
        return Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        });
    };
    let month = parse_month(variant, action_id, month)?;
    let kind = parse_kind(variant, action_id, kind)?;

    Ok(HanafudaCard { month, kind })
}

fn hanafuda_phase(public: &HanafudaPublicState) -> &'static str {
    if public.decision_window_open {
        "decision"
    } else {
        "capture"
    }
}

fn capture_options(public: &HanafudaPublicState) -> Vec<HanafudaCard> {
    public
        .hand
        .iter()
        .copied()
        .filter(|card| public.field.iter().any(|field| field.month == card.month))
        .collect()
}

fn remove_hanafuda_card(cards: &mut Vec<HanafudaCard>, card: HanafudaCard) {
    let mut removed = false;
    cards.retain(|candidate| {
        if *candidate == card && !removed {
            removed = true;
            false
        } else {
            true
        }
    });
}

fn remove_first_month(cards: &mut Vec<HanafudaCard>, month: HanafudaMonth) -> Option<HanafudaCard> {
    let position = cards
        .iter()
        .position(|candidate| candidate.month == month)?;
    Some(cards.remove(position))
}

fn capture_includes_bright(public: &HanafudaPublicState, card: HanafudaCard) -> bool {
    card.kind == HanafudaKind::Bright
        || public
            .field
            .iter()
            .any(|field| field.month == card.month && field.kind == HanafudaKind::Bright)
}

fn capture_upside_gain(public: &HanafudaPublicState, card: HanafudaCard) -> u8 {
    let Some(field_card) = public
        .field
        .iter()
        .find(|field| field.month == card.month)
        .copied()
    else {
        return 0;
    };
    let current_upside = continuation_upside(&public.captured);
    let mut captured = public.captured.clone();
    captured.push(card);
    captured.push(field_card);
    usize_to_u8(continuation_upside(&captured).saturating_sub(current_upside))
}

fn continuation_upside(captured: &[HanafudaCard]) -> usize {
    let bright_count = captured_kind_count(captured, HanafudaKind::Bright);
    let ribbon_count = captured_kind_count(captured, HanafudaKind::Ribbon);
    let animal_count = captured_kind_count(captured, HanafudaKind::Animal);
    let bright_bonus = bright_count.saturating_sub(2).saturating_mul(2);
    let ribbon_bonus = ribbon_count.saturating_sub(1);
    let animal_bonus = animal_count.saturating_sub(1);

    bright_count
        .saturating_add(bright_bonus)
        .saturating_add(ribbon_count)
        .saturating_add(ribbon_bonus)
        .saturating_add(animal_count)
        .saturating_add(animal_bonus)
}

fn captured_kind_count(captured: &[HanafudaCard], kind: HanafudaKind) -> usize {
    captured.iter().filter(|card| card.kind == kind).count()
}

fn estimated_points(public: &HanafudaPublicState) -> u8 {
    usize_to_u8(
        public
            .yaku
            .iter()
            .map(|yaku| yaku_points(public.variant, *yaku))
            .sum(),
    )
}

fn supported_yaku(_variant: FlowerVariant, captured: &[HanafudaCard]) -> Vec<SupportedYaku> {
    let bright_count = captured
        .iter()
        .filter(|card| card.kind == HanafudaKind::Bright)
        .count();
    let ribbon_count = captured
        .iter()
        .filter(|card| card.kind == HanafudaKind::Ribbon)
        .count();
    let animal_count = captured
        .iter()
        .filter(|card| card.kind == HanafudaKind::Animal)
        .count();
    let mut yaku = Vec::new();
    if bright_count >= 3 {
        yaku.push(SupportedYaku::ThreeBrights);
    }
    if ribbon_count >= 2 {
        yaku.push(SupportedYaku::RibbonSet);
    }
    if animal_count >= 2 {
        yaku.push(SupportedYaku::AnimalSet);
    }
    yaku
}

fn yaku_points(variant: FlowerVariant, yaku: SupportedYaku) -> usize {
    match (variant, yaku) {
        (_, SupportedYaku::ThreeBrights) => 5,
        (FlowerVariant::HanafudaKoiKoi, SupportedYaku::RibbonSet) => 1,
        (FlowerVariant::HanafudaKoiKoi, SupportedYaku::AnimalSet) => 1,
        (FlowerVariant::HwatuGoStop, SupportedYaku::RibbonSet) => 1,
        (FlowerVariant::HwatuGoStop, SupportedYaku::AnimalSet) => 1,
    }
}

fn hanafuda_card_token(card: HanafudaCard) -> String {
    format!("{}-{}", month_token(card.month), kind_token(card.kind))
}

fn month_token(month: HanafudaMonth) -> &'static str {
    match month {
        HanafudaMonth::January => "january",
        HanafudaMonth::February => "february",
        HanafudaMonth::March => "march",
        HanafudaMonth::April => "april",
        HanafudaMonth::May => "may",
        HanafudaMonth::June => "june",
        HanafudaMonth::July => "july",
        HanafudaMonth::August => "august",
        HanafudaMonth::September => "september",
        HanafudaMonth::October => "october",
        HanafudaMonth::November => "november",
        HanafudaMonth::December => "december",
    }
}

fn kind_token(kind: HanafudaKind) -> &'static str {
    match kind {
        HanafudaKind::Bright => "bright",
        HanafudaKind::Animal => "animal",
        HanafudaKind::Ribbon => "ribbon",
        HanafudaKind::Chaff => "chaff",
    }
}

fn parse_month(
    variant: FlowerVariant,
    action_id: &str,
    month: &str,
) -> Result<HanafudaMonth, CoreGameError> {
    match month {
        "january" => Ok(HanafudaMonth::January),
        "february" => Ok(HanafudaMonth::February),
        "march" => Ok(HanafudaMonth::March),
        "april" => Ok(HanafudaMonth::April),
        "may" => Ok(HanafudaMonth::May),
        "june" => Ok(HanafudaMonth::June),
        "july" => Ok(HanafudaMonth::July),
        "august" => Ok(HanafudaMonth::August),
        "september" => Ok(HanafudaMonth::September),
        "october" => Ok(HanafudaMonth::October),
        "november" => Ok(HanafudaMonth::November),
        "december" => Ok(HanafudaMonth::December),
        _ => Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        }),
    }
}

fn parse_kind(
    variant: FlowerVariant,
    action_id: &str,
    kind: &str,
) -> Result<HanafudaKind, CoreGameError> {
    match kind {
        "bright" => Ok(HanafudaKind::Bright),
        "animal" => Ok(HanafudaKind::Animal),
        "ribbon" => Ok(HanafudaKind::Ribbon),
        "chaff" => Ok(HanafudaKind::Chaff),
        _ => Err(CoreGameError::UnknownAction {
            game: variant.game(),
            action_id: action_id.to_string(),
        }),
    }
}

fn illegal_hanafuda_action(variant: FlowerVariant, action_id: &str, reason: &str) -> CoreGameError {
    CoreGameError::IllegalAction {
        game: variant.game(),
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
    fn hanafuda_capture_matches_month() {
        let state = no_yaku_state();
        let transition = apply_action(&state, "hanafuda-koi-koi.capture.april-ribbon", json!({}))
            .unwrap_or_else(|error| panic!("matching capture should apply: {error}"));
        let public: HanafudaPublicState = serde_json::from_value(transition.after.public_state)
            .unwrap_or_else(|error| panic!("hanafuda public state should decode: {error}"));

        assert_eq!(public.captured.len(), 2);
    }

    #[test]
    fn hanafuda_bootstrap_exposes_koi_koi_window() {
        let state = flower_state(ResearchGame::HanafudaKoiKoi);

        assert_eq!(state.phase, "decision");
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "hanafuda-koi-koi.koi-koi")
        );
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "hanafuda-koi-koi.stop-round")
        );
        assert_eq!(
            supported_yaku_names(),
            vec!["three-brights", "ribbon-set", "animal-set"]
        );
    }

    #[test]
    fn hanafuda_koi_koi_gated_on_yaku() {
        let state = no_yaku_state();

        assert!(
            !state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "hanafuda-koi-koi.koi-koi")
        );
    }

    #[test]
    fn hwatu_bootstrap_exposes_call_go() {
        let state = flower_state(ResearchGame::HwatuGoStop);

        assert_eq!(state.phase, "decision");
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "hwatu-go-stop.call-go")
        );
        assert!(
            state
                .legal_actions
                .iter()
                .any(|action| action.action_id == "hwatu-go-stop.stop-round")
        );
    }

    #[test]
    fn hanafuda_rejects_capture_without_matching_month() {
        let state = flower_state(ResearchGame::HanafudaKoiKoi);

        assert!(matches!(
            apply_action(&state, "hanafuda-koi-koi.capture.october-chaff", json!({})),
            Err(CoreGameError::IllegalAction { reason, .. }) if reason.contains("matching month")
        ));
    }

    #[test]
    fn hanafuda_transition_is_deterministic() {
        let state = flower_state(ResearchGame::HanafudaKoiKoi);
        let first = apply_action(&state, "hanafuda-koi-koi.capture.april-ribbon", json!({}));
        let second = apply_action(&state, "hanafuda-koi-koi.capture.april-ribbon", json!({}));

        assert_eq!(first, second);
    }

    #[test]
    fn hanafuda_feature_view_tracks_score_window() {
        let state = flower_state(ResearchGame::HanafudaKoiKoi);
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("hanafuda feature view should decode: {error}"));

        assert_eq!(view.points, 7);
        assert_eq!(view.bright_count, 3);
        assert_eq!(view.ribbon_yaku, 2);
        assert_eq!(view.animal_yaku, 2);
        assert_eq!(view.bonus_cards, 1);
        assert_eq!(view.yaku_count, 3);
        assert_eq!(view.bright_capture_options, 0);
        assert_eq!(view.opponent_pressure, 2);
        assert_eq!(view.hand_count, 2);
        assert!(view.decision_window);
        assert_eq!(view.locked_points, 0);
        assert_eq!(view.continuation_calls, 0);
        assert_eq!(view.upside_capture_options, 1);
        assert_eq!(view.max_upside_gain, 2);
    }

    #[test]
    fn hwatu_feature_view_tracks_capture_pressure() {
        let state = flower_state(ResearchGame::HwatuGoStop);
        let view = feature_view(&state)
            .unwrap_or_else(|error| panic!("hwatu feature view should decode: {error}"));

        assert_eq!(view.points, 5);
        assert_eq!(view.bright_count, 3);
        assert_eq!(view.bonus_cards, 2);
        assert_eq!(view.yaku_count, 1);
        assert_eq!(view.bright_capture_options, 1);
        assert_eq!(view.opponent_pressure, 2);
        assert_eq!(view.hand_count, 2);
        assert!(view.decision_window);
        assert_eq!(view.locked_points, 0);
        assert_eq!(view.continuation_calls, 0);
        assert_eq!(view.upside_capture_options, 2);
        assert_eq!(view.max_upside_gain, 4);
    }

    #[test]
    fn koi_koi_closes_window_and_returns_to_capture_phase() {
        let state = flower_state(ResearchGame::HanafudaKoiKoi);
        let transition = apply_action(&state, "hanafuda-koi-koi.koi-koi", json!({}))
            .unwrap_or_else(|error| panic!("koi-koi should apply: {error}"));
        let public: HanafudaPublicState = serde_json::from_value(transition.after.public_state)
            .unwrap_or_else(|error| panic!("continued hanafuda state should decode: {error}"));

        assert_eq!(transition.after.phase, "capture");
        assert!(!public.decision_window_open);
        assert_eq!(public.continuation_calls, 1);
        assert_eq!(public.locked_points, 7);
        assert!(
            transition
                .after
                .legal_actions
                .iter()
                .all(|action| !action.action_id.ends_with("koi-koi"))
        );
        assert!(
            transition
                .after
                .legal_actions
                .iter()
                .any(|action| action.action_id == "hanafuda-koi-koi.capture.april-ribbon")
        );
    }

    #[test]
    fn scoring_capture_reopens_window_after_continuation() {
        let state = state_from_public(
            HanafudaPublicState {
                variant: FlowerVariant::HanafudaKoiKoi,
                hand: vec![
                    HanafudaCard {
                        month: HanafudaMonth::May,
                        kind: HanafudaKind::Animal,
                    },
                    HanafudaCard {
                        month: HanafudaMonth::October,
                        kind: HanafudaKind::Chaff,
                    },
                ],
                field: vec![
                    HanafudaCard {
                        month: HanafudaMonth::May,
                        kind: HanafudaKind::Chaff,
                    },
                    HanafudaCard {
                        month: HanafudaMonth::December,
                        kind: HanafudaKind::Chaff,
                    },
                ],
                captured: vec![
                    HanafudaCard {
                        month: HanafudaMonth::January,
                        kind: HanafudaKind::Bright,
                    },
                    HanafudaCard {
                        month: HanafudaMonth::March,
                        kind: HanafudaKind::Bright,
                    },
                    HanafudaCard {
                        month: HanafudaMonth::November,
                        kind: HanafudaKind::Bright,
                    },
                    HanafudaCard {
                        month: HanafudaMonth::June,
                        kind: HanafudaKind::Animal,
                    },
                ],
                draw_pile_commitment: "hanafuda.draw.bootstrap-v1".to_string(),
                yaku: vec![SupportedYaku::ThreeBrights],
                decision_window_open: false,
                continuation_calls: 1,
                locked_points: 5,
            },
            Some(0),
            false,
            None,
        )
        .unwrap_or_else(|error| panic!("continued hanafuda state should build: {error}"));
        let transition = apply_action(&state, "hanafuda-koi-koi.capture.may-animal", json!({}))
            .unwrap_or_else(|error| panic!("scoring capture should apply: {error}"));
        let public: HanafudaPublicState = serde_json::from_value(transition.after.public_state)
            .unwrap_or_else(|error| panic!("reopened hanafuda state should decode: {error}"));

        assert_eq!(transition.after.phase, "decision");
        assert!(public.decision_window_open);
        assert_eq!(public.continuation_calls, 1);
        assert_eq!(estimated_points(&public), 6);
        assert!(
            transition
                .after
                .legal_actions
                .iter()
                .any(|action| action.action_id == "hanafuda-koi-koi.stop-round")
        );
    }

    fn flower_state(game: ResearchGame) -> CoreGameState {
        bootstrap_state(game)
            .unwrap_or_else(|error| panic!("{} bootstrap should succeed: {error}", game.slug()))
    }

    fn supported_yaku_names() -> Vec<&'static str> {
        vec!["three-brights", "ribbon-set", "animal-set"]
    }

    fn no_yaku_state() -> CoreGameState {
        state_from_public(
            HanafudaPublicState {
                variant: FlowerVariant::HanafudaKoiKoi,
                hand: vec![
                    HanafudaCard {
                        month: HanafudaMonth::April,
                        kind: HanafudaKind::Ribbon,
                    },
                    HanafudaCard {
                        month: HanafudaMonth::October,
                        kind: HanafudaKind::Chaff,
                    },
                ],
                field: vec![
                    HanafudaCard {
                        month: HanafudaMonth::April,
                        kind: HanafudaKind::Chaff,
                    },
                    HanafudaCard {
                        month: HanafudaMonth::February,
                        kind: HanafudaKind::Chaff,
                    },
                ],
                captured: Vec::new(),
                draw_pile_commitment: "hanafuda.draw.bootstrap-v1".to_string(),
                yaku: Vec::new(),
                decision_window_open: false,
                continuation_calls: 0,
                locked_points: 0,
            },
            Some(0),
            false,
            None,
        )
        .unwrap_or_else(|error| panic!("no-yaku hanafuda state should build: {error}"))
    }
}
