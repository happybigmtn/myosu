//! PokerSolver: wrapper around `rbp_nlhe::Flagship` with checkpoint support.
//!
//! ## Checkpoint Format
//!
//! File layout (all little-endian):
//! - Bytes 0..4:   Magic `MYOS` (0x4D594F53)
//! - Bytes 4..8:   Version u32
//! - Bytes 8..end: bincode-serialized `NlheProfile`

use anyhow::{Context, Result, ensure};
use rbp_core::Probability;
use rbp_mccfr::{Profile, Solver};
use rbp_nlhe::{Flagship, NlheEncoder, NlheInfo, NlheProfile};
use std::fs;
use std::path::Path;

/// Magic bytes for MYOS checkpoint files.
const CHECKPOINT_MAGIC: u32 = 0x4D594F53;
/// Current checkpoint format version.
const CHECKPOINT_VERSION: u32 = 1;

/// Wrapper around `rbp_nlhe::Flagship` exposing the public API needed by miners and validators.
pub struct PokerSolver {
    inner: Flagship,
}

impl PokerSolver {
    /// Creates a new solver with default encoder and profile.
    pub fn new() -> Self {
        Self {
            inner: Flagship::new(NlheProfile::default(), NlheEncoder::default()),
        }
    }

    /// Creates a solver from a saved checkpoint file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let data = fs::read(path.as_ref()).context("failed to read checkpoint")?;
        Self::load_bytes(&data).context("failed to load checkpoint")
    }

    /// Deserializes a checkpoint from raw bytes.
    pub fn load_bytes(data: &[u8]) -> Result<Self> {
        ensure!(data.len() >= 8, "checkpoint data too short");
        let magic = u32::from_le_bytes(data[..4].try_into().unwrap());
        ensure!(
            magic == CHECKPOINT_MAGIC,
            "invalid checkpoint magic: {:x}",
            magic
        );
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        ensure!(
            version == CHECKPOINT_VERSION,
            "unsupported checkpoint version: {}",
            version
        );
        let profile: NlheProfile = serde_json::from_slice(&data[8..])
            .context("failed to decode profile")?;
        Ok(Self {
            inner: Flagship::new(profile, NlheEncoder::default()),
        })
    }

    /// Saves the solver state to a checkpoint file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let profile_bytes = serde_json::to_vec(&self.inner.profile())
            .context("failed to serialize profile")?;
        let mut file = Vec::with_capacity(8 + profile_bytes.len());
        file.extend_from_slice(&CHECKPOINT_MAGIC.to_le_bytes());
        file.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
        file.extend_from_slice(&profile_bytes);
        fs::write(path.as_ref(), &file).context("failed to write checkpoint file")?;
        Ok(())
    }

    /// Runs `iterations` training steps.
    pub fn train(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.inner.step();
        }
    }

    /// Returns the current epoch (iteration count).
    pub fn epochs(&self) -> usize {
        self.inner.profile().epochs()
    }

    /// Returns the action probability distribution for the given info set.
    ///
    /// Uses the averaged (historical) strategy rather than the current iterated policy.
    pub fn strategy(&self, info: &NlheInfo) -> Vec<(rbp_nlhe::NlheEdge, Probability)> {
        self.inner.profile().averaged_distribution(info)
    }

    /// Returns the exploitability of the current strategy in milli-big-blinds per hand.
    pub fn exploitability(&self) -> f32 {
        // Solver::exploitability returns absolute utility; convert to mbb/h
        // (1 big blind = 1.0 utility units in robopoker)
        let exploit = self.inner.exploitability();
        // Multiply by 1000 to convert to milli-big-blinds, divide by 2 for average per player
        (exploit * 500.0).abs()
    }

    /// Returns an iterator over all info sets in the trained profile.
    pub fn info_sets(&self) -> impl Iterator<Item = &NlheInfo> {
        self.inner.profile().encounters.keys()
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
        // Get a sample info set from the profile
        let info = solver.inner.profile().encounters.keys().next().copied();
        if let Some(info) = info {
            let strat = solver.strategy(&info);
            let sum: f32 = strat.iter().map(|(_, p)| p).sum();
            assert!((sum - 1.0).abs() < 0.01, "strategy probabilities sum to {}", sum);
        }
    }

    #[test]
    fn checkpoint_roundtrip() {
        let mut solver = PokerSolver::new();
        solver.train(50);
        let epochs_before = solver.epochs();

        let path = "/tmp/test_poker_checkpoint.bin";
        solver.save(path).unwrap();
        let loaded = PokerSolver::load(path).unwrap();
        assert_eq!(loaded.epochs(), epochs_before);
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn exploitability_decreases() {
        let mut solver = PokerSolver::new();
        solver.train(200);
        let exploit_after_train = solver.exploitability();

        // Create a fresh random solver
        let random_solver = PokerSolver::new();
        let random_exploit = random_solver.exploitability();

        // Trained solver should be less exploitable than random
        assert!(
            exploit_after_train < random_exploit,
            "trained {} < random {}",
            exploit_after_train,
            random_exploit
        );
    }
}
