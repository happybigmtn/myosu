use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use myosu_games::{
    CanonicalStateSnapshot, CanonicalStrategyBinding, CanonicalTransitionTrace,
    CanonicalTruthError, canonical_hash,
};
use myosu_games_portfolio::core::{
    CoreAction, CoreGameError, CoreGameState, apply_action, bootstrap_state,
};
use myosu_games_portfolio::{
    ALL_RESEARCH_GAMES, PortfolioAction, PortfolioChallenge, ResearchGame, answer_game,
    answer_typed_challenge, recommended_action,
};
use serde_json::{Value, json};

use crate::CANONICAL_TEN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaytracePolicy {
    BestLocal,
    LegalFirst,
    RandomLegal,
    ForcedIllegal,
}

impl PlaytracePolicy {
    pub fn parse(value: &str) -> Result<Self, PlaytraceError> {
        match value {
            "best-local" => Ok(Self::BestLocal),
            "legal-first" => Ok(Self::LegalFirst),
            "random-legal" => Ok(Self::RandomLegal),
            "forced-illegal" => Ok(Self::ForcedIllegal),
            _ => Err(PlaytraceError::InvalidPolicy {
                policy: value.to_string(),
            }),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::BestLocal => "best-local",
            Self::LegalFirst => "legal-first",
            Self::RandomLegal => "random-legal",
            Self::ForcedIllegal => "forced-illegal",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaytraceRequest {
    pub game: ResearchGame,
    pub max_steps: usize,
    pub seed: u64,
    pub policy: PlaytracePolicy,
}

impl PlaytraceRequest {
    pub const fn new(game: ResearchGame) -> Self {
        Self {
            game,
            max_steps: 200,
            seed: 1,
            policy: PlaytracePolicy::BestLocal,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlaytraceReport {
    pub game: ResearchGame,
    pub status: String,
    pub steps: usize,
    pub strategy_source: String,
    pub terminal: bool,
    pub payoff: Option<Vec<i64>>,
    pub truth_hash: String,
    pub traces: Vec<CanonicalTransitionTrace>,
}

#[derive(Debug, PartialEq)]
pub enum PlaytraceError {
    UnsupportedGame {
        game: ResearchGame,
    },
    InvalidPolicy {
        policy: String,
    },
    NoLegalActions {
        game: ResearchGame,
    },
    IllegalRecommendedAction {
        game: ResearchGame,
        action_id: String,
    },
    Core(CoreGameError),
    Canonical(CanonicalTruthError),
}

impl fmt::Display for PlaytraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedGame { game } => {
                write!(formatter, "{} does not have a playtrace core", game.slug())
            }
            Self::InvalidPolicy { policy } => {
                write!(formatter, "invalid playtrace policy {policy}")
            }
            Self::NoLegalActions { game } => {
                write!(formatter, "{} has no legal action to continue", game.slug())
            }
            Self::IllegalRecommendedAction { game, action_id } => write!(
                formatter,
                "{} recommended illegal action {}",
                game.slug(),
                action_id
            ),
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Canonical(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for PlaytraceError {}

impl From<CoreGameError> for PlaytraceError {
    fn from(error: CoreGameError) -> Self {
        Self::Core(error)
    }
}

impl From<CanonicalTruthError> for PlaytraceError {
    fn from(error: CanonicalTruthError) -> Self {
        Self::Canonical(error)
    }
}

#[derive(Clone, Debug)]
struct PlaytraceDecision {
    action: CoreAction,
    strategy_source: String,
}

pub fn run_playtrace(request: PlaytraceRequest) -> Result<PlaytraceReport, PlaytraceError> {
    let mut state = match bootstrap_state(request.game) {
        Ok(state) => state,
        Err(CoreGameError::UnsupportedGame { .. }) => {
            return Err(PlaytraceError::UnsupportedGame { game: request.game });
        }
        Err(error) => return Err(error.into()),
    };
    let mut rng = request.seed;
    let mut traces = Vec::new();
    let mut sources = BTreeSet::new();

    for step in 0..request.max_steps {
        if state.terminal {
            break;
        }
        if state.legal_actions.is_empty() {
            break;
        }
        let before_snapshot = CanonicalStateSnapshot::from(state.clone());
        let decision = select_action(request.policy, request.game, &state, &mut rng)?;
        sources.insert(decision.strategy_source.clone());
        if !state
            .legal_actions
            .iter()
            .any(|action| action.action_id == decision.action.action_id)
        {
            return Err(PlaytraceError::IllegalRecommendedAction {
                game: request.game,
                action_id: decision.action.action_id,
            });
        }

        let transition = apply_action(&state, &decision.action.action_id, json!({}))?;
        let after_snapshot = CanonicalStateSnapshot::from(transition.after.clone());
        let trace = CanonicalTransitionTrace {
            trace_id: format!("{}:playtrace:{step}", request.game.chain_id()),
            game_id: request.game.slug().to_string(),
            ruleset_version: 1,
            state_hash_before: canonical_hash(&before_snapshot)?,
            action_id: decision.action.action_id.clone(),
            action_params: decision.action.params.clone(),
            state_hash_after: canonical_hash(&after_snapshot)?,
            strategy_binding: local_strategy_binding(
                &before_snapshot,
                &decision.action,
                &decision.strategy_source,
                step,
            )?,
            payoff: transition.after.payoff.clone(),
        };
        validate_transition_trace(&trace, &before_snapshot, &after_snapshot)?;
        traces.push(trace);
        state = transition.after;
    }

    let status = if state.terminal {
        "terminal"
    } else {
        "bounded"
    }
    .to_string();
    let strategy_source = source_summary(&sources, request.policy);
    let truth_hash = canonical_hash(&traces)?;

    Ok(PlaytraceReport {
        game: request.game,
        status,
        steps: traces.len(),
        strategy_source,
        terminal: state.terminal,
        payoff: state.payoff,
        truth_hash,
        traces,
    })
}

pub fn validate_transition_trace(
    trace: &CanonicalTransitionTrace,
    before: &CanonicalStateSnapshot,
    after: &CanonicalStateSnapshot,
) -> Result<(), CanonicalTruthError> {
    let before_hash = canonical_hash(before)?;
    if before_hash != trace.state_hash_before {
        return Err(CanonicalTruthError::HashMismatch {
            expected: trace.state_hash_before.clone(),
            found: before_hash,
        });
    }
    let after_hash = canonical_hash(after)?;
    if after_hash != trace.state_hash_after {
        return Err(CanonicalTruthError::HashMismatch {
            expected: trace.state_hash_after.clone(),
            found: after_hash,
        });
    }

    Ok(())
}

fn select_action(
    policy: PlaytracePolicy,
    game: ResearchGame,
    state: &CoreGameState,
    rng: &mut u64,
) -> Result<PlaytraceDecision, PlaytraceError> {
    let legal_actions = &state.legal_actions;
    match policy {
        PlaytracePolicy::BestLocal => {
            if let Some(action) = best_local_core_action(game, state) {
                return Ok(PlaytraceDecision {
                    action,
                    strategy_source: "best-local".to_string(),
                });
            }
            let action = legal_actions
                .first()
                .cloned()
                .ok_or(PlaytraceError::NoLegalActions { game })?;
            Ok(PlaytraceDecision {
                action,
                strategy_source: "best-local+legal-continuation".to_string(),
            })
        }
        PlaytracePolicy::LegalFirst => {
            let action = legal_actions
                .first()
                .cloned()
                .ok_or(PlaytraceError::NoLegalActions { game })?;
            Ok(PlaytraceDecision {
                action,
                strategy_source: "legal-first".to_string(),
            })
        }
        PlaytracePolicy::RandomLegal => {
            let action = random_legal_action(legal_actions, rng)
                .ok_or(PlaytraceError::NoLegalActions { game })?;
            Ok(PlaytraceDecision {
                action,
                strategy_source: "random-legal".to_string(),
            })
        }
        PlaytracePolicy::ForcedIllegal => Ok(PlaytraceDecision {
            action: CoreAction {
                action_id: format!("{}.forced-illegal", game.slug()),
                display_label: "forced-illegal".to_string(),
                params: json!({}),
            },
            strategy_source: "forced-illegal".to_string(),
        }),
    }
}

fn best_local_core_action(game: ResearchGame, state: &CoreGameState) -> Option<CoreAction> {
    let response = PortfolioChallenge::from_core_state(state)
        .and_then(|challenge| {
            answer_typed_challenge(&challenge, 0)
                .ok()
                .map(|answer| answer.response)
        })
        .unwrap_or_else(|| answer_game(game, 0).response);
    let action = recommended_action(&response)?;
    semantic_core_action(game, action, state).or_else(|| {
        let hint = portfolio_action_hint(game, action)?;
        state
            .legal_actions
            .iter()
            .find(|legal| legal.action_id.contains(hint) || legal.display_label.contains(hint))
            .cloned()
    })
}

fn portfolio_action_hint(game: ResearchGame, action: PortfolioAction) -> Option<&'static str> {
    match game {
        ResearchGame::NlheHeadsUp
        | ResearchGame::NlheSixMax
        | ResearchGame::Plo
        | ResearchGame::NlheTournament
        | ResearchGame::ShortDeck => match action {
            PortfolioAction::IcmFold => Some("fold"),
            PortfolioAction::PotControl
            | PortfolioAction::DefendBlind
            | PortfolioAction::DrawToNuts => Some("call"),
            PortfolioAction::ValueBet
            | PortfolioAction::TightOpen
            | PortfolioAction::PotSizedRaise
            | PortfolioAction::AnteSteal
            | PortfolioAction::PushFold => Some("raise-to"),
            _ => None,
        },
        ResearchGame::TeenPatti => match action {
            PortfolioAction::SeeCards => Some("see-cards"),
            PortfolioAction::PotControl => Some("call"),
            PortfolioAction::ValueBet => Some("raise-to"),
            _ => None,
        },
        ResearchGame::HanafudaKoiKoi => match action {
            PortfolioAction::KoiKoi => Some("koi-koi"),
            PortfolioAction::StopRound => Some("stop-round"),
            _ => None,
        },
        ResearchGame::HwatuGoStop => match action {
            PortfolioAction::CallGo => Some("call-go"),
            PortfolioAction::KoiKoi => Some("koi-koi"),
            PortfolioAction::StopRound => Some("stop-round"),
            _ => None,
        },
        ResearchGame::RiichiMahjong => match action {
            PortfolioAction::DeclareRiichi => Some("declare-riichi"),
            PortfolioAction::FoldDanger => Some("east"),
            _ => None,
        },
        ResearchGame::Bridge
        | ResearchGame::Spades
        | ResearchGame::CallBreak
        | ResearchGame::Hearts => Some(".play."),
        ResearchGame::GinRummy => match action {
            PortfolioAction::Knock => Some("knock"),
            PortfolioAction::DiscardDeadwood => Some("discard"),
            _ => None,
        },
        ResearchGame::Stratego => Some("stratego.move"),
        ResearchGame::OfcChinesePoker => Some("place."),
        ResearchGame::LiarsDice => match action {
            PortfolioAction::Challenge => Some("challenge"),
            _ => Some("bid."),
        },
        ResearchGame::DouDiZhu | ResearchGame::PusoyDos | ResearchGame::TienLen => match action {
            PortfolioAction::PassControl => Some("pass"),
            _ => Some("play."),
        },
        ResearchGame::Backgammon => match action {
            PortfolioAction::BearOff => Some("-off"),
            _ => Some("move."),
        },
        ResearchGame::Cribbage => Some("play-"),
    }
}

fn semantic_core_action(
    game: ResearchGame,
    action: PortfolioAction,
    state: &CoreGameState,
) -> Option<CoreAction> {
    let mut best: Option<(i32, usize, CoreAction)> = None;
    for (index, legal) in state.legal_actions.iter().enumerate() {
        let Some(score) = semantic_match_score(game, action, legal, state) else {
            continue;
        };
        let replace = match &best {
            Some((best_score, best_index, _)) => {
                score > *best_score || (score == *best_score && index < *best_index)
            }
            None => true,
        };
        if replace {
            best = Some((score, index, legal.clone()));
        }
    }

    best.map(|(_, _, legal)| legal)
}

fn semantic_match_score(
    game: ResearchGame,
    action: PortfolioAction,
    legal: &CoreAction,
    state: &CoreGameState,
) -> Option<i32> {
    match game {
        ResearchGame::NlheHeadsUp
        | ResearchGame::NlheSixMax
        | ResearchGame::Plo
        | ResearchGame::NlheTournament
        | ResearchGame::ShortDeck
        | ResearchGame::TeenPatti => score_poker_like(action, legal),
        ResearchGame::HanafudaKoiKoi | ResearchGame::HwatuGoStop => score_hanafuda(action, legal),
        ResearchGame::RiichiMahjong => score_mahjong(action, legal, state),
        ResearchGame::Bridge
        | ResearchGame::Spades
        | ResearchGame::CallBreak
        | ResearchGame::Hearts => score_trick_taking(action, legal, state),
        ResearchGame::GinRummy => score_gin_rummy(action, legal),
        ResearchGame::Stratego => score_stratego(action, legal, state),
        ResearchGame::OfcChinesePoker => score_ofc(action, legal),
        ResearchGame::LiarsDice => score_liars_dice(action, legal),
        ResearchGame::DouDiZhu | ResearchGame::PusoyDos | ResearchGame::TienLen => {
            score_shedding(action, legal, state)
        }
        ResearchGame::Backgammon => score_backgammon(action, legal),
        ResearchGame::Cribbage => score_cribbage(action, legal, state),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PokerLikeCoreKind {
    Fold,
    Call,
    Check,
    Raise,
    SeeCards,
}

fn score_poker_like(action: PortfolioAction, legal: &CoreAction) -> Option<i32> {
    let kind = poker_like_kind(legal)?;
    Some(match action {
        PortfolioAction::IcmFold => match kind {
            PokerLikeCoreKind::Fold => 140,
            PokerLikeCoreKind::Call => 60,
            PokerLikeCoreKind::Check => 40,
            PokerLikeCoreKind::Raise => 15,
            PokerLikeCoreKind::SeeCards => 10,
        },
        PortfolioAction::PotControl | PortfolioAction::DefendBlind => match kind {
            PokerLikeCoreKind::Call => 140,
            PokerLikeCoreKind::Check => 130,
            PokerLikeCoreKind::Fold => 60,
            PokerLikeCoreKind::Raise => 20,
            PokerLikeCoreKind::SeeCards => 90,
        },
        PortfolioAction::DrawToNuts => match kind {
            PokerLikeCoreKind::Raise => 150,
            PokerLikeCoreKind::Call => 130,
            PokerLikeCoreKind::Check => 110,
            PokerLikeCoreKind::SeeCards => 70,
            PokerLikeCoreKind::Fold => 5,
        },
        PortfolioAction::ValueBet
        | PortfolioAction::TightOpen
        | PortfolioAction::PotSizedRaise
        | PortfolioAction::AnteSteal => match kind {
            PokerLikeCoreKind::Raise => 150,
            PokerLikeCoreKind::Check => 80,
            PokerLikeCoreKind::Call => 70,
            PokerLikeCoreKind::SeeCards => 60,
            PokerLikeCoreKind::Fold => 10,
        },
        PortfolioAction::PushFold => match kind {
            PokerLikeCoreKind::Raise => 150,
            PokerLikeCoreKind::Fold => 140,
            PokerLikeCoreKind::Call => 50,
            PokerLikeCoreKind::Check => 20,
            PokerLikeCoreKind::SeeCards => 10,
        },
        PortfolioAction::SeeCards => match kind {
            PokerLikeCoreKind::SeeCards => 160,
            PokerLikeCoreKind::Raise => 110,
            PokerLikeCoreKind::Call => 100,
            PokerLikeCoreKind::Check => 90,
            PokerLikeCoreKind::Fold => 40,
        },
        _ => match kind {
            PokerLikeCoreKind::Raise => 100,
            PokerLikeCoreKind::Call => 90,
            PokerLikeCoreKind::Check => 80,
            PokerLikeCoreKind::SeeCards => 70,
            PokerLikeCoreKind::Fold => 20,
        },
    })
}

fn poker_like_kind(legal: &CoreAction) -> Option<PokerLikeCoreKind> {
    if legal.action_id.ends_with(".fold") {
        return Some(PokerLikeCoreKind::Fold);
    }
    if legal.action_id.ends_with(".call") {
        return Some(PokerLikeCoreKind::Call);
    }
    if legal.action_id.ends_with(".check") {
        return Some(PokerLikeCoreKind::Check);
    }
    if legal.action_id.contains(".raise-to.") {
        return Some(PokerLikeCoreKind::Raise);
    }
    if legal.action_id.ends_with(".see-cards") {
        return Some(PokerLikeCoreKind::SeeCards);
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HanafudaCoreKind {
    Capture,
    Discard,
    KoiKoi,
    CallGo,
    StopRound,
}

fn score_hanafuda(action: PortfolioAction, legal: &CoreAction) -> Option<i32> {
    let kind = hanafuda_kind(legal)?;
    let card_bonus = hanafuda_card_bonus(legal);
    Some(match action {
        PortfolioAction::StopRound => match kind {
            HanafudaCoreKind::StopRound => 160,
            HanafudaCoreKind::Capture => score_add(110, card_bonus),
            HanafudaCoreKind::Discard => score_add(60, card_bonus),
            HanafudaCoreKind::CallGo | HanafudaCoreKind::KoiKoi => 20,
        },
        PortfolioAction::KoiKoi => match kind {
            HanafudaCoreKind::KoiKoi => 160,
            HanafudaCoreKind::CallGo => 150,
            HanafudaCoreKind::Capture => score_add(110, card_bonus),
            HanafudaCoreKind::Discard => score_add(60, card_bonus),
            HanafudaCoreKind::StopRound => 20,
        },
        PortfolioAction::CallGo => match kind {
            HanafudaCoreKind::CallGo => 160,
            HanafudaCoreKind::KoiKoi => 150,
            HanafudaCoreKind::Capture => score_add(110, card_bonus),
            HanafudaCoreKind::Discard => score_add(60, card_bonus),
            HanafudaCoreKind::StopRound => 20,
        },
        _ => match kind {
            HanafudaCoreKind::Capture => score_add(120, card_bonus),
            HanafudaCoreKind::Discard => score_add(70, card_bonus),
            HanafudaCoreKind::StopRound => 60,
            HanafudaCoreKind::CallGo | HanafudaCoreKind::KoiKoi => 50,
        },
    })
}

fn hanafuda_kind(legal: &CoreAction) -> Option<HanafudaCoreKind> {
    if legal.action_id.contains(".capture.") {
        return Some(HanafudaCoreKind::Capture);
    }
    if legal.action_id.contains(".discard.") {
        return Some(HanafudaCoreKind::Discard);
    }
    if legal.action_id.ends_with(".koi-koi") {
        return Some(HanafudaCoreKind::KoiKoi);
    }
    if legal.action_id.ends_with(".call-go") {
        return Some(HanafudaCoreKind::CallGo);
    }
    if legal.action_id.ends_with(".stop-round") {
        return Some(HanafudaCoreKind::StopRound);
    }

    None
}

fn hanafuda_card_bonus(legal: &CoreAction) -> i32 {
    match param_str(legal, "card", "kind") {
        Some("bright") => 30,
        Some("animal") => 20,
        Some("ribbon") => 10,
        Some("chaff") => 0,
        _ => 0,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MahjongCoreKind {
    DeclareRiichi,
    Discard(String),
}

fn score_mahjong(
    action: PortfolioAction,
    legal: &CoreAction,
    state: &CoreGameState,
) -> Option<i32> {
    let kind = mahjong_kind(legal)?;
    let dangerous = string_array_set(&state.public_state, "danger_discards");
    Some(match action {
        PortfolioAction::DeclareRiichi => match kind {
            MahjongCoreKind::DeclareRiichi => 160,
            MahjongCoreKind::Discard(tile) => score_add(90, safe_tile_bonus(&dangerous, &tile)),
        },
        PortfolioAction::FoldDanger => match kind {
            MahjongCoreKind::DeclareRiichi => 10,
            MahjongCoreKind::Discard(tile) => score_add(140, safe_tile_bonus(&dangerous, &tile)),
        },
        _ => match kind {
            MahjongCoreKind::DeclareRiichi => 120,
            MahjongCoreKind::Discard(tile) => score_add(100, safe_tile_bonus(&dangerous, &tile)),
        },
    })
}

fn mahjong_kind(legal: &CoreAction) -> Option<MahjongCoreKind> {
    if legal.action_id == "riichi-mahjong.declare-riichi" {
        return Some(MahjongCoreKind::DeclareRiichi);
    }
    legal
        .action_id
        .strip_prefix("riichi-mahjong.discard.")
        .map(|tile| MahjongCoreKind::Discard(tile.to_string()))
}

fn safe_tile_bonus(dangerous: &[String], tile: &str) -> i32 {
    if dangerous.iter().any(|candidate| candidate == tile) {
        0
    } else {
        20
    }
}

fn score_trick_taking(
    action: PortfolioAction,
    legal: &CoreAction,
    state: &CoreGameState,
) -> Option<i32> {
    let rank = card_rank_score(legal)?;
    let suit = param_str(legal, "card", "suit").unwrap_or_default();
    let led_suit = state.public_state.get("led_suit").and_then(Value::as_str);
    let trump = state.public_state.get("trump").and_then(Value::as_str);
    let is_led = led_suit.is_some_and(|expected| expected == suit);
    let is_trump = trump.is_some_and(|expected| expected == suit);
    let penalty = if legal.action_id.contains("queen-spades") {
        20
    } else if suit == "hearts" {
        10
    } else {
        0
    };
    Some(match action {
        PortfolioAction::FollowSuit => score_sub(score_add(110, if is_led { 20 } else { 0 }), rank),
        PortfolioAction::DoubleDummyPlay | PortfolioAction::BidContract => score_add(
            score_add(100, rank),
            if is_trump {
                20
            } else if is_led {
                10
            } else {
                0
            },
        ),
        PortfolioAction::TrumpControl | PortfolioAction::CallTrump => {
            score_add(score_add(100, if is_trump { 40 } else { 0 }), rank)
        }
        PortfolioAction::BidNil | PortfolioAction::AvoidPenalty => score_sub(
            score_sub(120, if is_trump { 20 } else { 0 }),
            score_add(rank, penalty),
        ),
        PortfolioAction::ShootMoon => {
            score_add(score_add(100, if suit == "hearts" { 20 } else { 0 }), rank)
        }
        _ => score_add(100, rank),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GinCoreKind {
    DrawStock,
    DrawDiscard,
    Discard,
    Knock,
    Gin,
}

fn score_gin_rummy(action: PortfolioAction, legal: &CoreAction) -> Option<i32> {
    let kind = gin_kind(legal)?;
    let discard_bonus = card_rank_score(legal).unwrap_or_default();
    Some(match action {
        PortfolioAction::Knock => match kind {
            GinCoreKind::Knock => 160,
            GinCoreKind::Gin => 150,
            GinCoreKind::Discard => score_add(80, discard_bonus),
            GinCoreKind::DrawDiscard => 70,
            GinCoreKind::DrawStock => 60,
        },
        PortfolioAction::DiscardDeadwood => match kind {
            GinCoreKind::Discard => score_add(140, discard_bonus),
            GinCoreKind::DrawDiscard => 110,
            GinCoreKind::DrawStock => 100,
            GinCoreKind::Knock => 80,
            GinCoreKind::Gin => 85,
        },
        PortfolioAction::PotControl => match kind {
            GinCoreKind::DrawStock => 130,
            GinCoreKind::DrawDiscard => 110,
            GinCoreKind::Discard => score_sub(90, discard_bonus),
            GinCoreKind::Knock => 70,
            GinCoreKind::Gin => 75,
        },
        _ => match kind {
            GinCoreKind::Discard => score_add(120, discard_bonus),
            GinCoreKind::DrawDiscard => 100,
            GinCoreKind::DrawStock => 90,
            GinCoreKind::Knock => 80,
            GinCoreKind::Gin => 85,
        },
    })
}

fn gin_kind(legal: &CoreAction) -> Option<GinCoreKind> {
    match legal.action_id.as_str() {
        "gin-rummy.draw.stock" => Some(GinCoreKind::DrawStock),
        "gin-rummy.draw.discard" => Some(GinCoreKind::DrawDiscard),
        "gin-rummy.knock" => Some(GinCoreKind::Knock),
        "gin-rummy.gin" => Some(GinCoreKind::Gin),
        _ if legal.action_id.starts_with("gin-rummy.discard.") => Some(GinCoreKind::Discard),
        _ => None,
    }
}

fn score_stratego(
    action: PortfolioAction,
    legal: &CoreAction,
    state: &CoreGameState,
) -> Option<i32> {
    let distance = stratego_move_distance(legal)?;
    let attack = stratego_attack_bonus(legal, state);
    Some(match action {
        PortfolioAction::Scout => score_add(score_add(100, attack), score_mul(distance, 20)),
        PortfolioAction::AdvancePiece => {
            score_add(score_add(120, attack), if distance == 1 { 20 } else { 0 })
        }
        PortfolioAction::PlaceSafe => {
            score_add(score_sub(120, attack), if distance == 1 { 10 } else { 0 })
        }
        _ => score_add(score_add(100, attack), distance),
    })
}

fn stratego_move_distance(legal: &CoreAction) -> Option<i32> {
    let from_x = param_u64(legal, "from", "x")?;
    let from_y = param_u64(legal, "from", "y")?;
    let to_x = param_u64(legal, "to", "x")?;
    let to_y = param_u64(legal, "to", "y")?;
    let dx = from_x.abs_diff(to_x);
    let dy = from_y.abs_diff(to_y);

    i32::try_from(dx.saturating_add(dy)).ok()
}

fn stratego_attack_bonus(legal: &CoreAction, state: &CoreGameState) -> i32 {
    let Some(to_x) = param_u64(legal, "to", "x") else {
        return 0;
    };
    let Some(to_y) = param_u64(legal, "to", "y") else {
        return 0;
    };
    let Some(opponents) = state
        .public_state
        .get("opponent_pieces")
        .and_then(Value::as_array)
    else {
        return 0;
    };
    if opponents.iter().any(|piece| {
        piece
            .get("position")
            .and_then(Value::as_object)
            .is_some_and(|position| {
                position
                    .get("x")
                    .and_then(Value::as_u64)
                    .is_some_and(|x| x == to_x)
                    && position
                        .get("y")
                        .and_then(Value::as_u64)
                        .is_some_and(|y| y == to_y)
            })
    }) {
        20
    } else {
        0
    }
}

fn score_ofc(action: PortfolioAction, legal: &CoreAction) -> Option<i32> {
    let row = param_str(legal, "row", "")?;
    let rank = card_rank_score(legal).unwrap_or_default();
    Some(match action {
        PortfolioAction::PlaceSafe => match row {
            "front" => score_sub(140, rank),
            "middle" => 120,
            "back" => score_add(110, rank / 2),
            _ => 100,
        },
        PortfolioAction::DrawToNuts => match row {
            "back" => score_add(140, rank),
            "middle" => score_add(120, rank / 2),
            "front" => 80,
            _ => 100,
        },
        PortfolioAction::PotControl => match row {
            "middle" => 140,
            "back" => 120,
            "front" => 90,
            _ => 100,
        },
        _ => score_add(100, rank),
    })
}

fn score_liars_dice(action: PortfolioAction, legal: &CoreAction) -> Option<i32> {
    if legal.action_id == "liars-dice.challenge" {
        return Some(match action {
            PortfolioAction::Challenge => 160,
            _ => 40,
        });
    }
    let count = param_u64(legal, "count", "")? as i32;
    let face = param_u64(legal, "face", "")? as i32;
    let size_bonus = score_add(score_mul(count, 10), face);
    Some(match action {
        PortfolioAction::Challenge => score_add(20, score_add(count, face)),
        PortfolioAction::ValueBet => score_add(120, size_bonus),
        PortfolioAction::PotControl => score_sub(120, size_bonus),
        _ => score_add(100, size_bonus),
    })
}

fn score_shedding(
    action: PortfolioAction,
    legal: &CoreAction,
    state: &CoreGameState,
) -> Option<i32> {
    let current_lead = state
        .public_state
        .get("current_lead")
        .is_some_and(|lead| !lead.is_null());
    if legal.display_label == "pass" {
        return Some(match action {
            PortfolioAction::PassControl => {
                if current_lead {
                    160
                } else {
                    40
                }
            }
            PortfolioAction::PreserveBomb => {
                if current_lead {
                    150
                } else {
                    50
                }
            }
            PortfolioAction::ShedLowest => 20,
            PortfolioAction::LeadControl | PortfolioAction::LandlordBid => {
                if current_lead {
                    30
                } else {
                    10
                }
            }
            _ => 30,
        });
    }
    let class = param_str(legal, "class", "")?;
    let rank = i32::try_from(param_u64(legal, "rank", "")?).ok()?;
    let strength = shedding_strength(class, rank);
    Some(match action {
        PortfolioAction::PassControl => {
            if current_lead {
                score_sub(80, strength)
            } else {
                score_add(120, strength)
            }
        }
        PortfolioAction::PreserveBomb => {
            if class == "bomb" {
                10
            } else if current_lead {
                score_sub(120, strength)
            } else {
                score_sub(130, strength)
            }
        }
        PortfolioAction::ShedLowest => score_sub(160, strength),
        PortfolioAction::LeadControl | PortfolioAction::LandlordBid => score_add(120, strength),
        _ => score_add(100, strength),
    })
}

fn shedding_strength(class: &str, rank: i32) -> i32 {
    let class_weight = match class {
        "single" => 0,
        "pair" => 20,
        "straight" => 40,
        "bomb" => 100,
        _ => 0,
    };

    score_add(class_weight, rank)
}

fn score_backgammon(action: PortfolioAction, legal: &CoreAction) -> Option<i32> {
    if legal.action_id == "backgammon.take-double" {
        return Some(match action {
            PortfolioAction::AcceptDouble => 180,
            PortfolioAction::BearOff => 130,
            PortfolioAction::AdvancePiece => 120,
            _ => 80,
        });
    }
    if legal.action_id == "backgammon.drop-double" {
        return Some(match action {
            PortfolioAction::AcceptDouble => 20,
            PortfolioAction::BearOff | PortfolioAction::AdvancePiece => 170,
            _ => 120,
        });
    }

    let source = param_str(legal, "source", "")?;
    let dest = param_str(legal, "dest", "")?;
    let source_score = source.parse::<i32>().unwrap_or_default();
    let dest_score = if dest == "off" {
        25
    } else {
        dest.parse::<i32>().unwrap_or_default()
    };
    Some(match action {
        PortfolioAction::BearOff => {
            if dest == "off" {
                170
            } else if source == "bar" {
                160
            } else {
                score_add(score_add(120, dest_score), source_score / 2)
            }
        }
        PortfolioAction::AdvancePiece | PortfolioAction::AcceptDouble => {
            if source == "bar" {
                170
            } else if dest == "off" {
                160
            } else {
                score_add(120, dest_score)
            }
        }
        _ => score_add(100, dest_score),
    })
}

fn score_cribbage(
    action: PortfolioAction,
    legal: &CoreAction,
    state: &CoreGameState,
) -> Option<i32> {
    let running_count = state
        .public_state
        .get("running_count")
        .and_then(Value::as_u64)
        .and_then(|count| i32::try_from(count).ok())
        .unwrap_or_default();
    let card_value = cribbage_card_value(legal)?;
    let immediate = cribbage_immediate_score(legal, state);
    let distance_to_31 =
        i32::try_from(31_i32.abs_diff(score_add(running_count, card_value))).unwrap_or(i32::MAX);
    Some(match action {
        PortfolioAction::PegRun => {
            score_sub(score_add(120, score_mul(immediate, 20)), distance_to_31)
        }
        PortfolioAction::KeepCrib => score_sub(120, card_value),
        PortfolioAction::DiscardDeadwood => score_add(120, card_value),
        _ => score_add(score_add(100, score_mul(immediate, 20)), card_value),
    })
}

fn cribbage_card_value(legal: &CoreAction) -> Option<i32> {
    Some(match param_str(legal, "card", "rank")? {
        "ace" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" | "jack" | "queen" | "king" => 10,
        _ => return None,
    })
}

fn cribbage_immediate_score(legal: &CoreAction, state: &CoreGameState) -> i32 {
    let Some(card_value) = cribbage_card_value(legal) else {
        return 0;
    };
    let running_count = state
        .public_state
        .get("running_count")
        .and_then(Value::as_u64)
        .and_then(|count| i32::try_from(count).ok())
        .unwrap_or_default();
    let next = score_add(running_count, card_value);
    let mut score = 0;
    if next == 15 || next == 31 {
        score = score_add(score, 2);
    }
    score
}

fn card_rank_score(legal: &CoreAction) -> Option<i32> {
    Some(match param_str(legal, "card", "rank")? {
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "six" => 6,
        "seven" => 7,
        "eight" => 8,
        "nine" => 9,
        "ten" => 10,
        "jack" => 11,
        "queen" => 12,
        "king" => 13,
        "ace" => 14,
        _ => return None,
    })
}

fn string_array_set(state: &Value, key: &str) -> Vec<String> {
    state
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn score_add(left: i32, right: i32) -> i32 {
    left.saturating_add(right)
}

fn score_sub(left: i32, right: i32) -> i32 {
    left.saturating_sub(right)
}

fn score_mul(left: i32, right: i32) -> i32 {
    left.saturating_mul(right)
}

fn param_str<'a>(action: &'a CoreAction, key: &str, nested: &str) -> Option<&'a str> {
    let value = action.params.get(key)?;
    if nested.is_empty() {
        return value.as_str();
    }
    value.get(nested)?.as_str()
}

fn param_u64(action: &CoreAction, key: &str, nested: &str) -> Option<u64> {
    let value = action.params.get(key)?;
    if nested.is_empty() {
        return value.as_u64();
    }
    value.get(nested)?.as_u64()
}

fn random_legal_action(legal_actions: &[CoreAction], rng: &mut u64) -> Option<CoreAction> {
    let len = u64::try_from(legal_actions.len()).ok()?;
    if len == 0 {
        return None;
    }
    *rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let index = usize::try_from(rng.checked_rem(len)?).ok()?;
    legal_actions.get(index).cloned()
}

fn local_strategy_binding(
    snapshot: &CanonicalStateSnapshot,
    action: &CoreAction,
    strategy_source: &str,
    step: usize,
) -> Result<CanonicalStrategyBinding, CanonicalTruthError> {
    Ok(CanonicalStrategyBinding {
        query_hash: canonical_hash(snapshot)?,
        response_hash: canonical_hash(&json!({
            "action_id": action.action_id,
            "policy": strategy_source,
            "step": step,
        }))?,
        checkpoint_hash: None,
        engine_tier: "core-legal".to_string(),
        engine_family: strategy_source.to_string(),
        quality_summary: Some(format!("policy={strategy_source}")),
    })
}

fn source_summary(sources: &BTreeSet<String>, policy: PlaytracePolicy) -> String {
    if sources.is_empty() {
        return policy.label().to_string();
    }
    if sources.len() == 1 {
        return sources
            .first()
            .cloned()
            .unwrap_or_else(|| policy.label().to_string());
    }
    if sources.contains("best-local") && sources.contains("best-local+legal-continuation") {
        return "best-local+legal-continuation".to_string();
    }

    sources.iter().cloned().collect::<Vec<_>>().join("+")
}

pub fn canonical_ten_playtrace_requests(
    max_steps: usize,
    seed: u64,
    policy: PlaytracePolicy,
) -> Vec<PlaytraceRequest> {
    CANONICAL_TEN
        .into_iter()
        .map(|game| PlaytraceRequest {
            game,
            max_steps,
            seed,
            policy,
        })
        .collect()
}

pub fn research_playtrace_requests(
    max_steps: usize,
    seed: u64,
    policy: PlaytracePolicy,
) -> Vec<PlaytraceRequest> {
    ALL_RESEARCH_GAMES
        .into_iter()
        .map(|game| PlaytraceRequest {
            game,
            max_steps,
            seed,
            policy,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playtrace_runs_for_all_canonical_ten_games() {
        for request in canonical_ten_playtrace_requests(8, 1, PlaytracePolicy::BestLocal) {
            let report = run_playtrace(request).unwrap_or_else(|error| {
                panic!("{} playtrace should run: {error}", request.game.slug())
            });

            assert_eq!(report.game, request.game);
            assert!(!report.truth_hash.is_empty());
            assert!(matches!(report.status.as_str(), "bounded" | "terminal"));
        }
    }

    #[test]
    fn playtrace_runs_for_all_research_games() {
        for request in research_playtrace_requests(6, 1, PlaytracePolicy::BestLocal) {
            let report = run_playtrace(request).unwrap_or_else(|error| {
                panic!("{} playtrace should run: {error}", request.game.slug())
            });

            assert_eq!(report.game, request.game);
            assert!(!report.truth_hash.is_empty());
            assert!(matches!(report.status.as_str(), "bounded" | "terminal"));
        }
    }

    #[test]
    fn best_local_playtrace_stays_direct_for_research_games() {
        for request in research_playtrace_requests(8, 42, PlaytracePolicy::BestLocal) {
            let report = run_playtrace(request).unwrap_or_else(|error| {
                panic!("{} playtrace should run: {error}", request.game.slug())
            });

            assert_eq!(
                report.strategy_source,
                "best-local",
                "{} should stay on direct best-local selection",
                request.game.slug()
            );
        }
    }

    #[test]
    fn playtrace_forced_illegal_action_fails_before_transition() {
        let mut request = PlaytraceRequest::new(ResearchGame::Bridge);
        request.policy = PlaytracePolicy::ForcedIllegal;
        request.max_steps = 1;

        assert!(matches!(
            run_playtrace(request),
            Err(PlaytraceError::IllegalRecommendedAction { game, action_id })
                if game == ResearchGame::Bridge && action_id == "bridge.forced-illegal"
        ));
    }

    #[test]
    fn playtrace_truth_hash_is_deterministic_for_same_seed() {
        let request = PlaytraceRequest {
            game: ResearchGame::RiichiMahjong,
            max_steps: 8,
            seed: 42,
            policy: PlaytracePolicy::RandomLegal,
        };
        let first = run_playtrace(request)
            .unwrap_or_else(|error| panic!("first playtrace should run: {error}"));
        let second = run_playtrace(request)
            .unwrap_or_else(|error| panic!("second playtrace should run: {error}"));

        assert_eq!(first.truth_hash, second.truth_hash);
    }

    #[test]
    fn playtrace_trace_hash_mismatch_is_rejected() {
        let report = run_playtrace(PlaytraceRequest {
            game: ResearchGame::Bridge,
            max_steps: 1,
            seed: 1,
            policy: PlaytracePolicy::LegalFirst,
        })
        .unwrap_or_else(|error| panic!("playtrace should run: {error}"));
        let Some(trace) = report.traces.first().cloned() else {
            panic!("one-step trace should exist");
        };
        let before = bootstrap_state(ResearchGame::Bridge)
            .map(CanonicalStateSnapshot::from)
            .unwrap_or_else(|error| panic!("bridge state should bootstrap: {error}"));
        let transition = apply_action(
            &bootstrap_state(ResearchGame::Bridge)
                .unwrap_or_else(|error| panic!("bridge state should bootstrap: {error}")),
            &trace.action_id,
            json!({}),
        )
        .unwrap_or_else(|error| panic!("bridge action should apply: {error}"));
        let after = CanonicalStateSnapshot::from(transition.after);
        let mut corrupted = trace;
        corrupted.state_hash_after = "bad-hash".to_string();

        assert!(matches!(
            validate_transition_trace(&corrupted, &before, &after),
            Err(CanonicalTruthError::HashMismatch { .. })
        ));
    }
}
