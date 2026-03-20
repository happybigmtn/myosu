//! NLHE poker engine integration for the myosu game-solving chain.
//!
//! This crate owns:
//! - `PokerSolver` wrapper around `rbp_nlhe::Flagship`
//! - `StrategyQuery` / `StrategyResponse` bridge for miner-validator communication
//! - Wire serialization for `NlheInfo` and `NlheEdge` types (bincode)
//! - Exploitability computation via `Profile::exploitability()`
//! - File-based checkpoint format (4-byte magic `MYOS` + u32 version + bincode)

#![doc = include_str!("../README.md")]

// Public API re-exports (stubs — fleshed out in subsequent slices)
pub use solver::PokerSolver;
pub use query::handle_query;
pub use wire::{WireStrategy, NlheInfoCodec, NlheEdgeCodec};
pub use exploit::{poker_exploitability, remote_poker_exploitability};
pub use training::TrainingSession;

// Re-export from myosu-games for convenience
pub use myosu_games::{StrategyQuery, StrategyResponse, GameType, GameConfig};

mod solver;
mod query;
mod wire;
mod exploit;
mod training;
