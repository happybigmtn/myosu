// wire.rs: bincode roundtrip for NlheInfo and NlheEdge
//
// Provides WireSerializable trait and Poker marker type for wire serialization

use rbp_nlhe::{NlheEdge, NlheInfo};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WireError {
    #[error("serialization failed: {0}")]
    SerializationError(String),
    #[error("deserialization failed: {0}")]
    DeserializationError(String),
    #[error("bincode error: {0}")]
    BincodeError(#[from] bincode::Error),
}

/// Marker type for poker-specific wire serialization
pub struct Poker;

/// Trait for types that can be serialized/deserialized over the wire
pub trait WireSerializable: Sized {
    /// Serialize to bytes for network transmission
    fn serialize(&self) -> Result<Vec<u8>, WireError>;

    /// Deserialize from bytes
    fn deserialize(bytes: &[u8]) -> Result<Self, WireError>;
}

impl WireSerializable for NlheInfo {
    fn serialize(&self) -> Result<Vec<u8>, WireError> {
        bincode::serialize(self).map_err(|e| WireError::SerializationError(e.to_string()))
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, WireError> {
        bincode::deserialize(bytes).map_err(|e| WireError::DeserializationError(e.to_string()))
    }
}

impl WireSerializable for NlheEdge {
    fn serialize(&self) -> Result<Vec<u8>, WireError> {
        bincode::serialize(self).map_err(|e| WireError::SerializationError(e.to_string()))
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, WireError> {
        bincode::deserialize(bytes).map_err(|e| WireError::DeserializationError(e.to_string()))
    }
}

/// Serialize a vector of (Edge, Probability) pairs for wire transmission
pub fn serialize_action_distribution(
    actions: &[(NlheEdge, f64)],
) -> Result<Vec<u8>, WireError> {
    bincode::serialize(actions).map_err(|e| WireError::SerializationError(e.to_string()))
}

/// Deserialize a vector of (Edge, Probability) pairs from wire bytes
pub fn deserialize_action_distribution(bytes: &[u8]) -> Result<Vec<(NlheEdge, f64)>, WireError> {
    bincode::deserialize(bytes).map_err(|e| WireError::DeserializationError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_core::Arbitrary;
    use rbp_gameplay::Edge;

    #[test]
    fn nlhe_info_roundtrip() {
        let info = NlheInfo::random();
        let bytes = info.serialize().unwrap();
        let recovered = NlheInfo::deserialize(&bytes).unwrap();
        assert_eq!(format!("{:?}", info), format!("{:?}", recovered));
    }

    #[test]
    fn nlhe_edge_roundtrip() {
        // Test with known edge variants constructed from Edge
        let edges = vec![
            NlheEdge::from(Edge::Fold),
            NlheEdge::from(Edge::Check),
            NlheEdge::from(Edge::Call),
            NlheEdge::from(Edge::Draw),
            NlheEdge::from(Edge::Shove),
        ];

        for edge in edges {
            let bytes = edge.serialize().unwrap();
            let recovered = NlheEdge::deserialize(&bytes).unwrap();
            assert_eq!(format!("{:?}", edge), format!("{:?}", recovered));
        }
    }

    #[test]
    fn all_edge_variants_serialize() {
        // Test all major edge variants
        let variants = vec![
            NlheEdge::from(Edge::Fold),
            NlheEdge::from(Edge::Check),
            NlheEdge::from(Edge::Call),
            NlheEdge::from(Edge::Draw),
            NlheEdge::from(Edge::Shove),
        ];

        for edge in variants {
            let bytes = edge.serialize().unwrap();
            let recovered = NlheEdge::deserialize(&bytes).unwrap();
            assert_eq!(format!("{:?}", edge), format!("{:?}", recovered));
        }
    }
}
