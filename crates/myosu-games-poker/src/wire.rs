//! Wire serialization for NLHE types using bincode.
//!
//! Provides roundtrip serialization for `NlheInfo` and `NlheEdge` types.

use rbp_nlhe::{NlheInfo, NlheEdge};
use serde::{Deserialize, Serialize};

/// The poker game wire module.
pub struct Poker;

/// Wire-format strategy query/response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireStrategy {
    /// Serialized information set (NlheInfo bytes)
    pub info_bytes: Vec<u8>,
    /// Serialized action probabilities (NlheEdge + probability pairs)
    pub action_bytes: Vec<u8>,
}

/// Codec for `NlheInfo` types.
pub struct NlheInfoCodec;

impl NlheInfoCodec {
    /// Serialize `NlheInfo` to bytes.
    pub fn encode(_info: &NlheInfo) -> std::io::Result<Vec<u8>> {
        unimplemented!("Slice 3")
    }

    /// Deserialize bytes to `NlheInfo`.
    pub fn decode(_bytes: &[u8]) -> std::io::Result<NlheInfo> {
        unimplemented!("Slice 3")
    }
}

/// Codec for `NlheEdge` types.
pub struct NlheEdgeCodec;

impl NlheEdgeCodec {
    /// Serialize `NlheEdge` to bytes.
    pub fn encode(_edge: &NlheEdge) -> std::io::Result<Vec<u8>> {
        unimplemented!("Slice 3")
    }

    /// Deserialize bytes to `NlheEdge`.
    pub fn decode(_bytes: &[u8]) -> std::io::Result<NlheEdge> {
        unimplemented!("Slice 3")
    }
}

/// Marker trait for wire-serializable poker types.
pub trait WireSerializable: Sized {
    fn to_wire(&self) -> std::io::Result<Vec<u8>>;
    fn from_wire(bytes: &[u8]) -> std::io::Result<Self>;
}

impl WireSerializable for NlheInfo {
    fn to_wire(&self) -> std::io::Result<Vec<u8>> {
        NlheInfoCodec::encode(self)
    }

    fn from_wire(bytes: &[u8]) -> std::io::Result<Self> {
        NlheInfoCodec::decode(bytes)
    }
}

impl WireSerializable for NlheEdge {
    fn to_wire(&self) -> std::io::Result<Vec<u8>> {
        NlheEdgeCodec::encode(self)
    }

    fn from_wire(bytes: &[u8]) -> std::io::Result<Self> {
        NlheEdgeCodec::decode(bytes)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nlhe_info_roundtrip() {
        // Slice 3
    }

    #[test]
    fn nlhe_edge_roundtrip() {
        // Slice 3
    }

    #[test]
    fn all_edge_variants_serialize() {
        // Slice 3
    }
}
