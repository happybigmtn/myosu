# Stage 2: Game Complexity Classification

## Objective

Rank and classify the 20 imperfect-information games by their computational
complexity dimensions: number of information sets, branching factor, state
space size, game tree depth, and degree of imperfect information. Produce a
quantitative complexity profile for each game that drives algorithm selection
in stage 3.

## Research Questions

### Primary

1. What is the information set count for each game, and how does it compare
   across the 20-game set?

2. What is the effective branching factor at each decision point, and how does
   it vary across game phases (early, mid, late)?

3. What is the total state space size (number of game states reachable from the
   root), and can it be computed exactly or must it be estimated?

### Secondary

4. What is the ratio of information sets to game states (a measure of
   information asymmetry)?

5. Which games exhibit chance nodes, and what is the chance branching factor?

6. How do partnership/team structures affect the effective complexity (e.g.,
   cooperative information sharing in Bridge vs. adversarial in Poker)?

7. Are there games with state spaces small enough for exact equilibrium
   computation vs. those requiring approximation?

## Scope

All 20 games from the myosu survey. For each game, compute or estimate the
following complexity dimensions:

| Dimension                | Notation  | Description                              |
|--------------------------|-----------|------------------------------------------|
| Information set count    | |H|       | Total information sets across all players|
| State space size         | |S|       | Reachable game states                    |
| Game tree size           | |T|       | Nodes in the unfolded game tree          |
| Average branching factor | b_avg     | Mean actions per decision point          |
| Max branching factor     | b_max     | Largest action set at any decision point |
| Game tree depth          | d         | Longest path from root to terminal       |
| Chance factor            | c         | Branching factor of chance nodes         |
| Info asymmetry ratio     | |H|/|S|   | Degree of information hiding             |
| Player count             | n         | Number of decision-making agents         |

## Methodology

### Phase 1: Exact Computation (Small Games)

For games with tractable state spaces (< 10^8 states), enumerate the game tree
programmatically. Record exact values for all complexity dimensions.

Tools: Custom game tree enumerators, OpenSpiel game implementations.

### Phase 2: Estimation (Large Games)

For games with intractable state spaces, use:

- Sampling-based estimation: random game rollouts to estimate branching factor
  distributions and depth
- Combinatorial counting: card/tile permutation counts for chance node analysis
- Published bounds: leverage state space estimates from the literature survey
  (stage 1)

### Phase 3: Classification

Assign each game to a complexity tier:

- **Tier 1 (Toy)**: |S| < 10^6, suitable for tabular exact methods
- **Tier 2 (Small)**: 10^6 <= |S| < 10^12, abstraction or function
  approximation helpful
- **Tier 3 (Medium)**: 10^12 <= |S| < 10^20, requires abstraction and deep
  learning
- **Tier 4 (Large)**: |S| >= 10^20, requires aggressive abstraction, may be
  intractable for current methods

### Phase 4: Ranking

Produce a composite complexity score that weights information set count,
branching factor, and state space size. Rank games from simplest to most
complex.

## Expected Outcomes

1. A complexity profile table with all 9 dimensions for each of the 20 games.

2. A tier assignment (1-4) for each game.

3. A composite complexity ranking from 1 (simplest) to 20 (most complex).

4. Identification of games that are "boundary cases" -- near a tier threshold
   and potentially interesting for algorithm stress-testing.

5. Visualization of the complexity landscape (PCA or similar dimensionality
   reduction over the 9 dimensions).

## Success Criteria

- All 20 games have at least estimated values for all 9 complexity dimensions.
- Exact values computed for all Tier 1 and Tier 2 games.
- Tier assignments are consistent with known results from the literature (e.g.,
  Kuhn Poker is Tier 1, No-Limit Texas Hold'em is Tier 4).
- The composite ranking produces a clear ordering without ties.

## Dependencies

- Stage 1 (Literature Survey): published state space estimates and complexity
  results.

## Outputs

- `complexity_profiles.csv` -- 20 rows x 9 columns of complexity dimensions
- `tier_assignments.json` -- game-to-tier mapping
- `complexity_ranking.json` -- ordered list with composite scores

## Notes

The tier boundaries are chosen to align with practical algorithm capability
thresholds. Tier 1 games can be solved exactly. Tier 2 games are at the edge
of tabular methods. Tier 3 and 4 games require the architecture search
approach that this pipeline is building toward.
