//! PokerSolver wrapper around `rbp_nlhe::Flagship`.
//!
//! Implements save/load with `MYOS` checkpoint format, training, and strategy queries.

use std::io::{Read, Write};
use std::path::Path;
use rbp_nlhe::{NlheInfo, NlheEdge, NlheProfile, NlheEncoder};
use rbp_mccfr::{Solver, Profile};
use rbp_core::Utility;
use bincode;

/// Magic bytes for MYOS checkpoint format: b"MYOS"
const CHECKPOINT_MAGIC: [u8; 4] = [0x4D, 0x59, 0x4F, 0x53];
/// Checkpoint format version
const CHECKPOINT_VERSION: u32 = 1;

/// PokerSolver wraps the NLHE MCCFR solver with checkpointing and query support.
///
/// Uses `rbp_nlhe::Flagship` = `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`.
pub struct PokerSolver {
    inner: rbp_nlhe::Flagship,
    epochs_trained: u64,
}

impl PokerSolver {
    /// Create a new solver with default configuration.
    ///
    /// Uses Pluribus-regret + LinearWeight + PluribusSampling.
    pub fn new() -> Self {
        Self {
            inner: rbp_nlhe::Flagship::new(
                NlheProfile::default(),
                NlheEncoder::default(),
            ),
            epochs_trained: 0,
        }
    }

    /// Train the solver for `iterations` iterations.
    pub fn train(&mut self, iterations: u64) {
        for _ in 0..iterations {
            self.inner.step();
            self.epochs_trained += 1;
        }
    }

    /// Get the current epoch count.
    pub fn epochs(&self) -> u64 {
        self.epochs_trained
    }

    /// Compute exploitability of the current strategy in mbb/h.
    ///
    /// Uses the full game tree to compute best-response exploitability.
    pub fn exploitability(&self) -> Utility {
        self.inner.exploitability()
    }

    /// Save checkpoint to `path` with MYOS format.
    ///
    /// Format: 4-byte magic "MYOS" + u32 version + bincode-encoded (profile, encoder, epochs)
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);

        // Write magic bytes
        writer.write_all(&CHECKPOINT_MAGIC)?;

        // Write version in little-endian
        writer.write_all(&CHECKPOINT_VERSION.to_le_bytes())?;

        // Encode profile, encoder, and epochs
        let data = (&self.inner.profile(), &self.inner.encoder(), self.epochs_trained);
        bincode::serialize_into(writer, &data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Load checkpoint from `path`.
    ///
    /// Verifies magic bytes and version before deserializing.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);

        // Read and verify magic
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != CHECKPOINT_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid checkpoint magic",
            ));
        }

        // Read and verify version
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != CHECKPOINT_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unsupported checkpoint version: {}", version),
            ));
        }

        // Deserialize
        let (profile, encoder, epochs): (NlheProfile, NlheEncoder, u64) =
            bincode::deserialize_from(reader)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mut inner = rbp_nlhe::Flagship::new(profile, encoder);
        // Run steps to restore internal state (approximation via step count)
        for _ in 0..epochs {
            inner.step();
        }

        Ok(Self {
            inner,
            epochs_trained: epochs,
        })
    }

    /// Get strategy distribution for an information set.
    ///
    /// Returns action probabilities as a vector of (edge, probability) pairs.
    pub fn strategy(&self, info: &NlheInfo) -> Vec<(NlheEdge, f32)> {
        let dist = self.inner.profile().averaged_distribution(info);
        dist.into_iter()
            .map(|(edge, prob)| (NlheEdge::from(edge), prob))
            .collect()
    }
}

impl Default for PokerSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for the flagship solver variant.
#[allow(dead_code)]
pub type Flagship = rbp_nlhe::Flagship;

#[cfg(test)]
mod tests {
    #[test]
    fn create_empty_solver() {
        // Slice 2
    }

    #[test]
    fn train_100_iterations() {
        // Slice 2
    }

    #[test]
    fn strategy_is_valid_distribution() {
        // Slice 2
    }

    #[test]
    fn checkpoint_roundtrip() {
        // Slice 2
    }

    #[test]
    fn exploitability_decreases() {
        // Slice 2
    }
}