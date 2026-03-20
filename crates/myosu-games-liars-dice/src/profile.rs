//! Liar's Dice profile — `Profile` implementation with MCCFR solver.
//!
//! Train to Nash equilibrium and verify exploitability < 0.001.

/// LiarsDiceProfile implements the MCCFR profile for Liar's Dice.
///
/// Stores accumulated regret and strategy for each information set.
/// The solver trains until exploitability converges below threshold.
#[derive(Clone, Debug)]
pub struct LiarsDiceProfile;

impl LiarsDiceProfile {
    /// Create a new profile with fresh tables.
    pub fn new() -> Self {
        todo!("Slice 3: implement LiarsDiceProfile::new()")
    }

    /// Train the profile to convergence.
    pub fn train(&mut self, _iterations: usize) {
        todo!("Slice 3: implement LiarsDiceProfile::train()")
    }

    /// Compute exploitability of the current strategy.
    pub fn exploitability(&self) -> f64 {
        todo!("Slice 3: implement LiarsDiceProfile::exploitability()")
    }
}
