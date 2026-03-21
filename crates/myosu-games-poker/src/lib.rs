//! NLHE poker engine integration surface for myosu.
//!
//! Slice 2 adds the solver wrapper surface and checkpoint format while keeping
//! the remaining query, wire, exploit, and training-session work for later
//! slices.

use std::marker::PhantomData;

pub use myosu_games::{GameConfig, GameType, StrategyQuery, StrategyResponse};
pub use rbp_nlhe::{NlheEdge, NlheEncoder, NlheInfo, NlheProfile};
pub use solver::{CHECKPOINT_MAGIC, CHECKPOINT_VERSION, Flagship, PokerSolver, PokerSolverError};

pub mod solver;

/// Marker type for the NLHE heads-up engine family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Poker;

/// Public slot for the training coordinator that lands in Slice 6.
#[derive(Debug)]
pub struct TrainingSession;

// Compile-time guard for Slice 1: if serde is not enabled on `rbp-nlhe`, this
// crate should fail to build before the later wire/query slices begin.
struct SerdeReady<T>(PhantomData<T>)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>;

const _: SerdeReady<NlheInfo> = SerdeReady(PhantomData);
const _: SerdeReady<NlheEdge> = SerdeReady(PhantomData);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robopoker_nlhe_serde_surface_is_enabled() {
        fn assert_serde<T>()
        where
            T: serde::Serialize + for<'de> serde::Deserialize<'de>,
        {
        }

        assert_serde::<NlheInfo>();
        assert_serde::<NlheEdge>();
        assert!(std::mem::size_of::<Flagship>() > 0);
    }

    #[test]
    fn game_type_reexport_includes_nlhe_heads_up() {
        assert_eq!(GameType::NlheHeadsUp.to_bytes(), b"nlhe_hu".to_vec());
    }
}
