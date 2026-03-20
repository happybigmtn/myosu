//! `games:poker-engine` — NLHE poker engine integration surface.
//!
//! This crate provides the concrete implementation of the poker engine for the myosu
//! game-solving chain. It wraps robopoker's `rbp_nlhe::Flagship` solver and exposes
//! a typed bridge for miner-validator communication via `StrategyQuery`/`StrategyResponse`.
//!
//! # Crate Structure
//!
//! - [`solver`](solver) — `PokerSolver` wrapper with training, checkpoint, and strategy queries
//! - [`wire`](wire) — bincode serialization for `NlheInfo` and `NlheEdge`
//! - [`query`](query) — `handle_query` bridge converting wire types to responses
//! - [`exploit`](exploit) — Exploitability computation (local and remote)
//! - [`training`](training) — `TrainingSession` for batch iteration with checkpoint management

pub use solver::{PokerSolver, Flagship};
pub use query::handle_query;
pub use wire::{WireSerializable, Poker};
pub use exploit::{poker_exploitability, remote_poker_exploitability};
pub use training::TrainingSession;

// Re-exports from myosu-games for convenience
pub use myosu_games::{StrategyQuery, StrategyResponse, GameType, GameConfig};

mod solver;
mod query;
mod wire;
mod exploit;
mod training;
