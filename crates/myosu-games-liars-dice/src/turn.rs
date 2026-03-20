//! Liar's Dice turns — `CfrTurn` implementation.
//!
//! Indicates whose turn it is: Player(0), Player(1), Chance, or Terminal.

/// LiarsDiceTurn indicates whose turn it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiarsDiceTurn {
    /// Player 0's turn to act
    Player(u8),
    /// Chance node (dice roll at game start)
    Chance,
    /// Game has ended
    Terminal,
}

impl LiarsDiceTurn {
    /// Return true if this is a player turn.
    pub fn is_player(&self) -> bool {
        matches!(self, LiarsDiceTurn::Player(_))
    }

    /// Return the player index if this is a player turn.
    pub fn player(&self) -> Option<u8> {
        match self {
            LiarsDiceTurn::Player(p) => Some(*p),
            _ => None,
        }
    }
}
