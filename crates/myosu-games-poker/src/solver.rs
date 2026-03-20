//! Slice 2 solver wrapper and checkpoint handling for NLHE MCCFR.

use std::io::{Read, Write};
use std::path::Path;

use rbp_core::{Probability, Utility};
use rbp_mccfr::{Profile, Solver};
use rbp_nlhe::{NlheEdge, NlheEncoder, NlheInfo, NlheProfile};
use thiserror::Error;

/// Magic bytes for MYOS checkpoint format.
const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOS";
/// Current checkpoint payload version.
const CHECKPOINT_VERSION: u32 = 1;

/// Configured NLHE solver using the flagship robopoker settings.
pub type Flagship = rbp_nlhe::Flagship;

/// Errors emitted while persisting or reconstructing `PokerSolver`.
#[derive(Debug, Error)]
pub enum PokerSolverError {
    #[error("failed to read checkpoint at {path}: {source}")]
    ReadCheckpoint {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write checkpoint at {path}: {source}")]
    WriteCheckpoint {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid checkpoint magic")]
    InvalidCheckpointMagic,
    #[error("checkpoint version {found}, expected {expected}; re-train required")]
    UnsupportedCheckpointVersion { found: u32, expected: u32 },
    #[error("failed to encode checkpoint payload: {0}")]
    EncodePayload(#[source] bincode::Error),
    #[error("failed to decode checkpoint payload: {0}")]
    DecodePayload(#[source] bincode::Error),
    #[error("failed to clone solver state: {0}")]
    CloneState(#[source] bincode::Error),
}

/// Thin wrapper around robopoker's flagship NLHE solver.
pub struct PokerSolver {
    solver: Flagship,
}

impl PokerSolver {
    /// Create a new solver with an empty profile and the caller-supplied encoder.
    pub fn new(encoder: NlheEncoder) -> Self {
        Self {
            solver: Flagship::new(NlheProfile::default(), encoder),
        }
    }

    /// Rebuild a solver from explicit profile + encoder parts.
    pub fn from_parts(profile: NlheProfile, encoder: NlheEncoder) -> Self {
        Self {
            solver: Flagship::new(profile, encoder),
        }
    }

    /// Load a checkpoint using a caller-supplied encoder artifact.
    ///
    /// The checkpoint stores profile state only. Encoder pinning stays external
    /// so downstream miner and validator lanes can verify shared abstraction
    /// artifacts independently of checkpoint files.
    pub fn load(path: &Path, encoder: NlheEncoder) -> Result<Self, PokerSolverError> {
        let file =
            std::fs::File::open(path).map_err(|source| PokerSolverError::ReadCheckpoint {
                path: path.display().to_string(),
                source,
            })?;
        let mut reader = std::io::BufReader::new(file);

        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|source| PokerSolverError::ReadCheckpoint {
                path: path.display().to_string(),
                source,
            })?;
        if magic != CHECKPOINT_MAGIC {
            return Err(PokerSolverError::InvalidCheckpointMagic);
        }

        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes).map_err(|source| {
            PokerSolverError::ReadCheckpoint {
                path: path.display().to_string(),
                source,
            }
        })?;
        let version = u32::from_le_bytes(version_bytes);
        if version != CHECKPOINT_VERSION {
            return Err(PokerSolverError::UnsupportedCheckpointVersion {
                found: version,
                expected: CHECKPOINT_VERSION,
            });
        }

        let profile = bincode::deserialize_from(reader).map_err(PokerSolverError::DecodePayload)?;
        Ok(Self::from_parts(profile, encoder))
    }

    /// Persist the current profile to a MYOS-framed checkpoint file.
    pub fn save(&self, path: &Path) -> Result<(), PokerSolverError> {
        let file =
            std::fs::File::create(path).map_err(|source| PokerSolverError::WriteCheckpoint {
                path: path.display().to_string(),
                source,
            })?;
        let mut writer = std::io::BufWriter::new(file);

        writer.write_all(&CHECKPOINT_MAGIC).map_err(|source| {
            PokerSolverError::WriteCheckpoint {
                path: path.display().to_string(),
                source,
            }
        })?;
        writer
            .write_all(&CHECKPOINT_VERSION.to_le_bytes())
            .map_err(|source| PokerSolverError::WriteCheckpoint {
                path: path.display().to_string(),
                source,
            })?;
        bincode::serialize_into(&mut writer, self.profile())
            .map_err(PokerSolverError::EncodePayload)?;
        writer
            .flush()
            .map_err(|source| PokerSolverError::WriteCheckpoint {
                path: path.display().to_string(),
                source,
            })?;
        Ok(())
    }

    /// Run `iterations` MCCFR steps.
    ///
    /// Honest training requires a populated abstraction encoder. If the encoder
    /// cannot answer encountered observations, robopoker will panic while
    /// expanding sampled trees.
    pub fn train(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.solver.step();
        }
    }

    /// Get the current training epoch count.
    pub fn epochs(&self) -> usize {
        self.profile().epochs()
    }

    /// Borrow the underlying encoder.
    pub fn encoder(&self) -> &NlheEncoder {
        &self.solver.encoder
    }

    /// Borrow the underlying profile.
    pub fn profile(&self) -> &NlheProfile {
        &self.solver.profile
    }

    /// Return the average strategy at an information set.
    pub fn strategy(&self, info: &NlheInfo) -> Vec<(NlheEdge, Probability)> {
        self.profile()
            .averaged_distribution(info)
            .into_iter()
            .collect()
    }

    /// Compute exploitability of the current profile.
    ///
    /// When the solver lacks a usable encoder or the profile produces a
    /// non-finite value, return `INFINITY` instead of propagating panics/NaNs.
    pub fn exploitability(&self) -> Utility {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.solver.exploitability()
        }))
        .ok()
        .filter(|value| value.is_finite())
        .unwrap_or(f32::INFINITY)
    }

    /// Deep-clone the solver through the serializable state.
    pub fn snapshot_profile(&self) -> Result<Self, PokerSolverError> {
        let profile = bincode::serialize(self.profile())
            .and_then(|bytes| bincode::deserialize::<NlheProfile>(&bytes))
            .map_err(PokerSolverError::CloneState)?;
        let encoder = bincode::serialize(self.encoder())
            .and_then(|bytes| bincode::deserialize::<NlheEncoder>(&bytes))
            .map_err(PokerSolverError::CloneState)?;
        Ok(Self::from_parts(profile, encoder))
    }
}

impl Default for PokerSolver {
    fn default() -> Self {
        let empty_encoder = bincode::deserialize(&[0u8; 8]).expect("empty encoder fixture");
        Self::new(empty_encoder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_core::Arbitrary;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn empty_encoder() -> NlheEncoder {
        bincode::deserialize(&[0u8; 8]).expect("empty encoder fixture")
    }

    fn checkpoint_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "myosu-games-poker-{label}-{}-{nanos}.bin",
            std::process::id()
        ))
    }

    #[test]
    fn create_empty_solver() {
        let solver = PokerSolver::new(empty_encoder());
        assert_eq!(solver.epochs(), 0);
    }

    #[test]
    fn strategy_is_valid_distribution() {
        let solver = PokerSolver::new(empty_encoder());
        let info = NlheInfo::random();
        let strategy = solver.strategy(&info);
        let total: Probability = strategy.iter().map(|(_, probability)| *probability).sum();

        assert!(!strategy.is_empty());
        assert!(
            (total - 1.0).abs() < 0.001,
            "expected probability mass ~1.0, got {total}"
        );
    }

    #[test]
    fn checkpoint_roundtrip() {
        let path = checkpoint_path("roundtrip");
        let solver = PokerSolver::new(empty_encoder());
        let info = NlheInfo::random();
        let expected_strategy = solver.strategy(&info);

        solver.save(&path).expect("save checkpoint");
        let loaded = PokerSolver::load(&path, empty_encoder()).expect("load checkpoint");

        assert_eq!(loaded.epochs(), solver.epochs());
        assert_eq!(loaded.strategy(&info), expected_strategy);

        std::fs::remove_file(path).expect("remove checkpoint");
    }

    #[test]
    fn rejects_invalid_checkpoint_magic() {
        let path = checkpoint_path("bad-magic");
        std::fs::write(&path, b"NOPE\x01\0\0\0").expect("write invalid checkpoint");

        let error = PokerSolver::load(&path, empty_encoder())
            .err()
            .expect("invalid magic");
        assert!(matches!(error, PokerSolverError::InvalidCheckpointMagic));

        std::fs::remove_file(path).expect("remove invalid checkpoint");
    }

    #[test]
    fn rejects_unsupported_checkpoint_version() {
        let path = checkpoint_path("bad-version");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&(CHECKPOINT_VERSION + 1).to_le_bytes());
        std::fs::write(&path, bytes).expect("write invalid checkpoint");

        let error = PokerSolver::load(&path, empty_encoder())
            .err()
            .expect("unsupported version");
        assert!(matches!(
            error,
            PokerSolverError::UnsupportedCheckpointVersion { found, expected }
                if found == CHECKPOINT_VERSION + 1 && expected == CHECKPOINT_VERSION
        ));

        std::fs::remove_file(path).expect("remove invalid checkpoint");
    }

    #[test]
    #[ignore = "requires a populated abstraction artifact / RF-02 to train honestly"]
    fn train_100_iterations() {
        panic!("requires a populated abstraction artifact / RF-02");
    }

    #[test]
    #[ignore = "requires a populated abstraction artifact / RF-02 to measure exploitability honestly"]
    fn exploitability_decreases() {
        panic!("requires a populated abstraction artifact / RF-02");
    }
}
