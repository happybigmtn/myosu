// solver.rs: PokerSolver wrapper around rbp_nlhe::Flagship
//
// Provides train(), strategy(), exploitability(), epochs(), save(), load()

use std::path::Path;
use thiserror::Error;
use rbp_mccfr::{Profile, Solver};

#[derive(Error, Debug)]
pub enum SolverError {
    #[error("failed to train solver: {0}")]
    TrainingError(String),
    #[error("failed to query strategy: {0}")]
    QueryError(String),
    #[error("checkpoint version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u32, got: u32 },
    #[error("invalid checkpoint magic bytes")]
    InvalidMagic,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("bincode error: {0}")]
    BincodeError(#[from] bincode::Error),
}

/// PokerSolver is a wrapper around rbp_nlhe::Flagship = NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>
pub struct PokerSolver {
    inner: Flagship,
    epochs_trained: usize,
}

pub type Flagship = rbp_nlhe::NlheSolver<
    rbp_mccfr::PluribusRegret,
    rbp_mccfr::LinearWeight,
    rbp_mccfr::PluribusSampling,
>;

impl PokerSolver {
    /// Create a new empty solver with default configuration
    pub fn new() -> Result<Self, SolverError> {
        let profile = rbp_nlhe::NlheProfile::default();
        let encoder = rbp_nlhe::NlheEncoder::default();
        let inner = Flagship::new(profile, encoder);
        Ok(Self {
            inner,
            epochs_trained: 0,
        })
    }

    /// Train the solver for a number of iterations
    ///
    /// Each iteration calls `step()` once which runs a batch of CFR updates.
    pub fn train(&mut self, iterations: u32) -> Result<(), SolverError> {
        for _ in 0..iterations {
            self.inner.step();
        }
        self.epochs_trained += iterations as usize;
        Ok(())
    }

    /// Get the current strategy for a given information set
    ///
    /// Returns a vector of (action, probability) pairs from the averaged distribution.
    pub fn strategy(&self, info: &rbp_nlhe::NlheInfo) -> Vec<(rbp_nlhe::NlheEdge, f64)> {
        self.inner
            .profile()
            .averaged_distribution(info)
            .into_iter()
            .map(|(edge, prob)| (edge, prob as f64))
            .collect()
    }

    /// Compute exploitability of the current strategy in milli-big-blinds per hand (mbb/h)
    ///
    /// Uses the Solver trait's exploitability method which builds a full game tree
    /// and computes the Nash equilibrium distance.
    pub fn exploitability(&self) -> Result<f64, SolverError> {
        let exploit: f32 = self.inner.exploitability();
        // Convert to milli-big-blinds per hand (exploitability is in big-blinds)
        Ok(exploit as f64 * 1000.0)
    }

    /// Get the number of epochs the solver has been trained
    pub fn epochs(&self) -> usize {
        self.epochs_trained
    }

    /// Save the solver to a file with MYOS checkpoint format
    ///
    /// Format: 4-byte magic "MYOS" + u32 version + bincode-encoded profile
    pub fn save(&self, path: &Path) -> Result<(), SolverError> {
        use std::io::Write;

        let mut file = std::fs::File::create(path)?;
        file.write_all(b"MYOS")?;
        let version: u32 = 1;
        file.write_all(&version.to_le_bytes())?;
        let encoded = bincode::serialize(self.inner.profile())
            .map_err(|e| SolverError::BincodeError(e))?;
        file.write_all(&encoded)?;
        Ok(())
    }

    /// Load the solver from a file with MYOS checkpoint format
    pub fn load(path: &Path) -> Result<Self, SolverError> {
        use std::io::Read;

        let mut file = std::fs::File::open(path)?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"MYOS" {
            return Err(SolverError::InvalidMagic);
        }
        let mut version_bytes = [0u8; 4];
        file.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != 1 {
            return Err(SolverError::VersionMismatch {
                expected: 1,
                got: version,
            });
        }
        let mut encoded = Vec::new();
        file.read_to_end(&mut encoded)?;
        let profile: rbp_nlhe::NlheProfile = bincode::deserialize(&encoded)
            .map_err(|e| SolverError::BincodeError(e))?;

        let encoder = rbp_nlhe::NlheEncoder::default();
        let inner = Flagship::new(profile, encoder);

        Ok(Self {
            inner,
            epochs_trained: 0,
        })
    }
}

impl Default for PokerSolver {
    fn default() -> Self {
        Self::new().expect("default solver should always be creatable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rbp_core::Arbitrary;

    #[test]
    fn create_empty_solver() {
        let solver = PokerSolver::new();
        assert!(solver.is_ok());
        let solver = solver.unwrap();
        assert_eq!(solver.epochs(), 0);
    }

    #[test]
    fn train_100_iterations() {
        let mut solver = PokerSolver::new().unwrap();
        solver.train(100).unwrap();
        assert_eq!(solver.epochs(), 100);
    }

    #[test]
    fn strategy_is_valid_distribution() {
        let solver = PokerSolver::new().unwrap();
        // Create a random info set to query
        let info = rbp_nlhe::NlheInfo::random();
        let strategy = solver.strategy(&info);
        // Probabilities should sum to approximately 1.0
        let sum: f64 = strategy.iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 0.001 || sum == 0.0);
    }

    #[test]
    fn checkpoint_roundtrip() {
        let mut solver = PokerSolver::new().unwrap();
        solver.train(50).unwrap();

        let temp_path = std::env::temp_dir().join("poker_test_checkpoint.bin");
        solver.save(&temp_path).unwrap();

        let loaded = PokerSolver::load(&temp_path).unwrap();
        assert_eq!(loaded.epochs(), 0); // Epochs don't persist

        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn exploitability_decreases() {
        let mut solver = PokerSolver::new().unwrap();

        // Random initial strategy has some exploitability
        let initial_exp = solver.exploitability().unwrap();

        // Train for more iterations
        solver.train(200).unwrap();
        let trained_exp = solver.exploitability().unwrap();

        // Trained strategy should have lower or equal exploitability
        assert!(trained_exp <= initial_exp);
    }
}
