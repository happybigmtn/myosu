//! `myosu-solver-read-dedicated` — read-only, JSON-in / line-out solver
//! surface for the two dedicated-solver games.
//!
//! W-07 sibling to the W-03 `myosu-solver-read` portfolio binary: that
//! binary explicitly rejects `NlheHeadsUp` and `LiarsDice` because they
//! have dedicated solver crates (not the rule-aware engine portfolio
//! pattern). This crate fills the gap with a parallel read surface for
//! the two dedicated games. The line protocol mirrors the W-03 shape
//! (`SOLVER_READ ...` / `SOLVER_READ_FAIL reason=...`) so a wrapper
//! script can `grep ^SOLVER_READ` across both binaries identically.
//!
//! Read-only by design: no chain RPC, no wallet, no bundle emission,
//! no operator registration, no token requirement. The only state
//! touch is the read-only `fs::read` of the supplied checkpoint
//! (Liar's Dice) or the supplied checkpoint + encoder directory (NLHE).

use std::fmt;
use std::fs;
use std::path::Path;

use myosu_games_liars_dice::{
    LiarsDiceSolver, LiarsDiceStrategyQuery, LiarsDiceStrategyResponse, recommended_edge,
};
use myosu_games_poker::{
    NlheStrategyQuery, NlheStrategyResponse, PokerSolver, RbpNlheEncoder as NlheEncoder,
    load_encoder_dir,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// The two dedicated-solver games this crate serves.
///
/// W-07 deliberately limits the surface to the two `!is_portfolio_routed`
/// slugs the W-03 binary explicitly rejects: `liars-dice` and
/// `nlhe-heads-up`. A future expansion to additional dedicated games
/// would land as a new variant here and a new dispatch arm in
/// `answer_for_game`, not a widening of this enum's contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DedicatedGame {
    /// `liars-dice` — the Liar's Dice dedicated solver crate.
    LiarsDice,
    /// `nlhe-heads-up` — the NLHE dedicated solver crate.
    NlheHeadsUp,
}

impl DedicatedGame {
    /// Inverse of [`DedicatedGame::slug`]. Returns `None` for any
    /// string that is not one of the two documented dedicated slugs
    /// (including the empty / whitespace case, which fails closed
    /// the same way the W-03 `empty_game_slug` branch does).
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug.trim() {
            "liars-dice" => Some(Self::LiarsDice),
            "nlhe-heads-up" => Some(Self::NlheHeadsUp),
            _ => None,
        }
    }

    /// The stable slug this game emits on the `SOLVER_READ game=...`
    /// line and accepts on the input JSON's `game` field.
    pub fn slug(self) -> &'static str {
        match self {
            Self::LiarsDice => "liars-dice",
            Self::NlheHeadsUp => "nlhe-heads-up",
        }
    }
}

/// Every error the binary can surface on the `SOLVER_READ_FAIL reason=...`
/// line. The `reason()` method maps every variant to a stable,
/// grep-friendly reason token; a wrapper script can grep for the exact
/// reason without parsing free-form text.
#[derive(Debug, Error)]
pub enum DedicatedReadError {
    /// `game` slug did not match a known dedicated game.
    #[error("unknown_game: {0}")]
    UnknownGame(String),
    /// `checkpoint` field was missing from the JSON request.
    #[error("missing_checkpoint_field")]
    MissingCheckpointField,
    /// `checkpoint` field pointed at an empty file.
    #[error("empty_checkpoint: {0}")]
    EmptyCheckpoint(String),
    /// `encoder_dir` field was missing for an NLHE request.
    #[error("missing_encoder_dir")]
    MissingEncoderDir,
    /// `encoder_dir` (Liar's Dice: `checkpoint`) was not a directory
    /// (or for Liar's Dice: a file at that path did not exist).
    #[error("not_a_directory: {0}")]
    NotADirectory(String),
    /// Checkpoint magic did not match the expected `MYOS` header.
    #[error("checkpoint_magic: found={0}")]
    CheckpointMagic(String),
    /// Checkpoint version was not the expected `1`.
    #[error("checkpoint_version: found={found} expected={expected}")]
    CheckpointVersion { found: u32, expected: u32 },
    /// Encoder directory failed to load via `load_encoder_dir`.
    #[error("encoder_load: {0}")]
    EncoderLoad(String),
    /// Solver checkpoint failed to load via `LiarsDiceSolver::load` /
    /// `PokerSolver::load`.
    #[error("solver_load: {0}")]
    SolverLoad(String),
    /// The query payload failed to deserialize.
    #[error("query_decode: {0}")]
    Query(String),
    /// The solver answered with an empty / no-action response.
    #[error("empty_recommendation")]
    EmptyRecommendation,
    /// The recommended action carried a non-finite confidence value.
    #[error("non_finite_confidence: {0}")]
    NonFiniteConfidence(f32),
    /// An I/O error escaped the typed-error path.
    #[error("io: {0}")]
    Io(String),
}

impl DedicatedReadError {
    /// Stable, grep-friendly reason token for the `SOLVER_READ_FAIL
    /// reason=...` line. Mirrors the W-03 binary's `fail()` helper so
    /// a wrapper script can `grep ^SOLVER_READ_FAIL reason=...` over
    /// the union of the two binaries and triage identically.
    pub fn reason(&self) -> String {
        match self {
            Self::UnknownGame(slug) => format!("unknown_game: {slug}"),
            Self::MissingCheckpointField => "missing_checkpoint_field".to_string(),
            Self::EmptyCheckpoint(path) => format!("empty_checkpoint: {path}"),
            Self::MissingEncoderDir => "missing_encoder_dir".to_string(),
            Self::NotADirectory(path) => format!("not_a_directory: {path}"),
            Self::CheckpointMagic(found) => format!("checkpoint_magic: {found}"),
            Self::CheckpointVersion { found, expected } => {
                format!("checkpoint_version: found={found} expected={expected}")
            }
            Self::EncoderLoad(detail) => format!("encoder_load: {detail}"),
            Self::SolverLoad(detail) => format!("solver_load: {detail}"),
            Self::Query(detail) => format!("query_decode: {detail}"),
            Self::EmptyRecommendation => "empty_recommendation".to_string(),
            Self::NonFiniteConfidence(c) => format!("non_finite_confidence: {c}"),
            Self::Io(detail) => format!("io: {detail}"),
        }
    }
}

/// Inputs the Liar's Dice dispatch needs.
pub struct LiarsDiceReadInput<'a> {
    /// Path to the `MYOS`-magic + version-1 Liar's Dice checkpoint.
    pub checkpoint: &'a Path,
    /// The Liar's Dice strategy query (a `LiarsDiceInfo` payload).
    pub query: LiarsDiceStrategyQuery,
}

/// Inputs the NLHE dispatch needs. The encoder directory is required
/// because `PokerSolver::load` needs a separate encoder artifact.
pub struct NlheReadInput<'a> {
    /// Path to the `MYOS`-magic + version-1 NLHE profile checkpoint.
    pub checkpoint: &'a Path,
    /// Path to the NLHE encoder directory (the
    /// `bootstrap_encoder_streets()` output shape).
    pub encoder_dir: &'a Path,
    /// The NLHE strategy query (an `NlheInfoKey` payload).
    pub query: NlheStrategyQuery,
}

/// Run the Liar's Dice dispatch: load the checkpoint, run the query,
/// return the response. The checkpoint's `MYOS` magic + version 1
/// header are re-verified at the wire boundary (not just inside
/// `from_checkpoint_bytes`) so the failure reason surfaces cleanly
/// to a wrapper script that does not want to read the source.
pub fn answer_liars_dice(
    input: &LiarsDiceReadInput<'_>,
) -> Result<LiarsDiceStrategyResponse, DedicatedReadError> {
    let bytes = fs::read(input.checkpoint).map_err(|error| {
        DedicatedReadError::Io(format!(
            "read {} failed: {error}",
            input.checkpoint.display()
        ))
    })?;
    if bytes.is_empty() {
        return Err(DedicatedReadError::EmptyCheckpoint(
            input.checkpoint.display().to_string(),
        ));
    }
    // Re-verify the `MYOS` magic + version 1 header at the wire
    // boundary (the `from_checkpoint_bytes` call also checks, but
    // surfacing the typed reason here means a wrapper script can
    // grep for `checkpoint_magic:` / `checkpoint_version:` without
    // having to parse `solver_load:` text).
    if bytes.len() < 8 {
        return Err(DedicatedReadError::Io(format!(
            "checkpoint {} is too short to carry a header",
            input.checkpoint.display()
        )));
    }
    if &bytes[..4] != b"MYOS" {
        return Err(DedicatedReadError::CheckpointMagic(
            String::from_utf8_lossy(&bytes[..4]).into_owned(),
        ));
    }
    let found_version = u32::from_le_bytes(bytes[4..8].try_into().expect("4-byte slice"));
    if found_version != 1 {
        return Err(DedicatedReadError::CheckpointVersion {
            found: found_version,
            expected: 1,
        });
    }

    let solver: LiarsDiceSolver<1> = LiarsDiceSolver::load(input.checkpoint)
        .map_err(|error| DedicatedReadError::SolverLoad(error.to_string()))?;
    Ok(solver.answer(input.query.clone()))
}

/// Run the NLHE dispatch: load the encoder directory, load the
/// profile checkpoint, run the query, return the response. The
/// checkpoint's `MYOS` magic + version 1 header are re-verified at
/// the wire boundary so the failure reason surfaces cleanly.
pub fn answer_nlhe(input: &NlheReadInput<'_>) -> Result<NlheStrategyResponse, DedicatedReadError> {
    if !input.encoder_dir.exists() {
        return Err(DedicatedReadError::NotADirectory(
            input.encoder_dir.display().to_string(),
        ));
    }
    if !input.encoder_dir.is_dir() {
        return Err(DedicatedReadError::NotADirectory(
            input.encoder_dir.display().to_string(),
        ));
    }
    let encoder: NlheEncoder = load_encoder_dir(input.encoder_dir)
        .map_err(|error| DedicatedReadError::EncoderLoad(error.to_string()))?;

    let bytes = fs::read(input.checkpoint).map_err(|error| {
        DedicatedReadError::Io(format!(
            "read {} failed: {error}",
            input.checkpoint.display()
        ))
    })?;
    if bytes.is_empty() {
        return Err(DedicatedReadError::EmptyCheckpoint(
            input.checkpoint.display().to_string(),
        ));
    }
    if bytes.len() < 8 {
        return Err(DedicatedReadError::Io(format!(
            "checkpoint {} is too short to carry a header",
            input.checkpoint.display()
        )));
    }
    if &bytes[..4] != b"MYOS" {
        return Err(DedicatedReadError::CheckpointMagic(
            String::from_utf8_lossy(&bytes[..4]).into_owned(),
        ));
    }
    let found_version = u32::from_le_bytes(bytes[4..8].try_into().expect("4-byte slice"));
    if found_version != 1 {
        return Err(DedicatedReadError::CheckpointVersion {
            found: found_version,
            expected: 1,
        });
    }

    let solver = PokerSolver::load(input.checkpoint, encoder)
        .map_err(|error| DedicatedReadError::SolverLoad(error.to_string()))?;
    Ok(solver.answer(input.query.clone()))
}

/// SHA-256 of a checkpoint file, lowercase hex. Used by the binary
/// to populate the `SOLVER_READ checkpoint_sha256=...` line so an
/// operator can cross-check the answer against the exact byte stream
/// the read surfaced. Returns `EmptyCheckpoint` for a zero-byte file
/// (so the wrapper script gets the same reason whether the file is
/// missing-and-empty-by-extension or genuinely empty).
pub fn checkpoint_sha256(path: &Path) -> Result<String, DedicatedReadError> {
    let bytes = fs::read(path).map_err(|error| {
        DedicatedReadError::Io(format!("read {} failed: {error}", path.display()))
    })?;
    if bytes.is_empty() {
        return Err(DedicatedReadError::EmptyCheckpoint(
            path.display().to_string(),
        ));
    }
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    Ok(hex)
}

/// Pick the recommended Liar's Dice edge + its confidence. Mirrors
/// the W-03 `recommended_action_picks_argmax` test contract: the
/// `confidence` is the probability associated with the recommended
/// edge, and ties on probability break deterministically by edge
/// order (the `recommended_edge` helper's contract).
pub fn liars_dice_recommendation(
    response: &LiarsDiceStrategyResponse,
) -> Result<(String, f32), DedicatedReadError> {
    let Some(edge) = recommended_edge(response) else {
        return Err(DedicatedReadError::EmptyRecommendation);
    };
    let probability = response
        .actions
        .iter()
        .find(|(candidate, _)| *candidate == edge)
        .map(|(_, probability)| *probability)
        .unwrap_or(0.0);
    if !probability.is_finite() {
        return Err(DedicatedReadError::NonFiniteConfidence(probability));
    }
    Ok((format!("{edge:?}"), probability))
}

/// Pick the recommended NLHE edge + its confidence. Same contract as
/// the Liar's Dice helper: deterministic tiebreak by edge order, fail
/// closed on empty response or non-finite confidence.
pub fn nlhe_recommendation(
    response: &NlheStrategyResponse,
) -> Result<(String, f32), DedicatedReadError> {
    use myosu_games_poker::recommended_edge as nlhe_recommended_edge;
    let Some(edge) = nlhe_recommended_edge(response) else {
        return Err(DedicatedReadError::EmptyRecommendation);
    };
    let probability = response
        .actions
        .iter()
        .find(|(candidate, _)| *candidate == edge)
        .map(|(_, probability)| *probability)
        .unwrap_or(0.0);
    if !probability.is_finite() {
        return Err(DedicatedReadError::NonFiniteConfidence(probability));
    }
    Ok((format!("{edge:?}"), probability))
}

#[cfg(test)]
mod tests {
    use super::*;
    use myosu_games_liars_dice::{LiarsDiceClaim, LiarsDiceEdge};

    #[test]
    fn dedicated_game_from_slug_resolves_liars_dice() {
        assert_eq!(
            DedicatedGame::from_slug("liars-dice"),
            Some(DedicatedGame::LiarsDice)
        );
    }

    #[test]
    fn dedicated_game_from_slug_resolves_nlhe_heads_up() {
        assert_eq!(
            DedicatedGame::from_slug("nlhe-heads-up"),
            Some(DedicatedGame::NlheHeadsUp)
        );
    }

    #[test]
    fn dedicated_game_from_slug_rejects_unknown_slug() {
        assert_eq!(DedicatedGame::from_slug("cribbage"), None);
        assert_eq!(DedicatedGame::from_slug("not-a-real-game"), None);
    }

    #[test]
    fn dedicated_game_from_slug_rejects_empty_and_whitespace_slug() {
        assert_eq!(DedicatedGame::from_slug(""), None);
        assert_eq!(DedicatedGame::from_slug("   "), None);
        assert_eq!(DedicatedGame::from_slug("\t\n"), None);
    }

    #[test]
    fn dedicated_game_slug_round_trips() {
        assert_eq!(DedicatedGame::LiarsDice.slug(), "liars-dice");
        assert_eq!(DedicatedGame::NlheHeadsUp.slug(), "nlhe-heads-up");
        for game in [DedicatedGame::LiarsDice, DedicatedGame::NlheHeadsUp] {
            assert_eq!(DedicatedGame::from_slug(game.slug()), Some(game));
        }
    }

    #[test]
    fn dedicated_read_error_reason_is_stable_per_variant() {
        // Every variant's `reason()` returns the documented token. A
        // future refactor that changes a reason string is loud at
        // code-review time and the `solver_read_dedicated.sh` proof
        // harness's regex would fail-closed.
        assert_eq!(
            DedicatedReadError::UnknownGame("foo".to_string()).reason(),
            "unknown_game: foo"
        );
        assert_eq!(
            DedicatedReadError::MissingCheckpointField.reason(),
            "missing_checkpoint_field"
        );
        assert_eq!(
            DedicatedReadError::EmptyCheckpoint("/tmp/x".to_string()).reason(),
            "empty_checkpoint: /tmp/x"
        );
        assert_eq!(
            DedicatedReadError::MissingEncoderDir.reason(),
            "missing_encoder_dir"
        );
        assert_eq!(
            DedicatedReadError::NotADirectory("/tmp/y".to_string()).reason(),
            "not_a_directory: /tmp/y"
        );
        assert_eq!(
            DedicatedReadError::CheckpointMagic("XYZW".to_string()).reason(),
            "checkpoint_magic: XYZW"
        );
        assert_eq!(
            DedicatedReadError::CheckpointVersion {
                found: 7,
                expected: 1
            }
            .reason(),
            "checkpoint_version: found=7 expected=1"
        );
        assert_eq!(
            DedicatedReadError::EncoderLoad("bad".to_string()).reason(),
            "encoder_load: bad"
        );
        assert_eq!(
            DedicatedReadError::SolverLoad("bad".to_string()).reason(),
            "solver_load: bad"
        );
        assert_eq!(
            DedicatedReadError::Query("bad".to_string()).reason(),
            "query_decode: bad"
        );
        assert_eq!(
            DedicatedReadError::EmptyRecommendation.reason(),
            "empty_recommendation"
        );
        assert_eq!(
            DedicatedReadError::NonFiniteConfidence(f32::NAN).reason(),
            "non_finite_confidence: NaN"
        );
        assert_eq!(
            DedicatedReadError::Io("bad".to_string()).reason(),
            "io: bad"
        );
    }

    #[test]
    fn liars_dice_recommendation_picks_highest_probability_then_deterministic_tiebreak() {
        use myosu_games_liars_dice::LiarsDiceStrategyResponse;
        let response = LiarsDiceStrategyResponse::new(vec![
            (
                LiarsDiceEdge::Bid(LiarsDiceClaim::new(1, 3).expect("claim")),
                0.4,
            ),
            (LiarsDiceEdge::Challenge, 0.4),
            (
                LiarsDiceEdge::Bid(LiarsDiceClaim::new(2, 1).expect("claim")),
                0.7,
            ),
        ]);
        let (action, confidence) =
            liars_dice_recommendation(&response).expect("recommendation should resolve");
        assert_eq!(action, "Bid(LiarsDiceClaim { count: 2, face: 1 })");
        assert!((confidence - 0.7).abs() < 1e-6);
    }

    #[test]
    fn liars_dice_recommendation_rejects_empty_response() {
        use myosu_games_liars_dice::LiarsDiceStrategyResponse;
        let response = LiarsDiceStrategyResponse::new(Vec::new());
        let error =
            liars_dice_recommendation(&response).expect_err("empty response should fail closed");
        assert!(matches!(error, DedicatedReadError::EmptyRecommendation));
    }

    #[test]
    fn nlhe_recommendation_rejects_empty_response() {
        use myosu_games_poker::NlheStrategyResponse;
        let response = NlheStrategyResponse::new(Vec::new());
        let error = nlhe_recommendation(&response).expect_err("empty response should fail closed");
        assert!(matches!(error, DedicatedReadError::EmptyRecommendation));
    }

    #[test]
    fn checkpoint_sha256_matches_independent_sha256sum_for_known_bytes() {
        // The W-07 deterministic byte-stability discipline: hash a
        // known byte stream and assert the helper's output is
        // byte-identical to a system `sha256sum` invocation.
        let work_root = std::env::temp_dir().join(format!(
            "myosu-solver-read-dedicated-sha256-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&work_root);
        let path = work_root.join("checkpoint.bin");
        let bytes: Vec<u8> = (0u8..=255).cycle().take(4096).collect();
        fs::write(&path, &bytes).expect("write checkpoint bytes");

        let helper_hash = checkpoint_sha256(&path).expect("hash should resolve");
        // Run the system `sha256sum` to get the independent reference.
        let output = std::process::Command::new("sha256sum")
            .arg(&path)
            .output()
            .expect("sha256sum should run");
        assert!(output.status.success(), "sha256sum failed: {output:?}");
        let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
        let reference = stdout
            .split_whitespace()
            .next()
            .expect("sha256sum output should carry a hash");
        assert_eq!(helper_hash, reference.to_lowercase());
        // Hash must be 64 lowercase hex chars.
        assert_eq!(helper_hash.len(), 64);
        assert!(
            helper_hash
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn checkpoint_sha256_rejects_empty_file() {
        let work_root = std::env::temp_dir().join(format!(
            "myosu-solver-read-dedicated-empty-{}",
            std::process::id()
        ));
        let _ = fs::create_dir_all(&work_root);
        let path = work_root.join("empty.bin");
        fs::write(&path, Vec::<u8>::new()).expect("write empty file");

        let error = checkpoint_sha256(&path).expect_err("empty file should fail closed");
        assert!(matches!(error, DedicatedReadError::EmptyCheckpoint(_)));

        let _ = fs::remove_file(&path);
    }
}
