// myosu-games-poker: NLHE poker engine integration for the myosu game-solving chain.
//
// This crate provides:
// - PokerSolver: wrapper around rbp_nlhe::Flagship (NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>)
// - StrategyQuery/StrategyResponse bridge for miner-validator communication
// - Wire serialization for NlheInfo and NlheEdge (bincode)
// - Exploitability computation via Profile::exploitability()
// - File-based checkpoint format (MYOS magic + u32 version + bincode)

pub mod solver;
pub mod query;
pub mod wire;
pub mod exploit;
pub mod training;

// Re-exports from myosu-games for convenience
pub use myosu_games::{
    GameConfig, GameType, GameParams, Probability, StrategyQuery, StrategyResponse,
};

// Re-export solver types
pub use solver::{PokerSolver, Flagship};

// Re-export query handler
pub use query::handle_query;

// Re-export exploitability functions
pub use exploit::{poker_exploitability, remote_poker_exploitability};

// Re-export training session
pub use training::TrainingSession;

// Re-export wire serialization types
pub use wire::{WireSerializable, Poker};
