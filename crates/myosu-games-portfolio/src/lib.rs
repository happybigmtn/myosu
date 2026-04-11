//! Solver portfolio for the research game-rule corpus.
//!
//! These policies are intentionally compact rule-aware reference engines: they
//! make every surveyed game addressable by the myosu miner/validator wire
//! contract while deeper game-specific engines are built behind the same
//! surface.

pub mod cards;
pub mod combinatorics;
pub mod core;
pub mod engine;
mod engines;
pub mod eval;
pub mod game;
pub mod protocol;
pub mod quality;
pub mod renderer;
pub mod rng;
pub mod rollout;
pub mod solver;
pub mod state;
pub mod wire;

pub use engine::{
    EngineAnswer, EngineTier, PortfolioEngineError, answer_game, answer_typed_challenge,
};
pub use game::{
    ALL_PORTFOLIO_ROUTED_GAMES, ALL_RESEARCH_GAMES, ParseResearchGameError, ResearchGame,
};
pub use protocol::{
    PortfolioAction, PortfolioInfo, PortfolioStrategyQuery, PortfolioStrategyResponse,
    PortfolioStrengthInfo, PortfolioStrengthQuery, STRENGTH_WIRE_VERSION, recommended_action,
};
pub use quality::EngineQualityReport;
pub use renderer::{PortfolioRenderer, PortfolioSnapshot};
pub use solver::{PortfolioSolver, PortfolioSolverError};
pub use state::{PortfolioChallenge, PortfolioChallengeSpot};
pub use wire::{
    WireCodecError, decode_info, decode_strategy_query, decode_strategy_response,
    decode_strength_query, encode_info, encode_strategy_query, encode_strategy_response,
    encode_strength_query,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_splits_the_combined_twenty_first_rules_file() {
        assert_eq!(ALL_RESEARCH_GAMES.len(), 22);
        assert_eq!(ALL_PORTFOLIO_ROUTED_GAMES.len(), 20);
        assert!(ALL_RESEARCH_GAMES.contains(&ResearchGame::Hearts));
        assert!(ALL_RESEARCH_GAMES.contains(&ResearchGame::Cribbage));
        assert!(!ALL_PORTFOLIO_ROUTED_GAMES.contains(&ResearchGame::NlheHeadsUp));
        assert!(!ALL_PORTFOLIO_ROUTED_GAMES.contains(&ResearchGame::LiarsDice));
        assert!(ALL_PORTFOLIO_ROUTED_GAMES.contains(&ResearchGame::Hearts));
        assert!(ALL_PORTFOLIO_ROUTED_GAMES.contains(&ResearchGame::Cribbage));
    }

    #[test]
    fn every_portfolio_game_has_a_valid_rule_aware_policy() {
        let solver = PortfolioSolver::new();

        for game in ALL_PORTFOLIO_ROUTED_GAMES {
            let query = PortfolioSolver::bootstrap_query(game);
            let response = solver.answer(query);

            assert!(
                response.is_valid(),
                "{} policy should be a valid probability distribution",
                game.slug()
            );
            assert!(
                recommended_action(&response).is_some(),
                "{} policy should recommend an action",
                game.slug()
            );
        }
    }

    #[test]
    fn every_research_game_resolves_through_shared_registry() {
        for game in ALL_RESEARCH_GAMES {
            let descriptor = myosu_games::GameRegistry::from_bytes(game.chain_id().as_bytes())
                .unwrap_or_else(|| {
                    panic!("{} should resolve through shared registry", game.slug())
                });

            assert_eq!(descriptor.game_type, game.game_type());
            assert_eq!(descriptor.num_players, game.default_players());
            assert!(descriptor.builtin);
        }
    }

    #[test]
    fn every_research_game_builds_a_shared_game_config() {
        for game in ALL_RESEARCH_GAMES {
            let config = game.game_config();

            assert_eq!(config.game_type, game.game_type());
            assert_eq!(config.num_players, game.default_players());
            assert!(myosu_games::GameRegistry::is_builtin(&config.game_type));
        }
    }
}
