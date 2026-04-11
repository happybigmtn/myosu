use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use myosu_games::StrategyResponse;
use myosu_games_portfolio::{
    PortfolioAction, PortfolioSolver, ResearchGame, decode_strategy_query,
    decode_strategy_response, encode_strategy_query, encode_strategy_response, recommended_action,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let game_arg = required_arg(&mut args, "<game>")?;
    let output_dir = PathBuf::from(required_arg(&mut args, "<output-dir>")?);
    let iterations = match args.next() {
        Some(value) => value.parse::<usize>()?,
        None => 0,
    };
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

    fs::create_dir_all(&output_dir)?;
    let checkpoint_file = output_dir.join("checkpoint.bin");
    let query_file = output_dir.join("query.bin");
    let response_file = output_dir.join("response.bin");

    let mut solver = PortfolioSolver::for_game(game);
    solver.train(iterations);
    solver.save(&checkpoint_file)?;

    let query = PortfolioSolver::bootstrap_query(game);
    fs::write(&query_file, encode_strategy_query(&query)?)?;

    let restored = PortfolioSolver::load(&checkpoint_file)?;
    let query_bytes = fs::read(&query_file)?;
    let decoded_query = decode_strategy_query(&query_bytes)?;
    if decoded_query.info.game != game {
        return Err(format!(
            "query game mismatch: expected `{}`, got `{}`",
            game.slug(),
            decoded_query.info.game.slug()
        )
        .into());
    }

    let expected = restored.answer_checked(decoded_query.clone())?;
    let response_bytes = encode_strategy_response(&expected)?;
    fs::write(&response_file, response_bytes)?;

    let observed = decode_strategy_response(&fs::read(&response_file)?)?;
    if !observed.is_valid() {
        return Err("portfolio response is not a valid probability distribution".into());
    }

    let l1_distance = l1_distance(&expected, &observed);
    let score = 1.0 / (1.0 + l1_distance.max(0.0));
    let exact_match = l1_distance < f64::EPSILON;
    let recommended = recommended_action(&observed)
        .map(PortfolioAction::label)
        .unwrap_or("none");

    println!("BOOTSTRAP game={}", game.slug());
    println!("BOOTSTRAP output_dir={}", output_dir.display());
    println!(
        "BOOTSTRAP checkpoint_file={}",
        display_path(&checkpoint_file)
    );
    println!("BOOTSTRAP query_file={}", display_path(&query_file));
    println!("BOOTSTRAP response_file={}", display_path(&response_file));
    println!("BOOTSTRAP iterations={iterations}");
    println!("BOOTSTRAP exact_match={exact_match}");
    println!("BOOTSTRAP l1_distance={l1_distance:.6}");
    println!("BOOTSTRAP score={score:.6}");
    println!("BOOTSTRAP action_count={}", observed.actions.len());
    println!("BOOTSTRAP recommended_action={recommended}");

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
