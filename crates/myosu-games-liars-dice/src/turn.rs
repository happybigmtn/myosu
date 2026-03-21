use myosu_games::CfrTurn;
use rbp_transport::Support;

/// Node classification for the one-die Liar's Dice game tree.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiarsDiceTurn {
    Chance,
    Player0,
    Player1,
    Terminal,
}

impl LiarsDiceTurn {
    pub(crate) fn actor_index(self) -> Option<usize> {
        match self {
            Self::Player0 => Some(0),
            Self::Player1 => Some(1),
            Self::Chance | Self::Terminal => None,
        }
    }
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
