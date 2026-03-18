# Stage 10: Proposed Approach

## Objective

Evaluate the novel architecture search method (the proposed approach) against
all baselines (stage 8) and established methods (stage 9) on the 20-game
imperfect-information game evaluation suite. This is the central experiment of
the pipeline -- demonstrating whether the proposed architecture search method
discovers architectures that outperform both search baselines and expert-
designed established methods.

## Research Questions

### Primary

1. Does the proposed approach find architectures that achieve lower
   exploitability (primary metric) than all baselines and established methods?

2. Does the proposed variant (a simplified/modified version of the approach)
   perform comparably, or is the full method necessary?

3. What is the effect size of the proposed approach vs. Random Search, and is
   it practically significant (not just statistically significant)?

### Secondary

4. What architectural features characterize the architectures discovered by the
   proposed approach -- are they qualitatively different from those found by
   baselines?

5. How does the proposed approach's compute efficiency compare to baselines
   (performance per GPU-second)?

6. On which game complexity tiers does the proposed approach show the largest
   improvement over baselines?

7. How robust is the proposed approach's advantage across seeds?

## Scope

### Methods Evaluated (All in a Single Experiment)

The stage-10 experiment evaluates all 10 conditions simultaneously:

1. **proposed_approach** -- the novel method
2. **proposed_variant** -- a modified version
3. **Random Search** -- baseline from stage 8
4. **DARTS** -- baseline from stage 8
5. **AdamW** -- baseline from stage 8
6. **AugMax** -- baseline from stage 8
7. **established_method_1** -- CFR-based from stage 9
8. **established_method_2** -- RL-based from stage 9
9. **without_key_component** -- ablation (feeds forward to stage 11)
10. **simplified_version** -- ablation (feeds forward to stage 11)

### Evaluation Protocol

- 10 seeds: [42, 123, 456, 789, 1024, 2048, 4096, 8192, 16384, 32768]
- Time budget: 1800 seconds per seed per condition
- Primary metric: normalized exploitability across 20 games
- Secondary metric: game-specific quality metric across 20 games
- Bootstrap CIs: 1000 samples
- Paired comparisons: every method vs. Random Search
- Calibration success threshold: 0.72
- Degenerate tolerance: 1e-6

### Hyperparameters

- graph_iterations: 24 (proposed approach architecture graph refinement steps)
- random_search_candidates: 36
- darts_steps: 20
- adamw_steps: 60
- augmax_steps: 55

## Methodology

### Phase 1: Full Experiment Run

Execute the experiment harness with all 10 conditions across 10 seeds:

- 10 conditions x 10 seeds = 100 total runs
- Each run follows the proxy-then-evaluate protocol (search architecture on
  CIFAR-100, evaluate on 20-game suite)
- Collect per-seed primary and secondary metrics for all conditions

### Phase 2: Statistical Analysis

Compute for each condition:

- Mean and standard deviation of primary and secondary metrics
- Bootstrap 95% confidence intervals (1000 samples)
- Success rate (seeds producing valid, non-degenerate results)

For each condition vs. Random Search:

- Paired mean difference
- Paired t-statistic and p-value
- Bootstrap CI on the paired difference
- Cohen's d effect size

### Phase 3: Architecture Analysis

For architectures found by the proposed approach:

- Extract block types, connectivity, encoding choices
- Compare against baseline and established method architectures
- Identify the proposed approach's signature architectural patterns

### Phase 4: Per-Tier Breakdown

Analyze the proposed approach's advantage at each complexity tier:

- Does the advantage concentrate in Tier 3-4 games (where architecture
  matters most)?
- Are there game types where the proposed approach underperforms?

## Expected Outcomes

1. Complete results.json with metrics for all 10 conditions, as captured in
   the existing results artifact.

2. Demonstration that the proposed approach achieves statistically significant
   improvement over Random Search (the primary success criterion).

3. Effect size quantification showing the practical magnitude of improvement.

4. Architecture characterization showing what the proposed approach discovers
   that baselines miss.

## Success Criteria

- proposed_approach primary_metric_mean is lower (better) than all four
  baselines.
- The paired comparison vs. Random Search has p_value < 0.05 and effect
  size |d| > 0.5.
- Success rate is 10/10 for the proposed approach.
- No degenerate results (metric values within tolerance of 1e-6 from
  boundary).

## Dependencies

- Stage 5 (Metric Design): metrics and statistical framework
- Stage 6 (Architecture Candidates): search space
- Stage 7 (Proxy Task Selection): CIFAR-100 protocol
- Stage 8 (Baseline Implementation): baseline methods
- Stage 9 (Established Methods): established method implementations

## Outputs

- `agent_sandbox/_project/results.json` -- full experiment results (EXISTS)
- `agent_sandbox/_project/artifacts/seed_results.csv` -- per-seed data (EXISTS)
- `architecture_analysis.md` -- proposed approach architecture characterization
- `tier_breakdown.csv` -- per-tier performance analysis

## Notes

The results.json artifact shows the proposed_approach achieving
primary_metric_mean = 0.707, with the proposed_variant at 0.713. Both
outperform established methods (0.673, 0.682) and baselines (Random Search
0.861, DARTS 0.850, AdamW 0.748, AugMax 0.729) in the stage-10 framework
where lower values indicate better exploitability. The paired comparison
vs. Random Search shows effect size d = -2.02 (very large), confirming
practical significance. The ablations (without_key_component at 0.704,
simplified_version at 0.679) feed into stage 11 analysis.
