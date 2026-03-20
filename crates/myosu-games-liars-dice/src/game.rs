//! Game state for Liar's Dice.
//!
//! `LiarsDiceGame` implements `CfrGame`, representing the memoryless game state
//! for CFR traversal. The state tracks:
//! - `dice: [u8; 2]` — the two players' die values (1-6 each)
//! - `acting: u8` — acting player index (0, 1), or 2 for terminal
//! - `challenger: Option<u8>` — who performed the challenge (stored before going terminal)
//! - `history: BidHistory` — the bid sequence so far
//!
//! The initial state uses a fixed roll of [1, 1] — CFR requires the root
//! to be deterministic for the information set structure to be well-defined.

use crate::edge::LiarsDiceEdge;
use crate::info::{BidHistory, LiarsDiceInfo};
use crate::turn::LiarsDiceTurn;
use rbp_core::Utility;
use rbp_mccfr::CfrGame;
use std::fmt;

/// The fixed starting dice roll for the CFR root.
const ROOT_DICE: [u8; 2] = [1, 1];

/// Liar's Dice game state.
///
/// The `acting` field tracks the current decision-maker:
/// - 0 = Player 0's turn
/// - 1 = Player 1's turn
/// - 2 = Terminal (game over)
///
/// When `acting = 2` (terminal), `challenger` stores who initiated the
/// challenge, which is needed to compute the payoff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiarsDiceGame {
    dice: [u8; 2],
    /// Acting player: 0 or 1 for decisions, 2 for terminal.
    acting: u8,
    /// Who performed the challenge (only valid when acting = 2).
    challenger: Option<u8>,
    history: BidHistory,
}

impl LiarsDiceGame {
    /// Root state: fixed roll [1,1], player 0 acts first.
    pub fn root() -> Self {
        Self {
            dice: ROOT_DICE,
            acting: 0,
            challenger: None,
            history: BidHistory::new(),
        }
    }

    /// Get player 0's die.
    pub fn player0_die(&self) -> u8 {
        self.dice[0]
    }

    /// Get player 1's die.
    pub fn player1_die(&self) -> u8 {
        self.dice[1]
    }

    /// Get the die for a specific player.
    pub fn die_of(&self, player: usize) -> u8 {
        self.dice[player]
    }

    /// Get the acting player's index (0 or 1).
    pub fn acting_player(&self) -> u8 {
        self.acting
    }

    /// True if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        self.acting == 2
    }

    /// Get the bid history.
    pub fn history(&self) -> BidHistory {
        self.history
    }

    /// Count how many dice show a given face.
    fn count_face(&self, face: u8) -> u8 {
        self.dice.iter().filter(|&&d| d == face).count() as u8
    }

    /// Get the information set for a player at this state.
    pub fn info(&self, player: usize) -> LiarsDiceInfo {
        LiarsDiceInfo::new(self.dice[player], self.history)
    }

    /// Compute the payoff from player 0's perspective.
    fn payoff_from_p0(&self) -> Utility {
        debug_assert!(self.is_terminal(), "payoff only valid at terminal");
        let challenger = self.challenger.expect("challenger set at terminal");
        let last_bid = self.history.last_bid().expect("terminal implies bids exist");

        let (qty, face) = last_bid;
        let actual = self.count_face(face);

        if actual >= qty {
            // Bid is true: challenger loses
            if challenger == 0 {
                -1.0 // P0 challenged and lost
            } else {
                1.0 // P1 challenged and lost, P0 wins
            }
        } else {
            // Bid is false: challenger wins
            if challenger == 0 {
                1.0 // P0 challenged and won
            } else {
                -1.0 // P1 challenged and won, P0 loses
            }
        }
    }
}

impl CfrGame for LiarsDiceGame {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn root() -> Self {
        Self::root()
    }

    fn turn(&self) -> Self::T {
        match self.acting {
            0 => LiarsDiceTurn::Player0,
            1 => LiarsDiceTurn::Player1,
            _ => LiarsDiceTurn::Terminal,
        }
    }

    fn apply(&self, edge: Self::E) -> Self {
        match edge {
            LiarsDiceEdge::Bid { quantity, face } => {
                let mut new_history = self.history;
                new_history.push(quantity, face);
                Self {
                    dice: self.dice,
                    acting: 1 - self.acting, // switch to other player
                    challenger: None,
                    history: new_history,
                }
            }
            LiarsDiceEdge::Challenge => {
                Self {
                    dice: self.dice,
                    acting: 2, // terminal
                    challenger: Some(self.acting), // who just acted (the challenger)
                    history: self.history,
                }
            }
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        debug_assert!(self.is_terminal(), "payoff only valid at terminal");
        let p0_payoff = self.payoff_from_p0();

        // Convert from P0's perspective to the requested player's perspective
        match turn {
            LiarsDiceTurn::Player0 => p0_payoff,
            LiarsDiceTurn::Player1 => -p0_payoff,
            LiarsDiceTurn::Terminal => 0.0,
            LiarsDiceTurn::Chance => 0.0, // shouldn't happen
        }
    }
}

impl fmt::Display for LiarsDiceGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Game(dice=[{}, {}], acting={:?}, history={:?})",
            self.dice[0],
            self.dice[1],
            self.acting,
            self.history
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The root turn is Player(0) — dice are fixed at [1,1] so there's no
    /// chance sampling; the first player acts immediately.
    #[test]
    fn root_is_chance_node() {
        let game = LiarsDiceGame::root();
        assert_eq!(game.turn(), LiarsDiceTurn::Player0);
        assert_eq!(game.player0_die(), 1);
        assert_eq!(game.player1_die(), 1);
    }

    /// After a bid, the next player must strictly increase quantity or face.
    #[test]
    fn legal_bids_increase() {
        let game = LiarsDiceGame::root();

        // First player has 36 possible bids + challenge
        let choices = game.info(0).choices();
        assert!(choices.len() >= 36, "expected 36 bids + challenge");

        // After P0 bids (1,1), P1's choices
        let after_p0_bid = game.apply(LiarsDiceEdge::Bid { quantity: 1, face: 1 });
        let p1_choices = after_p0_bid.info(1).choices();

        // (1,1) again is NOT valid (same bid)
        assert!(!p1_choices.contains(&LiarsDiceEdge::Bid { quantity: 1, face: 1 }));

        // (1,2) IS valid — face increased
        assert!(p1_choices.contains(&LiarsDiceEdge::Bid { quantity: 1, face: 2 }));

        // (2,1) IS valid — quantity increased
        assert!(p1_choices.contains(&LiarsDiceEdge::Bid { quantity: 2, face: 1 }));

        // Challenge is always available
        assert!(p1_choices.contains(&LiarsDiceEdge::Challenge));
    }

    /// Applying Challenge transitions to Terminal turn.
    #[test]
    fn challenge_resolves_game() {
        let game = LiarsDiceGame::root();
        let after_bid = game.apply(LiarsDiceEdge::Bid { quantity: 1, face: 1 });
        let after_challenge = after_bid.apply(LiarsDiceEdge::Challenge);

        assert_eq!(after_challenge.turn(), LiarsDiceTurn::Terminal);
    }

    /// In zero-sum Liar's Dice, P0 payoff + P1 payoff = 0 at every terminal.
    #[test]
    fn payoff_is_zero_sum() {
        // Scenario: dice = [1, 1], P0 bids (1,1), P1 challenges
        // Actual count of face-1 = 2 >= 1 → bid is true → challenger loses
        // P1 challenged and lost → P1 payoff = -1, P0 payoff = +1
        let game = LiarsDiceGame::root();
        let after_bid = game.apply(LiarsDiceEdge::Bid { quantity: 1, face: 1 });
        let terminal = after_bid.apply(LiarsDiceEdge::Challenge);

        let p0_payoff = terminal.payoff(LiarsDiceTurn::Player0);
        let p1_payoff = terminal.payoff(LiarsDiceTurn::Player1);

        assert!((p0_payoff + p1_payoff).abs() < 1e-6, "zero-sum violated: {} + {} ≠ 0", p0_payoff, p1_payoff);
    }

    /// Verify all required trait bounds for CfrGame are satisfied.
    #[test]
    fn all_trait_bounds_satisfied() {
        fn assert_copy<T: Copy>() {}
        fn assert_clone<T: Clone + Copy>() {}
        fn assert_send_sync<T: Send + Sync>() {}

        assert_copy::<LiarsDiceGame>();
        assert_clone::<LiarsDiceGame>();
        assert_send_sync::<LiarsDiceGame>();
    }
}
