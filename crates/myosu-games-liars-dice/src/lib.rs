//! Liar's Dice proof-of-architecture crate.
//!
//! Slice 1 establishes the workspace package and the stable public type names.
//! Game logic, solver wiring, and trait implementations land in later slices.

use myosu_games::GameType;

mod stub {
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceGame;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceEdge;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceTurn;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceInfo;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceEncoder;

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LiarsDiceProfile;
}

pub use stub::{
    LiarsDiceEdge, LiarsDiceEncoder, LiarsDiceGame, LiarsDiceInfo, LiarsDiceProfile,
    LiarsDiceTurn,
};

/// Canonical registry hook for this crate's game type.
pub const GAME_TYPE: GameType = GameType::LiarsDice;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_liars_dice_game_type() {
        assert_eq!(GAME_TYPE.to_bytes(), b"liars_dice".to_vec());
    }

    #[test]
    fn stub_public_api_is_constructible() {
        let _ = LiarsDiceGame;
        let _ = LiarsDiceEdge;
        let _ = LiarsDiceTurn;
        let _ = LiarsDiceInfo;
        let _ = LiarsDiceEncoder;
        let _ = LiarsDiceProfile;
    }
}
