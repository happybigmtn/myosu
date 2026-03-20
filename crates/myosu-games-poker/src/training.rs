// training.rs: TrainingSession for batch iteration with checkpoint management
//
// Wraps PokerSolver for batch training with configurable checkpoint frequency

use std::path::Path;
use thiserror::Error;

use crate::solver::{PokerSolver, SolverError};

#[derive(Error, Debug)]
pub enum TrainingError {
    #[error("solver error: {0}")]
    SolverError(#[from] SolverError),
    #[error("checkpoint directory error: {0}")]
    CheckpointDirError(String),
    #[error("no checkpoints found")]
    NoCheckpoints,
}

/// Configuration for a training session
#[derive(Clone, Debug)]
pub struct TrainingConfig {
    /// Number of iterations per epoch
    pub iterations_per_epoch: u32,
    /// Save checkpoint every N epochs (0 = no intermediate checkpoints)
    pub checkpoint_every: u32,
    /// Checkpoint directory path
    pub checkpoint_dir: Option<std::path::PathBuf>,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            iterations_per_epoch: 100,
            checkpoint_every: 10,
            checkpoint_dir: None,
        }
    }
}

/// A training session that manages batch iteration and checkpointing
pub struct TrainingSession {
    solver: PokerSolver,
    config: TrainingConfig,
    current_epoch: u32,
}

impl TrainingSession {
    /// Create a new training session
    pub fn new(config: TrainingConfig) -> Result<Self, TrainingError> {
        let solver = PokerSolver::new()?;
        Ok(Self {
            solver,
            config,
            current_epoch: 0,
        })
    }

    /// Resume from a checkpoint
    pub fn resume_from(checkpoint_path: &Path, config: TrainingConfig) -> Result<Self, TrainingError> {
        let solver = PokerSolver::load(checkpoint_path)?;
        Ok(Self {
            solver,
            config,
            current_epoch: 0,
        })
    }

    /// Train for one epoch
    pub fn train_epoch(&mut self) -> Result<(), TrainingError> {
        self.solver.train(self.config.iterations_per_epoch)?;
        self.current_epoch += 1;

        // Auto-checkpoint if configured
        if self.config.checkpoint_every > 0
            && self.current_epoch % self.config.checkpoint_every == 0
        {
            if let Some(ref dir) = self.config.checkpoint_dir {
                let path = dir.join(format!("checkpoint_epoch_{}.bin", self.current_epoch));
                self.save_checkpoint(&path)?;
            }
        }

        Ok(())
    }

    /// Train for N epochs
    pub fn train_epochs(&mut self, num_epochs: u32) -> Result<(), TrainingError> {
        for _ in 0..num_epochs {
            self.train_epoch()?;
        }
        Ok(())
    }

    /// Save a checkpoint manually
    pub fn save_checkpoint(&self, path: &Path) -> Result<(), TrainingError> {
        if let Some(ref dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| TrainingError::CheckpointDirError(e.to_string()))?;
        }
        self.solver.save(path)?;
        Ok(())
    }

    /// Get the current epoch count
    pub fn epoch(&self) -> u32 {
        self.current_epoch
    }

    /// Get the solver for querying
    pub fn solver(&self) -> &PokerSolver {
        &self.solver
    }

    /// Get a mutable reference to the solver
    pub fn solver_mut(&mut self) -> &mut PokerSolver {
        &mut self.solver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_checkpoint_frequency() {
        let temp_dir = std::env::temp_dir().join("poker_training_test");
        std::fs::create_dir_all(&temp_dir).ok();

        let config = TrainingConfig {
            iterations_per_epoch: 10,
            checkpoint_every: 2,
            checkpoint_dir: Some(temp_dir.clone()),
        };

        let mut session = TrainingSession::new(config).unwrap();

        // Train 5 epochs
        session.train_epochs(5).unwrap();
        assert_eq!(session.epoch(), 5);

        // Check that checkpoints exist for epochs 2 and 4
        assert!(temp_dir.join("checkpoint_epoch_2.bin").exists());
        assert!(temp_dir.join("checkpoint_epoch_4.bin").exists());

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
