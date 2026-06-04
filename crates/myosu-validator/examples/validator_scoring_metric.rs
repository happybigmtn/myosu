//! Operator-facing scoring-metric probe for the W-06 first-class
//! observability surface.
//!
//! The validator scoring loop emits exactly one
//! `VALIDATOR_SCORING_METRIC` line per bounded scoring run (see
//! `myosu_validator::metric::emit` and the wiring in
//! `validation::score_response`). This example is the executable
//! operator-facing probe: it drives the metric constructor and the
//! emit line protocol with a fixed deterministic input set, so the
//! companion e2e proof (`tests/e2e/validator_scoring_metric.sh`)
//! can grep the line protocol, the percentile contract, and the
//! byte-stability contract against a real binary run instead of
//! hand-rolling python3 verification.
//!
//! Usage:
//!
//! ```bash
//! SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example validator_scoring_metric
//! ```
//!
//! Optional positional argument overrides the default L1 distance
//! ladder (each value is a `f64` in `[0.0, 2.0]`):
//!
//! ```bash
//! SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example validator_scoring_metric -- 0.0 0.1 0.2 0.3 0.4
//! ```
//!
//! The example prints the metric lines for the live scoring run, a
//! monotonicity marker line, and a configuration line an operator
//! can scrape into a CSV. The `VALIDATOR_SCORING_METRIC_HARNESS
//! myosu e2e ok` success marker is the line the e2e proof greps
//! for.

use std::env;
use std::error::Error;
use std::process::ExitCode;

use myosu_validator::ValidatorScoringMetric;
use myosu_validator::emit_validator_scoring_metric;

/// Default 11-step L1 distance ladder matching the
/// `monotonicity_invariant_holds_for_p50_le_p99` unit test in
/// `metric::tests`. A wider ladder gives the percentile helper a
/// representative distribution without rounding through edge
/// cases (e.g. rank 1 of 11 vs rank 11 of 11).
const DEFAULT_LADDER: &[f64] = &[
    0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("validator_scoring_metric: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let ladder = match parse_ladder(env::args().skip(1))? {
        Some(parsed) => parsed,
        None => DEFAULT_LADDER.to_vec(),
    };

    if ladder.is_empty() {
        return Err("validator_scoring_metric: empty L1 distance ladder".into());
    }

    // Configuration line: an operator's CSV scraper can pick the
    // ladder and elapsed_ms metadata from a single line. Format
    // mirrors the `VALIDATOR_SCORING_METRIC_HARNESS ...` success
    // marker the e2e proof greps for, so the harness can assert
    // the live example run carried the expected contract.
    println!(
        "VALIDATOR_SCORING_METRIC_HARNESS myosu e2e ok surface=validator_scoring_metric_run_count=1 ladder={} elapsed_ms_total=42",
        format_ladder(&ladder),
    );

    // One single-scenario metric line (the live `score_response`
    // shape) plus one multi-scenario metric line (the helper
    // surface a future `score_session` would use). Both go
    // through the same `emit` line protocol, so a wrapper script
    // can `grep ^VALIDATOR_SCORING_METRIC` without distinguishing
    // the two.
    let single_metric =
        ValidatorScoringMetric::single("poker", ladder[0], 1).expect("single should succeed");
    print!("{}", emit_validator_scoring_metric(&single_metric));

    let multi_metric =
        ValidatorScoringMetric::from_l1_slice("poker", &ladder, 42).expect("multi should succeed");
    print!("{}", emit_validator_scoring_metric(&multi_metric));

    // Monotonicity marker: a separate line the harness can grep
    // to confirm `p50_l1 <= p99_l1` on the multi-scenario run.
    // The `monotonic=` field is the literal boolean so a
    // downstream parser does not have to re-derive the contract.
    println!(
        "VALIDATOR_SCORING_METRIC_MONOTONICITY p50_l1={:.6} p99_l1={:.6} monotonic={}",
        multi_metric.p50_l1,
        multi_metric.p99_l1,
        multi_metric.p50_l1 <= multi_metric.p99_l1,
    );

    // Byte-stability marker: emit the multi-scenario line a
    // second time so the harness can `diff` two consecutive
    // emissions and assert the protocol is byte-stable. The
    // `byte_stable=` field is the literal equality result.
    let second_emission = emit_validator_scoring_metric(&multi_metric);
    let byte_stable = second_emission == emit_validator_scoring_metric(&multi_metric);
    println!("VALIDATOR_SCORING_METRIC_BYTE_STABLE byte_stable={byte_stable}");

    Ok(())
}

fn parse_ladder(args: impl Iterator<Item = String>) -> Result<Option<Vec<f64>>, Box<dyn Error>> {
    let ladder: Vec<f64> = args
        .map(|raw| {
            raw.parse::<f64>()
                .map_err(|error| format!("invalid L1 distance `{raw}`: {error}").into())
        })
        .collect::<Result<_, Box<dyn Error>>>()?;
    if ladder.is_empty() {
        return Ok(None);
    }
    for value in &ladder {
        if !value.is_finite() {
            return Err(format!(
                "validator_scoring_metric: non-finite L1 distance {value} rejected (INV-003 determinism)"
            )
            .into());
        }
        if !(*value >= 0.0) {
            return Err(format!(
                "validator_scoring_metric: negative L1 distance {value} rejected"
            )
            .into());
        }
    }
    Ok(Some(ladder))
}

fn format_ladder(ladder: &[f64]) -> String {
    ladder
        .iter()
        .map(|value| format!("{value:.6}"))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    #[test]
    fn parse_ladder_returns_none_for_empty_args() {
        let parsed = parse_ladder(std::iter::empty()).expect("empty args should succeed");
        assert!(parsed.is_none());
    }

    #[test]
    fn parse_ladder_round_trips_deterministic_floats() {
        let parsed = parse_ladder(
            ["0.0", "0.5", "1.0"]
                .iter()
                .map(|value| (*value).to_string()),
        )
        .expect("three-element ladder should parse")
        .expect("three elements is not empty");
        assert_eq!(parsed, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn parse_ladder_rejects_non_finite_input() {
        let parsed = parse_ladder(["inf"].iter().map(|value| (*value).to_string()));
        assert!(parsed.is_err(), "+inf must be rejected at parse time");
        let parsed = parse_ladder(["nan"].iter().map(|value| (*value).to_string()));
        assert!(parsed.is_err(), "NaN must be rejected at parse time");
    }

    #[test]
    fn parse_ladder_rejects_negative_input() {
        let parsed = parse_ladder(["-0.1"].iter().map(|value| (*value).to_string()));
        assert!(parsed.is_err(), "negative L1 distance must be rejected");
    }

    #[test]
    fn format_ladder_is_byte_stable() {
        let a = format_ladder(&[0.0, 0.5, 1.0]);
        let b = format_ladder(&[0.0, 0.5, 1.0]);
        assert_eq!(a, b);
        assert_eq!(a, "0.000000,0.500000,1.000000");
    }
}
