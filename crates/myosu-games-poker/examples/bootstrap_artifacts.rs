use std::env;
use std::fs;
use std::path::PathBuf;

use myosu_games_poker::NlheBlueprint;
use myosu_games_poker::bootstrap_encoder_streets;
use myosu_games_poker::encode_strategy_query;
use myosu_games_poker::load_encoder_bundle;
use myosu_games_poker::write_encoder_dir;
use rbp_gameplay::Abstraction;
use rbp_gameplay::Edge;
use rbp_gameplay::Odds;
use rbp_nlhe::NlheInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(encoder_dir) = args.next() else {
        return Err("usage: cargo run -p myosu-games-poker --example bootstrap_artifacts -- <encoder-dir> <query-file>".into());
    };
    let Some(query_file) = args.next() else {
        return Err("usage: cargo run -p myosu-games-poker --example bootstrap_artifacts -- <encoder-dir> <query-file>".into());
    };
    if args.next().is_some() {
        return Err("expected exactly two positional arguments".into());
    }

    let encoder_dir = PathBuf::from(encoder_dir);
    let query_file = PathBuf::from(query_file);
    let streets = bootstrap_encoder_streets();

    let manifest = write_encoder_dir(&encoder_dir, streets)?;
    let artifact_summary = load_encoder_bundle(&encoder_dir)?.summary();
    let query = NlheBlueprint::query_for_info(&sample_info());
    let query_bytes = encode_strategy_query(&query)?;
    let query_parent = query_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&query_parent)?;
    fs::write(&query_file, query_bytes)?;

    println!("BOOTSTRAP encoder_dir={}", encoder_dir.display());
    println!("BOOTSTRAP query_file={}", query_file.display());
    println!("BOOTSTRAP total_sha256={}", manifest.total_sha256);
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

    Ok(())
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
