//! Information set type for Liar's Dice.
//!
//! An information set groups game states that a player cannot distinguish.
//! In Liar's Dice, a player knows:
//! - Their own die
//! - The bid history (public)
//! - Whose turn it is (public)
//!
//! So an information set is (my_die, turn, bid_history).

use crate::edge::LiarsDiceEdge;
use crate::game::LiarsDiceGame;
use crate::turn::LiarsDiceTurn;
use rbp_mccfr::{CfrEdge, CfrInfo, CfrPublic, CfrSecret, CfrTurn};
use serde::{Deserialize, Serialize};

/// Maximum bids we can encode in the u64.
const MAX_INFO_BIDS: usize = 12;

/// Information set for Liar's Dice.
///
/// Groups states that a player cannot distinguish: their own die value
/// plus the public bid history and current turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiarsDiceInfo {
    /// Encoded public state: turn (1 bit) + bid_count (4 bits) + bid_faces (7 bits × 12 = 84 bits).
    /// We pack this into a u64 for Ord/Hash derive.
    /// Layout: bits 0-83 = bids (12 × 7 bits each), bits 84-87 = bid_count, bit 88 = turn
    /// But that's 89 bits. Let me use a simpler encoding.
    ///
    /// Actually, for Ord derive to work on a struct, all fields must be Ord.
    /// Let me use a u64 encoding and manually implement Ord.
    encoding: u64,
    /// Private die value (1-6) for this player.
    my_die: u8,
}

impl LiarsDiceInfo {
    /// Create a new info set from turn, bid count, encoded bids, and private die.
    fn new(turn: LiarsDiceTurn, bid_count: u8, bid_encoding: u64, my_die: u8) -> Self {
        // Layout: bits 0-3 = bid_count, bit 4 = turn, bits 5-8*7 = bids
        // Each bid takes 7 bits: quantity (4 bits, 0-12) + face (3 bits, 1-6 encoded as 0-5)
        // Actually, let me just store the bid_encoding directly
        let mut enc = bid_encoding;
        // Add bid_count at bits 60-63 (4 bits)
        enc |= (bid_count as u64 & 0x0F) << 60;
        // Add turn at bit 64 (would overflow, use bit 63)
        // Actually, let me use a different layout
        // Bits 0-3: bid_count (0-12)
        // Bit 4: turn (0 = Player0, 1 = Player1)
        // Bits 5+: bid data
        let enc = ((bid_encoding & 0x1FFFFFFFFFFFFF) as u64) // 45 bits of bid data
            | ((bid_count as u64 & 0x0F) << 0) // bits 0-3: bid_count
            | ((if turn.is_player0() { 0u64 } else { 1u64 }) << 4); // bit 4: turn
        Self { encoding: enc, my_die }
    }

    /// Create from game state and private die.
    pub fn from_game(game: &LiarsDiceGame, my_die: u8) -> Self {
        // Encode bids - bid data starts at bit 5 to leave room for bid_count (bits 0-3) and turn (bit 4)
        let mut bid_encoding = 0u64;
        for i in 0..game.bid_count.min(MAX_INFO_BIDS as u8) as usize {
            let encoded = game.bid_faces[i];
            bid_encoding |= (encoded as u64) << (5 + i * 7);
        }
        Self::new(game.turn, game.bid_count, bid_encoding, my_die)
    }

    /// Get the turn from encoding.
    pub fn turn(&self) -> LiarsDiceTurn {
        let turn_bit = (self.encoding >> 4) & 1;
        if turn_bit == 0 {
            LiarsDiceTurn::Player(0)
        } else {
            LiarsDiceTurn::Player(1)
        }
    }

    /// Get the bid count from encoding.
    pub fn bid_count(&self) -> u8 {
        (self.encoding & 0x0F) as u8
    }

    /// Decode a bid at the given index from the encoding.
    fn decode_bid(&self, index: usize) -> Option<(u8, u8)> {
        if index >= self.bid_count() as usize {
            return None;
        }
        let shift = index * 7;
        let encoded = ((self.encoding >> (5 + shift)) & 0x7F) as u8;
        if encoded == 0 {
            return None;
        }
        let quantity = (encoded >> 4) & 0x0F;
        let face = (encoded & 0x07);
        if quantity == 0 || face == 0 || face > 6 {
            return None;
        }
        Some((quantity, face))
    }
}

/// Total ordering based on encoding.
impl Ord for LiarsDiceInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.encoding.cmp(&other.encoding)
    }
}

impl PartialOrd for LiarsDiceInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for LiarsDiceInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.encoding.hash(state);
    }
}

/// Public component of the information set (for CfrInfo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct LiarsDicePublic {
    /// Encoded public state.
    encoding: u64,
}

impl LiarsDicePublic {
    fn from_encoding(encoding: u64) -> Self {
        Self { encoding }
    }
}

impl CfrPublic for LiarsDicePublic {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn choices(&self) -> Vec<Self::E> {
        let bid_count = (self.encoding & 0x0F) as u8;
        let turn_bit = (self.encoding >> 4) & 1;

        if bid_count == 0 {
            // Opening bid: can bid any face at quantity 1
            vec![
                LiarsDiceEdge::Bid(1, 1),
                LiarsDiceEdge::Bid(1, 2),
                LiarsDiceEdge::Bid(1, 3),
                LiarsDiceEdge::Bid(1, 4),
                LiarsDiceEdge::Bid(1, 5),
                LiarsDiceEdge::Bid(1, 6),
            ]
        } else {
            // Get previous bid
            let prev_idx = (bid_count - 1) as usize;
            let shift = prev_idx * 7;
            let encoded = ((self.encoding >> (5 + shift)) & 0x7F) as u8;
            let prev_q = (encoded >> 4) & 0x0F;
            let prev_f = encoded & 0x07;

            let mut choices = Vec::new();

            // Can always challenge if there's a bid
            choices.push(LiarsDiceEdge::Challenge);

            // Can raise: higher quantity or same quantity with higher face
            for q in 1..=12u8 {
                for f in 1..=6u8 {
                    if q > prev_q || (q == prev_q && f > prev_f) {
                        // Valid raise
                        if q == 1 || q > prev_q || (q == prev_q && f > prev_f) {
                            choices.push(LiarsDiceEdge::Bid(q, f));
                        }
                    }
                }
            }

            choices
        }
    }

    fn history(&self) -> Vec<Self::E> {
        let bid_count = (self.encoding & 0x0F) as u8;
        let mut bids = Vec::new();
        for i in 0..bid_count as usize {
            let shift = i * 7;
            let encoded = ((self.encoding >> (5 + shift)) & 0x7F) as u8;
            if encoded == 0 {
                break;
            }
            let q = (encoded >> 4) & 0x0F;
            let f = encoded & 0x07;
            if q == 0 || f == 0 || f > 6 {
                break;
            }
            bids.push(LiarsDiceEdge::Bid(q, f));
        }
        bids
    }
}

/// Private component (player's own die).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct LiarsDiceSecret(u8);

impl LiarsDiceSecret {
    fn new(die: u8) -> Self {
        Self(die)
    }
}

impl rbp_transport::Support for LiarsDiceSecret {}
impl CfrSecret for LiarsDiceSecret {}

impl CfrInfo for LiarsDiceInfo {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;
    type X = LiarsDicePublic;
    type Y = LiarsDiceSecret;

    fn public(&self) -> Self::X {
        LiarsDicePublic::from_encoding(self.encoding)
    }

    fn secret(&self) -> Self::Y {
        LiarsDiceSecret::new(self.my_die)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn info_from_game() {
        let mut game = LiarsDiceGame::new();
        game.dice = [3, 5];
        game.bid_faces[0] = ((1 & 0x0F) << 4) | (3 & 0x07);
        game.bid_count = 1;
        game.turn = LiarsDiceTurn::Player(1);

        let info = LiarsDiceInfo::from_game(&game, 3);
        assert_eq!(info.turn(), LiarsDiceTurn::Player(1));
        assert_eq!(info.bid_count(), 1);
    }

    #[test]
    fn info_ord() {
        // Verify that LiarsDiceInfo can be ordered (required for CfrInfo)
        let game1 = LiarsDiceGame::new();
        let game2 = LiarsDiceGame::new();

        let info1 = LiarsDiceInfo::from_game(&game1, 3);
        let info2 = LiarsDiceInfo::from_game(&game2, 3);

        // Two infos from different games may or may not be equal
        let _ = info1.cmp(&info2);
    }
}
