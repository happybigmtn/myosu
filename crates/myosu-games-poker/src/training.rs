//! Training session management for batch iteration with checkpointing.
//!
//! `TrainingSession` wraps `PokerSolver` and provides batch training with
//! configurable checkpoint frequency. Checkpoints are saved to disk and
//! can be used to resume training or serve queries.

use super::PokerSolver;
use thiserror::Error;

/// Errors from training session operations.
#[derive(Error, Debug)]
pub enum TrainingError {
    #[error("checkpoint save failed")]
    Save,

    #[error("checkpoint load failed")]
    Load,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// A training session that manages batch iteration and checkpointing.
///
/// # Checkpoint Frequency
///
/// The session saves a checkpoint every `checkpoint_every` iterations.
/// Set to `0` to disable checkpointing.
pub struct TrainingSession {
    solver: PokerSolver,
    checkpoint_every: usize,
    checkpoint_dir: Option<std::path::PathBuf>,
}

impl TrainingSession {
    /// Create a new training session.
    ///
    /// - `checkpoint_every`: save checkpoint every N iterations (0 = disable)
    /// - `checkpoint_dir`: directory for checkpoint files (created if missing)
    pub fn new(checkpoint_every: usize, checkpoint_dir: Option<std::path::PathBuf>) -> Self {
        Self {
            solver: PokerSolver::new(),
            checkpoint_every,
            checkpoint_dir,
        }
    }

    /// Create from an existing solver.
    pub fn from_solver(
        solver: PokerSolver,
        checkpoint_every: usize,
        checkpoint_dir: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            solver,
            checkpoint_every,
            checkpoint_dir,
        }
    }

    /// Run `iterations` training iterations.
    ///
    /// Saves checkpoints at configured frequency.
    pub fn train(&mut self, iterations: usize) -> Result<(), TrainingError> {
        for i in 0..iterations {
            self.solver.train(1);

            if self.checkpoint_every > 0 && (i + 1) % self.checkpoint_every == 0 {
                self.save_checkpoint()?;
            }
        }
        Ok(())
    }

    /// Save the current state to a checkpoint file.
    pub fn save_checkpoint(&self) -> Result<(), TrainingError> {
        let Some(dir) = &self.checkpoint_dir else {
            return Ok(()); // checkpointing disabled
        };

        std::fs::create_dir_all(dir)?;

        let epoch = self.solver.epochs();
        let filename = format!("checkpoint_{:08}.myos", epoch);
        let path = dir.join(filename);

        self.solver.save(&path).map_err(|_| TrainingError::Save)?;
        Ok(())
    }

    /// Load the most recent checkpoint from the checkpoint directory.
    pub fn load_latest_checkpoint(&mut self) -> Result<(), TrainingError> {
        let Some(dir) = &self.checkpoint_dir else {
            return Err(TrainingError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no checkpoint directory configured",
            )));
        };

        let entries = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "myos"));

        let latest = entries
            .filter_map(|e| {
                e.path()
                    .file_stem()?
                    .to_str()?
                    .strip_prefix("checkpoint_")
                    .and_then(|s| s.parse::<usize>().ok())
                    .map(|epoch| (epoch, e.path()))
            })
            .max_by_key(|(epoch, _)| *epoch);

        match latest {
            Some((_epoch, path)) => {
                self.solver = PokerSolver::load(&path).map_err(|_| TrainingError::Load)?;
                Ok(())
            }
            None => Err(TrainingError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no checkpoint files found",
            ))),
        }
    }

    /// Get the current solver (shared reference).
    pub fn solver(&self) -> &PokerSolver {
        &self.solver
    }

    /// Get the current solver (mutable reference).
    pub fn solver_mut(&mut self) -> &mut PokerSolver {
        &mut self.solver
    }

    /// Get the current epoch count.
    pub fn epochs(&self) -> usize {
        self.solver.epochs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn session_checkpoint_frequency() {
        let temp_dir = TempDir::new().unwrap();

        let mut session = TrainingSession::new(
            /* checkpoint_every = */ 10,
            Some(temp_dir.path().to_path_buf()),
        );

        // Train 25 iterations — should produce checkpoints at epochs 10 and 20
        session.train(25).unwrap();

        assert_eq!(session.epochs(), 25);

        // Check that checkpoint files exist
        let entries: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "myos"))
            .collect();

        // Should have checkpoints at epoch 10 and 20 (not 25 since it's not a multiple)
        assert!(
            entries.len() >= 2,
            "expected at least 2 checkpoints, got {}",
            entries.len()
        );
    }

    #[test]
    fn session_resume_from_checkpoint() {
        let temp_dir = TempDir::new().unwrap();

        let mut session = TrainingSession::new(
            /* checkpoint_every = */ 10,
            Some(temp_dir.path().to_path_buf()),
        );

        // Train 15 iterations
        session.train(15).unwrap();
        assert_eq!(session.epochs(), 15);

        // Create new session and load from checkpoint
        let mut session2 = TrainingSession::new(
            /* checkpoint_every = */ 10,
            Some(temp_dir.path().to_path_buf()),
        );
        session2.load_latest_checkpoint().unwrap();

        // Should have resumed from the latest checkpoint
        assert_eq!(session2.epochs(), 10); // last checkpoint was at epoch 10
    }
}
