use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use bincode::Options;
use rbp_cards::{Isomorphism, IsomorphismIterator, Observation, Street};
use rbp_gameplay::Abstraction;
use rbp_nlhe::NlheEncoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const MANIFEST_FILE: &str = "manifest.json";
// Local artifact loads must accommodate full NLHE abstraction bundles. This is
// intentionally much larger than the 1 MiB network wire budget because these
// bytes come from operator-owned disk artifacts, not untrusted miner payloads.
const MAX_DECODE_BYTES: u64 = 16 * 1024 * 1024 * 1024;

// Keep the repo-owned bootstrap bundle sampled but cover a few real board classes.
const FLOP_BOOTSTRAP_SAMPLES: [(&str, usize); 24] = [
    ("AcKh~Ts7h2c", 0),
    ("QdJd~Td8d2d", 10),
    ("9c9d~9h7s2d", 20),
    ("7c6c~8d5s2h", 30),
    ("As5s~Ad8c4d", 40),
    ("JhTh~9s8h3c", 50),
    ("6d5d~4s3d2c", 60),
    ("KcQc~JdTc9h", 70),
    ("AhQh~KhJh2s", 80),
    ("Td9d~8c7d6h", 90),
    ("8s8c~Kh8d3s", 100),
    ("5c4c~Ac5d5h", 110),
    ("AdTc~Qh9c8s", 120),
    ("KsJs~QsTs3d", 130),
    ("Qc9c~7c6s2c", 140),
    ("7h7d~AhKdQc", 150),
    ("KhQh~AhTd4c", 160),
    ("9h8h~7h6c2d", 170),
    ("AcJc~Jd6c3c", 180),
    ("QsQh~8d5c5s", 190),
    ("4d4h~5d6s7d", 200),
    ("Kd9d~9c4d2h", 210),
    ("Js8s~Tc9h2s", 220),
    ("Ah7c~Qd7d3s", 230),
];

const TURN_BOOTSTRAP_SAMPLES: [(&str, usize); 24] = [
    ("AcKh~Ts7h2c9d", 0),
    ("QdJd~Td8d2d9c", 10),
    ("9c9d~9h7s2dKc", 20),
    ("7c6c~8d5s2h4c", 30),
    ("As5s~Ad8c4dKs", 40),
    ("JhTh~9s8h3c2h", 50),
    ("6d5d~4s3d2cAh", 60),
    ("KcQc~JdTc9h2s", 70),
    ("AhQh~KhJh2s9s", 80),
    ("Td9d~8c7d6h5c", 90),
    ("8s8c~Kh8d3s2d", 100),
    ("5c4c~Ac5d5hKs", 110),
    ("AdTc~Qh9c8s2h", 120),
    ("KsJs~QsTs3d2c", 130),
    ("Qc9c~7c6s2cKd", 140),
    ("7h7d~AhKdQc2s", 150),
    ("KhQh~AhTd4c2h", 160),
    ("9h8h~7h6c2dAs", 170),
    ("AcJc~Jd6c3c9s", 180),
    ("QsQh~8d5c5s2c", 190),
    ("4d4h~5d6s7dAc", 200),
    ("Kd9d~9c4d2hJh", 210),
    ("Js8s~Tc9h2s7c", 220),
    ("Ah7c~Qd7d3s2h", 230),
];

const RIVER_BOOTSTRAP_SAMPLES: [(&str, usize); 24] = [
    ("AcKh~Ts7h2c9dJc", 50),
    ("QdJd~Td8d2d9cAd", 60),
    ("9c9d~9h7s2dKc2c", 70),
    ("7c6c~8d5s2h4c9h", 80),
    ("As5s~Ad8c4dKs7c", 90),
    ("JhTh~9s8h3c2hKd", 100),
    ("6d5d~4s3d2cAhQh", 110),
    ("KcQc~JdTc9h2sAs", 120),
    ("AhQh~KhJh2s9s3c", 130),
    ("Td9d~8c7d6h5cAs", 140),
    ("8s8c~Kh8d3s2dAc", 150),
    ("5c4c~Ac5d5hKs2c", 160),
    ("AdTc~Qh9c8s2hKd", 170),
    ("KsJs~QsTs3d2cAc", 180),
    ("Qc9c~7c6s2cKdJh", 190),
    ("7h7d~AhKdQc2sJc", 200),
    ("KhQh~AhTd4c2hJd", 210),
    ("9h8h~7h6c2dAs5h", 220),
    ("AcJc~Jd6c3c9sAd", 230),
    ("QsQh~8d5c5s2cKc", 240),
    ("4d4h~5d6s7dAc8d", 250),
    ("Kd9d~9c4d2hJhQc", 260),
    ("Js8s~Tc9h2s7c6s", 270),
    ("Ah7c~Qd7d3s2hKs", 280),
];

/// Named bootstrap scenario used by the dedicated NLHE artifact/query pack proofs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NlheBootstrapScenario {
    pub label: &'static str,
    pub street: NlheAbstractionStreet,
    pub observation: &'static str,
}

const PREFLOP_BOOTSTRAP_SCENARIOS: [NlheBootstrapScenario; 8] = [
    NlheBootstrapScenario {
        label: "preflop_ak_offsuit",
        street: NlheAbstractionStreet::Preflop,
        observation: "AcKh",
    },
    NlheBootstrapScenario {
        label: "preflop_qj_suited",
        street: NlheAbstractionStreet::Preflop,
        observation: "QdJd",
    },
    NlheBootstrapScenario {
        label: "preflop_nines",
        street: NlheAbstractionStreet::Preflop,
        observation: "9c9d",
    },
    NlheBootstrapScenario {
        label: "preflop_suited_connector",
        street: NlheAbstractionStreet::Preflop,
        observation: "7c6c",
    },
    NlheBootstrapScenario {
        label: "preflop_ax_suited",
        street: NlheAbstractionStreet::Preflop,
        observation: "As5s",
    },
    NlheBootstrapScenario {
        label: "preflop_jt_suited",
        street: NlheAbstractionStreet::Preflop,
        observation: "JhTh",
    },
    NlheBootstrapScenario {
        label: "preflop_low_connector",
        street: NlheAbstractionStreet::Preflop,
        observation: "6d5d",
    },
    NlheBootstrapScenario {
        label: "preflop_kq_suited",
        street: NlheAbstractionStreet::Preflop,
        observation: "KcQc",
    },
];

const FLOP_BOOTSTRAP_SCENARIOS: [NlheBootstrapScenario; 24] = [
    NlheBootstrapScenario {
        label: "flop_broadway_two_tone",
        street: NlheAbstractionStreet::Flop,
        observation: "AcKh~Ts7h2c",
    },
    NlheBootstrapScenario {
        label: "flop_monotone_broadway",
        street: NlheAbstractionStreet::Flop,
        observation: "QdJd~Td8d2d",
    },
    NlheBootstrapScenario {
        label: "flop_paired_set_board",
        street: NlheAbstractionStreet::Flop,
        observation: "9c9d~9h7s2d",
    },
    NlheBootstrapScenario {
        label: "flop_open_ender",
        street: NlheAbstractionStreet::Flop,
        observation: "7c6c~8d5s2h",
    },
    NlheBootstrapScenario {
        label: "flop_top_pair_wheel_kicker",
        street: NlheAbstractionStreet::Flop,
        observation: "As5s~Ad8c4d",
    },
    NlheBootstrapScenario {
        label: "flop_combo_draw",
        street: NlheAbstractionStreet::Flop,
        observation: "JhTh~9s8h3c",
    },
    NlheBootstrapScenario {
        label: "flop_wheel_draw",
        street: NlheAbstractionStreet::Flop,
        observation: "6d5d~4s3d2c",
    },
    NlheBootstrapScenario {
        label: "flop_broadway_wrap",
        street: NlheAbstractionStreet::Flop,
        observation: "KcQc~JdTc9h",
    },
    NlheBootstrapScenario {
        label: "flop_royal_draw_two_tone",
        street: NlheAbstractionStreet::Flop,
        observation: "AhQh~KhJh2s",
    },
    NlheBootstrapScenario {
        label: "flop_low_wrap",
        street: NlheAbstractionStreet::Flop,
        observation: "Td9d~8c7d6h",
    },
    NlheBootstrapScenario {
        label: "flop_trips_kicker_race",
        street: NlheAbstractionStreet::Flop,
        observation: "8s8c~Kh8d3s",
    },
    NlheBootstrapScenario {
        label: "flop_paired_ace_board",
        street: NlheAbstractionStreet::Flop,
        observation: "5c4c~Ac5d5h",
    },
    NlheBootstrapScenario {
        label: "flop_broadway_gutshot",
        street: NlheAbstractionStreet::Flop,
        observation: "AdTc~Qh9c8s",
    },
    NlheBootstrapScenario {
        label: "flop_double_broadway_suited",
        street: NlheAbstractionStreet::Flop,
        observation: "KsJs~QsTs3d",
    },
    NlheBootstrapScenario {
        label: "flop_backdoor_flush_pressure",
        street: NlheAbstractionStreet::Flop,
        observation: "Qc9c~7c6s2c",
    },
    NlheBootstrapScenario {
        label: "flop_underpair_overcards",
        street: NlheAbstractionStreet::Flop,
        observation: "7h7d~AhKdQc",
    },
    NlheBootstrapScenario {
        label: "flop_broadway_nut_gutshot",
        street: NlheAbstractionStreet::Flop,
        observation: "KhQh~AhTd4c",
    },
    NlheBootstrapScenario {
        label: "flop_flush_open_ender",
        street: NlheAbstractionStreet::Flop,
        observation: "9h8h~7h6c2d",
    },
    NlheBootstrapScenario {
        label: "flop_top_pair_nut_flush_draw",
        street: NlheAbstractionStreet::Flop,
        observation: "AcJc~Jd6c3c",
    },
    NlheBootstrapScenario {
        label: "flop_overpair_paired_board",
        street: NlheAbstractionStreet::Flop,
        observation: "QsQh~8d5c5s",
    },
    NlheBootstrapScenario {
        label: "flop_pair_plus_flush_draw",
        street: NlheAbstractionStreet::Flop,
        observation: "4d4h~5d6s7d",
    },
    NlheBootstrapScenario {
        label: "flop_top_pair_flush_draw",
        street: NlheAbstractionStreet::Flop,
        observation: "Kd9d~9c4d2h",
    },
    NlheBootstrapScenario {
        label: "flop_gapper_combo_draw",
        street: NlheAbstractionStreet::Flop,
        observation: "Js8s~Tc9h2s",
    },
    NlheBootstrapScenario {
        label: "flop_middle_pair_backdoor",
        street: NlheAbstractionStreet::Flop,
        observation: "Ah7c~Qd7d3s",
    },
];

const TURN_BOOTSTRAP_SCENARIOS: [NlheBootstrapScenario; 24] = [
    NlheBootstrapScenario {
        label: "turn_broadway_two_tone",
        street: NlheAbstractionStreet::Turn,
        observation: "AcKh~Ts7h2c9d",
    },
    NlheBootstrapScenario {
        label: "turn_monotone_broadway",
        street: NlheAbstractionStreet::Turn,
        observation: "QdJd~Td8d2d9c",
    },
    NlheBootstrapScenario {
        label: "turn_paired_set_board",
        street: NlheAbstractionStreet::Turn,
        observation: "9c9d~9h7s2dKc",
    },
    NlheBootstrapScenario {
        label: "turn_open_ender",
        street: NlheAbstractionStreet::Turn,
        observation: "7c6c~8d5s2h4c",
    },
    NlheBootstrapScenario {
        label: "turn_top_pair_wheel_kicker",
        street: NlheAbstractionStreet::Turn,
        observation: "As5s~Ad8c4dKs",
    },
    NlheBootstrapScenario {
        label: "turn_combo_draw",
        street: NlheAbstractionStreet::Turn,
        observation: "JhTh~9s8h3c2h",
    },
    NlheBootstrapScenario {
        label: "turn_wheel_draw",
        street: NlheAbstractionStreet::Turn,
        observation: "6d5d~4s3d2cAh",
    },
    NlheBootstrapScenario {
        label: "turn_broadway_wrap",
        street: NlheAbstractionStreet::Turn,
        observation: "KcQc~JdTc9h2s",
    },
    NlheBootstrapScenario {
        label: "turn_royal_draw_two_tone",
        street: NlheAbstractionStreet::Turn,
        observation: "AhQh~KhJh2s9s",
    },
    NlheBootstrapScenario {
        label: "turn_low_wrap",
        street: NlheAbstractionStreet::Turn,
        observation: "Td9d~8c7d6h5c",
    },
    NlheBootstrapScenario {
        label: "turn_trips_kicker_race",
        street: NlheAbstractionStreet::Turn,
        observation: "8s8c~Kh8d3s2d",
    },
    NlheBootstrapScenario {
        label: "turn_paired_ace_board",
        street: NlheAbstractionStreet::Turn,
        observation: "5c4c~Ac5d5hKs",
    },
    NlheBootstrapScenario {
        label: "turn_broadway_gutshot",
        street: NlheAbstractionStreet::Turn,
        observation: "AdTc~Qh9c8s2h",
    },
    NlheBootstrapScenario {
        label: "turn_double_broadway_suited",
        street: NlheAbstractionStreet::Turn,
        observation: "KsJs~QsTs3d2c",
    },
    NlheBootstrapScenario {
        label: "turn_backdoor_flush_pressure",
        street: NlheAbstractionStreet::Turn,
        observation: "Qc9c~7c6s2cKd",
    },
    NlheBootstrapScenario {
        label: "turn_underpair_overcards",
        street: NlheAbstractionStreet::Turn,
        observation: "7h7d~AhKdQc2s",
    },
    NlheBootstrapScenario {
        label: "turn_broadway_nut_gutshot",
        street: NlheAbstractionStreet::Turn,
        observation: "KhQh~AhTd4c2h",
    },
    NlheBootstrapScenario {
        label: "turn_flush_open_ender",
        street: NlheAbstractionStreet::Turn,
        observation: "9h8h~7h6c2dAs",
    },
    NlheBootstrapScenario {
        label: "turn_top_pair_nut_flush_draw",
        street: NlheAbstractionStreet::Turn,
        observation: "AcJc~Jd6c3c9s",
    },
    NlheBootstrapScenario {
        label: "turn_overpair_paired_board",
        street: NlheAbstractionStreet::Turn,
        observation: "QsQh~8d5c5s2c",
    },
    NlheBootstrapScenario {
        label: "turn_pair_plus_flush_draw",
        street: NlheAbstractionStreet::Turn,
        observation: "4d4h~5d6s7dAc",
    },
    NlheBootstrapScenario {
        label: "turn_top_pair_flush_draw",
        street: NlheAbstractionStreet::Turn,
        observation: "Kd9d~9c4d2hJh",
    },
    NlheBootstrapScenario {
        label: "turn_gapper_combo_draw",
        street: NlheAbstractionStreet::Turn,
        observation: "Js8s~Tc9h2s7c",
    },
    NlheBootstrapScenario {
        label: "turn_middle_pair_backdoor",
        street: NlheAbstractionStreet::Turn,
        observation: "Ah7c~Qd7d3s2h",
    },
];

const RIVER_BOOTSTRAP_SCENARIOS: [NlheBootstrapScenario; 24] = [
    NlheBootstrapScenario {
        label: "river_broadway_two_tone",
        street: NlheAbstractionStreet::River,
        observation: "AcKh~Ts7h2c9dJc",
    },
    NlheBootstrapScenario {
        label: "river_monotone_broadway",
        street: NlheAbstractionStreet::River,
        observation: "QdJd~Td8d2d9cAd",
    },
    NlheBootstrapScenario {
        label: "river_paired_set_board",
        street: NlheAbstractionStreet::River,
        observation: "9c9d~9h7s2dKc2c",
    },
    NlheBootstrapScenario {
        label: "river_open_ender",
        street: NlheAbstractionStreet::River,
        observation: "7c6c~8d5s2h4c9h",
    },
    NlheBootstrapScenario {
        label: "river_top_pair_wheel_kicker",
        street: NlheAbstractionStreet::River,
        observation: "As5s~Ad8c4dKs7c",
    },
    NlheBootstrapScenario {
        label: "river_combo_draw",
        street: NlheAbstractionStreet::River,
        observation: "JhTh~9s8h3c2hKd",
    },
    NlheBootstrapScenario {
        label: "river_wheel_draw",
        street: NlheAbstractionStreet::River,
        observation: "6d5d~4s3d2cAhQh",
    },
    NlheBootstrapScenario {
        label: "river_broadway_wrap",
        street: NlheAbstractionStreet::River,
        observation: "KcQc~JdTc9h2sAs",
    },
    NlheBootstrapScenario {
        label: "river_royal_draw_two_tone",
        street: NlheAbstractionStreet::River,
        observation: "AhQh~KhJh2s9s3c",
    },
    NlheBootstrapScenario {
        label: "river_low_wrap",
        street: NlheAbstractionStreet::River,
        observation: "Td9d~8c7d6h5cAs",
    },
    NlheBootstrapScenario {
        label: "river_trips_kicker_race",
        street: NlheAbstractionStreet::River,
        observation: "8s8c~Kh8d3s2dAc",
    },
    NlheBootstrapScenario {
        label: "river_paired_ace_board",
        street: NlheAbstractionStreet::River,
        observation: "5c4c~Ac5d5hKs2c",
    },
    NlheBootstrapScenario {
        label: "river_broadway_gutshot",
        street: NlheAbstractionStreet::River,
        observation: "AdTc~Qh9c8s2hKd",
    },
    NlheBootstrapScenario {
        label: "river_double_broadway_suited",
        street: NlheAbstractionStreet::River,
        observation: "KsJs~QsTs3d2cAc",
    },
    NlheBootstrapScenario {
        label: "river_backdoor_flush_pressure",
        street: NlheAbstractionStreet::River,
        observation: "Qc9c~7c6s2cKdJh",
    },
    NlheBootstrapScenario {
        label: "river_underpair_overcards",
        street: NlheAbstractionStreet::River,
        observation: "7h7d~AhKdQc2sJc",
    },
    NlheBootstrapScenario {
        label: "river_broadway_nut_gutshot",
        street: NlheAbstractionStreet::River,
        observation: "KhQh~AhTd4c2hJd",
    },
    NlheBootstrapScenario {
        label: "river_flush_open_ender",
        street: NlheAbstractionStreet::River,
        observation: "9h8h~7h6c2dAs5h",
    },
    NlheBootstrapScenario {
        label: "river_top_pair_nut_flush_draw",
        street: NlheAbstractionStreet::River,
        observation: "AcJc~Jd6c3c9sAd",
    },
    NlheBootstrapScenario {
        label: "river_overpair_paired_board",
        street: NlheAbstractionStreet::River,
        observation: "QsQh~8d5c5s2cKc",
    },
    NlheBootstrapScenario {
        label: "river_pair_plus_flush_draw",
        street: NlheAbstractionStreet::River,
        observation: "4d4h~5d6s7dAc8d",
    },
    NlheBootstrapScenario {
        label: "river_top_pair_flush_draw",
        street: NlheAbstractionStreet::River,
        observation: "Kd9d~9c4d2hJhQc",
    },
    NlheBootstrapScenario {
        label: "river_gapper_combo_draw",
        street: NlheAbstractionStreet::River,
        observation: "Js8s~Tc9h2s7c6s",
    },
    NlheBootstrapScenario {
        label: "river_middle_pair_backdoor",
        street: NlheAbstractionStreet::River,
        observation: "Ah7c~Qd7d3s2hKs",
    },
];

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

    pub fn as_str(self) -> &'static str {
        self.file_stem()
    }

    fn file_stem(self) -> &'static str {
        match self {
            Self::Preflop => "preflop",
            Self::Flop => "flop",
            Self::Turn => "turn",
            Self::River => "river",
        }
    }

    pub fn expected_entries(self) -> u64 {
        match self {
            Self::Preflop => Street::Pref.n_isomorphisms() as u64,
            Self::Flop => Street::Flop.n_isomorphisms() as u64,
            Self::Turn => Street::Turn.n_isomorphisms() as u64,
            Self::River => Street::Rive.n_isomorphisms() as u64,
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

impl NlheAbstractionManifest {
    pub fn summary(&self) -> NlheEncoderArtifactSummary {
        let street_entries = self
            .streets
            .iter()
            .map(|(street, entry)| (*street, entry.entries))
            .collect::<BTreeMap<_, _>>();
        let total_entries = street_entries.values().copied().sum();
        let postflop_complete = [
            NlheAbstractionStreet::Flop,
            NlheAbstractionStreet::Turn,
            NlheAbstractionStreet::River,
        ]
        .into_iter()
        .all(|street| {
            street_entries.get(&street).copied().unwrap_or_default() == street.expected_entries()
        });

        NlheEncoderArtifactSummary {
            version: self.version,
            game: self.game.clone(),
            total_sha256: self.total_sha256.clone(),
            street_entries,
            total_entries,
            postflop_complete,
        }
    }
}

/// Operator-facing summary of a manifest-backed artifact set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NlheEncoderArtifactSummary {
    pub version: u32,
    pub game: String,
    pub total_sha256: String,
    pub street_entries: BTreeMap<NlheAbstractionStreet, u64>,
    pub total_entries: u64,
    pub postflop_complete: bool,
}

impl NlheEncoderArtifactSummary {
    pub fn entries_for(&self, street: NlheAbstractionStreet) -> u64 {
        self.street_entries
            .get(&street)
            .copied()
            .unwrap_or_default()
    }

    pub fn preflop_entries(&self) -> u64 {
        self.entries_for(NlheAbstractionStreet::Preflop)
    }

    pub fn is_complete_street(&self, street: NlheAbstractionStreet) -> bool {
        self.entries_for(street) == street.expected_entries()
    }

    pub fn is_sampled_street(&self, street: NlheAbstractionStreet) -> bool {
        let entries = self.entries_for(street);
        entries > 0 && entries < street.expected_entries()
    }

    pub fn available_streets(&self) -> Vec<NlheAbstractionStreet> {
        self.street_entries.keys().copied().collect()
    }

    pub fn complete_streets(&self) -> Vec<NlheAbstractionStreet> {
        NlheAbstractionStreet::ordered()
            .into_iter()
            .filter(|street| self.is_complete_street(*street))
            .collect()
    }

    pub fn sampled_streets(&self) -> Vec<NlheAbstractionStreet> {
        NlheAbstractionStreet::ordered()
            .into_iter()
            .filter(|street| self.is_sampled_street(*street))
            .collect()
    }

    pub fn missing_streets(&self) -> Vec<NlheAbstractionStreet> {
        NlheAbstractionStreet::ordered()
            .into_iter()
            .filter(|street| !self.street_entries.contains_key(street))
            .collect()
    }

    pub fn available_streets_token(&self) -> String {
        street_token(&self.available_streets())
    }

    pub fn complete_streets_token(&self) -> String {
        street_token(&self.complete_streets())
    }

    pub fn sampled_streets_token(&self) -> String {
        street_token(&self.sampled_streets())
    }

    pub fn missing_streets_token(&self) -> String {
        street_token(&self.missing_streets())
    }

    pub fn coverage_token(&self) -> String {
        NlheAbstractionStreet::ordered()
            .into_iter()
            .map(|street| {
                format!(
                    "{}={}/{}",
                    street.as_str(),
                    self.entries_for(street),
                    street.expected_entries()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }
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

impl NlheEncoderArtifactBundle {
    pub fn summary(&self) -> NlheEncoderArtifactSummary {
        self.manifest.summary()
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

/// Repo-owned bootstrap artifact shape used by examples and stage-0 proofs.
pub fn bootstrap_scenarios() -> Vec<NlheBootstrapScenario> {
    PREFLOP_BOOTSTRAP_SCENARIOS
        .into_iter()
        .chain(FLOP_BOOTSTRAP_SCENARIOS)
        .chain(TURN_BOOTSTRAP_SCENARIOS)
        .chain(RIVER_BOOTSTRAP_SCENARIOS)
        .collect()
}

/// Repo-owned bootstrap artifact shape used by examples and stage-0 proofs.
pub fn bootstrap_encoder_streets()
-> BTreeMap<NlheAbstractionStreet, BTreeMap<Isomorphism, Abstraction>> {
    BTreeMap::from([
        (
            NlheAbstractionStreet::Preflop,
            IsomorphismIterator::from(Street::Pref)
                .map(|isomorphism| (isomorphism, Abstraction::from((Street::Pref, 42))))
                .collect(),
        ),
        (
            NlheAbstractionStreet::Flop,
            representative_lookups(Street::Flop, &FLOP_BOOTSTRAP_SAMPLES),
        ),
        (
            NlheAbstractionStreet::Turn,
            representative_lookups(Street::Turn, &TURN_BOOTSTRAP_SAMPLES),
        ),
        (
            NlheAbstractionStreet::River,
            representative_lookups(Street::Rive, &RIVER_BOOTSTRAP_SAMPLES),
        ),
    ])
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

fn representative_lookups(
    street: Street,
    entries: &[(&str, usize)],
) -> BTreeMap<Isomorphism, Abstraction> {
    entries
        .iter()
        .map(|(raw, bucket)| {
            let observation = Observation::try_from(*raw)
                .unwrap_or_else(|error| panic!("bootstrap observation should parse: {error}"));
            (
                Isomorphism::from(observation),
                Abstraction::from((street, *bucket)),
            )
        })
        .collect()
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

fn street_token(streets: &[NlheAbstractionStreet]) -> String {
    if streets.is_empty() {
        return "none".to_string();
    }

    streets
        .iter()
        .map(|street| street.as_str())
        .collect::<Vec<_>>()
        .join(",")
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
    fn manifest_summary_reports_sparse_bootstrap_shape() {
        let manifest = NlheAbstractionManifest {
            version: 1,
            game: "nlhe_hu".to_string(),
            streets: BTreeMap::from([(
                NlheAbstractionStreet::Preflop,
                NlheAbstractionArtifactEntry {
                    file: "preflop.lookup.bin".to_string(),
                    entries: 169,
                    sha256: "deadbeef".to_string(),
                },
            )]),
            total_sha256: "beadfeed".to_string(),
        };

        let summary = manifest.summary();

        assert_eq!(summary.version, 1);
        assert_eq!(summary.game, "nlhe_hu");
        assert_eq!(summary.preflop_entries(), 169);
        assert_eq!(summary.total_entries, 169);
        assert!(!summary.postflop_complete);
        assert_eq!(summary.available_streets_token(), "preflop");
        assert_eq!(summary.complete_streets_token(), "preflop");
        assert_eq!(summary.sampled_streets_token(), "none");
        assert_eq!(summary.missing_streets_token(), "flop,turn,river");
        assert_eq!(
            summary.coverage_token(),
            "preflop=169/169,flop=0/1286792,turn=0/13960050,river=0/123156254"
        );
    }

    #[test]
    fn bootstrap_scenarios_cover_every_bootstrap_street() {
        let scenarios = bootstrap_scenarios();

        assert_eq!(scenarios.len(), 80);
        assert_eq!(
            scenarios
                .iter()
                .filter(|scenario| scenario.street == NlheAbstractionStreet::Preflop)
                .count(),
            8
        );
        assert_eq!(
            scenarios
                .iter()
                .filter(|scenario| scenario.street == NlheAbstractionStreet::Flop)
                .count(),
            24
        );
        assert_eq!(
            scenarios
                .iter()
                .filter(|scenario| scenario.street == NlheAbstractionStreet::Turn)
                .count(),
            24
        );
        assert_eq!(
            scenarios
                .iter()
                .filter(|scenario| scenario.street == NlheAbstractionStreet::River)
                .count(),
            24
        );
    }

    #[test]
    fn bootstrap_encoder_streets_report_sampled_postflop_shape() {
        let directory = unique_artifact_dir();
        let manifest = write_encoder_dir(&directory, bootstrap_encoder_streets())
            .expect("bootstrap encoder dir should write");
        let summary = manifest.summary();

        assert_eq!(summary.available_streets_token(), "preflop,flop,turn,river");
        assert_eq!(summary.complete_streets_token(), "preflop");
        assert_eq!(summary.sampled_streets_token(), "flop,turn,river");
        assert_eq!(summary.missing_streets_token(), "none");
        assert!(!summary.postflop_complete);
        assert_eq!(summary.preflop_entries(), 169);
        assert_eq!(summary.entries_for(NlheAbstractionStreet::Flop), 24);
        assert_eq!(summary.entries_for(NlheAbstractionStreet::Turn), 24);
        assert_eq!(summary.entries_for(NlheAbstractionStreet::River), 24);
        assert_eq!(summary.total_entries, 241);
        assert_eq!(
            summary.coverage_token(),
            "preflop=169/169,flop=24/1286792,turn=24/13960050,river=24/123156254"
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

    #[test]
    fn artifact_decode_budget_leaves_room_for_full_river_files() {
        assert!(
            MAX_DECODE_BYTES >= 4 * 1024 * 1024 * 1024,
            "full river abstraction files are multi-gigabyte and should fit under the local artifact budget"
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
