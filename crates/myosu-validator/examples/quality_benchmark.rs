//! Operator-facing quality benchmark for the Liar's Dice MCCFR solver.
//!
//! F-003 / F-007 requires the repo to expose a truthful training-quality
//! threshold that "actually varies with solver quality" so operators know
//! how many MCCFR iterations to train before serving a strategy. The
//! current truthful surface for Liar's Dice is exact best-response
//! exploitability, measured from a fresh `LiarsDiceSolver::new()` start so
//! the result is independent of the same-checkpoint validator happy path
//! the spec explicitly calls out as not-a-convergence-metric.
//!
//! This example reuses the public `liars_dice_benchmark_points` helper
//! (defined next to the validator scoring code so the unit test and the
//! operator-facing surface share one ladder) and emits a stable
//! `QUALITY_BENCHMARK_*` key=value report. The companion e2e proof
//! (`tests/e2e/quality_benchmark_liars_dice.sh`) runs this binary and
//! asserts the recommendation the operator guide currently quotes
//! (512 iterations at the `0.70` threshold) stays truthful across
//! solver-constant changes.
//!
//! Usage:
//!
//! ```bash
//! cargo run -p myosu-validator --example quality_benchmark
//! SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example quality_benchmark
//! ```
//!
//! Optional positional argument overrides the default iteration ladder:
//!
//! ```bash
//! cargo run -p myosu-validator --example quality_benchmark -- 0 64 128 256 512 1024
//! ```

use std::error::Error;

use myosu_validator::validation::{
    LIARS_DICE_SOLVER_TREES, LIARS_DICE_USEFUL_EXPLOITABILITY_THRESHOLD, QualityBenchmarkReport,
    liars_dice_benchmark_points,
};

/// Default iteration ladder matching the operator guide and the
/// `quality_benchmark_liars_dice_exploitability_converges` unit test. The
/// 1024-iteration point is included as a "best local target" so operators
/// can see the curve past the recommendation.
const DEFAULT_LADDER: &[usize] = &[0, 128, 256, 512, 1024];

fn main() -> Result<(), Box<dyn Error>> {
    let ladder = match parse_ladder(std::env::args().skip(1))? {
        Some(parsed) => parsed,
        None => DEFAULT_LADDER.to_vec(),
    };

    println!("QUALITY_BENCHMARK game=liars-dice");
    println!("QUALITY_BENCHMARK surface=exact_best_response_exploitability");
    println!("QUALITY_BENCHMARK solver_trees={}", LIARS_DICE_SOLVER_TREES);
    println!(
        "QUALITY_BENCHMARK exploitability_threshold={:.6}",
        f64::from(LIARS_DICE_USEFUL_EXPLOITABILITY_THRESHOLD)
    );
    println!("QUALITY_BENCHMARK iterations={}", format_ladder(&ladder));

    let points = liars_dice_benchmark_points(&ladder)?;
    for point in &points {
        println!(
            "QUALITY_BENCHMARK_POINT iterations={} exploitability={:.6}",
            point.iterations, point.exploitability
        );
    }

    let report = QualityBenchmarkReport::from_benchmark_points(
        &points,
        LIARS_DICE_USEFUL_EXPLOITABILITY_THRESHOLD,
    );

    match (
        report.recommended_minimum_iterations,
        report.recommended_exploitability,
    ) {
        (Some(min_iters), Some(exploitability)) => {
            println!(
                "QUALITY_BENCHMARK_RECOMMENDATION recommended_minimum_iterations={} exploitability={:.6} threshold={:.6}",
                min_iters,
                exploitability,
                f64::from(report.exploitability_threshold)
            );
        }
        (None, None) => {
            println!(
                "QUALITY_BENCHMARK_RECOMMENDATION status=below_threshold threshold={:.6}",
                f64::from(report.exploitability_threshold)
            );
        }
        _ => unreachable!("QualityBenchmarkReport is internally consistent"),
    }

    Ok(())
}

fn parse_ladder(args: impl Iterator<Item = String>) -> Result<Option<Vec<usize>>, Box<dyn Error>> {
    let ladder: Vec<usize> = args
        .map(|raw| {
            raw.parse::<usize>()
                .map_err(|error| format!("invalid iteration count `{raw}`: {error}").into())
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    if ladder.is_empty() {
        return Ok(None);
    }
    for window in ladder.windows(2) {
        if window[0] >= window[1] {
            return Err(format!(
                "iteration ladder must be strictly increasing (saw {} then {})",
                window[0], window[1]
            )
            .into());
        }
    }
    Ok(Some(ladder))
}

fn format_ladder(ladder: &[usize]) -> String {
    ladder
        .iter()
        .map(|count| count.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
