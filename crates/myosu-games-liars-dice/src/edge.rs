use myosu_games::CfrEdge;
use rbp_transport::Support;

pub const NUM_PLAYERS: usize = 2;
pub const NUM_FACES: u8 = 6;
pub const MAX_QUANTITY: u8 = NUM_PLAYERS as u8;
pub const MAX_BIDS: usize = (NUM_FACES as usize) * (MAX_QUANTITY as usize);
pub(crate) const NO_DIE: u8 = 0;
pub(crate) const NO_BID: u8 = 0;

/// A chance outcome or player action in one-die-per-player Liar's Dice.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiarsDiceEdge {
    /// Root chance event: both hidden dice are dealt at once.
    Roll { player0: u8, player1: u8 },
    /// A public bid of `quantity` dice showing `face`.
    Bid { quantity: u8, face: u8 },
    /// Challenge the previous bid.
    Challenge,
}

impl LiarsDiceEdge {
    pub fn is_roll(&self) -> bool {
        matches!(self, Self::Roll { .. })
    }

    pub fn is_bid(&self) -> bool {
        matches!(self, Self::Bid { .. })
    }

    pub(crate) fn all_rolls() -> Vec<Self> {
        (1..=NUM_FACES)
            .flat_map(|player0| (1..=NUM_FACES).map(move |player1| Self::Roll { player0, player1 }))
            .collect()
    }

    pub(crate) fn all_bids() -> Vec<Self> {
        (1..=MAX_QUANTITY)
            .flat_map(|quantity| (1..=NUM_FACES).map(move |face| Self::Bid { quantity, face }))
            .collect()
    }

    pub(crate) fn is_valid_roll(&self) -> bool {
        match self {
            Self::Roll { player0, player1 } => is_valid_die(*player0) && is_valid_die(*player1),
            _ => false,
        }
    }

    pub(crate) fn is_valid_bid(&self) -> bool {
        match self {
            Self::Bid { quantity, face } => {
                (1..=MAX_QUANTITY).contains(quantity) && (1..=NUM_FACES).contains(face)
            }
            _ => false,
        }
    }

    pub(crate) fn from_bid_rank(rank: u8) -> Self {
        debug_assert!((1..=MAX_BIDS as u8).contains(&rank));
        let zero_based = rank - 1;
        let quantity = (zero_based / NUM_FACES) + 1;
        let face = (zero_based % NUM_FACES) + 1;
        Self::Bid { quantity, face }
    }

    pub(crate) fn bid_rank(&self) -> Option<u8> {
        match self {
            Self::Bid { quantity, face } if self.is_valid_bid() => {
                Some((quantity - 1) * NUM_FACES + face)
            }
            _ => None,
        }
    }
}

impl Support for LiarsDiceEdge {}
impl CfrEdge for LiarsDiceEdge {}

pub(crate) fn is_valid_die(die: u8) -> bool {
    (1..=NUM_FACES).contains(&die)
}
