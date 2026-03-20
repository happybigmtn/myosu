use crate::training::BotBackend;
use rbp_cards::Street;
use rbp_gameplay::{Action, Game, Recall};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::ptr;
use std::slice;
use thiserror::Error;

const BLUEPRINT_SCHEMA_VERSION: u32 = 1;
const MANIFEST_FILE: &str = "blueprint.manifest.json";
const KEYS_FILE: &str = "blueprint.keys.bin";
const VALUES_FILE: &str = "blueprint.values.bin";
const ISOMORPHISM_FILE: &str = "blueprint.isomorphism.bin";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlueprintManifest {
    pub schema_version: u32,
    pub game_format: GameFormat,
    pub player_count: u8,
    pub abstraction_hash: String,
    pub profile_hash: String,
    pub iterations: u64,
    pub exploitability: f64,
}

impl BlueprintManifest {
    fn validate_schema(&self) -> Result<(), BlueprintError> {
        if self.schema_version == BLUEPRINT_SCHEMA_VERSION {
            Ok(())
        } else {
            Err(BlueprintError::SchemaMismatch {
                found: self.schema_version,
                expected: BLUEPRINT_SCHEMA_VERSION,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameFormat {
    Cash,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum StreetBucket {
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PressureBucket {
    CanCheck,
    FacingBet,
    HighPressure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BucketRule {
    street: StreetBucket,
    pressure: PressureBucket,
    bucket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IsomorphismIndex {
    default_bucket: String,
    rules: Vec<BucketRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeightedBlueprintAction {
    action: ActionLabel,
    weight: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ActionLabel {
    Fold,
    Check,
    Call,
    Raise,
    Shove,
}

#[derive(Debug)]
struct ReadOnlyMmap {
    ptr: *const u8,
    len: usize,
}

unsafe impl Send for ReadOnlyMmap {}
unsafe impl Sync for ReadOnlyMmap {}

impl ReadOnlyMmap {
    fn open(path: &Path) -> Result<Self, BlueprintError> {
        let file = File::open(path).map_err(|source| BlueprintError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        let len = usize::try_from(
            file.metadata()
                .map_err(|source| BlueprintError::ReadFile {
                    path: path.to_path_buf(),
                    source,
                })?
                .len(),
        )
        .map_err(|_| BlueprintError::FileTooLarge {
            path: path.to_path_buf(),
        })?;

        if len == 0 {
            return Err(BlueprintError::EmptyFile {
                path: path.to_path_buf(),
            });
        }

        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(BlueprintError::MapFile {
                path: path.to_path_buf(),
                source: io::Error::last_os_error(),
            });
        }

        Ok(Self {
            ptr: ptr.cast::<u8>(),
            len,
        })
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl Drop for ReadOnlyMmap {
    fn drop(&mut self) {
        if self.len > 0 {
            unsafe {
                libc::munmap(self.ptr.cast_mut().cast::<libc::c_void>(), self.len);
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum BlueprintError {
    #[error("blueprint not found (set MYOSU_BLUEPRINT_DIR)")]
    ArtifactNotFound { searched: Vec<PathBuf> },
    #[error("blueprint schema v{found} unsupported (expected v{expected})")]
    SchemaMismatch { found: u32, expected: u32 },
    #[error("abstraction hash mismatch (artifact corrupted)")]
    AbstractionHashMismatch { expected: String, actual: String },
    #[error("profile hash mismatch (artifact corrupted)")]
    ProfileHashMismatch { expected: String, actual: String },
    #[error("failed to read {path}: {source}")]
    ReadFile { path: PathBuf, source: io::Error },
    #[error("failed to memory-map {path}: {source}")]
    MapFile { path: PathBuf, source: io::Error },
    #[error("failed to parse blueprint manifest: {source}")]
    InvalidManifest { source: serde_json::Error },
    #[error("failed to parse blueprint keys: {source}")]
    InvalidKeys { source: serde_json::Error },
    #[error("failed to parse blueprint values: {source}")]
    InvalidValues { source: serde_json::Error },
    #[error("failed to parse blueprint isomorphism: {source}")]
    InvalidIsomorphism { source: serde_json::Error },
    #[error("blueprint keys and values length mismatch")]
    KeyValueLengthMismatch,
    #[error("blueprint file is empty: {path}")]
    EmptyFile { path: PathBuf },
    #[error("blueprint file is too large to map in-process: {path}")]
    FileTooLarge { path: PathBuf },
    #[error("blueprint references missing bucket {bucket}")]
    MissingBucket { bucket: String },
    #[error("blueprint contains no distributions")]
    EmptyDistributions,
}

impl BlueprintError {
    pub fn fallback_reason(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug)]
pub struct BlueprintBackend {
    manifest: BlueprintManifest,
    keys_index: HashMap<String, usize>,
    distributions: Vec<Vec<WeightedBlueprintAction>>,
    isomorphism: IsomorphismIndex,
    _keys_map: ReadOnlyMmap,
    _values_map: ReadOnlyMmap,
    _isomorphism_map: ReadOnlyMmap,
}

impl BlueprintBackend {
    pub fn load_default() -> Result<Self, BlueprintError> {
        let candidates = default_blueprint_dirs();
        for candidate in &candidates {
            if candidate.join(MANIFEST_FILE).is_file() {
                return Self::load_from_dir(candidate);
            }
        }

        Err(BlueprintError::ArtifactNotFound {
            searched: candidates,
        })
    }

    pub fn load_from_dir(path: impl AsRef<Path>) -> Result<Self, BlueprintError> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() || !path.join(MANIFEST_FILE).is_file() {
            return Err(BlueprintError::ArtifactNotFound {
                searched: vec![path],
            });
        }

        let manifest_bytes =
            fs::read(path.join(MANIFEST_FILE)).map_err(|source| BlueprintError::ReadFile {
                path: path.join(MANIFEST_FILE),
                source,
            })?;
        let manifest: BlueprintManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|source| BlueprintError::InvalidManifest { source })?;
        manifest.validate_schema()?;

        let keys_map = ReadOnlyMmap::open(&path.join(KEYS_FILE))?;
        let values_map = ReadOnlyMmap::open(&path.join(VALUES_FILE))?;
        let isomorphism_map = ReadOnlyMmap::open(&path.join(ISOMORPHISM_FILE))?;

        let abstraction_hash = sha256_hex(isomorphism_map.as_bytes());
        if abstraction_hash != manifest.abstraction_hash {
            return Err(BlueprintError::AbstractionHashMismatch {
                expected: manifest.abstraction_hash.clone(),
                actual: abstraction_hash,
            });
        }

        let profile_hash = sha256_hex_many([keys_map.as_bytes(), values_map.as_bytes()]);
        if profile_hash != manifest.profile_hash {
            return Err(BlueprintError::ProfileHashMismatch {
                expected: manifest.profile_hash.clone(),
                actual: profile_hash,
            });
        }

        let keys: Vec<String> = serde_json::from_slice(keys_map.as_bytes())
            .map_err(|source| BlueprintError::InvalidKeys { source })?;
        let distributions: Vec<Vec<WeightedBlueprintAction>> =
            serde_json::from_slice(values_map.as_bytes())
                .map_err(|source| BlueprintError::InvalidValues { source })?;
        let isomorphism: IsomorphismIndex = serde_json::from_slice(isomorphism_map.as_bytes())
            .map_err(|source| BlueprintError::InvalidIsomorphism { source })?;

        if keys.len() != distributions.len() {
            return Err(BlueprintError::KeyValueLengthMismatch);
        }
        if distributions.is_empty() {
            return Err(BlueprintError::EmptyDistributions);
        }

        let keys_index = keys
            .into_iter()
            .enumerate()
            .map(|(index, key)| (key, index))
            .collect::<HashMap<_, _>>();

        validate_buckets(&isomorphism, &keys_index)?;

        Ok(Self {
            manifest,
            keys_index,
            distributions,
            isomorphism,
            _keys_map: keys_map,
            _values_map: values_map,
            _isomorphism_map: isomorphism_map,
        })
    }

    pub fn strategy_status(&self) -> String {
        format!(
            "bot strategy: blueprint · exploit {:.1} mbb/h",
            self.manifest.exploitability
        )
    }

    fn distribution_for_game(&self, game: &Game) -> Vec<(Action, f64)> {
        let legal = game.legal();
        if legal.is_empty() {
            return Vec::new();
        }

        let bucket = self.lookup_bucket(game);
        let index = self.keys_index.get(bucket).copied().or_else(|| {
            self.keys_index
                .get(&self.isomorphism.default_bucket)
                .copied()
        });

        let Some(index) = index else {
            return uniform_legal_distribution(game);
        };

        let mut mapped = Vec::new();
        for entry in &self.distributions[index] {
            if entry.weight <= 0.0 {
                continue;
            }
            if let Some(action) = resolve_action_label(game, entry.action) {
                if let Some((_, existing_weight)) =
                    mapped.iter_mut().find(|(existing, _)| *existing == action)
                {
                    *existing_weight += entry.weight;
                } else {
                    mapped.push((action, entry.weight));
                }
            }
        }

        if mapped.is_empty() {
            return uniform_legal_distribution(game);
        }

        normalize_distribution(&mut mapped);
        mapped
    }

    fn lookup_bucket<'a>(&'a self, game: &Game) -> &'a str {
        let street = StreetBucket::from_street(game.street());
        let pressure = PressureBucket::for_game(game);

        self.isomorphism
            .rules
            .iter()
            .find(|rule| rule.street == street && rule.pressure == pressure)
            .map(|rule| rule.bucket.as_str())
            .unwrap_or(self.isomorphism.default_bucket.as_str())
    }
}

impl BotBackend for BlueprintBackend {
    fn strategy_name(&self) -> &str {
        "blueprint"
    }

    fn action_distribution(&self, recall: &dyn Recall, _seat: usize) -> Vec<(Action, f64)> {
        self.distribution_for_game(&recall.head())
    }
}

impl StreetBucket {
    fn from_street(street: Street) -> Self {
        match street {
            Street::Pref => Self::Preflop,
            Street::Flop => Self::Flop,
            Street::Turn => Self::Turn,
            Street::Rive => Self::River,
        }
    }
}

impl PressureBucket {
    fn for_game(game: &Game) -> Self {
        if game.may_check() {
            Self::CanCheck
        } else if game.to_call() > (game.pot().max(1) / 2) {
            Self::HighPressure
        } else {
            Self::FacingBet
        }
    }
}

fn validate_buckets(
    isomorphism: &IsomorphismIndex,
    keys_index: &HashMap<String, usize>,
) -> Result<(), BlueprintError> {
    if !keys_index.contains_key(&isomorphism.default_bucket) {
        return Err(BlueprintError::MissingBucket {
            bucket: isomorphism.default_bucket.clone(),
        });
    }

    for rule in &isomorphism.rules {
        if !keys_index.contains_key(&rule.bucket) {
            return Err(BlueprintError::MissingBucket {
                bucket: rule.bucket.clone(),
            });
        }
    }

    Ok(())
}

fn default_blueprint_dirs() -> Vec<PathBuf> {
    if let Ok(explicit) = env::var("MYOSU_BLUEPRINT_DIR") {
        return vec![PathBuf::from(explicit)];
    }

    let mut dirs = Vec::new();
    if let Ok(data_dir) = env::var("MYOSU_DATA_DIR") {
        dirs.push(PathBuf::from(data_dir).join(".myosu/blueprints"));
    }
    if let Ok(home) = env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".myosu/blueprints"));
    }
    dirs
}

fn resolve_action_label(game: &Game, label: ActionLabel) -> Option<Action> {
    match label {
        ActionLabel::Fold if game.may_fold() => Some(Action::Fold),
        ActionLabel::Check if game.may_check() => Some(Action::Check),
        ActionLabel::Call => {
            if game.may_call() {
                Some(game.calls())
            } else if game.may_check() {
                Some(Action::Check)
            } else {
                None
            }
        }
        ActionLabel::Raise => {
            if game.may_raise() {
                Some(game.raise())
            } else if game.may_shove() {
                Some(game.shove())
            } else {
                None
            }
        }
        ActionLabel::Shove if game.may_shove() => Some(game.shove()),
        _ => None,
    }
}

fn uniform_legal_distribution(game: &Game) -> Vec<(Action, f64)> {
    let legal = game.legal();
    if legal.is_empty() {
        return Vec::new();
    }

    let even = 1.0 / legal.len() as f64;
    legal.into_iter().map(|action| (action, even)).collect()
}

fn normalize_distribution(distribution: &mut Vec<(Action, f64)>) {
    let total = distribution.iter().map(|(_, weight)| *weight).sum::<f64>();
    if total <= f64::EPSILON {
        let even = 1.0 / distribution.len() as f64;
        for (_, weight) in distribution.iter_mut() {
            *weight = even;
        }
        return;
    }

    for (_, weight) in distribution.iter_mut() {
        *weight /= total;
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn sha256_hex_many<const N: usize>(chunks: [&[u8]; N]) -> String {
    let mut hasher = Sha256::new();
    for chunk in chunks {
        hasher.update(chunk);
    }
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::training::{HeuristicBackend, TrainingTable};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Debug)]
    struct TestRecall {
        root: Game,
        actions: Vec<Action>,
    }

    impl Recall for TestRecall {
        fn root(&self) -> Game {
            self.root
        }

        fn actions(&self) -> &[Action] {
            &self.actions
        }
    }

    #[derive(Debug)]
    struct TempArtifactDir {
        path: PathBuf,
    }

    impl TempArtifactDir {
        fn new(name: &str) -> Self {
            static NEXT_ID: AtomicU64 = AtomicU64::new(1);

            let path = env::temp_dir().join(format!(
                "myosu-blueprint-{name}-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).expect("temp dir should be creatable");
            Self { path }
        }
    }

    impl Drop for TempArtifactDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn load_valid_artifact() {
        let artifact = TempArtifactDir::new("valid");
        write_valid_artifact(&artifact.path);

        let backend = BlueprintBackend::load_from_dir(&artifact.path).expect("load artifact");

        assert_eq!(backend.strategy_name(), "blueprint");
        assert_eq!(
            backend.strategy_status(),
            "bot strategy: blueprint · exploit 2.3 mbb/h"
        );
    }

    #[test]
    fn missing_dir_returns_error() {
        let missing = env::temp_dir().join("myosu-missing-blueprint-dir");
        let error = BlueprintBackend::load_from_dir(&missing).expect_err("missing artifact");

        assert!(matches!(error, BlueprintError::ArtifactNotFound { .. }));
    }

    #[test]
    fn schema_mismatch_returns_error() {
        let artifact = TempArtifactDir::new("schema");
        let mut manifest = write_valid_artifact(&artifact.path);
        manifest.schema_version = 2;
        fs::write(
            artifact.path.join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("manifest bytes"),
        )
        .expect("manifest rewrite");

        let error = BlueprintBackend::load_from_dir(&artifact.path).expect_err("schema mismatch");

        assert!(matches!(
            error,
            BlueprintError::SchemaMismatch {
                found: 2,
                expected: BLUEPRINT_SCHEMA_VERSION
            }
        ));
    }

    #[test]
    fn hash_mismatch_returns_error() {
        let artifact = TempArtifactDir::new("hash");
        let mut manifest = write_valid_artifact(&artifact.path);
        manifest.profile_hash = "deadbeef".to_string();
        fs::write(
            artifact.path.join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("manifest bytes"),
        )
        .expect("manifest rewrite");

        let error = BlueprintBackend::load_from_dir(&artifact.path).expect_err("hash mismatch");

        assert!(matches!(error, BlueprintError::ProfileHashMismatch { .. }));
    }

    #[test]
    fn lookup_returns_valid_distribution() {
        let artifact = TempArtifactDir::new("lookup");
        write_valid_artifact(&artifact.path);
        let backend = BlueprintBackend::load_from_dir(&artifact.path).expect("load artifact");
        let recall = hero_decision_recall();

        let distribution = backend.action_distribution(&recall, 0);
        let legal = recall.head().legal();

        assert!(!distribution.is_empty());
        for (action, _) in &distribution {
            assert!(legal.contains(action));
        }
    }

    #[test]
    fn distribution_sums_to_one() {
        let artifact = TempArtifactDir::new("sum");
        write_valid_artifact(&artifact.path);
        let backend = BlueprintBackend::load_from_dir(&artifact.path).expect("load artifact");
        let recall = hero_decision_recall();

        let distribution = backend.action_distribution(&recall, 0);
        let sum = distribution.iter().map(|(_, weight)| *weight).sum::<f64>();

        assert!((sum - 1.0).abs() < 0.001);
    }

    fn hero_decision_recall() -> TestRecall {
        let backend = Arc::new(HeuristicBackend);
        let mut table = TrainingTable::with_backend_and_delay(backend, 0);
        table
            .advance_until_hero_or_terminal_sync()
            .expect("hero decision state");
        TestRecall {
            root: Game::root(),
            actions: table.history().to_vec(),
        }
    }

    fn write_valid_artifact(path: &Path) -> BlueprintManifest {
        let isomorphism = IsomorphismIndex {
            default_bucket: "preflop_facing_bet".to_string(),
            rules: vec![
                BucketRule {
                    street: StreetBucket::Preflop,
                    pressure: PressureBucket::CanCheck,
                    bucket: "preflop_can_check".to_string(),
                },
                BucketRule {
                    street: StreetBucket::Preflop,
                    pressure: PressureBucket::FacingBet,
                    bucket: "preflop_facing_bet".to_string(),
                },
                BucketRule {
                    street: StreetBucket::Preflop,
                    pressure: PressureBucket::HighPressure,
                    bucket: "preflop_high_pressure".to_string(),
                },
                BucketRule {
                    street: StreetBucket::Flop,
                    pressure: PressureBucket::CanCheck,
                    bucket: "flop_can_check".to_string(),
                },
                BucketRule {
                    street: StreetBucket::Flop,
                    pressure: PressureBucket::FacingBet,
                    bucket: "flop_facing_bet".to_string(),
                },
            ],
        };

        let keys = vec![
            "preflop_can_check".to_string(),
            "preflop_facing_bet".to_string(),
            "preflop_high_pressure".to_string(),
            "flop_can_check".to_string(),
            "flop_facing_bet".to_string(),
        ];
        let values = vec![
            vec![
                WeightedBlueprintAction {
                    action: ActionLabel::Raise,
                    weight: 0.62,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Check,
                    weight: 0.38,
                },
            ],
            vec![
                WeightedBlueprintAction {
                    action: ActionLabel::Raise,
                    weight: 0.45,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Call,
                    weight: 0.40,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Fold,
                    weight: 0.15,
                },
            ],
            vec![
                WeightedBlueprintAction {
                    action: ActionLabel::Call,
                    weight: 0.55,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Shove,
                    weight: 0.30,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Fold,
                    weight: 0.15,
                },
            ],
            vec![
                WeightedBlueprintAction {
                    action: ActionLabel::Raise,
                    weight: 0.50,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Check,
                    weight: 0.50,
                },
            ],
            vec![
                WeightedBlueprintAction {
                    action: ActionLabel::Call,
                    weight: 0.55,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Raise,
                    weight: 0.30,
                },
                WeightedBlueprintAction {
                    action: ActionLabel::Fold,
                    weight: 0.15,
                },
            ],
        ];

        let isomorphism_bytes = serde_json::to_vec(&isomorphism).expect("isomorphism bytes");
        let keys_bytes = serde_json::to_vec(&keys).expect("keys bytes");
        let values_bytes = serde_json::to_vec(&values).expect("values bytes");

        let manifest = BlueprintManifest {
            schema_version: BLUEPRINT_SCHEMA_VERSION,
            game_format: GameFormat::Cash,
            player_count: 2,
            abstraction_hash: sha256_hex(&isomorphism_bytes),
            profile_hash: sha256_hex_many([&keys_bytes, &values_bytes]),
            iterations: 12_345,
            exploitability: 2.3,
        };

        fs::write(
            path.join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("manifest bytes"),
        )
        .expect("manifest write");
        fs::write(path.join(KEYS_FILE), keys_bytes).expect("keys write");
        fs::write(path.join(VALUES_FILE), values_bytes).expect("values write");
        fs::write(path.join(ISOMORPHISM_FILE), isomorphism_bytes).expect("isomorphism write");

        manifest
    }
}
