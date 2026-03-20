//! `myosu-games-poker`: NLHE poker engine integration for the myosu game-solving chain.
//!
//! This crate provides:
//! - [`PokerSolver`] — wrapper around `rbp_nlhe::Flagship`
//! - [`handle_query`] — bridge from `WireStrategy` → `StrategyResponse`
//! - Wire serialization for [`NlheInfo`] and [`NlheEdge`] via bincode
//! - Exploitability computation via [`poker_exploitability`] and [`remote_poker_exploitability`]
//! - [`TrainingSession`] for batch training with checkpoint management

pub mod solver;
pub mod query;
pub mod wire;
pub mod exploit;
pub mod training;

// Public re-exports
pub use solver::PokerSolver;
pub use rbp_nlhe::Flagship;
pub use query::handle_query;
pub use wire::{WireSerializable, Poker};
pub use exploit::{poker_exploitability, remote_poker_exploitability};
pub use training::TrainingSession;

// Re-export from myosu-games for convenience
pub use myosu_games::{
    StrategyQuery, StrategyResponse, GameType, GameConfig,
    CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, Profile,
    Probability, Utility,
};
