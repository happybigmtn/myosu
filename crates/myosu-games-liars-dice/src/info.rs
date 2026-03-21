use crate::edge::{BidHistory, LiarsDiceEdge};
use crate::game::LiarsDiceGame;
use crate::turn::LiarsDiceTurn;
use myosu_games::{CfrGame, CfrInfo};
use rbp_mccfr::{CfrPublic, CfrSecret};
use rbp_transport::Support;

/// Acting-player information set: visible bid history plus the player's die.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDiceInfo {
    public: LiarsDicePublic,
    secret: LiarsDiceSecret,
}

impl LiarsDiceInfo {
    pub fn from_game(game: &LiarsDiceGame) -> Self {
        let secret = game
            .turn()
            .player_index()
            .and_then(|player| game.die(player))
            .unwrap_or(0);
        Self {
            public: LiarsDicePublic::from_game(game),
            secret: LiarsDiceSecret(secret),
        }
    }

    pub fn my_die(&self) -> Option<u8> {
        self.secret.face()
    }
}

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

/// Public information visible to both players: turn marker and bid history.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDicePublic {
    turn: LiarsDiceTurn,
    history: BidHistory,
}

impl LiarsDicePublic {
    pub(crate) fn from_game(game: &LiarsDiceGame) -> Self {
        Self {
            turn: game.turn(),
            history: game.bid_history(),
        }
    }
}

impl CfrPublic for LiarsDicePublic {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn choices(&self) -> Vec<Self::E> {
        match self.turn {
            LiarsDiceTurn::Chance => LiarsDiceEdge::all_rolls(),
            LiarsDiceTurn::Terminal => Vec::new(),
            LiarsDiceTurn::Player0 | LiarsDiceTurn::Player1 => {
                let mut choices = LiarsDiceEdge::bids_after(self.history.last());
                if !self.history.is_empty() {
                    choices.push(LiarsDiceEdge::Challenge);
                }
                choices
            }
        }
    }

    fn history(&self) -> Vec<Self::E> {
        self.history.edges()
    }
}

/// Private information for the acting player: their die face, or `0` when none.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDiceSecret(u8);

impl LiarsDiceSecret {
    pub fn face(&self) -> Option<u8> {
        (self.0 != 0).then_some(self.0)
    }
}

impl Support for LiarsDiceSecret {}
impl CfrSecret for LiarsDiceSecret {}
