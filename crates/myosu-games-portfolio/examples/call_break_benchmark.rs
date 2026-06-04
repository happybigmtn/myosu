use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use myosu_games_portfolio::{
    CallBreakBenchmarkDossier, core::trick_taking::call_break_scenario_pack,
};

const DEFAULT_BENCHMARK_ID: &str = "call-break-pack-v1";
const DEFAULT_OUTPUT_DIR: &str = "outputs/solver-promotion/call-break";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let benchmark_id = args
        .next()
        .unwrap_or_else(|| DEFAULT_BENCHMARK_ID.to_string());
    if args.next().is_some() {
        return Err("usage: cargo run -p myosu-games-portfolio --example call_break_benchmark -- [benchmark-id]".into());
    }

    let output_dir = env::var("MYOSU_CALL_BREAK_BENCHMARK_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root().join(DEFAULT_OUTPUT_DIR));
    fs::create_dir_all(&output_dir)?;

    let dossier = CallBreakBenchmarkDossier::build(&benchmark_id)?;

    // Action histogram (e.g. call-trump=8 trump-control=7 follow-suit=7 ...) for the human reader.
    let mut histogram: BTreeMap<String, usize> = BTreeMap::new();
    for action in dossier.recommendations.values() {
        *histogram.entry(action.clone()).or_insert(0) += 1;
    }
    let histogram_repr: String = histogram
        .iter()
        .map(|(action, count)| format!("{action}={count}"))
        .collect::<Vec<_>>()
        .join(" ");

    println!(
        "CALL_BREAK_BENCHMARK benchmark_id={} method={} engine_tier={} rule_file={} scenario_count={} recommendation_count={} metric_name={} metric_value={:.0} threshold={:.0} passing={} scenario_hash={} histogram=[{}]",
        dossier.benchmark_id,
        dossier.benchmark_method,
        dossier.engine_tier,
        dossier.rule_file,
        dossier.scenario_count,
        dossier.recommendation_count,
        dossier.metric_name,
        dossier.metric_value,
        dossier.threshold,
        if dossier.passing { "yes" } else { "no" },
        dossier.scenario_hash,
        histogram_repr,
    );

    for scenario in call_break_scenario_pack() {
        let action = dossier
            .recommendations
            .get(scenario.scenario_id)
            .map(String::as_str)
            .unwrap_or("missing");
        println!(
            "CALL_BREAK_SCENARIO scenario_id={} decision=\"{}\" trump_count={} winners={} void_suits={} contract_pressure={} penalty_pressure={} cards_in_trick={} follow_suit_forced={} nil_viable={} recommended_action={action}",
            scenario.scenario_id,
            shell_escape(scenario.decision),
            scenario.trump_count,
            scenario.winners,
            scenario.void_suits,
            scenario.contract_pressure,
            scenario.penalty_pressure,
            scenario.cards_in_trick,
            yes_no(scenario.follow_suit_forced),
            yes_no(scenario.nil_viable),
        );
    }

    let json_path = output_dir.join("call-break-benchmark-dossier.json");
    fs::write(&json_path, serde_json::to_string_pretty(&dossier)?)?;
    println!(
        "CALL_BREAK_BENCHMARK_DOSSIER path={} bytes={}",
        json_path.display(),
        fs::metadata(&json_path).map(|meta| meta.len()).unwrap_or(0),
    );

    if !dossier.passing {
        return Err(format!(
            "call-break benchmark {} did not pass (recommendation_count={} scenario_count={})",
            benchmark_id, dossier.recommendation_count, dossier.scenario_count
        )
        .into());
    }

    println!("CALL_BREAK_BENCHMARK status=ok");
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn shell_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
