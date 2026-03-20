//! Turn representation for Liar's Dice.

use rbp_mccfr::CfrTurn;
use std::fmt;

/// Turn node type for Liar's Dice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LiarsDiceTurn {
    Chance,
    Player0,
    Player1,
    Terminal,
}

impl From<usize> for LiarsDiceTurn {
    fn from(n: usize) -> Self {
        match n {
            0 => Self::Player0,
            1 => Self::Player1,
            _ => panic!("LiarsDiceTurn::from({n}) — only 0 or 1 valid"),
        }
    }
}

impl LiarsDiceTurn {
    pub fn player_index(&self) -> Option<usize> {
        match self {
            Self::Player0 => Some(0),
            Self::Player1 => Some(1),
            _ => None,
        }
    }

    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player0 | Self::Player1)
    }
}

impl CfrTurn for LiarsDiceTurn {
    fn chance() -> Self {
        Self::Chance
    }

    fn terminal() -> Self {
        Self::Terminal
    }
}

impl fmt::Display for LiarsDiceTurn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chance => write!(f, "Chance"),
            Self::Player0 => write!(f, "Player(0)"),
            Self::Player1 => write!(f, "Player(1)"),
            Self::Terminal => write!(f, "Terminal"),
        }
    }
}
