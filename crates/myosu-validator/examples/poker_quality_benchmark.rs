//! Operator-facing quality benchmark for the F-003 / NEM-001B poker
//! reference-vs-candidate surface.
//!
//! F-003 requires the repo to expose a truthful training-quality
//! threshold that "actually varies with solver quality" so operators
//! know how many MCCFR iterations to train before serving a strategy.
//! The current truthful surface for Poker is *not* exploitability:
//! the checked-in bootstrap encoder is too sparse to run any
//! positive-iteration MCCFR training (the upstream `bootstrap_reference_solver`
//! returns `isomorphism not found` past the zero-th iteration against the
//! stage-0 bootstrap fixtures — see
//! `crates/myosu-games-poker/src/benchmark.rs:571-591` and the
//! `benchmark_reports_sparse_encoder_failure_cleanly` test), so the
//! F-003 / NEM-001B substitute is a convex mix-ladder between the
//! closed-form reference profile (`mix=1.0`) and a uniform-weight
//! perturbation (`mix=0.0`).
//!
//! This example reuses the public `poker_quality_benchmark_points`
//! helper (defined next to the validator scoring code so the unit test
//! and the operator-facing surface share one ladder) and emits a
//! stable `POKER_QUALITY_BENCHMARK_*` key=value report. The companion
//! e2e proof (`tests/e2e/poker_quality_benchmark.sh`) runs this
//! binary and asserts the recommendation the operator guide quotes
//! stays truthful across solver-constant changes.
//!
//! Usage:
//!
//! ```bash
//! SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example poker_quality_benchmark
//! ```
//!
//! Optional positional argument overrides the default mix ladder
//! (each value is a `f32` in `[0.0, 1.0]`):
//!
//! ```bash
//! SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example poker_quality_benchmark -- 0.0 0.5 1.0
//! ```

use std::error::Error;

use myosu_games_poker::POKER_REFERENCE_SELF_MATCH_COUNT;
use myosu_games_poker::POKER_REFERENCE_SELF_MATCH_L1;
use myosu_validator::validation::{
    POKER_REFERENCE_LADDER, POKER_USEFUL_REFERENCE_MATCH_RATIO,
    PokerQualityBenchmarkReport, poker_quality_benchmark_points,
};

fn main() -> Result<(), Box<dyn Error>> {
    let ladder = match parse_ladder(std::env::args().skip(1))? {
        Some(parsed) => parsed,
        None => POKER_REFERENCE_LADDER.to_vec(),
    };

    println!("POKER_QUALITY_BENCHMARK game=nlhe-heads-up");
    println!("POKER_QUALITY_BENCHMARK surface=mixed_bootstrap_reference_l1");
    println!(
        "POKER_QUALITY_BENCHMARK reference_self_match_count={}",
        POKER_REFERENCE_SELF_MATCH_COUNT
    );
    println!(
        "POKER_QUALITY_BENCHMARK reference_self_match_l1={:.6}",
        POKER_REFERENCE_SELF_MATCH_L1
    );
    println!(
        "POKER_QUALITY_BENCHMARK match_ratio_threshold={:.6}",
        POKER_USEFUL_REFERENCE_MATCH_RATIO
    );
    println!("POKER_QUALITY_BENCHMARK mix_ladder={}", format_ladder(&ladder));

    let points = poker_quality_benchmark_points(&ladder)?;
    for point in &points {
        println!(
            "POKER_QUALITY_BENCHMARK_POINT mix={:.6} mean_l1_distance={:.6} exact_action_matches={} scenario_count={} exact_action_match_ratio={:.6}",
            point.mix,
            point.mean_l1_distance,
            point.exact_action_matches,
            point.scenario_count,
            point.exact_action_match_ratio
        );
    }

    let report = PokerQualityBenchmarkReport::from_poker_quality_benchmark_points(
        &points,
        POKER_USEFUL_REFERENCE_MATCH_RATIO,
    );

    match (
        report.recommended_minimum_mix,
        report.recommended_match_ratio,
    ) {
        (Some(min_mix), Some(match_ratio)) => {
            println!(
                "POKER_QUALITY_BENCHMARK_RECOMMENDATION recommended_minimum_mix={:.6} exact_action_match_ratio={:.6} threshold={:.6}",
                min_mix, match_ratio, report.match_ratio_threshold
            );
        }
        (None, None) => {
            println!(
                "POKER_QUALITY_BENCHMARK_RECOMMENDATION status=below_threshold threshold={:.6}",
                report.match_ratio_threshold
            );
        }
        _ => unreachable!("PokerQualityBenchmarkReport is internally consistent"),
    }

    Ok(())
}

fn parse_ladder(args: impl Iterator<Item = String>) -> Result<Option<Vec<f32>>, Box<dyn Error>> {
    let ladder: Vec<f32> = args
        .map(|raw| {
            raw.parse::<f32>()
                .map_err(|error| format!("invalid mix value `{raw}`: {error}").into())
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    if ladder.is_empty() {
        return Ok(None);
    }
    for window in ladder.windows(2) {
        if window[0] >= window[1] {
            return Err(format!(
                "mix ladder must be strictly increasing (saw {} then {})",
                window[0], window[1]
            )
            .into());
        }
    }
    Ok(Some(ladder))
}

fn format_ladder(ladder: &[f32]) -> String {
    ladder
        .iter()
        .map(|mix| format!("{:.6}", mix))
        .collect::<Vec<_>>()
        .join(",")
}
