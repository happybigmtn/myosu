use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use myosu_games::StrategyResponse;
use myosu_games_portfolio::{
    PortfolioAction, PortfolioSolver, ResearchGame, decode_strategy_response,
    decode_strength_query, encode_strategy_response, encode_strength_query, recommended_action,
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
    let query_file = output_dir.join("strength-query.bin");
    let response_file = output_dir.join("strength-response.bin");

    let mut solver = PortfolioSolver::for_game(game);
    solver.train(iterations);
    solver.save(&checkpoint_file)?;

    let query = PortfolioSolver::strength_query(game)?;
    fs::write(&query_file, encode_strength_query(&query)?)?;

    let restored = PortfolioSolver::load(&checkpoint_file)?;
    let query_bytes = fs::read(&query_file)?;
    let decoded_query = decode_strength_query(&query_bytes)?;
    if decoded_query.info.game != game {
        return Err(format!(
            "query game mismatch: expected `{}`, got `{}`",
            game.slug(),
            decoded_query.info.game.slug()
        )
        .into());
    }

    let budget_ms = optional_budget_ms()?;
    let started_at = Instant::now();
    let expected = restored.answer_strength_checked(decoded_query.clone())?;
    let quality = restored.strength_quality(decoded_query)?;
    let elapsed_ms = started_at.elapsed().as_secs_f64() * 1_000.0;
    let budget_status = match budget_ms {
        Some(budget_ms) if elapsed_ms <= budget_ms => "pass",
        Some(_) => "fail",
        None => "skipped",
    };
    let response_bytes = encode_strategy_response(&expected)?;
    fs::write(&response_file, response_bytes)?;

    let observed = decode_strategy_response(&fs::read(&response_file)?)?;
    if !observed.is_valid() {
        return Err("portfolio strength response is not a valid probability distribution".into());
    }

    let l1_distance = l1_distance(&expected, &observed);
    let roundtrip_score = 1.0 / (1.0 + l1_distance.max(0.0));
    let exact_match = l1_distance < f64::EPSILON;
    let recommended = recommended_action(&observed)
        .map(PortfolioAction::label)
        .unwrap_or("none");

    println!("STRENGTH game={}", game.slug());
    println!("STRENGTH output_dir={}", output_dir.display());
    println!(
        "STRENGTH checkpoint_file={}",
        display_path(&checkpoint_file)
    );
    println!("STRENGTH query_file={}", display_path(&query_file));
    println!("STRENGTH response_file={}", display_path(&response_file));
    println!("STRENGTH iterations={iterations}");
    println!("STRENGTH exact_match={exact_match}");
    println!("STRENGTH l1_distance={l1_distance:.6}");
    println!("STRENGTH roundtrip_score={roundtrip_score:.6}");
    println!("STRENGTH score={:.6}", quality.score);
    println!(
        "STRENGTH baseline_l1_distance={:.6}",
        quality.baseline_l1_distance
    );
    println!("STRENGTH engine_tier={}", quality.engine_tier.as_str());
    println!(
        "STRENGTH engine_family={}",
        shell_token(&quality.engine_family)
    );
    println!("STRENGTH legal_actions={}", quality.legal_action_count);
    println!("STRENGTH deterministic={}", quality.deterministic);
    println!("STRENGTH seed={}", quality.seed);
    println!("STRENGTH baseline_action={}", quality.baseline_action);
    println!("STRENGTH engine_action={}", quality.engine_action);
    println!("STRENGTH recommended_action={recommended}");
    println!("STRENGTH elapsed_ms={elapsed_ms:.3}");
    println!(
        "STRENGTH budget_ms={}",
        budget_ms
            .map(|budget_ms| format!("{budget_ms:.3}"))
            .unwrap_or_else(|| "unconfigured".to_string())
    );
    println!("STRENGTH budget_status={budget_status}");

    if budget_status == "fail" {
        return Err(format!(
            "{} exceeded configured engine budget: elapsed_ms={elapsed_ms:.3} budget_ms={:.3}",
            game.slug(),
            budget_ms.unwrap_or_default()
        )
        .into());
    }

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

fn shell_token(value: &str) -> String {
    value.replace(' ', "_")
}

fn optional_budget_ms() -> Result<Option<f64>, Box<dyn Error>> {
    match env::var("MYOSU_ENGINE_BUDGET_MS") {
        Ok(value) => Ok(Some(value.parse::<f64>()?)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error.into()),
    }
}
