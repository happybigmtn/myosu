use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use myosu_games::{CfrGame, StrategyResponse};
use myosu_games_liars_dice::{
    LiarsDiceEdge, LiarsDiceGame, LiarsDiceSolver, LiarsDiceStrategyQuery, decode_strategy_query,
    decode_strategy_response, encode_strategy_query, encode_strategy_response, recommended_edge,
};

const SOLVER_TREES: usize = 1 << 10;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let output_dir = PathBuf::from(required_arg(&mut args, "<output-dir>")?);
    let iterations = match args.next() {
        Some(value) => value.parse::<usize>()?,
        None => 64,
    };
    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    fs::create_dir_all(&output_dir)?;
    let checkpoint_file = output_dir.join("checkpoint.bin");
    let query_file = output_dir.join("strength-query.bin");
    let response_file = output_dir.join("strength-response.bin");

    let mut solver = LiarsDiceSolver::<SOLVER_TREES>::new();
    let training = solver.train_select_best(iterations)?;
    solver.save(&checkpoint_file)?;

    let query = strength_query()?;
    fs::write(&query_file, encode_strategy_query(&query)?)?;

    let restored = LiarsDiceSolver::<SOLVER_TREES>::load(&checkpoint_file)?;
    let decoded_query = decode_strategy_query(&fs::read(&query_file)?)?;
    let expected = restored.answer(decoded_query);
    fs::write(&response_file, encode_strategy_response(&expected)?)?;

    let observed = decode_strategy_response(&fs::read(&response_file)?)?;
    if !observed.is_valid() {
        return Err("liar's dice response is not a valid probability distribution".into());
    }

    let l1_distance = l1_distance(&expected, &observed);
    let score = 1.0 / (1.0 + l1_distance.max(0.0));
    let exact_match = l1_distance < f64::EPSILON;
    let recommended = recommended_edge(&observed)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string());

    println!("STRENGTH game=liars-dice");
    println!("STRENGTH output_dir={}", output_dir.display());
    println!(
        "STRENGTH checkpoint_file={}",
        display_path(&checkpoint_file)
    );
    println!("STRENGTH query_file={}", display_path(&query_file));
    println!("STRENGTH response_file={}", display_path(&response_file));
    println!("STRENGTH iterations={iterations}");
    println!("STRENGTH trained_epochs={}", training.end_epochs);
    println!("STRENGTH selected_epochs={}", training.selected_epochs);
    println!(
        "STRENGTH exact_exploitability={:.6}",
        training.selected_exploitability
    );
    println!(
        "STRENGTH final_exploitability={:.6}",
        training.final_exploitability
    );
    println!("STRENGTH exact_match={exact_match}");
    println!("STRENGTH l1_distance={l1_distance:.6}");
    println!("STRENGTH score={score:.6}");
    println!("STRENGTH engine_tier=dedicated-mccfr");
    println!("STRENGTH solver_family=liars-dice-cfr");
    println!("STRENGTH legal_actions={}", observed.actions.len());
    println!("STRENGTH deterministic=true");
    println!("STRENGTH recommended_action={recommended}");

    Ok(())
}

fn required_arg(
    args: &mut impl Iterator<Item = String>,
    label: &'static str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing required argument {label}").into())
}

fn strength_query() -> Result<LiarsDiceStrategyQuery, Box<dyn Error>> {
    let opening = LiarsDiceGame::root().apply(LiarsDiceEdge::Roll { p1: 2, p2: 5 });
    let info = opening
        .info()
        .ok_or("opening player turn should expose info")?;

    Ok(LiarsDiceStrategyQuery::new(info))
}

fn l1_distance(
    expected: &StrategyResponse<LiarsDiceEdge>,
    observed: &StrategyResponse<LiarsDiceEdge>,
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
