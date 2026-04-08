use std::collections::BTreeMap;
use std::io::BufRead;
use std::path::Path;

use rbp_cards::Isomorphism;
use rbp_cards::Observation;
use rbp_cards::Street;
use rbp_gameplay::Abstraction;
use thiserror::Error;

use crate::ArtifactCodecError;
use crate::NlheAbstractionManifest;
use crate::NlheAbstractionStreet;
use crate::write_encoder_dir;

/// Errors returned while importing a robopoker `isomorphism` dump.
#[derive(Debug, Error)]
pub enum LookupDumpError {
    #[error("failed to read lookup dump line {line}: {source}")]
    ReadLine {
        line: usize,
        #[source]
        source: std::io::Error,
    },
    #[error("lookup dump line {line} must contain exactly two tab-separated columns")]
    MissingColumns { line: usize },
    #[error("lookup dump line {line} has invalid obs `{value}`: {source}")]
    InvalidObservation {
        line: usize,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("lookup dump line {line} has invalid abs `{value}`: {source}")]
    InvalidAbstraction {
        line: usize,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("failed to write encoder directory `{path}`: {source}")]
    WriteEncoder {
        path: String,
        #[source]
        source: ArtifactCodecError,
    },
}

/// Write a manifest-backed encoder directory from a tab-separated lookup dump.
pub fn write_encoder_dir_from_lookup_dump<R>(
    reader: R,
    directory: impl AsRef<Path>,
) -> Result<NlheAbstractionManifest, LookupDumpError>
where
    R: BufRead,
{
    let mut streets = BTreeMap::new();

    for (index, line_result) in reader.lines().enumerate() {
        let line_number = index + 1;
        let line = line_result.map_err(|source| LookupDumpError::ReadLine {
            line: line_number,
            source,
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut columns = trimmed.split('\t');
        let obs_value = columns
            .next()
            .ok_or(LookupDumpError::MissingColumns { line: line_number })?;
        let abs_value = columns
            .next()
            .ok_or(LookupDumpError::MissingColumns { line: line_number })?;
        if columns.next().is_some() {
            return Err(LookupDumpError::MissingColumns { line: line_number });
        }

        let observation = obs_value
            .parse::<i64>()
            .map(Observation::from)
            .map_err(|source| LookupDumpError::InvalidObservation {
                line: line_number,
                value: obs_value.to_string(),
                source,
            })?;
        let abstraction = abs_value
            .parse::<i16>()
            .map(Abstraction::from)
            .map_err(|source| LookupDumpError::InvalidAbstraction {
                line: line_number,
                value: abs_value.to_string(),
                source,
            })?;
        let isomorphism = Isomorphism::from(observation);
        streets
            .entry(map_street(observation.street()))
            .or_insert_with(BTreeMap::new)
            .insert(isomorphism, abstraction);
    }

    let directory = directory.as_ref();
    write_encoder_dir(directory, streets).map_err(|source| LookupDumpError::WriteEncoder {
        path: directory.display().to_string(),
        source,
    })
}

fn map_street(street: Street) -> NlheAbstractionStreet {
    match street {
        Street::Pref => NlheAbstractionStreet::Preflop,
        Street::Flop => NlheAbstractionStreet::Flop,
        Street::Turn => NlheAbstractionStreet::Turn,
        Street::Rive => NlheAbstractionStreet::River,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rbp_cards::Observation;

    use crate::load_encoder_bundle;

    use super::*;

    #[test]
    fn lookup_dump_writes_manifest_backed_encoder_dir() {
        let directory = unique_artifact_dir();
        let preflop = Observation::try_from("AcKh").expect("preflop observation should parse");
        let flop = Observation::try_from("AcKh~Ts7h2c").expect("flop observation should parse");
        let dump = format!(
            "{}\t{}\n{}\t{}\n",
            i64::from(preflop),
            42,
            i64::from(flop),
            87
        );

        let manifest = write_encoder_dir_from_lookup_dump(Cursor::new(dump), &directory)
            .expect("lookup dump should import");
        let bundle = load_encoder_bundle(&directory).expect("bundle should load");

        assert_eq!(bundle.manifest, manifest);
        assert_eq!(i16::from(bundle.encoder.abstraction(&preflop)), 42);
        assert_eq!(i16::from(bundle.encoder.abstraction(&flop)), 87);

        fs::remove_dir_all(&directory).expect("artifact dir should clean up");
    }

    #[test]
    fn lookup_dump_rejects_bad_rows() {
        let directory = unique_artifact_dir();
        let error =
            write_encoder_dir_from_lookup_dump(Cursor::new("not-a-number\t42\n"), &directory)
                .expect_err("bad row should fail");

        assert!(matches!(error, LookupDumpError::InvalidObservation { .. }));
    }

    fn unique_artifact_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("myosu-poker-lookup-{nanos}"))
    }
}
