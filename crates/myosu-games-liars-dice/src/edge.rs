use myosu_games::CfrEdge;
use rbp_transport::Support;

pub(crate) const NUM_FACES: u8 = 6;
pub(crate) const MAX_QUANTITY: u8 = 2;
pub(crate) const BID_COUNT: usize = (NUM_FACES as usize) * (MAX_QUANTITY as usize);
const EMPTY_BID_SLOT: u8 = u8::MAX;

/// A transition in the Liar's Dice game tree.
///
/// `Roll` is the root chance outcome. `Bid` and `Challenge` are the player
/// decisions described by AC-MG-01.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiarsDiceEdge {
    Roll { p0: u8, p1: u8 },
    Bid { quantity: u8, face: u8 },
    Challenge,
}

impl LiarsDiceEdge {
    pub fn roll(p0: u8, p1: u8) -> Self {
        assert!(Self::is_valid_face(p0), "player 0 die must be in 1..=6");
        assert!(Self::is_valid_face(p1), "player 1 die must be in 1..=6");
        Self::Roll { p0, p1 }
    }

    pub fn bid(quantity: u8, face: u8) -> Self {
        assert!(
            Self::is_valid_quantity(quantity),
            "bid quantity must be in 1..=2"
        );
        assert!(Self::is_valid_face(face), "bid face must be in 1..=6");
        Self::Bid { quantity, face }
    }

    pub fn is_roll(&self) -> bool {
        matches!(self, Self::Roll { .. })
    }

    pub fn is_bid(&self) -> bool {
        matches!(self, Self::Bid { .. })
    }

    pub fn is_challenge(&self) -> bool {
        matches!(self, Self::Challenge)
    }

    pub fn quantity(&self) -> Option<u8> {
        match self {
            Self::Bid { quantity, .. } => Some(*quantity),
            _ => None,
        }
    }

    pub fn face(&self) -> Option<u8> {
        match self {
            Self::Bid { face, .. } => Some(*face),
            _ => None,
        }
    }

    pub(crate) fn all_rolls() -> Vec<Self> {
        (1..=NUM_FACES)
            .flat_map(|p0| (1..=NUM_FACES).map(move |p1| Self::roll(p0, p1)))
            .collect()
    }

    pub(crate) fn bids_after(last_bid: Option<Self>) -> Vec<Self> {
        let start = last_bid
            .map(|edge| edge.bid_index().expect("history stores bids"))
            .map_or(0usize, |rank| rank as usize + 1);
        (start..BID_COUNT)
            .map(|rank| Self::from_bid_index(rank as u8))
            .collect()
    }

    pub(crate) fn bid_index(&self) -> Option<u8> {
        match *self {
            Self::Bid { quantity, face } => Some((quantity - 1) * NUM_FACES + (face - 1)),
            _ => None,
        }
    }

    pub(crate) fn from_bid_index(rank: u8) -> Self {
        assert!((rank as usize) < BID_COUNT, "bid rank out of bounds");
        let quantity = (rank / NUM_FACES) + 1;
        let face = (rank % NUM_FACES) + 1;
        Self::bid(quantity, face)
    }

    fn is_valid_face(face: u8) -> bool {
        (1..=NUM_FACES).contains(&face)
    }

    fn is_valid_quantity(quantity: u8) -> bool {
        (1..=MAX_QUANTITY).contains(&quantity)
    }
}

impl Support for LiarsDiceEdge {}
impl CfrEdge for LiarsDiceEdge {}

/// Compact bid history for `Copy` game and info-set state.
///
/// Unused slots are kept at `EMPTY_BID_SLOT` so the representation follows the
/// reviewed "fixed-size array with sentinel" constraint.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BidHistory {
    bids: [u8; BID_COUNT],
    len: u8,
}

impl BidHistory {
    pub(crate) fn push(&mut self, edge: LiarsDiceEdge) {
        let rank = edge
            .bid_index()
            .expect("only bid edges may enter bid history");
        assert!((self.len as usize) < BID_COUNT, "bid history overflow");
        self.bids[self.len as usize] = rank;
        self.len += 1;
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn last(&self) -> Option<LiarsDiceEdge> {
        if self.is_empty() {
            None
        } else {
            Some(LiarsDiceEdge::from_bid_index(
                self.bids[self.len as usize - 1],
            ))
        }
    }

    pub(crate) fn edges(&self) -> Vec<LiarsDiceEdge> {
        self.bids[..self.len as usize]
            .iter()
            .copied()
            .map(LiarsDiceEdge::from_bid_index)
            .collect()
    }
}

impl Default for BidHistory {
    fn default() -> Self {
        Self {
            bids: [EMPTY_BID_SLOT; BID_COUNT],
            len: 0,
        }
    }
}
