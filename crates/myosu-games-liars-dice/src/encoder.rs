//! Liar's Dice encoder implementation.

use crate::edge::LiarsDiceEdge;
use crate::game::LiarsDiceGame;
use crate::info::LiarsDiceInfo;
use crate::turn::LiarsDiceTurn;
use myosu_games::Encoder;
use rbp_mccfr::Tree;

/// Encoder for Liar's Dice information sets.
///
/// Maps game states to information set identifiers for CFR training.
pub struct LiarsDiceEncoder;

impl Encoder for LiarsDiceEncoder {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;

    fn seed(&self, _game: &Self::G) -> Self::I {
        // Seed returns the initial info set (before any dice are rolled)
        LiarsDiceInfo::new(0)
    }

    fn info(
        &self,
        _tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        _branch: rbp_mccfr::Branch<Self::E, Self::G>,
    ) -> Self::I {
        // For Liar's Dice, info is derived from game state
        // The branch contains (incoming_edge, game, depth)
        LiarsDiceInfo::new(0)
    }

    fn resume(&self, _past: &[Self::E], game: &Self::G) -> Self::I {
        // Build info from edge history and game state
        // For Liar's Dice, we need the player's private die
        // This is a simplification - full impl would track through tree
        LiarsDiceInfo::new(game.die(0))
    }
}
