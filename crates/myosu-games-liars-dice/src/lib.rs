//! Liar's Dice proof-of-architecture crate for the multi-game lane.
//!
//! Slice 2 adds the fixed-size CFR game engine surface:
//! - [`LiarsDiceGame`]
//! - [`LiarsDiceEdge`]
//! - [`LiarsDiceTurn`]
//! - [`LiarsDiceInfo`]
//!
//! Encoder and profile behavior remain intentionally stubbed until slice 3.

mod edge;
mod game;
mod info;
mod turn;

pub use edge::{LiarsDiceEdge, MAX_BIDS, NUM_FACES, NUM_PLAYERS};
pub use game::LiarsDiceGame;
pub use info::LiarsDiceInfo;
pub use turn::LiarsDiceTurn;

/// Placeholder for the future Liar's Dice encoder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceEncoder;

/// Placeholder for the future Liar's Dice solver profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceProfile;

#[cfg(test)]
mod tests {
    use super::*;
    use myosu_games::CfrGame;

    #[test]
    fn public_api_stubs_exist() {
        let _ = LiarsDiceGame::root();
        let _ = LiarsDiceEdge::Challenge;
        let _ = LiarsDiceTurn::Terminal;
        let _ = LiarsDiceInfo::from_game(&LiarsDiceGame::root());
        let _ = LiarsDiceEncoder;
        let _ = LiarsDiceProfile;
    }
}
