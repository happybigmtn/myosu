//! Training session management for batch iteration with checkpointing.
//!
//! `TrainingSession` wraps a `PokerSolver` and provides:
//! - Configurable checkpoint frequency (save every N iterations)
//! - Batch training with progress tracking
//! - Automatic checkpoint loading on startup

use crate::solver::{PokerSolver, PokerSolverError};
use rbp_core::Utility;
use rbp_nlhe::NlheEncoder;
use std::path::Path;
use thiserror::Error;

/// Errors in training session management.
#[derive(Error, Debug)]
pub enum TrainingError {
    #[error("checkpoint save failed: {0}")]
    CheckpointSave(#[source] PokerSolverError),
    #[error("checkpoint load failed: {0}")]
    CheckpointLoad(#[source] PokerSolverError),
    #[error("training failed: {0}")]
    TrainingFailed(#[source] PokerSolverError),
    #[error("exploitability failed: {0}")]
    ExploitabilityFailed(#[source] PokerSolverError),
}

/// Configuration for a training session.
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    /// Save a checkpoint every `checkpoint_frequency` iterations.
    pub checkpoint_frequency: usize,
    /// Directory to save checkpoints.
    pub checkpoint_dir: std::path::PathBuf,
    /// Base name for checkpoint files.
    pub checkpoint_name: String,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            checkpoint_frequency: 100,
            checkpoint_dir: std::env::current_dir().unwrap_or_default(),
            checkpoint_name: "poker_checkpoint".to_string(),
        }
    }
}

impl TrainingConfig {
    /// Creates a new training config with the given checkpoint frequency.
    pub fn with_frequency(checkpoint_frequency: usize) -> Self {
        Self {
            checkpoint_frequency,
            ..Default::default()
        }
    }

    /// Sets the checkpoint directory.
    pub fn with_checkpoint_dir(mut self, dir: impl AsRef<Path>) -> Self {
        self.checkpoint_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Sets the checkpoint base name.
    pub fn with_checkpoint_name(mut self, name: impl Into<String>) -> Self {
        self.checkpoint_name = name.into();
        self
    }
}

/// A training session that manages batch iteration with checkpointing.
///
/// # Example
///
/// ```ignore
/// let config = TrainingConfig::default()
///     .with_frequency(100)
///     .with_checkpoint_dir("/tmp/checkpoints");
///
/// let encoder = /* load abstraction artifact */;
/// let mut session = TrainingSession::new_with_encoder(config, encoder)?;
/// session.train(1000)?;
/// ```
#[derive(Debug)]
pub struct TrainingSession {
    solver: PokerSolver,
    config: TrainingConfig,
    start_epoch: usize,
}

impl TrainingSession {
    /// Creates a new training session.
    ///
    /// If a checkpoint exists at the configured path, it is loaded.
    /// Otherwise, starts fresh with a checkpoint-capable but untrained solver.
    pub fn new(config: TrainingConfig) -> Result<Self, TrainingError> {
        let checkpoint_path = config.checkpoint_path();

        let solver = if checkpoint_path.exists() {
            PokerSolver::load(&checkpoint_path).map_err(TrainingError::CheckpointLoad)?
        } else {
            PokerSolver::new()
        };

        let start_epoch = solver.epochs();

        Ok(Self {
            solver,
            config,
            start_epoch,
        })
    }

    /// Creates a training session backed by a concrete encoder artifact.
    pub fn new_with_encoder(
        config: TrainingConfig,
        encoder: NlheEncoder,
    ) -> Result<Self, TrainingError> {
        let checkpoint_path = config.checkpoint_path();

        let solver = if checkpoint_path.exists() {
            PokerSolver::load_with_encoder(&checkpoint_path, encoder)
                .map_err(TrainingError::CheckpointLoad)?
        } else {
            PokerSolver::with_encoder(encoder).map_err(TrainingError::TrainingFailed)?
        };

        let start_epoch = solver.epochs();

        Ok(Self {
            solver,
            config,
            start_epoch,
        })
    }

    /// Returns the current epoch (iteration count).
    pub fn epochs(&self) -> usize {
        self.solver.epochs()
    }

    /// Returns the number of epochs trained in this session.
    pub fn session_epochs(&self) -> usize {
        self.solver.epochs() - self.start_epoch
    }

    /// Returns a reference to the underlying solver.
    pub fn solver(&self) -> &PokerSolver {
        &self.solver
    }

    /// Returns a mutable reference to the underlying solver.
    pub fn mut_solver(&mut self) -> &mut PokerSolver {
        &mut self.solver
    }

    /// Trains for the specified number of iterations.
    ///
    /// Checkpoints are saved automatically according to the configuration.
    pub fn train(&mut self, iterations: usize) -> Result<(), TrainingError> {
        for _ in 0..iterations {
            self.solver
                .train(1)
                .map_err(TrainingError::TrainingFailed)?;

            // Save checkpoint if needed
            let epoch = self.solver.epochs();
            if epoch % self.config.checkpoint_frequency == 0 {
                self.save_checkpoint()
                    .map_err(TrainingError::CheckpointSave)?;
            }
        }
        Ok(())
    }

    /// Saves a checkpoint immediately.
    pub fn save_checkpoint(&self) -> Result<(), PokerSolverError> {
        let path = self.config.checkpoint_path();
        self.solver.save(&path)
    }

    /// Computes the current exploitability of the strategy.
    pub fn exploitability(&self) -> Result<Utility, TrainingError> {
        self.solver
            .exploitability()
            .map_err(TrainingError::ExploitabilityFailed)
    }
}

impl TrainingConfig {
    /// Returns the full path to the checkpoint file.
    pub fn checkpoint_path(&self) -> std::path::PathBuf {
        self.checkpoint_dir
            .join(format!("{}.myos", self.checkpoint_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn session_checkpoint_frequency() {
        let temp_dir = TempDir::new().unwrap();
        let config = TrainingConfig::with_frequency(50)
            .with_checkpoint_dir(temp_dir.path())
            .with_checkpoint_name("test_checkpoint");

        let mut session = TrainingSession::new(config.clone()).unwrap();
        let checkpoint_path = config.checkpoint_path();

        let error = session.train(120).unwrap_err();

        assert!(matches!(
            error,
            TrainingError::TrainingFailed(PokerSolverError::MissingEncoderAbstractions {
                context: "training"
            })
        ));
        assert_eq!(session.epochs(), 0);
        assert_eq!(session.session_epochs(), 0);
        assert!(
            !checkpoint_path.exists(),
            "failed training should not checkpoint"
        );
    }

    #[test]
    fn new_with_encoder_rejects_empty_abstraction_map() {
        let temp_dir = TempDir::new().unwrap();
        let config = TrainingConfig::with_frequency(10).with_checkpoint_dir(temp_dir.path());

        let error = TrainingSession::new_with_encoder(config, NlheEncoder::default()).unwrap_err();

        assert!(matches!(
            error,
            TrainingError::TrainingFailed(PokerSolverError::MissingEncoderAbstractions {
                context: "encoder validation"
            })
        ));
    }
}
