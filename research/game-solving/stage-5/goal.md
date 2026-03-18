# Stage 5: Metric Design

## Objective

Design and validate the evaluation metrics used throughout the remainder of the
pipeline. The metrics must capture solver quality across diverse game types:
adversarial (exploitability), cooperative (partner signaling), stochastic
(duplicate scoring), and asymmetric (role imbalance). Ensure metrics are
comparable across games with different scales and structures.

## Research Questions

### Primary

1. What is the appropriate exploitability metric for each game type, and how
   should it be normalized to allow cross-game comparison?

2. For partnership/team games, how should cooperative performance be measured
   -- partner signaling accuracy, team win rate, or regret against the
   team-optimal strategy?

3. What secondary metrics capture aspects of solver quality not reflected in
   exploitability (e.g., convergence speed, strategy interpretability,
   robustness to opponent model errors)?

### Secondary

4. For games with stochastic elements (card/tile draws), should metrics be
   computed in expectation over deals (duplicate-style scoring) or on a
   per-deal basis?

5. How should metrics handle asymmetric games where player roles have
   fundamentally different objectives?

6. What sample sizes are needed for metric estimates with acceptable confidence
   intervals?

7. Can a single "primary metric" be defined that is meaningful across all 20
   games, or must we use game-family-specific metrics?

## Scope

### Metric Categories

1. **Primary Metric (Exploitability-Based)**:
   The primary performance measure for the architecture search. Must be
   computable for all 20 games.

   - For 2-player zero-sum: standard exploitability (sum of players' regrets
     against best response)
   - For multiplayer: epsilon-Nash distance
   - For team games: team regret against team-optimal correlated strategy
   - Normalization: divide by the game value range to produce a 0-1 scale

2. **Secondary Metric (Domain-Specific Quality)**:
   Captures game-specific performance aspects.

   - Poker: chip win rate (bb/100) against a fixed opponent pool
   - Partnership games: partner signal detection accuracy
   - Tile/board games: territory control efficiency
   - Asymmetric games: per-role win rate balance

3. **Convergence Metrics**:
   - Iterations to reach a target exploitability threshold
   - Wall-clock time to convergence
   - Exploitability decay rate (log-linear fit)

4. **Robustness Metrics**:
   - Performance against adversarial best response
   - Performance against a distribution of opponent strategies
   - Sensitivity to abstraction granularity

### Bootstrap and Statistical Framework

All metrics will be reported with bootstrap confidence intervals (1000 samples
minimum). Paired comparisons between methods will use Wilcoxon signed-rank
tests due to the non-normal distribution of exploitability across seeds.

## Methodology

### Phase 1: Primary Metric Definition

For each game structure type (2-player zero-sum, multiplayer, team,
asymmetric), define the primary metric formula, normalization constant, and
computation procedure.

Validate that the normalized metric produces values in [0, 1] where 0 is
optimal and 1 is the worst possible strategy.

### Phase 2: Secondary Metric Definition

For each of the 20 games, define a game-specific secondary metric. Ensure
that:

- Higher is better (or clearly document the direction)
- The metric is efficiently computable (not requiring full game tree traversal)
- The metric correlates with but is not identical to the primary metric

### Phase 3: Statistical Framework Validation

On 3-4 games with known solutions:

- Compute metrics for the known-optimal strategy (should yield primary
  metric = 0 or near-zero)
- Compute metrics for random play (should yield primary metric near 1)
- Verify that bootstrap CIs have correct coverage (95% CI contains true value
  95% of the time across 100 replications)
- Verify that paired tests have correct size (reject at <= 5% when comparing
  identical strategies)

### Phase 4: Sample Size Determination

For each game, determine the number of evaluation episodes needed for:

- Primary metric with CI width < 0.05
- Secondary metric with CI width < 10% of the metric range
- Paired comparison with 80% power to detect an effect size of 0.3

## Expected Outcomes

1. Formal definitions of primary and secondary metrics for all 20 games.

2. A normalization scheme that makes the primary metric comparable across
   games.

3. Validated bootstrap and testing procedures with demonstrated correct
   coverage and size.

4. Sample size recommendations per game.

5. A metrics library (Python) implementing all metric computations with
   standard interfaces.

## Success Criteria

- Primary metric correctly identifies known-optimal strategies (metric value
  < 0.01 for solved games).
- Random play strategies produce primary metric values > 0.5 for all games.
- Bootstrap 95% CIs achieve 93-97% empirical coverage in validation.
- Paired test achieves <= 6% false positive rate in validation.
- All metric computations complete within 60 seconds per evaluation on a
  single CPU core.

## Dependencies

- Stage 1 (Literature Survey): known exploitability bounds for benchmarking
- Stage 2 (Complexity Classification): game structure types
- Stage 3 (Algorithm-Game Matching): which algorithm-game pairs will be
  evaluated (determines which metrics matter most)

## Outputs

- `metric_definitions.json` -- formal definitions for all 20 games
- `normalization_constants.json` -- per-game normalization values
- `validation_results.json` -- bootstrap coverage and test size verification
- `sample_sizes.json` -- recommended episodes per game
- `metrics/` -- Python library implementing the metric computations

## Notes

The bootstrap sample count of 1000 was chosen based on standard practice for
95% CI estimation. The stage-10 results use this same bootstrap framework,
confirming that the metric design carries through the full pipeline. The
degenerate_tolerance parameter (1e-6) in the results safeguards against
numerical precision issues in metric computation.
