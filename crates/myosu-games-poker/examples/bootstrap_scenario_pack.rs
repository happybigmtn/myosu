use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use myosu_games_poker::{
    NlheAbstractionStreet, NlheStrategyRequest, NlheTablePosition, bootstrap_encoder_streets,
    bootstrap_scenarios, load_encoder_bundle, write_encoder_dir,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let output_dir = PathBuf::from(required_arg(&mut args, "<output-dir>")?);
    if let Some(extra) = args.next() {
        return Err(format!("unexpected extra argument `{extra}`").into());
    }

    fs::create_dir_all(&output_dir)?;
    let encoder_dir = output_dir.join("encoder");
    write_encoder_dir(&encoder_dir, bootstrap_encoder_streets())?;
    let artifact_bundle = load_encoder_bundle(&encoder_dir)?;
    let artifact_summary = artifact_bundle.summary();
    let scenarios = bootstrap_scenarios();

    let mut per_street = BTreeMap::new();
    let mut distinct_query_keys = BTreeSet::new();
    let mut distinct_buckets = BTreeSet::new();

    println!("SCENARIO_PACK output_dir={}", output_dir.display());
    println!("SCENARIO_PACK encoder_dir={}", encoder_dir.display());
    println!(
        "SCENARIO_PACK artifact_streets={}",
        artifact_summary.available_streets_token()
    );
    println!(
        "SCENARIO_PACK complete_streets={}",
        artifact_summary.complete_streets_token()
    );
    println!(
        "SCENARIO_PACK sampled_streets={}",
        artifact_summary.sampled_streets_token()
    );
    println!(
        "SCENARIO_PACK coverage={}",
        artifact_summary.coverage_token()
    );

    for scenario in &scenarios {
        let request = NlheStrategyRequest::from_observation_text(
            NlheTablePosition::Button,
            scenario.observation,
            Vec::new(),
            0,
        )?;
        let query = request.query_with_encoder(&artifact_bundle.encoder)?;
        *per_street.entry(scenario.street).or_insert(0usize) += 1;
        distinct_query_keys.insert((query.info.subgame, query.info.bucket, query.info.choices));
        distinct_buckets.insert((scenario.street.as_str().to_string(), query.info.bucket));
        println!(
            "SCENARIO label={} street={} observation={} bucket={} subgame={} choices={}",
            scenario.label,
            scenario.street.as_str(),
            scenario.observation,
            query.info.bucket,
            query.info.subgame,
            query.info.choices
        );
    }

    println!("SCENARIO_PACK scenario_count={}", scenarios.len());
    println!(
        "SCENARIO_PACK by_street={}",
        street_counts_token(&per_street)
    );
    println!(
        "SCENARIO_PACK unique_query_keys={}",
        distinct_query_keys.len()
    );
    println!("SCENARIO_PACK unique_buckets={}", distinct_buckets.len());

    Ok(())
}

fn required_arg(
    args: &mut impl Iterator<Item = String>,
    label: &'static str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing required argument {label}").into())
}

fn street_counts_token(counts: &BTreeMap<NlheAbstractionStreet, usize>) -> String {
    [
        NlheAbstractionStreet::Preflop,
        NlheAbstractionStreet::Flop,
        NlheAbstractionStreet::Turn,
        NlheAbstractionStreet::River,
    ]
    .into_iter()
    .map(|street| {
        format!(
            "{}={}",
            street.as_str(),
            counts.get(&street).copied().unwrap_or(0)
        )
    })
    .collect::<Vec<_>>()
    .join(",")
}
