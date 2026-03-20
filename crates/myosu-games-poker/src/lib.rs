//! NLHE poker engine integration for the myosu game-solving chain.
//!
//! The currently implemented Slice 2 surface owns:
//! - `PokerSolver` wrapper around `rbp_nlhe::Flagship`
//! - File-based checkpoint framing (`MYOS` + version + bincode)

#![doc = include_str!("../README.md")]

pub use solver::{Flagship, PokerSolver, PokerSolverError};

// Re-export from myosu-games for convenience.
pub use myosu_games::{GameConfig, GameType, StrategyQuery, StrategyResponse};

mod solver;
