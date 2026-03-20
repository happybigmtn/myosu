//! Liar's Dice edge (action) representation.

use myosu_games::CfrEdge;
use rbp_transport::Support;
use serde::{Deserialize, Serialize};

/// A bid in Liar's Dice specifies a quantity and face.
/// The sentinel value (0, 0) represents no bid (starting state).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Bid {
    pub quantity: u8,
    pub face: u8,
}

impl Bid {
    /// Create a new bid. Panics if quantity or face is 0 (use SEALED sentinel instead).
    pub fn new(quantity: u8, face: u8) -> Self {
        assert!(quantity > 0 && face > 0, "Bid components must be non-zero; use SEALED sentinel");
        Self { quantity, face }
    }

    /// Sentinel value representing "no bid" (sealed/unreachable state).
    pub const SEALED: Bid = Bid { quantity: 0, face: 0 };
}

/// An edge in the Liar's Dice game tree.
/// Either a Bid action or a Challenge action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiarsDiceEdge {
    /// Place a bid: assert that at least `quantity` dice show `face`
    Bid(Bid),
    /// Challenge the previous bid (or a bid that doesn't exist if no bids yet)
    Challenge,
}

impl Support for Bid {}
impl CfrEdge for Bid {}
impl Support for LiarsDiceEdge {}
impl CfrEdge for LiarsDiceEdge {}

/// Maximum number of bids in a valid Liar's Dice game.
/// With 1 die each and 6 faces, no more than 12 bids can be made before
/// all outcomes are mathematically impossible.
pub const MAX_BIDS: usize = 12;
