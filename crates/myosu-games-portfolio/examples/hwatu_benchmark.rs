use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use myosu_games_portfolio::{
    HwatuBenchmarkDossier, core::hanafuda::hwatu_scenario_pack,
};

const DEFAULT_BENCHMARK_ID: &str = "hwatu-pack-v1";
const DEFAULT_OUTPUT_DIR: &str = "outputs/solver-promotion/hwatu-go-stop";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let benchmark_id = args
        .next()
        .unwrap_or_else(|| DEFAULT_BENCHMARK_ID.to_string());
    if args.next().is_some() {
        return Err("usage: cargo run -p myosu-games-portfolio --example hwatu_benchmark -- [benchmark-id]".into());
    }

    let output_dir = env::var("MYOSU_HWATU_BENCHMARK_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root().join(DEFAULT_OUTPUT_DIR));
    fs::create_dir_all(&output_dir)?;

    let dossier = HwatuBenchmarkDossier::build(&benchmark_id)?;

    // Action histogram (e.g. call-go=8 stop-round=8 koi-koi=0 ...) for the human reader.
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
        "HWATU_BENCHMARK benchmark_id={} method={} engine_tier={} rule_file={} scenario_count={} recommendation_count={} metric_name={} metric_value={:.0} threshold={:.0} passing={} scenario_hash={} histogram=[{}]",
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

    for scenario in hwatu_scenario_pack() {
        let action = dossier
            .recommendations
            .get(scenario.scenario_id)
            .map(String::as_str)
            .unwrap_or("missing");
        println!(
            "HWATU_SCENARIO scenario_id={} decision=\"{}\" points={} bright_count={} ribbon_yaku={} animal_yaku={} bonus_cards={} yaku_count={} bright_capture_options={} opponent_pressure={} hand_count={} decision_window={} locked_points={} continuation_calls={} upside_capture_options={} max_upside_gain={} recommended_action={action}",
            scenario.scenario_id,
            shell_escape(scenario.decision),
            scenario.points,
            scenario.bright_count,
            scenario.ribbon_yaku,
            scenario.animal_yaku,
            scenario.bonus_cards,
            scenario.yaku_count,
            scenario.bright_capture_options,
            scenario.opponent_pressure,
            scenario.hand_count,
            yes_no(scenario.decision_window),
            scenario.locked_points,
            scenario.continuation_calls,
            scenario.upside_capture_options,
            scenario.max_upside_gain,
        );
    }

    let json_path = output_dir.join("hwatu-benchmark-dossier.json");
    fs::write(&json_path, serde_json::to_string_pretty(&dossier)?)?;
    println!(
        "HWATU_BENCHMARK_DOSSIER path={} bytes={}",
        json_path.display(),
        fs::metadata(&json_path).map(|meta| meta.len()).unwrap_or(0),
    );

    if !dossier.passing {
        return Err(format!(
            "hwatu benchmark {} did not pass (recommendation_count={} scenario_count={})",
            benchmark_id, dossier.recommendation_count, dossier.scenario_count
        )
        .into());
    }

    println!("HWATU_BENCHMARK status=ok");
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
