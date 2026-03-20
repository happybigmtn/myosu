//! Edge (action) representation for Liar's Dice.

use rbp_mccfr::CfrEdge;
use rbp_transport::Support;
use std::fmt;
use std::hash::Hash;

/// Edge (action) in Liar's Dice.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LiarsDiceEdge {
    Bid { quantity: u8, face: u8 },
    Challenge,
}

impl LiarsDiceEdge {
    pub fn is_bid(&self) -> bool {
        matches!(self, Self::Bid { .. })
    }

    pub fn as_bid(&self) -> (u8, u8) {
        match self {
            Self::Bid { quantity, face } => (*quantity, *face),
            Self::Challenge => panic!("Challenge has no bid values"),
        }
    }

    pub fn is_valid_escalation(&self, previous: Option<(u8, u8)>) -> bool {
        match (self, previous) {
            (Self::Challenge, _) => true,
            (Self::Bid { quantity: q, face: f }, None) => *q >= 1 && (*f >= 1 && *f <= 6),
            (Self::Bid { quantity: q, face: f }, Some((pq, pf))) => {
                (*q > pq) || (*q == pq && *f > pf)
            }
        }
    }
}

impl Support for LiarsDiceEdge {}
impl CfrEdge for LiarsDiceEdge {}

impl fmt::Display for LiarsDiceEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bid { quantity, face } => write!(f, "Bid({quantity},{face})"),
            Self::Challenge => write!(f, "Challenge"),
        }
    }
}
