use crate::edge::{LiarsDiceEdge, MAX_BIDS, NO_BID, NO_DIE, is_valid_die};
use crate::info::LiarsDiceInfo;
use crate::turn::LiarsDiceTurn;
use myosu_games::{CfrGame, Utility};

const UNRESOLVED_WINNER: u8 = u8::MAX;

/// Compact, `Copy`-safe game state for one-die-per-player Liar's Dice.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LiarsDiceGame {
    dice: [u8; 2],
    bids: [u8; MAX_BIDS],
    bid_count: u8,
    turn: LiarsDiceTurn,
    winner: u8,
}

impl LiarsDiceGame {
    pub fn dealt(player0: u8, player1: u8) -> Self {
        assert!(is_valid_die(player0), "player0 die must be 1..=6");
        assert!(is_valid_die(player1), "player1 die must be 1..=6");
        Self {
            dice: [player0, player1],
            bids: [NO_BID; MAX_BIDS],
            bid_count: 0,
            turn: LiarsDiceTurn::Player0,
            winner: UNRESOLVED_WINNER,
        }
    }

    pub fn die(&self, player: usize) -> u8 {
        self.dice[player]
    }

    pub fn bid_count(&self) -> u8 {
        self.bid_count
    }

    pub fn bid_ranks(&self) -> [u8; MAX_BIDS] {
        self.bids
    }

    pub fn bid_history(&self) -> Vec<LiarsDiceEdge> {
        self.bids[..self.bid_count as usize]
            .iter()
            .copied()
            .filter(|rank| *rank != NO_BID)
            .map(LiarsDiceEdge::from_bid_rank)
            .collect()
    }

    pub fn info_set(&self) -> LiarsDiceInfo {
        LiarsDiceInfo::from_game(self)
    }

    pub fn winner(&self) -> Option<usize> {
        (self.turn == LiarsDiceTurn::Terminal).then_some(self.winner as usize)
    }

    fn last_bid_rank(&self) -> Option<u8> {
        (self.bid_count > 0).then(|| self.bids[(self.bid_count - 1) as usize])
    }

    fn is_legal_bid(&self, edge: LiarsDiceEdge) -> bool {
        let Some(rank) = edge.bid_rank() else {
            return false;
        };
        match self.last_bid_rank() {
            Some(last_rank) => rank > last_rank,
            None => true,
        }
    }

    fn next_player(turn: LiarsDiceTurn) -> LiarsDiceTurn {
        match turn {
            LiarsDiceTurn::Player0 => LiarsDiceTurn::Player1,
            LiarsDiceTurn::Player1 => LiarsDiceTurn::Player0,
            _ => panic!("only player turns have a next player"),
        }
    }

    fn resolve_challenge(&self, challenger: usize) -> usize {
        let LiarsDiceEdge::Bid { quantity, face } =
            LiarsDiceEdge::from_bid_rank(self.last_bid_rank().expect("challenge after bid"))
        else {
            unreachable!("bid rank always decodes to a bid");
        };

        let actual = self.dice.iter().copied().filter(|die| *die == face).count() as u8;
        let bidder = 1 - challenger;
        if actual >= quantity {
            bidder
        } else {
            challenger
        }
    }
}

impl CfrGame for LiarsDiceGame {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn root() -> Self {
        Self {
            dice: [NO_DIE; 2],
            bids: [NO_BID; MAX_BIDS],
            bid_count: 0,
            turn: LiarsDiceTurn::Chance,
            winner: UNRESOLVED_WINNER,
        }
    }

    fn turn(&self) -> Self::T {
        self.turn
    }

    fn apply(&self, edge: Self::E) -> Self {
        match (self.turn, edge) {
            (LiarsDiceTurn::Chance, LiarsDiceEdge::Roll { player0, player1 })
                if edge.is_valid_roll() =>
            {
                Self::dealt(player0, player1)
            }
            (LiarsDiceTurn::Player0 | LiarsDiceTurn::Player1, LiarsDiceEdge::Bid { .. })
                if self.is_legal_bid(edge) =>
            {
                let mut next = *self;
                next.bids[next.bid_count as usize] =
                    edge.bid_rank().expect("validated bid has rank");
                next.bid_count += 1;
                next.turn = Self::next_player(self.turn);
                next
            }
            (LiarsDiceTurn::Player0 | LiarsDiceTurn::Player1, LiarsDiceEdge::Challenge)
                if self.bid_count > 0 =>
            {
                let mut next = *self;
                let challenger = self
                    .turn
                    .actor_index()
                    .expect("challenge only valid on player turns");
                next.turn = LiarsDiceTurn::Terminal;
                next.winner = self.resolve_challenge(challenger) as u8;
                next
            }
            _ => panic!(
                "invalid Liar's Dice transition: {:?} via {:?}",
                self.turn, edge
            ),
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        assert_eq!(
            self.turn,
            LiarsDiceTurn::Terminal,
            "payoff only exists at terminal nodes"
        );

        let player = turn
            .actor_index()
            .expect("payoff must be queried for a player turn");
        if self.winner == player as u8 {
            1.0
        } else {
            -1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LiarsDiceEdge, LiarsDiceTurn};
    use myosu_games::{CfrEdge, CfrGame, CfrInfo, CfrTurn};
    use rbp_mccfr::{CfrPublic, CfrSecret};

    #[test]
    fn root_is_chance_node() {
        let root = LiarsDiceGame::root();
        assert_eq!(root.turn(), LiarsDiceTurn::Chance);

        let info = root.info_set();
        assert_eq!(info.my_die(), None);
        assert_eq!(info.choices().len(), 36);
        assert!(info.choices().iter().all(LiarsDiceEdge::is_roll));
    }

    #[test]
    fn legal_bids_increase() {
        let dealt = LiarsDiceGame::dealt(2, 5);
        let info = dealt.info_set();
        let opening = info.choices();
        assert_eq!(opening.len(), MAX_BIDS);
        assert!(opening.iter().all(LiarsDiceEdge::is_bid));
        assert!(!opening.contains(&LiarsDiceEdge::Challenge));

        let after_open = dealt.apply(LiarsDiceEdge::Bid {
            quantity: 1,
            face: 3,
        });
        let reply_choices = after_open.info_set().choices();

        assert_eq!(reply_choices.len(), 10);
        assert_eq!(reply_choices.last(), Some(&LiarsDiceEdge::Challenge));
        assert!(
            reply_choices[..reply_choices.len() - 1]
                .iter()
                .all(|edge| edge.bid_rank().expect("reply choices are bids") > 3)
        );
    }

    #[test]
    fn challenge_resolves_game() {
        let dealt = LiarsDiceGame::dealt(1, 2);
        let bid = dealt.apply(LiarsDiceEdge::Bid {
            quantity: 2,
            face: 1,
        });
        let resolved = bid.apply(LiarsDiceEdge::Challenge);

        assert_eq!(resolved.turn(), LiarsDiceTurn::Terminal);
        assert_eq!(resolved.winner(), Some(1));
    }

    #[test]
    fn payoff_is_zero_sum() {
        let dealt = LiarsDiceGame::dealt(4, 4);
        let bid = dealt.apply(LiarsDiceEdge::Bid {
            quantity: 2,
            face: 4,
        });
        let resolved = bid.apply(LiarsDiceEdge::Challenge);

        let total =
            resolved.payoff(LiarsDiceTurn::Player0) + resolved.payoff(LiarsDiceTurn::Player1);
        assert_eq!(total, 0.0);
        assert_eq!(resolved.payoff(LiarsDiceTurn::Player0), 1.0);
        assert_eq!(resolved.payoff(LiarsDiceTurn::Player1), -1.0);
    }

    #[test]
    fn all_trait_bounds_satisfied() {
        fn assert_edge<E: CfrEdge>() {}
        fn assert_turn<T: CfrTurn>() {}
        fn assert_game<G: CfrGame<E = LiarsDiceEdge, T = LiarsDiceTurn>>() {}
        fn assert_public<P: CfrPublic<E = LiarsDiceEdge, T = LiarsDiceTurn>>() {}
        fn assert_secret<S: CfrSecret>() {}
        fn assert_info<I: CfrInfo<E = LiarsDiceEdge, T = LiarsDiceTurn>>() {}

        assert_edge::<LiarsDiceEdge>();
        assert_turn::<LiarsDiceTurn>();
        assert_game::<LiarsDiceGame>();
        assert_public::<<LiarsDiceInfo as CfrInfo>::X>();
        assert_secret::<<LiarsDiceInfo as CfrInfo>::Y>();
        assert_info::<LiarsDiceInfo>();
    }
}
