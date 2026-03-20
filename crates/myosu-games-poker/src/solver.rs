//! PokerSolver wrapper around `rbp_nlhe::Flagship`.
//!
//! Implements save/load with `MYOS` checkpoint format, training, and strategy queries.

use std::path::Path;

/// PokerSolver wraps the NLHE MCCFR solver with checkpointing and query support.
///
/// Uses `rbp_nlhe::Flagship` = `NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>`.
pub struct PokerSolver {
    inner: rbp_nlhe::Flagship,
}

impl PokerSolver {
    /// Create a new solver with default configuration.
    ///
    /// Uses Pluribus-regret + LinearWeight + PluribusSampling.
    pub fn new() -> Self {
        Self {
            inner: rbp_nlhe::Flagship::new(
                rbp_nlhe::NlheProfile::default(),
                rbp_nlhe::NlheEncoder::default(),
            ),
        }
    }

    /// Train the solver for `iterations` iterations.
    pub fn train(&mut self, _iterations: u64) {
        unimplemented!("Slice 2")
    }

    /// Get the current epoch count.
    pub fn epochs(&self) -> u64 {
        unimplemented!("Slice 2")
    }

    /// Compute exploitability of the current strategy in mbb/h.
    pub fn exploitability(&self) -> f32 {
        unimplemented!("Slice 2")
    }

    /// Save checkpoint to `path` with MYOS format.
    pub fn save(&self, _path: &Path) -> std::io::Result<()> {
        unimplemented!("Slice 2")
    }

    /// Load checkpoint from `path`.
    pub fn load(_path: &Path) -> std::io::Result<Self> {
        unimplemented!("Slice 2")
    }

    /// Get strategy distribution for an information set.
    pub fn strategy(&self, _info: &rbp_nlhe::NlheInfo) -> Vec<(rbp_nlhe::NlheEdge, f32)> {
        unimplemented!("Slice 2")
    }
}

impl Default for PokerSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for the flagship solver variant.
pub type Flagship = rbp_nlhe::Flagship;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_empty_solver() {
        let _solver = PokerSolver::new();
        // Solver is created successfully with default profile and encoder.
        // Full functionality (epochs, train, strategy) implemented in Slice 2.
    }
}
