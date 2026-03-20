//! Information set representation for Liar's Dice.
//!
//! An information set in Liar's Dice consists of:
//! - **Private (secret)**: the player's own die value (1-6)
//! - **Public**: the bid history leading to this decision point
//!
//! Two game states are in the same information set if the acting player
//! sees the same private state and the same public history. The opponent's
//! die is unknown — that's the imperfect information.
//!
//! ## Bid History Encoding
//!
//! The bid history is stored as a fixed-size array `[Option<(u8,u8)>; MAX_BIDS]`
//! with a `count` field. The `Copy` bound on `CfrGame` requires the full game
//! state (including history) to be `Copy`. Using a fixed array + count avoids
//! heap allocation while satisfying the trait bound.
//!
//! ## Maximum Bids
//!
//! For 1-die-each Liar's Dice, the maximum number of bids before someone
//! must challenge is 12: with 6 faces and alternating players, the longest
//! bid sequence is P0 bids (1,1), P1 bids (1,2), P0 bids (2,2), ..., (6,6)
//! (11 bids) then P0 must challenge on (6,6) — so max 11 actual bids, but
//! we use 12 to have headroom.

use crate::edge::LiarsDiceEdge;
use crate::turn::LiarsDiceTurn;
use rbp_mccfr::{CfrInfo, CfrPublic, CfrSecret};
use rbp_transport::Support;
use std::fmt;

/// Maximum number of bids in a 1-die Liar's Dice game.
/// With 6 faces and 2 players, the longest possible bid sequence
/// before forced challenge is 11 (P0: (1,1), P1: (1,2), ..., (6,6)).
/// We use 12 to have headroom.
pub const MAX_BIDS: usize = 12;

/// Fixed-size bid history for Copy constraint.
/// `None` = slot not yet used; `Some((qty, face))` = bid at that position.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BidHistory([Option<(u8, u8)>; MAX_BIDS]);

impl BidHistory {
    /// Create an empty bid history.
    pub fn new() -> Self {
        Self([None; MAX_BIDS])
    }

    /// Push a bid onto the history. Panics if full.
    pub fn push(&mut self, quantity: u8, face: u8) {
        let count = self.count();
        assert!(count < MAX_BIDS, "bid history overflow");
        self.0[count] = Some((quantity, face));
    }

    /// Get the last bid (None if no bids yet).
    pub fn last_bid(&self) -> Option<(u8, u8)> {
        self.0[..self.count()].iter().rev().find_map(|x| *x)
    }

    /// Number of bids in the history.
    pub fn count(&self) -> usize {
        self.0.iter().take_while(|x| x.is_some()).count()
    }

    /// Iterate over all bids in order.
    pub fn iter(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        self.0[..self.count()].iter().map(|x| x.unwrap())
    }
}

impl Default for BidHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for BidHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BidHistory[")?;
        for (i, (q, face)) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "({},{})", q, face)?;
        }
        write!(f, "]")
    }
}

/// Public information — observable by all players.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LiarsDicePublic {
    /// Bid history leading to this decision point.
    pub history: BidHistory,
}

impl LiarsDicePublic {
    /// Returns all valid bid actions at this point.
    ///
    /// Valid bids must strictly increase the previous bid in quantity OR face.
    /// Any player can always challenge.
    pub fn choices(&self) -> Vec<LiarsDiceEdge> {
        let last = self.history.last_bid();
        let mut edges = Vec::with_capacity(12);

        if let Some((last_qty, last_face)) = last {
            for q in last_qty..=6 {
                for f in 1..=6 {
                    if q > last_qty || (q == last_qty && f > last_face) {
                        edges.push(LiarsDiceEdge::Bid { quantity: q, face: f });
                    }
                }
            }
        } else {
            for q in 1..=6 {
                for f in 1..=6 {
                    edges.push(LiarsDiceEdge::Bid { quantity: q, face: f });
                }
            }
        }

        edges.push(LiarsDiceEdge::Challenge);
        edges
    }

    /// Returns the bid history as a Vec.
    pub fn history_vec(&self) -> Vec<LiarsDiceEdge> {
        self.history
            .iter()
            .map(|(q, f)| LiarsDiceEdge::Bid { quantity: q, face: f })
            .collect()
    }
}

impl CfrPublic for LiarsDicePublic {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn choices(&self) -> Vec<Self::E> {
        LiarsDicePublic::choices(self)
    }

    fn history(&self) -> Vec<Self::E> {
        self.history_vec()
    }
}

/// Private information — the player's own die value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LiarsDiceSecret(pub u8);

impl LiarsDiceSecret {
    /// The die value (1-6).
    pub fn die(&self) -> u8 {
        self.0
    }
}

impl CfrSecret for LiarsDiceSecret {}
impl Support for LiarsDiceSecret {}

/// Information set for Liar's Dice: (my die, bid history).
///
/// This type must be `Copy` to satisfy `CfrGame: Copy`. The `BidHistory`
/// is fixed-size with no heap allocation, making the whole type `Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LiarsDiceInfo {
    /// The player's own die value.
    pub die: u8,
    /// The public bid history.
    pub history: BidHistory,
}

impl LiarsDiceInfo {
    /// Create a new info set for a player with a given die and history.
    pub fn new(die: u8, history: BidHistory) -> Self {
        debug_assert!(die >= 1 && die <= 6);
        Self { die, history }
    }

    /// Create from the root game state — both players have die = 1.
    pub fn root() -> Self {
        Self {
            die: 1,
            history: BidHistory::new(),
        }
    }

    /// The secret component (die value).
    pub fn secret(&self) -> LiarsDiceSecret {
        LiarsDiceSecret(self.die)
    }

    /// The public component (bid history).
    pub fn public(&self) -> LiarsDicePublic {
        LiarsDicePublic { history: self.history }
    }

    /// Returns available actions at this info set.
    pub fn choices(&self) -> Vec<LiarsDiceEdge> {
        self.public().choices()
    }
}

impl CfrInfo for LiarsDiceInfo {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;
    type X = LiarsDicePublic;
    type Y = LiarsDiceSecret;

    fn public(&self) -> Self::X {
        LiarsDiceInfo::public(self)
    }

    fn secret(&self) -> Self::Y {
        LiarsDiceInfo::secret(self)
    }
}

impl fmt::Display for LiarsDiceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Info(die={}, bids=[", self.die)?;
        for (i, (q, face)) in self.history.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "({},{})", q, face)?;
        }
        write!(f, "])")
    }
}
