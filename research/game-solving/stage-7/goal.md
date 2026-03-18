# Stage 7: Proxy Task Selection

## Objective

Establish CIFAR-100 as a computationally cheap proxy task for ranking neural
network architectures, then validate that architecture rankings on the proxy
task correlate with rankings on the actual imperfect-information game tasks.
This enables rapid architecture evaluation without expensive game solver
training.

## Research Questions

### Primary

1. Does the rank correlation between CIFAR-100 performance and game solver
   performance exceed 0.6 (Spearman's rho), making CIFAR-100 a useful proxy?

2. What is the minimum CIFAR-100 training duration that preserves rank
   correlation with full game solver training?

3. Does the proxy task ranking correlation vary across game complexity tiers,
   or is it consistent?

### Secondary

4. Are there architecture features that perform well on CIFAR-100 but poorly
   on game tasks (false positives), and can they be identified in advance?

5. Does fine-tuning a CIFAR-100-ranked architecture on game data preserve its
   relative ranking, or does the ordering change significantly?

6. Would a simpler proxy task (CIFAR-10, MNIST) be sufficient, or is the
   additional complexity of CIFAR-100 needed to discriminate architectures?

7. What adaptations are needed to make game-solver architectures compatible
   with image classification (and vice versa)?

## Scope

### Proxy Task Definition

- Dataset: CIFAR-100 (100 classes, 50K train, 10K test, 32x32 RGB images)
- Training protocol: fixed for all architectures (SGD, cosine annealing,
  standard augmentation, 50 epochs)
- Metric: top-1 test accuracy at epoch 50
- Budget per architecture: ~5 minutes on a single GPU

### Architecture Adaptation

Architectures from the stage-6 search space are adapted for CIFAR-100:

- Replace game-state encoding with a 3-channel image input layer
- Replace action probability output with 100-class softmax
- Keep all processing blocks (MLP, Conv, Attention, etc.) unchanged
- This tests the processing architecture's representation capacity
  independent of the game-specific input/output layers

### Validation Protocol

1. Sample 36 architectures from the search space (random_search_candidates in
   stage-10 results = 36)
2. Train each on CIFAR-100 for 50 epochs
3. Train each on 3 games (one per complexity tier) using the matched algorithm
   from stage 3
4. Compute Spearman rank correlation between CIFAR-100 accuracy and game solver
   primary metric

## Methodology

### Phase 1: CIFAR-100 Training Protocol Calibration

Establish a fixed training protocol that:

- Produces stable accuracy rankings (low ranking noise across random seeds)
- Runs in < 10 minutes per architecture (enabling rapid evaluation)
- Uses standard hyperparameters (not architecture-specific tuning) to
  ensure the ranking reflects architecture quality, not tuning quality

Test 3 candidate protocols (different learning rates, schedulers) on 10
architectures and select the one with lowest ranking variance across seeds.

### Phase 2: Architecture-Game Training

For each of the 36 sampled architectures, train on 3 representative games:

- Tier 2 game: small enough for fast iteration
- Tier 3 game: the primary target complexity level
- Tier 4 game: stress test for large-scale games

Training uses the matched algorithm from stage 3 with fixed hyperparameters
(not architecture-specific tuning), matching the CIFAR-100 protocol's
philosophy.

### Phase 3: Correlation Analysis

Compute:

- Spearman's rho between CIFAR-100 accuracy and primary metric for each game
- Kendall's tau as a secondary rank correlation measure
- Scatter plots with architecture features annotated
- Analysis of correlation outliers (architectures that rank differently on
  proxy vs. game task)

### Phase 4: Proxy Task Validation Decision

Based on correlation analysis:

- If rho >= 0.6 for all 3 games: adopt CIFAR-100 as primary proxy
- If rho >= 0.6 for 2/3 games: adopt with caveats, note which game type
  is poorly correlated
- If rho < 0.6 for 2+ games: reject CIFAR-100, consider alternative proxy
  tasks or direct game evaluation only

## Expected Outcomes

1. Rank correlation coefficients between CIFAR-100 and game tasks for 3
   representative games.

2. A validated (or rejected) proxy task with documented correlation strength.

3. Identification of architecture features that are proxy-predictive vs.
   proxy-misleading.

4. A calibrated CIFAR-100 training protocol for use in stages 8-10.

5. Estimated speedup factor from using the proxy task (wall-clock time for
   CIFAR-100 ranking vs. direct game evaluation).

## Success Criteria

- Spearman's rho >= 0.6 for at least 2 of 3 test games.
- The CIFAR-100 training protocol produces ranking variance (across seeds)
  < 0.1 standard deviation in rank.
- The proxy evaluation is at least 10x faster than direct game evaluation.
- No systematic false-positive pattern (architecture type that always ranks
  high on proxy but low on games).

## Dependencies

- Stage 5 (Metric Design): primary metric definition for game evaluation
- Stage 6 (Architecture Candidates): search space specification and sampler

## Outputs

- `proxy_protocol.json` -- CIFAR-100 training configuration
- `correlation_results.json` -- rank correlations per game
- `proxy_validation_decision.md` -- accept/reject decision with evidence
- `architecture_rankings_proxy.csv` -- 36 architectures ranked by CIFAR-100
- `architecture_rankings_game.csv` -- same 36 ranked by game performance

## Notes

The use of CIFAR-100 as a proxy for game solving is a methodological choice
that requires strong validation. The core assumption is that the processing
architecture's capacity to learn hierarchical representations (tested by
image classification) predicts its capacity to learn game state
representations. This is not guaranteed and the validation in this stage is
a necessary checkpoint before committing to proxy-based search in stages 8-10.
