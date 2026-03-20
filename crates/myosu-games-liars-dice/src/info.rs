//! Liar's Dice information sets — `CfrInfo` implementation.
//!
//! Information sets encode what a player knows at a given decision point.
//! In Liar's Dice, a player sees only their own die and the bid history.

/// LiarsDiceInfo is the information set for a player at a decision point.
///
/// Contains the player's own die (1-6) and the bid history.
/// The opponent's die is NOT visible — this is the imperfect information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LiarsDiceInfo {
    /// This player's die face (1-6)
    pub my_die: u8,
    /// Number of valid bids in the history
    pub num_bids: u8,
    /// Bid history — same fixed-size array as game state
    pub bids: [Option<super::game::Bid>; 12],
}

impl LiarsDiceInfo {
    /// Create info from game state for a specific player.
    pub fn from_game(_game: &super::game::LiarsDiceGame, _player: u8) -> Self {
        todo!("Slice 2: implement LiarsDiceInfo::from_game()")
    }
}
