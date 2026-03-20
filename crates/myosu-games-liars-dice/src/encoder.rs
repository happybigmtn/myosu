//! Encoder for Liar's Dice.
//!
//! Maps game states to information set identifiers.

use crate::edge::LiarsDiceEdge;
use crate::game::LiarsDiceGame;
use crate::info::LiarsDiceInfo;
use crate::turn::LiarsDiceTurn;
use rbp_mccfr::Encoder;
use rbp_mccfr::Tree;

/// Encoder for Liar's Dice.
///
/// The info set is (my_die, turn, bid_history). Since Liar's Dice is
/// a small game with only 36 die outcomes, we use direct enumeration
/// rather than abstraction.
#[derive(Debug, Clone, Default)]
pub struct LiarsDiceEncoder;

impl Encoder for LiarsDiceEncoder {
    type T = LiarsDiceTurn;
    type E = LiarsDiceEdge;
    type G = LiarsDiceGame;
    type I = LiarsDiceInfo;

    fn seed(&self, game: &Self::G) -> Self::I {
        // At seed, it's always P0's turn and P0 knows their own die
        LiarsDiceInfo::from_game(game, game.dice[0])
    }

    fn info(
        &self,
        _tree: &Tree<Self::T, Self::E, Self::G, Self::I>,
        leaf: rbp_mccfr::Branch<Self::E, Self::G>,
    ) -> Self::I {
        // leaf is (incoming_edge, game_state, depth)
        // We need to compute the info set from the game state
        let game = &leaf.1;
        // The acting player's die is game.dice[0] for P0, game.dice[1] for P1
        let my_die = match game.turn {
            LiarsDiceTurn::Player(0) => game.dice[0],
            LiarsDiceTurn::Player(1) => game.dice[1],
            LiarsDiceTurn::Player(n) => game.dice[n % 2],
            LiarsDiceTurn::Terminal => game.dice[0], // Shouldn't happen
        };
        LiarsDiceInfo::from_game(game, my_die)
    }

    fn resume(&self, _past: &[Self::E], game: &Self::G) -> Self::I {
        // Similar to seed - compute info from game state
        let my_die = match game.turn {
            LiarsDiceTurn::Player(0) => game.dice[0],
            LiarsDiceTurn::Player(1) => game.dice[1],
            LiarsDiceTurn::Player(n) => game.dice[n % 2],
            LiarsDiceTurn::Terminal => game.dice[0],
        };
        LiarsDiceInfo::from_game(game, my_die)
    }
}
