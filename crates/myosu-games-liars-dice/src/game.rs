use myosu_games::{CfrEdge, CfrGame, CfrInfo, CfrTurn, Utility};
use rbp_mccfr::{CfrPublic, CfrSecret};
use rbp_transport::Support;
use serde::{Deserialize, Serialize};

pub(crate) const NO_CLAIM: u8 = u8::MAX;
const NO_DIE: u8 = 0;
const NO_WINNER: u8 = u8::MAX;

/// A bid in minimal two-player Liar's Dice.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LiarsDiceClaim {
    /// Number of dice claimed to show the face.
    pub count: u8,
    /// Face value being claimed.
    pub face: u8,
}

impl LiarsDiceClaim {
    /// Build a validated claim for the 2-player, 1-die-per-player proof game.
    pub fn new(count: u8, face: u8) -> Option<Self> {
        if !(1..=2).contains(&count) || !(1..=6).contains(&face) {
            return None;
        }
        Some(Self { count, face })
    }

    pub(crate) fn rank(self) -> u8 {
        (self.count - 1) * 6 + (self.face - 1)
    }

    fn from_rank(rank: u8) -> Option<Self> {
        if rank > 11 {
            return None;
        }
        let count = (rank / 6) + 1;
        let face = (rank % 6) + 1;
        Self::new(count, face)
    }
}

/// Turn classification for minimal Liar's Dice.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LiarsDiceTurn {
    /// Chance node dealing both dice.
    Chance,
    /// Player one to act.
    P1,
    /// Player two to act.
    P2,
    /// Terminal node after challenge resolution.
    Terminal,
}

impl From<usize> for LiarsDiceTurn {
    fn from(player: usize) -> Self {
        match player {
            0 => Self::P1,
            1 => Self::P2,
            _ => panic!("minimal liar's dice only has 2 players"),
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

/// A transition in the minimal Liar's Dice tree.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LiarsDiceEdge {
    /// Chance deals one hidden die to each player.
    Roll { p1: u8, p2: u8 },
    /// Active player raises the current claim.
    Bid(LiarsDiceClaim),
    /// Active player challenges the current claim.
    Challenge,
}

impl Support for LiarsDiceEdge {}
impl CfrEdge for LiarsDiceEdge {}

/// Public information needed for decision making.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LiarsDicePublic {
    actor: LiarsDiceTurn,
    last_claim_rank: u8,
}

impl LiarsDicePublic {
    pub(crate) fn new(actor: LiarsDiceTurn, last_claim_rank: u8) -> Self {
        Self {
            actor,
            last_claim_rank,
        }
    }

    /// Return the last claim if one exists.
    pub fn last_claim(self) -> Option<LiarsDiceClaim> {
        LiarsDiceClaim::from_rank(self.last_claim_rank)
    }
}

impl CfrPublic for LiarsDicePublic {
    type E = LiarsDiceEdge;
    type T = LiarsDiceTurn;

    fn choices(&self) -> Vec<Self::E> {
        match self.actor {
            LiarsDiceTurn::Chance => {
                let mut rolls = Vec::with_capacity(36);
                for p1 in 1..=6 {
                    for p2 in 1..=6 {
                        rolls.push(LiarsDiceEdge::Roll { p1, p2 });
                    }
                }
                rolls
            }
            LiarsDiceTurn::Terminal => Vec::new(),
            LiarsDiceTurn::P1 | LiarsDiceTurn::P2 => {
                let start = if self.last_claim_rank == NO_CLAIM {
                    0
                } else {
                    self.last_claim_rank + 1
                };
                let mut bids = Vec::with_capacity(13);
                for rank in start..=11 {
                    if let Some(claim) = LiarsDiceClaim::from_rank(rank) {
                        bids.push(LiarsDiceEdge::Bid(claim));
                    }
                }
                if self.last_claim_rank != NO_CLAIM {
                    bids.push(LiarsDiceEdge::Challenge);
                }
                bids
            }
        }
    }

    fn history(&self) -> Vec<Self::E> {
        match self.last_claim() {
            Some(claim) => vec![LiarsDiceEdge::Bid(claim)],
            None => Vec::new(),
        }
    }
}

/// Private information for the acting player: their die.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LiarsDiceSecret(pub u8);

impl Support for LiarsDiceSecret {}
impl CfrSecret for LiarsDiceSecret {}

/// Information set combining public state and the acting player's die.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LiarsDiceInfo {
    public: LiarsDicePublic,
    secret: LiarsDiceSecret,
}

impl LiarsDiceInfo {
    pub(crate) fn new(public: LiarsDicePublic, secret: LiarsDiceSecret) -> Self {
        Self { public, secret }
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

/// Minimal 2-player, 1-die-per-player Liar's Dice game state.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LiarsDiceGame {
    p1_die: u8,
    p2_die: u8,
    last_claim_rank: u8,
    actor: LiarsDiceTurn,
    winner: u8,
}

impl LiarsDiceGame {
    /// Build the information set for the acting player, if applicable.
    pub fn info(self) -> Option<LiarsDiceInfo> {
        let public = LiarsDicePublic::new(self.actor, self.last_claim_rank);
        let secret = match self.actor {
            LiarsDiceTurn::P1 => LiarsDiceSecret(self.p1_die),
            LiarsDiceTurn::P2 => LiarsDiceSecret(self.p2_die),
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => return None,
        };
        Some(LiarsDiceInfo::new(public, secret))
    }

    pub(crate) fn encoded_info(self) -> LiarsDiceInfo {
        let public = LiarsDicePublic::new(self.actor, self.last_claim_rank);
        let secret = match self.actor {
            LiarsDiceTurn::P1 => LiarsDiceSecret(self.p1_die),
            LiarsDiceTurn::P2 => LiarsDiceSecret(self.p2_die),
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => LiarsDiceSecret(NO_DIE),
        };
        LiarsDiceInfo::new(public, secret)
    }

    /// Return the last claim if one exists.
    pub fn last_claim(self) -> Option<LiarsDiceClaim> {
        LiarsDiceClaim::from_rank(self.last_claim_rank)
    }

    fn validate_roll(p1: u8, p2: u8) {
        assert!((1..=6).contains(&p1), "p1 roll must be 1..=6");
        assert!((1..=6).contains(&p2), "p2 roll must be 1..=6");
    }

    fn next_actor(self) -> LiarsDiceTurn {
        match self.actor {
            LiarsDiceTurn::P1 => LiarsDiceTurn::P2,
            LiarsDiceTurn::P2 => LiarsDiceTurn::P1,
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => self.actor,
        }
    }

    fn current_player(self) -> u8 {
        match self.actor {
            LiarsDiceTurn::P1 => 0,
            LiarsDiceTurn::P2 => 1,
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => NO_WINNER,
        }
    }

    fn actual_count(self, face: u8) -> u8 {
        let mut count = 0;
        if self.p1_die == face {
            count += 1;
        }
        if self.p2_die == face {
            count += 1;
        }
        count
    }

    fn winner_after_challenge(self) -> u8 {
        let claim = self
            .last_claim()
            .expect("challenge requires an existing claim");
        let challenger = self.current_player();
        let claimant = 1_u8.saturating_sub(challenger);
        if self.actual_count(claim.face) >= claim.count {
            claimant
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
            p1_die: NO_DIE,
            p2_die: NO_DIE,
            last_claim_rank: NO_CLAIM,
            actor: LiarsDiceTurn::Chance,
            winner: NO_WINNER,
        }
    }

    fn turn(&self) -> Self::T {
        self.actor
    }

    fn apply(&self, edge: Self::E) -> Self {
        match (self.actor, edge) {
            (LiarsDiceTurn::Chance, LiarsDiceEdge::Roll { p1, p2 }) => {
                Self::validate_roll(p1, p2);
                Self {
                    p1_die: p1,
                    p2_die: p2,
                    last_claim_rank: NO_CLAIM,
                    actor: LiarsDiceTurn::P1,
                    winner: NO_WINNER,
                }
            }
            (LiarsDiceTurn::P1 | LiarsDiceTurn::P2, LiarsDiceEdge::Bid(claim)) => {
                let claim_rank = claim.rank();
                assert!(
                    self.last_claim_rank == NO_CLAIM || claim_rank > self.last_claim_rank,
                    "new claim must strictly raise the prior claim"
                );
                Self {
                    p1_die: self.p1_die,
                    p2_die: self.p2_die,
                    last_claim_rank: claim_rank,
                    actor: self.next_actor(),
                    winner: NO_WINNER,
                }
            }
            (LiarsDiceTurn::P1 | LiarsDiceTurn::P2, LiarsDiceEdge::Challenge) => Self {
                p1_die: self.p1_die,
                p2_die: self.p2_die,
                last_claim_rank: self.last_claim_rank,
                actor: LiarsDiceTurn::Terminal,
                winner: self.winner_after_challenge(),
            },
            (LiarsDiceTurn::P1 | LiarsDiceTurn::P2, LiarsDiceEdge::Roll { .. }) => {
                panic!("player turns do not accept roll edges")
            }
            (LiarsDiceTurn::Terminal, _) => panic!("terminal states do not accept actions"),
            (LiarsDiceTurn::Chance, _) => panic!("chance node only accepts roll edges"),
        }
    }

    fn payoff(&self, turn: Self::T) -> Utility {
        assert_eq!(
            self.actor,
            LiarsDiceTurn::Terminal,
            "payoff requires terminal state"
        );
        let player = match turn {
            LiarsDiceTurn::P1 => 0,
            LiarsDiceTurn::P2 => 1,
            LiarsDiceTurn::Chance | LiarsDiceTurn::Terminal => {
                panic!("payoff only applies to player turns")
            }
        };
        if self.winner == player { 1.0 } else { -1.0 }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::{
        LiarsDiceClaim, LiarsDiceEdge, LiarsDiceGame, LiarsDicePublic, LiarsDiceTurn,
    };
    use myosu_games::CfrGame;
    use rbp_mccfr::{CfrInfo, CfrPublic};

    #[test]
    fn root_is_chance_with_all_rolls() {
        let root = LiarsDiceGame::root();
        let public = LiarsDicePublic::new(LiarsDiceTurn::Chance, u8::MAX);

        assert_eq!(root.turn(), LiarsDiceTurn::Chance);
        assert_eq!(public.choices().len(), 36);
    }

    #[test]
    fn first_bid_hands_turn_to_opponent() {
        let rolled = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 });
        let raised = rolled.apply(LiarsDiceEdge::Bid(
            LiarsDiceClaim::new(1, 3).expect("claim should be valid"),
        ));

        assert_eq!(rolled.turn(), LiarsDiceTurn::P1);
        assert_eq!(raised.turn(), LiarsDiceTurn::P2);
        assert_eq!(
            raised.last_claim().expect("claim should exist"),
            LiarsDiceClaim::new(1, 3).expect("claim should be valid")
        );
    }

    #[test]
    fn challenge_resolves_true_claim_for_claimant() {
        let game = LiarsDiceGame::root()
            .apply(LiarsDiceEdge::Roll { p1: 4, p2: 4 })
            .apply(LiarsDiceEdge::Bid(
                LiarsDiceClaim::new(2, 4).expect("claim should be valid"),
            ))
            .apply(LiarsDiceEdge::Challenge);

        assert_eq!(game.turn(), LiarsDiceTurn::Terminal);
        assert_eq!(game.payoff(LiarsDiceTurn::P1), 1.0);
        assert_eq!(game.payoff(LiarsDiceTurn::P2), -1.0);
    }

    #[test]
    fn challenge_resolves_false_claim_for_challenger() {
        let game = LiarsDiceGame::root()
            .apply(LiarsDiceEdge::Roll { p1: 1, p2: 2 })
            .apply(LiarsDiceEdge::Bid(
                LiarsDiceClaim::new(2, 6).expect("claim should be valid"),
            ))
            .apply(LiarsDiceEdge::Challenge);

        assert_eq!(game.turn(), LiarsDiceTurn::Terminal);
        assert_eq!(game.payoff(LiarsDiceTurn::P1), -1.0);
        assert_eq!(game.payoff(LiarsDiceTurn::P2), 1.0);
    }

    #[test]
    fn info_exposes_private_die_and_legal_choices() {
        let game = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 6, p2: 1 });
        let info = game.info().expect("player turn should expose info");

        assert_eq!(info.secret().0, 6);
        assert_eq!(info.public().choices().len(), 12);
    }
}
