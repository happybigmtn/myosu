# Stage 13 (v2): Refinement -- Second Iteration

## Objective

Continue iterative improvement of the proposed approach based on v1 findings.
The v2 refinement explores a broader range of hyperparameter settings and
tests additional hypotheses about the method's behavior, including the
determinism_noise_scale parameter and reduced bootstrap samples for faster
iteration.

## Research Questions

### Primary

1. Does adding determinism noise (determinism_noise_scale parameter) improve
   the method's robustness across seeds?

2. Can faster iteration (reduced bootstrap_samples from 1000 to 256) identify
   good hyperparameter settings without full statistical rigor?

3. Which of the 10+ sandbox variants in v2 produces the best overall result?

### Secondary

4. Is there a consistent relationship between hyperparameter settings and
   performance, or is the landscape noisy?

5. Do the v2 results change the conclusions from stage 10 and v1 about
   which method is best?

6. Does the determinism metric (reported in v2 results) provide useful
   information about architecture quality?

## Scope

### Refinement Changes (v2)

The v2 iteration explores a wider set of modifications:

1. **determinism_noise_scale**: new parameter (0.0005 - 0.00075) adding
   small noise to test reproducibility of the architecture search
2. **Reduced bootstrap_samples**: 256 instead of 1000 for faster per-run
   evaluation (final analysis uses full 1000 from stage 12)
3. **Multiple sandbox variants**: systematic exploration with v1, v1_fix,
   v3_fix, v4, v5, v6_fix, v7, v7_fix, v8, v8_fix, v9, v9_fix

### Evaluation Protocol

Same 10-condition, 10-seed protocol as stage 10 and v1. Each sandbox variant
tests a different hyperparameter combination. The determinism_mean metric is
tracked as a new diagnostic.

### Variant Strategy

- **v1, v1_fix**: baseline v2 settings with and without bug fixes
- **v3_fix through v9_fix**: systematic exploration of determinism_noise_scale
  and other parameters
- Variants without _fix suffix are the initial runs; _fix variants correct
  issues discovered during execution

## Methodology

### Phase 1: Systematic Exploration

Run 10+ sandbox variants spanning the hyperparameter space:

- Vary determinism_noise_scale: [0.0, 0.0005, 0.00075, 0.001]
- Vary bootstrap_samples: [256, 512, 1000]
- Keep the core algorithm and time budget fixed
- Record all 10-condition results for each variant

### Phase 2: Determinism Analysis

The v2 results include determinism_mean metrics for each condition:

- Analyze the relationship between determinism_noise_scale and determinism_mean
- Determine if higher determinism correlates with better primary metric
- Test if the noise injection helps or hurts convergence

### Phase 3: Variant Selection

From the 10+ variants, select the best-performing configuration:

- Rank by proposed_approach primary_metric_mean
- Verify the best configuration is not an outlier (consistent across seeds)
- Cross-check against secondary metric

### Phase 4: Comparison with Stage 10 and v1

Compare the best v2 variant against:

- Stage 10 results (original proposed approach)
- v1 best variant
- Determine the total improvement from the refinement process

## Expected Outcomes

1. Best hyperparameter configuration identified from 10+ variants.

2. Understanding of the determinism_noise_scale parameter's effect.

3. Evidence for whether faster iteration (reduced bootstrap) is a viable
   strategy for hyperparameter exploration.

4. Final refined method configuration for use in stage 14 cross-game
   validation.

## Success Criteria

- At least one v2 variant improves over the best v1 result.
- The determinism_noise_scale analysis produces a clear recommendation
  (use or don't use).
- The best v2 variant maintains 10/10 success rate across seeds.
- The reduced-bootstrap results are directionally consistent with full-
  bootstrap results (rank ordering of methods is preserved).

## Dependencies

- Stage 13_v1 (Refinement v1): v1 results and insights
- Stage 12 (Statistical Analysis): statistical framework
- Stage 10 (Proposed Approach): baseline comparison

## Outputs

- `refine_sandbox_v*/_project/results.json` -- per-variant results (EXISTS)
- `refine_sandbox_v*/_project/artifacts/seed_results.csv` -- per-seed data (EXISTS)
- `variant_comparison.json` -- cross-variant analysis
- `determinism_analysis.json` -- noise_scale parameter study
- `best_configuration.json` -- recommended hyperparameter settings

## Notes

The v2 results show a distinct pattern from stage 10 and v1. The
determinism_mean values are very close to 1.0 (0.9999+), indicating near-
perfect reproducibility even with noise injection. The established_method_1
and simplified_version conditions show notably different behavior in v2
compared to stage 10, with primary_metric_mean values of ~0.40 (vs. 0.67
in stage 10). This suggests the hyperparameter changes interact with the
method implementations in non-trivial ways, warranting careful analysis
before finalizing the configuration.
