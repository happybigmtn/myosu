use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use myosu_games::StrategyResponse;
use myosu_games_portfolio::{
    PortfolioAction, PortfolioSolver, ResearchGame, decode_strategy_query,
    decode_strategy_response, recommended_action,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let game_arg = required_arg(&mut args, "<game>")?;
    let checkpoint_file = PathBuf::from(required_arg(&mut args, "<checkpoint-file>")?);
    let query_file = PathBuf::from(required_arg(&mut args, "<query-file>")?);
    let response_file = PathBuf::from(required_arg(&mut args, "<response-file>")?);
    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    let game = ResearchGame::parse(&game_arg)?;
    if !game.is_portfolio_routed() {
        return Err(format!(
            "`{}` uses a dedicated solver crate; use a portfolio-routed game",
            game.slug()
        )
        .into());
    }

    let solver = PortfolioSolver::load(&checkpoint_file)?;
    solver.ensure_supports(game)?;

    let query = decode_strategy_query(&fs::read(&query_file)?)?;
    if query.info.game != game {
        return Err(format!(
            "query game mismatch: expected `{}`, got `{}`",
            game.slug(),
            query.info.game.slug()
        )
        .into());
    }

    let observed = decode_strategy_response(&fs::read(&response_file)?)?;
    if !observed.is_valid() {
        return Err("portfolio response is not a valid probability distribution".into());
    }

    let expected = solver.answer_checked(query)?;
    let l1_distance = l1_distance(&expected, &observed);
    let score = 1.0 / (1.0 + l1_distance.max(0.0));
    let exact_match = l1_distance < f64::EPSILON;
    let expected_action = recommended_action(&expected)
        .map(PortfolioAction::label)
        .unwrap_or("none");
    let observed_action = recommended_action(&observed)
        .map(PortfolioAction::label)
        .unwrap_or("none");

    println!("VALIDATION game={}", game.slug());
    println!(
        "VALIDATION checkpoint_file={}",
        display_path(&checkpoint_file)
    );
    println!("VALIDATION query_file={}", display_path(&query_file));
    println!("VALIDATION response_file={}", display_path(&response_file));
    println!("VALIDATION exact_match={exact_match}");
    println!("VALIDATION l1_distance={l1_distance:.6}");
    println!("VALIDATION score={score:.6}");
    println!("VALIDATION action_count={}", observed.actions.len());
    println!("VALIDATION expected_action={expected_action}");
    println!("VALIDATION observed_action={observed_action}");

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

fn display_path(path: &Path) -> String {
    path.display().to_string()
}
