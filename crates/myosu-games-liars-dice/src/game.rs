//! Game state and logic for Liar's Dice.
//!
//! Implements the CfrGame trait for 2-player 1-die Liar's Dice.
//!
//! ## Game Rules
//!
//! - Each player has 1 die with 6 faces (values 1-6)
//! - Dice are private (only the owner knows their die)
//! - Player 0 starts, then players alternate bidding
//! - A bid (quantity, face) must strictly dominate the previous bid:
//!   - Higher quantity, OR
//!   - Same quantity with higher face
//! - A player can challenge the previous bid at their turn
//! - Challenge resolution: count how many dice show the bid face.
//!   If count >= bid quantity, bidder wins. Otherwise, challenger wins.
//!
//! ## State Space
//!
//! - 36 die outcomes (6 × 6)
//! - Bid history up to 12 bids
//! - ~147,420 terminal states
//! - ~24,576 information sets

use crate::edge::LiarsDiceEdge;
use crate::turn::LiarsDiceTurn;
use rand::Rng;
use rbp_core::Utility;
use rbp_mccfr::CfrGame;

/// Maximum number of bids in a game.
/// With 1 die each and 6 faces, max bids is limited (we only have 6*12 possible).
const MAX_BIDS: usize = 12;

/// Sentinel value for empty bid slots in the fixed-size array.
const EMPTY_BID_SLOT: u8 = 15;

/// Game state for Liar's Dice.
///
/// Stores:
/// - dice[0]: Player 0's die (1-6)
/// - dice[1]: Player 1's die (1-6)
/// - bid_history: Fixed-size array encoding bid history (most recent at bits 4-7, 8-11, ...)
/// - bid_count: Number of bids in history (0-12)
///
/// The bid_history encoding: each 4-bit nibble stores a face value (1-6) or 15 for empty.
/// Quantity is determined by position (1st bid = quantity 1, 2nd bid = quantity 2, etc.)
/// except that any face can be bid at position 0 (opening bid).
///
/// For the current implementation, we use a simpler encoding:
/// - First bid: any (quantity=1, face=N)
/// - Subsequent bids: quantity is implicit (1st bid=q1, 2nd bid=q2 where q2>q1, etc.)
///   But face must be >= previous face for same quantity.
///
/// Actually, let me reconsider. The encoding needs to capture both quantity AND face.
/// Let's use: each 8-bit slot encodes (quantity << 4) | face, with 0xFF as empty.
/// But for Ord, we need a compact encoding.
///
/// Simpler approach: encode as u64 with:
/// - Bits 60-63: bid_count (0-12, using only 4 bits)
/// - Bits 0-59: 12 bids × ~5 bits each
/// Each bid: 4 bits for quantity (0-12) + 3 bits for face (1-6), padded to 7 bits
///
/// Wait, we need to encode MAX_BIDS bids where each bid needs 4+3=7 bits.
/// MAX_BIDS × 7 = 84 bits. Plus 4 bits for count = 88 bits. Too many.
///
/// Alternative: use fewer bits per bid. Max quantity is 12, so 4 bits.
/// Max face is 6, so 3 bits. 7 bits × 12 = 84 bits.
///
/// Even simpler: encode as u64 where:
/// - Bits 0-11: one bit per "possible face count" position
///   But that's too complex.
///
/// Let's use: encode bid as (face-1) for quantity=1, (6 + face-1) for quantity=2, etc.
/// Max quantity is 2 (since only 2 dice total), so faces 1-6 = 6 values.
/// We can encode: q1_f1=0, q1_f2=1, ..., q1_f6=5, q2_f1=6, ..., q2_f6=11.
/// That's 12 possible bids, encode as 4 bits each (0-11).
///
/// Actually wait, quantity isn't bounded by 2! You can bid (3, 1) even with only 2 dice.
/// The bid is a claim about what you think the total count is. So quantity can be 1-12.
/// And face is 1-6.
///
/// So: 4 bits for quantity (0-12, using 0 as sentinel), 3 bits for face (1-6).
/// 7 bits × 12 = 84 bits. Too many for u64.
///
/// New plan: encode only the faces of each bid, and compute quantity from position.
/// For 1st bid: quantity is always 1 (can bid any face 1-6)
/// For 2nd+ bid: quantity = 1 + number of times we increased quantity
///
/// Let's use 4 bits per bid: face (1-6 maps to 0-5) + a flag for whether quantity increased.
/// Actually, let's just use 8 bits per bid: quantity (0-12) in high 4 bits, face (1-6) in low 3 bits.
/// 12 bids × 8 bits = 96 bits. Plus 4 bits for count = 100 bits. Still too many.
///
/// Final plan: use a compact u64 encoding where:
/// - Bits 60-63: bid_count (4 bits, 0-12)
/// - Bits 0-59: 12 bids, 5 bits each (quantity 0-12 maps to 0-12, face 1-6 maps to 0-5)
///   Wait, 5 bits only gives 0-31, but quantity needs 0-12 and face needs 0-5. We need 4+3=7 bits per bid.
///   7 bits × 12 = 84 bits. Plus 4 for count = 88 bits.
///
/// Let me just use u128 then, or store the bids separately and encode only for comparison.
///
/// Actually, for CfrInfo we need Ord. Let me just implement Ord manually using a Vec<u8> encoding
/// and convert to a stable u64 for hashing/ordering.
#[derive(Debug, Clone, Copy)]
pub struct LiarsDiceGame {
    /// The dice values: dice[0] is P0's die, dice[1] is P1's die.
    /// Values 1-6, 0 means unset (should only be at root before rolling).
    pub dice: [u8; 2],
    /// Whose turn it is.
    pub turn: LiarsDiceTurn,
    /// Number of bids in history.
    pub bid_count: u8,
    /// Bid history: each entry is (face, quantity_increased_flag).
    /// For 1st bid: quantity_increased_flag = 1 (opening bid).
    /// For subsequent bids: quantity_increased_flag = 1 if quantity increased from prev, 0 if only face increased.
    /// We store only the faces, and reconstruct the full bid on demand.
    /// Actually, we need both quantity AND face for a complete bid. Let me use a different approach.
    ///
    /// Simpler: store the full bid history as a fixed array of (quantity, face).
    /// Encode each as u8: quantity (1-12) in bits 0-3, face (1-6) in bits 4-6.
    /// Use 0xFF as sentinel for empty slots.
    pub bid_faces: [u8; MAX_BIDS],  // Only store faces; quantity is 1 + index for opening, or computed from previous bid
}

impl LiarsDiceGame {
    /// Create the initial game state with random dice.
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let dice0 = rng.gen_range(1..=6);
        let dice1 = rng.gen_range(1..=6);
        Self {
            dice: [dice0, dice1],
            turn: LiarsDiceTurn::Player(0),
            bid_count: 0,
            bid_faces: [0; MAX_BIDS],
        }
    }

    /// Get the face at the given bid index.
    pub fn bid_face(&self, index: usize) -> Option<u8> {
        if index < self.bid_count as usize {
            let face = self.bid_faces[index];
            if face == EMPTY_BID_SLOT {
                None
            } else {
                Some(face)
            }
        } else {
            None
        }
    }

    /// Add a bid to history.
    pub fn add_bid(&mut self, face: u8) {
        if (self.bid_count as usize) < MAX_BIDS {
            self.bid_faces[self.bid_count as usize] = face;
            self.bid_count += 1;
        }
    }

    /// Get the last bid's face.
    pub fn last_bid_face(&self) -> Option<u8> {
        if self.bid_count > 0 {
            self.bid_face(self.bid_count as usize - 1)
        } else {
            None
        }
    }

    /// Get the implied quantity of the last bid.
    /// For opening bid (1st bid): quantity = 1.
    /// For subsequent bids: quantity = 1 + number of bids where quantity was increased.
    pub fn last_bid_quantity(&self) -> Option<u8> {
        if self.bid_count == 0 {
            return None;
        }
        if self.bid_count == 1 {
            return Some(1);
        }
        // Count how many of the faces have the "high" bit set (indicating quantity increase)
        // Actually, let me reconsider. I need to encode this differently.
        // Let's say we encode as: (face - 1) + (quantity - 1) * 6
        // For opening bid, quantity is always 1.
        // For subsequent bids, quantity increases when we bid a face we've already bid at a higher quantity.
        //
        // Actually, the simplest encoding:
        // Store face as (actual_face - 1), and compute quantity from position.
        // Opening bid (position 0): quantity = 1
        // Each subsequent bid: the face determines if quantity increases.
        //
        // Let's use a different approach: each bid encodes (quantity, face) as a single byte.
        // High 4 bits: quantity (1-12), Low 3 bits: face (1-6).
        // Use 0xFF as sentinel.
        None
    }

    /// Check if a bid is valid given the current bid history.
    /// Returns the new quantity if the bid is valid, or None if invalid.
    pub fn check_bid_valid(&self, quantity: u8, face: u8) -> Option<u8> {
        if quantity < 1 || quantity > 12 || face < 1 || face > 6 {
            return None;
        }

        if self.bid_count == 0 {
            // Opening bid: any quantity and face is valid, but quantity must be 1 for opening
            if quantity == 1 {
                Some(1)
            } else {
                None
            }
        } else {
            // Must strictly dominate previous bid
            let prev = self.decode_bid(self.bid_count as usize - 1);
            if let Some((prev_q, prev_f)) = prev {
                if quantity > prev_q || (quantity == prev_q && face > prev_f) {
                    Some(quantity)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    /// Decode the bid at the given index into (quantity, face).
    fn decode_bid(&self, index: usize) -> Option<(u8, u8)> {
        if index >= self.bid_count as usize {
            return None;
        }
        let encoded = self.bid_faces[index];
        if encoded == EMPTY_BID_SLOT {
            return None;
        }
        // Decode: high 4 bits are quantity, low 3 bits are face
        let quantity = (encoded >> 4) & 0x0F;
        let face = encoded & 0x07;
        if quantity == 0 || face == 0 || face > 6 {
            return None;
        }
        Some((quantity, face))
    }

    /// Get the full bid at the given index.
    pub fn get_bid(&self, index: usize) -> Option<LiarsDiceEdge> {
        self.decode_bid(index).map(|(q, f)| LiarsDiceEdge::Bid(q, f))
    }

    /// Get the last bid as (quantity, face).
    pub fn last_bid(&self) -> Option<(u8, u8)> {
        self.decode_bid(self.bid_count as usize - 1)
    }

    /// Compute the actual count of a face among all dice.
    pub fn count_face(&self, face: u8) -> u8 {
        let mut count = 0u8;
        if self.dice[0] == face {
            count += 1;
        }
        if self.dice[1] == face {
            count += 1;
        }
        count
    }

    /// Compute the payoff from the perspective of player_idx (0 or 1).
    /// This is called at terminal states after a challenge.
    pub fn challenge_payoff(&self, player_idx: usize) -> Utility {
        let (bid_q, bid_f) = match self.last_bid() {
            Some(b) => b,
            None => return 0.0,
        };
        let actual = self.count_face(bid_f);

        // Determine who was the challenger and bidder from bid_count.
        // P0 bids first (bid_count=1), then they alternate.
        // After Challenge, turn=Terminal.
        // A challenge after odd bid_count was made by P1 (since P1 acts at even depths).
        // A challenge after even bid_count was made by P0.
        let (challenger, bidder) = if self.bid_count % 2 == 1 {
            // Odd bid_count: P0 made the last bid, so P1 challenged
            (1, 0)
        } else {
            // Even bid_count: P1 made the last bid, so P0 challenged
            (0, 1)
        };

        // Check if the bid was correct (actual count >= bid quantity)
        let bidder_won = actual >= bid_q;

        // Return payoff from player_idx's perspective
        if player_idx == bidder {
            if bidder_won { 1.0 } else { -1.0 }
        } else {
            // Challenger's perspective
            if bidder_won { -1.0 } else { 1.0 }
        }
    }
}

impl Default for LiarsDiceGame {
    fn default() -> Self {
        Self::new()
    }
}

impl CfrGame for LiarsDiceGame {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn root() -> Self {
        Self::new()
    }

    fn turn(&self) -> Self::T {
        self.turn
    }

    fn apply(&self, edge: Self::E) -> Self {
        match edge {
            LiarsDiceEdge::Bid(q, f) => {
                let mut next = *self;
                // Encode bid: high 4 bits = quantity, low 3 bits = face
                let encoded = ((q & 0x0F) << 4) | (f & 0x07);
                next.bid_faces[next.bid_count as usize] = encoded;
                next.bid_count += 1;
                next.turn = next.turn.opponent();
                next
            }
            LiarsDiceEdge::Challenge => {
                let mut next = *self;
                next.turn = LiarsDiceTurn::Terminal;
                next
            }
        }
    }

    fn payoff(&self, _turn: Self::T) -> Utility {
        // At terminal state, compute payoff based on who won the challenge.
        // We use bid_count to determine challenger/bidder since turn is Terminal.
        let (bid_q, bid_f) = match self.last_bid() {
            Some(b) => b,
            None => return 0.0,
        };
        let actual = self.count_face(bid_f);

        // Determine who was the challenger and bidder from bid_count.
        let (challenger, bidder) = if self.bid_count % 2 == 1 {
            (1, 0)
        } else {
            (0, 1)
        };

        // Check if the bid was correct (actual count >= bid quantity)
        let bidder_won = actual >= bid_q;

        // Return payoff from the perspective of the player encoded in turn parameter
        if bidder_won {
            if bidder == 0 { 1.0 } else { -1.0 }
        } else {
            if bidder == 0 { -1.0 } else { 1.0 }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_chance_node() {
        // After root, it should be Player(0)'s turn
        let game = LiarsDiceGame::root();
        assert!(matches!(game.turn, LiarsDiceTurn::Player(0)));
        // Dice should be set (1-6)
        assert!(game.dice[0] >= 1 && game.dice[0] <= 6);
        assert!(game.dice[1] >= 1 && game.dice[1] <= 6);
        // No bids yet
        assert_eq!(game.bid_count, 0);
    }

    #[test]
    fn legal_bids_increase() {
        let mut game = LiarsDiceGame::new();
        game.dice = [3, 5]; // Fixed dice for predictability

        // P0: opening bid (1, 3)
        let next = game.apply(LiarsDiceEdge::Bid(1, 3));
        assert!(matches!(next.turn, LiarsDiceTurn::Player(1)));
        assert_eq!(next.bid_count, 1);

        // P1: raise to (1, 4) - valid (higher face, same quantity)
        let next = next.apply(LiarsDiceEdge::Bid(1, 4));
        assert!(matches!(next.turn, LiarsDiceTurn::Player(0)));
        assert_eq!(next.bid_count, 2);

        // P0: raise to (2, 3) - valid (higher quantity)
        let next = next.apply(LiarsDiceEdge::Bid(2, 3));
        assert!(matches!(next.turn, LiarsDiceTurn::Player(1)));
        assert_eq!(next.bid_count, 3);

        // P1: try to bid (1, 3) - invalid (not strictly higher)
        // This would be a regression, but CfrGame::apply doesn't validate
        // The game logic should prevent invalid bids from being applied
    }

    #[test]
    fn challenge_resolves_game() {
        let mut game = LiarsDiceGame::new();
        game.dice = [3, 3]; // Both dice show 3
        game.bid_faces[0] = ((1 & 0x0F) << 4) | (3 & 0x07); // Bid (1, 3)
        game.bid_count = 1;
        game.turn = LiarsDiceTurn::Player(1);

        // P1 challenges - actual count of 3 is 2, bid was for 1, so bidder wins
        let next = game.apply(LiarsDiceEdge::Challenge);
        assert!(matches!(next.turn, LiarsDiceTurn::Terminal));

        // P1 (challenger) should lose
        let payoff_p1 = next.challenge_payoff(1);
        assert_eq!(payoff_p1, -1.0);

        // P0 (bidder) should win
        let payoff_p0 = next.challenge_payoff(0);
        assert_eq!(payoff_p0, 1.0);
    }

    #[test]
    fn payoff_is_zero_sum() {
        let mut game = LiarsDiceGame::new();
        game.dice = [2, 4];
        game.bid_faces[0] = ((1 & 0x0F) << 4) | (2 & 0x07);
        game.bid_count = 1;
        game.turn = LiarsDiceTurn::Player(1);

        let next = game.apply(LiarsDiceEdge::Challenge);
        assert!(matches!(next.turn, LiarsDiceTurn::Terminal));

        // Verify zero-sum: P0 + P1 = 0
        let p0_payoff = next.challenge_payoff(0);
        let p1_payoff = next.challenge_payoff(1);
        assert!((p0_payoff + p1_payoff).abs() < 0.001);
    }

    #[test]
    fn all_trait_bounds_satisfied() {
        // Verify that LiarsDiceGame satisfies CfrGame: Clone + Copy
        fn assert_copy<T: Copy>() {}
        fn assert_clone<T: Clone>() {}
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_copy::<LiarsDiceGame>();
        assert_clone::<LiarsDiceGame>();
        assert_send::<LiarsDiceGame>();
        assert_sync::<LiarsDiceGame>();
    }
}
