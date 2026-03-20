//! Test harness for validating `CfrGame` implementations.
//!
//! This module provides assertion helpers that verify a game implementation
//! satisfies the CFR invariants:
//!
//! - Root node is chance or player
//! - Legal actions are non-empty except at terminal states
//! - State transitions are valid
//! - Terminal states have utility
//! - Payoff is zero-sum
//! - Information sets are distinct for different observations
//!
//! # Example
//!
//! ```rust,ignore
//! use myosu_sdk::testing::assert_game_valid;
//! use myosu_sdk::CfrGame;
//!
//! assert_game_valid::<MyRockPaperScissors>();
//! ```

pub mod game_valid;
pub mod convergence;
pub mod tests;
