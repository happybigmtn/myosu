// myosu-games-poker: NLHE poker engine integration for the myosu game-solving chain.
//
// This crate wraps robopoker's NLHE solver and provides:
// - PokerSolver: trainable MCCFR solver wrapper
// - Wire serialization: bincode roundtrip for NlheInfo and NlheEdge
// - Query handler: bridge for miner-validator communication
// - Exploitability: best-response computation for validator scoring
// - Training session: batch training with checkpoint management

pub mod exploit;
pub mod query;
pub mod solver;
pub mod training;
pub mod wire;

// Re-export robopoker NLHE types for consumers
pub use rbp_nlhe::{NlheEdge, NlheInfo, NlheProfile, NlheEncoder, NlheGame, NlheTurn};
pub use rbp_nlhe::Flagship;

// Re-export myosu-games types for convenience
pub use myosu_games::{
    StrategyQuery, StrategyResponse, GameType, GameConfig, GameParams,
    CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, Profile, Probability, Utility,
};

/// Convenience type alias for the production NLHE solver (Pluribus configuration).
pub type PokerSolver = solver::PokerSolver;

/// Convenience type alias for the debug NLHE solver (faster iteration).
pub type DebugSolver = solver::DebugSolver;