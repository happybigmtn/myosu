//! Turn type for Liar's Dice.
//!
//! The turn indicates whose decision it is: Player(0), Player(1), or Terminal.

use rbp_mccfr::CfrTurn;
use rbp_transport::Support;
use serde::{Deserialize, Serialize};

/// Player or terminal indicator for Liar's Dice.
///
/// The game proceeds: dice are rolled at root, then players alternate
/// bidding until one challenges. The turn tracks whose decision it is.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LiarsDiceTurn {
    /// Player 0's turn (first player, also called "challenger" in some contexts)
    Player(usize),
    /// Terminal state (game over after challenge)
    Terminal,
}

impl From<usize> for LiarsDiceTurn {
    fn from(player: usize) -> Self {
        match player {
            0 => Self::Player(0),
            1 => Self::Player(1),
            _ => panic!("Liar's Dice only has 2 players"),
        }
    }
}

impl Support for LiarsDiceTurn {}
impl CfrTurn for LiarsDiceTurn {
    fn chance() -> Self {
        Self::Terminal
    }
    fn terminal() -> Self {
        Self::Terminal
    }
}

impl LiarsDiceTurn {
    /// Returns true if this is Player(0)'s turn.
    pub fn is_player0(&self) -> bool {
        matches!(self, Self::Player(0))
    }

    /// Returns true if this is Player(1)'s turn.
    pub fn is_player1(&self) -> bool {
        matches!(self, Self::Player(1))
    }

    /// Returns the player index (0 or 1) if this is a player turn.
    pub fn player_index(&self) -> Option<usize> {
        match self {
            Self::Player(p) => Some(*p),
            _ => None,
        }
    }

    /// Returns the opponent's turn.
    pub fn opponent(&self) -> Self {
        match self {
            Self::Player(0) => Self::Player(1),
            Self::Player(1) => Self::Player(0),
            Self::Player(n) => Self::Player(1 - n),
            Self::Terminal => Self::Terminal,
        }
    }
}
