//! Edge (action) type for Liar's Dice.
//!
//! Actions are either a bid (quantity, face) or a challenge.

use rbp_mccfr::CfrEdge;
use rbp_transport::Support;
use serde::{Deserialize, Serialize};

/// An action in Liar's Dice.
///
/// Players can either place a bid (quantity of a given face) or challenge
/// the previous bid.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LiarsDiceEdge {
    /// A bid: (quantity, face) where quantity >= 1 and face in 1..=6.
    /// A bid is valid only if it strictly dominates the previous bid
    /// (higher quantity or same quantity with higher face).
    Bid(u8, u8),
    /// Challenge the previous bid. The game resolves immediately.
    Challenge,
}

impl Support for LiarsDiceEdge {}
impl CfrEdge for LiarsDiceEdge {}

impl LiarsDiceEdge {
    /// Returns true if this is a bid edge.
    pub fn is_bid(&self) -> bool {
        matches!(self, Self::Bid(_, _))
    }

    /// Returns true if this is a challenge edge.
    pub fn is_challenge(&self) -> bool {
        matches!(self, Self::Challenge)
    }

    /// Get the quantity and face if this is a bid.
    pub fn as_bid(&self) -> Option<(u8, u8)> {
        match self {
            Self::Bid(q, f) => Some((*q, *f)),
            _ => None,
        }
    }
}
