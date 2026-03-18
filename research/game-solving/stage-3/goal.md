# Stage 3: Algorithm-Game Matching

## Objective

Determine which solver algorithms are appropriate for which games based on
the complexity profiles from stage 2 and the literature coverage from stage 1.
Produce a recommendation matrix that maps each game to its most promising
algorithm families, with justification.

## Research Questions

### Primary

1. For each complexity tier (1-4), which algorithm families are computationally
   feasible and have demonstrated convergence guarantees or empirical success?

2. Which game-specific structural properties (sequential vs. simultaneous
   moves, public vs. private information, team vs. adversarial) constrain
   algorithm selection beyond raw complexity?

3. Are there games where multiple algorithm families are competitive, making
   them good candidates for architecture search (the focus of stages 6-10)?

### Secondary

4. For games with existing solver implementations (from stage 1), what is the
   gap between the best published result and the theoretical optimum?

5. Which algorithm families have the most room for improvement through
   architectural innovation (the core thesis of this research)?

6. Are there algorithm-game combinations that are theoretically sound but have
   never been attempted, representing low-hanging fruit?

## Scope

### Algorithm Families Under Consideration

1. **Tabular CFR variants**: Vanilla CFR, CFR+, LCFR, DCFR
2. **Deep CFR / function approximation CFR**: Deep CFR, DREAM, ARMAC
3. **Neural Fictitious Self-Play (NFSP)**: and its variants
4. **Policy gradient methods**: PPO, A2C, SAC adapted for imperfect info
5. **Monte Carlo Tree Search with information sets**: IS-MCTS, POMCP, ISMCTS
6. **Regret-based search**: ReBeL, Player of Games
7. **Evolutionary / population-based**: PSRO, double oracle

### Matching Criteria

For each game-algorithm pair, evaluate:

- **Feasibility**: Can the algorithm handle the game's state space within
  reasonable compute budget (< 10^4 GPU-hours)?
- **Convergence**: Does the algorithm have convergence guarantees for this
  game structure (2-player zero-sum, multiplayer, cooperative)?
- **Prior evidence**: Has this combination been attempted? What were the results?
- **Architecture sensitivity**: How much does performance depend on network
  architecture choices (relevant to the architecture search focus)?

## Methodology

### Phase 1: Feasibility Filtering

For each game, eliminate algorithm families that cannot handle the game's
complexity tier:

- Tier 1: All families feasible
- Tier 2: Tabular CFR may be borderline; all others feasible
- Tier 3: Tabular CFR infeasible; deep methods required
- Tier 4: Only deep methods with aggressive abstraction feasible

### Phase 2: Structural Matching

Apply structural constraints:

- 2-player zero-sum games: CFR variants have convergence guarantees
- Multiplayer games: CFR convergence not guaranteed; NFSP or policy gradient
  preferred
- Team/cooperative games: team-aware equilibrium concepts required; standard
  CFR needs modification
- Simultaneous-move games: normal-form game solvers or extensive-form conversion

### Phase 3: Architecture Sensitivity Analysis

For each feasible game-algorithm pair, assess how much the algorithm's
performance varies with network architecture choices:

- Low sensitivity: algorithm is robust to architecture (e.g., tabular CFR)
- Medium sensitivity: architecture matters but known designs work well
- High sensitivity: performance highly dependent on architecture, making this
  pair a strong candidate for architecture search

### Phase 4: Recommendation Synthesis

Produce the final recommendation matrix with confidence levels:

- **Recommended**: feasible, convergence-appropriate, architecture-sensitive
- **Viable**: feasible and convergence-appropriate but low architecture
  sensitivity
- **Exploratory**: feasible but untested for this game structure
- **Not recommended**: infeasible or convergence-inappropriate

## Expected Outcomes

1. A 20-by-7 recommendation matrix (games x algorithm families) with
   confidence levels.

2. For each game, a ranked list of recommended algorithms with justification.

3. Identification of 5-8 game-algorithm pairs with high architecture
   sensitivity that are priority targets for the architecture search pipeline.

4. A list of unexplored game-algorithm combinations that represent research
   opportunities.

## Success Criteria

- Every game has at least one "Recommended" algorithm family.
- The high-architecture-sensitivity pairs span at least 3 complexity tiers,
  ensuring the architecture search results generalize.
- Recommendations are consistent with published results (i.e., known
  successful combinations are marked "Recommended").

## Dependencies

- Stage 1 (Literature Survey): algorithm coverage per game
- Stage 2 (Complexity Classification): tier assignments and complexity profiles

## Outputs

- `recommendation_matrix.csv` -- 20 x 7 matrix with confidence levels
- `priority_targets.json` -- high architecture-sensitivity pairs for stage 6+
- `matching_rationale.md` -- per-game justification narrative

## Notes

The architecture sensitivity analysis is the key novel contribution of this
stage. Standard algorithm selection considers only feasibility and convergence.
By adding the architecture sensitivity dimension, we identify where
architecture search (stages 6-10) will have the most impact.
