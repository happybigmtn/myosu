//! Liar's Dice — proof-of-architecture CFR game engine for myosu.
//!
//! This crate implements a 2-player, 1-die-each Liar's Dice game as a
//! concrete [`CfrGame`] implementation, validating that the robopoker trait
//! system generalizes beyond poker.
//!
//! ## Game Rules (1-die variant)
//!
//! - Each player has one die (values 1–6), hidden from the opponent.
//! - Player 0 bids first; play alternates.
//! - A valid bid must strictly increase the previous bid in quantity or face.
//! - A player may challenge ("Liars!") instead of bidding.
//! - If challenged, the last bid is evaluated against the true dice state:
//!   the challenger wins if the claimed quantity of the claimed face is false.
//!
//! ## Architecture
//!
//! The implementation follows the robopoker CFR trait hierarchy:
//!
//! - [`game::LiarsDiceGame`] — root node, dice roll distribution, action space
//! - [`edge::LiarsDiceEdge`] — `Bid(quantity, face)` or `Challenge`
//! - [`turn::LiarsDiceTurn`] — `Player(0)`, `Player(1)`, `Chance`, `Terminal`
//! - [`info::LiarsDiceInfo`] — `(my_die: u8, bid_history: [Bid; 12])`
//! - [`encoder::LiarsDiceEncoder`] — direct enumeration encoding
//! - [`profile::LiarsDiceProfile`] — CFR training + Nash verification

pub mod edge;
pub mod encoder;
pub mod game;
pub mod info;
pub mod profile;
pub mod turn;

// Re-export for convenience
pub use edge::LiarsDiceEdge;
pub use encoder::LiarsDiceEncoder;
pub use game::LiarsDiceGame;
pub use info::LiarsDiceInfo;
pub use profile::LiarsDiceProfile;
pub use turn::LiarsDiceTurn;
