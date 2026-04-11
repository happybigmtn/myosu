use serde::{Deserialize, Serialize};

use myosu_games::{StrategyQuery, StrategyResponse};

use crate::game::ResearchGame;
use crate::state::PortfolioChallenge;

/// Strength-query wire schema version for typed portfolio challenges.
pub const STRENGTH_WIRE_VERSION: u32 = 12;

/// Wire-safe information set for a bootstrap portfolio query.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioInfo {
    pub game: ResearchGame,
    pub decision: String,
    pub solver_family: String,
    pub rule_file: String,
}

impl PortfolioInfo {
    /// Create the representative bootstrap query info for a research game.
    pub fn bootstrap(game: ResearchGame) -> Self {
        Self {
            game,
            decision: game.bootstrap_decision().to_string(),
            solver_family: game.solver_family().to_string(),
            rule_file: game.rule_file().to_string(),
        }
    }
}

/// Wire-safe typed information set for a game-specific portfolio engine query.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioStrengthInfo {
    pub game: ResearchGame,
    pub wire_version: u32,
    pub challenge: PortfolioChallenge,
    pub rule_file: String,
    pub solver_family: String,
}

impl PortfolioStrengthInfo {
    /// Create the representative typed engine query info for a portfolio game.
    pub fn bootstrap(game: ResearchGame) -> Option<Self> {
        let challenge = PortfolioChallenge::bootstrap(game)?;
        Some(Self {
            game,
            wire_version: STRENGTH_WIRE_VERSION,
            rule_file: game.rule_file().to_string(),
            solver_family: game.solver_family().to_string(),
            challenge,
        })
    }
}

/// Typed actions used by the portfolio reference engines.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioAction {
    TightOpen,
    DefendBlind,
    PotControl,
    ValueBet,
    PushFold,
    IcmFold,
    PotSizedRaise,
    DrawToNuts,
    AnteSteal,
    SeeCards,
    KoiKoi,
    StopRound,
    CallGo,
    DeclareRiichi,
    FoldDanger,
    BidContract,
    DoubleDummyPlay,
    Knock,
    DiscardDeadwood,
    Scout,
    AdvancePiece,
    PlaceSafe,
    BidNil,
    FollowSuit,
    TrumpControl,
    Challenge,
    LandlordBid,
    ShedLowest,
    PreserveBomb,
    LeadControl,
    PassControl,
    CallTrump,
    BearOff,
    AcceptDouble,
    AvoidPenalty,
    ShootMoon,
    PegRun,
    KeepCrib,
}

impl PortfolioAction {
    /// Stable user-facing action label for pipe and TUI surfaces.
    pub const fn label(self) -> &'static str {
        match self {
            Self::TightOpen => "tight-open",
            Self::DefendBlind => "defend-blind",
            Self::PotControl => "pot-control",
            Self::ValueBet => "value-bet",
            Self::PushFold => "push-fold",
            Self::IcmFold => "icm-fold",
            Self::PotSizedRaise => "pot-sized-raise",
            Self::DrawToNuts => "draw-to-nuts",
            Self::AnteSteal => "ante-steal",
            Self::SeeCards => "see-cards",
            Self::KoiKoi => "koi-koi",
            Self::StopRound => "stop-round",
            Self::CallGo => "call-go",
            Self::DeclareRiichi => "declare-riichi",
            Self::FoldDanger => "fold-danger",
            Self::BidContract => "bid-contract",
            Self::DoubleDummyPlay => "double-dummy-play",
            Self::Knock => "knock",
            Self::DiscardDeadwood => "discard-deadwood",
            Self::Scout => "scout",
            Self::AdvancePiece => "advance-piece",
            Self::PlaceSafe => "place-safe",
            Self::BidNil => "bid-nil",
            Self::FollowSuit => "follow-suit",
            Self::TrumpControl => "trump-control",
            Self::Challenge => "challenge",
            Self::LandlordBid => "landlord-bid",
            Self::ShedLowest => "shed-lowest",
            Self::PreserveBomb => "preserve-bomb",
            Self::LeadControl => "lead-control",
            Self::PassControl => "pass-control",
            Self::CallTrump => "call-trump",
            Self::BearOff => "bear-off",
            Self::AcceptDouble => "accept-double",
            Self::AvoidPenalty => "avoid-penalty",
            Self::ShootMoon => "shoot-moon",
            Self::PegRun => "peg-run",
            Self::KeepCrib => "keep-crib",
        }
    }
}

/// Wire-safe portfolio strategy query.
pub type PortfolioStrategyQuery = StrategyQuery<PortfolioInfo>;

/// Wire-safe typed strength query for portfolio game-specific engines.
pub type PortfolioStrengthQuery = StrategyQuery<PortfolioStrengthInfo>;

/// Wire-safe portfolio strategy response.
pub type PortfolioStrategyResponse = StrategyResponse<PortfolioAction>;

/// Pick the highest-probability action from a portfolio strategy response.
pub fn recommended_action(response: &PortfolioStrategyResponse) -> Option<PortfolioAction> {
    response
        .actions
        .iter()
        .copied()
        .max_by(|(left_action, left_prob), (right_action, right_prob)| {
            left_prob
                .total_cmp(right_prob)
                .then_with(|| left_action.cmp(right_action))
        })
        .map(|(action, _)| action)
}

#[cfg(test)]
mod tests {
    use crate::game::ResearchGame;

    use super::{
        PortfolioAction, PortfolioStrategyResponse, PortfolioStrengthInfo, STRENGTH_WIRE_VERSION,
        recommended_action,
    };

    #[test]
    fn recommended_action_prefers_probability_then_action_order() {
        let response = PortfolioStrategyResponse::new(vec![
            (PortfolioAction::PotControl, 0.4),
            (PortfolioAction::TightOpen, 0.7),
            (PortfolioAction::ValueBet, 0.7),
        ]);

        assert_eq!(
            recommended_action(&response),
            Some(PortfolioAction::ValueBet)
        );
    }

    #[test]
    fn recommended_action_handles_empty_response() {
        let response = PortfolioStrategyResponse::new(Vec::new());

        assert_eq!(recommended_action(&response), None);
    }

    #[test]
    fn action_labels_are_pipe_safe() {
        let response = PortfolioStrategyResponse::new(vec![
            (PortfolioAction::DoubleDummyPlay, 0.7),
            (PortfolioAction::BidContract, 0.3),
        ]);

        for (action, _) in response.actions {
            let label = action.label();
            assert!(!label.is_empty());
            assert!(label.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-'));
        }
    }

    #[test]
    fn strength_info_uses_typed_challenge_for_portfolio_games() {
        let info = match PortfolioStrengthInfo::bootstrap(ResearchGame::Bridge) {
            Some(info) => info,
            None => panic!("bridge should have typed strength info"),
        };

        assert_eq!(info.game, ResearchGame::Bridge);
        assert_eq!(info.wire_version, STRENGTH_WIRE_VERSION);
        assert_eq!(info.challenge.game(), ResearchGame::Bridge);
        assert_eq!(info.rule_file, ResearchGame::Bridge.rule_file());
    }

    #[test]
    fn strength_info_rejects_dedicated_solver_games() {
        assert!(PortfolioStrengthInfo::bootstrap(ResearchGame::NlheHeadsUp).is_none());
        assert!(PortfolioStrengthInfo::bootstrap(ResearchGame::LiarsDice).is_none());
    }
}
