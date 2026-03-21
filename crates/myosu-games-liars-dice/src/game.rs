use crate::edge::{BidHistory, LiarsDiceEdge};
use crate::info::LiarsDiceInfo;
use crate::turn::LiarsDiceTurn;
use myosu_games::{CfrGame, CfrInfo, Utility};
use rbp_mccfr::CfrPublic;

/// Compact 2-player, 1-die-each Liar's Dice state.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDiceGame {
    dice: [u8; 2],
    bids: BidHistory,
    turn: LiarsDiceTurn,
    loser: Option<u8>,
}

impl LiarsDiceGame {
    pub fn dealt(p0: u8, p1: u8) -> Self {
        Self::root().apply(LiarsDiceEdge::roll(p0, p1))
    }

    pub fn die(&self, player: usize) -> Option<u8> {
        self.dice
            .get(player)
            .copied()
            .and_then(|die| (die != 0).then_some(die))
    }

    pub fn dice(&self) -> [u8; 2] {
        self.dice
    }

    pub(crate) fn bid_history(&self) -> BidHistory {
        self.bids
    }

    pub fn bids(&self) -> Vec<LiarsDiceEdge> {
        self.bids.edges()
    }

    pub fn last_bid(&self) -> Option<LiarsDiceEdge> {
        self.bids.last()
    }

    pub fn choices(&self) -> Vec<LiarsDiceEdge> {
        LiarsDiceInfo::from_game(self).public().choices()
    }

    pub fn count_face(&self, face: u8) -> u8 {
        self.dice.into_iter().filter(|die| *die == face).count() as u8
    }

    fn resolve_challenge(&self) -> u8 {
        let challenger = self
            .turn
            .player_index()
            .expect("challenge only valid at a player node") as u8;
        let LiarsDiceEdge::Bid { quantity, face } =
            self.last_bid().expect("challenge requires an existing bid")
        else {
            unreachable!("history only stores bids");
        };
        let truthful = self.count_face(face) >= quantity;
        if truthful { challenger } else { 1 - challenger }
    }
}

impl Default for LiarsDiceGame {
    fn default() -> Self {
        Self::root()
    }
}

impl CfrGame for LiarsDiceGame {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn root() -> Self {
        Self {
            dice: [0, 0],
            bids: BidHistory::default(),
            turn: LiarsDiceTurn::Chance,
            loser: None,
        }
    }

    fn turn(&self) -> Self::T {
        self.turn
    }

    fn apply(&self, edge: Self::E) -> Self {
        match (self.turn, edge) {
            (LiarsDiceTurn::Chance, LiarsDiceEdge::Roll { p0, p1 }) => Self {
                dice: [p0, p1],
                bids: self.bids,
                turn: LiarsDiceTurn::Player0,
                loser: None,
            },
            (LiarsDiceTurn::Player0 | LiarsDiceTurn::Player1, LiarsDiceEdge::Bid { .. }) => {
                let legal = self.choices();
                assert!(legal.contains(&edge), "bid must strictly increase");
                let mut bids = self.bids;
                bids.push(edge);
                Self {
                    dice: self.dice,
                    bids,
                    turn: self.turn.other_player(),
                    loser: None,
                }
            }
            (LiarsDiceTurn::Player0 | LiarsDiceTurn::Player1, LiarsDiceEdge::Challenge) => {
                assert!(!self.bids.is_empty(), "challenge requires an existing bid");
                Self {
                    dice: self.dice,
                    bids: self.bids,
                    turn: LiarsDiceTurn::Terminal,
                    loser: Some(self.resolve_challenge()),
                }
            }
            _ => panic!("illegal edge {:?} from {:?}", edge, self.turn),
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        let player = turn
            .player_index()
            .expect("payoff is only defined for players") as u8;
        let loser = self.loser.expect("payoff only defined at terminal nodes");
        if player == loser { -1.0 } else { 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LiarsDiceEdge::{Bid, Challenge};
    use myosu_games::{CfrEdge, CfrTurn};

    #[test]
    fn root_is_chance_node() {
        let root = LiarsDiceGame::root();
        assert!(root.turn().is_chance());

        let dealt = root.apply(LiarsDiceEdge::roll(2, 5));
        assert_eq!(dealt.turn(), LiarsDiceTurn::Player0);
        assert_eq!(dealt.dice(), [2, 5]);
    }

    #[test]
    fn legal_bids_increase() {
        let dealt = LiarsDiceGame::dealt(4, 6);
        let opening = dealt.choices();
        assert_eq!(opening.len(), 12);
        assert!(!opening.contains(&Challenge));
        assert_eq!(
            opening.first(),
            Some(&Bid {
                quantity: 1,
                face: 1
            })
        );
        assert_eq!(
            opening.last(),
            Some(&Bid {
                quantity: 2,
                face: 6
            })
        );

        let after_bid = dealt.apply(Bid {
            quantity: 1,
            face: 3,
        });
        let legal = after_bid.choices();
        assert!(legal.contains(&Challenge));
        assert!(!legal.contains(&Bid {
            quantity: 1,
            face: 3,
        }));
        assert_eq!(
            legal.first(),
            Some(&Bid {
                quantity: 1,
                face: 4
            })
        );
        assert!(
            legal
                .iter()
                .filter_map(LiarsDiceEdge::bid_index)
                .all(|rank| rank > 2),
            "every follow-up bid must exceed the previous bid"
        );
    }

    #[test]
    fn challenge_resolves_game() {
        let truthful = LiarsDiceGame::dealt(3, 5)
            .apply(Bid {
                quantity: 1,
                face: 3,
            })
            .apply(Challenge);
        assert_eq!(truthful.turn(), LiarsDiceTurn::Terminal);
        assert_eq!(truthful.payoff(LiarsDiceTurn::Player0), 1.0);
        assert_eq!(truthful.payoff(LiarsDiceTurn::Player1), -1.0);

        let bluff = LiarsDiceGame::dealt(3, 5)
            .apply(Bid {
                quantity: 2,
                face: 1,
            })
            .apply(Challenge);
        assert_eq!(bluff.turn(), LiarsDiceTurn::Terminal);
        assert_eq!(bluff.payoff(LiarsDiceTurn::Player0), -1.0);
        assert_eq!(bluff.payoff(LiarsDiceTurn::Player1), 1.0);
    }

    #[test]
    fn payoff_is_zero_sum() {
        let terminals = [
            LiarsDiceGame::dealt(2, 2)
                .apply(Bid {
                    quantity: 2,
                    face: 2,
                })
                .apply(Challenge),
            LiarsDiceGame::dealt(1, 6)
                .apply(Bid {
                    quantity: 2,
                    face: 3,
                })
                .apply(Challenge),
        ];

        for game in terminals {
            let total = game.payoff(LiarsDiceTurn::Player0) + game.payoff(LiarsDiceTurn::Player1);
            assert_eq!(total, 0.0);
        }
    }

    #[test]
    fn all_trait_bounds_satisfied() {
        fn assert_edge<T: CfrEdge>() {}
        fn assert_turn<T: CfrTurn>() {}
        fn assert_info<T: CfrInfo<E = LiarsDiceEdge, T = LiarsDiceTurn>>() {}
        fn assert_game<T: CfrGame<E = LiarsDiceEdge, T = LiarsDiceTurn>>() {}
        fn assert_copy<T: Copy + Clone + Send + Sync>() {}

        assert_edge::<LiarsDiceEdge>();
        assert_turn::<LiarsDiceTurn>();
        assert_info::<LiarsDiceInfo>();
        assert_game::<LiarsDiceGame>();
        assert_copy::<LiarsDiceGame>();
    }
}
