//! Liar's Dice game engine for the myosu game-solving chain.
//!
//! This crate implements a 2-player Liar's Dice game (1 die each, 6 faces)
//! as a CFR game, providing the proof-of-architecture that the trait system
//! generalizes beyond poker.

pub mod edge;
pub mod encoder;
pub mod game;
pub mod info;
pub mod profile;
pub mod solver;
pub mod turn;

// Re-export public API
pub use edge::LiarsDiceEdge;
pub use encoder::LiarsDiceEncoder;
pub use game::LiarsDiceGame;
pub use info::LiarsDiceInfo;
pub use profile::LiarsDiceProfile;
pub use turn::LiarsDiceTurn;
