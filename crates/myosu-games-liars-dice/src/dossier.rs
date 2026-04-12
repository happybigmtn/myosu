//! Promotion dossier types for exact Liar's Dice checkpoint benchmarks.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::solver::{LiarsDiceSolver, LiarsDiceSolverError};

const CHECKPOINT_FORMAT: &str = "myos-v1-bincode";
const BENCHMARK_METHOD: &str = "liars-dice-exact-exploitability-v1";
const BENCHMARK_METRIC: &str = "exact_exploitability";

/// Exact exploitability evidence attached to a Liar's Dice checkpoint dossier.
///
/// These fields intentionally mirror `CanonicalPolicyBenchmarkSummary` so the
/// promotion pipeline can attach the benchmark without lossy translation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiarsDiceBenchmarkDossier {
    pub benchmark_id: String,
    pub benchmark_method: String,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold: f64,
    pub passing: bool,
}

impl LiarsDiceBenchmarkDossier {
    pub fn exact_exploitability(
        benchmark_id: impl Into<String>,
        metric_value: f64,
        threshold: f64,
    ) -> Self {
        let passing =
            metric_value.is_finite() && threshold.is_finite() && metric_value <= threshold;

        Self {
            benchmark_id: benchmark_id.into(),
            benchmark_method: BENCHMARK_METHOD.to_string(),
            metric_name: BENCHMARK_METRIC.to_string(),
            metric_value,
            threshold,
            passing,
        }
    }
}

/// Hash-pinned Liar's Dice checkpoint and exact benchmark evidence.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LiarsDiceArtifactDossier {
    pub checkpoint_hash: String,
    pub checkpoint_format: String,
    pub solver_family: String,
    pub tree_count: usize,
    pub epochs: usize,
    pub exploitability: f64,
    pub threshold: f64,
    pub passing: bool,
    pub benchmark_summary: LiarsDiceBenchmarkDossier,
    pub provenance_chain: Vec<String>,
}

impl LiarsDiceArtifactDossier {
    pub fn from_solver<const N: usize>(
        solver: &LiarsDiceSolver<N>,
        threshold: f64,
    ) -> Result<Self, LiarsDiceDossierError> {
        let checkpoint_bytes = solver.checkpoint_bytes()?;

        Ok(Self::from_solver_and_checkpoint_bytes(
            solver,
            &checkpoint_bytes,
            threshold,
        ))
    }

    pub fn from_checkpoint_bytes<const N: usize>(
        bytes: &[u8],
        threshold: f64,
    ) -> Result<Self, LiarsDiceDossierError> {
        let solver = LiarsDiceSolver::<N>::from_checkpoint_bytes(bytes)?;

        Ok(Self::from_solver_and_checkpoint_bytes(
            &solver, bytes, threshold,
        ))
    }

    pub fn from_checkpoint_file<const N: usize>(
        path: impl AsRef<Path>,
        threshold: f64,
    ) -> Result<Self, LiarsDiceDossierError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| LiarsDiceDossierError::Read {
            path: path.display().to_string(),
            source,
        })?;

        Self::from_checkpoint_bytes::<N>(&bytes, threshold)
    }

    fn from_solver_and_checkpoint_bytes<const N: usize>(
        solver: &LiarsDiceSolver<N>,
        checkpoint_bytes: &[u8],
        threshold: f64,
    ) -> Self {
        let exploitability = f64::from(solver.exact_exploitability());
        let benchmark_summary = LiarsDiceBenchmarkDossier::exact_exploitability(
            format!("liars-dice-exact-n{N}-epochs{}", solver.epochs()),
            exploitability,
            threshold,
        );

        Self {
            checkpoint_hash: sha256_hex(checkpoint_bytes),
            checkpoint_format: CHECKPOINT_FORMAT.to_string(),
            solver_family: "liars-dice-cfr".to_string(),
            tree_count: N,
            epochs: solver.epochs(),
            exploitability,
            threshold,
            passing: benchmark_summary.passing,
            benchmark_summary,
            provenance_chain: vec![
                "checkpoint_bytes_serialized".to_string(),
                "checkpoint_sha256_computed".to_string(),
                "exact_exploitability_computed".to_string(),
                "threshold_evaluated".to_string(),
            ],
        }
    }
}

/// Errors returned when reading or writing Liar's Dice promotion dossiers.
#[derive(Debug, Error)]
pub enum LiarsDiceDossierError {
    #[error("{0}")]
    Solver(#[from] LiarsDiceSolverError),
    #[error("failed to read `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write `{path}`: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse dossier `{path}`: {source}")]
    DossierParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialize dossier `{path}`: {source}")]
    DossierSerialize {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

/// Read a serialized Liar's Dice artifact dossier from disk.
pub fn read_liars_dice_artifact_dossier(
    path: impl AsRef<Path>,
) -> Result<LiarsDiceArtifactDossier, LiarsDiceDossierError> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| LiarsDiceDossierError::Read {
        path: path.display().to_string(),
        source,
    })?;

    serde_json::from_slice(&bytes).map_err(|source| LiarsDiceDossierError::DossierParse {
        path: path.display().to_string(),
        source,
    })
}

/// Write a serialized Liar's Dice artifact dossier to disk.
pub fn write_liars_dice_artifact_dossier(
    path: impl AsRef<Path>,
    dossier: &LiarsDiceArtifactDossier,
) -> Result<(), LiarsDiceDossierError> {
    let path = path.as_ref();
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| LiarsDiceDossierError::Write {
            path: parent.display().to_string(),
            source,
        })?;
    }

    let bytes = serde_json::to_vec_pretty(dossier).map_err(|source| {
        LiarsDiceDossierError::DossierSerialize {
            path: path.display().to_string(),
            source,
        }
    })?;
    fs::write(path, bytes).map_err(|source| LiarsDiceDossierError::Write {
        path: path.display().to_string(),
        source,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{
        LiarsDiceArtifactDossier, LiarsDiceSolver, read_liars_dice_artifact_dossier,
        write_liars_dice_artifact_dossier,
    };

    const SOLVER_TREES: usize = 1 << 10;
    const PROMOTION_THRESHOLD: f64 = 0.70;

    #[test]
    fn zero_iteration_dossier_fails_exact_exploitability_threshold() {
        let solver = LiarsDiceSolver::<SOLVER_TREES>::new();
        let dossier = LiarsDiceArtifactDossier::from_solver(&solver, PROMOTION_THRESHOLD)
            .expect("zero-iteration dossier should build");

        assert_eq!(dossier.checkpoint_hash.len(), 64);
        assert_eq!(dossier.tree_count, SOLVER_TREES);
        assert_eq!(dossier.epochs, 0);
        assert_eq!(dossier.threshold, PROMOTION_THRESHOLD);
        assert!(dossier.exploitability > dossier.threshold);
        assert!(!dossier.passing);
        assert_eq!(
            dossier.benchmark_summary.metric_name,
            "exact_exploitability"
        );
        assert_eq!(
            serde_json::to_value(&dossier.benchmark_summary)
                .expect("benchmark summary should serialize")["benchmark_id"],
            dossier.benchmark_summary.benchmark_id
        );
    }

    #[test]
    fn trained_dossier_passes_exact_exploitability_threshold() {
        let mut solver = LiarsDiceSolver::<SOLVER_TREES>::new();
        solver
            .train(512)
            .expect("promotion-grade training fixture should run");

        let dossier = LiarsDiceArtifactDossier::from_solver(&solver, PROMOTION_THRESHOLD)
            .expect("trained dossier should build");

        assert_eq!(dossier.tree_count, SOLVER_TREES);
        assert!(dossier.epochs > 0);
        assert!(dossier.exploitability <= dossier.threshold);
        assert!(dossier.passing);
    }

    #[test]
    fn dossier_json_roundtrips_for_solver_promotion_output() {
        let output_root = unique_output_dir();
        let dossier_path = output_root
            .join("outputs")
            .join("solver-promotion")
            .join("liars-dice")
            .join("artifact-dossier.json");
        let solver = LiarsDiceSolver::<SOLVER_TREES>::new();
        let dossier = LiarsDiceArtifactDossier::from_solver(&solver, PROMOTION_THRESHOLD)
            .expect("dossier should build");

        write_liars_dice_artifact_dossier(&dossier_path, &dossier)
            .expect("dossier should write to promotion output path");
        let roundtrip = read_liars_dice_artifact_dossier(&dossier_path)
            .expect("dossier should read from promotion output path");
        let json = fs::read_to_string(&dossier_path).expect("dossier JSON should be readable");

        assert_eq!(roundtrip, dossier);
        assert!(json.contains("\"checkpoint_hash\""));
        assert!(json.contains("\"exact_exploitability\""));

        fs::remove_dir_all(output_root).expect("temporary promotion output should clean up");
    }

    fn unique_output_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("myosu-liars-dice-dossier-{nanos}"))
    }
}
