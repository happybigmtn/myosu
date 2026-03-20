//! Liar's Dice CfrGame implementation.

use myosu_games::{CfrGame, CfrTurn, Utility};
use serde::{Deserialize, Serialize};

use crate::edge::{Bid, LiarsDiceEdge, MAX_BIDS};
use crate::turn::LiarsDiceTurn;

/// Maximum dice count in the game (1 die per player, 2 players).
pub const NUM_DICE: usize = 2;
/// Number of faces on each die.
pub const NUM_FACES: u8 = 6;

/// Liar's Dice game state.
///
/// This struct must be `Copy` because `CfrGame` requires it.
/// The bid history uses a fixed-size array with SEALED sentinel values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LiarsDiceGame {
    /// Each player's die value (1-6). Values of 0 mean "not rolled yet".
    dice: [u8; NUM_DICE],
    /// Bid history with SEALED sentinel for empty slots.
    bids: [Bid; MAX_BIDS],
    /// Number of valid bids in history.
    num_bids: u8,
    /// Current player: 0 or 1. Invalid after terminal.
    player: u8,
}

impl LiarsDiceGame {
    /// Create a new game at the initial chance node (pre-roll).
    pub fn new() -> Self {
        Self {
            dice: [0; NUM_DICE],
            bids: [Bid::SEALED; MAX_BIDS],
            num_bids: 0,
            player: 0,
        }
    }

    /// Check if this game state is at the root (pre-roll).
    pub fn is_root(&self) -> bool {
        self.dice[0] == 0 && self.dice[1] == 0 && self.num_bids == 0
    }

    /// Roll both dice (chance outcome).
    pub fn roll(&mut self, die0: u8, die1: u8) {
        self.dice = [die0, die1];
    }

    /// Get a player's die value.
    pub fn die(&self, player: usize) -> u8 {
        self.dice[player]
    }

    /// Get the last bid, if any.
    pub fn last_bid(&self) -> Option<Bid> {
        if self.num_bids == 0 {
            None
        } else {
            Some(self.bids[(self.num_bids - 1) as usize])
        }
    }

    /// Check if a bid is legal (proper raise or first bid).
    pub fn is_legal_bid(&self, bid: &Bid) -> bool {
        if let Some(last) = self.last_bid() {
            // Must strictly increase quantity or face
            bid.quantity > last.quantity
                || (bid.quantity == last.quantity && bid.face > last.face)
        } else {
            // First bid: any (q >= 1, f >= 1, q <= 2) is valid
            bid.quantity >= 1 && bid.face >= 1 && bid.quantity <= 2
        }
    }

    /// Count how many dice show a given face value.
    pub fn count_dice_showing(&self, face: u8) -> u8 {
        let mut count = 0u8;
        for &die in &self.dice {
            if die == face {
                count += 1;
            }
        }
        count
    }

    /// Check if the last bid is true (met or exceeded by actual dice).
    pub fn last_bid_is_true(&self) -> bool {
        if let Some(bid) = self.last_bid() {
            self.count_dice_showing(bid.face) >= bid.quantity
        } else {
            // No bid to challenge
            false
        }
    }

    /// Apply an action (bid or challenge) to this game state.
    pub fn apply_action(&mut self, edge: LiarsDiceEdge) {
        match edge {
            LiarsDiceEdge::Bid(bid) => {
                if self.num_bids < MAX_BIDS as u8 {
                    self.bids[self.num_bids as usize] = bid;
                    self.num_bids += 1;
                    self.player = 1 - self.player;
                }
            }
            LiarsDiceEdge::Challenge => {
                // Game resolves at terminal
                self.player = 2; // Invalid player marks terminal
            }
        }
    }

    /// Compute payoff for a specific player.
    fn payoff_for_player(&self, player: usize) -> Utility {
        // Player 2 (invalid) marks terminal after challenge
        // The player who CHALLENGED wins if the bid was false
        // The challenged player wins if the bid was true
        let last_bid = self.last_bid();

        if last_bid.is_none() {
            // No bid was made: shouldn't happen in normal play
            return 0.0;
        }

        let bid_true = self.last_bid_is_true();

        // The player who would act next (self.player) is the challenger
        // The player who just acted (1 - self.player) made the last bid
        let challenger = self.player.min(1);
        let challenged_player = 1 - challenger;

        let challenger_wins = !bid_true;
        let payoff = if challenger_wins {
            if challenger == 0 { 1.0 } else { -1.0 }
        } else {
            // Bid was true: challenged player wins
            if challenged_player == 0 { 1.0 } else { -1.0 }
        };

        // Return payoff from the perspective of the requested player
        if player == 0 {
            payoff
        } else {
            -payoff
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
        if self.player >= 2 {
            LiarsDiceTurn::Terminal
        } else if self.dice[0] == 0 {
            LiarsDiceTurn::Chance
        } else {
            LiarsDiceTurn::from(self.player as usize)
        }
    }

    fn apply(&self, edge: Self::E) -> Self {
        let mut next = *self;
        next.apply_action(edge);
        next
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        // The `turn` parameter is which player's payoff we want: Player0 or Player1
        // It's NOT the game state - use self.turn() for that
        match turn {
            LiarsDiceTurn::Player0 => self.payoff_for_player(0),
            LiarsDiceTurn::Player1 => self.payoff_for_player(1),
            _ => unreachable!("payoff called with non-player turn: {:?}", turn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_is_chance_node() {
        let game = LiarsDiceGame::new();
        assert!(game.turn() == LiarsDiceTurn::Chance);
    }

    #[test]
    fn legal_bids_increase() {
        let mut game = LiarsDiceGame::new();
        game.roll(3, 5);

        // First bid can be anything valid
        let first = Bid::new(1, 3);
        assert!(game.is_legal_bid(&first));

        // Apply first bid
        let game_after_bid = game.apply(LiarsDiceEdge::Bid(first));
        assert_eq!(game_after_bid.turn(), LiarsDiceTurn::Player1);

        // Raise must increase
        let raise = Bid::new(1, 4);
        assert!(game_after_bid.is_legal_bid(&raise));

        let raise2 = Bid::new(2, 3);
        assert!(game_after_bid.is_legal_bid(&raise2));

        // Same bid is illegal
        let same = Bid::new(1, 3);
        assert!(!game_after_bid.is_legal_bid(&same));

        // Lower is illegal
        let lower = Bid::new(1, 2);
        assert!(!game_after_bid.is_legal_bid(&lower));
    }

    #[test]
    fn challenge_resolves_game() {
        let mut game = LiarsDiceGame::new();
        game.roll(3, 5);

        // Bid that both dice are 3s - which is false (only one is 3)
        let game_after_bid = game.apply(LiarsDiceEdge::Bid(Bid::new(2, 3)));
        assert!(game_after_bid.turn() != LiarsDiceTurn::Terminal);

        // Challenge
        let game_after_challenge = game_after_bid.apply(LiarsDiceEdge::Challenge);
        assert!(game_after_challenge.turn() == LiarsDiceTurn::Terminal);

        // Challenger (player 1, who would act next) should win
        let payoff_p0 = game_after_challenge.payoff(LiarsDiceTurn::from(0));
        let payoff_p1 = game_after_challenge.payoff(LiarsDiceTurn::from(1));
        assert_eq!(payoff_p0, -1.0);
        assert_eq!(payoff_p1, 1.0);
    }

    #[test]
    fn payoff_is_zero_sum() {
        let mut game = LiarsDiceGame::new();
        game.roll(3, 3);

        // Bid two 3s - which is true (both dice are 3)
        let game_after_bid = game.apply(LiarsDiceEdge::Bid(Bid::new(2, 3)));
        let game_after_challenge = game_after_bid.apply(LiarsDiceEdge::Challenge);

        assert!(game_after_challenge.turn() == LiarsDiceTurn::Terminal);

        // Challenged player (player 1 who made the bid) wins
        let payoff_p0 = game_after_challenge.payoff(LiarsDiceTurn::from(0));
        let payoff_p1 = game_after_challenge.payoff(LiarsDiceTurn::from(1));
        assert_eq!(payoff_p0, 1.0);
        assert_eq!(payoff_p1, -1.0);
        assert_eq!(payoff_p0, -payoff_p1); // Zero-sum
    }

    #[test]
    fn all_trait_bounds_satisfied() {
        fn assert_copy<T: Copy>() {}
        fn assert_cfr_game<T: CfrGame>() {}

        assert_copy::<LiarsDiceGame>();
        assert_cfr_game::<LiarsDiceGame>();
    }
}
