// myosu-games: game engine trait abstraction for the myosu game-solving chain.
//
// This crate defines the traits that all game engines must implement.
// Poker, backgammon, mahjong, bridge, and other games each provide
// an implementation of these traits. The solver (miner), validator,
// and gameplay layers depend on this crate.

#![doc = include_str!("../README.md")]

pub mod registry;
pub mod traits;

// Re-export commonly used types for convenience
pub use traits::{
    CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, ExploitMetric, ExploitScale, GameConfig,
    GameParams, GameType, Probability, Profile, StrategyQuery, StrategyResponse, Utility,
};
