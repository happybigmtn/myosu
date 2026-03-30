use std::any::Any;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use bincode::Options;
use myosu_games::{Profile as _, StrategyResponse, Utility};
use rbp_mccfr::{Metrics, Solver as _};
use rbp_nlhe::{NlheEdge, NlheEncoder, NlheInfo, NlheProfile};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::robopoker::{
    NlheBlueprint, NlheFlagshipSolver, NlheInfoKey, NlheStrategyQuery, NlheStrategyResponse,
    recommended_edge,
};

const MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024;
const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOS";
const CHECKPOINT_VERSION: u32 = 1;
const CHECKPOINT_HEADER_LEN: usize = 8;

/// Thin wrapper around robopoker's flagship NLHE solver.
pub struct PokerSolver {
    solver: NlheFlagshipSolver,
}

impl std::fmt::Debug for PokerSolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PokerSolver")
            .field("epochs", &self.epochs())
            .finish()
    }
}

impl PokerSolver {
    /// Create a solver with an empty strategy profile and a supplied encoder.
    pub fn new(encoder: NlheEncoder) -> Self {
        Self::from_parts(NlheProfile::default(), encoder)
    }

    /// Rebuild a solver from explicit profile and encoder parts.
    pub fn from_parts(mut profile: NlheProfile, encoder: NlheEncoder) -> Self {
        profile.metrics = Metrics::with_epoch(profile.iterations);

        Self {
            solver: NlheFlagshipSolver::new(profile, encoder),
        }
    }

    /// Load a solver checkpoint using an already-selected encoder artifact.
    pub fn load(path: impl AsRef<Path>, encoder: NlheEncoder) -> Result<Self, PokerSolverError> {
        let path = path.as_ref();
        let bytes = fs::read(path).map_err(|source| PokerSolverError::Read {
            path: path.display().to_string(),
            source,
        })?;

        Self::from_checkpoint_bytes(&bytes, encoder)
    }

    /// Decode a solver from checkpoint bytes and a separately supplied encoder.
    pub fn from_checkpoint_bytes(
        bytes: &[u8],
        encoder: NlheEncoder,
    ) -> Result<Self, PokerSolverError> {
        if bytes.len() < CHECKPOINT_HEADER_LEN {
            return Err(PokerSolverError::CheckpointTooShort { bytes: bytes.len() });
        }

        let found_magic: [u8; 4] = bytes[..4]
            .try_into()
            .expect("checkpoint header should include magic bytes");
        if found_magic != CHECKPOINT_MAGIC {
            return Err(PokerSolverError::CheckpointMagic {
                found: String::from_utf8_lossy(&found_magic).into_owned(),
            });
        }

        let found_version = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .expect("checkpoint header should include version bytes"),
        );
        if found_version != CHECKPOINT_VERSION {
            return Err(PokerSolverError::CheckpointVersion {
                found: found_version,
                expected: CHECKPOINT_VERSION,
            });
        }

        let mut profile =
            decode_bincode::<NlheProfile>(&bytes[CHECKPOINT_HEADER_LEN..], "nlhe profile")?;
        profile.metrics = Metrics::with_epoch(profile.iterations);

        Ok(Self::from_parts(profile, encoder))
    }

    /// Save the current profile checkpoint to disk.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), PokerSolverError> {
        let path = path.as_ref();
        let bytes = self.checkpoint_bytes()?;

        fs::write(path, bytes).map_err(|source| PokerSolverError::Write {
            path: path.display().to_string(),
            source,
        })
    }

    /// Serialize the current profile checkpoint to bytes.
    pub fn checkpoint_bytes(&self) -> Result<Vec<u8>, PokerSolverError> {
        let profile = self.snapshot_profile()?;
        let encoded = encode_bincode(&profile, "nlhe profile")?;
        let mut bytes = Vec::with_capacity(CHECKPOINT_HEADER_LEN + encoded.len());

        bytes.extend_from_slice(&CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&encoded);

        Ok(bytes)
    }

    /// Borrow the current encoder.
    pub fn encoder(&self) -> &NlheEncoder {
        self.solver.encoder()
    }

    /// Borrow the current profile.
    pub fn profile(&self) -> &NlheProfile {
        self.solver.profile()
    }

    /// Clone the current encoder through the artifact codec boundary.
    pub fn snapshot_encoder(&self) -> Result<NlheEncoder, PokerSolverError> {
        clone_with_bincode(self.encoder(), "nlhe encoder")
    }

    /// Clone the current profile through the checkpoint codec boundary.
    pub fn snapshot_profile(&self) -> Result<NlheProfile, PokerSolverError> {
        let mut profile = clone_with_bincode(self.profile(), "nlhe profile")?;
        profile.metrics = Metrics::with_epoch(profile.iterations);
        Ok(profile)
    }

    /// Snapshot the current solver state as an inference-ready blueprint.
    pub fn blueprint(&self) -> Result<NlheBlueprint, PokerSolverError> {
        Ok(NlheBlueprint::new(
            self.snapshot_encoder()?,
            self.snapshot_profile()?,
        ))
    }

    /// Return the current training epoch count.
    pub fn epochs(&self) -> usize {
        self.profile().epochs()
    }

    /// Run one MCCFR iteration.
    pub fn step(&mut self) -> Result<(), PokerSolverError> {
        catch_unwind(AssertUnwindSafe(|| self.solver.step())).map_err(|payload| {
            PokerSolverError::UpstreamPanic {
                operation: "solver step",
                message: panic_message(payload.as_ref()),
            }
        })
    }

    /// Run a fixed number of MCCFR iterations.
    pub fn train(&mut self, iterations: usize) -> Result<(), PokerSolverError> {
        for _ in 0..iterations {
            self.step()?;
        }

        Ok(())
    }

    /// Query the current average strategy for a robopoker information set.
    pub fn query(&self, info: NlheInfo) -> NlheStrategyResponse {
        StrategyResponse::new(self.profile().averaged_distribution(&info))
    }

    /// Query the current average strategy using a wire-safe key.
    pub fn query_key(&self, key: NlheInfoKey) -> NlheStrategyResponse {
        self.query(key.into_info())
    }

    /// Answer a wire-safe strategy query.
    pub fn answer(&self, query: NlheStrategyQuery) -> NlheStrategyResponse {
        self.query_key(query.info)
    }

    /// Return the highest-probability action for the supplied information set.
    pub fn recommend(&self, info: NlheInfo) -> Option<NlheEdge> {
        recommended_edge(&self.query(info))
    }

    /// Return the highest-probability action for a wire-safe key.
    pub fn recommend_key(&self, key: NlheInfoKey) -> Option<NlheEdge> {
        recommended_edge(&self.query_key(key))
    }

    /// Return the highest-probability action for a wire-safe query.
    pub fn recommend_query(&self, query: NlheStrategyQuery) -> Option<NlheEdge> {
        recommended_edge(&self.answer(query))
    }

    /// Compute exploitability using the solver's current encoder and profile.
    pub fn exploitability(&self) -> Result<Utility, PokerSolverError> {
        catch_unwind(AssertUnwindSafe(|| self.solver.exploitability())).map_err(|payload| {
            PokerSolverError::UpstreamPanic {
                operation: "solver exploitability",
                message: panic_message(payload.as_ref()),
            }
        })
    }
}

/// Errors returned by the poker solver wrapper.
#[derive(Debug, Error)]
pub enum PokerSolverError {
    #[error("failed to encode {context}: {source}")]
    Encode {
        context: &'static str,
        source: bincode::Error,
    },
    #[error("failed to decode {context}: {source}")]
    Decode {
        context: &'static str,
        source: bincode::Error,
    },
    #[error("failed to read checkpoint `{path}`: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write checkpoint `{path}`: {source}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    #[error("checkpoint too short: {bytes} bytes")]
    CheckpointTooShort { bytes: usize },
    #[error("checkpoint magic mismatch: found `{found}`, expected `MYOS`")]
    CheckpointMagic { found: String },
    #[error("checkpoint version {found} does not match expected version {expected}")]
    CheckpointVersion { found: u32, expected: u32 },
    #[error("{operation} failed upstream: {message}")]
    UpstreamPanic {
        operation: &'static str,
        message: String,
    },
}

fn clone_with_bincode<T>(value: &T, context: &'static str) -> Result<T, PokerSolverError>
where
    T: Serialize + DeserializeOwned,
{
    let bytes = encode_bincode(value, context)?;
    decode_bincode(&bytes, context)
}

fn encode_bincode<T>(value: &T, context: &'static str) -> Result<Vec<u8>, PokerSolverError>
where
    T: Serialize,
{
    encode_codec()
        .serialize(value)
        .map_err(|source| PokerSolverError::Encode { context, source })
}

fn decode_bincode<T>(bytes: &[u8], context: &'static str) -> Result<T, PokerSolverError>
where
    T: DeserializeOwned,
{
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| PokerSolverError::Decode { context, source })
}

fn encode_codec() -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .reject_trailing_bytes()
}

fn decode_codec(limit: u64) -> impl Options {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(limit)
        .reject_trailing_bytes()
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    "non-string panic payload".to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::time::{SystemTime, UNIX_EPOCH};

    use proptest::prelude::*;
    use rbp_cards::{Isomorphism, Observation};
    use rbp_gameplay::{Abstraction, Edge, Odds};
    use rbp_mccfr::Encounter;

    use super::*;
    use crate::artifacts::encoder_from_lookup;

    #[test]
    fn create_empty_solver() {
        let solver = PokerSolver::new(sample_encoder());

        assert_eq!(solver.epochs(), 0);
        assert!(solver.query(sample_info()).is_valid());
    }

    #[test]
    fn checkpoint_roundtrip_preserves_epoch_and_strategy() {
        let path = unique_checkpoint_path();
        let solver = PokerSolver::from_parts(weighted_profile(sample_info()), sample_encoder());

        solver.save(&path).expect("checkpoint should save");
        let restored = PokerSolver::load(&path, sample_encoder()).expect("checkpoint should load");
        let _ = fs::remove_file(&path);

        assert_eq!(restored.epochs(), solver.epochs());
        assert_eq!(restored.query(sample_info()), solver.query(sample_info()));
    }

    #[test]
    fn blueprint_snapshot_matches_solver_query() {
        let info = sample_info();
        let solver = PokerSolver::from_parts(weighted_profile(info), sample_encoder());
        let blueprint = solver
            .blueprint()
            .expect("blueprint snapshot should succeed");

        assert_eq!(blueprint.query(info), solver.query(info));
    }

    #[test]
    fn recommend_uses_weighted_profile() {
        let info = sample_info();
        let solver = PokerSolver::from_parts(weighted_profile(info), sample_encoder());

        assert_eq!(solver.recommend(info), Some(NlheEdge::from(Edge::Call)));
    }

    #[test]
    fn step_reports_sparse_encoder_failure_instead_of_panicking() {
        let mut solver = PokerSolver::new(sample_encoder());

        let error = solver
            .step()
            .expect_err("sparse encoder should fail cleanly");
        let message = error.to_string();

        assert!(message.contains("solver step failed upstream"));
        assert!(message.contains("isomorphism not found"));
    }

    #[test]
    fn checkpoint_rejects_wrong_magic() {
        let error = PokerSolver::from_checkpoint_bytes(b"NOPE\x01\0\0\0", sample_encoder())
            .expect_err("bad checkpoint magic should fail");

        assert!(matches!(error, PokerSolverError::CheckpointMagic { .. }));
    }

    #[test]
    fn checkpoint_decode_codec_carries_a_real_byte_limit() {
        let profile = weighted_profile(sample_info());
        let result = decode_codec(0).serialized_size(&profile);

        assert!(
            result.is_err(),
            "bounded codec should reject over-budget values"
        );
    }

    proptest! {
        #[test]
        fn prop_checkpoint_decode_rejects_truncated_payloads(trim_seed in any::<usize>()) {
            let solver = PokerSolver::from_parts(weighted_profile(sample_info()), sample_encoder());
            let encoded = solver
                .checkpoint_bytes()
                .expect("checkpoint bytes should encode");
            let trim = trim_seed % encoded.len();
            let truncated = &encoded[..trim];

            let result = catch_unwind(AssertUnwindSafe(|| {
                PokerSolver::from_checkpoint_bytes(truncated, sample_encoder())
            }));

            prop_assert!(result.is_ok());
            prop_assert!(result.expect("decode should not panic").is_err());
        }
    }

    fn sample_encoder() -> NlheEncoder {
        encoder_from_lookup(BTreeMap::from([(
            Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
            Abstraction::from(42_i16),
        )]))
        .expect("encoder lookup should build")
    }

    fn sample_info() -> NlheInfo {
        let subgame = vec![Edge::Check, Edge::Raise(Odds::new(1, 2))]
            .into_iter()
            .collect();
        let choices = vec![Edge::Fold, Edge::Call, Edge::Raise(Odds::new(1, 1))]
            .into_iter()
            .collect();
        let bucket = Abstraction::from(42_i16);

        NlheInfo::from((subgame, bucket, choices))
    }

    fn weighted_profile(info: NlheInfo) -> NlheProfile {
        NlheProfile {
            iterations: 12,
            encounters: BTreeMap::from([(
                info,
                BTreeMap::from([
                    (
                        NlheEdge::from(Edge::Fold),
                        Encounter::new(0.05, 0.0, 0.0, 1),
                    ),
                    (
                        NlheEdge::from(Edge::Call),
                        Encounter::new(0.80, 0.0, 0.0, 1),
                    ),
                    (
                        NlheEdge::from(Edge::Raise(Odds::new(1, 1))),
                        Encounter::new(0.15, 0.0, 0.0, 1),
                    ),
                ]),
            )]),
            metrics: Metrics::with_epoch(12),
        }
    }

    fn unique_checkpoint_path() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("myosu-nlhe-solver-{nanos}.bin"))
    }
}
