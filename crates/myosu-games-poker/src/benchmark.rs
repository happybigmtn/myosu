use std::path::Path;

use thiserror::Error;

use crate::ArtifactCodecError;
use crate::PokerSolver;
use crate::PokerSolverError;
use crate::RbpNlheEncoder;
use crate::decode_encoder;
use crate::encode_encoder;
use crate::load_encoder_dir;

/// One exploitability sample in a poker training benchmark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PokerBenchmarkPoint {
    pub iterations: usize,
    pub exploitability: f32,
}

/// Errors returned while building poker exploitability benchmark points.
#[derive(Debug, Error)]
pub enum PokerBenchmarkError {
    #[error("failed to load encoder directory `{path}`: {source}")]
    LoadEncoder {
        path: String,
        #[source]
        source: ArtifactCodecError,
    },
    #[error("failed to clone encoder artifact bytes for benchmark reuse: {source}")]
    CloneEncoder {
        #[source]
        source: ArtifactCodecError,
    },
    #[error(transparent)]
    Solver(#[from] PokerSolverError),
}

/// Build exploitability samples for the requested training ladder.
pub fn benchmark_points_from_encoder_dir(
    directory: impl AsRef<Path>,
    iterations: &[usize],
) -> Result<Vec<PokerBenchmarkPoint>, PokerBenchmarkError> {
    let directory = directory.as_ref();
    let encoder =
        load_encoder_dir(directory).map_err(|source| PokerBenchmarkError::LoadEncoder {
            path: directory.display().to_string(),
            source,
        })?;

    benchmark_points_from_encoder(encoder, iterations)
}

fn benchmark_points_from_encoder(
    encoder: RbpNlheEncoder,
    iterations: &[usize],
) -> Result<Vec<PokerBenchmarkPoint>, PokerBenchmarkError> {
    let encoder_bytes =
        encode_encoder(&encoder).map_err(|source| PokerBenchmarkError::CloneEncoder { source })?;
    let mut points = Vec::with_capacity(iterations.len());

    for iteration_count in iterations {
        let encoder = decode_encoder(&encoder_bytes)
            .map_err(|source| PokerBenchmarkError::CloneEncoder { source })?;
        let mut solver = PokerSolver::new(encoder);
        solver.train(*iteration_count)?;
        points.push(PokerBenchmarkPoint {
            iterations: *iteration_count,
            exploitability: solver.exploitability()?,
        });
    }

    Ok(points)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rbp_cards::Isomorphism;
    use rbp_cards::Observation;
    use rbp_gameplay::Abstraction;

    use crate::NlheAbstractionStreet;
    use crate::write_encoder_dir;

    use super::*;

    #[test]
    fn benchmark_reports_sparse_encoder_failure_cleanly() {
        let directory = unique_artifact_dir();
        let observation = Observation::try_from("AcKh").expect("observation should parse");
        let streets = BTreeMap::from([(
            NlheAbstractionStreet::Preflop,
            BTreeMap::from([(Isomorphism::from(observation), Abstraction::from(42_i16))]),
        )]);
        write_encoder_dir(&directory, streets).expect("artifact dir should write");

        let error = benchmark_points_from_encoder_dir(&directory, &[1])
            .expect_err("sparse encoder should fail cleanly");
        let _ = fs::remove_dir_all(&directory);

        match error {
            PokerBenchmarkError::Solver(PokerSolverError::UpstreamPanic { operation, message }) => {
                assert_eq!(operation, "solver step");
                assert!(message.contains("isomorphism not found"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    fn unique_artifact_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("myosu-poker-benchmark-{nanos}"))
    }
}
