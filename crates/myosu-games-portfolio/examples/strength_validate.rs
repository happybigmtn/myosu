use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use myosu_games::StrategyResponse;
use myosu_games_portfolio::{
    PortfolioAction, PortfolioSolver, ResearchGame, decode_strategy_response,
    decode_strength_query, recommended_action,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let game_arg = required_arg(&mut args, "<game>")?;
    let checkpoint_file = PathBuf::from(required_arg(&mut args, "<checkpoint-file>")?);
    let query_file = PathBuf::from(required_arg(&mut args, "<strength-query-file>")?);
    let response_file = PathBuf::from(required_arg(&mut args, "<strength-response-file>")?);
    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    let game = ResearchGame::parse(&game_arg)?;
    let restored = PortfolioSolver::load(&checkpoint_file).map_err(|error| error.to_string())?;
    let query =
        decode_strength_query(&fs::read(&query_file)?).map_err(|error| error.to_string())?;
    if query.info.game != game {
        return Err(format!(
            "strength query game mismatch: expected `{}`, got `{}`",
            game.slug(),
            query.info.game.slug()
        )
        .into());
    }

    let expected = restored
        .answer_strength_checked(query.clone())
        .map_err(|error| error.to_string())?;
    let observed =
        decode_strategy_response(&fs::read(&response_file)?).map_err(|error| error.to_string())?;
    if !observed.is_valid() {
        return Err("portfolio strength response is not a valid probability distribution".into());
    }

    let l1_distance = l1_distance(&expected, &observed);
    let score = 1.0 / (1.0 + l1_distance.max(0.0));
    let exact_match = l1_distance < f64::EPSILON;
    let recommended = recommended_action(&observed)
        .map(PortfolioAction::label)
        .unwrap_or("none");

    println!("STRENGTH_VALIDATION game={}", game.slug());
    println!("STRENGTH_VALIDATION exact_match={exact_match}");
    println!("STRENGTH_VALIDATION l1_distance={l1_distance:.6}");
    println!("STRENGTH_VALIDATION score={score:.6}");
    println!(
        "STRENGTH_VALIDATION challenge_id={}",
        query.info.challenge.spot().challenge_id
    );
    println!("STRENGTH_VALIDATION recommended_action={recommended}");

    Ok(())
}

fn required_arg(
    args: &mut impl Iterator<Item = String>,
    label: &'static str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing required argument {label}").into())
}

fn l1_distance(
    expected: &StrategyResponse<PortfolioAction>,
    observed: &StrategyResponse<PortfolioAction>,
) -> f64 {
    expected
        .actions
        .iter()
        .map(|(action, _)| *action)
        .chain(observed.actions.iter().map(|(action, _)| *action))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|action| {
            let expected_probability = expected.probability_for(&action);
            let observed_probability = observed.probability_for(&action);
            f64::from((expected_probability - observed_probability).abs())
        })
        .sum()
}
