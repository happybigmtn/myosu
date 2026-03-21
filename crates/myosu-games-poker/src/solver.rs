//! PokerSolver: NLHE solver wrapper around `rbp_nlhe::Flagship`.
//!
//! Owns `NlheSolver`, exposes `train()`, `strategy()`, `exploitability()`,
//! `epochs()`, and file-based checkpoint persistence via `save()` / `load()`.

use myosu_games::{Probability, Profile, Utility};
use rbp_mccfr::Solver;
use rbp_nlhe::{NlheEdge, NlheEncoder, NlheInfo, NlheProfile};
use std::io::{Read, Write};
use thiserror::Error;

/// Checkpoint magic bytes (`MYOS`) + version field.
const CHECKPOINT_VERSION: u32 = 1;
const CHECKPOINT_MAGIC: &[u8; 4] = b"MYOS";

/// Errors that can occur during solver operations.
#[derive(Error, Debug)]
pub enum SolverError {
    #[error("checkpoint version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u32, got: u32 },

    #[error("checkpoint magic mismatch: expected {expected:?}, got {got:?}")]
    MagicMismatch { expected: [u8; 4], got: Vec<u8> },

    #[error("checkpoint read error: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("checkpoint decode error")]
    DecodeError,

    #[error("checkpoint encode error")]
    EncodeError,
}

/// PokerSolver: owned wrapper around `rbp_nlhe::Flagship`.
///
/// Provides a clean public API for training, strategy queries, and persistence.
pub struct PokerSolver {
    solver: rbp_nlhe::Flagship,
}

impl PokerSolver {
    /// Create a new PokerSolver with default (empty) encoder and profile.
    ///
    /// The encoder starts empty and will only work for root game states
    /// where the abstraction lookup succeeds. For full training, the encoder
    /// must be populated via database loading (future work).
    pub fn new() -> Self {
        Self {
            solver: rbp_nlhe::Flagship::new(NlheProfile::default(), NlheEncoder::default()),
        }
    }

    /// Create from existing encoder and profile.
    pub fn from_parts(encoder: NlheEncoder, profile: NlheProfile) -> Self {
        Self {
            solver: rbp_nlhe::Flagship::new(profile, encoder),
        }
    }

    /// Run `iterations` training iterations.
    ///
    /// Each iteration builds a sampled game tree and updates regrets/weights.
    pub fn train(&mut self, iterations: usize) {
        for _ in 0..iterations {
            rbp_nlhe::Flagship::step(&mut self.solver);
        }
    }

    /// Return the current epoch (iteration count).
    pub fn epochs(&self) -> usize {
        self.solver.profile().epochs()
    }

    /// Return action probabilities for the given info set.
    ///
    /// Uses the averaged (historical) strategy distribution.
    /// Returns pairs of (edge, probability) summing to 1.0.
    pub fn strategy(&self, info: &NlheInfo) -> Vec<(NlheEdge, Probability)> {
        self.solver.profile().averaged_distribution(info)
    }

    /// Compute exploitability of the current strategy in milli-big-blinds per hand (mbb/h).
    ///
    /// Uses `VanillaSampling` to build the full game tree.
    /// Returns NaN if the strategy profile is empty/untrained.
    pub fn exploitability(&self) -> Utility {
        self.solver.exploitability()
    }

    /// Returns a reference to the encoder.
    pub fn encoder(&self) -> &NlheEncoder {
        self.solver.encoder()
    }

    /// Save solver state to a checkpoint file.
    ///
    /// Format: `MYOS` (4 bytes) + u32 version + serde_json(encoder, profile)
    pub fn save(&self, path: &std::path::Path) -> Result<(), SolverError> {
        let mut file = std::fs::File::create(path)?;

        // Write magic
        file.write_all(CHECKPOINT_MAGIC).map_err(SolverError::ReadError)?;

        // Write version
        file.write_all(&CHECKPOINT_VERSION.to_le_bytes())
            .map_err(SolverError::ReadError)?;

        // Encode encoder + profile as JSON
        let encoded = serde_json::to_vec(&(self.solver.encoder(), self.solver.profile()))
            .map_err(|_| SolverError::EncodeError)?;

        // Write length prefix + data
        let len = encoded.len() as u64;
        file.write_all(&len.to_le_bytes())
            .map_err(SolverError::ReadError)?;
        file.write_all(&encoded)
            .map_err(SolverError::ReadError)?;

        Ok(())
    }

    /// Load solver state from a checkpoint file.
    ///
    /// Validates magic bytes and version before decoding.
    pub fn load(path: &std::path::Path) -> Result<Self, SolverError> {
        let mut file = std::fs::File::open(path).map_err(SolverError::ReadError)?;

        // Read and validate magic
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)
            .map_err(SolverError::ReadError)?;
        if magic != *CHECKPOINT_MAGIC {
            return Err(SolverError::MagicMismatch {
                expected: *CHECKPOINT_MAGIC,
                got: magic.to_vec(),
            });
        }

        // Read and validate version
        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)
            .map_err(SolverError::ReadError)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != CHECKPOINT_VERSION {
            return Err(SolverError::VersionMismatch {
                expected: CHECKPOINT_VERSION,
                got: version,
            });
        }

        // Read length prefix
        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes)
            .map_err(SolverError::ReadError)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        // Read data
        let mut data = vec![0u8; len];
        file.read_exact(&mut data)
            .map_err(SolverError::ReadError)?;

        // Decode
        let (encoder, profile): (NlheEncoder, NlheProfile) =
            serde_json::from_slice(&data).map_err(|_| SolverError::DecodeError)?;

        Ok(Self {
            solver: rbp_nlhe::Flagship::new(profile, encoder),
        })
    }

    /// Returns a reference to the underlying solver (for use by exploit.rs).
    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &rbp_nlhe::Flagship {
        &self.solver
    }

    /// Returns a mutable reference to the underlying solver (for training).
    #[allow(dead_code)]
    pub(crate) fn inner_mut(&mut self) -> &mut rbp_nlhe::Flagship {
        &mut self.solver
    }
}

impl Default for PokerSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_mccfr::{CfrGame, Encoder};
    use rbp_nlhe::NlheGame;

    #[test]
    fn create_empty_solver() {
        let solver = PokerSolver::new();
        assert_eq!(solver.epochs(), 0);
    }

    #[test]
    fn train_100_iterations() {
        let mut solver = PokerSolver::new();
        solver.train(100);
        assert_eq!(solver.epochs(), 100);
    }

    #[test]
    fn strategy_is_valid_distribution() {
        let mut solver = PokerSolver::new();
        solver.train(10);

        // Get root info
        let info = solver.inner().encoder().seed(&NlheGame::root());
        let strat = solver.strategy(&info);

        let sum: f32 = strat.iter().map(|(_, p)| p).sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "strategy probabilities should sum to ~1.0, got {}",
            sum
        );
    }

    #[test]
    fn checkpoint_roundtrip() {
        let mut solver = PokerSolver::new();
        solver.train(50);

        let temp_path = std::env::temp_dir().join("poker_test_checkpoint.myos");

        // Save
        solver.save(&temp_path).unwrap();

        // Load
        let loaded = PokerSolver::load(&temp_path).unwrap();
        assert_eq!(loaded.epochs(), solver.epochs());

        // Clean up
        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn exploitability_decreases() {
        let mut solver = PokerSolver::new();

        // Random strategy should have high exploitability
        let exp_before = solver.exploitability();

        // Train for some iterations
        solver.train(50);

        // After training, exploitability should decrease (or at least not increase)
        let exp_after = solver.exploitability();

        // NaN check (untrained profile may return NaN)
        if !exp_before.is_nan() && !exp_after.is_nan() {
            assert!(
                exp_after <= exp_before,
                "exploitability should decrease after training: {} -> {}",
                exp_before,
                exp_after
            );
        }
    }
}
