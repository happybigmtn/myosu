//! Liar's Dice game engine for the myosu game-solving chain.
//!
//! This crate implements the CFR trait system for Liar's Dice, proving that
//! the game-agnostic trait architecture generalizes beyond poker.
//!
//! # Game Variant
//!
//! This implementation uses the 1-die-each variant:
//! - 2 players, each with 1 die
//! - 6 faces per die
//! - Variable-length bid history (max 12 bids)
//!
//! # Architecture
//!
//! - `game.rs`: `LiarsDiceGame` — root node, action space, terminal detection
//! - `edge.rs`: `LiarsDiceEdge` — `Bid(quantity, face)` or `Challenge`
//! - `turn.rs`: `LiarsDiceTurn` — `Player(0)`, `Player(1)`, `Chance`, `Terminal`
//! - `info.rs`: `LiarsDiceInfo` — `(my_die, bid_history)` information set
//! - `encoder.rs`: `LiarsDiceEncoder` — trivial direct enumeration
//! - `profile.rs`: `LiarsDiceProfile` — MCCFR profile with Nash convergence

#![doc = include_str!("../README.md")]

// Public re-exports — these types are implemented in subsequent slices
pub use game::LiarsDiceGame;
pub use edge::LiarsDiceEdge;
pub use turn::LiarsDiceTurn;
pub use info::LiarsDiceInfo;
pub use encoder::LiarsDiceEncoder;
pub use profile::LiarsDiceProfile;

// Common re-exports from robopoker
pub use rbp_core::{Probability, Utility};
pub use rbp_mccfr::{CfrEdge, CfrGame, CfrInfo, CfrTurn, Encoder, Profile};

// Module stubs — implemented in slices 2 and 3
mod game;
mod edge;
mod turn;
mod info;
mod encoder;
mod profile;