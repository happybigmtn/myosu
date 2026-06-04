# Validator Scoring Observability (W-06)

This document is the operator-facing contract for the
`VALIDATOR_SCORING_METRIC` line the validator scoring loop emits
once per bounded scoring run. The metric surface is the
operator-readable, grep-friendly complement to the chain-side
quality score: an operator running `myosu-validator --query-file
... --response-file ...` gets exactly one structured
`VALIDATOR_SCORING_METRIC` line on stdout per scoring run, with
the per-run latency and the L1-distance distribution. A wrapper
script can `grep ^VALIDATOR_SCORING_METRIC` to scrape the line
into a CSV without going through the chain RPC.

> The Design + Eng lens: an operator can NOT today see scoring
> latency / quality at a glance. The chain-side `WEIGHTS ...
> submission ok` line tells you the validator submitted
> weights, but not whether the scoring loop itself is fast
> (sub-millisecond) or slow (multi-second), and not what the
> per-scenario L1 distribution looked like. W-06 surfaces both
> as a single grep-friendly line. The companion executable drift
> guard lives at `tests/e2e/validator_scoring_metric.sh` and is
> wired into the `validator-scoring-metric` CI job.

## What the metric guarantees

### One line per scoring run

The validator's `score_response` function emits exactly one
`VALIDATOR_SCORING_METRIC` line at the end of each scoring
run. A `cargo run -p myosu-validator -- --query-file <q>
--response-file <r>` invocation that scores N query/response
pairs in one run produces N `VALIDATOR_SCORING_METRIC` lines
(one per `score_response` call). Each line is independent: a
future scoring shape that runs N scenarios in one call can
report `scenario_count = N` and the line protocol is unchanged.

### Line protocol

The line is rendered by `myosu_validator::metric::emit(...)`
and has the following shape:

```text
VALIDATOR_SCORING_METRIC game=<slug> scenario_count=<usize> mean_l1=<f64> p50_l1=<f64> p99_l1=<f64> elapsed_ms=<u64>
```

Field-by-field contract:

| Field          | Type   | Contract                                                                  |
|----------------|--------|---------------------------------------------------------------------------|
| `game`         | string | Game slug the scoring run executed against. Matches the `GameSelection` variant (e.g. `Poker`, `Kuhn`, `LiarsDice`, `Cribbage`, `Hearts`, ...). |
| `scenario_count` | `usize` | Number of scenarios the run scored. The live `score_response` shape reports `1`; a future per-run shape that scores N scenarios reports `N`. |
| `mean_l1`      | `f64`  | Mean per-scenario L1 distance. For a single-scenario run, this equals the run's L1 distance. |
| `p50_l1`       | `f64`  | 50th-percentile L1 distance, nearest-rank. For a single-scenario run, this equals the run's L1 distance. |
| `p99_l1`       | `f64`  | 99th-percentile L1 distance, nearest-rank. For a single-scenario run, this equals the run's L1 distance. |
| `elapsed_ms`   | `u64`  | Wall-clock elapsed milliseconds for the scoring run. |

All `f64` values are rendered with the `%.6` formatter
(6-decimal fixed point), so the line is byte-stable across
hosts (the INV-003 determinism invariant carries through to the
metric — a metric that disagrees between hosts is the same bug
class as a score that disagrees).

### Nearest-rank percentile

`mean_l1` and the two percentile fields are computed by the
public `myosu_validator::metric::percentile(slice, p)` helper,
which uses the nearest-rank algorithm (no interpolation). The
choice is determinism-friendly: nearest-rank always returns a
value the input slice actually contains, so two hosts running
the same scoring loop on the same input emit byte-stable
percentile values. The alternative (linear interpolation)
would tie the metric to a floating-point rounding policy that
the INV-003 epsilon does not need to cover.

For an N-element slice sorted ascending, the nearest-rank
percentile at `p ∈ [0.0, 1.0]` is `slice[ceil(p * N) - 1]`. For
`N = 11` and `p = 0.50` the rank is `ceil(5.5) = 6` so the
return is `slice[5]`. For `N = 1` and any `p` in `[0.0, 1.0]`
the return is that single element.

### Non-finite L1 distances are rejected

A `NaN` or `+inf` / `-inf` L1 distance is a regression (the
current per-game scoring paths bound L1 in `[0.0, 2.0]`), so
the metric constructor rejects non-finite inputs with the
`MetricError::NonFiniteL1 { field, value }` variant. The
scoring loop catches the rejection at the metric boundary and
emits a `tracing::warn!` line instead of a malformed
`VALIDATOR_SCORING_METRIC` line, so a future regression that
emits NaN scores is loud instead of silent.

## Sample line

For a single-scenario poker scoring run with L1 distance
`0.000000` and elapsed time `1 ms`:

```text
VALIDATOR_SCORING_METRIC game=Poker scenario_count=1 mean_l1=0.000000 p50_l1=0.000000 p99_l1=0.000000 elapsed_ms=1
```

For an 11-scenario scoring run with the default L1 distance
ladder `[0.0, 0.1, ..., 1.0]`:

```text
VALIDATOR_SCORING_METRIC game=Poker scenario_count=11 mean_l1=0.500000 p50_l1=0.500000 p99_l1=1.000000 elapsed_ms=42
```

## Scraping the metric into a CSV

The line protocol is grep-friendly by design. A wrapper script
can scrape the metric into a CSV with a single `awk` pipeline:

```bash
cargo run -p myosu-validator -- \
    --query-file <query> --response-file <response> \
    2>&1 | grep ^VALIDATOR_SCORING_METRIC \
    | awk -F'[= ]' '{
        for (i = 2; i <= NF; i += 2) {
            printf "%s%s=%s", sep, $i, $(i + 1);
            sep = ",";
        }
        printf "\n";
        sep = "";
    }' \
    > scoring-metrics.csv
```

The output CSV carries one row per scoring run, with the field
list pinned to the line protocol above. The metric module
guarantees the line is byte-stable across hosts, so the CSV
diff between two operators is empty (modulo the `elapsed_ms`
column, which is wall-clock and intentionally not
deterministic).

## Failure modes

The metric surface fails closed in three cases:

1. **Non-finite L1 distance.** The constructor returns
   `MetricError::NonFiniteL1` and the scoring loop emits a
   `tracing::warn!` line instead of a `VALIDATOR_SCORING_METRIC`
   line. The CI proof `tests/e2e/validator_scoring_metric.sh`
   does not assert the non-finite case (the live scoring paths
   cannot produce one today); the `metric::tests` unit tests
   cover the rejection contract directly.

2. **Empty scoring run.** The `from_l1_slice` constructor
   accepts an empty L1 slice and reports
   `scenario_count = 0` with `mean_l1 = 0.0`,
   `p50_l1 = 0.0`, `p99_l1 = 0.0`. The line is well-formed;
   the metric is meaningless (no scenarios to summarize) but
   it does not break the operator's scraper.

3. **Scoring loop error.** A `score_response` failure (e.g.
   an invalid response distribution) propagates as
   `ValidationError` and the metric line is not emitted. The
   scoring loop's `info!` log line still records the failure
   so the operator can grep the validator's stderr for the
   error context.

## Reproduction commands

The full metric surface is reproducible from a fresh checkout:

```bash
# 1. Build the metric module + example binary.
SKIP_WASM_BUILD=1 cargo build -p myosu-validator --example validator_scoring_metric

# 2. Run the example to see the line protocol in action.
SKIP_WASM_BUILD=1 cargo run -p myosu-validator --example validator_scoring_metric

# 3. Run the metric module's 15 unit tests.
SKIP_WASM_BUILD=1 cargo test -p myosu-validator --lib metric::

# 4. Run the e2e drift guard.
bash tests/e2e/validator_scoring_metric.sh
```

The example's output includes:

- one `VALIDATOR_SCORING_METRIC_HARNESS myosu e2e ok ...` line
  with the configuration marker an operator can grep
- one `VALIDATOR_SCORING_METRIC game=poker scenario_count=1
  ...` line for the single-scenario constructor
- one `VALIDATOR_SCORING_METRIC game=poker scenario_count=11
  mean_l1=0.500000 p50_l1=0.500000 p99_l1=1.000000 ...` line
  for the multi-scenario constructor
- one `VALIDATOR_SCORING_METRIC_MONOTONICITY ... monotonic=true`
  line confirming the `p50_l1 <= p99_l1` contract
- one `VALIDATOR_SCORING_METRIC_BYTE_STABLE byte_stable=true`
  line confirming two consecutive `emit()` calls produce
  byte-identical output

The full `cargo test -p myosu-validator --lib` suite (the live
suite has 83 tests after this row ships) covers the metric
module's 15 unit tests in addition to the pre-existing
scoring / exploitability / poker-quality tests, so a single
`cargo test` invocation exercises the full surface.

## Where to find the surface in code

- `crates/myosu-validator/src/metric.rs` — the metric module
  (`ValidatorScoringMetric`, `emit`, `percentile`, `MetricError`)
- `crates/myosu-validator/src/lib.rs` — the `pub mod metric;`
  declaration and the `pub use metric::{...}` re-export
- `crates/myosu-validator/src/validation.rs` — the
  `score_response` function calls
  `emit_validator_scoring_metric(&metric)` after a successful
  `ValidationReport`
- `crates/myosu-validator/examples/validator_scoring_metric.rs`
  — the operator-facing probe binary
- `tests/e2e/validator_scoring_metric.sh` — the executable
  drift guard
- `.github/workflows/ci.yml` — the `validator-scoring-metric`
  CI job (mirrors the `solver-read` / `quality-benchmark` /
  `miner-convergence-doc` job shape)
- `.github/zizmor.yml` — the per-line `dtolnay/rust-toolchain`
  ignore entry for the new job, with the same `# reason:`
  justification discipline the CI-SEC-001 row introduced
- `INVARIANTS.md` INV-003 — the determinism invariant the
  metric carries through (the byte-stability marker line is
  the live proof)
- `README.md` "Operator Path" — the new `### Validator scoring
  observability` subsection linking this doc

## Operator's-eye view

> "I run the validator for an hour. I want a per-game
> latency/quality summary at the end. What do I do?"

```bash
cargo run -p myosu-validator -- \
    --query-file query.bin --response-file response.bin \
    2>&1 | grep ^VALIDATOR_SCORING_METRIC
```

That gives you one line per scoring run. Pipe it through `awk`
or `cut` and you have a CSV. The line protocol is
intentionally minimal: one record per scoring run, with the
six fields an operator needs to spot a slow run (`elapsed_ms`)
or a quality regression (`mean_l1` / `p50_l1` / `p99_l1`).
