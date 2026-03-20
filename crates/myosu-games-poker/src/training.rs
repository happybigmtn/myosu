//! Training session management with configurable checkpoint frequency.
//!
//! Wraps `PokerSolver` for batch training, epoch tracking, and periodic saves.

use crate::solver::PokerSolver;
use std::path::Path;

/// Training session with periodic checkpointing.
pub struct TrainingSession {
    solver: PokerSolver,
    checkpoint_frequency: u64,
}

impl TrainingSession {
    /// Create a new training session.
    pub fn new(solver: PokerSolver, checkpoint_frequency: u64) -> Self {
        Self {
            solver,
            checkpoint_frequency,
        }
    }

    /// Run training for `iterations`, checkpointing periodically.
    pub fn train(&mut self, iterations: u64, checkpoint_dir: &Path) -> std::io::Result<u64> {
        unimplemented!("Slice 6")
    }

    /// Get the current epoch count.
    pub fn epochs(&self) -> u64 {
        self.solver.epochs()
    }

    /// Get the checkpoint frequency.
    pub fn checkpoint_frequency(&self) -> u64 {
        self.checkpoint_frequency
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn session_checkpoint_frequency() {
        // Slice 6
    }
}
