use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::game::ResearchGame;
use crate::protocol::{PortfolioAction, PortfolioStrategyResponse};
use crate::state::PortfolioChallenge;

/// Current implementation tier for a portfolio engine.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineTier {
    StaticBaseline,
    RuleAware,
}

impl EngineTier {
    /// Stable manifest token for shell harnesses.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StaticBaseline => "static-baseline",
            Self::RuleAware => "rule-aware",
        }
    }
}

/// Fully evaluated engine answer before it is reduced to the wire response.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineAnswer {
    pub game: ResearchGame,
    pub challenge_id: String,
    pub engine_family: String,
    pub engine_tier: EngineTier,
    pub legal_actions: Vec<PortfolioAction>,
    pub response: PortfolioStrategyResponse,
}

impl EngineAnswer {
    pub(crate) fn new(
        game: ResearchGame,
        challenge_id: impl Into<String>,
        engine_family: impl Into<String>,
        engine_tier: EngineTier,
        response: PortfolioStrategyResponse,
    ) -> Self {
        let legal_actions = response.actions.iter().map(|(action, _)| *action).collect();

        Self {
            game,
            challenge_id: challenge_id.into(),
            engine_family: engine_family.into(),
            engine_tier,
            legal_actions,
            response,
        }
    }
}

/// Error returned when an engine cannot answer a typed challenge.
#[derive(Debug, Error)]
pub enum PortfolioEngineError {
    #[error("game `{game}` is not routed through the research portfolio engine")]
    UnsupportedPortfolioGame { game: ResearchGame },
}

/// Answer a legacy bootstrap query through the engine dispatch layer.
pub fn answer_game(game: ResearchGame, epochs: usize) -> EngineAnswer {
    crate::engines::answer_game(game, epochs)
}

/// Answer a typed challenge through the engine dispatch layer.
pub fn answer_typed_challenge(
    challenge: &PortfolioChallenge,
    epochs: usize,
) -> Result<EngineAnswer, PortfolioEngineError> {
    let game = challenge.game();
    if !game.is_portfolio_routed() {
        return Err(PortfolioEngineError::UnsupportedPortfolioGame { game });
    }

    Ok(crate::engines::answer_challenge(challenge, epochs))
}

#[cfg(test)]
mod tests {
    use crate::engine::{EngineTier, answer_game, answer_typed_challenge};
    use crate::game::ResearchGame;
    use crate::state::PortfolioChallenge;

    #[test]
    fn bridge_uses_rule_aware_engine_tier() {
        let challenge = match PortfolioChallenge::bootstrap(ResearchGame::Bridge) {
            Some(challenge) => challenge,
            None => panic!("bridge should have typed challenge"),
        };
        let answer = match answer_typed_challenge(&challenge, 0) {
            Ok(answer) => answer,
            Err(error) => panic!("bridge should answer: {error}"),
        };

        assert_eq!(answer.game, ResearchGame::Bridge);
        assert_eq!(answer.engine_tier, EngineTier::RuleAware);
        assert!(answer.response.is_valid());
    }

    #[test]
    fn portfolio_games_route_through_rule_aware_engines() {
        let answer = answer_game(ResearchGame::Cribbage, 0);

        assert_eq!(answer.game, ResearchGame::Cribbage);
        assert_eq!(answer.engine_tier, EngineTier::RuleAware);
        assert!(answer.response.is_valid());
    }

    #[test]
    fn dedicated_games_stay_on_static_compatibility_baseline() {
        let answer = answer_game(ResearchGame::LiarsDice, 0);

        assert_eq!(answer.game, ResearchGame::LiarsDice);
        assert_eq!(answer.engine_tier, EngineTier::StaticBaseline);
        assert!(answer.response.is_valid());
    }
}
