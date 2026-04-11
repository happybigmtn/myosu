use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use myosu_games_poker::{
    PokerSolver, benchmark_solver_against_reference, bootstrap_encoder_streets,
    bootstrap_reference_solver, load_encoder_bundle, load_encoder_dir, write_encoder_dir,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let output_dir = PathBuf::from(required_arg(&mut args, "<output-dir>")?);
    let external_candidate_checkpoint = args.next().map(PathBuf::from);
    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    fs::create_dir_all(&output_dir)?;
    let encoder_dir = output_dir.join("encoder");
    write_encoder_dir(&encoder_dir, bootstrap_encoder_streets())?;
    let artifact_bundle = load_encoder_bundle(&encoder_dir)?;
    let artifact_summary = artifact_bundle.summary();

    let (candidate_source, candidate_checkpoint, candidate_solver) =
        if let Some(checkpoint) = external_candidate_checkpoint {
            let solver = PokerSolver::load(&checkpoint, load_encoder_dir(&encoder_dir)?)?;
            ("external-checkpoint", checkpoint, solver)
        } else {
            let checkpoint = output_dir.join("candidate-checkpoint.bin");
            let solver = PokerSolver::new(load_encoder_dir(&encoder_dir)?);
            solver.save(&checkpoint)?;
            ("bootstrap-zero-checkpoint", checkpoint, solver)
        };

    let reference_checkpoint = output_dir.join("reference-checkpoint.bin");
    let reference_solver = bootstrap_reference_solver(load_encoder_dir(&encoder_dir)?)?;
    reference_solver.save(&reference_checkpoint)?;

    let report = benchmark_solver_against_reference(&candidate_solver, &reference_solver)?;

    println!("BENCHMARK game=nlhe-heads-up");
    println!("BENCHMARK output_dir={}", output_dir.display());
    println!("BENCHMARK encoder_dir={}", display_path(&encoder_dir));
    println!("BENCHMARK candidate_source={candidate_source}");
    println!(
        "BENCHMARK candidate_checkpoint={}",
        display_path(&candidate_checkpoint)
    );
    println!(
        "BENCHMARK reference_checkpoint={}",
        display_path(&reference_checkpoint)
    );
    println!(
        "BENCHMARK artifact_streets={}",
        artifact_summary.available_streets_token()
    );
    println!(
        "BENCHMARK complete_streets={}",
        artifact_summary.complete_streets_token()
    );
    println!(
        "BENCHMARK sampled_streets={}",
        artifact_summary.sampled_streets_token()
    );
    println!(
        "BENCHMARK missing_streets={}",
        artifact_summary.missing_streets_token()
    );
    println!("BENCHMARK coverage={}", artifact_summary.coverage_token());
    println!(
        "BENCHMARK postflop_complete={}",
        artifact_summary.postflop_complete
    );
    println!("BENCHMARK total_entries={}", artifact_summary.total_entries);
    println!("BENCHMARK scenario_count={}", report.scenario_count);
    println!("BENCHMARK unique_query_count={}", report.unique_query_count);
    println!(
        "BENCHMARK exact_distribution_matches={}",
        report.exact_distribution_matches
    );
    println!(
        "BENCHMARK exact_action_matches={}",
        report.exact_action_matches
    );
    println!(
        "BENCHMARK recommendation_agreement={:.6}",
        report.recommendation_agreement()
    );
    println!("BENCHMARK mean_l1_distance={:.6}", report.mean_l1_distance);
    println!("BENCHMARK max_l1_distance={:.6}", report.max_l1_distance);
    println!(
        "BENCHMARK by_street_mean_l1={}",
        report.street_mean_l1_token()
    );
    println!(
        "BENCHMARK by_street_action_matches={}",
        report.street_action_match_token()
    );
    println!("BENCHMARK benchmark_surface=repo-owned-reference-pack");
    println!("BENCHMARK engine_tier=dedicated-reference-pack");

    Ok(())
}

fn required_arg(
    args: &mut impl Iterator<Item = String>,
    label: &'static str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing required argument {label}").into())
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}
