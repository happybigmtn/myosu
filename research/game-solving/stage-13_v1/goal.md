# Stage 13 (v1): Refinement -- First Iteration

## Objective

Iteratively improve the proposed approach based on findings from the ablation
study (stage 11) and statistical analysis (stage 12). The v1 refinement
addresses the primary issue identified: the relationship between the key
component and the simplified version, and attempts to improve the proposed
approach's primary metric.

## Research Questions

### Primary

1. Can the proposed approach be improved by incorporating insights from the
   simplified_version's better primary metric performance?

2. Do hyperparameter adjustments (graph_iterations, architecture scoring) close
   the gap between the proposed approach and the simplified_version?

3. Does the refined method maintain its advantage on the secondary metric
   while improving the primary metric?

### Secondary

4. How sensitive is the refinement to the specific hyperparameter changes made?

5. Do the refinement changes affect the method's behavior differently across
   game complexity tiers?

6. Is the refined method more or less consistent across seeds than the
   original?

## Scope

### Refinement Changes (v1)

Based on stage 11 findings, the v1 refinement explores:

1. **Adjusted graph iterations**: increase from 24 to 26 to allow more
   architecture convergence
2. **Increased search candidates**: from 36 to 40 random_search_candidates
3. **Modified DARTS/AdamW/AugMax steps**: slight increases (22/64/58) to
   allow more baseline optimization
4. **Same 10-seed protocol**: [42, 123, ..., 32768]
5. **Same time budget**: 1800 seconds

### Experimental Variants

The v1 refinement includes three sandbox runs:

- **refine_sandbox_v1**: initial refinement with adjusted hyperparameters
- **refine_sandbox_v1_fix**: corrected version addressing an issue discovered
  during refine_sandbox_v1
- **refine_sandbox_v4**: further hyperparameter exploration

### Evaluation Protocol

Identical to stage 10 except for the hyperparameter changes. All 10 conditions
re-evaluated with adjusted settings.

## Methodology

### Phase 1: Hyperparameter Adjustment

Apply the refined hyperparameter settings and re-run the full experiment:

- Modified settings: graph_iterations=26, random_search_candidates=40,
  darts_steps=22, adamw_steps=64, augmax_steps=58
- Full 10-condition x 10-seed evaluation
- Compare results against stage 10 baseline

### Phase 2: Issue Detection and Fix

During refine_sandbox_v1 execution, monitor for:

- Degenerate results (metrics near tolerance boundary)
- Seed failures (success rate < 10/10)
- Unexpected metric distributions

If issues are found, diagnose root cause and produce refine_sandbox_v1_fix
with corrections.

### Phase 3: Further Exploration (v4)

Based on v1 and v1_fix results, explore additional hyperparameter settings
in refine_sandbox_v4. Focus on the parameters that showed the most
sensitivity in the v1 results.

### Phase 4: Comparative Analysis

Compare v1 refinement results against stage 10 results:

- Did the primary metric improve?
- Was the secondary metric maintained?
- Did the proposed approach's ranking relative to baselines change?
- Are the bootstrap CIs tighter (more consistent results)?

## Expected Outcomes

1. Refined hyperparameter settings that improve the proposed approach's primary
   metric.

2. Identification and fix of any issues discovered during the refinement runs.

3. Clearer picture of the hyperparameter sensitivity landscape.

4. Data feeding into the v2 refinement for further improvement.

## Success Criteria

- The refined method produces valid results (10/10 success rate).
- At least one refinement variant improves the primary metric relative to
  stage 10.
- The fix variant (v1_fix) resolves any issues from the initial v1 run.
- Results are reproducible and consistent across seeds.

## Dependencies

- Stage 10 (Proposed Approach): baseline results for comparison
- Stage 11 (Ablation Study): insights guiding the refinement direction
- Stage 12 (Statistical Analysis): statistical framework for evaluation

## Outputs

- `refine_sandbox_v1/_project/results.json` -- initial refinement results (EXISTS)
- `refine_sandbox_v1_fix/_project/results.json` -- corrected results (EXISTS)
- `refine_sandbox_v4/_project/results.json` -- further exploration (EXISTS)
- `refine_sandbox_*/`_project/artifacts/seed_results.csv` -- per-seed data (EXISTS)

## Notes

The v1 refinement results show changes in the method rankings relative to
stage 10. The refine_sandbox_v1 and v1_fix results use graph_iterations=26
and expanded search candidates (40), giving the proposed approach more room
to refine its architecture graph. The v4 sandbox explores additional settings.
All three runs preserve the 10-condition structure from stage 10 and use the
same seed set for paired comparison.
