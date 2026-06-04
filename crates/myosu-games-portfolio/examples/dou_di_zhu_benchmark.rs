use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use myosu_games_portfolio::{
    DouDiZhuBenchmarkDossier, core::shedding::dou_di_zhu_scenario_pack,
};

const DEFAULT_BENCHMARK_ID: &str = "dou-di-zhu-pack-v1";
const DEFAULT_OUTPUT_DIR: &str = "outputs/solver-promotion/dou-di-zhu";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let benchmark_id = args
        .next()
        .unwrap_or_else(|| DEFAULT_BENCHMARK_ID.to_string());
    if args.next().is_some() {
        return Err("usage: cargo run -p myosu-games-portfolio --example dou_di_zhu_benchmark -- [benchmark-id]".into());
    }

    let output_dir = env::var("MYOSU_DOU_DI_ZHU_BENCHMARK_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| repo_root().join(DEFAULT_OUTPUT_DIR));
    fs::create_dir_all(&output_dir)?;

    let dossier = DouDiZhuBenchmarkDossier::build(&benchmark_id)?;

    // Action histogram (e.g. preserve-bomb=8 landlord-bid=8 shed-lowest=6 ...) for the human reader.
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
        "DOU_DI_ZHU_BENCHMARK benchmark_id={} method={} engine_tier={} rule_file={} scenario_count={} recommendation_count={} metric_name={} metric_value={:.0} threshold={:.0} passing={} scenario_hash={} histogram=[{}]",
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

    for scenario in dou_di_zhu_scenario_pack() {
        let action = dossier
            .recommendations
            .get(scenario.scenario_id)
            .map(String::as_str)
            .unwrap_or("missing");
        println!(
            "DOU_DI_ZHU_SCENARIO scenario_id={} decision=\"{}\" bomb_count={} control_combos={} low_singles={} opponents_min_cards={} danger_opponents={} next_actor_cards={} on_lead={} play_options={} finishing_plays={} bomb_only_escape={} forced_pass={} lead_rank_pressure={} recommended_action={action}",
            scenario.scenario_id,
            shell_escape(scenario.decision),
            scenario.bomb_count,
            scenario.control_combos,
            scenario.low_singles,
            scenario.opponents_min_cards,
            scenario.danger_opponents,
            scenario.next_actor_cards,
            yes_no(scenario.on_lead),
            scenario.play_options,
            scenario.finishing_plays,
            yes_no(scenario.bomb_only_escape),
            yes_no(scenario.forced_pass),
            scenario.lead_rank_pressure,
        );
    }

    let json_path = output_dir.join("dou-di-zhu-benchmark-dossier.json");
    fs::write(&json_path, serde_json::to_string_pretty(&dossier)?)?;
    println!(
        "DOU_DI_ZHU_BENCHMARK_DOSSIER path={} bytes={}",
        json_path.display(),
        fs::metadata(&json_path).map(|meta| meta.len()).unwrap_or(0),
    );

    if !dossier.passing {
        return Err(format!(
            "dou-di-zhu benchmark {} did not pass (recommendation_count={} scenario_count={})",
            benchmark_id, dossier.recommendation_count, dossier.scenario_count
        )
        .into());
    }

    println!("DOU_DI_ZHU_BENCHMARK status=ok");
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
