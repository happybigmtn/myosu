//! PokerSolver — NLHE solver wrapper with checkpoint persistence.
//!
//! Wraps `rbp_nlhe::Flagship` = `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`
//! and adds file-based checkpoint save/load with the MYOS format.

use rbp_core::{Probability, Utility};
use rbp_mccfr::{CfrGame, Encoder, Profile, Solver};
use rbp_nlhe::{NlheEdge, NlheEncoder, NlheGame, NlheInfo, NlheProfile};
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use thiserror::Error;

/// Checkpoint magic bytes ("MYOS" in ASCII)
const CHECKPOINT_MAGIC: [u8; 4] = [0x4D, 0x59, 0x4F, 0x53];
/// Checkpoint format version
const CHECKPOINT_VERSION: u32 = 1;

/// Errors that can occur during solver operations.
#[derive(Error, Debug)]
pub enum PokerSolverError {
    #[error("failed to open checkpoint file: {path}")]
    FileOpen {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("checkpoint has invalid magic: expected MYOS, found {found:?}")]
    InvalidMagic { found: Vec<u8> },
    #[error("checkpoint has unsupported version: {version}")]
    UnsupportedVersion { version: u32 },
    #[error("checkpoint is corrupted")]
    CorruptedCheckpoint(#[source] bincode::Error),
    #[error("failed to serialize checkpoint")]
    Serialization(#[source] bincode::Error),
    #[error("failed to read encoder artifact: {path}")]
    EncoderArtifactRead {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to decode encoder artifact")]
    EncoderArtifactDecode(#[source] bincode::Error),
    #[error("encoder abstractions are unavailable during {context}")]
    MissingEncoderAbstractions { context: &'static str },
    #[error("solver panicked during {context}: {message}")]
    OperationPanicked {
        context: &'static str,
        message: String,
    },
    #[error("exploitability is not finite: {value}")]
    InvalidExploitability { value: Utility },
}

/// NLHE poker solver with training and persistence.
///
/// Wraps `rbp_nlhe::Flagship` and exposes:
/// - `train(iterations)` — run CFR iterations
/// - `strategy(&NlheInfo)` — query average strategy at an information set
/// - `exploitability()` — compute Nash equilibrium distance in mb/100
/// - `epochs()` — number of training iterations completed
/// - `save(path)` / `load(path)` — checkpoint persistence
pub struct PokerSolver {
    inner: rbp_nlhe::Flagship,
}

impl PokerSolver {
    /// Creates a new solver with an empty profile and no abstraction data.
    ///
    /// This is sufficient for checkpoint serialization and strategy lookups over
    /// explicit [`NlheInfo`] values, but training and exploitability require a
    /// populated encoder loaded from abstraction artifacts.
    pub fn new() -> Self {
        Self {
            inner: rbp_nlhe::Flagship::new(NlheProfile::default(), NlheEncoder::default()),
        }
    }

    /// Creates a solver backed by an externally-provided encoder.
    pub fn with_encoder(encoder: NlheEncoder) -> Result<Self, PokerSolverError> {
        let solver = Self {
            inner: rbp_nlhe::Flagship::new(NlheProfile::default(), encoder),
        };
        solver.validate_abstractions()?;
        Ok(solver)
    }

    /// Creates a solver backed by a bincode-serialized encoder artifact.
    pub fn with_encoder_bytes(bytes: &[u8]) -> Result<Self, PokerSolverError> {
        Self::with_encoder(Self::decode_encoder(bytes)?)
    }

    /// Creates a solver backed by a file-based encoder artifact.
    pub fn with_encoder_file(path: &Path) -> Result<Self, PokerSolverError> {
        let bytes = Self::read_encoder_artifact(path)?;
        Self::with_encoder_bytes(&bytes)
    }

    /// Verifies that the encoder can at least construct the root infoset.
    pub fn validate_abstractions(&self) -> Result<(), PokerSolverError> {
        catch_unwind(AssertUnwindSafe(|| {
            let _ = self.inner.encoder.seed(&NlheGame::root());
        }))
        .map_err(|payload| Self::map_operation_panic("encoder validation", payload))?;
        Ok(())
    }

    /// Trains the solver for `iterations` MCCFR iterations.
    pub fn train(&mut self, iterations: usize) -> Result<(), PokerSolverError> {
        for _ in 0..iterations {
            catch_unwind(AssertUnwindSafe(|| self.inner.step()))
                .map_err(|payload| Self::map_operation_panic("training", payload))?;
        }
        Ok(())
    }

    /// Returns the current number of training iterations (epochs).
    pub fn epochs(&self) -> usize {
        self.inner.profile().epochs()
    }

    /// Returns the average strategy distribution at the given information set.
    ///
    /// Returns action-probability pairs for all legal actions at the info set.
    /// The probabilities sum to 1.0 (within floating-point tolerance).
    pub fn strategy(&self, info: &NlheInfo) -> Vec<(NlheEdge, Probability)> {
        use rbp_mccfr::Profile;
        self.inner.profile().averaged_distribution(info)
    }

    /// Computes exploitability of the current average strategy.
    ///
    /// Returns the Nash equilibrium distance in milli-big-blinds per hand (mbb/h).
    /// Lower values indicate strategies closer to equilibrium.
    /// A trained strategy should have exploitability significantly below random play.
    pub fn exploitability(&self) -> Result<Utility, PokerSolverError> {
        let value = catch_unwind(AssertUnwindSafe(|| Solver::exploitability(&self.inner)))
            .map_err(|payload| Self::map_operation_panic("exploitability", payload))?;

        if !value.is_finite() {
            return Err(PokerSolverError::InvalidExploitability { value });
        }

        Ok(value)
    }

    /// Saves the solver state to a checkpoint file.
    ///
    /// Format: 4-byte "MYOS" magic + u32 version + bincode(NlheProfile)
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be opened or data cannot be serialized.
    pub fn save(&self, path: &Path) -> Result<(), PokerSolverError> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|source| PokerSolverError::FileOpen {
                path: path.display().to_string(),
                source,
            })?;

        // Write magic
        file.write_all(&CHECKPOINT_MAGIC)
            .map_err(|source| PokerSolverError::FileOpen {
                path: path.display().to_string(),
                source,
            })?;

        // Write version
        file.write_all(&CHECKPOINT_VERSION.to_le_bytes())
            .map_err(|source| PokerSolverError::FileOpen {
                path: path.display().to_string(),
                source,
            })?;

        // Write profile
        bincode::serialize_into(&file, &self.inner.profile)
            .map_err(PokerSolverError::Serialization)?;

        Ok(())
    }

    /// Loads the solver state from a checkpoint file.
    ///
    /// Validates the magic bytes and version before loading.
    ///
    /// # Errors
    ///
    /// Returns error if the file cannot be opened, has invalid format,
    /// or contains unsupported version.
    pub fn load(path: &Path) -> Result<Self, PokerSolverError> {
        let profile = Self::read_profile(path)?;
        Ok(Self {
            inner: rbp_nlhe::Flagship::new(profile, NlheEncoder::default()),
        })
    }

    /// Loads a checkpoint and pairs it with a caller-provided encoder.
    pub fn load_with_encoder(path: &Path, encoder: NlheEncoder) -> Result<Self, PokerSolverError> {
        let profile = Self::read_profile(path)?;
        let solver = Self {
            inner: rbp_nlhe::Flagship::new(profile, encoder),
        };
        solver.validate_abstractions()?;
        Ok(solver)
    }

    /// Loads a checkpoint and pairs it with a serialized encoder artifact.
    pub fn load_with_encoder_bytes(
        path: &Path,
        encoder_bytes: &[u8],
    ) -> Result<Self, PokerSolverError> {
        Self::load_with_encoder(path, Self::decode_encoder(encoder_bytes)?)
    }

    /// Loads a checkpoint and pairs it with an encoder artifact from disk.
    pub fn load_with_encoder_file(
        path: &Path,
        encoder_path: &Path,
    ) -> Result<Self, PokerSolverError> {
        let bytes = Self::read_encoder_artifact(encoder_path)?;
        Self::load_with_encoder_bytes(path, &bytes)
    }

    fn read_profile(path: &Path) -> Result<NlheProfile, PokerSolverError> {
        let mut file = File::open(path).map_err(|source| PokerSolverError::FileOpen {
            path: path.display().to_string(),
            source,
        })?;

        // Read and validate magic
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)
            .map_err(|source| PokerSolverError::FileOpen {
                path: path.display().to_string(),
                source,
            })?;
        if magic != CHECKPOINT_MAGIC {
            return Err(PokerSolverError::InvalidMagic {
                found: magic.to_vec(),
            });
        }

        // Read and validate version
        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)
            .map_err(|source| PokerSolverError::FileOpen {
                path: path.display().to_string(),
                source,
            })?;
        let version = u32::from_le_bytes(version_bytes);
        if version != CHECKPOINT_VERSION {
            return Err(PokerSolverError::UnsupportedVersion { version });
        }

        // Load profile
        bincode::deserialize_from(&file).map_err(PokerSolverError::CorruptedCheckpoint)
    }

    fn decode_encoder(bytes: &[u8]) -> Result<NlheEncoder, PokerSolverError> {
        bincode::deserialize(bytes).map_err(PokerSolverError::EncoderArtifactDecode)
    }

    fn read_encoder_artifact(path: &Path) -> Result<Vec<u8>, PokerSolverError> {
        std::fs::read(path).map_err(|source| PokerSolverError::EncoderArtifactRead {
            path: path.display().to_string(),
            source,
        })
    }

    /// Returns a reference to the inner solver for use by other crate modules.
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &rbp_nlhe::Flagship {
        &self.inner
    }

    /// Returns a mutable reference to the inner solver.
    #[allow(dead_code)]
    pub(crate) fn mut_inner(&mut self) -> &mut rbp_nlhe::Flagship {
        &mut self.inner
    }

    fn map_operation_panic(
        context: &'static str,
        payload: Box<dyn Any + Send>,
    ) -> PokerSolverError {
        let message = panic_message(payload.as_ref());
        if message.contains("isomorphism not found in abstraction lookup") {
            PokerSolverError::MissingEncoderAbstractions { context }
        } else {
            PokerSolverError::OperationPanicked { context, message }
        }
    }
}

impl Default for PokerSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PokerSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PokerSolver")
            .field("epochs", &self.epochs())
            .finish()
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else {
        "non-string panic payload".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::root_encoder_bytes;
    use rbp_core::Arbitrary;
    use std::fs;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn create_empty_solver() {
        let solver = PokerSolver::new();
        assert_eq!(solver.epochs(), 0);
    }

    #[test]
    fn train_100_iterations() {
        let mut solver = PokerSolver::new();
        let error = solver.train(100).unwrap_err();

        assert!(matches!(
            error,
            PokerSolverError::MissingEncoderAbstractions {
                context: "training"
            }
        ));
        assert_eq!(solver.epochs(), 0);
    }

    #[test]
    fn strategy_is_valid_distribution() {
        let solver = PokerSolver::new();
        let info = rbp_nlhe::NlheInfo::random();
        let strategy = solver.strategy(&info);
        let sum: f32 = strategy.iter().map(|(_, p)| *p).sum();

        assert!(
            !strategy.is_empty(),
            "random infos should expose legal actions"
        );
        assert!((sum - 1.0).abs() < 0.001, "probabilities should sum to 1.0");
    }

    #[test]
    fn checkpoint_roundtrip() {
        let solver = PokerSolver::new();
        let info = rbp_nlhe::NlheInfo::random();
        let baseline_strategy = solver.strategy(&info);

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        solver.save(temp_path).unwrap();

        // Verify magic bytes
        let mut file = File::open(temp_path).unwrap();
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).unwrap();
        assert_eq!(&magic, b"MYOS");

        // Verify version
        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes).unwrap();
        assert_eq!(u32::from_le_bytes(version_bytes), 1u32);

        // Load and verify
        let loaded = PokerSolver::load(temp_path).unwrap();
        assert_eq!(loaded.epochs(), 0);
        assert_eq!(loaded.strategy(&info), baseline_strategy);
    }

    #[test]
    fn exploitability_decreases() {
        let solver = PokerSolver::new();
        let error = solver.exploitability().unwrap_err();

        assert!(matches!(
            error,
            PokerSolverError::MissingEncoderAbstractions {
                context: "exploitability"
            }
        ));
    }

    #[test]
    fn with_encoder_rejects_empty_abstraction_map() {
        let error = PokerSolver::with_encoder(NlheEncoder::default()).unwrap_err();

        assert!(matches!(
            error,
            PokerSolverError::MissingEncoderAbstractions {
                context: "encoder validation"
            }
        ));
    }

    #[test]
    fn with_encoder_bytes_accepts_root_artifact() {
        let solver = PokerSolver::with_encoder_bytes(root_encoder_bytes()).unwrap();
        assert_eq!(solver.epochs(), 0);
    }

    #[test]
    fn with_encoder_file_accepts_root_artifact() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), root_encoder_bytes()).unwrap();

        let solver = PokerSolver::with_encoder_file(temp_file.path()).unwrap();

        assert_eq!(solver.epochs(), 0);
    }

    #[test]
    fn checkpoint_roundtrip_with_encoder_artifact_preserves_epoch() {
        let solver = PokerSolver::with_encoder_bytes(root_encoder_bytes()).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        solver.save(temp_file.path()).unwrap();

        let loaded = PokerSolver::load_with_encoder_bytes(temp_file.path(), root_encoder_bytes())
            .unwrap();
        assert_eq!(loaded.epochs(), 0);
    }
}
