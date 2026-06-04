//! Multi-host validator determinism example.
//!
//! W-04 ships the multi-host (or multi-data-dir) determinism proof
//! that pairs with the on-host `tests/e2e/validator_determinism.sh`
//! surface. INV-003 requires two validator processes scoring the same
//! miner artifact to produce weights that agree within floating-point
//! tolerance; this binary is the per-operator scoring unit the proof
//! harness composes twice — once for each distinct
//! `MYOSU_OPERATOR_CHAIN` data dir and distinct SURI — and then
//! compares the two outputs on the `weight=<u16>` line.
//!
//! The example runs the SAME scoring loop the live `myosu-validator
//! score` binary runs (via `myosu_validator::validation::score_response`),
//! but with all inputs supplied on the command line so the e2e
//! harness can drive it without going through the chain RPC. The
//! `weight` value the example emits is the validator's local scoring
//! result, normalized to a `u16` (the same domain the on-chain
//! `Weights` row stores) so the harness can compare the two runs with
//! a simple integer-equality check — which is exactly the
//! `evaluate_validator_agreement` invariant from
//! `myosu-chain-client::lib` (the u16 weight domain collapses the
//! INV-003 epsilon window to `delta == 0`).
//!
//! Usage:
//!
//! ```bash
//! cargo run -p myosu-validator --example multi_host_validator_determinism -- \
//!   --miner-endpoint http://127.0.0.1:8191 \
//!   --operator-data-dir /tmp/mhosvd-operator-a \
//!   --suri //myosu//devnet//validator-1 \
//!   --query-path /tmp/mhosvd-operator-a/query.bin \
//!   --response-path /tmp/mhosvd-operator-a/response.bin \
//!   --checkpoint-path /tmp/mhosvd/checkpoint.bin \
//!   --encoder-dir /tmp/mhosvd/encoder \
//!   --game poker
//! ```
//!
//! Output (one line, grep-friendly):
//!
//! ```text
//! MULTI_HOST_VALIDATOR_DETERMINISM operator=<suri> miner=<endpoint> game=<slug> weight=<u16> l1_distance=<f64> score=<f64> scenario_count=<n> exact_match=<true|false> expected_action=<label> observed_action=<label> elapsed_ms=<u64>
//! ```
//!
//! Failure path (one line, also grep-friendly):
//!
//! ```text
//! MULTI_HOST_VALIDATOR_DETERMINISM_FAIL operator=<suri> miner=<endpoint> reason=<sanitized>
//! ```

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

use clap::Parser;

use myosu_validator::cli::{Cli, GameSelection};
use myosu_validator::validation::{ValidationPlan, score_response, validation_plan_from_cli};

const USAGE: &str = "usage: cargo run -p myosu-validator --example multi_host_validator_determinism -- \
    --miner-endpoint <http://HOST:PORT> \
    --operator-data-dir <path> \
    --suri <suri> \
    --query-path <file> \
    --response-path <file> \
    --checkpoint-path <file> \
    --encoder-dir <dir> \
    [--game <slug>]";

#[derive(Debug)]
struct ParsedArgs {
    miner_endpoint: String,
    operator_data_dir: PathBuf,
    suri: String,
    query_path: PathBuf,
    response_path: PathBuf,
    checkpoint_path: PathBuf,
    encoder_dir: PathBuf,
    game: GameSelection,
}

fn parse_args(args: &[String]) -> Result<ParsedArgs, String> {
    let mut miner_endpoint: Option<String> = None;
    let mut operator_data_dir: Option<PathBuf> = None;
    let mut suri: Option<String> = None;
    let mut query_path: Option<PathBuf> = None;
    let mut response_path: Option<PathBuf> = None;
    let mut checkpoint_path: Option<PathBuf> = None;
    let mut encoder_dir: Option<PathBuf> = None;
    let mut game: Option<GameSelection> = None;

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| format!("flag `{arg}` requires a value"))?;
        match arg.as_str() {
            "--miner-endpoint" => miner_endpoint = Some(value.clone()),
            "--operator-data-dir" => operator_data_dir = Some(PathBuf::from(value)),
            "--suri" => suri = Some(value.clone()),
            "--query-path" => query_path = Some(PathBuf::from(value)),
            "--response-path" => response_path = Some(PathBuf::from(value)),
            "--checkpoint-path" => checkpoint_path = Some(PathBuf::from(value)),
            "--encoder-dir" => encoder_dir = Some(PathBuf::from(value)),
            "--game" => {
                game = Some(
                    parse_game_slug(value)
                        .map_err(|err| format!("invalid --game `{value}`: {err}"))?,
                );
            }
            other => return Err(format!("unknown flag `{other}`")),
        }
    }

    let miner_endpoint =
        miner_endpoint.ok_or_else(|| "--miner-endpoint is required".to_string())?;
    let operator_data_dir =
        operator_data_dir.ok_or_else(|| "--operator-data-dir is required".to_string())?;
    let suri = suri.ok_or_else(|| "--suri is required".to_string())?;
    let query_path = query_path.ok_or_else(|| "--query-path is required".to_string())?;
    let response_path = response_path.ok_or_else(|| "--response-path is required".to_string())?;
    let checkpoint_path =
        checkpoint_path.ok_or_else(|| "--checkpoint-path is required".to_string())?;
    let encoder_dir = encoder_dir.ok_or_else(|| "--encoder-dir is required".to_string())?;

    Ok(ParsedArgs {
        miner_endpoint,
        operator_data_dir,
        suri,
        query_path,
        response_path,
        checkpoint_path,
        encoder_dir,
        game: game.unwrap_or(GameSelection::Poker),
    })
}

fn parse_game_slug(slug: &str) -> Result<GameSelection, String> {
    match slug {
        "poker" | "nlhe-heads-up" | "nlhe-hu" | "nlhe_hu" => Ok(GameSelection::Poker),
        "kuhn" | "kuhn_poker" => Ok(GameSelection::Kuhn),
        "liars-dice" | "liars_dice" => Ok(GameSelection::LiarsDice),
        "nlhe-six-max" | "nlhe_6max" => Ok(GameSelection::NlheSixMax),
        "plo" => Ok(GameSelection::Plo),
        "nlhe-tournament" | "nlhe_tournament" => Ok(GameSelection::NlheTournament),
        "short-deck" | "short_deck" => Ok(GameSelection::ShortDeck),
        "teen-patti" | "teen_patti" => Ok(GameSelection::TeenPatti),
        "hanafuda-koi-koi" | "hanafuda_koi_koi" => Ok(GameSelection::HanafudaKoiKoi),
        "hwatu-go-stop" | "hwatu_go_stop" => Ok(GameSelection::HwatuGoStop),
        "riichi-mahjong" | "riichi_mahjong" => Ok(GameSelection::RiichiMahjong),
        "bridge" => Ok(GameSelection::Bridge),
        "gin-rummy" | "gin_rummy" => Ok(GameSelection::GinRummy),
        "stratego" => Ok(GameSelection::Stratego),
        "ofc-chinese-poker" | "ofc_chinese_poker" => Ok(GameSelection::OfcChinesePoker),
        "spades" => Ok(GameSelection::Spades),
        "dou-di-zhu" | "dou_di_zhu" => Ok(GameSelection::DouDiZhu),
        "pusoy-dos" | "pusoy_dos" => Ok(GameSelection::PusoyDos),
        "tien-len" | "tien_len" => Ok(GameSelection::TienLen),
        "call-break" | "call_break" => Ok(GameSelection::CallBreak),
        "backgammon" => Ok(GameSelection::Backgammon),
        "hearts" => Ok(GameSelection::Hearts),
        "cribbage" => Ok(GameSelection::Cribbage),
        other => Err(format!("unsupported game slug `{other}`")),
    }
}

fn game_slug(game: GameSelection) -> &'static str {
    match game {
        GameSelection::Poker => "poker",
        GameSelection::Kuhn => "kuhn",
        GameSelection::LiarsDice => "liars-dice",
        GameSelection::NlheSixMax => "nlhe-six-max",
        GameSelection::Plo => "plo",
        GameSelection::NlheTournament => "nlhe-tournament",
        GameSelection::ShortDeck => "short-deck",
        GameSelection::TeenPatti => "teen-patti",
        GameSelection::HanafudaKoiKoi => "hanafuda-koi-koi",
        GameSelection::HwatuGoStop => "hwatu-go-stop",
        GameSelection::RiichiMahjong => "riichi-mahjong",
        GameSelection::Bridge => "bridge",
        GameSelection::GinRummy => "gin-rummy",
        GameSelection::Stratego => "stratego",
        GameSelection::OfcChinesePoker => "ofc-chinese-poker",
        GameSelection::Spades => "spades",
        GameSelection::DouDiZhu => "dou-di-zhu",
        GameSelection::PusoyDos => "pusoy-dos",
        GameSelection::TienLen => "tien-len",
        GameSelection::CallBreak => "call-break",
        GameSelection::Backgammon => "backgammon",
        GameSelection::Hearts => "hearts",
        GameSelection::Cribbage => "cribbage",
    }
}

fn fail_line(suri: &str, miner_endpoint: &str, reason: &str) -> String {
    let sanitized = reason.replace('\n', "\\n");
    format!(
        "MULTI_HOST_VALIDATOR_DETERMINISM_FAIL operator={} miner={} reason={}",
        suri, miner_endpoint, sanitized
    )
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let parsed = match parse_args(&args) {
        Ok(p) => p,
        Err(error) => {
            eprintln!("multi_host_validator_determinism: {error}");
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    // The validator scoring path is keyed on the (query, response) wire
    // pair the live `myosu-validator score` command consumes. The
    // harness writes those files; we just consume them.
    let plan = ValidationPlan {
        game: parsed.game,
        encoder_dir: parsed.encoder_dir.clone(),
        checkpoint_path: parsed.checkpoint_path.clone(),
        query_path: parsed.query_path.clone(),
        response_path: parsed.response_path.clone(),
    };

    // Sanity check: the constructed plan must also pass the public
    // `validation_plan_from_cli` validation. We re-construct the
    // minimum CLI shape that the helper expects and assert it
    // round-trips. This catches a regression where the example's
    // ValidationPlan shape diverges from the live CLI's accepted
    // shape (which is the same shape `myosu-validator score` uses).
    if let Err(error) = plan_sanity_check(&plan) {
        eprintln!(
            "{}",
            fail_line(&parsed.suri, &parsed.miner_endpoint, &error)
        );
        return ExitCode::from(1);
    }

    let started_at = Instant::now();
    let report = match score_response(&plan) {
        Ok(report) => report,
        Err(error) => {
            let reason = format!("scoring failed: {error}");
            eprintln!(
                "{}",
                fail_line(&parsed.suri, &parsed.miner_endpoint, &reason)
            );
            return ExitCode::from(1);
        }
    };
    let elapsed_ms = started_at.elapsed().as_millis() as u64;

    // Normalize the floating-point score to a u16 weight — the same
    // domain the on-chain `Weights` row stores. The score is
    // monotonically decreasing in L1 distance and bounded to [0, 1],
    // so a linear remap to [0, u16::MAX] is the truthful weight
    // quantization. Two validators running the same scoring loop
    // against the same (query, response) pair MUST produce the same
    // weight — that is the INV-003 determinism contract this example
    // is the unit of.
    let weight = score_to_u16_weight(report.score);

    println!(
        "MULTI_HOST_VALIDATOR_DETERMINISM operator={} miner={} game={} weight={} l1_distance={:.12} score={:.12} scenario_count={} exact_match={} expected_action={} observed_action={} elapsed_ms={}",
        parsed.suri,
        parsed.miner_endpoint,
        game_slug(parsed.game),
        weight,
        report.l1_distance,
        report.score,
        report.action_count,
        report.exact_match,
        report.expected_action,
        report.observed_action,
        elapsed_ms,
    );
    // The operator-data-dir is part of the contract (the W-04 spec
    // names it as a required argument for the multi-host proof). Emit
    // a follow-on line so a wrapper script can grep the local data
    // root the scoring ran from, without having to re-parse the
    // environment.
    println!(
        "MULTI_HOST_VALIDATOR_DETERMINISM_DATA_DIR operator={} operator_data_dir={}",
        parsed.suri,
        parsed.operator_data_dir.display(),
    );

    ExitCode::SUCCESS
}

fn plan_sanity_check(plan: &ValidationPlan) -> Result<(), String> {
    // The CLI helper enforces the same shape the live binary uses. We
    // hand-build a minimal Cli that mirrors the plan and ask the helper
    // to validate it. A regression in either the helper or the plan
    // shape is surfaced as a typed error.
    if !plan.query_path.exists() {
        return Err(format!(
            "query_path does not exist: {}",
            plan.query_path.display()
        ));
    }
    if !plan.response_path.exists() {
        return Err(format!(
            "response_path does not exist: {}",
            plan.response_path.display()
        ));
    }
    if !plan.checkpoint_path.exists() {
        return Err(format!(
            "checkpoint_path does not exist: {}",
            plan.checkpoint_path.display()
        ));
    }
    if matches!(plan.game, GameSelection::Poker) && !plan.encoder_dir.exists() {
        return Err(format!(
            "encoder_dir required for poker: {}",
            plan.encoder_dir.display()
        ));
    }
    // Pair the (query_path, response_path) assertion the live helper
    // also enforces (one without the other is invalid). The live `Cli`
    // requires either `--key` or `--key_config_dir` via an ArgGroup;
    // pass a sentinel SURI here so the parser is satisfied (we never
    // call `resolve_key_uri` or otherwise exercise the key path — the
    // score_response call above already consumed the plan via its
    // own ValidationPlan shape, and the point of this sanity check is
    // to assert the plan's CLI shape round-trips, not to bind a key).
    let cli = Cli::try_parse_from([
        "myosu-validator",
        "--key",
        "//_w04_sanity_check_placeholder",
        "--query-file",
        plan.query_path.to_str().unwrap_or(""),
        "--response-file",
        plan.response_path.to_str().unwrap_or(""),
        "--checkpoint",
        plan.checkpoint_path.to_str().unwrap_or(""),
        "--encoder-dir",
        plan.encoder_dir.to_str().unwrap_or(""),
        "--game",
        game_slug(plan.game),
    ])
    .map_err(|error| format!("plan failed CLI parse: {error}"))?;
    match validation_plan_from_cli(&cli) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err("validation_plan_from_cli returned None for the supplied plan".into()),
        Err(error) => Err(format!(
            "validation_plan_from_cli rejected the plan: {error}"
        )),
    }
}

fn score_to_u16_weight(score: f64) -> u16 {
    if !score.is_finite() {
        return 0;
    }
    let clamped = score.clamp(0.0, 1.0);
    let scaled = clamped * f64::from(u16::MAX);
    scaled.round() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_to_u16_weight_zero_for_zero_score() {
        assert_eq!(score_to_u16_weight(0.0), 0);
    }

    #[test]
    fn score_to_u16_weight_max_for_perfect_score() {
        assert_eq!(score_to_u16_weight(1.0), u16::MAX);
    }

    #[test]
    fn score_to_u16_weight_is_monotonic() {
        // INV-003: two operators scoring the same (query, response)
        // pair must produce the same weight. The score→weight map
        // must therefore be deterministic (no random tie-breaks).
        let samples = [0.0_f64, 0.25, 0.5, 0.75, 1.0];
        let mut prev = 0_u16;
        for s in samples {
            let w = score_to_u16_weight(s);
            assert!(
                w >= prev,
                "weight must be non-decreasing in score: s={s} w={w} prev={prev}"
            );
            prev = w;
        }
    }

    #[test]
    fn score_to_u16_weight_clamps_out_of_range_finite() {
        // A score outside [0, 1] must be clamped so the on-chain u16
        // weight is always a well-defined quantized value (the live
        // `score_response` is bounded to [0, 1], so this is a
        // defense-in-depth test, not a real-world codepath).
        assert_eq!(score_to_u16_weight(-0.5), 0);
        assert_eq!(score_to_u16_weight(1.5), u16::MAX);
    }

    #[test]
    fn score_to_u16_weight_rejects_non_finite() {
        // Non-finite scores must collapse to 0 so a degenerate
        // input can never produce a non-deterministic u16 weight.
        // (The early-return in `score_to_u16_weight` is the
        // defense-in-depth guard for the same failure mode — the
        // live `score_response` is bounded to [0, 1], so this is
        // a regression test for the guard, not a real codepath.)
        assert_eq!(score_to_u16_weight(f64::NAN), 0);
        assert_eq!(score_to_u16_weight(f64::INFINITY), 0);
        assert_eq!(score_to_u16_weight(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn game_slug_round_trip() {
        // The CLI's `--game` parser must accept every slug the
        // `game_slug` function emits, so a harness can pass the slug
        // through unchanged.
        let slugs = [
            "poker",
            "kuhn",
            "liars-dice",
            "nlhe-six-max",
            "plo",
            "nlhe-tournament",
            "short-deck",
            "teen-patti",
            "hanafuda-koi-koi",
            "hwatu-go-stop",
            "riichi-mahjong",
            "bridge",
            "gin-rummy",
            "stratego",
            "ofc-chinese-poker",
            "spades",
            "dou-di-zhu",
            "pusoy-dos",
            "tien-len",
            "call-break",
            "backgammon",
            "hearts",
            "cribbage",
        ];
        for slug in slugs {
            let game = parse_game_slug(slug).expect("slug must parse");
            assert_eq!(game_slug(game), slug, "round-trip mismatch for {slug}");
        }
    }

    #[test]
    fn parse_args_requires_miner_endpoint() {
        let args = vec![
            "multi_host_validator_determinism".to_string(),
            "--operator-data-dir".to_string(),
            "/tmp/a".to_string(),
            "--suri".to_string(),
            "//myosu//devnet//validator-1".to_string(),
            "--query-path".to_string(),
            "/tmp/a/query.bin".to_string(),
            "--response-path".to_string(),
            "/tmp/a/response.bin".to_string(),
            "--checkpoint-path".to_string(),
            "/tmp/a/checkpoint.bin".to_string(),
            "--encoder-dir".to_string(),
            "/tmp/a/encoder".to_string(),
        ];
        let err = parse_args(&args).expect_err("missing --miner-endpoint must error");
        assert!(err.contains("--miner-endpoint"));
    }

    #[test]
    fn parse_args_rejects_unknown_flag() {
        let args = vec![
            "multi_host_validator_determinism".to_string(),
            "--nonsense".to_string(),
            "value".to_string(),
        ];
        let err = parse_args(&args).expect_err("unknown flag must error");
        assert!(err.contains("unknown flag"));
    }
}
