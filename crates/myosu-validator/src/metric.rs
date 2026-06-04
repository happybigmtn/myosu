//! First-class observability metric for validator scoring runs (W-06).
//!
//! Every bounded validator scoring pass emits exactly one
//! `VALIDATOR_SCORING_METRIC` line so an operator can scrape the
//! validator's stderr into a CSV and an agent can `grep` the per-run
//! latency / quality distribution without going through the chain RPC.
//!
//! The shape is intentionally minimal — one record per `score_response`
//! call carries the `game` slug, the per-scenario L1 distribution
//! (`mean_l1` / `p50_l1` / `p99_l1`), the `scenario_count`, and the
//! `elapsed_ms` wall-clock. A future scoring-run shape that runs N
//! scenarios in one call can use the same line protocol with
//! `scenario_count = N` and percentile fields computed from the per-call
//! L1 set; the on-the-wire line is unchanged.
//!
//! Determinism: [`percentile`] uses the nearest-rank method (no
//! interpolation) and the line is rendered from a single `format!` call
//! with `f64` rounded to 6 decimals via `%.6`, so two hosts running the
//! same scoring loop emit byte-stable lines (the INV-003 determinism
//! invariant must hold for the metric too — a metric that disagrees
//! between hosts is the same bug class as a score that disagrees).
//!
//! Failure mode: non-finite L1 distances (NaN / +inf / -inf) are
//! rejected at the [`ValidatorScoringMetric`] constructor with a
//! [`MetricError::NonFiniteL1`] variant. The scoring loop in
//! `validation::score_response` is the only caller and the
//! `l1_distance` field is `f64` arithmetic, so a NaN would only
//! surface if a future scoring code path lets it; failing closed at
//! the constructor means a future regression that emits NaN scores
//! does not silently emit a NaN metric line.

use thiserror::Error;

/// Errors raised by the validator scoring metric surface.
#[derive(Debug, Error, PartialEq)]
pub enum MetricError {
    /// Returned when the constructor is given a non-finite L1 distance
    /// (`NaN` / `+inf` / `-inf`). L1 distances are bounded in `[0.0, 2.0]`
    /// for the current per-game scoring paths; a non-finite value is
    /// a regression and must be caught at the metric boundary.
    #[error("validator scoring metric rejects non-finite L1 distance: {field}={value}")]
    NonFiniteL1 { field: &'static str, value: f64 },
}

/// First-class observability record for one bounded validator scoring run.
///
/// One record per `score_response` call. The constructor rejects
/// non-finite L1 distances so a future regression that emits `NaN`
/// scores is caught at the metric boundary instead of silently
/// propagating into the operator's scraper.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatorScoringMetric {
    /// Game slug the scoring run executed against (matches
    /// `myosu_validator::cli::GameSelection` variants, serialized
    /// via `Debug` so the line protocol is grep-friendly on either
    /// the variant name or the slug).
    pub game: String,
    /// Number of scenarios the scoring run executed. One
    /// `score_response` call scores exactly one query/response pair,
    /// so the current live value is `1`; a future per-run shape that
    /// scores N scenarios in one call will report `N` here.
    pub scenario_count: usize,
    /// Mean L1 distance across the scoring run's per-scenario L1 set.
    pub mean_l1: f64,
    /// 50th-percentile L1 distance (nearest-rank). For a single-scenario
    /// run this equals `mean_l1`.
    pub p50_l1: f64,
    /// 99th-percentile L1 distance (nearest-rank). For a single-scenario
    /// run this equals `mean_l1`.
    pub p99_l1: f64,
    /// Wall-clock elapsed milliseconds for the scoring run.
    pub elapsed_ms: u64,
}

impl ValidatorScoringMetric {
    /// Build a metric record for a single-scenario scoring run.
    ///
    /// This is the constructor the `score_response` loop uses today;
    /// it is the only entry point the live scoring path needs. The
    /// [`from_l1_slice`] constructor is the multi-scenario variant a
    /// future scoring-run shape can call.
    ///
    /// Args:
    ///     game: Game slug the scoring run executed against.
    ///     l1_distance: Per-scenario L1 distance for this single-scenario
    ///         run.
    ///     elapsed_ms: Wall-clock elapsed milliseconds.
    ///
    /// Returns:
    ///     A metric record whose `mean_l1` / `p50_l1` / `p99_l1` all
    ///     equal `l1_distance` and whose `scenario_count` is `1`.
    pub fn single(game: &str, l1_distance: f64, elapsed_ms: u64) -> Result<Self, MetricError> {
        Self::from_l1_slice(game, &[l1_distance], elapsed_ms)
    }

    /// Build a metric record for a multi-scenario scoring run.
    ///
    /// Args:
    ///     game: Game slug the scoring run executed against.
    ///     l1_distances: Per-scenario L1 distances from the scoring
    ///         run. May be empty (the constructor then reports
    ///         `scenario_count = 0` and finite `mean_l1` / percentile
    ///         fields of `0.0` — the operator's scraper still emits
    ///         a well-formed line, just one with no L1 distribution
    ///         to summarize).
    ///     elapsed_ms: Wall-clock elapsed milliseconds.
    ///
    /// Returns:
    ///     A metric record with `scenario_count = l1_distances.len()`
    ///     and the mean / 50th-percentile / 99th-percentile L1
    ///     distances computed from the slice.
    pub fn from_l1_slice(
        game: &str,
        l1_distances: &[f64],
        elapsed_ms: u64,
    ) -> Result<Self, MetricError> {
        for (idx, value) in l1_distances.iter().enumerate() {
            if !value.is_finite() {
                // The exact field name is operator-facing diagnostics.
                // A future `[T; N]`-typed signature could lift this
                // to a const generic, but the live scoring paths all
                // use single-scenario `&[f64]` slices so the runtime
                // match is sufficient and never allocates.
                let field: &'static str = if idx == 0 {
                    "l1_distances[0]"
                } else if idx == 1 {
                    "l1_distances[1]"
                } else {
                    "l1_distances[n]"
                };
                return Err(MetricError::NonFiniteL1 { field, value: *value });
            }
        }
        if !mean_l1(l1_distances).is_finite() {
            return Err(MetricError::NonFiniteL1 {
                field: "mean_l1",
                value: mean_l1(l1_distances),
            });
        }
        let mean = mean_l1(l1_distances);
        let p50 = percentile(l1_distances, 0.50);
        let p99 = percentile(l1_distances, 0.99);
        Ok(Self {
            game: game.to_string(),
            scenario_count: l1_distances.len(),
            mean_l1: mean,
            p50_l1: p50,
            p99_l1: p99,
            elapsed_ms,
        })
    }
}

/// Format the `VALIDATOR_SCORING_METRIC` line for a metric record.
///
/// The line is the operator-facing contract: a wrapper script can
/// `grep ^VALIDATOR_SCORING_METRIC` to scrape scoring metrics, and
/// `awk -F= '{print $1}'` to enumerate field names. The line is
/// byte-stable across hosts (the `%.6` `f64` formatter and the
/// `u64` / `usize` decimal formatters are deterministic) so the
/// INV-003 determinism invariant carries through to the metric.
///
/// Args:
///     metric: Metric record to render.
///
/// Returns:
///     A single-line `String` ending in a `\n`. The line is the
///     full record; callers may either print it verbatim or split
///     on `=` to parse the fields.
pub fn emit(metric: &ValidatorScoringMetric) -> String {
    format!(
        "VALIDATOR_SCORING_METRIC game={} scenario_count={} mean_l1={:.6} p50_l1={:.6} p99_l1={:.6} elapsed_ms={}\n",
        metric.game,
        metric.scenario_count,
        metric.mean_l1,
        metric.p50_l1,
        metric.p99_l1,
        metric.elapsed_ms,
    )
}

/// Compute the nearest-rank percentile of an L1 distance slice.
///
/// Nearest-rank is the percentile algorithm that produces a value
/// the input slice actually contains (no interpolation), so two
/// hosts running the same scoring loop on the same input emit
/// byte-stable percentile values. The choice is the
/// determinism-friendly variant; the alternative (linear
/// interpolation) would tie the metric to a floating-point rounding
/// policy that the INV-003 epsilon does not need to cover.
///
/// Args:
///     slice: L1 distances. An empty slice returns `0.0` (so the
///         metric line is well-formed even on empty runs).
///     p: Percentile in the closed interval `[0.0, 1.0]`. Values
///         outside the range are clamped (negative → `0.0`, `> 1.0`
///         → `1.0`) so a caller bug does not surface as a NaN.
///
/// Returns:
///     The element at the nearest rank. For an N-element slice
///     sorted ascending, the rank is
///     `max(0, ceil(p * N) - 1)`. For a single-element slice the
///     return is that element for any `p` in `[0.0, 1.0]`.
pub fn percentile(slice: &[f64], p: f64) -> f64 {
    if slice.is_empty() {
        return 0.0;
    }
    let p = if p.is_nan() {
        0.0
    } else if p < 0.0 {
        0.0
    } else if p > 1.0 {
        1.0
    } else {
        p
    };
    let mut sorted: Vec<f64> = slice.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    // Nearest-rank: `rank = ceil(p * N)` with `rank` clamped to
    // `[1, N]`, then index `rank - 1` in the sorted slice. The
    // `f64::ceil` is deterministic for finite values; non-finite
    // inputs are caught by the constructor's `is_finite` check.
    let n = sorted.len();
    let rank_f = (p * n as f64).ceil();
    let rank = if rank_f < 1.0 {
        1usize
    } else if rank_f > n as f64 {
        n
    } else {
        rank_f as usize
    };
    sorted[rank - 1]
}

/// Mean of an L1 distance slice.
///
/// Empty slice returns `0.0` (a well-formed line, not a NaN, so the
/// operator's scraper does not crash on an empty run).
fn mean_l1(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return 0.0;
    }
    let sum: f64 = slice.iter().copied().sum();
    sum / slice.len() as f64
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    fn metric() -> ValidatorScoringMetric {
        ValidatorScoringMetric::single("poker", 0.125, 12).expect("single-constructor should succeed")
    }

    #[test]
    fn line_protocol_is_byte_stable_across_two_runs() {
        // Same input → same line. The INV-003 determinism
        // guarantee must carry through to the metric: a metric
        // that disagrees between two hosts is the same bug
        // class as a score that disagrees.
        let a = emit(&metric());
        let b = emit(&metric());
        assert_eq!(a, b);
    }

    #[test]
    fn line_protocol_carries_every_field() {
        let line = emit(&metric());
        assert!(line.starts_with("VALIDATOR_SCORING_METRIC "));
        assert!(line.contains("game=poker"));
        assert!(line.contains("scenario_count=1"));
        assert!(line.contains("mean_l1=0.125000"));
        assert!(line.contains("p50_l1=0.125000"));
        assert!(line.contains("p99_l1=0.125000"));
        assert!(line.contains("elapsed_ms=12"));
        assert!(line.ends_with('\n'));
    }

    #[test]
    fn percentile_is_nearest_rank() {
        // Sorted ascending; the nearest-rank percentile at `p`
        // is `slice[ceil(p * N) - 1]`. For N=5 and p=0.50 the
        // rank is `ceil(2.5) = 3` so the answer is `slice[2]`.
        let slice = [0.0_f64, 0.1, 0.2, 0.3, 0.4];
        assert_eq!(percentile(&slice, 0.0), 0.0);
        assert_eq!(percentile(&slice, 0.50), 0.2);
        assert_eq!(percentile(&slice, 0.99), 0.4);
        assert_eq!(percentile(&slice, 1.0), 0.4);
    }

    #[test]
    fn percentile_does_not_interpolate() {
        // Linear interpolation would land at `0.05` for `p=0.50`
        // on a `[0.0, 0.1]` slice; nearest-rank returns `0.0` or
        // `0.1` (the rank-1 element for N=2, p=0.5 is
        // `ceil(1.0) = 1` → `slice[0] = 0.0`).
        let slice = [0.0_f64, 0.1];
        assert_eq!(percentile(&slice, 0.50), 0.0);
    }

    #[test]
    fn percentile_clamps_out_of_range_inputs() {
        let slice = [0.0_f64, 0.5, 1.0];
        // Negative clamps to 0.0 → rank 1 → slice[0].
        assert_eq!(percentile(&slice, -0.1), 0.0);
        // > 1.0 clamps to 1.0 → rank N → slice[N-1].
        assert_eq!(percentile(&slice, 1.5), 1.0);
    }

    #[test]
    fn percentile_is_deterministic_for_unsorted_input() {
        // Two runs on the same unsorted input must produce the
        // same percentile (sort-then-pick-rank is the canonical
        // nearest-rank implementation).
        let slice_a = [0.4_f64, 0.1, 0.0, 0.3, 0.2];
        let slice_b = [0.0_f64, 0.1, 0.2, 0.3, 0.4];
        assert_eq!(percentile(&slice_a, 0.50), percentile(&slice_b, 0.50));
        assert_eq!(percentile(&slice_a, 0.99), percentile(&slice_b, 0.99));
    }

    #[test]
    fn percentile_empty_slice_returns_zero() {
        // An empty scoring run must not produce a NaN line.
        assert_eq!(percentile(&[], 0.50), 0.0);
        assert_eq!(percentile(&[], 0.99), 0.0);
    }

    #[test]
    fn single_scenario_run_reports_mean_eq_p50_eq_p99() {
        // For a single-scenario scoring run the L1 distribution
        // is one element, so all three summary statistics
        // collapse to that element. The operator's scraper
        // should not see a NaN or a p99 < mean.
        let m = ValidatorScoringMetric::single("kuhn", 0.0, 1)
            .expect("single should succeed");
        assert_eq!(m.scenario_count, 1);
        assert_eq!(m.mean_l1, 0.0);
        assert_eq!(m.p50_l1, 0.0);
        assert_eq!(m.p99_l1, 0.0);
    }

    #[test]
    fn multi_scenario_run_reports_live_mean() {
        // A 4-scenario run with L1 distances
        // `[0.0, 0.2, 0.4, 0.6]` should report
        // `mean_l1 = 0.3`, `p50_l1 = 0.2` (rank 2 of 4),
        // `p99_l1 = 0.6` (rank 4 of 4).
        let distances = [0.0_f64, 0.2, 0.4, 0.6];
        let m = ValidatorScoringMetric::from_l1_slice("liars-dice", &distances, 100)
            .expect("multi-scenario should succeed");
        assert_eq!(m.scenario_count, 4);
        assert!((m.mean_l1 - 0.3).abs() < 1e-9);
        assert_eq!(m.p50_l1, 0.2);
        assert_eq!(m.p99_l1, 0.6);
        assert_eq!(m.elapsed_ms, 100);
    }

    #[test]
    fn mean_l1_is_zero_for_empty_slice() {
        // The mean helper returns 0.0 on an empty slice so the
        // metric line stays well-formed; the constructor's
        // `is_finite` check is the only non-finite guard.
        let m = ValidatorScoringMetric::from_l1_slice("poker", &[], 5)
            .expect("empty-slice should succeed");
        assert_eq!(m.scenario_count, 0);
        assert_eq!(m.mean_l1, 0.0);
        assert_eq!(m.p50_l1, 0.0);
        assert_eq!(m.p99_l1, 0.0);
    }

    #[test]
    fn nan_l1_distance_is_rejected_at_constructor() {
        // NaN scores are a regression; the constructor
        // catches them at the metric boundary so they never
        // reach the operator's scraper.
        let err = ValidatorScoringMetric::single("poker", f64::NAN, 1)
            .expect_err("NaN must be rejected");
        assert!(matches!(err, MetricError::NonFiniteL1 { .. }));
    }

    #[test]
    fn infinity_l1_distance_is_rejected_at_constructor() {
        let err_pos = ValidatorScoringMetric::single("poker", f64::INFINITY, 1)
            .expect_err("+inf must be rejected");
        let err_neg = ValidatorScoringMetric::single("poker", f64::NEG_INFINITY, 1)
            .expect_err("-inf must be rejected");
        assert!(matches!(err_pos, MetricError::NonFiniteL1 { .. }));
        assert!(matches!(err_neg, MetricError::NonFiniteL1 { .. }));
    }

    #[test]
    fn nan_in_multi_scenario_slice_is_rejected() {
        let distances = [0.1_f64, 0.2, f64::NAN, 0.4];
        let err = ValidatorScoringMetric::from_l1_slice("poker", &distances, 1)
            .expect_err("NaN in slice must be rejected");
        assert!(matches!(err, MetricError::NonFiniteL1 { .. }));
    }

    #[test]
    fn monotonicity_invariant_holds_for_p50_le_p99() {
        // INV-003 determinism: a regression that swaps p50 and
        // p99 is loud. The contract is p50 <= p99; for a
        // single-scenario run this collapses to equality.
        let distances = [0.0_f64, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let m = ValidatorScoringMetric::from_l1_slice("poker", &distances, 50)
            .expect("multi-scenario should succeed");
        assert!(m.p50_l1 <= m.p99_l1, "p50={} must be <= p99={}", m.p50_l1, m.p99_l1);
    }

    #[test]
    fn line_is_grep_friendly_single_field_per_token() {
        // An operator's `awk -F= '{print $1}'` should produce
        // the field-name list, not a malformed `key=value=other`
        // token. The line protocol must not contain `=` inside
        // any value — `game`, `scenario_count`, `elapsed_ms`
        // are integers/strings, and the f64 fields use the
        // `%.6` formatter (no embedded `=`).
        let line = emit(&metric());
        let tokens: Vec<&str> = line.trim_end().split(' ').collect();
        let first = tokens.first().copied().expect("line should have first token");
        assert_eq!(first, "VALIDATOR_SCORING_METRIC");
        for token in tokens.iter().skip(1) {
            assert!(token.contains('='), "every field token must contain '=': {}", token);
            let parts: Vec<&str> = token.split('=').collect();
            assert_eq!(parts.len(), 2, "no embedded '=' allowed in field values: {}", token);
        }
    }
}
