//! Training session management with configurable checkpoint frequency.
//!
//! Provides `TrainingSession` which wraps a `PokerSolver` for batch training
//! with automatic checkpoint saving. This is the primary interface for the
//! miner binary to run extended training runs.

use crate::PokerSolver;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Configuration for a training session.
#[derive(Clone, Debug)]
pub struct TrainingConfig {
    /// Number of iterations per training batch.
    pub batch_size: usize,
    /// Save a checkpoint every `checkpoint_every` iterations.
    pub checkpoint_every: usize,
    /// Directory to save checkpoints.
    pub checkpoint_dir: PathBuf,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            checkpoint_every: 10_000,
            checkpoint_dir: PathBuf::from("checkpoints"),
        }
    }
}

impl TrainingConfig {
    /// Create a new training config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the batch size.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set the checkpoint frequency.
    pub fn with_checkpoint_every(mut self, every: usize) -> Self {
        self.checkpoint_every = every;
        self
    }

    /// Set the checkpoint directory.
    pub fn with_checkpoint_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.checkpoint_dir = dir.into();
        self
    }
}

/// A training session that manages batched iteration with checkpointing.
///
/// # Example
///
/// ```ignore
/// let mut session = TrainingSession::new(TrainingConfig::default())?;
/// session.run(50_000).await?;
/// ```
pub struct TrainingSession {
    solver: PokerSolver,
    config: TrainingConfig,
    total_iterations: usize,
}

impl TrainingSession {
    /// Create a new training session with the given configuration.
    pub fn new(config: TrainingConfig) -> Result<Self> {
        // Ensure checkpoint directory exists
        std::fs::create_dir_all(&config.checkpoint_dir)
            .context("failed to create checkpoint directory")?;

        Ok(Self {
            solver: PokerSolver::new(),
            config,
            total_iterations: 0,
        })
    }

    /// Create a new training session and load from a checkpoint if available.
    pub fn resume(config: TrainingConfig, checkpoint_path: &Path) -> Result<Self> {
        let solver = PokerSolver::load(checkpoint_path)
            .context("failed to load checkpoint")?;

        std::fs::create_dir_all(&config.checkpoint_dir)
            .context("failed to create checkpoint directory")?;

        let total_iterations = solver.epochs();

        Ok(Self {
            solver,
            config,
            total_iterations,
        })
    }

    /// Run training for `iterations` total iterations.
    ///
    /// This performs multiple batches, saving checkpoints at the configured frequency.
    pub fn run(&mut self, target_iterations: usize) -> Result<()> {
        while self.total_iterations < target_iterations {
            let remaining = target_iterations - self.total_iterations;
            let batch = remaining.min(self.config.batch_size);

            self.solver.train(batch);
            self.total_iterations += batch;

            // Check if we should save a checkpoint
            if self.total_iterations % self.config.checkpoint_every == 0 {
                self.save_checkpoint()?;
            }
        }

        Ok(())
    }

    /// Save a checkpoint with the current iteration count in the filename.
    pub fn save_checkpoint(&self) -> Result<PathBuf> {
        let filename = format!("checkpoint_{:08}.myos", self.total_iterations);
        let path = self.config.checkpoint_dir.join(filename);

        self.solver.save(&path)?;
        Ok(path)
    }

    /// Get the current epoch count.
    pub fn epochs(&self) -> usize {
        self.total_iterations
    }

    /// Get the current exploitability estimate.
    pub fn exploitability(&self) -> rbp_core::Utility {
        self.solver.exploitability()
    }

    /// Get a reference to the underlying solver (for querying).
    pub fn solver(&self) -> &PokerSolver {
        &self.solver
    }

    /// Get a mutable reference to the underlying solver (for querying).
    pub fn solver_mut(&mut self) -> &mut PokerSolver {
        &mut self.solver
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_checkpoint_frequency() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = TrainingConfig::default()
            .with_batch_size(10)
            .with_checkpoint_every(25)
            .with_checkpoint_dir(temp_dir.path());

        let mut session = TrainingSession::new(config.clone()).unwrap();

        // Run 50 iterations (should trigger 1 checkpoint at iteration 25)
        session.run(50).unwrap();

        assert_eq!(session.epochs(), 50);

        // Check that a checkpoint was saved
        let checkpoints: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "myos"))
            .collect();

        // Should have at least one checkpoint
        assert!(
            !checkpoints.is_empty(),
            "expected at least one checkpoint file"
        );
    }
}
