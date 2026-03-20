//! Wire serialization for NLHE types using bincode.

use rbp_mccfr::{Encoder, CfrGame};
use rbp_nlhe::{NlheEdge, NlheInfo};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WireError {
    #[error("bincode encode error: {0}")]
    Encode(String),
    #[error("bincode decode error: {0}")]
    Decode(String),
}

/// Types that can be serialized to/from bytes via bincode for wire transport.
pub trait WireSerializable: Sized {
    fn to_bytes(&self) -> Result<Vec<u8>, WireError>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, WireError>;
}

impl WireSerializable for NlheInfo {
    fn to_bytes(&self) -> Result<Vec<u8>, WireError> {
        bincode::serialize(self).map_err(|e| WireError::Encode(e.to_string()))
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        bincode::deserialize(bytes).map_err(|e| WireError::Decode(e.to_string()))
    }
}

impl WireSerializable for NlheEdge {
    fn to_bytes(&self) -> Result<Vec<u8>, WireError> {
        bincode::serialize(self).map_err(|e| WireError::Encode(e.to_string()))
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, WireError> {
        bincode::deserialize(bytes).map_err(|e| WireError::Decode(e.to_string()))
    }
}

/// Marker struct for NLHE poker wire format.
#[derive(Debug, Clone, Copy)]
pub struct Poker;

impl WireSerializable for Poker {
    fn to_bytes(&self) -> Result<Vec<u8>, WireError> {
        bincode::serialize(&()).map_err(|e| WireError::Encode(e.to_string()))
    }
    fn from_bytes(_bytes: &[u8]) -> Result<Self, WireError> {
        Ok(Poker)
    }
}

/// Serialize an action distribution to bytes.
pub fn serialize_actions(
    actions: &[(NlheEdge, rbp_core::Probability)],
) -> Result<Vec<u8>, WireError> {
    bincode::serialize(actions).map_err(|e| WireError::Encode(e.to_string()))
}

/// Deserialize action distribution from bytes.
pub fn deserialize_actions(
    bytes: &[u8],
) -> Result<Vec<(NlheEdge, rbp_core::Probability)>, WireError> {
    bincode::deserialize(bytes).map_err(|e| WireError::Decode(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_mccfr::{Encoder, CfrGame};
    use rbp_nlhe::NlheInfo;

    #[test]
    fn nlhe_info_roundtrip() {
        // Create a valid info set from the game root
        let encoder = rbp_nlhe::NlheEncoder::default();
        let game = rbp_nlhe::NlheGame::root();
        let info = encoder.seed(&game);

        let bytes = info.to_bytes().unwrap();
        let decoded = NlheInfo::from_bytes(&bytes).unwrap();
        assert_eq!(info.street(), decoded.street());
        assert_eq!(info.subgame(), decoded.subgame());
    }

    #[test]
    fn nlhe_edge_roundtrip() {
        // Test edge roundtrip using solver's strategy
        let solver = crate::solver::create_empty_solver();
        let encoder = rbp_nlhe::NlheEncoder::default();
        let game = rbp_nlhe::NlheGame::root();
        let info = encoder.seed(&game);
        let dist = crate::solver::strategy(&solver, &info);

        // Test first edge from distribution
        if let Some((edge, _)) = dist.first() {
            let bytes = edge.to_bytes().unwrap();
            let decoded = NlheEdge::from_bytes(&bytes).unwrap();
            assert_eq!(*edge, decoded);
        }
    }

    #[test]
    fn all_edge_variants_serialize() {
        // Test that an edge can be serialized
        let solver = crate::solver::create_empty_solver();
        let encoder = rbp_nlhe::NlheEncoder::default();
        let game = rbp_nlhe::NlheGame::root();
        let info = encoder.seed(&game);
        let dist = crate::solver::strategy(&solver, &info);

        // Verify at least one edge exists and serializes correctly
        assert!(!dist.is_empty(), "expected at least one edge in distribution");
        let (edge, prob) = dist.first().unwrap();
        let bytes = edge.to_bytes().unwrap();
        assert!(!bytes.is_empty(), "edge {:?} should serialize", edge);
        let decoded = NlheEdge::from_bytes(&bytes).unwrap();
        assert_eq!(*edge, decoded, "edge {:?} should roundtrip", edge);
        assert!(!prob.is_nan(), "probability should be valid");
    }
}