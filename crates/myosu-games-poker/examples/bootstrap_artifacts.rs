use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use myosu_games_poker::NlheAbstractionStreet;
use myosu_games_poker::NlheBlueprint;
use myosu_games_poker::encode_strategy_query;
use myosu_games_poker::write_encoder_dir;
use rbp_cards::Isomorphism;
use rbp_cards::Observation;
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
    let preflop_observation = Observation::try_from("AcKh")?;
    let streets = BTreeMap::from([(
        NlheAbstractionStreet::Preflop,
        BTreeMap::from([(
            Isomorphism::from(preflop_observation),
            Abstraction::from(42_i16),
        )]),
    )]);

    let manifest = write_encoder_dir(&encoder_dir, streets)?;
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
