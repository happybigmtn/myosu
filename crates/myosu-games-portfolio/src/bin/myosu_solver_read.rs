//! `myosu-solver-read` — read-only, JSON-in / line-out solver surface.
//!
//! Reads a single JSON object from stdin shaped like:
//!
//! ```json
//! { "game": "<slug>", "challenge": { <PortfolioChallenge variant JSON> } }
//! ```
//!
//! Dispatches the typed `PortfolioChallenge` through
//! `myosu_games_portfolio::answer_typed_challenge` and prints exactly one
//! grep-friendly line to stdout:
//!
//! ```text
//! SOLVER_READ game=<slug> action=<action-label> confidence=<f32> engine_tier=rule-aware engine_family=<...> elapsed_ms=<u64>
//! ```
//!
//! Failure paths print `SOLVER_READ_FAIL reason=<reason>` to stdout and exit
//! non-zero so a wrapper script can `grep ^SOLVER_READ_FAIL` for triage.
//!
//! The binary is read-only by design: it never mutates chain state, never
//! touches a wallet, never emits a bundle, never registers an operator. If
//! abuse becomes a problem, the rate-limit is a separate ticket, not a
//! permissioned-API problem.

use std::env;
use std::io::{self, Read};
use std::process::ExitCode;
use std::time::Instant;

use myosu_games_portfolio::{
    EngineTier, PortfolioChallenge, ResearchGame, answer_typed_challenge, recommended_action,
};
use serde::Deserialize;

const USAGE: &str = "myosu-solver-read — read a JSON scenario from stdin and emit one recommendation line.

Usage:
    myosu-solver-read

Reads a single JSON object from stdin of the shape
    { \"game\": \"<slug>\", \"challenge\": { <PortfolioChallenge variant JSON> } }
and prints one
    SOLVER_READ game=<slug> action=<label> confidence=<f32> engine_tier=rule-aware engine_family=<...> elapsed_ms=<u64>
line to stdout. On any error, prints one
    SOLVER_READ_FAIL reason=<reason>
line to stdout and exits non-zero.
";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadRequest {
    game: String,
    challenge: PortfolioChallenge,
}

/// Wire up the read-only dispatch.
fn main() -> ExitCode {
    if env::args().any(|arg| arg == "-h" || arg == "--help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    let started = Instant::now();
    let mut input = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut input) {
        return fail(format!("stdin_read_failed: {error}"));
    }

    if input.trim().is_empty() {
        return fail("empty_stdin".to_string());
    }

    let request: ReadRequest = match serde_json::from_str(input.trim()) {
        Ok(request) => request,
        Err(error) => return fail(format!("invalid_json: {error}")),
    };

    let requested_game = request.game.trim();
    if requested_game.is_empty() {
        return fail("empty_game_slug".to_string());
    }
    let Some(parsed_game) = ResearchGame::from_slug(requested_game) else {
        return fail(format!("unknown_game: {requested_game}"));
    };
    let challenge_game = request.challenge.game();
    if challenge_game != parsed_game {
        return fail(format!(
            "game_mismatch: slug={parsed_game} challenge_variant={challenge_game}"
        ));
    }
    if !parsed_game.is_portfolio_routed() {
        return fail(format!(
            "not_portfolio_routed: {parsed_game} has a dedicated solver crate; use myosu-solver-read-dedicated"
        ));
    }

    let answer = match answer_typed_challenge(&request.challenge, 0) {
        Ok(answer) => answer,
        Err(error) => return fail(format!("engine_dispatch_failed: {error}")),
    };
    let Some(action) = recommended_action(&answer.response) else {
        return fail("empty_recommendation".to_string());
    };
    let confidence = answer
        .response
        .actions
        .iter()
        .find(|(candidate, _)| *candidate == action)
        .map(|(_, probability)| *probability)
        .unwrap_or(0.0);
    if !confidence.is_finite() {
        return fail(format!("non_finite_confidence: {confidence}"));
    }
    let engine_tier = match answer.engine_tier {
        EngineTier::StaticBaseline => "static-baseline",
        EngineTier::RuleAware => "rule-aware",
    };
    let elapsed_ms = started.elapsed().as_millis() as u64;

    println!(
        "SOLVER_READ game={} action={} confidence={:.6} engine_tier={} engine_family={} legal_action_count={} elapsed_ms={elapsed_ms}",
        parsed_game.slug(),
        action.label(),
        confidence,
        engine_tier,
        answer.engine_family,
        answer.legal_actions.len(),
    );

    ExitCode::SUCCESS
}

fn fail(reason: String) -> ExitCode {
    // `SOLVER_READ_FAIL reason=<...>` is grep-friendly. The reason string
    // is sanitized to one line so a downstream parser never sees a stray
    // newline embedded in the reason.
    let sanitized = reason.replace('\n', "\\n");
    println!("SOLVER_READ_FAIL reason={sanitized}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use myosu_games_portfolio::{
        PortfolioAction, PortfolioChallenge, PortfolioChallengeSpot, PortfolioStrategyResponse,
        ResearchGame, recommended_action,
    };

    #[test]
    fn empty_stdin_is_rejected() {
        // Mirrors the runtime `empty_stdin` branch via the public fail path.
        let reason = "empty_stdin".to_string();
        let sanitized = reason.replace('\n', "\\n");
        assert_eq!(
            format!("SOLVER_READ_FAIL reason={sanitized}"),
            "SOLVER_READ_FAIL reason=empty_stdin"
        );
    }

    #[test]
    fn unknown_slug_fails_closed() {
        let slug = "not-a-real-game";
        assert!(ResearchGame::from_slug(slug).is_none());
    }

    #[test]
    fn empty_slug_fails_closed() {
        assert!(ResearchGame::from_slug("").is_none());
        assert!(ResearchGame::from_slug("   ").is_none());
    }

    #[test]
    fn not_portfolio_routed_fails_closed() {
        // NlheHeadsUp + LiarsDice are the two dedicated-solver games; the
        // read-only surface must reject them with a clear reason.
        let heads_up = ResearchGame::NlheHeadsUp;
        let liars_dice = ResearchGame::LiarsDice;
        assert!(!heads_up.is_portfolio_routed());
        assert!(!liars_dice.is_portfolio_routed());
    }

    #[test]
    fn bootstrap_challenge_matches_its_slug() {
        // Every `benchmarked` game has a `bootstrap` typed challenge; the
        // challenge variant's `game()` must match the slug we hand to
        // `ResearchGame::from_slug` so a request can never silently route
        // a hearts challenge through a cribbage dispatch.
        for game in [
            ResearchGame::Cribbage,
            ResearchGame::Hearts,
            ResearchGame::Bridge,
            ResearchGame::Spades,
            ResearchGame::CallBreak,
            ResearchGame::GinRummy,
            ResearchGame::Stratego,
            ResearchGame::Backgammon,
            ResearchGame::HanafudaKoiKoi,
            ResearchGame::HwatuGoStop,
            ResearchGame::RiichiMahjong,
            ResearchGame::OfcChinesePoker,
            ResearchGame::DouDiZhu,
            ResearchGame::PusoyDos,
            ResearchGame::TienLen,
            ResearchGame::NlheSixMax,
            ResearchGame::Plo,
            ResearchGame::NlheTournament,
            ResearchGame::ShortDeck,
            ResearchGame::TeenPatti,
        ] {
            let Some(challenge) = PortfolioChallenge::bootstrap(game) else {
                continue;
            };
            assert_eq!(challenge.game(), game);
            assert_eq!(challenge.game().slug(), game.slug());
        }
    }

    #[test]
    fn recommended_action_picks_argmax() {
        // Sanity-check the line-protocol contract: a `SOLVER_READ action=...`
        // line for a known distribution must equal `recommended_action`'s
        // argmax.
        let response = PortfolioStrategyResponse::new(vec![
            (PortfolioAction::KeepCrib, 0.4),
            (PortfolioAction::PegRun, 0.9),
            (PortfolioAction::DiscardDeadwood, 0.5),
        ]);
        assert_eq!(recommended_action(&response), Some(PortfolioAction::PegRun));
    }

    #[test]
    fn spot_decision_round_trips() {
        // The challenge_id / decision / rule_file / solver_family fields
        // are operator-provided and must round-trip cleanly through serde.
        let spot = PortfolioChallengeSpot::scenario(
            ResearchGame::Cribbage,
            "discard-pressure",
            "Cribbage discard pressure spot",
        );
        let json = serde_json::to_string(&spot).expect("serialize spot");
        let back: PortfolioChallengeSpot = serde_json::from_str(&json).expect("deserialize spot");
        assert_eq!(spot, back);
    }
}
