//! Wire serialization for NLHE types.
//!
//! Provides bincode roundtrip serialization for `NlheInfo` and `NlheEdge`,
//! which are used in `StrategyQuery` and `StrategyResponse` for
//! miner-validator communication.
//!
//! # Requirements
//!
//! The `serde` feature must be enabled on `rbp-nlhe` (and its transitive
//! dependencies) for this module to compile. This is handled automatically
//! by the crate's `features = ["serde"]` on `rbp-nlhe`.

use rbp_nlhe::NlheEdge;
use serde::{Deserialize, Serialize};

/// Marker type indicating serialization is for the Poker engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Poker;

/// Trait for types that can be serialized to/from bytes for wire transport.
///
/// This is implemented for `NlheInfo` and `NlheEdge` to enable
/// type-safe serialization in the query handler.
pub trait WireSerializable: Sized + Serialize + for<'de> Deserialize<'de> {
    /// Serialize to a byte vector using bincode.
    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("bincode serialization should not fail for valid types")
    }

    /// Deserialize from a byte slice using bincode.
    fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
}

impl WireSerializable for rbp_nlhe::NlheInfo {}
impl WireSerializable for NlheEdge {}

// All edge variants that can appear in NLHE strategy responses
#[cfg(test)]
mod tests {
    use super::*;
    use rbp_gameplay::Edge;
    use rbp_mccfr::{CfrGame, Encoder};
    use rbp_nlhe::NlheGame;

    #[test]
    fn nlhe_info_roundtrip() {
        let encoder = rbp_nlhe::NlheEncoder::default();
        let root = NlheGame::root();
        let info = encoder.seed(&root);

        let bytes = info.to_bytes();
        let recovered = <rbp_nlhe::NlheInfo as WireSerializable>::from_bytes(&bytes).unwrap();

        assert_eq!(info, recovered);
    }

    #[test]
    fn nlhe_edge_roundtrip() {
        // Test a few common edge variants
        let edges = vec![
            NlheEdge::from(Edge::Check),
            NlheEdge::from(Edge::Call),
            NlheEdge::from(Edge::Fold),
            NlheEdge::from(Edge::Draw),
        ];

        for edge in edges {
            let bytes = edge.to_bytes();
            let recovered = NlheEdge::from_bytes(&bytes).unwrap();
            assert_eq!(edge, recovered);
        }
    }

    #[test]
    fn all_edge_variants_serialize() {
        // Test that all edge variants in the game can roundtrip
        let game = NlheGame::root();
        let choices = game.as_ref().choices(0);

        for edge in choices {
            let nlhe_edge = NlheEdge::from(edge);
            let bytes = nlhe_edge.to_bytes();
            let recovered = NlheEdge::from_bytes(&bytes).unwrap();
            assert_eq!(nlhe_edge, recovered);
        }
    }
}
