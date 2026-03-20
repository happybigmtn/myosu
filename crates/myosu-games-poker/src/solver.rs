//! PokerSolver — NLHE solver wrapper with training and persistence.
//!
//! Wraps `rbp_nlhe::Flagship` (Pluribus-configured MCCFR solver) and exposes:
//! - Training via `train(iterations)`
//! - Strategy queries via `strategy(&NlheInfo)`
//! - Exploitability computation via `exploitability()`
//! - Checkpoint save/load via `save(path)` / `load(path)`
//!
//! # Checkpoint Format
//!
//! - 4-byte magic: `b"MYOS"`
//! - u32 version: currently version 1
//! - bincode-encoded `NlheProfile`

use anyhow::{Context, Result};
use rbp_core::{Probability, Utility};
use rbp_mccfr::{
    CfrGame, Encoder, PluribusRegret, LinearWeight, PluribusSampling,
    Profile, Solver, TreeBuilder,
};
use rbp_nlhe::{NlheEncoder, NlheGame, NlheInfo, NlheProfile, NlheSolver};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use thiserror::Error;

/// Checkpoint magic bytes.
const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOS";
/// Checkpoint format version.
const CHECKPOINT_VERSION: u32 = 1;

/// Errors that can occur during solver operations.
#[derive(Error, Debug)]
pub enum SolverError {
    #[error("checkpoint has unsupported version {0} (expected {1})")]
    UnsupportedVersion(u32, u32),

    #[error("checkpoint has invalid magic bytes")]
    InvalidMagic,

    #[error("checkpoint file is corrupted: {0}")]
    CorruptedFile(String),

    #[error("io error during checkpoint operation: {0}")]
    IoError(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
}

/// PokerSolver wraps the robopoker NLHE MCCFR solver.
///
/// This is the top-level entry point for the poker engine. It owns a solver
/// instance and exposes a clean API for training, querying, and persistence.
pub struct PokerSolver {
    solver: NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>,
}

impl PokerSolver {
    /// Create a new poker solver with default (empty) profile and encoder.
    pub fn new() -> Self {
        Self {
            solver: NlheSolver::new(NlheProfile::default(), NlheEncoder::default()),
        }
    }

    /// Run `iterations` training iterations.
    ///
    /// Each iteration performs one CFR update pass, alternating the traversing player.
    pub fn train(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.solver.step();
        }
    }

    /// Get the current number of training epochs completed.
    pub fn epochs(&self) -> usize {
        self.solver.profile().epochs()
    }

    /// Get the strategy distribution for a given information set.
    ///
    /// Returns a vector of (edge, probability) pairs. Probabilities sum to 1.0
    /// (within floating-point tolerance).
    pub fn strategy(&self, info: &NlheInfo) -> Vec<(rbp_nlhe::NlheEdge, Probability)> {
        let profile = self.solver.profile();
        info.choices()
            .into_iter()
            .map(|edge| {
                let nlhe_edge = rbp_nlhe::NlheEdge::from(edge);
                let prob = profile.averaged(info, &nlhe_edge);
                (nlhe_edge, prob)
            })
            .collect()
    }

    /// Compute exploitability of the current strategy profile in mbb/h (milli-big-blinds per hand).
    ///
    /// A fully converged Nash equilibrium has exploitability of 0.
    /// Returns `f64::INFINITY` if exploitability cannot be computed (e.g., empty profile).
    pub fn exploitability(&self) -> rbp_core::Utility {
        let tree = TreeBuilder::<_, _, _, _, _, _, rbp_mccfr::VanillaSampling>::new(
            self.solver.encoder(),
            self.solver.profile(),
            NlheGame::root(),
        )
        .build();

        let exploit = self.solver.profile().exploitability(tree);
        // Convert to mbb/h: utility is in chips, scale appropriately
        // Robopoker uses big blinds as the unit, so we multiply by 1000 for milli-big-blinds
        exploit * 1000.0
    }

    /// Save the solver state to a checkpoint file.
    ///
    /// The checkpoint format is:
    /// - 4-byte magic: `b"MYOS"`
    /// - u32 version: 1
    /// - bincode-encoded `NlheProfile`
    pub fn save(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path).context("failed to create checkpoint file")?;

        // Write magic
        file.write_all(&CHECKPOINT_MAGIC)
            .context("failed to write magic bytes")?;

        // Write version
        let version_bytes = CHECKPOINT_VERSION.to_le_bytes();
        file.write_all(&version_bytes)
            .context("failed to write version")?;

        // Encode profile using bincode 1.x API
        let encoded = bincode::serialize(&self.solver.profile())
            .context("failed to encode profile")?;
        // Write length as 8 bytes
        let len_bytes = (encoded.len() as u64).to_le_bytes();
        file.write_all(&len_bytes)
            .context("failed to write length")?;
        file.write_all(&encoded)
            .context("failed to write profile data")?;

        Ok(())
    }

    /// Load the solver state from a checkpoint file.
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path).context("failed to open checkpoint file")?;

        // Read and verify magic
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).context("failed to read magic")?;
        if magic != CHECKPOINT_MAGIC {
            return Err(SolverError::InvalidMagic).context("checkpoint has invalid magic");
        }

        // Read version
        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)
            .context("failed to read version")?;
        let version = u32::from_le_bytes(version_bytes);
        if version != CHECKPOINT_VERSION {
            return Err(SolverError::UnsupportedVersion(version, CHECKPOINT_VERSION))
                .context("checkpoint version mismatch");
        }

        // Read length
        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes)
            .context("failed to read length")?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        // Read profile data
        let mut profile_data = vec![0u8; len];
        file.read_exact(&mut profile_data)
            .context("failed to read profile data")?;

        // Decode profile using bincode 1.x API
        let profile = bincode::deserialize(&profile_data)
            .context("failed to decode profile")?;

        Ok(Self {
            solver: NlheSolver::new(profile, NlheEncoder::default()),
        })
    }

    /// Get the underlying solver reference (for advanced use).
    pub fn inner(&self) -> &NlheSolver<PluribusRegret, LinearWeight, PluribusSampling> {
        &self.solver
    }
}

impl Default for PokerSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for the flagship NLHE solver configuration.
pub type Flagship = NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>;

#[cfg(test)]
mod tests {
    use super::*;

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
        solver.train(50);

        // Query strategy at root info set
        let root_info = solver.solver.encoder().seed(&NlheGame::root());
        let distribution = solver.strategy(&root_info);

        // Should have some actions
        assert!(!distribution.is_empty());

        // Probabilities should sum to ~1.0
        let sum: Probability = distribution.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 0.001, "probabilities sum to {}", sum);
    }

    #[test]
    fn checkpoint_roundtrip() {
        let mut solver = PokerSolver::new();
        solver.train(50);

        let temp_path = std::env::temp_dir().join("poker_test_checkpoint.myos");
        solver.save(&temp_path).unwrap();

        let loaded = PokerSolver::load(&temp_path).unwrap();
        assert_eq!(loaded.epochs(), solver.epochs());

        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn exploitability_decreases() {
        let mut solver = PokerSolver::new();

        // Initial exploitability (untrained)
        let initial_exploit = solver.exploitability();

        // Train for a while
        solver.train(200);

        // Trained exploitability should be lower (or at least not worse)
        let trained_exploit = solver.exploitability();

        // Note: with very limited training this may not converge,
        // but we at least check it doesn't go to infinity
        assert!(
            trained_exploit.is_finite(),
            "exploitability should be finite, got {}",
            trained_exploit
        );
        assert!(
            trained_exploit <= initial_exploit || initial_exploit.is_infinite(),
            "trained ({}) should have <= exploitability than initial ({})",
            trained_exploit,
            initial_exploit
        );
    }
}
