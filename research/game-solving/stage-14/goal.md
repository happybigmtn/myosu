# Stage 14: Cross-Game Validation

## Objective

Verify that the architecture rankings discovered by the proposed approach
transfer across the 20-game evaluation suite. Confirm that an architecture
found to be good on one subset of games also performs well on held-out games,
validating the generality of the architecture search results.

## Research Questions

### Primary

1. Does an architecture optimized on a subset of games (e.g., 10 games)
   maintain its ranking advantage on the remaining 10 held-out games?

2. What is the rank correlation between architecture performance on the
   training subset and the held-out subset (Spearman's rho)?

3. Are there game types (complexity tiers, structural categories) where
   transfer is strong vs. weak?

### Secondary

4. How many games are needed in the training subset for the architecture
   ranking to transfer (learning curve analysis)?

5. Does the proposed approach's advantage over baselines persist on held-out
   games, or does it partially stem from overfitting the training subset?

6. Are there specific architectural features that predict good transfer
   (e.g., simpler architectures transfer better)?

7. Does the refinement from stage 13 improve or hurt transferability compared
   to the stage 10 original?

## Scope

### Cross-Validation Protocol

1. **Random 50/50 Split**: randomly partition the 20 games into 10 training
   and 10 held-out games. Repeat 5 times with different random splits.

2. **Tier-Stratified Split**: ensure each split has proportional representation
   from each complexity tier (from stage 2).

3. **Leave-One-Tier-Out**: train on 3 tiers, test on the held-out tier. Four
   folds total.

### Methods Evaluated

All 10 conditions from stage 10, using the best configuration from stage 13:

- proposed_approach, proposed_variant
- Random Search, DARTS, AdamW, AugMax
- established_method_1, established_method_2
- without_key_component, simplified_version

### Transfer Metric

For each method, compute:

- Training subset primary metric (average across training games)
- Held-out subset primary metric (average across held-out games)
- Transfer gap: held-out metric - training metric
- Rank correlation between training and held-out rankings across methods

## Methodology

### Phase 1: Data Reanalysis

Using the per-game results from stages 10 and 13, partition the 20-game
metrics by game subset. No new experiments are needed -- the existing
per-seed, per-game data is re-aggregated under different subsets.

This is computationally cheap because the expensive runs (architecture search
and evaluation) were already done in stages 10 and 13.

### Phase 2: Random Split Analysis

For each of the 5 random 50/50 splits:

- Compute each method's mean primary metric on the training 10 games
- Compute each method's mean primary metric on the held-out 10 games
- Compute Spearman's rho between training and held-out rankings (across the
  10 methods)
- Compute each method's transfer gap

Average the rho and transfer gap across the 5 splits.

### Phase 3: Tier-Stratified Split Analysis

Same as Phase 2 but with tier-stratified splits. Compare the rho values with
the random splits to determine if tier stratification improves transfer
estimation.

### Phase 4: Leave-One-Tier-Out Analysis

For each of the 4 complexity tiers:

- Hold out all games from one tier
- Compute method rankings on the remaining 3 tiers
- Compute method rankings on the held-out tier
- Compute Spearman's rho

This reveals which tiers are the hardest to predict from the others.

### Phase 5: Transfer Gap Analysis

For the proposed approach specifically:

- Is the transfer gap positive (held-out performance is worse) or near-zero?
- Is the transfer gap larger for the proposed approach than for baselines
  (suggesting overfitting)?
- How does the transfer gap relate to the number of graph_iterations (more
  iterations = more specialization = potentially worse transfer)?

### Phase 6: Feature Importance for Transfer

Analyze which architectural features predict good transfer:

- Simpler architectures (fewer blocks, lower parameter count)
- Specific block types (attention may transfer better than game-specific
  encodings)
- Encoding choices (learned embeddings may transfer better than one-hot)

## Expected Outcomes

1. Spearman's rho values for architecture ranking transfer across random,
   stratified, and leave-one-tier-out splits.

2. Transfer gap measurements for each method, revealing which methods
   generalize and which overfit.

3. Per-tier transferability analysis showing which game types are most
   predictable from others.

4. Architectural feature analysis identifying transfer-friendly design choices.

## Success Criteria

- Average Spearman's rho >= 0.7 across random splits (strong transfer).
- The proposed approach's transfer gap is not significantly worse than
  baselines' transfer gap (no evidence of overfitting).
- The proposed approach maintains its ranking advantage on held-out games in
  at least 4 of 5 random splits.
- Leave-one-tier-out analysis achieves rho >= 0.5 for all 4 tiers.

## Dependencies

- Stage 10 (Proposed Approach): per-game, per-seed results
- Stage 13 (Refinement): refined configuration results
- Stage 2 (Complexity Classification): tier assignments for stratification
- Stage 12 (Statistical Analysis): statistical methods for correlation testing

## Outputs

- `random_split_results.json` -- 5-fold random split correlations and gaps
- `stratified_split_results.json` -- tier-stratified split analysis
- `leave_one_tier_out.json` -- per-tier transfer analysis
- `transfer_features.json` -- architectural features predicting transfer
- `cross_game_validation_summary.md` -- overall transfer assessment

## Notes

Cross-game validation is critical for the practical value of the research.
If architecture rankings do not transfer, the architecture search is only
useful when applied to a specific game -- it cannot produce general
architectural guidelines. Strong transfer would support the conclusion that
the proposed approach discovers architectures with genuinely superior
representation capacity, not just game-specific tuning.
