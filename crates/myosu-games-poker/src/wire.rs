//! Wire serialization for NLHE types using bincode.
//!
//! Provides bincode roundtrip serialization for `NlheInfo` and `NlheEdge` types.
//! Requires the `serde` feature on robopoker crates (enabled by default in this crate).

use anyhow::{Context, Result};
use rbp_nlhe::{NlheEdge, NlheInfo};
use serde::{Deserialize, Serialize};

/// Types that can be serialized to/from bytes via bincode for wire transmission.
pub trait WireSerializable: Sized + Serialize + for<'de> Deserialize<'de> {
    /// Serialize to a byte vector.
    fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .context("failed to serialize type to bincode")
    }

    /// Deserialize from a byte slice.
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .context("failed to deserialize type from bincode")
    }
}

impl WireSerializable for NlheInfo {}
impl WireSerializable for NlheEdge {}

/// Marker type for poker-related wire serialization.
#[derive(Clone, Debug, Default)]
pub struct Poker;

/// Extension trait for encoding/decoding strategy info and edge types.
pub trait WireEncode {
    /// Encode info set to wire format.
    fn encode_info(info: &NlheInfo) -> Result<Vec<u8>> {
        info.to_bytes()
    }

    /// Decode info set from wire format.
    fn decode_info(bytes: &[u8]) -> Result<NlheInfo> {
        NlheInfo::from_bytes(bytes)
    }

    /// Encode edge to wire format.
    fn encode_edge(edge: &NlheEdge) -> Result<Vec<u8>> {
        edge.to_bytes()
    }

    /// Decode edge from wire format.
    fn decode_edge(bytes: &[u8]) -> Result<NlheEdge> {
        NlheEdge::from_bytes(bytes)
    }
}

impl WireEncode for Poker {}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_mccfr::{CfrGame, Encoder};
    use rbp_nlhe::{NlheEncoder, NlheGame};

    #[test]
    fn nlhe_info_roundtrip() {
        let encoder = NlheEncoder::default();
        let root_info = encoder.seed(&NlheGame::root());

        let encoded = root_info.to_bytes().unwrap();
        let decoded = NlheInfo::from_bytes(&encoded).unwrap();

        assert_eq!(decoded.street(), root_info.street());
        assert_eq!(decoded.subgame(), root_info.subgame());
        assert_eq!(decoded.bucket(), root_info.bucket());
    }

    #[test]
    fn nlhe_edge_roundtrip() {
        // Get edges from the game directly
        let game = NlheGame::root();
        let choices = game.as_ref().choices(0);

        // Test each available edge
        for edge in choices {
            let nlhe_edge = NlheEdge::from(edge);
            let encoded = nlhe_edge.to_bytes().unwrap();
            let decoded = NlheEdge::from_bytes(&encoded).unwrap();
            assert_eq!(decoded, nlhe_edge);
        }
    }

    #[test]
    fn all_edge_variants_serialize() {
        let game = NlheGame::root();
        let choices = game.as_ref().choices(0);
        for edge in choices {
            let nlhe_edge = NlheEdge::from(edge);
            let encoded = nlhe_edge.to_bytes().unwrap();
            let decoded = NlheEdge::from_bytes(&encoded).unwrap();
            assert_eq!(decoded, nlhe_edge);
        }
    }
}
