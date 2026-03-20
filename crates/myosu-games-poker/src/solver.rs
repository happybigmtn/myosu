//! PokerSolver: MCCFR solver wrapper for NLHE.
//!
//! Wraps robopoker's `NlheSolver` with:
//! - Checkpoint save/load via `MYOS` magic + version + bincode
//! - Training via `train(iterations)`
//! - Strategy queries via `strategy(&NlheInfo)`
//! - Exploitability via `exploitability()`
//! - Epoch tracking via `epochs()`

use rbp_core::Utility;
use rbp_mccfr::{
    Solver, Profile, CfrGame, Encoder,
    PluribusRegret, LinearWeight, PluribusSampling, VanillaSampling,
    TreeBuilder,
};
use rbp_nlhe::{NlheEncoder, NlheGame, NlheInfo, NlheProfile, NlheSolver};
use std::path::Path;
use std::io::{Read, Write};
use thiserror::Error;

// MYOS checkpoint magic bytes
const CHECKPOINT_MAGIC: &[u8; 4] = b"MYOS";
const CHECKPOINT_VERSION: u32 = 1;

/// Errors that can occur during solver operations.
#[derive(Error, Debug)]
pub enum SolverError {
    #[error("checkpoint format version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u32, got: u32 },
    #[error("checkpoint has invalid magic bytes: expected MYOS, got {0:?}")]
    InvalidMagic([u8; 4]),
    #[error("checkpoint file is too short")]
    Truncated,
    #[error("checkpoint IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("bincode decode error: {0}")]
    Decode(#[from] bincode::Error),
    #[error("encoder is not initialized (is default)")]
    UninitializedEncoder,
    #[error("exploitability computation returned NaN")]
    NaNExploitability,
}

/// Production NLHE solver using Pluribus configuration.
pub type PokerSolver = NlheSolver<PluribusRegret, LinearWeight, PluribusSampling>;

/// Debug NLHE solver using simpler configuration for faster iteration.
///
/// Uses SummedRegret + ConstantWeight + ExternalSampling.
/// Suitable for CI and development; production uses `PokerSolver`.
pub type DebugSolver = NlheSolver<rbp_mccfr::SummedRegret, rbp_mccfr::ConstantWeight, VanillaSampling>;

/// Number of training iterations completed.
pub fn epochs(solver: &PokerSolver) -> usize {
    solver.profile().epochs()
}

/// Create a new empty solver with default encoder and profile.
pub fn create_empty_solver() -> PokerSolver {
    PokerSolver::new(NlheProfile::default(), NlheEncoder::default())
}

/// Train the solver for `iterations` steps.
pub fn train(solver: &mut PokerSolver, iterations: usize) {
    for _ in 0..iterations {
        solver.step();
    }
}

/// Get action probabilities for a given information set.
///
/// Returns a vector of (edge, probability) pairs from the averaged strategy.
pub fn strategy(solver: &PokerSolver, info: &NlheInfo) -> Vec<(rbp_nlhe::NlheEdge, rbp_core::Probability)> {
    solver.profile().averaged_distribution(info)
}

/// Check if a strategy distribution is valid (sums to ~1.0).
pub fn is_valid_distribution(dist: &[(rbp_nlhe::NlheEdge, rbp_core::Probability)]) -> bool {
    if dist.is_empty() {
        return true;
    }
    let sum: rbp_core::Probability = dist.iter().map(|(_, p)| p).sum();
    (sum - 1.0).abs() < 0.001
}

/// Compute exploitability of the current strategy in mbb/h (milli-big-blinds per hand).
///
/// Returns NaN if the strategy is untrained (no encounters recorded).
pub fn exploitability(solver: &PokerSolver) -> Result<Utility, SolverError> {
    let tree = TreeBuilder::<_, _, _, _, _, _, VanillaSampling>::new(
        solver.encoder(),
        solver.profile(),
        NlheGame::root(),
    )
    .build();

    let exploit = solver.profile().exploitability(tree);
    if exploit.is_nan() {
        return Err(SolverError::NaNExploitability);
    }
    Ok(exploit)
}

/// Save solver state to a checkpoint file.
///
/// Format: [MYOS][version (u32)][encoder (bincode)][profile (bincode)]
pub fn save(solver: &PokerSolver, path: &Path) -> Result<(), SolverError> {
    use std::fs::File;
    use std::io::BufWriter;

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Write magic
    writer.write_all(CHECKPOINT_MAGIC)?;
    // Write version
    writer.write_all(&CHECKPOINT_VERSION.to_le_bytes())?;

    // Write encoder
    bincode::serialize_into(&mut writer, solver.encoder())?;
    // Write profile
    bincode::serialize_into(&mut writer, solver.profile())?;

    writer.flush()?;
    Ok(())
}

/// Load solver state from a checkpoint file.
pub fn load(path: &Path) -> Result<PokerSolver, SolverError> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read and verify magic
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != CHECKPOINT_MAGIC {
        return Err(SolverError::InvalidMagic(magic));
    }

    // Read and verify version
    let mut version_bytes = [0u8; 4];
    reader.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);
    if version != CHECKPOINT_VERSION {
        return Err(SolverError::VersionMismatch {
            expected: CHECKPOINT_VERSION,
            got: version,
        });
    }

    // Read encoder
    let encoder: NlheEncoder = bincode::deserialize_from(&mut reader)?;

    // Read profile
    let profile: NlheProfile = bincode::deserialize_from(&mut reader)?;

    Ok(PokerSolver::new(profile, encoder))
}

#[cfg(test)]
mod tests {
    use rbp_mccfr::{Encoder, Solver, CfrGame, Profile};

    #[test]
    fn create_empty_solver() {
        let solver = super::create_empty_solver();
        assert_eq!(solver.profile().epochs(), 0);
    }

    #[test]
    fn train_100_iterations() {
        let mut solver = super::create_empty_solver();
        super::train(&mut solver, 100);
        assert_eq!(solver.profile().epochs(), 100);
    }

    #[test]
    fn strategy_is_valid_distribution() {
        let solver = super::create_empty_solver();
        // Get strategy at root info
        let root_info = solver.encoder().seed(&super::NlheGame::root());
        let dist = super::strategy(&solver, &root_info);
        assert!(super::is_valid_distribution(&dist));
    }

    #[test]
    fn checkpoint_roundtrip() {
        let mut solver = super::create_empty_solver();
        super::train(&mut solver, 50);
        let epochs_before = solver.profile().epochs();

        // Save
        let temp_path = std::env::temp_dir().join("poker_solver_test.checkpoint");
        super::save(&solver, &temp_path).unwrap();

        // Load
        let loaded = super::load(&temp_path).unwrap();
        assert_eq!(loaded.profile().epochs(), epochs_before);

        // Clean up
        std::fs::remove_file(temp_path).ok();
    }

    #[test]
    fn exploitability_decreases() {
        let mut solver = super::create_empty_solver();

        // Random strategy has some exploitability
        let exploit_before = super::exploitability(&solver).unwrap_or(999.0);

        // Train
        super::train(&mut solver, 200);

        // Trained strategy should have lower or equal exploitability
        let exploit_after = super::exploitability(&solver).unwrap_or(999.0);
        assert!(
            exploit_after <= exploit_before + 0.001,
            "exploitability should decrease: {} -> {}",
            exploit_before,
            exploit_after
        );
    }
}