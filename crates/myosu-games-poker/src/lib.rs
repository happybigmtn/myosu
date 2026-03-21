//! `myosu-games-poker`: NLHE poker engine integration surface.
//!
//! This crate owns:
//! - [`PokerSolver`] — wrapper around `rbp_nlhe::Flagship`
//! - [`handle_query`] — bridge for miner-validator `StrategyQuery`/`StrategyResponse`
//! - [`wire`] — bincode roundtrip for `NlheInfo` and `NlheEdge`
//! - [`poker_exploitability`] / [`remote_poker_exploitability`] — exploitability scoring
//! - [`TrainingSession`] — batch iteration with checkpoint management
//!
//! Does NOT own robopoker internals or miner/validator binaries.

pub mod exploit;
pub mod query;
pub mod solver;
pub mod training;
pub mod wire;

// Re-export public API
pub use exploit::{poker_exploitability, remote_poker_exploitability};
pub use query::handle_query;
pub use solver::PokerSolver;
pub use rbp_nlhe::Flagship;
pub use training::TrainingSession;

// Re-export from myosu-games for convenience
pub use myosu_games::{GameConfig, GameType, StrategyQuery, StrategyResponse};
