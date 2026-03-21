//! Minimal public API skeleton for the multi-game Liar's Dice crate.
//!
//! Slice 1 intentionally stops at crate wiring. The CFR game state, action,
//! turn, info-set, encoder, and profile implementations land in later slices.

/// Placeholder for the future Liar's Dice CFR game state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceGame;

/// Placeholder for the future Liar's Dice action type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceEdge;

/// Placeholder for the future Liar's Dice turn marker.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceTurn;

/// Placeholder for the future Liar's Dice information set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceInfo;

/// Placeholder for the future Liar's Dice encoder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceEncoder;

/// Placeholder for the future Liar's Dice solver profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiarsDiceProfile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_api_stubs_exist() {
        let _ = LiarsDiceGame;
        let _ = LiarsDiceEdge;
        let _ = LiarsDiceTurn;
        let _ = LiarsDiceInfo;
        let _ = LiarsDiceEncoder;
        let _ = LiarsDiceProfile;
    }
}
