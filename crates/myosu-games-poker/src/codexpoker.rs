use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use memmap2::Mmap;
use rbp_cards::{Isomorphism, Observation};
use rbp_gameplay::{Abstraction, Edge};
use rbp_nlhe::{NlheEdge, NlheInfo};
use serde::Deserialize;
use thiserror::Error;

use crate::request::NlheStrategyRequest;
use crate::robopoker::{NlheInfoKey, NlheStrategyResponse};
use crate::state::NlheTablePosition;

const MANIFEST_FILE: &str = "blueprint.manifest.json";
const ISOMORPHISM_FILE: &str = "blueprint.isomorphism.bin";
const KEY_ENTRY_SIZE: usize = 30;
const VALUE_ENTRY_SIZE: usize = 12;
const ISO_ENTRY_SIZE_V2: usize = 10;
const ISO_ENTRY_SIZE_V3: usize = 11;

#[derive(Debug)]
pub struct CodexpokerBlueprint {
    store: CodexpokerStore,
    isomorphism: CodexpokerIsomorphism,
}

impl CodexpokerBlueprint {
    pub fn load(directory: &Path) -> Result<Self, CodexpokerBlueprintError> {
        let manifest_path = directory.join(MANIFEST_FILE);
        let manifest = load_manifest(&manifest_path)?;
        let keys_path = directory.join(&manifest.keys_file);
        let values_path = directory.join(&manifest.values_file);
        let isomorphism_path = directory.join(ISOMORPHISM_FILE);

        let store = CodexpokerStore::open(&keys_path, &values_path)?;
        let isomorphism = CodexpokerIsomorphism::open(&isomorphism_path, manifest.position_aware)?;

        Ok(Self { store, isomorphism })
    }

    pub fn recommend_request(&self, request: &NlheStrategyRequest) -> Option<NlheEdge> {
        let response = self.response_for_request(request)?;
        response
            .actions
            .iter()
            .copied()
            .max_by(|(left_edge, left_prob), (right_edge, right_prob)| {
                left_prob
                    .total_cmp(right_prob)
                    .then_with(|| left_edge.cmp(right_edge))
            })
            .map(|(edge, _)| edge)
    }

    fn response_for_request(&self, request: &NlheStrategyRequest) -> Option<NlheStrategyResponse> {
        let observation = request.observation().ok()?;
        let partial = request.partial().ok()?;
        let seat_position = seat_position(request.hero_position);
        let abstraction = self.isomorphism.lookup(&observation, seat_position)?;
        let info = NlheInfo::from((&partial, abstraction));
        let key = NlheInfoKey::from(info);
        let actions = self
            .store
            .lookup(&key)?
            .into_iter()
            .map(|(edge, weight)| (NlheEdge::from(edge), weight))
            .collect();
        Some(NlheStrategyResponse::new(actions))
    }
}

#[derive(Debug)]
struct CodexpokerStore {
    keys: Mmap,
    values: Mmap,
    entries: usize,
}

impl CodexpokerStore {
    fn open(keys_path: &Path, values_path: &Path) -> Result<Self, CodexpokerBlueprintError> {
        let keys = open_mmap(keys_path)?;
        let values = open_mmap(values_path)?;
        if keys.len() % KEY_ENTRY_SIZE != 0 {
            return Err(CodexpokerBlueprintError::MalformedKeys {
                path: keys_path.to_path_buf(),
            });
        }
        validate_value_ranges(keys_path, values_path, &keys, &values)?;

        Ok(Self {
            entries: keys.len() / KEY_ENTRY_SIZE,
            keys,
            values,
        })
    }

    fn lookup(&self, key: &NlheInfoKey) -> Option<Vec<(Edge, f32)>> {
        let mut low = 0usize;
        let mut high = self.entries;
        while low < high {
            let mid = low + (high - low) / 2;
            let found = self.key_at(mid)?;
            match cmp_key(key, &found) {
                std::cmp::Ordering::Less => high = mid,
                std::cmp::Ordering::Greater => low = mid + 1,
                std::cmp::Ordering::Equal => {
                    let (offset, len) = self.offset_len_at(mid)?;
                    return self.decode_values(offset, len);
                }
            }
        }
        None
    }

    fn key_at(&self, index: usize) -> Option<NlheInfoKey> {
        let start = index.checked_mul(KEY_ENTRY_SIZE)?;
        let end = start.checked_add(KEY_ENTRY_SIZE)?;
        let bytes = self.keys.get(start..end)?;

        Some(NlheInfoKey {
            subgame: u64::from_le_bytes(bytes[0..8].try_into().ok()?),
            bucket: i16::from_le_bytes(bytes[8..10].try_into().ok()?),
            choices: u64::from_le_bytes(bytes[10..18].try_into().ok()?),
        })
    }

    fn offset_len_at(&self, index: usize) -> Option<(u64, u32)> {
        let start = index.checked_mul(KEY_ENTRY_SIZE)?.checked_add(18)?;
        let end = start.checked_add(12)?;
        let bytes = self.keys.get(start..end)?;

        Some((
            u64::from_le_bytes(bytes[0..8].try_into().ok()?),
            u32::from_le_bytes(bytes[8..12].try_into().ok()?),
        ))
    }

    fn decode_values(&self, offset: u64, len: u32) -> Option<Vec<(Edge, f32)>> {
        let start = usize::try_from(offset).ok()?;
        let len = usize::try_from(len).ok()?;
        let end = start.checked_add(len)?;
        let bytes = self.values.get(start..end)?;
        let count = usize::from(u16::from_le_bytes(bytes.get(0..2)?.try_into().ok()?));
        let expected = 2usize.checked_add(count.checked_mul(VALUE_ENTRY_SIZE)?)?;
        if bytes.len() < expected {
            return None;
        }

        let mut entries = Vec::with_capacity(count);
        let mut cursor = 2usize;
        for _ in 0..count {
            let edge = Edge::from(u64::from_le_bytes(
                bytes.get(cursor..cursor + 8)?.try_into().ok()?,
            ));
            cursor += 8;
            let policy = f32::from_le_bytes(bytes.get(cursor..cursor + 4)?.try_into().ok()?);
            cursor += 4;
            entries.push((edge, policy));
        }
        Some(entries)
    }
}

#[derive(Debug)]
struct CodexpokerIsomorphism {
    mmap: Mmap,
    entries: usize,
    position_aware: bool,
}

impl CodexpokerIsomorphism {
    fn open(
        path: &Path,
        position_aware_hint: Option<bool>,
    ) -> Result<Self, CodexpokerBlueprintError> {
        let file = File::open(path).map_err(|source| CodexpokerBlueprintError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let metadata = file
            .metadata()
            .map_err(|source| CodexpokerBlueprintError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        let len = usize::try_from(metadata.len()).map_err(|_| {
            CodexpokerBlueprintError::MalformedIsomorphism {
                path: path.to_path_buf(),
            }
        })?;
        let (entry_size, position_aware) =
            infer_iso_format(len, position_aware_hint).ok_or_else(|| {
                CodexpokerBlueprintError::MalformedIsomorphism {
                    path: path.to_path_buf(),
                }
            })?;
        let mmap = unsafe { Mmap::map(&file) }.map_err(|source| CodexpokerBlueprintError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self {
            entries: len / entry_size,
            mmap,
            position_aware,
        })
    }

    fn lookup(&self, observation: &Observation, seat_position: u8) -> Option<Abstraction> {
        let target = i64::from(Isomorphism::from(*observation));
        let mut low = 0usize;
        let mut high = self.entries;
        while low < high {
            let mid = low + (high - low) / 2;
            let entry = self.entry_at(mid)?;
            let ordering = if self.position_aware {
                entry
                    .obs
                    .cmp(&target)
                    .then_with(|| entry.seat_position.cmp(&seat_position))
            } else {
                entry.obs.cmp(&target)
            };
            match ordering {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return Some(Abstraction::from(entry.bucket)),
            }
        }
        if self.position_aware && seat_position != 0 {
            return self.lookup(observation, 0);
        }
        Some(Abstraction::from(observation.equity()))
    }

    fn entry_at(&self, index: usize) -> Option<CodexpokerIsoEntry> {
        let entry_size = if self.position_aware {
            ISO_ENTRY_SIZE_V3
        } else {
            ISO_ENTRY_SIZE_V2
        };
        let start = index.checked_mul(entry_size)?;
        let end = start.checked_add(entry_size)?;
        let bytes = self.mmap.get(start..end)?;
        let obs = i64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let bucket = i16::from_le_bytes(bytes[8..10].try_into().ok()?);
        let seat_position = if self.position_aware { bytes[10] } else { 0 };
        Some(CodexpokerIsoEntry {
            obs,
            bucket,
            seat_position,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct CodexpokerIsoEntry {
    obs: i64,
    bucket: i16,
    seat_position: u8,
}

#[derive(Debug, Deserialize)]
struct CodexpokerManifest {
    keys_file: String,
    values_file: String,
    position_aware: Option<bool>,
}

#[derive(Debug, Error)]
pub enum CodexpokerBlueprintError {
    #[error("failed to read `{path}`: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse manifest `{path}`: {source}")]
    ManifestParse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("malformed keys file `{path}`")]
    MalformedKeys { path: PathBuf },
    #[error("malformed values file `{path}`")]
    MalformedValues { path: PathBuf },
    #[error("malformed isomorphism file `{path}`")]
    MalformedIsomorphism { path: PathBuf },
}

fn open_mmap(path: &Path) -> Result<Mmap, CodexpokerBlueprintError> {
    let file = File::open(path).map_err(|source| CodexpokerBlueprintError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    unsafe { Mmap::map(&file) }.map_err(|source| CodexpokerBlueprintError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn load_manifest(path: &Path) -> Result<CodexpokerManifest, CodexpokerBlueprintError> {
    let content = std::fs::read_to_string(path).map_err(|source| CodexpokerBlueprintError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&content).map_err(|source| CodexpokerBlueprintError::ManifestParse {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_value_ranges(
    keys_path: &Path,
    values_path: &Path,
    keys: &Mmap,
    values: &Mmap,
) -> Result<(), CodexpokerBlueprintError> {
    let entries = keys.len() / KEY_ENTRY_SIZE;
    for index in 0..entries {
        let start = index * KEY_ENTRY_SIZE + 18;
        let end = start + 12;
        let bytes =
            keys.get(start..end)
                .ok_or_else(|| CodexpokerBlueprintError::MalformedKeys {
                    path: keys_path.to_path_buf(),
                })?;
        let offset = u64::from_le_bytes(bytes[0..8].try_into().map_err(|_| {
            CodexpokerBlueprintError::MalformedKeys {
                path: keys_path.to_path_buf(),
            }
        })?);
        let len = u32::from_le_bytes(bytes[8..12].try_into().map_err(|_| {
            CodexpokerBlueprintError::MalformedKeys {
                path: keys_path.to_path_buf(),
            }
        })?);
        let start =
            usize::try_from(offset).map_err(|_| CodexpokerBlueprintError::MalformedValues {
                path: values_path.to_path_buf(),
            })?;
        let len = usize::try_from(len).map_err(|_| CodexpokerBlueprintError::MalformedValues {
            path: values_path.to_path_buf(),
        })?;
        let end =
            start
                .checked_add(len)
                .ok_or_else(|| CodexpokerBlueprintError::MalformedValues {
                    path: values_path.to_path_buf(),
                })?;
        let bytes =
            values
                .get(start..end)
                .ok_or_else(|| CodexpokerBlueprintError::MalformedValues {
                    path: values_path.to_path_buf(),
                })?;
        let expected = expected_value_record_len(bytes).ok_or_else(|| {
            CodexpokerBlueprintError::MalformedValues {
                path: values_path.to_path_buf(),
            }
        })?;
        if len != expected {
            return Err(CodexpokerBlueprintError::MalformedValues {
                path: values_path.to_path_buf(),
            });
        }
    }
    Ok(())
}

fn expected_value_record_len(bytes: &[u8]) -> Option<usize> {
    let count = usize::from(u16::from_le_bytes(bytes.get(0..2)?.try_into().ok()?));
    2usize.checked_add(count.checked_mul(VALUE_ENTRY_SIZE)?)
}

fn cmp_key(left: &NlheInfoKey, right: &NlheInfoKey) -> std::cmp::Ordering {
    left.subgame
        .cmp(&right.subgame)
        .then_with(|| left.bucket.cmp(&right.bucket))
        .then_with(|| left.choices.cmp(&right.choices))
}

fn infer_iso_format(len: usize, hint: Option<bool>) -> Option<(usize, bool)> {
    match hint {
        Some(true) => len
            .is_multiple_of(ISO_ENTRY_SIZE_V3)
            .then_some((ISO_ENTRY_SIZE_V3, true)),
        Some(false) => len
            .is_multiple_of(ISO_ENTRY_SIZE_V2)
            .then_some((ISO_ENTRY_SIZE_V2, false)),
        None => {
            let v2 = len.is_multiple_of(ISO_ENTRY_SIZE_V2);
            let v3 = len.is_multiple_of(ISO_ENTRY_SIZE_V3);
            if v2 {
                return Some((ISO_ENTRY_SIZE_V2, false));
            }
            if v3 {
                return Some((ISO_ENTRY_SIZE_V3, true));
            }
            None
        }
    }
}

fn seat_position(position: NlheTablePosition) -> u8 {
    match position {
        NlheTablePosition::Button => 0,
        NlheTablePosition::BigBlind => 1,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::request::NlheHistoryAction;
    use crate::state::{NlheActor, NlhePlayerState, NlheSnapshot, NlheStreet};

    #[test]
    fn codexpoker_blueprint_answers_request() {
        let request = sample_request();
        let observation = request.observation().expect("observation should build");
        let key = request.info_key().expect("info key should build");
        let root = temp_test_dir("codexpoker-blueprint");

        write_codexpoker_fixture(&root, observation, key, Edge::Call, 0.8);

        let blueprint = CodexpokerBlueprint::load(&root).expect("blueprint should load");
        let recommended = blueprint
            .recommend_request(&request)
            .expect("request should resolve a recommendation");

        assert_eq!(recommended, NlheEdge::from(Edge::Call));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn codexpoker_blueprint_rejects_truncated_values_file() {
        let request = sample_request();
        let observation = request.observation().expect("observation should build");
        let key = request.info_key().expect("info key should build");
        let root = temp_test_dir("codexpoker-blueprint-truncated-values");

        write_codexpoker_fixture(&root, observation, key, Edge::Call, 0.8);
        fs::write(root.join("blueprint.values.bin"), [1_u8])
            .expect("truncated values should write");

        let error =
            CodexpokerBlueprint::load(&root).expect_err("truncated values should fail at load");
        assert!(matches!(
            error,
            CodexpokerBlueprintError::MalformedValues { .. }
        ));

        let _ = fs::remove_dir_all(root);
    }

    fn write_codexpoker_fixture(
        root: &Path,
        observation: Observation,
        key: NlheInfoKey,
        edge: Edge,
        policy: f32,
    ) {
        fs::create_dir_all(root).expect("fixture root should create");
        fs::write(
            root.join(MANIFEST_FILE),
            r#"{"keys_file":"blueprint.keys.bin","values_file":"blueprint.values.bin","position_aware":false}"#,
        )
        .expect("manifest should write");

        let iso = i64::from(Isomorphism::from(observation));
        let mut iso_bytes = Vec::new();
        iso_bytes.extend_from_slice(&iso.to_le_bytes());
        iso_bytes.extend_from_slice(&42_i16.to_le_bytes());
        fs::write(root.join(ISOMORPHISM_FILE), iso_bytes).expect("iso should write");

        let mut values = Vec::new();
        values.extend_from_slice(&1_u16.to_le_bytes());
        values.extend_from_slice(&u64::from(edge).to_le_bytes());
        values.extend_from_slice(&policy.to_le_bytes());
        fs::write(root.join("blueprint.values.bin"), &values).expect("values should write");

        let mut keys = Vec::new();
        keys.extend_from_slice(&key.subgame.to_le_bytes());
        keys.extend_from_slice(&key.bucket.to_le_bytes());
        keys.extend_from_slice(&key.choices.to_le_bytes());
        keys.extend_from_slice(&0_u64.to_le_bytes());
        keys.extend_from_slice(&(values.len() as u32).to_le_bytes());
        fs::write(root.join("blueprint.keys.bin"), &keys).expect("keys should write");
    }

    fn sample_request() -> NlheStrategyRequest {
        let snapshot = NlheSnapshot {
            hand_number: 17,
            street: NlheStreet::Preflop,
            pot_bb: 3,
            board: Vec::new(),
            hero_hole: ["Ac".to_string(), "Kh".to_string()],
            action_on: NlheActor::Hero,
            to_call_bb: 1,
            min_raise_to_bb: Some(6),
            legal_actions: vec![],
            hero: NlhePlayerState::new("Hero", NlheTablePosition::Button, 99),
            villain: NlhePlayerState::new("Villain", NlheTablePosition::BigBlind, 98),
        };

        NlheStrategyRequest::from_snapshot(&snapshot, Vec::<NlheHistoryAction>::new(), 42)
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let unique = format!(
            "myosu-codexpoker-{}-{}-{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        std::env::temp_dir().join(unique)
    }
}
