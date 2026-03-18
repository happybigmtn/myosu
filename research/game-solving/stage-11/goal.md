# Stage 11: Ablation Study

## Objective

Analyze the ablation variants (without_key_component, simplified_version)
from the stage-10 experiment to determine which components of the proposed
approach are essential for its performance advantage. Decompose the method's
contribution into its constituent parts.

## Research Questions

### Primary

1. How much performance does the proposed approach lose when its key component
   is removed (without_key_component variant)?

2. How much performance does the proposed approach lose when the method is
   simplified (simplified_version variant)?

3. Is the key component the primary driver of improvement over baselines, or
   does the simplified version already capture most of the benefit?

### Secondary

4. Does the relative importance of components vary across game complexity
   tiers?

5. Are the ablation variants still competitive with established methods, even
   if they underperform the full proposed approach?

6. What architectural differences emerge between the full proposed approach
   and its ablation variants -- do they find qualitatively different
   architectures?

7. Is the effect of removing the key component additive (independent of other
   components) or interactive (dependent on other components being present)?

## Scope

### Ablation Variants (Data from Stage 10)

1. **without_key_component**: The proposed approach with its distinguishing
   algorithmic component removed. This tests whether the component is
   necessary or whether the remaining framework is sufficient.

2. **simplified_version**: A streamlined version of the proposed approach
   with reduced complexity (fewer graph iterations, simpler architecture
   scoring). This tests whether the full complexity is justified.

### Comparison Framework

For each ablation variant, compare against:

- **proposed_approach**: to measure the component's contribution
- **Random Search**: to confirm the ablation still outperforms baselines
- **established_method_1 / _2**: to determine if the ablation remains
  competitive with state-of-the-art

### Decomposition Analysis

The ablation study enables a three-level decomposition:

```
Level 0: Random Search baseline               (primary_metric ~ 0.861)
Level 1: Proposed framework without key component (primary_metric ~ 0.704)
Level 2: Simplified version with key component    (primary_metric ~ 0.679)
Level 3: Full proposed approach                    (primary_metric ~ 0.707)
```

The improvement from Level 0 to Level 1 quantifies the framework's
contribution. The improvement from Level 1 to Level 3 quantifies the key
component's contribution.

## Methodology

### Phase 1: Component Contribution Quantification

Using the stage-10 results data, compute:

- Mean difference: proposed_approach - without_key_component
- Mean difference: proposed_approach - simplified_version
- Bootstrap CIs on both differences
- Effect sizes for both differences
- Percentage of the total improvement (vs. Random Search) attributable
  to each component

### Phase 2: Per-Seed Analysis

Examine whether the component contribution is consistent across seeds:

- Plot proposed_approach vs. without_key_component for each seed
- Identify seeds where removing the component helps (if any)
- Compute the proportion of seeds where the full method outperforms the
  ablation

### Phase 3: Per-Tier Breakdown

Analyze component importance by game complexity tier:

- Compute the component contribution separately for Tier 1-2, Tier 3, and
  Tier 4 games
- Determine if the key component is more important for complex games (where
  architecture quality matters more, per stage 3 analysis)

### Phase 4: Architectural Comparison

Compare the architectures discovered by the ablation variants vs. the full
proposed approach:

- Do they differ in block types, depth, encoding choices?
- Does the key component steer the search toward specific architectural
  patterns?
- Is the simplified version finding simpler architectures (fewer blocks,
  fewer parameters)?

### Phase 5: Interaction Analysis

Test for interaction effects:

- Is the benefit of the key component larger when other components are present
  (synergy)?
- Would adding the key component to a baseline method (e.g., Random Search +
  key component) produce similar improvement?

## Expected Outcomes

1. Quantified contribution of each component to the total improvement over
   baselines.

2. Evidence for whether both components are necessary or whether the
   simplified version is sufficient.

3. Per-tier analysis showing where each component matters most.

4. Architectural characterization of what the key component does to the
   search process.

## Success Criteria

- The component contribution analysis produces bootstrap CIs that do not
  include zero (each component contributes measurably).
- The decomposition accounts for at least 80% of the total improvement
  (no large unexplained residual).
- The per-tier analysis reveals a coherent pattern (not random variation).
- Architectural comparison identifies at least one concrete architectural
  feature attributable to the key component.

## Dependencies

- Stage 10 (Proposed Approach): complete results.json with all 10 conditions
  including ablation variants

## Outputs

- `component_contributions.json` -- quantified contribution of each component
- `per_seed_analysis.csv` -- seed-by-seed ablation comparison
- `per_tier_breakdown.csv` -- tier-specific component importance
- `architecture_comparison.json` -- architectural differences across variants
- `interaction_analysis.json` -- synergy/independence test results

## Notes

The stage-10 results show without_key_component at primary_metric_mean 0.704
and simplified_version at 0.679. The full proposed_approach is at 0.707.
This suggests the key component provides a modest improvement (~0.003 in
primary metric) while the simplified version actually achieves better
exploitability (0.679 < 0.707). This warrants careful investigation -- the
simplified version outperforming the full method on the primary metric may
indicate that the key component introduces noise in some settings, or that
the additional complexity does not translate to better exploitability on
average. The secondary metric tells a different story and should be examined.
