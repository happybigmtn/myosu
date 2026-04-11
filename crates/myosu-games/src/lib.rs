// myosu-games: game engine trait abstraction for the myosu game-solving chain.
//
// This crate defines the traits that all game engines must implement.
// Poker, backgammon, mahjong, bridge, and other games each provide
// an implementation of these traits. The solver (miner), validator,
// and gameplay layers depend on this crate.

#![doc = include_str!("../README.md")]

pub mod canonical;
pub mod registry;
pub mod traits;

// Re-export commonly used types for convenience
pub use canonical::{
    CanonicalActionSpec, CanonicalGameSpec, CanonicalStateSnapshot, CanonicalStrategyBinding,
    CanonicalTransitionTrace, CanonicalTruthError, canonical_hash, validate_action_id,
    validate_unique_action_ids,
};
pub use registry::{GameDescriptor, GameRegistry};
pub use traits::{
    CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, GameConfig, GameParams, GameType, Probability,
    Profile, StrategyQuery, StrategyResponse, Utility,
};
