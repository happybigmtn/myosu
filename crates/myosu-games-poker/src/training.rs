//! Training session management for batch iteration with checkpointing.
//!
//! Provides a `TrainingSession` that wraps a `PokerSolver` and manages
//! batch training runs with configurable checkpoint frequency.

use crate::solver::{self, PokerSolver, SolverError};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TrainingError {
    #[error("checkpoint save failed")]
    CheckpointSave,
    #[error("checkpoint load failed: {0}")]
    CheckpointLoad(SolverError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A training session that manages batch training with periodic checkpointing.
pub struct TrainingSession {
    solver: PokerSolver,
    checkpoint_path: Option<std::path::PathBuf>,
    checkpoint_frequency: usize,
}

impl TrainingSession {
    /// Create a new training session.
    ///
    /// - `solver` — The poker solver to train
    /// - `checkpoint_path` — Path to save checkpoints (if None, no checkpoints are saved)
    /// - `checkpoint_frequency` — Save checkpoint every N iterations
    pub fn new(
        solver: PokerSolver,
        checkpoint_path: Option<std::path::PathBuf>,
        checkpoint_frequency: usize,
    ) -> Self {
        Self {
            solver,
            checkpoint_path,
            checkpoint_frequency,
        }
    }

    /// Create a new session from scratch.
    pub fn new_from_scratch(checkpoint_path: Option<std::path::PathBuf>, checkpoint_frequency: usize) -> Self {
        Self::new(solver::create_empty_solver(), checkpoint_path, checkpoint_frequency)
    }

    /// Resume a session from a checkpoint file.
    pub fn from_checkpoint(path: &Path) -> Result<Self, TrainingError> {
        let solver = solver::load(path).map_err(TrainingError::CheckpointLoad)?;
        Ok(Self {
            solver,
            checkpoint_path: Some(path.to_path_buf()),
            checkpoint_frequency: 100,
        })
    }

    /// Get the current solver.
    pub fn solver(&self) -> &PokerSolver {
        &self.solver
    }

    /// Get mutable reference to the solver.
    pub fn solver_mut(&mut self) -> &mut PokerSolver {
        &mut self.solver
    }

    /// Get the current epoch count.
    pub fn epochs(&self) -> usize {
        solver::epochs(&self.solver)
    }

    /// Train for the given number of iterations.
    ///
    /// Saves a checkpoint every `checkpoint_frequency` iterations.
    pub fn train(&mut self, iterations: usize) -> Result<(), TrainingError> {
        for i in 0..iterations {
            solver::train(&mut self.solver, 1);

            // Checkpoint if needed
            if let Some(ref path) = self.checkpoint_path {
                if self.checkpoint_frequency > 0 && (i + 1) % self.checkpoint_frequency == 0 {
                    solver::save(&self.solver, path)
                        .map_err(|_| TrainingError::CheckpointSave)?;
                }
            }
        }
        Ok(())
    }

    /// Save a checkpoint immediately.
    pub fn save_checkpoint(&self) -> Result<(), TrainingError> {
        if let Some(ref path) = self.checkpoint_path {
            solver::save(&self.solver, path)
                .map_err(|_| TrainingError::CheckpointSave)?;
        }
        Ok(())
    }

    /// Get the current exploitability.
    pub fn exploitability(&self) -> Result<rbp_core::Utility, SolverError> {
        solver::exploitability(&self.solver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_checkpoint_frequency() {
        // Create a session with checkpoint every 50 iterations
        let temp_path = std::env::temp_dir().join("poker_training_test.checkpoint");

        let mut session = TrainingSession::new_from_scratch(Some(temp_path.clone()), 50);

        // Train 120 iterations — should save checkpoints at 50 and 100
        session.train(120).unwrap();

        assert_eq!(session.epochs(), 120);

        // Verify checkpoint was saved
        assert!(temp_path.exists());

        // Load from checkpoint and verify epoch count
        let reloaded = TrainingSession::from_checkpoint(&temp_path).unwrap();
        assert_eq!(reloaded.epochs(), 120);

        // Clean up
        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn session_no_checkpoint() {
        // Session without checkpoint path should not fail
        let mut session = TrainingSession::new_from_scratch(None, 50);
        session.train(10).unwrap();
        assert_eq!(session.epochs(), 10);
    }
}