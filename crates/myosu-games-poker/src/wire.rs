//! Wire serialization for NLHE types.
//!
//! Provides JSON roundtrip for `NlheInfo` and `NlheEdge`.
//! Requires the `serde` feature on robopoker crates (verified in Slice 1).

use myosu_games::Probability;
use rbp_nlhe::{NlheEdge, NlheInfo};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors from wire serialization/deserialization.
#[derive(Error, Debug)]
pub enum WireError {
    #[error("encode error: {0}")]
    Encode(serde_json::Error),

    #[error("decode error: {0}")]
    Decode(serde_json::Error),
}

/// Marker type for wire serialization of poker types.
pub struct Poker;

/// Trait for types that can be serialized to/from wire format.
pub trait WireSerializable: Sized + Serialize + for<'de> Deserialize<'de> {
    /// Serialize to a byte vector.
    fn to_wire(&self) -> Result<Vec<u8>, WireError>;

    /// Deserialize from a byte slice.
    fn from_wire(bytes: &[u8]) -> Result<Self, WireError>;
}

impl WireSerializable for NlheInfo {
    fn to_wire(&self) -> Result<Vec<u8>, WireError> {
        serde_json::to_vec(self).map_err(WireError::Encode)
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, WireError> {
        serde_json::from_slice(bytes).map_err(WireError::Decode)
    }
}

impl WireSerializable for NlheEdge {
    fn to_wire(&self) -> Result<Vec<u8>, WireError> {
        serde_json::to_vec(self).map_err(WireError::Encode)
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, WireError> {
        serde_json::from_slice(bytes).map_err(WireError::Decode)
    }
}

/// Wire format for a strategy query/response.
///
/// Contains serialized `NlheInfo` and an optional serialized action distribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireStrategy {
    /// Serialized `NlheInfo` (information set).
    pub info_bytes: Vec<u8>,
    /// Serialized action distribution `Vec<(NlheEdge, Probability)>`.
    pub actions_bytes: Vec<u8>,
}

impl WireStrategy {
    /// Create a new wire strategy with info bytes.
    pub fn new(info_bytes: Vec<u8>) -> Self {
        Self {
            info_bytes,
            actions_bytes: Vec::new(),
        }
    }

    /// Deserialize the info field.
    pub fn info(&self) -> Result<NlheInfo, WireError> {
        NlheInfo::from_wire(&self.info_bytes)
    }

    /// Set the actions by serializing the given distribution.
    pub fn set_actions(&mut self, actions: &[(NlheEdge, Probability)]) {
        self.actions_bytes = serde_json::to_vec(actions).unwrap_or_default();
    }

    /// Deserialize the actions field.
    pub fn actions(&self) -> Result<Vec<(NlheEdge, Probability)>, WireError> {
        if self.actions_bytes.is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_slice(&self.actions_bytes).map_err(WireError::Decode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_mccfr::{CfrGame, Encoder};
    use rbp_nlhe::NlheGame;

    #[test]
    fn nlhe_info_roundtrip() {
        // Create a solver just to get a valid root info
        let solver = super::super::PokerSolver::new();
        let info = solver.encoder().seed(&NlheGame::root());

        // Roundtrip
        let wire = info.to_wire().unwrap();
        let recovered = NlheInfo::from_wire(&wire).unwrap();

        assert_eq!(info.street(), recovered.street());
        assert_eq!(info.subgame(), recovered.subgame());
    }

    #[test]
    fn nlhe_edge_roundtrip() {
        let game = NlheGame::root();
        let info = rbp_nlhe::NlheEncoder::default().seed(&game);

        // Get first available edge and convert to NlheEdge
        let edge = info.choices().into_iter().next().unwrap();
        let nlhe_edge = NlheEdge::from(edge);

        let wire = nlhe_edge.to_wire().unwrap();
        let recovered = NlheEdge::from_wire(&wire).unwrap();

        assert_eq!(nlhe_edge, recovered);
    }

    #[test]
    fn all_edge_variants_serialize() {
        let game = NlheGame::root();
        let info = rbp_nlhe::NlheEncoder::default().seed(&game);

        for edge in info.choices().into_iter() {
            let nlhe_edge = NlheEdge::from(edge);
            let wire = nlhe_edge.to_wire();
            assert!(
                wire.is_ok(),
                "edge {:?} should serialize, got {:?}",
                nlhe_edge,
                wire.err()
            );

            let recovered = NlheEdge::from_wire(&wire.unwrap()).unwrap();
            assert_eq!(
                nlhe_edge, recovered,
                "edge {:?} should roundtrip correctly",
                nlhe_edge
            );
        }
    }
}
