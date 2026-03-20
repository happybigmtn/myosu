//! Liar's Dice edges — `CfrEdge` implementation.
//!
//! An edge represents a single action in the game tree.

/// LiarsDiceEdge represents an action: either a `Bid` or a `Challenge`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiarsDiceEdge {
    /// Make a bid: `Bid(quantity, face)`
    Bid(super::game::Bid),
    /// Challenge the previous bid
    Challenge,
}

impl LiarsDiceEdge {
    /// Return the bid if this is a Bid edge.
    pub fn as_bid(&self) -> Option<&super::game::Bid> {
        match self {
            LiarsDiceEdge::Bid(b) => Some(b),
            LiarsDiceEdge::Challenge => None,
        }
    }
}
