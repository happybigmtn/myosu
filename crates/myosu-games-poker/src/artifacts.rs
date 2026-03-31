use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use bincode::Options;
use rbp_cards::Isomorphism;
use rbp_gameplay::Abstraction;
use rbp_nlhe::NlheEncoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const MANIFEST_FILE: &str = "manifest.json";
const MAX_DECODE_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct EncoderLookupArtifact(BTreeMap<Isomorphism, Abstraction>);

/// Street-scoped artifact entry in an abstraction manifest.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NlheAbstractionStreet {
    Preflop,
    Flop,
    Turn,
    River,
}

impl NlheAbstractionStreet {
    fn ordered() -> [Self; 4] {
        [Self::Preflop, Self::Flop, Self::Turn, Self::River]
    }

    fn file_stem(self) -> &'static str {
        match self {
            Self::Preflop => "preflop",
            Self::Flop => "flop",
            Self::Turn => "turn",
            Self::River => "river",
        }
    }
}

/// One file entry in an abstraction manifest.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NlheAbstractionArtifactEntry {
    pub file: String,
    pub entries: u64,
    pub sha256: String,
}

/// Manifest describing a versioned abstraction artifact set.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NlheAbstractionManifest {
    pub version: u32,
    pub game: String,
    pub streets: BTreeMap<NlheAbstractionStreet, NlheAbstractionArtifactEntry>,
    pub total_sha256: String,
}

/// Verified abstraction bundle loaded from a manifest-backed directory.
pub struct NlheEncoderArtifactBundle {
    pub encoder: NlheEncoder,
    pub manifest: NlheAbstractionManifest,
    pub total_sha256: String,
}

impl std::fmt::Debug for NlheEncoderArtifactBundle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NlheEncoderArtifactBundle")
            .field("manifest", &self.manifest)
            .field("total_sha256", &self.total_sha256)
            .finish()
    }
}

/// Error returned when loading or encoding local poker artifacts.
#[derive(Debug, Error)]
pub enum ArtifactCodecError {
    #[error("failed to encode {context}: {source}")]
    Encode {
        context: &'static str,
        #[source]
        source: bincode::Error,
    },
    #[error("failed to decode {context}: {source}")]
    Decode {
        context: &'static str,
        #[source]
        source: bincode::Error,
    },
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse manifest `{path}`: {source}")]
    ManifestParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write {path}: {source}")]
    Write {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("manifest `{path}` does not list any abstraction files")]
    EmptyManifest { path: String },
    #[error("artifact file `{path}` hash mismatch: expected {expected}, got {actual}")]
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    #[error("manifest total hash mismatch: expected {expected}, got {actual}")]
    TotalHashMismatch { expected: String, actual: String },
    #[error("artifact `{path}` entry count mismatch: expected {expected}, got {actual}")]
    EntryCountMismatch {
        path: String,
        expected: u64,
        actual: u64,
    },
    #[error("duplicate isomorphism found while merging `{path}`")]
    DuplicateIsomorphism { path: String },
}

/// Encode an `rbp_nlhe::NlheEncoder` into its binary artifact form.
pub fn encode_encoder(encoder: &NlheEncoder) -> Result<Vec<u8>, ArtifactCodecError> {
    encode_codec()
        .serialize(encoder)
        .map_err(|source| ArtifactCodecError::Encode {
            context: "nlhe encoder",
            source,
        })
}

/// Decode an `rbp_nlhe::NlheEncoder` from its binary artifact form.
pub fn decode_encoder(bytes: &[u8]) -> Result<NlheEncoder, ArtifactCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize(bytes)
        .map_err(|source| ArtifactCodecError::Decode {
            context: "nlhe encoder",
            source,
        })
}

/// Load and parse an abstraction manifest from disk.
fn load_manifest(path: impl AsRef<Path>) -> Result<NlheAbstractionManifest, ArtifactCodecError> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|source| ArtifactCodecError::Read {
        path: path.display().to_string(),
        source,
    })?;

    serde_json::from_slice(&bytes).map_err(|source| ArtifactCodecError::ManifestParse {
        path: path.display().to_string(),
        source,
    })
}

/// Load and verify a manifest-backed directory of abstraction artifacts.
pub fn load_encoder_bundle(
    directory: impl AsRef<Path>,
) -> Result<NlheEncoderArtifactBundle, ArtifactCodecError> {
    let directory = directory.as_ref();
    let manifest_path = directory.join(MANIFEST_FILE);
    let manifest = load_manifest(&manifest_path)?;

    if manifest.streets.is_empty() {
        return Err(ArtifactCodecError::EmptyManifest {
            path: manifest_path.display().to_string(),
        });
    }

    let mut merged = BTreeMap::new();
    let mut file_hashes = Vec::new();

    for street in NlheAbstractionStreet::ordered() {
        let Some(entry) = manifest.streets.get(&street) else {
            continue;
        };

        let file_path = directory.join(&entry.file);
        let bytes = fs::read(&file_path).map_err(|source| ArtifactCodecError::Read {
            path: file_path.display().to_string(),
            source,
        })?;
        let actual_hash = sha256_hex(&bytes);
        let expected_hash = normalized_hex(&entry.sha256);

        if actual_hash != expected_hash {
            return Err(ArtifactCodecError::HashMismatch {
                path: file_path.display().to_string(),
                expected: expected_hash,
                actual: actual_hash,
            });
        }

        let lookup = decode_lookup(&bytes)?;
        let actual_entries = lookup.len() as u64;
        if actual_entries != entry.entries {
            return Err(ArtifactCodecError::EntryCountMismatch {
                path: file_path.display().to_string(),
                expected: entry.entries,
                actual: actual_entries,
            });
        }

        for (isomorphism, abstraction) in lookup {
            if merged.insert(isomorphism, abstraction).is_some() {
                return Err(ArtifactCodecError::DuplicateIsomorphism {
                    path: file_path.display().to_string(),
                });
            }
        }

        file_hashes.push(expected_hash);
    }

    let actual_total = total_sha256(&file_hashes);
    let expected_total = normalized_hex(&manifest.total_sha256);
    if actual_total != expected_total {
        return Err(ArtifactCodecError::TotalHashMismatch {
            expected: expected_total,
            actual: actual_total,
        });
    }

    Ok(NlheEncoderArtifactBundle {
        encoder: encoder_from_lookup(merged)?,
        manifest,
        total_sha256: actual_total,
    })
}

/// Load only the encoder from a manifest-backed directory.
pub fn load_encoder_dir(directory: impl AsRef<Path>) -> Result<NlheEncoder, ArtifactCodecError> {
    load_encoder_bundle(directory).map(|bundle| bundle.encoder)
}

/// Write a manifest-backed directory of abstraction artifacts from street lookups.
pub fn write_encoder_dir(
    directory: impl AsRef<Path>,
    streets: BTreeMap<NlheAbstractionStreet, BTreeMap<Isomorphism, Abstraction>>,
) -> Result<NlheAbstractionManifest, ArtifactCodecError> {
    let directory = directory.as_ref();
    fs::create_dir_all(directory).map_err(|source| ArtifactCodecError::Write {
        path: directory.display().to_string(),
        source,
    })?;

    let mut manifest_entries = BTreeMap::new();
    let mut file_hashes = Vec::new();

    for street in NlheAbstractionStreet::ordered() {
        let Some(lookup) = streets.get(&street) else {
            continue;
        };

        let encoder = encoder_from_lookup(lookup.clone())?;
        let bytes = encode_encoder(&encoder)?;
        let hash = sha256_hex(&bytes);
        let file = format!("{}.lookup.bin", street.file_stem());
        let path = directory.join(&file);

        fs::write(&path, &bytes).map_err(|source| ArtifactCodecError::Write {
            path: path.display().to_string(),
            source,
        })?;

        manifest_entries.insert(
            street,
            NlheAbstractionArtifactEntry {
                file,
                entries: lookup.len() as u64,
                sha256: hash.clone(),
            },
        );
        file_hashes.push(hash);
    }

    let manifest = NlheAbstractionManifest {
        version: 1,
        game: "nlhe_hu".to_string(),
        streets: manifest_entries,
        total_sha256: total_sha256(&file_hashes),
    };
    let manifest_path = directory.join(MANIFEST_FILE);
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|source| {
        ArtifactCodecError::ManifestParse {
            path: manifest_path.display().to_string(),
            source,
        }
    })?;
    fs::write(&manifest_path, manifest_bytes).map_err(|source| ArtifactCodecError::Write {
        path: manifest_path.display().to_string(),
        source,
    })?;

    Ok(manifest)
}

/// Build an encoder from a raw isomorphism lookup map.
///
/// This is a local compatibility helper while upstream constructor support is
/// still missing. The resulting bytes match the same `serde`/`bincode`
/// representation used by `NlheEncoder`.
pub fn encoder_from_lookup(
    lookup: BTreeMap<Isomorphism, Abstraction>,
) -> Result<NlheEncoder, ArtifactCodecError> {
    let bytes = encode_codec()
        .serialize(&EncoderLookupArtifact(lookup))
        .map_err(|source| ArtifactCodecError::Encode {
            context: "nlhe encoder lookup",
            source,
        })?;

    decode_encoder(&bytes)
}

fn decode_lookup(bytes: &[u8]) -> Result<BTreeMap<Isomorphism, Abstraction>, ArtifactCodecError> {
    decode_codec(MAX_DECODE_BYTES)
        .deserialize::<EncoderLookupArtifact>(bytes)
        .map(|artifact| artifact.0)
        .map_err(|source| ArtifactCodecError::Decode {
            context: "nlhe encoder lookup",
            source,
        })
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

fn normalized_hex(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn total_sha256(file_hashes: &[String]) -> String {
    let mut hasher = Sha256::new();

    for hash in file_hashes {
        hasher.update(hash.as_bytes());
    }

    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::time::{SystemTime, UNIX_EPOCH};

    use proptest::prelude::*;
    use rbp_cards::Observation;

    use super::*;

    #[test]
    fn encoder_roundtrips_through_bincode() {
        let observation = Observation::try_from("AcKh").expect("observation should parse");
        let encoder = encoder_from_lookup(BTreeMap::from([(
            Isomorphism::from(observation),
            Abstraction::from(42_i16),
        )]))
        .expect("encoder should build");

        let encoded = encode_encoder(&encoder).expect("encoder should encode");
        let decoded = decode_encoder(&encoded).expect("encoder should decode");

        assert_eq!(
            i16::from(decoded.abstraction(&observation)),
            42,
            "decoded encoder should preserve lookup"
        );
    }

    #[test]
    fn manifest_bundle_loads_and_merges_lookup_files() {
        let directory = unique_artifact_dir();
        fs::create_dir_all(&directory).expect("artifact dir should create");

        let preflop_observation =
            Observation::try_from("AcKh").expect("preflop observation should parse");
        let flop_observation =
            Observation::try_from("AcKh~Ts7h2c").expect("flop observation should parse");

        let preflop_bytes = lookup_bytes(BTreeMap::from([(
            Isomorphism::from(preflop_observation),
            Abstraction::from(42_i16),
        )]));
        let flop_bytes = lookup_bytes(BTreeMap::from([(
            Isomorphism::from(flop_observation),
            Abstraction::from(87_i16),
        )]));

        let preflop_file = "preflop.lookup.bin";
        let flop_file = "flop.lookup.bin";
        fs::write(directory.join(preflop_file), &preflop_bytes)
            .expect("preflop bytes should write");
        fs::write(directory.join(flop_file), &flop_bytes).expect("flop bytes should write");

        let preflop_hash = sha256_hex(&preflop_bytes);
        let flop_hash = sha256_hex(&flop_bytes);
        let manifest = NlheAbstractionManifest {
            version: 1,
            game: "nlhe_hu".to_string(),
            streets: BTreeMap::from([
                (
                    NlheAbstractionStreet::Preflop,
                    NlheAbstractionArtifactEntry {
                        file: preflop_file.to_string(),
                        entries: 1,
                        sha256: preflop_hash.clone(),
                    },
                ),
                (
                    NlheAbstractionStreet::Flop,
                    NlheAbstractionArtifactEntry {
                        file: flop_file.to_string(),
                        entries: 1,
                        sha256: flop_hash.clone(),
                    },
                ),
            ]),
            total_sha256: total_sha256(&[preflop_hash.clone(), flop_hash.clone()]),
        };
        fs::write(
            directory.join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");

        let bundle = load_encoder_bundle(&directory).expect("artifact bundle should load");

        assert_eq!(bundle.total_sha256, manifest.total_sha256);
        assert_eq!(bundle.manifest, manifest);
        assert_eq!(
            i16::from(bundle.encoder.abstraction(&preflop_observation)),
            42
        );
        assert_eq!(i16::from(bundle.encoder.abstraction(&flop_observation)), 87);

        fs::remove_dir_all(&directory).expect("artifact dir should clean up");
    }

    #[test]
    fn write_encoder_dir_roundtrips_through_manifest_loader() {
        let directory = unique_artifact_dir();
        let preflop_observation =
            Observation::try_from("AcKh").expect("preflop observation should parse");
        let streets = BTreeMap::from([(
            NlheAbstractionStreet::Preflop,
            BTreeMap::from([(
                Isomorphism::from(preflop_observation),
                Abstraction::from(42_i16),
            )]),
        )]);

        let manifest = write_encoder_dir(&directory, streets).expect("artifact dir should write");
        let bundle = load_encoder_bundle(&directory).expect("written artifact dir should load");

        assert_eq!(bundle.manifest, manifest);
        assert_eq!(
            i16::from(bundle.encoder.abstraction(&preflop_observation)),
            42
        );

        fs::remove_dir_all(&directory).expect("artifact dir should clean up");
    }

    #[test]
    fn manifest_bundle_rejects_tampered_file() {
        let directory = unique_artifact_dir();
        fs::create_dir_all(&directory).expect("artifact dir should create");

        let bytes = lookup_bytes(BTreeMap::from([(
            Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
            Abstraction::from(42_i16),
        )]));
        let file = "preflop.lookup.bin";
        fs::write(directory.join(file), &bytes).expect("artifact bytes should write");

        let manifest = NlheAbstractionManifest {
            version: 1,
            game: "nlhe_hu".to_string(),
            streets: BTreeMap::from([(
                NlheAbstractionStreet::Preflop,
                NlheAbstractionArtifactEntry {
                    file: file.to_string(),
                    entries: 1,
                    sha256: "deadbeef".to_string(),
                },
            )]),
            total_sha256: total_sha256(&["deadbeef".to_string()]),
        };
        fs::write(
            directory.join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");

        let error = load_encoder_bundle(&directory).expect_err("tampered file should fail");
        assert!(matches!(error, ArtifactCodecError::HashMismatch { .. }));

        fs::remove_dir_all(&directory).expect("artifact dir should clean up");
    }

    #[test]
    fn manifest_bundle_rejects_total_hash_mismatch() {
        let directory = unique_artifact_dir();
        fs::create_dir_all(&directory).expect("artifact dir should create");

        let bytes = lookup_bytes(BTreeMap::from([(
            Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
            Abstraction::from(42_i16),
        )]));
        let file = "preflop.lookup.bin";
        let file_hash = sha256_hex(&bytes);
        fs::write(directory.join(file), &bytes).expect("artifact bytes should write");

        let manifest = NlheAbstractionManifest {
            version: 1,
            game: "nlhe_hu".to_string(),
            streets: BTreeMap::from([(
                NlheAbstractionStreet::Preflop,
                NlheAbstractionArtifactEntry {
                    file: file.to_string(),
                    entries: 1,
                    sha256: file_hash,
                },
            )]),
            total_sha256: "nottherighthash".to_string(),
        };
        fs::write(
            directory.join(MANIFEST_FILE),
            serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
        )
        .expect("manifest should write");

        let error = load_encoder_bundle(&directory).expect_err("bad total hash should fail");
        assert!(matches!(
            error,
            ArtifactCodecError::TotalHashMismatch { .. }
        ));

        fs::remove_dir_all(&directory).expect("artifact dir should clean up");
    }

    #[test]
    fn decode_codec_carries_a_real_byte_limit() {
        let encoder = encoder_from_lookup(BTreeMap::from([(
            Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
            Abstraction::from(42_i16),
        )]))
        .expect("encoder should build");
        let result = decode_codec(0).serialized_size(&encoder);

        assert!(
            result.is_err(),
            "bounded codec should reject over-budget values"
        );
    }

    proptest! {
        #[test]
        fn prop_decode_encoder_rejects_truncated_payloads(trim_seed in any::<usize>()) {
            let bytes = lookup_bytes(BTreeMap::from([(
                Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
                Abstraction::from(42_i16),
            )]));
            let trim = trim_seed % bytes.len();
            let truncated = &bytes[..trim];

            let result = catch_unwind(AssertUnwindSafe(|| decode_encoder(truncated)));

            prop_assert!(result.is_ok());
            prop_assert!(result.expect("decode should not panic").is_err());
        }

        #[test]
        fn prop_manifest_bundle_rejects_tampered_file(
            byte_index in 0usize..256,
            replacement in any::<u8>(),
        ) {
            let directory = unique_artifact_dir();
            fs::create_dir_all(&directory).expect("artifact dir should create");

            let mut bytes = lookup_bytes(BTreeMap::from([(
                Isomorphism::from(Observation::try_from("AcKh").expect("observation should parse")),
                Abstraction::from(42_i16),
            )]));
            prop_assume!(!bytes.is_empty());
            let index = byte_index % bytes.len();
            prop_assume!(replacement != bytes[index]);
            let expected_hash = sha256_hex(&bytes);
            bytes[index] = replacement;

            let file = "preflop.lookup.bin";
            fs::write(directory.join(file), &bytes).expect("artifact bytes should write");
            let manifest = NlheAbstractionManifest {
                version: 1,
                game: "nlhe_hu".to_string(),
                streets: BTreeMap::from([(
                    NlheAbstractionStreet::Preflop,
                    NlheAbstractionArtifactEntry {
                        file: file.to_string(),
                        entries: 1,
                        sha256: expected_hash.clone(),
                    },
                )]),
                total_sha256: total_sha256(&[expected_hash]),
            };
            fs::write(
                directory.join(MANIFEST_FILE),
                serde_json::to_vec_pretty(&manifest).expect("manifest should serialize"),
            )
            .expect("manifest should write");

            let result = catch_unwind(AssertUnwindSafe(|| load_encoder_bundle(&directory)));
            let _ = fs::remove_dir_all(&directory);

            prop_assert!(result.is_ok());
            prop_assert!(
                matches!(
                    result.expect("load should not panic"),
                    Err(ArtifactCodecError::HashMismatch { .. })
                ),
                "tampered artifact should fail with hash mismatch"
            );
        }
    }

    fn lookup_bytes(lookup: BTreeMap<Isomorphism, Abstraction>) -> Vec<u8> {
        let encoder = encoder_from_lookup(lookup).expect("encoder should build");
        encode_encoder(&encoder).expect("encoder should encode")
    }

    fn unique_artifact_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("myosu-nlhe-artifacts-{nanos}"))
    }
}
