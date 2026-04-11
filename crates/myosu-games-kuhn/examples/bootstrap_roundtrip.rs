use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use myosu_games::{CfrGame, StrategyResponse};
use myosu_games_kuhn::{
    KuhnCard, KuhnEdge, KuhnGame, KuhnSolver, KuhnStrategyQuery, decode_strategy_query,
    decode_strategy_response, encode_strategy_query, encode_strategy_response, recommended_edge,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let output_dir = PathBuf::from(required_arg(&mut args, "<output-dir>")?);
    let iterations = match args.next() {
        Some(value) => value.parse::<usize>()?,
        None => 0,
    };
    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    fs::create_dir_all(&output_dir)?;
    let checkpoint_file = output_dir.join("checkpoint.bin");
    let query_file = output_dir.join("query.bin");
    let response_file = output_dir.join("response.bin");

    let mut solver = KuhnSolver::new();
    solver.train(iterations);
    solver.save(&checkpoint_file)?;

    let query = bootstrap_query()?;
    fs::write(&query_file, encode_strategy_query(&query)?)?;

    let restored = KuhnSolver::load(&checkpoint_file)?;
    let decoded_query = decode_strategy_query(&fs::read(&query_file)?)?;
    let expected = restored.answer(decoded_query);
    fs::write(&response_file, encode_strategy_response(&expected)?)?;

    let observed = decode_strategy_response(&fs::read(&response_file)?)?;
    if !observed.is_valid() {
        return Err("kuhn response is not a valid probability distribution".into());
    }

    let l1_distance = l1_distance(&expected, &observed);
    let score = 1.0 / (1.0 + l1_distance.max(0.0));
    let exact_match = l1_distance < f64::EPSILON;
    let recommended = recommended_edge(&observed)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string());

    println!("BOOTSTRAP game=kuhn");
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

fn bootstrap_query() -> Result<KuhnStrategyQuery, Box<dyn Error>> {
    let opening = KuhnGame::root().apply(KuhnEdge::Deal {
        p1: KuhnCard::King,
        p2: KuhnCard::Queen,
    });
    let info = opening
        .info()
        .ok_or("opening player turn should expose info")?;

    Ok(KuhnStrategyQuery::new(info))
}

fn l1_distance(
    expected: &StrategyResponse<KuhnEdge>,
    observed: &StrategyResponse<KuhnEdge>,
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
