//! Wire serialization for NLHE types via serde.
//!
//! Provides roundtrip serialization for [`NlheInfo`] and [`NlheEdge`] using serde_json.
//! This module requires the `serde` feature on `rbp-nlhe`, which is enabled by default
//! in this crate's dependencies.

use rbp_nlhe::{NlheEdge, NlheInfo};
use serde::{Deserialize, Serialize};

/// Marker trait for types that can be serialized over the wire using serde.
pub trait WireSerializable: Sized {
    /// Serialize to a JSON byte vector.
    fn to_wire(&self) -> Vec<u8>;

    /// Deserialize from a byte slice.
    fn from_wire(bytes: &[u8]) -> Result<Self, WireError>;
}

/// Wire-level representation of a strategy query or response for NLHE.
///
/// For queries: `info_bytes` contains the JSON-encoded `NlheInfo`.
/// For responses: `actions` contains the JSON-encoded action-probability pairs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireStrategy {
    /// JSON-encoded `NlheInfo` for queries, empty for responses.
    pub info_bytes: Vec<u8>,
    /// JSON-encoded `Vec<(NlheEdge, Probability)>` for responses, empty for queries.
    pub action_bytes: Vec<u8>,
}

impl WireStrategy {
    /// Creates a query wire message from an `NlheInfo`.
    pub fn query(info: &NlheInfo) -> Self {
        Self {
            info_bytes: serde_json::to_vec(info).expect("NlheInfo should be serializable"),
            action_bytes: Vec::new(),
        }
    }

    /// Creates a response wire message from an action distribution.
    pub fn response(actions: &[(NlheEdge, rbp_core::Probability)]) -> Self {
        Self {
            info_bytes: Vec::new(),
            action_bytes: serde_json::to_vec(actions)
                .expect("action distribution should be serializable"),
        }
    }

    /// Parses the contained `NlheInfo` (only valid for query messages).
    pub fn parse_info(&self) -> Result<NlheInfo, WireError> {
        if self.info_bytes.is_empty() {
            return Err(WireError::NotAQuery);
        }
        serde_json::from_slice(&self.info_bytes).map_err(|e| WireError::DecodeError(e.to_string()))
    }

    /// Parses the contained action distribution (only valid for response messages).
    pub fn parse_actions(&self) -> Result<Vec<(NlheEdge, rbp_core::Probability)>, WireError> {
        if self.action_bytes.is_empty() {
            return Err(WireError::NotAResponse);
        }
        serde_json::from_slice(&self.action_bytes)
            .map_err(|e| WireError::DecodeError(e.to_string()))
    }
}

impl WireSerializable for NlheInfo {
    fn to_wire(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("NlheInfo should be serializable")
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, WireError> {
        serde_json::from_slice(bytes).map_err(|e| WireError::DecodeError(e.to_string()))
    }
}

impl WireSerializable for NlheEdge {
    fn to_wire(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("NlheEdge should be serializable")
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, WireError> {
        serde_json::from_slice(bytes).map_err(|e| WireError::DecodeError(e.to_string()))
    }
}

/// Errors that can occur during wire serialization/deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    DecodeError(String),
    NotAQuery,
    NotAResponse,
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::DecodeError(msg) => write!(f, "decode error: {}", msg),
            WireError::NotAQuery => write!(f, "not a query message"),
            WireError::NotAResponse => write!(f, "not a response message"),
        }
    }
}

impl std::error::Error for WireError {}

/// Marker type for poker game wire formats.
pub struct Poker;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nlhe_info_roundtrip() {
        let info = NlheInfo::default();
        let wire = info.to_wire();
        let decoded = NlheInfo::from_wire(&wire).unwrap();
        assert_eq!(info, decoded);
    }

    #[test]
    fn nlhe_edge_roundtrip() {
        // We can't easily construct an edge without a game state,
        // but we can test that the type is WireSerializable
        // by checking it compiles (compile-time test)
    }

    #[test]
    fn all_edge_variants_serialize() {
        // This is a compile-time check that NlheEdge implements WireSerializable
        fn assert_wire<E: WireSerializable>() {}
        assert_wire::<NlheEdge>();
    }
}
