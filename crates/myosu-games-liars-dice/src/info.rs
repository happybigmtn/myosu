use crate::edge::{LiarsDiceEdge, MAX_BIDS, NO_BID};
use crate::game::LiarsDiceGame;
use crate::turn::LiarsDiceTurn;
use myosu_games::{CfrGame, CfrInfo};
use rbp_mccfr::{CfrPublic, CfrSecret};
use rbp_transport::Support;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDicePublic {
    turn: LiarsDiceTurn,
    bids: [u8; MAX_BIDS],
    bid_count: u8,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDiceSecret(u8);

/// Acting player's information set: their private die plus the public bid path.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDiceInfo {
    public: LiarsDicePublic,
    secret: LiarsDiceSecret,
}

impl LiarsDiceInfo {
    pub fn from_game(game: &LiarsDiceGame) -> Self {
        let secret = match game.turn().actor_index() {
            Some(actor) => LiarsDiceSecret(game.die(actor)),
            None => LiarsDiceSecret(0),
        };

        Self {
            public: LiarsDicePublic {
                turn: game.turn(),
                bids: game.bid_ranks(),
                bid_count: game.bid_count(),
            },
            secret,
        }
    }

    pub fn my_die(&self) -> Option<u8> {
        match self.secret.0 {
            0 => None,
            die => Some(die),
        }
    }

    pub fn bid_history(&self) -> Vec<LiarsDiceEdge> {
        self.public.history()
    }
}

impl CfrPublic for LiarsDicePublic {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn choices(&self) -> Vec<Self::E> {
        match self.turn {
            LiarsDiceTurn::Chance => LiarsDiceEdge::all_rolls(),
            LiarsDiceTurn::Player0 | LiarsDiceTurn::Player1 => {
                let last_rank = self.last_bid_rank().unwrap_or(NO_BID);
                let mut choices: Vec<_> = LiarsDiceEdge::all_bids()
                    .into_iter()
                    .skip(last_rank as usize)
                    .collect();
                if self.bid_count > 0 {
                    choices.push(LiarsDiceEdge::Challenge);
                }
                choices
            }
            LiarsDiceTurn::Terminal => Vec::new(),
        }
    }

    fn history(&self) -> Vec<Self::E> {
        self.bids[..self.bid_count as usize]
            .iter()
            .copied()
            .filter(|rank| *rank != NO_BID)
            .map(LiarsDiceEdge::from_bid_rank)
            .collect()
    }
}

impl LiarsDicePublic {
    fn last_bid_rank(&self) -> Option<u8> {
        (self.bid_count > 0).then(|| self.bids[(self.bid_count - 1) as usize])
    }
}

impl Support for LiarsDiceSecret {}
impl CfrSecret for LiarsDiceSecret {}

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
