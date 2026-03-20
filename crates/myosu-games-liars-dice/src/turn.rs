//! Liar's Dice turn representation.

use myosu_games::CfrTurn;
use rbp_transport::Support;
use serde::{Deserialize, Serialize};

/// Turn in Liar's Dice: either a player's turn, chance (die roll), or terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiarsDiceTurn {
    /// Chance node: roll the dice before play begins
    Chance,
    /// Player 0's turn to act
    Player0,
    /// Player 1's turn to act
    Player1,
    /// Terminal state (game over after challenge)
    Terminal,
}

impl From<usize> for LiarsDiceTurn {
    fn from(player: usize) -> Self {
        match player {
            0 => Self::Player0,
            1 => Self::Player1,
            _ => panic!("Liar's Dice only has 2 players"),
        }
    }
}

impl Support for LiarsDiceTurn {}
impl CfrTurn for LiarsDiceTurn {
    fn chance() -> Self {
        Self::Chance
    }

    fn terminal() -> Self {
        Self::Terminal
    }
}
