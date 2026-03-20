//! Encoder for Liar's Dice.
//!
//! The encoder maps game states to information set identifiers.
//! For Liar's Dice, this is straightforward:
//! - `seed()` — creates the root info set (both dice = 1)
//! - `info()` — creates info set for a child state
//! - `resume()` — creates info set from an edge history path
//!
//! Since each player knows only their own die, the info set is:
//! `(my_die, bid_history_tuple)`

use crate::edge::LiarsDiceEdge;
use crate::game::LiarsDiceGame;
use crate::info::LiarsDiceInfo;
use crate::turn::LiarsDiceTurn;
use rbp_mccfr::{CfrGame, Encoder};

// Encoder is trivial for 1-die Liar's Dice — the info set is just (die, bid_history).
// The die is directly observable by the owning player, and the bid history is public.
#[derive(Clone, Copy, Debug, Default)]
pub struct LiarsDiceEncoder;

impl Encoder for LiarsDiceEncoder {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;

    fn seed(&self, game: &Self::G) -> Self::I {
        // At root, the acting player (P0) sees their own die = 1 (fixed roll)
        game.info(0)
    }

    fn info(
        &self,
        _tree: &rbp_mccfr::Tree<Self::T, Self::E, Self::G, Self::I>,
        leaf: rbp_mccfr::Branch<Self::E, Self::G>,
    ) -> Self::I {
        let (_edge, game, _parent) = leaf;
        // The acting player at this state sees their own die
        let acting = game.acting_player() as usize;
        game.info(acting)
    }

    fn resume(&self, past: &[Self::E], _game: &Self::G) -> Self::I {
        // Apply the edge history to get the info set
        let mut state = LiarsDiceGame::root();
        for edge in past {
            state = state.apply(*edge);
        }
        let acting = state.acting_player() as usize;
        state.info(acting)
    }
}
