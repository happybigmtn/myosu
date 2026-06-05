//! `myosu-solver-read-dedicated` — JSON-in / line-out solver surface
//! for the two dedicated-solver games (liars-dice, nlhe-heads-up).
//!
//! Reads a single JSON object from stdin, dispatches to the
//! appropriate dedicated solver, and prints exactly one `SOLVER_READ`
//! line on success or one `SOLVER_READ_FAIL` line on error.

use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Instant;

use serde::Deserialize;

use myosu_solver_read_dedicated::{
    DedicatedGame, DedicatedReadError, LiarsDiceReadInput, NlheReadInput, answer_liars_dice,
    answer_nlhe, checkpoint_sha256, liars_dice_recommendation, nlhe_recommendation,
};

/// The JSON request shape. `#[serde(deny_unknown_fields)]` makes any
/// unexpected field loud, so a future schema change cannot silently
/// break callers.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DedicatedReadRequest {
    game: String,
    checkpoint: PathBuf,
    #[serde(default)]
    encoder_dir: Option<PathBuf>,
    #[serde(default)]
    query: serde_json::Value,
}

fn main() {
    if let Err(error) = run() {
        eprintln!(
            "SOLVER_READ_FAIL reason={}",
            sanitize_reason(&error.reason())
        );
        std::process::exit(1);
    }
}

fn run() -> Result<(), DedicatedReadError> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| DedicatedReadError::Io(format!("stdin read failed: {error}")))?;

    if input.trim().is_empty() {
        return Err(DedicatedReadError::Io("empty stdin".to_string()));
    }

    let request: DedicatedReadRequest = serde_json::from_str(&input)
        .map_err(|error| DedicatedReadError::Query(format!("invalid JSON: {error}")))?;

    let game = DedicatedGame::from_slug(&request.game)
        .ok_or_else(|| DedicatedReadError::UnknownGame(request.game.clone()))?;

    if !request.checkpoint.exists() {
        return Err(DedicatedReadError::NotADirectory(
            request.checkpoint.display().to_string(),
        ));
    }

    let started = Instant::now();
    let sha256 = checkpoint_sha256(&request.checkpoint)?;

    let (game_slug, action, confidence, legal_action_count) = match game {
        DedicatedGame::LiarsDice => {
            let query: myosu_games_liars_dice::LiarsDiceStrategyQuery =
                serde_json::from_value(request.query.clone()).map_err(|error| {
                    DedicatedReadError::Query(format!("liars-dice query decode: {error}"))
                })?;
            let input = LiarsDiceReadInput {
                checkpoint: &request.checkpoint,
                query,
            };
            let response = answer_liars_dice(&input)?;
            let (action, confidence) = liars_dice_recommendation(&response)?;
            (game.slug(), action, confidence, response.actions.len())
        }
        DedicatedGame::NlheHeadsUp => {
            let encoder_dir = request
                .encoder_dir
                .ok_or(DedicatedReadError::MissingEncoderDir)?;
            let query: myosu_games_poker::NlheStrategyQuery =
                serde_json::from_value(request.query.clone()).map_err(|error| {
                    DedicatedReadError::Query(format!("nlhe-heads-up query decode: {error}"))
                })?;
            let input = NlheReadInput {
                checkpoint: &request.checkpoint,
                encoder_dir: &encoder_dir,
                query,
            };
            let response = answer_nlhe(&input)?;
            let (action, confidence) = nlhe_recommendation(&response)?;
            (game.slug(), action, confidence, response.actions.len())
        }
    };

    let elapsed_ms = started.elapsed().as_millis() as u64;

    println!(
        "SOLVER_READ game={} action={} confidence={:.6} engine_tier=dedicated-cfr checkpoint_sha256={} legal_action_count={} elapsed_ms={}",
        game_slug, action, confidence, sha256, legal_action_count, elapsed_ms,
    );
    Ok(())
}

/// Sanitize a reason string for the `SOLVER_READ_FAIL` line:
/// - collapse embedded newlines to `\n`
/// - replace embedded `=` with `_` so the line stays single-token parsable
fn sanitize_reason(reason: &str) -> String {
    reason.replace('\n', "\\n").replace('=', "_")
}
