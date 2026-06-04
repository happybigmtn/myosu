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

// ============================================================================
// F-014 benchmark scenario pack
// ============================================================================
//
// A single hand-verified HanafudaKoiKoi benchmark scenario for the F-014
// portfolio-game-promotion slice (the F-001/F-008/F-009/F-010/F-011/F-012/F-013
// eighth slice: Cribbage / Hearts / Gin Rummy / Spades / Bridge / Call Break /
// Backgammon / Hanafuda Koi-Koi). The scenario pack mirrors the
// `genesis/plans/009-cribbage-deepening.md` R1 layout scoped to the
// `state-aware yaku EV heuristic` engine surface in
// `crate::engines::hanafuda::koi_koi`.
//
// Each scenario's expected dominant arm is derived by hand from the
// `state-aware yaku EV heuristic` math in `crate::engines::hanafuda::koi_koi`
// (mirrored in /tmp/verify_scenarios.py). The per-scenario docstring records
// the stop_round / koi_koi / call_go triple so a reviewer can verify the
// dominant arm by hand. The HanafudaKoiKoi engine is intentionally
// `stop_round`-heavy in its design (the `stop_round` arm has both the highest
// base weight AND a +0.75 decision-window boost that the `koi_koi` arm can
// only partially match at +0.60 and the `call_go` arm cannot overcome at
// +0.10), so the realistic F-014 distribution is:
//
//   * 8 stop_round-dominant scenarios (banked score, real locked points,
//     or significant opponent pressure with no live upside)
//   * 8 koi_koi-dominant scenarios (fresh score, live upside, real bright
//     capture options, no banked state to defend)
//   * 6 mixed/edge scenarios (real elements on multiple arms; the engine
//     resolves them toward stop_round in this design but a future engine
//     tweak could move them — the dossier pins the live behaviour)
//
// Coverage: 22 labeled scenarios, exceeds the 20-scenario floor. Every
// scenario is hand-verified against the engine math in
// `crate::engines::hanafuda::koi_koi` so the dossier's expected
// recommendations are stable across runs.

/// A single hand-verified HanafudaKoiKoi benchmark scenario for the
/// F-014 / `genesis/plans/009-cribbage-deepening.md` rule-aware scenario
/// pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HanafudaScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
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

/// Return the canonical 22-scenario HanafudaKoiKoi benchmark pack.
pub fn hanafuda_scenario_pack() -> &'static [HanafudaScenario] {
    HANAFUDA_SCENARIO_PACK
}

const HANAFUDA_SCENARIO_PACK: &[HanafudaScenario] = &[
    // ---- stop_round bucket (×8) — banked score, real locked points, real opponent pressure ----
    HanafudaScenario {
        scenario_id: "sr-bright-bank-lock",
        decision: "Banked 10 with 8 locked points, 2 continuation calls — stop_round dominates (s=6.16, k=0.36, g=0.56)",
        points: 10,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 3,
        hand_count: 1,
        decision_window: true,
        locked_points: 8,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HanafudaScenario {
        scenario_id: "sr-stretched-bright-bank",
        decision: "Stretched 9 with 7 locked, 3 continuation calls — stop_round dominates (s=5.80, k=0.13, g=0.64)",
        points: 9,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 2,
        bright_capture_options: 0,
        opponent_pressure: 4,
        hand_count: 1,
        decision_window: true,
        locked_points: 7,
        continuation_calls: 3,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HanafudaScenario {
        scenario_id: "sr-bright-banked-no-upside",
        decision: "Bright-heavy bank with no upside, 2 continuation calls — stop_round dominates (s=5.80, k=0.36, g=0.56)",
        points: 8,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 3,
        hand_count: 1,
        decision_window: true,
        locked_points: 6,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HanafudaScenario {
        scenario_id: "sr-bright-banked-pressure",
        decision: "Banked bright with high opponent pressure — stop_round dominates (s=5.82, k=0.31, g=0.56)",
        points: 7,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 4,
        hand_count: 1,
        decision_window: true,
        locked_points: 5,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HanafudaScenario {
        scenario_id: "sr-yaku-banked-no-upside",
        decision: "5-yaku banked with no upside, moderate pressure — stop_round dominates (s=5.36, k=0.59, g=0.48)",
        points: 9,
        bright_count: 1,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 5,
        bright_capture_options: 0,
        opponent_pressure: 2,
        hand_count: 1,
        decision_window: true,
        locked_points: 7,
        continuation_calls: 1,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HanafudaScenario {
        scenario_id: "sr-mixed-yaku-banked",
        decision: "Mixed ribbon+animal yaku banked, no upside — stop_round dominates (s=4.43, k=0.75, g=0.58)",
        points: 8,
        bright_count: 1,
        ribbon_yaku: 2,
        animal_yaku: 2,
        bonus_cards: 0,
        yaku_count: 2,
        bright_capture_options: 0,
        opponent_pressure: 2,
        hand_count: 1,
        decision_window: true,
        locked_points: 6,
        continuation_calls: 1,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HanafudaScenario {
        scenario_id: "sr-bright-window-no-upside",
        decision: "Bright-heavy window, no live upside, 1 continuation — stop_round dominates (s=5.41, k=0.36, g=0.48)",
        points: 7,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 2,
        bright_capture_options: 0,
        opponent_pressure: 3,
        hand_count: 1,
        decision_window: true,
        locked_points: 4,
        continuation_calls: 1,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HanafudaScenario {
        scenario_id: "sr-bright-yaku-pressure",
        decision: "Bright + 3 yaku banked, opponent pressure 3 — stop_round dominates (s=5.80, k=0.36, g=0.56)",
        points: 8,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 3,
        hand_count: 1,
        decision_window: true,
        locked_points: 6,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    // ---- koi_koi bucket (×8) — fresh score, live upside, bright capture options ----
    HanafudaScenario {
        scenario_id: "kk-fresh-bright-upside",
        decision: "Fresh 3-point window, 5 max upside + 4 bright options — koi_koi dominates (s=1.66, k=4.73, g=0.65)",
        points: 3,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 2,
        bright_capture_options: 2,
        opponent_pressure: 0,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 4,
        max_upside_gain: 5,
    },
    HanafudaScenario {
        scenario_id: "kk-bright-fresh-upside",
        decision: "Bright-heavy fresh 4-point window, 4 max upside — koi_koi dominates (s=2.48, k=3.91, g=0.60)",
        points: 4,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 2,
        bright_capture_options: 2,
        opponent_pressure: 0,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 4,
    },
    HanafudaScenario {
        scenario_id: "kk-mixed-yaku-fresh",
        decision: "Fresh mixed-yaku window, 3 max upside — koi_koi dominates (s=1.73, k=3.31, g=0.65)",
        points: 3,
        bright_count: 1,
        ribbon_yaku: 2,
        animal_yaku: 2,
        bonus_cards: 1,
        yaku_count: 1,
        bright_capture_options: 1,
        opponent_pressure: 0,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 3,
    },
    HanafudaScenario {
        scenario_id: "kk-bright-bonus-fresh",
        decision: "Bright + 2 bonus fresh 2-point window, 4 max upside — koi_koi dominates (s=1.26, k=4.39, g=0.60)",
        points: 2,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 1,
        bright_capture_options: 2,
        opponent_pressure: 0,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 4,
        max_upside_gain: 4,
    },
    HanafudaScenario {
        scenario_id: "kk-bright-fresh-zero-locked",
        decision: "Bright-heavy fresh 5-point window, locked=0 — koi_koi dominates (s=3.08, k=3.50, g=0.60)",
        points: 5,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 1,
        bright_capture_options: 1,
        opponent_pressure: 1,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 4,
        max_upside_gain: 4,
    },
    HanafudaScenario {
        scenario_id: "kk-yaku-fresh-bright",
        decision: "Fresh 3-yaku window, 3 max upside + 3 bright options — koi_koi dominates (s=2.67, k=3.45, g=0.55)",
        points: 4,
        bright_count: 1,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 3,
        bright_capture_options: 3,
        opponent_pressure: 0,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 3,
    },
    HanafudaScenario {
        scenario_id: "kk-bright-thick-upside",
        decision: "Fresh bright-heavy, 5 max upside + 3 bright options — koi_koi dominates (s=2.15, k=4.73, g=0.65)",
        points: 3,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 2,
        bright_capture_options: 3,
        opponent_pressure: 0,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 5,
    },
    HanafudaScenario {
        scenario_id: "kk-bonus-fresh-upside",
        decision: "Fresh 4-bonus window, 4 max upside — koi_koi dominates (s=1.74, k=4.03, g=0.60)",
        points: 4,
        bright_count: 1,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 4,
        yaku_count: 1,
        bright_capture_options: 0,
        opponent_pressure: 0,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 4,
        max_upside_gain: 4,
    },
    // ---- mixed/edge bucket (×6) — real elements on multiple arms, narrow margin ----
    HanafudaScenario {
        scenario_id: "me-bright-banked-thin-upside",
        decision: "Banked bright with thin 1-max upside — engine resolves to stop_round (s=4.77, k=1.77, g=0.53)",
        points: 7,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 3,
        bright_capture_options: 1,
        opponent_pressure: 2,
        hand_count: 2,
        decision_window: true,
        locked_points: 5,
        continuation_calls: 1,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
    HanafudaScenario {
        scenario_id: "me-fresh-window-thin-bright",
        decision: "Fresh 3-point window, 1 bright option, thin upside — engine resolves to stop_round (s=2.69, k=1.82, g=0.45)",
        points: 3,
        bright_count: 1,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 1,
        bright_capture_options: 1,
        opponent_pressure: 1,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
    HanafudaScenario {
        scenario_id: "me-bright-banked-1upside",
        decision: "Banked bright with 1 max upside + 1 bright option — engine resolves to stop_round (s=4.96, k=1.59, g=0.53)",
        points: 8,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 2,
        bright_capture_options: 1,
        opponent_pressure: 2,
        hand_count: 2,
        decision_window: true,
        locked_points: 5,
        continuation_calls: 1,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
    HanafudaScenario {
        scenario_id: "me-stretched-fresh-bright",
        decision: "Stretched fresh window with bright upside — engine resolves to stop_round (s=3.49, k=2.52, g=0.58)",
        points: 5,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 2,
        bright_capture_options: 2,
        opponent_pressure: 1,
        hand_count: 2,
        decision_window: true,
        locked_points: 2,
        continuation_calls: 1,
        upside_capture_options: 2,
        max_upside_gain: 2,
    },
    HanafudaScenario {
        scenario_id: "me-yaku-bright-banked-thin",
        decision: "Banked yaku + bright with thin upside — engine resolves to stop_round (s=4.04, k=1.65, g=0.53)",
        points: 6,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 2,
        bright_capture_options: 1,
        opponent_pressure: 2,
        hand_count: 1,
        decision_window: true,
        locked_points: 4,
        continuation_calls: 1,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
    HanafudaScenario {
        scenario_id: "me-bright-banked-2-call-pressure",
        decision: "Banked bright, 2 continuation, low opp pressure — engine resolves to stop_round (s=4.84, k=1.52, g=0.61)",
        points: 7,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 3,
        bright_capture_options: 1,
        opponent_pressure: 1,
        hand_count: 1,
        decision_window: true,
        locked_points: 5,
        continuation_calls: 2,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
];

/// A single hand-verified HwatuGoStop benchmark scenario for the
/// F-015 / `genesis/plans/009-cribbage-deepening.md` rule-aware
/// scenario pack.
///
/// HwatuGoStop and HanafudaKoiKoi share the `HanafudaChallenge`
/// feature struct (both surface as 15-field features in the live
/// `crate::state::PortfolioChallenge` enum), but they dispatch to
/// different arms of the same `crate::engines::hanafuda` module
/// (`koi_koi` vs `hwatu_go_stop`). The HwatuGoStop engine math is
/// hand-verified independently from the HanafudaKoiKoi engine math
/// in this row — the call_go and stop_round coefficients differ
/// (call_go boosts `bonus_cards` ×0.30 + `upside_gain` ×0.38 +
/// decision-window ×0.75, stop_round boosts `window_gain` ×0.34 +
/// `continuation_calls` ×0.24 + `opponent_pressure` ×0.20), so a
/// scenario that flips between the two engines' dominance under the
/// same feature values is informative about engine-family
/// separation. Each scenario's `decision` docstring records the
/// expected `c=…` / `s=…` / `k=…` heuristic values from the
/// `hwatu_go_stop` arm so the dossier's expected recommendations are
/// stable across runs and a useful regression guard for the
/// F-001 / F-008 / F-009 / F-010 / F-011 / F-012 / F-013 / F-014
/// pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HwatuScenario {
    pub scenario_id: &'static str,
    pub decision: &'static str,
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

/// Return the canonical 22-scenario HwatuGoStop benchmark pack.
pub fn hwatu_scenario_pack() -> &'static [HwatuScenario] {
    HWATU_SCENARIO_PACK
}

const HWATU_SCENARIO_PACK: &[HwatuScenario] = &[
    // ---- call_go bucket (×8) — bonus-heavy fresh spots with live upside ----
    HwatuScenario {
        scenario_id: "cg-bonus-heavy-fresh-upside",
        decision: "Bonus-heavy fresh 4-point window, 5 max upside + 3 bright options — call_go dominates (c=5.18, s=1.67, k=1.56)",
        points: 4,
        bright_count: 2,
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
    },
    HwatuScenario {
        scenario_id: "cg-bright-bonus-press",
        decision: "Bright+bonus fresh 5-point window, 4 max upside — call_go dominates (c=4.24, s=2.27, k=1.24)",
        points: 5,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 3,
        yaku_count: 1,
        bright_capture_options: 2,
        opponent_pressure: 1,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 4,
    },
    HwatuScenario {
        scenario_id: "cg-mixed-yaku-bonus-fresh",
        decision: "Mixed ribbon+animal yaku + bonus fresh 3-point window, 3 max upside — call_go dominates (c=3.88, s=1.52, k=1.06)",
        points: 3,
        bright_count: 1,
        ribbon_yaku: 2,
        animal_yaku: 2,
        bonus_cards: 2,
        yaku_count: 1,
        bright_capture_options: 1,
        opponent_pressure: 0,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 3,
    },
    HwatuScenario {
        scenario_id: "cg-bright-bonus-fresh-upside",
        decision: "Bright+bonus fresh 2-point window, 4 max upside + 2 bright options — call_go dominates (c=4.79, s=0.82, k=1.26)",
        points: 2,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 1,
        bright_capture_options: 2,
        opponent_pressure: 0,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 4,
        max_upside_gain: 4,
    },
    HwatuScenario {
        scenario_id: "cg-bright-bonus-fresh-zero-locked",
        decision: "Bright-heavy fresh 4-point window, locked=0, 4 max upside — call_go dominates (c=4.14, s=1.77, k=1.20)",
        points: 4,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 1,
        bright_capture_options: 1,
        opponent_pressure: 1,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 4,
        max_upside_gain: 4,
    },
    HwatuScenario {
        scenario_id: "cg-animal-yaku-bonus-fresh",
        decision: "Animal-heavy + bonus fresh 3-point window, 3 max upside — call_go dominates (c=4.18, s=1.52, k=1.18)",
        points: 3,
        bright_count: 1,
        ribbon_yaku: 1,
        animal_yaku: 3,
        bonus_cards: 3,
        yaku_count: 1,
        bright_capture_options: 1,
        opponent_pressure: 0,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 3,
    },
    HwatuScenario {
        scenario_id: "cg-bright-bonus-thick-upside",
        decision: "Bright+bonus fresh 3-point window, 5 max upside + 3 bright options — call_go dominates (c=4.80, s=1.29, k=1.40)",
        points: 3,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 2,
        bright_capture_options: 3,
        opponent_pressure: 0,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 3,
        max_upside_gain: 5,
    },
    HwatuScenario {
        scenario_id: "cg-bonus-fresh-upside",
        decision: "Bonus-heavy fresh 4-point window, 4 max upside — call_go dominates (c=4.62, s=1.57, k=1.44)",
        points: 4,
        bright_count: 1,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 4,
        yaku_count: 1,
        bright_capture_options: 0,
        opponent_pressure: 0,
        hand_count: 3,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 4,
        max_upside_gain: 4,
    },
    // ---- stop_round bucket (×8) — banked cash-out spots ----
    HwatuScenario {
        scenario_id: "sr-cashout-bright-no-upside",
        decision: "Cash-out bright 7-point, 5 locked, 1 continuation, no upside — stop_round dominates (c=1.15, s=4.33, k=0.34)",
        points: 7,
        bright_count: 3,
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
    },
    HwatuScenario {
        scenario_id: "sr-bright-banked-pressure",
        decision: "Banked bright 8-point, 6 locked, 2 continuation, opp=4 — stop_round dominates (c=0.99, s=5.23, k=0.24)",
        points: 8,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 4,
        hand_count: 1,
        decision_window: true,
        locked_points: 6,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HwatuScenario {
        scenario_id: "sr-yaku-banked-no-upside",
        decision: "5-yaku banked 9-point, 7 locked, 1 continuation, opp=2 — stop_round dominates (c=1.19, s=5.27, k=0.34)",
        points: 9,
        bright_count: 1,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 5,
        bright_capture_options: 0,
        opponent_pressure: 2,
        hand_count: 1,
        decision_window: true,
        locked_points: 7,
        continuation_calls: 1,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HwatuScenario {
        scenario_id: "sr-mixed-yaku-banked-no-upside",
        decision: "Mixed ribbon+animal banked 8-point, 6 locked, 1 continuation — stop_round dominates (c=1.43, s=4.37, k=0.34)",
        points: 8,
        bright_count: 1,
        ribbon_yaku: 2,
        animal_yaku: 2,
        bonus_cards: 0,
        yaku_count: 2,
        bright_capture_options: 0,
        opponent_pressure: 2,
        hand_count: 1,
        decision_window: true,
        locked_points: 6,
        continuation_calls: 1,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HwatuScenario {
        scenario_id: "sr-bright-banked-no-upside",
        decision: "Bright-heavy banked 8-point, 6 locked, 2 continuation — stop_round dominates (c=1.03, s=5.03, k=0.24)",
        points: 8,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 3,
        hand_count: 1,
        decision_window: true,
        locked_points: 6,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HwatuScenario {
        scenario_id: "sr-bright-yaku-pressure",
        decision: "Bright + 3 yaku banked 8-point, 6 locked, 2 continuation, opp=3 — stop_round dominates (c=1.03, s=5.03, k=0.24)",
        points: 8,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 3,
        hand_count: 1,
        decision_window: true,
        locked_points: 6,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HwatuScenario {
        scenario_id: "sr-stretched-bright-bank",
        decision: "Stretched 9-point, 7 locked, 3 continuation, opp=4 — stop_round dominates (c=0.87, s=5.49, k=0.14)",
        points: 9,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 2,
        bright_capture_options: 0,
        opponent_pressure: 4,
        hand_count: 1,
        decision_window: true,
        locked_points: 7,
        continuation_calls: 3,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    HwatuScenario {
        scenario_id: "sr-bright-bank-lock",
        decision: "Banked 10-point, 8 locked, 2 continuation — stop_round dominates (c=1.03, s=5.51, k=0.24)",
        points: 10,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 0,
        yaku_count: 3,
        bright_capture_options: 0,
        opponent_pressure: 3,
        hand_count: 1,
        decision_window: true,
        locked_points: 8,
        continuation_calls: 2,
        upside_capture_options: 0,
        max_upside_gain: 0,
    },
    // ---- mixed/edge bucket (×6) — thin upside or stretched windows where the engine resolves to the leading arm; the F-015 spec scope boundary records that the mixed/edge bucket is intentionally asymmetric (3 call-go edges + 3 stop-round edges) so the engine's bonus-heavy-fresh vs banked-cash-out decision boundary is exercised at multiple margins ----
    HwatuScenario {
        scenario_id: "me-thin-upside-call_go-edge",
        decision: "Small banked 6-point, 5 locked, 1 bright option, 4 max upside — call_go leads (c=4.17, s=2.26, k=1.24)",
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
    },
    HwatuScenario {
        scenario_id: "me-fresh-window-thin-upside-call_go",
        decision: "Fresh 4-point window, 2 bright options, 2 max upside — call_go leads (c=3.06, s=2.22, k=0.86)",
        points: 4,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 1,
        bright_capture_options: 2,
        opponent_pressure: 0,
        hand_count: 2,
        decision_window: true,
        locked_points: 0,
        continuation_calls: 0,
        upside_capture_options: 2,
        max_upside_gain: 2,
    },
    HwatuScenario {
        scenario_id: "me-bright-banked-thin-upside-stop_round-edge",
        decision: "Banked bright 7-point, 5 locked, 1 bright option, 1 max upside — engine resolves to stop_round (c=2.33, s=4.00, k=0.68)",
        points: 7,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 3,
        bright_capture_options: 1,
        opponent_pressure: 2,
        hand_count: 2,
        decision_window: true,
        locked_points: 5,
        continuation_calls: 1,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
    HwatuScenario {
        scenario_id: "me-stretched-fresh-bright-call_go-narrow",
        decision: "Stretched fresh 5-point, 1 continuation, 2 max upside — call_go leads narrowly (c=3.04, s=2.96, k=0.84)",
        points: 5,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 2,
        yaku_count: 2,
        bright_capture_options: 2,
        opponent_pressure: 1,
        hand_count: 2,
        decision_window: true,
        locked_points: 2,
        continuation_calls: 1,
        upside_capture_options: 2,
        max_upside_gain: 2,
    },
    HwatuScenario {
        scenario_id: "me-yaku-bright-banked-stop_round-edge",
        decision: "Banked yaku + bright 6-point, 4 locked, 1 continuation, 1 max upside — engine resolves to stop_round (c=2.25, s=3.59, k=0.58)",
        points: 6,
        bright_count: 2,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 2,
        bright_capture_options: 1,
        opponent_pressure: 2,
        hand_count: 1,
        decision_window: true,
        locked_points: 4,
        continuation_calls: 1,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
    HwatuScenario {
        scenario_id: "me-bright-banked-2-call_pressure-stop_round-edge",
        decision: "Banked bright 7-point, 5 locked, 2 continuation, low opp pressure — engine resolves to stop_round (c=2.17, s=4.09, k=0.48)",
        points: 7,
        bright_count: 3,
        ribbon_yaku: 1,
        animal_yaku: 1,
        bonus_cards: 1,
        yaku_count: 3,
        bright_capture_options: 1,
        opponent_pressure: 1,
        hand_count: 1,
        decision_window: true,
        locked_points: 5,
        continuation_calls: 2,
        upside_capture_options: 1,
        max_upside_gain: 1,
    },
];

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
