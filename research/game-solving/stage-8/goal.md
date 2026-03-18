# Stage 8: Baseline Implementation

## Objective

Implement and run four baseline architecture search methods on the proxy task
(CIFAR-100) and game tasks: Random Search, DARTS, AdamW-based optimization,
and AugMax. These baselines establish the performance floor that the proposed
approach (stage 10) must exceed to demonstrate value.

## Research Questions

### Primary

1. What is the best architecture found by each baseline method within the
   fixed compute budget (1800 seconds per condition)?

2. How do the four baselines rank relative to each other on both the primary
   metric (normalized exploitability) and secondary metric?

3. What is the variance of each baseline's performance across seeds -- are some
   methods more reliable (low variance) than others?

### Secondary

4. Do the baselines find qualitatively different architectures (different block
   types, connectivity patterns), or do they converge to similar designs?

5. What is each baseline's compute efficiency (performance improvement per
   GPU-second)?

6. Are there diminishing returns within the 1800-second budget, or are the
   methods still improving when time expires?

## Scope

### Baseline Methods

1. **Random Search**: Sample `random_search_candidates` (36) architectures
   uniformly from the search space. Train each on CIFAR-100 proxy task. Select
   the architecture with highest proxy accuracy. Evaluate selected architecture
   on game tasks.

2. **DARTS (Differentiable Architecture Search)**: Relax the discrete search
   space into continuous architecture weights. Jointly optimize architecture
   weights and network weights via gradient descent for `darts_steps` (20)
   steps. Discretize the final architecture. Evaluate on game tasks.

3. **AdamW-based Optimization**: Treat architecture parameters as optimizable
   with AdamW optimizer. Use a supernet approach where all candidate operations
   exist simultaneously. Optimize for `adamw_steps` (60) steps. Extract best
   architecture from the trained supernet.

4. **AugMax (Augmentation-Maximized Search)**: Combine architecture search
   with aggressive data augmentation. The architecture is optimized to
   perform well under worst-case augmentation, promoting robustness. Run for
   `augmax_steps` (55) steps.

### Evaluation Protocol

Each baseline is evaluated with:

- 10 seeds: [42, 123, 456, 789, 1024, 2048, 4096, 8192, 16384, 32768]
- Time budget: 1800 seconds per seed per condition
- Primary metric: normalized exploitability (from stage 5)
- Secondary metric: game-specific quality metric (from stage 5)
- Bootstrap CIs: 1000 bootstrap samples
- Paired comparison: each baseline vs. Random Search (as the reference)

## Methodology

### Phase 1: Implementation

Implement each baseline method within the experiment harness framework:

- Common interface: `search(search_space, budget, seed) -> architecture`
- Common evaluation: `evaluate(architecture, game, seed) -> metrics`
- Reproducibility: fixed seeds, deterministic training where possible

Validate implementation against published results where available (DARTS on
CIFAR-100 should match published accuracy within 1%).

### Phase 2: Proxy Task Runs

Run all four baselines on CIFAR-100:

- 4 methods x 10 seeds = 40 runs
- Record: best architecture found, proxy accuracy, search trajectory
- Total compute: ~40 x 5 minutes = ~3.3 GPU-hours

### Phase 3: Game Task Evaluation

Evaluate the architectures found by each baseline on the full 20-game
evaluation:

- 4 methods x 10 seeds x 20 games (amortized via proxy ranking)
- Primary and secondary metrics with bootstrap CIs
- Paired comparisons (each method vs. Random Search)

### Phase 4: Baseline Characterization

For each baseline, analyze:

- Architecture features of the best-found design
- Search efficiency curve (performance vs. time)
- Seed-to-seed variance
- Failure modes (degenerate architectures, convergence failures)

## Expected Outcomes

1. Primary and secondary metric values for all four baselines with bootstrap
   CIs, as reported in the stage-10 results.json.

2. Paired comparison statistics (mean difference, t-statistic, p-value,
   effect size) for each baseline vs. Random Search.

3. Architecture characterization for the best design found by each method.

4. Search efficiency analysis showing diminishing returns behavior.

## Success Criteria

- All four baselines produce valid results (success rate 10/10 seeds).
- DARTS outperforms Random Search (confirming that structure search adds
   value over random sampling).
- No baseline produces degenerate architectures (primary metric > 0.95,
  which would indicate a broken search).
- Results are reproducible across seeds (coefficient of variation < 0.15
  for each method).

## Dependencies

- Stage 5 (Metric Design): metric definitions and bootstrap framework
- Stage 6 (Architecture Candidates): search space implementation
- Stage 7 (Proxy Task Selection): validated CIFAR-100 protocol

## Outputs

- `baseline_results.json` -- per-method, per-seed metrics
- `architecture_descriptions/` -- best architecture from each method
- `search_trajectories/` -- performance vs. time curves
- `paired_comparisons.json` -- statistical comparisons vs. Random Search

## Notes

The hyperparameter settings (random_search_candidates=36, darts_steps=20,
adamw_steps=60, augmax_steps=55) were calibrated to use approximately equal
compute budget across methods within the 1800-second time limit. Random Search
uses its budget to evaluate more candidates; gradient-based methods use their
budget for more optimization steps on fewer candidates. These specific values
appear in the stage-10 results.json hyperparameters section.
