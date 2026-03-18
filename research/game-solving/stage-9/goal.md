# Stage 9: Established Methods

## Objective

Implement and evaluate two established solver approaches -- one CFR-based
and one RL-based -- using their standard (published) architectures. These
represent the current state-of-the-art for imperfect-information game
solving and serve as the primary comparison points for the proposed
architecture search approach in stage 10.

## Research Questions

### Primary

1. How do established solver architectures perform on the 20-game evaluation
   suite, and how do they compare to the architecture-searched baselines from
   stage 8?

2. Is there a significant performance gap between CFR-based and RL-based
   established methods across the game complexity tiers?

3. Do established methods outperform all architecture search baselines, or do
   some baselines already match or exceed established performance?

### Secondary

4. What are the architectural features of established methods that differ most
   from architectures found by the search baselines?

5. How sensitive are established methods to seed variation compared to
   architecture-searched methods?

6. Do established methods show a performance advantage on specific game
   types (e.g., CFR-based on 2-player zero-sum) as predicted by the
   algorithm-game matching in stage 3?

## Scope

### Established Methods

1. **Established Method 1 (CFR-Based)**:
   A Deep CFR variant using the published network architecture. The
   architecture uses an MLP backbone with advantage-based regret
   estimation. Training follows the standard deep CFR loop: traverse game
   tree, estimate advantages, update cumulative strategy.

   This method directly targets exploitability minimization and has
   theoretical convergence guarantees for 2-player zero-sum games.

2. **Established Method 2 (RL-Based)**:
   A policy gradient method adapted for imperfect information. Uses an
   LSTM-based architecture to handle the sequential observation structure
   of imperfect-information games. Training follows self-play with
   importance-weighted policy updates.

   This method is more general (applies to multiplayer and team games)
   but lacks convergence guarantees.

### Evaluation Protocol

Identical to stage 8:

- 10 seeds: [42, 123, 456, 789, 1024, 2048, 4096, 8192, 16384, 32768]
- Time budget: 1800 seconds per seed per condition
- Primary and secondary metrics with bootstrap CIs (1000 samples)
- Paired comparison vs. Random Search baseline
- Paired comparison vs. each other

### Key Distinction from Stage 8

Stage 8 baselines use architecture search to discover new architectures.
Stage 9 methods use fixed, published architectures. The comparison reveals
whether architecture search adds value over expert-designed architectures.

## Methodology

### Phase 1: Implementation

Implement both established methods in the experiment harness:

- Use published architecture specifications (layer sizes, connectivity,
  activation functions) exactly as described in their papers
- Adapt only the game interface layer to work with the 20-game evaluation
  suite
- Validate on games where published results are available (accuracy within
  5% of published numbers)

### Phase 2: Evaluation Runs

Run both methods across all seeds:

- 2 methods x 10 seeds = 20 runs
- Same evaluation protocol as stage 8
- Record primary and secondary metrics per seed

### Phase 3: Comparative Analysis

Compare established methods against:

- Each other (CFR-based vs. RL-based)
- Stage 8 baselines (Random Search, DARTS, AdamW, AugMax)
- Theoretical bounds (for games where bounds exist from stage 1)

Compute:

- Mean and CI for primary and secondary metrics
- Paired differences and effect sizes
- Per-game-type breakdown (to test stage 3 predictions)

### Phase 4: Architecture Feature Analysis

Extract and compare the architectural features of established methods vs.
stage 8 baselines:

- Network depth, width, parameter count
- Block types used (MLP, LSTM, attention, etc.)
- Encoding strategy (how game state is represented)
- Output head structure

Identify which architectural choices are common to established methods but
absent from search-found architectures (potential search space gaps) and
vice versa (novel discoveries by search).

## Expected Outcomes

1. Primary and secondary metric values for both established methods, as
   reported in the stage-10 results.json under established_method_1 and
   established_method_2.

2. Statistical comparisons showing how established methods rank relative to
   architecture search baselines.

3. Per-game-type analysis validating the algorithm-game matching predictions
   from stage 3.

4. Architectural feature comparison identifying structural differences between
   expert-designed and search-found architectures.

## Success Criteria

- Both methods produce valid results (success rate 10/10 seeds).
- Established methods outperform Random Search baseline (confirming that
  expert architecture design adds value over random).
- At least one established method outperforms all stage 8 baselines (setting
  a meaningful bar for the proposed approach in stage 10).
- Results are consistent with published benchmarks where comparison is
  possible.

## Dependencies

- Stage 5 (Metric Design): metric definitions and statistical framework
- Stage 6 (Architecture Candidates): search space (for architecture feature
  comparison)
- Stage 8 (Baseline Implementation): baseline results for comparison

## Outputs

- `established_results.json` -- per-method, per-seed metrics
- `architecture_comparison.json` -- feature-by-feature comparison with
  baselines
- `game_type_breakdown.csv` -- per-game-type performance analysis
- `validation_vs_published.md` -- comparison against published benchmarks

## Notes

The stage-10 results show established_method_1 with primary_metric_mean of
0.673 and established_method_2 at 0.682. Both underperform the stage 8
baselines (Random Search at 0.861, DARTS at 0.850), which is notable -- in
the stage-10 evaluation framework, higher primary_metric_mean indicates
worse exploitability after normalization inversion. The paired comparisons
(all with p_value = 0.002) confirm these differences are statistically
significant.
