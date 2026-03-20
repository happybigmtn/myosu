//! Liar's Dice game engine for the myosu game-solving chain.
//!
//! This crate implements the CfrGame trait for Liar's Dice, providing
//! a second CFR game engine that validates the trait system's generality.

pub mod game;
pub mod edge;
pub mod turn;
pub mod info;
pub mod encoder;
pub mod profile;

// Re-export the public API
pub use game::LiarsDiceGame;
pub use edge::LiarsDiceEdge;
pub use turn::LiarsDiceTurn;
pub use info::LiarsDiceInfo;
pub use encoder::LiarsDiceEncoder;
pub use profile::LiarsDiceProfile;