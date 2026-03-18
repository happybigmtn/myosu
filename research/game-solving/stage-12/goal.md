# Stage 12: Statistical Analysis

## Objective

Conduct rigorous statistical analysis of all experimental results from stages
8-11. Apply bootstrap confidence intervals, significance testing, multiple
comparison corrections, and effect size estimation to produce publication-
quality statistical claims about the proposed approach's performance.

## Research Questions

### Primary

1. After correcting for multiple comparisons, which pairwise differences
   between methods remain statistically significant?

2. What are the tightest confidence intervals achievable for the proposed
   approach's advantage over each baseline and established method?

3. Is the proposed approach's performance improvement robust to the choice of
   statistical test (parametric vs. non-parametric)?

### Secondary

4. Are there outlier seeds that disproportionately influence the mean results,
   and are the conclusions robust to their removal?

5. What is the minimum detectable effect size given the experimental design
   (10 seeds, 10 conditions)?

6. Do the bootstrap CIs have correct coverage (validating the stage 5 metric
   design)?

7. Is there evidence of heterogeneous treatment effects (the proposed approach
   helps more for some games/tiers than others)?

## Scope

### Statistical Methods

1. **Bootstrap Confidence Intervals**:
   - BCa (bias-corrected and accelerated) bootstrap with 1000 resamples
   - Applied to all metric means and pairwise differences
   - 95% confidence level

2. **Paired Significance Tests**:
   - Wilcoxon signed-rank test (primary, non-parametric)
   - Paired t-test (secondary, parametric)
   - Permutation test (robustness check)

3. **Multiple Comparison Correction**:
   - Bonferroni correction for the full pairwise comparison family
   - Holm-Bonferroni as a less conservative alternative
   - False Discovery Rate (Benjamini-Hochberg) for exploratory comparisons

4. **Effect Size Estimation**:
   - Cohen's d for paired comparisons
   - Bootstrap CI on the effect size
   - Practical significance thresholds (small: 0.2, medium: 0.5, large: 0.8)

5. **Robustness Checks**:
   - Leave-one-out influence analysis (remove each seed, recompute)
   - Trimmed means (remove top and bottom seed, recompute)
   - Alternative metrics (secondary metric analysis)

### Data Sources

- Stage 10 results.json: all 10 conditions x 10 seeds
- Stage 13 results (v1, v2): refined method results across multiple sandbox
  iterations

## Methodology

### Phase 1: Descriptive Statistics

For each condition, compute:

- Mean, standard deviation, median, IQR of primary and secondary metrics
- Bootstrap 95% CIs (BCa method, 1000 resamples)
- Distribution shape assessment (Shapiro-Wilk normality test)

### Phase 2: Pairwise Comparisons

For each pair of conditions (45 pairs from 10 conditions):

- Paired mean difference with bootstrap CI
- Wilcoxon signed-rank test p-value
- Paired t-test p-value (for comparison with Wilcoxon)
- Cohen's d effect size with bootstrap CI

Organize results in a 10x10 comparison matrix.

### Phase 3: Multiple Comparison Correction

Apply corrections to the 45 pairwise p-values:

- Bonferroni: multiply each p-value by 45
- Holm-Bonferroni: step-down procedure
- Benjamini-Hochberg: FDR control at 5%

Report which comparisons survive each correction level.

### Phase 4: Robustness Analysis

For the key comparison (proposed_approach vs. Random Search):

- Leave-one-out: remove each seed in turn, recompute CI and p-value
- Trimmed: remove the best and worst seed, recompute
- Non-parametric bootstrap: resample seeds with replacement, 10000 times
- Compute the proportion of bootstrap resamples where the proposed approach
  outperforms (bootstrap probability of superiority)

### Phase 5: Power Analysis

Given the observed effect sizes and variances, compute:

- Post-hoc power for each comparison at alpha = 0.05
- Minimum detectable effect size at 80% power with 10 seeds
- Sample size needed to detect a small effect (d = 0.2) at 80% power

## Expected Outcomes

1. A corrected significance table showing which method comparisons survive
   multiple comparison correction.

2. Effect sizes with CIs for all key comparisons.

3. Robustness evidence showing conclusions hold under leave-one-out and
   alternative tests.

4. Power analysis informing whether additional seeds would be worthwhile in
   stage 13 refinement.

5. Publication-ready statistical summary suitable for a methods paper.

## Success Criteria

- The proposed approach vs. Random Search comparison survives Bonferroni
  correction (adjusted p < 0.05).
- Effect sizes for the proposed approach vs. all baselines have CIs that
  exclude zero.
- Leave-one-out analysis does not change the qualitative conclusions (no
  single seed drives the result).
- Power analysis confirms > 80% power for the observed effect sizes.

## Dependencies

- Stage 10 (Proposed Approach): complete results data
- Stage 11 (Ablation Study): ablation analysis results
- Stage 5 (Metric Design): bootstrap framework and statistical procedures

## Outputs

- `pairwise_comparisons.json` -- full 10x10 comparison matrix with corrected
  p-values
- `effect_sizes.json` -- Cohen's d with bootstrap CIs for all comparisons
- `robustness_analysis.json` -- leave-one-out and trimmed results
- `power_analysis.json` -- post-hoc power and sample size calculations
- `statistical_summary.md` -- publication-ready narrative summary

## Notes

The stage-10 results already contain t-statistics and p-values for each
condition vs. Random Search. This stage extends that analysis with multiple
comparison correction, robustness checks, and publication-quality reporting.
The p-values in stage-10 results (all 0.002) reflect the minimum achievable
p-value with 10 paired observations in a Wilcoxon test (1/2^9 = 0.00195),
suggesting that all observed differences are about as significant as the
sample size allows. Power analysis will determine whether finer discrimination
requires more seeds.
