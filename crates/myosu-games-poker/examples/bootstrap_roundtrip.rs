use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use myosu_games::StrategyResponse;
use myosu_games_poker::{
    NlheBlueprint, PokerSolver, RbpNlheEdge, bootstrap_encoder_streets, decode_strategy_query,
    decode_strategy_response, encode_strategy_query, encode_strategy_response, load_encoder_bundle,
    load_encoder_dir, recommended_edge, write_encoder_dir,
};
use rbp_gameplay::{Abstraction, Edge, Odds};
use rbp_nlhe::NlheInfo;

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
    if iterations != 0 {
        return Err(
            "NLHE bootstrap roundtrip uses a sparse encoder and supports 0 iterations only".into(),
        );
    }

    fs::create_dir_all(&output_dir)?;
    let encoder_dir = output_dir.join("encoder");
    let checkpoint_file = output_dir.join("checkpoint.bin");
    let query_file = output_dir.join("query.bin");
    let response_file = output_dir.join("response.bin");

    write_encoder_dir(&encoder_dir, bootstrap_encoder_streets())?;
    let artifact_bundle = load_encoder_bundle(&encoder_dir)?;
    let artifact_summary = artifact_bundle.summary();
    let solver = PokerSolver::new(artifact_bundle.encoder);
    solver.save(&checkpoint_file)?;

    let query = NlheBlueprint::query_for_info(&sample_info());
    fs::write(&query_file, encode_strategy_query(&query)?)?;

    let restored = PokerSolver::load(&checkpoint_file, load_encoder_dir(&encoder_dir)?)?;
    let decoded_query = decode_strategy_query(&fs::read(&query_file)?)?;
    let expected = restored.answer(decoded_query);
    fs::write(&response_file, encode_strategy_response(&expected)?)?;

    let observed = decode_strategy_response(&fs::read(&response_file)?)?;
    if !observed.is_valid() {
        return Err("poker response is not a valid probability distribution".into());
    }

    let l1_distance = l1_distance(&expected, &observed);
    let score = 1.0 / (1.0 + l1_distance.max(0.0));
    let exact_match = l1_distance < f64::EPSILON;
    let recommended = recommended_edge(&observed)
        .map(|edge| format!("{edge:?}"))
        .unwrap_or_else(|| "none".to_string());

    println!("BOOTSTRAP game=nlhe-heads-up");
    println!("BOOTSTRAP output_dir={}", output_dir.display());
    println!("BOOTSTRAP encoder_dir={}", display_path(&encoder_dir));
    println!(
        "BOOTSTRAP checkpoint_file={}",
        display_path(&checkpoint_file)
    );
    println!("BOOTSTRAP query_file={}", display_path(&query_file));
    println!("BOOTSTRAP response_file={}", display_path(&response_file));
    println!("BOOTSTRAP iterations={iterations}");
    println!(
        "BOOTSTRAP artifact_streets={}",
        artifact_summary.available_streets_token()
    );
    println!(
        "BOOTSTRAP complete_streets={}",
        artifact_summary.complete_streets_token()
    );
    println!(
        "BOOTSTRAP sampled_streets={}",
        artifact_summary.sampled_streets_token()
    );
    println!(
        "BOOTSTRAP missing_streets={}",
        artifact_summary.missing_streets_token()
    );
    println!("BOOTSTRAP coverage={}", artifact_summary.coverage_token());
    println!("BOOTSTRAP total_entries={}", artifact_summary.total_entries);
    println!(
        "BOOTSTRAP preflop_entries={}",
        artifact_summary.preflop_entries()
    );
    println!(
        "BOOTSTRAP postflop_complete={}",
        artifact_summary.postflop_complete
    );
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

fn sample_info() -> NlheInfo {
    let subgame = vec![Edge::Check, Edge::Raise(Odds::new(1, 2))]
        .into_iter()
        .collect();
    let choices = vec![Edge::Fold, Edge::Call, Edge::Raise(Odds::new(1, 1))]
        .into_iter()
        .collect();
    let bucket = Abstraction::from(42_i16);

    NlheInfo::from((subgame, bucket, choices))
}

fn l1_distance(
    expected: &StrategyResponse<RbpNlheEdge>,
    observed: &StrategyResponse<RbpNlheEdge>,
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
