use std::any::Any;
use std::fmt;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};

use myosu_games::{StrategyResponse, Utility};
use rbp_mccfr::{Profile as _, Solver as _};
pub use rbp_nlhe::Flagship;
use rbp_nlhe::{NlheEdge, NlheEncoder, NlheInfo, NlheProfile};
use thiserror::Error;

pub const CHECKPOINT_MAGIC: [u8; 4] = *b"MYOS";
pub const CHECKPOINT_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum PokerSolverError {
    #[error("checkpoint read failed at {path}: {source}")]
    CheckpointRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("checkpoint write failed at {path}: {source}")]
    CheckpointWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("checkpoint is too short: expected at least 8 bytes, found {found}")]
    CheckpointTooShort { found: usize },
    #[error("checkpoint magic mismatch: expected {expected:?}, found {found:?}")]
    InvalidCheckpointMagic {
        expected: [u8; 4],
        found: [u8; 4],
    },
    #[error("checkpoint version mismatch: expected {expected}, found {found}")]
    UnsupportedCheckpointVersion { expected: u32, found: u32 },
    #[error("checkpoint encode failed: {source}")]
    CheckpointEncode {
        #[source]
        source: Box<bincode::ErrorKind>,
    },
    #[error("checkpoint decode failed: {source}")]
    CheckpointDecode {
        #[source]
        source: Box<bincode::ErrorKind>,
    },
    #[error(
        "solver {operation} requires an encoder with abstraction data; the current encoder does not contain the needed lookup entries"
    )]
    MissingEncoderArtifacts {
        operation: &'static str,
        message: String,
    },
    #[error("solver {operation} failed: {message}")]
    SolverPanicked {
        operation: &'static str,
        message: String,
    },
}

pub struct PokerSolver {
    inner: Flagship,
}

impl fmt::Debug for PokerSolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PokerSolver")
            .field("epochs", &self.epochs())
            .finish()
    }
}

impl PokerSolver {
    pub fn new_empty() -> Self {
        Self::from_parts(NlheProfile::default(), NlheEncoder::default())
    }

    pub fn from_parts(profile: NlheProfile, encoder: NlheEncoder) -> Self {
        Self {
            inner: Flagship::new(profile, encoder),
        }
    }

    pub fn from_inner(inner: Flagship) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> Flagship {
        self.inner
    }

    pub fn profile(&self) -> &NlheProfile {
        &self.inner.profile
    }

    pub fn profile_mut(&mut self) -> &mut NlheProfile {
        &mut self.inner.profile
    }

    pub fn encoder(&self) -> &NlheEncoder {
        &self.inner.encoder
    }

    pub fn epochs(&self) -> usize {
        self.profile().epochs()
    }

    pub fn strategy(&self, info: &NlheInfo) -> StrategyResponse<NlheEdge> {
        StrategyResponse::new(self.profile().averaged_distribution(info).into_iter().collect())
    }

    pub fn train(&mut self, iterations: usize) -> Result<(), PokerSolverError> {
        self.run_solver_mut("train", |solver| {
            for _ in 0..iterations {
                solver.step();
            }
        })
    }

    pub fn exploitability(&self) -> Result<Utility, PokerSolverError> {
        self.run_solver("exploitability", |solver| solver.exploitability())
    }

    pub fn snapshot_profile(&self) -> Result<NlheProfile, PokerSolverError> {
        let encoded =
            bincode::serialize(self.profile()).map_err(|source| PokerSolverError::CheckpointEncode {
                source,
            })?;
        bincode::deserialize(&encoded).map_err(|source| PokerSolverError::CheckpointDecode {
            source,
        })
    }

    pub fn save<P>(&self, path: P) -> Result<(), PokerSolverError>
    where
        P: AsRef<Path>,
    {
        save_profile(self.profile(), path)
    }

    pub fn load<P>(path: P, encoder: NlheEncoder) -> Result<Self, PokerSolverError>
    where
        P: AsRef<Path>,
    {
        let profile = Self::load_profile(path)?;
        Ok(Self::from_parts(profile, encoder))
    }

    pub fn load_profile<P>(path: P) -> Result<NlheProfile, PokerSolverError>
    where
        P: AsRef<Path>,
    {
        load_profile(path)
    }

    fn run_solver<T, F>(&self, operation: &'static str, mut f: F) -> Result<T, PokerSolverError>
    where
        F: FnMut(&Flagship) -> T,
    {
        catch_unwind(AssertUnwindSafe(|| f(&self.inner)))
            .map_err(|payload| map_solver_panic(operation, payload))
    }

    fn run_solver_mut<T, F>(
        &mut self,
        operation: &'static str,
        mut f: F,
    ) -> Result<T, PokerSolverError>
    where
        F: FnMut(&mut Flagship) -> T,
    {
        catch_unwind(AssertUnwindSafe(|| f(&mut self.inner)))
            .map_err(|payload| map_solver_panic(operation, payload))
    }
}

fn save_profile<P>(profile: &NlheProfile, path: P) -> Result<(), PokerSolverError>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let payload =
        bincode::serialize(profile).map_err(|source| PokerSolverError::CheckpointEncode {
            source,
        })?;
    let mut bytes = Vec::with_capacity(CHECKPOINT_MAGIC.len() + std::mem::size_of::<u32>() + payload.len());
    bytes.extend_from_slice(&CHECKPOINT_MAGIC);
    bytes.extend_from_slice(&CHECKPOINT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&payload);
    fs::write(path, bytes).map_err(|source| PokerSolverError::CheckpointWrite {
        path: path.to_path_buf(),
        source,
    })
}

fn load_profile<P>(path: P) -> Result<NlheProfile, PokerSolverError>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| PokerSolverError::CheckpointRead {
        path: path.to_path_buf(),
        source,
    })?;
    if bytes.len() < CHECKPOINT_MAGIC.len() + std::mem::size_of::<u32>() {
        return Err(PokerSolverError::CheckpointTooShort { found: bytes.len() });
    }

    let found_magic: [u8; 4] = bytes[..4].try_into().expect("magic length matches");
    if found_magic != CHECKPOINT_MAGIC {
        return Err(PokerSolverError::InvalidCheckpointMagic {
            expected: CHECKPOINT_MAGIC,
            found: found_magic,
        });
    }

    let found_version = u32::from_le_bytes(bytes[4..8].try_into().expect("version length matches"));
    if found_version != CHECKPOINT_VERSION {
        return Err(PokerSolverError::UnsupportedCheckpointVersion {
            expected: CHECKPOINT_VERSION,
            found: found_version,
        });
    }

    bincode::deserialize(&bytes[8..]).map_err(|source| PokerSolverError::CheckpointDecode {
        source,
    })
}

fn map_solver_panic(operation: &'static str, payload: Box<dyn Any + Send>) -> PokerSolverError {
    let message = panic_message(payload);
    if message.contains("isomorphism not found in abstraction lookup") {
        PokerSolverError::MissingEncoderArtifacts { operation, message }
    } else {
        PokerSolverError::SolverPanicked { operation, message }
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    match payload.downcast::<String>() {
        Ok(message) => *message,
        Err(payload) => match payload.downcast::<&'static str>() {
            Ok(message) => (*message).to_string(),
            Err(_) => "non-string panic payload".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{SystemTime, UNIX_EPOCH};

    use myosu_games::CfrInfo;
    use rbp_core::Arbitrary;
    use rbp_mccfr::Encounter;

    #[test]
    fn create_empty_solver() {
        let solver = PokerSolver::new_empty();

        assert_eq!(solver.epochs(), 0);
        assert!(solver.profile().encounters.is_empty());
    }

    #[test]
    fn strategy_is_valid_distribution() {
        let solver = PokerSolver::new_empty();
        let info = NlheInfo::random();

        let response = solver.strategy(&info);

        assert!(response.is_valid());
        assert_eq!(response.actions.len(), CfrInfo::choices(&info).len());
    }

    #[test]
    fn checkpoint_roundtrip() {
        let mut solver = PokerSolver::new_empty();
        let info = NlheInfo::random();
        let edge = CfrInfo::choices(&info)
            .into_iter()
            .next()
            .expect("random infos always expose at least one action");
        solver.profile_mut().iterations = 42;
        solver
            .profile_mut()
            .encounters
            .entry(info)
            .or_default()
            .insert(edge, Encounter::new(0.6, 0.3, 0.2, 7));

        let path = temp_checkpoint_path("roundtrip");
        solver.save(&path).expect("checkpoint write succeeds");
        let loaded =
            PokerSolver::load(&path, NlheEncoder::default()).expect("checkpoint load succeeds");

        assert_eq!(loaded.epochs(), 42);
        assert_eq!(loaded.profile().encounters.len(), 1);
        assert_eq!(
            loaded
                .profile()
                .encounters
                .get(&info)
                .and_then(|edges| edges.get(&edge))
                .map(|encounter| encounter.counts),
            Some(7)
        );

        fs::remove_file(path).expect("checkpoint cleanup succeeds");
    }

    #[test]
    fn checkpoint_rejects_bad_magic() {
        let path = temp_checkpoint_path("bad-magic");
        fs::write(&path, b"NOPE\x01\x00\x00\x00payload").expect("fixture write succeeds");

        let error = match PokerSolver::load_profile(&path) {
            Ok(_) => panic!("invalid magic is rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            PokerSolverError::InvalidCheckpointMagic { .. }
        ));

        fs::remove_file(path).expect("fixture cleanup succeeds");
    }

    #[test]
    fn checkpoint_rejects_unknown_version() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&CHECKPOINT_MAGIC);
        bytes.extend_from_slice(&999u32.to_le_bytes());
        bytes.extend_from_slice(
            &bincode::serialize(&NlheProfile::default()).expect("fixture encode succeeds"),
        );

        let path = temp_checkpoint_path("bad-version");
        fs::write(&path, bytes).expect("fixture write succeeds");

        let error = match PokerSolver::load_profile(&path) {
            Ok(_) => panic!("unknown version is rejected"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            PokerSolverError::UnsupportedCheckpointVersion { found: 999, .. }
        ));

        fs::remove_file(path).expect("fixture cleanup succeeds");
    }

    #[test]
    fn train_surfaces_missing_encoder_artifacts() {
        let mut solver = PokerSolver::new_empty();

        let error = solver
            .train(1)
            .expect_err("training without abstraction data returns an error");

        assert!(matches!(
            error,
            PokerSolverError::MissingEncoderArtifacts {
                operation: "train",
                ..
            }
        ));
    }

    #[test]
    fn exploitability_surfaces_missing_encoder_artifacts() {
        let solver = PokerSolver::new_empty();

        let error = solver
            .exploitability()
            .expect_err("scoring without abstraction data returns an error");

        assert!(matches!(
            error,
            PokerSolverError::MissingEncoderArtifacts {
                operation: "exploitability",
                ..
            }
        ));
    }

    #[test]
    fn snapshot_profile_preserves_profile_data() {
        let mut solver = PokerSolver::new_empty();
        let info = NlheInfo::random();
        let edge = CfrInfo::choices(&info)
            .into_iter()
            .next()
            .expect("random infos always expose at least one action");
        solver.profile_mut().iterations = 5;
        solver
            .profile_mut()
            .encounters
            .entry(info)
            .or_default()
            .insert(edge, Encounter::new(0.25, 0.5, 0.75, 3));

        let snapshot = solver.snapshot_profile().expect("snapshot succeeds");

        assert_eq!(snapshot.iterations, 5);
        assert_eq!(snapshot.encounters.len(), 1);
        assert_eq!(
            snapshot
                .encounters
                .get(&info)
                .and_then(|edges| edges.get(&edge))
                .map(|encounter| encounter.weight),
            Some(0.25)
        );
    }

    fn temp_checkpoint_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "myosu-games-poker-{name}-{}-{nanos}.chk",
            std::process::id()
        ))
    }
}
