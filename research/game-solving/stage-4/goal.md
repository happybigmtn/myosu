# Stage 4: Abstraction Taxonomy

## Objective

Catalog and classify the abstraction techniques used across imperfect-
information games -- card abstraction, tile abstraction, board abstraction,
action abstraction, and information set abstraction. Determine which
abstraction methods apply to which games and how abstraction quality affects
solver performance.

## Research Questions

### Primary

1. What card/tile/piece abstraction techniques have been used in the 20 games,
   and what are their information loss characteristics?

2. How does abstraction granularity affect exploitability -- is there a
   predictable relationship between abstraction size and solution quality?

3. Can abstraction techniques transfer across structurally similar games, or
   must they be designed game-specifically?

### Secondary

4. What is the computational cost of abstraction construction vs. the savings
   it provides during solving?

5. Which games in the 20-game set are most sensitive to abstraction quality
   (i.e., small changes in abstraction produce large changes in
   exploitability)?

6. Are there games where no effective abstraction is known, requiring the
   solver to operate on the full game?

7. How do learned abstractions (e.g., neural network embeddings) compare to
   hand-crafted abstractions (e.g., card bucketing by equity)?

## Scope

### Abstraction Categories

1. **Card/Tile Abstraction**: Grouping private information elements (cards,
   tiles, dominoes) into equivalence classes based on strategic similarity.
   - Lossless isomorphism (suit permutations)
   - Equity-based bucketing (expected value clustering)
   - Learned embeddings (autoencoder-derived)
   - Earth Mover's Distance (EMD) clustering

2. **Action Abstraction**: Reducing the action space by merging similar actions.
   - Bet size abstraction in poker (fixed fractions of pot)
   - Move grouping in board games (territorial equivalence)
   - Hierarchical action decomposition

3. **Information Set Abstraction**: Coarsening information sets to reduce the
   number of decision points.
   - Public state abstraction
   - Belief-state clustering
   - History compression

4. **Board/State Abstraction**: Reducing the spatial or structural complexity
   of the game state.
   - Grid coarsening
   - Feature-based state hashing
   - Symmetry exploitation

### Games Covered

All 20 games, with particular focus on Tier 3 and Tier 4 games (from stage 2)
where abstraction is essential for solver feasibility.

## Methodology

### Phase 1: Technique Catalog

For each abstraction category, compile all known techniques from the stage 1
literature survey. Record:

- Technique name and original paper
- Games it has been applied to
- Abstraction ratio (original size / abstracted size)
- Reported information loss metric
- Computational cost to construct

### Phase 2: Cross-Game Applicability Analysis

For each technique, assess which games in the 20-game set it could apply to:

- Direct applicability: the technique was designed for this game type
- Transferable: the technique can be adapted with minor modifications
- Inapplicable: the game structure does not support this technique

### Phase 3: Abstraction-Quality Experiments

For a subset of 5-6 games spanning complexity tiers, measure the relationship
between abstraction granularity and solution quality:

- Apply 3-4 levels of abstraction coarseness
- Solve each abstracted game using a fixed algorithm (CFR+ or Deep CFR)
- Measure exploitability at each abstraction level
- Plot the abstraction-quality tradeoff curve

### Phase 4: Taxonomy Synthesis

Produce a structured taxonomy mapping abstraction techniques to game properties,
with guidelines for selecting abstraction methods given a new game's structural
features.

## Expected Outcomes

1. A catalog of 20+ abstraction techniques organized by category.

2. A 20-by-4 applicability matrix (games x abstraction categories) showing
   which techniques apply to which games.

3. Abstraction-quality tradeoff curves for 5-6 representative games.

4. Guidelines document for abstraction selection given game properties.

5. Identification of games where abstraction is the primary bottleneck for
   solver improvement (feeding into architecture search priorities).

## Success Criteria

- Every Tier 3 and Tier 4 game has at least one recommended abstraction
  strategy.
- Abstraction-quality curves show monotonic improvement as abstraction
  becomes finer-grained (validating the abstraction approaches).
- The taxonomy covers all four abstraction categories.
- Cross-game transferability analysis identifies at least 3 technique-game
  pairs that have not been tried but should work based on structural
  similarity.

## Dependencies

- Stage 1 (Literature Survey): published abstraction techniques and results
- Stage 2 (Complexity Classification): tier assignments determining where
  abstraction is needed
- Stage 3 (Algorithm-Game Matching): which algorithms are paired with which
  games (since abstraction interacts with algorithm choice)

## Outputs

- `technique_catalog.json` -- structured catalog of abstraction techniques
- `applicability_matrix.csv` -- games x abstraction categories
- `tradeoff_curves/` -- directory of per-game abstraction-quality plots
- `selection_guidelines.md` -- decision tree for abstraction method selection

## Notes

Abstraction quality is a first-order concern for the architecture search in
stages 6-10. A poor abstraction can make even the best architecture
underperform. This stage ensures the architecture search operates on
appropriately abstracted game representations, isolating the architecture
effect from the abstraction effect.
