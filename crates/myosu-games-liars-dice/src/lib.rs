//! Liar's Dice proof-of-architecture crate.
//!
//! Slice 2 implements the core game-engine surface:
//! - `LiarsDiceGame`: compact `CfrGame` state with chance roll, bids, and challenge
//! - `LiarsDiceEdge`: chance outcomes plus bidding/challenge edges
//! - `LiarsDiceTurn`: chance / player / terminal turn markers
//! - `LiarsDiceInfo`: acting-player information set (my die + public bid history)
//!
//! `LiarsDiceEncoder` and `LiarsDiceProfile` remain placeholders for Slice 3.

use myosu_games::GameType;

mod edge;
mod game;
mod info;
mod turn;

mod stub {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceEncoder;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceProfile;
}

pub use edge::LiarsDiceEdge;
pub use game::LiarsDiceGame;
pub use info::LiarsDiceInfo;
pub use stub::{LiarsDiceEncoder, LiarsDiceProfile};
pub use turn::LiarsDiceTurn;

/// Canonical registry hook for this crate's game type.
pub const GAME_TYPE: GameType = GameType::LiarsDice;

#[cfg(test)]
mod tests {
    use super::*;
    use myosu_games::CfrGame;

    #[test]
    fn exposes_liars_dice_game_type() {
        assert_eq!(GAME_TYPE.to_bytes(), b"liars_dice".to_vec());
    }

    #[test]
    fn public_api_is_constructible() {
        let _ = LiarsDiceGame::root();
        let _ = LiarsDiceEdge::Challenge;
        let _ = LiarsDiceTurn::Terminal;
        let _ = LiarsDiceInfo::from_game(&LiarsDiceGame::root());
        let _ = LiarsDiceEncoder;
        let _ = LiarsDiceProfile;
    }
}
