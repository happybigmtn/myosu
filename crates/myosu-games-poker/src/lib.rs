//! `myosu-games-poker` — NLHE poker engine integration surface.
//!
//! This crate provides the concrete implementation of the poker engine for the
//! myosu game-solving chain. It wraps `rbp-nlhe::Flagship` (Pluribus-configured
//! NLHE solver) and exposes:
//!
//! - [`PokerSolver`] — trainable solver with checkpoint save/load
//! - [`handle_query`] — stateless query handler for miner-validator communication
//! - [`poker_exploitability`] / [`remote_poker_exploitability`] — strategy scoring
//! - [`TrainingSession`] — batch training with configurable checkpointing
//!
//! # Wire Format
//!
//! `NlheInfo` and `NlheEdge` are serialized via bincode for network transport.
//! Both types require the `serde` feature on robopoker crates (enabled by default).

pub mod solver;
pub mod query;
pub mod wire;
pub mod exploit;
pub mod training;

// Re-exports from myosu-games for convenience
pub use myosu_games::{
    GameConfig, GameType, GameParams, StrategyQuery, StrategyResponse,
};

pub use solver::PokerSolver;
pub use query::handle_query;
pub use wire::{WireSerializable, Poker};
pub use exploit::{poker_exploitability, remote_poker_exploitability};
pub use training::TrainingSession;

// Re-export the Flagship type directly from robopoker
pub use rbp_nlhe::Flagship;
