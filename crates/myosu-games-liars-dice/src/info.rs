//! Liar's Dice information set representation.

use myosu_games::CfrInfo;
use rbp_mccfr::{CfrPublic as CfrPublicTrait, CfrSecret as CfrSecretTrait};
use rbp_transport::Support;
use serde::{Deserialize, Serialize};

use crate::edge::{Bid, LiarsDiceEdge};
use crate::turn::LiarsDiceTurn;

/// Public state for Liar's Dice - the bid history is public knowledge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LiarsDicePublic {
    /// All bids made so far, in order. Unfilled slots are SEALED sentinel.
    pub bid_history: [Bid; 12],
    /// Number of valid bids in the history (0 means no bids yet)
    pub num_bids: u8,
}

impl LiarsDicePublic {
    pub fn new() -> Self {
        Self {
            bid_history: [Bid::SEALED; 12],
            num_bids: 0,
        }
    }

    pub fn with_bids(bid_history: [Bid; 12], num_bids: u8) -> Self {
        Self { bid_history, num_bids }
    }

    /// Get the most recent bid, if any.
    pub fn last_bid(&self) -> Option<Bid> {
        if self.num_bids == 0 {
            None
        } else {
            Some(self.bid_history[(self.num_bids - 1) as usize])
        }
    }

    /// Add a bid to the history.
    pub fn push_bid(&mut self, bid: Bid) {
        if self.num_bids < 12 {
            self.bid_history[self.num_bids as usize] = bid;
            self.num_bids += 1;
        }
    }
}

impl Default for LiarsDicePublic {
    fn default() -> Self {
        Self::new()
    }
}

impl Support for LiarsDicePublic {}

impl CfrPublicTrait for LiarsDicePublic {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn choices(&self) -> Vec<Self::E> {
        self.legal_actions()
    }

    fn history(&self) -> Vec<Self::E> {
        self.bid_history[..self.num_bids as usize]
            .iter()
            .map(|&b| LiarsDiceEdge::Bid(b))
            .collect()
    }
}

impl LiarsDicePublic {
    /// Get legal actions at this point in the game.
    pub fn legal_actions(&self) -> Vec<LiarsDiceEdge> {
        let mut actions = Vec::new();

        // Challenge is always available if there are bids
        if self.num_bids > 0 {
            actions.push(LiarsDiceEdge::Challenge);
        }

        // Add all valid raise bids
        actions.extend(self.legal_bids());

        actions
    }

    /// Get all legal bid actions.
    pub fn legal_bids(&self) -> Vec<LiarsDiceEdge> {
        let mut bids = Vec::new();

        if let Some(last) = self.last_bid() {
            // Must strictly increase quantity or face
            for q in last.quantity..=2 {
                for f in 1..=6 {
                    if q > last.quantity || (q == last.quantity && f > last.face) {
                        bids.push(LiarsDiceEdge::Bid(Bid::new(q, f)));
                    }
                }
            }
        } else {
            // First bid: any (q >= 1, f >= 1) is valid
            for q in 1..=2 {
                for f in 1..=6 {
                    bids.push(LiarsDiceEdge::Bid(Bid::new(q, f)));
                }
            }
        }

        bids
    }

    /// Number of legal raises (not counting challenge).
    pub fn num_legal_raises(&self) -> usize {
        self.legal_bids().len()
    }
}

/// Private state for Liar's Dice - the player's own die.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LiarsDiceSecret {
    /// This player's die value (1-6)
    pub my_die: u8,
}

impl LiarsDiceSecret {
    pub fn new(my_die: u8) -> Self {
        Self { my_die }
    }
}

impl Support for LiarsDiceSecret {}
impl CfrSecretTrait for LiarsDiceSecret {}

/// An information set in Liar's Dice represents everything a player knows
/// at a decision point: their own die and all previous bids.
///
/// Player 0 cannot see player 1's die, and vice versa.
/// The bid history is public information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LiarsDiceInfo {
    /// Public state (bid history)
    pub public: LiarsDicePublic,
    /// Private state (own die)
    pub secret: LiarsDiceSecret,
}

impl LiarsDiceInfo {
    pub fn new(my_die: u8) -> Self {
        Self {
            public: LiarsDicePublic::new(),
            secret: LiarsDiceSecret::new(my_die),
        }
    }

    pub fn with_state(public: LiarsDicePublic, secret: LiarsDiceSecret) -> Self {
        Self { public, secret }
    }
}

impl Default for LiarsDiceInfo {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Support for LiarsDiceInfo {}

impl CfrInfo for LiarsDiceInfo {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;
    type X = LiarsDicePublic;
    type Y = LiarsDiceSecret;

    fn public(&self) -> Self::X {
        self.public
    }

    fn secret(&self) -> Self::Y {
        self.secret
    }
}
