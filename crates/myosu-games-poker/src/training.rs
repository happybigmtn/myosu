//! Training session management for batch training with checkpointing.
//!
//! Provides [`TrainingSession`] which wraps [`PokerSolver`] for high-level
//! training loop management with configurable checkpoint frequency.

use crate::solver::PokerSolver;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Configuration for a training session.
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Number of iterations to train per checkpoint.
    pub iterations_per_checkpoint: usize,
    /// Number of checkpoints to keep (0 = keep all).
    pub max_checkpoints: usize,
    /// Directory to save checkpoints in.
    pub checkpoint_dir: PathBuf,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            iterations_per_checkpoint: 1000,
            max_checkpoints: 5,
            checkpoint_dir: PathBuf::from("checkpoints"),
        }
    }
}

impl TrainingConfig {
    /// Sets the checkpoint directory.
    pub fn with_checkpoint_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.checkpoint_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Sets the iterations per checkpoint.
    pub fn with_iterations_per_checkpoint(mut self, n: usize) -> Self {
        self.iterations_per_checkpoint = n;
        self
    }

    /// Sets the maximum number of checkpoints to retain.
    pub fn with_max_checkpoints(mut self, n: usize) -> Self {
        self.max_checkpoints = n;
        self
    }
}

/// A training session managing batch training with periodic checkpointing.
pub struct TrainingSession {
    solver: PokerSolver,
    config: TrainingConfig,
    checkpoint_count: usize,
}

impl TrainingSession {
    /// Creates a new training session.
    pub fn new(solver: PokerSolver, config: TrainingConfig) -> Self {
        Self {
            solver,
            config,
            checkpoint_count: 0,
        }
    }

    /// Creates a new session from an existing checkpoint.
    pub fn from_checkpoint(path: impl AsRef<Path>, config: TrainingConfig) -> Result<Self> {
        let solver = PokerSolver::load(path.as_ref())
            .with_context(|| format!("failed to load checkpoint from {:?}", path.as_ref()))?;
        Ok(Self {
            solver,
            config,
            checkpoint_count: 0,
        })
    }

    /// Runs one batch of training iterations.
    pub fn train_batch(&mut self) {
        self.solver
            .train(self.config.iterations_per_checkpoint);
        self.checkpoint_count += 1;
    }

    /// Saves a checkpoint.
    pub fn save_checkpoint(&self, name: &str) -> Result<PathBuf> {
        let dir = &self.config.checkpoint_dir;
        std::fs::create_dir_all(dir).context("failed to create checkpoint directory")?;
        let path = dir.join(format!("{}_{}.bin", name, self.checkpoint_count));
        self.solver.save(&path)?;
        self.prune_old_checkpoints(name);
        Ok(path)
    }

    /// Returns the current epoch count.
    pub fn epochs(&self) -> usize {
        self.solver.epochs()
    }

    /// Returns a reference to the underlying solver.
    pub fn solver(&self) -> &PokerSolver {
        &self.solver
    }

    /// Prunes old checkpoints, keeping only the most recent `max_checkpoints`.
    fn prune_old_checkpoints(&self, name: &str) {
        if self.config.max_checkpoints == 0 {
            return;
        }
        let dir = &self.config.checkpoint_dir;
        let mut checkpoints: Vec<_> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(&format!("{}_", name)))
                    .unwrap_or(false)
            })
            .collect();
        checkpoints.sort_by_key(|e| e.path());
        while checkpoints.len() > self.config.max_checkpoints {
            if let Some(oldest) = checkpoints.first() {
                let _ = std::fs::remove_file(oldest.path());
                checkpoints.remove(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn session_checkpoint_frequency() {
        let config = TrainingConfig {
            iterations_per_checkpoint: 10,
            max_checkpoints: 3,
            checkpoint_dir: PathBuf::from("/tmp/poker_test_checkpoints"),
        };

        // Clean up any existing checkpoints
        let _ = fs::remove_dir_all(&config.checkpoint_dir);

        let mut session = TrainingSession::new(PokerSolver::new(), config.clone());
        assert_eq!(session.epochs(), 0);

        // Train 3 batches = 30 iterations total
        for i in 0..3 {
            session.train_batch();
            let path = session.save_checkpoint("test").unwrap();
            assert!(
                path.exists(),
                "checkpoint {} should exist",
                i
            );
        }

        assert_eq!(session.epochs(), 30);

        // Should only have 3 checkpoints (max_checkpoints = 3)
        let files: Vec<_> = fs::read_dir(&config.checkpoint_dir)
            .unwrap()
            .map(|e| e.unwrap())
            .filter(|e| e.path().extension().unwrap_or_default() == "bin")
            .collect();
        assert_eq!(files.len(), 3, "should have exactly 3 checkpoints");

        // Clean up
        let _ = fs::remove_dir_all(&config.checkpoint_dir);
    }
}
