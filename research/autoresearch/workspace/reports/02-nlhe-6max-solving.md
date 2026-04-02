# No-Limit Hold'em 6-Max: Solving Strategies Research Report

## Executive Summary

No-Limit Hold'em 6-Max (NLHE 6-max) represents the frontier of imperfect-information game solving. Unlike heads-up (HU) poker, which was effectively solved by Libratus (2017) and Cepheus (limit, 2015), the 6-player variant introduces fundamental theoretical barriers: Nash equilibria are non-unique, non-interchangeable, and computationally intractable (PPAD-hard). Despite this, Pluribus (Brown & Sandholm, 2019) demonstrated superhuman play using a blueprint + real-time search architecture that sidesteps the need for exact equilibrium computation.

The field has since advanced along several axes: faster CFR convergence via hyperparameter schedules (Zhang et al., 2024), neural function approximation replacing tabular methods (DeepCFR, DREAM, Kdb-D2CFR), knowledge-limited subgame solving for scalability (OLSS, Liu et al., 2023), unified search+learning frameworks (Student of Games, Schmid et al., 2023), and the emergence of Quantal Response Equilibrium (QRE) as an alternative to Nash in commercial solvers (GTO Wizard, 2025). LLM-based poker agents have also appeared (PokerBench, SpinGPT, 2025), though they remain far from solver-grade play.

The core tension in 6-max solving is between equilibrium approximation (safe but sub-optimal in practice) and exploitative adaptation (profitable but theoretically unsafe). The most promising modern approaches combine both: a CFR-derived blueprint provides a safety floor while real-time opponent modeling captures excess value. For Myosu's subnet, this means evaluating solvers on empirical win-rate across positions against diverse opponents, not on exploitability metrics that are undefined for multiplayer games.

---

## Game Complexity Analysis

### How 6-Player Changes the Game Tree vs Heads-Up

The transition from 2 to 6 players is not a linear scaling -- it is a combinatorial explosion across every dimension of the game:

| Dimension | Heads-Up | 6-Max | Scaling Factor |
|-----------|----------|-------|----------------|
| Preflop action sequences | ~170 distinct | ~10,000+ distinct | ~60x |
| Per-street decision points | 1 opponent to act | Up to 5 opponents to act | 3-5x branching per street |
| Private card combinations | 1,326 opponent hands | 1,326^5 joint opponent hands | Combinatorial |
| Raw game tree nodes | ~10^161 | >10^170 | >10^9x |
| Information sets (no abstraction) | ~10^161 | Intractable | -- |
| Information sets (with abstraction) | ~10^12 (Libratus) | ~10^12 (Pluribus, heavily abstracted) | Comparable via aggressive abstraction |

The critical difference is not merely size but structure. In HU, the game is two-player zero-sum: Nash equilibria are unique in value, interchangeable, and computable via linear programming. With 3+ players, none of these properties hold.

### Multiway Pot Dynamics

Multiway pots change the strategic landscape fundamentally:

- **Equity distribution narrows.** With 6 players seeing a flop, the best hand preflop (AA) has ~49% equity vs. ~85% in HU. Hands run closer together, reducing the value of aggressive play.
- **Bluffing efficacy drops.** A bluff must fold out all remaining opponents, not just one. The probability of success scales as p^(n-1) where p is per-opponent fold frequency and n is the number of opponents.
- **Implicit coordination.** Even without collusion, opponents' independent actions can create correlated effects. When two opponents both call, the third faces a qualitatively different situation than if only one called.
- **Side pot complexity.** Multiple all-in scenarios create independent sub-contests within a single hand, each requiring separate strategic reasoning.

---

## Historical Progression

### From HU Solvers to Pluribus and Beyond

| Year | Milestone | Significance |
|------|-----------|--------------|
| 2007 | Zinkevich et al., CFR | Foundation algorithm for imperfect-information game solving |
| 2013 | Johanson et al., CFR for multiplayer | First demonstration that CFR generates competitive 3-player agents (ACPC) |
| 2014 | CFR+ (Tammelin) | Non-negative regrets, faster convergence, solved Limit Hold'em HU |
| 2015 | Cepheus | Essentially solved Limit Hold'em HU |
| 2016 | Heinrich & Silver, NFSP | First end-to-end deep RL approach approximating Nash from self-play |
| 2017 | Libratus (Brown & Sandholm) | Superhuman HU NLHE via blueprint + nested subgame solving |
| 2017 | DeepStack (Moravcik et al.) | Superhuman HU NLHE via continual re-solving with neural value networks |
| 2018 | Brown & Sandholm, Depth-Limited Solving | Reduced subgame solving cost by not extending to end of game |
| 2019 | Pluribus (Brown & Sandholm) | Superhuman 6-max NLHE, published in *Science* |
| 2019 | DCFR (Brown & Sandholm) | Discounted regret weighting, 2-3x faster convergence than CFR+ |
| 2019 | DeepCFR (Brown et al.) | Neural function approximation replaces tabular CFR |
| 2020 | ReBeL (Brown et al.) | Unified RL+search for imperfect-info games via public belief states |
| 2020 | DREAM (Steinberger) | Model-free deep regret minimization, no perfect simulator needed |
| 2022 | AlphaHoldem | End-to-end actor-critic RL, superhuman HU on commodity hardware |
| 2023 | Student of Games (Schmid et al.) | Unified algorithm for perfect and imperfect-info games |
| 2023 | Liu et al., OLSS | Opponent-limited online search, orders of magnitude faster subgame solving |
| 2023 | Kdb-D2CFR | Knowledge distillation from 2-player to multiplayer DeepCFR |
| 2024 | Zhang et al., Hyperparameter Schedules | Orders-of-magnitude faster CFR convergence via dynamic discount tuning |
| 2025 | GTO Wizard QRE | Commercial solver switches from Nash to Quantal Response Equilibrium |
| 2025 | "Beyond GTO" (Yang et al.) | Adaptive agent combining CFR baseline with real-time exploitative play |
| 2025 | SpinGPT | LLM fine-tuned on solver data, 78% action match in 3-player Spin & Go |

---

## Current Best Approaches

### 1. Blueprint + Real-Time Search (Pluribus Architecture)

**The proven paradigm for multiplayer NLHE.** Pluribus remains the only system to empirically demonstrate superhuman 6-max play.

**Blueprint computation:**
- MCCFR with linear weighting (LCFR): iteration t weighted by t, converging up to 100x faster than vanilla CFR.
- Computed on an abstracted game (~10^12 information sets) using card buckets (based on equity distributions and Earth Mover's Distance) and discretized bet sizes (~200 sizes in blueprint).
- Training cost: ~12,400 CPU core-hours on a 64-core server over 8 days. No GPUs required.

**Real-time search:**
- Blueprint used only for preflop (small decision tree).
- From the flop onward, depth-limited search constructs a subgame with finer-grained abstraction (~4-8 bet sizes).
- Key innovation: at leaf nodes, Pluribus does not assume opponents follow a single continuation strategy. Instead, it considers multiple "search policies" -- perturbations of the blueprint (fold-biased, call-biased, raise-biased). This addresses the theoretical unsoundness of subgame solving in multiplayer settings.
- Opponent hand ranges maintained via Bayesian updates over 1,326 possible holdings per opponent.

**Limitations:**
- No exploitability bound. The strategy is validated only empirically.
- Search policies are heuristic; no proof they capture the right space of opponent behaviors.
- The blueprint's abstraction introduces errors that propagate through real-time search.

### 2. Deep CFR Family (DeepCFR, D2CFR, Kdb-D2CFR)

**Neural function approximation replaces tabular regret storage.**

DeepCFR (Brown et al., 2019) trains two neural networks:
- A **regret network** that predicts counterfactual regret values for each action at each information set.
- A **policy network** that predicts the average strategy.

This eliminates the need for pre-computed card abstractions -- the network learns to generalize across similar hands. In principle, this handles the exponential state space more gracefully than fixed abstraction schemes.

**Kdb-D2CFR (2023)** extends this to multiplayer games via knowledge distillation:
- Train a strong 2-player DeepCFR model.
- Transfer its learned representations to initialize a multiplayer model.
- Fine-tune on the multiplayer game.
- First DeepCFR-based method demonstrated to work on multiplayer imperfect-information games.

**Limitations:**
- Training instability with neural approximation. Recent work (2025) identifies theoretical risks in neural MCCFR and proposes mitigation frameworks.
- Convergence guarantees weaker than tabular CFR even in two-player games.
- Compute cost higher than tabular MCCFR for games where tabular methods fit in memory.

### 3. DREAM (Deep Regret Minimization with Advantage Baselines)

**Model-free deep RL for imperfect-information games.**

DREAM (Steinberger, 2020) converges to:
- Nash equilibrium in two-player zero-sum games.
- Extensive-form coarse correlated equilibrium (EFCCE) in general multiplayer games.

Key advantage: does not require a perfect game simulator, unlike DeepCFR. Uses a learned baseline to maintain low variance when sampling single actions per decision point.

**Relevance to 6-max:** DREAM's convergence to EFCCE (rather than Nash) is arguably more appropriate for multiplayer poker, since EFCCE is:
- Computationally more tractable than Nash in general-sum games.
- Can be induced by no-regret learning dynamics (each player running CFR independently).
- Accounts for the correlation structure inherent in multiplayer interactions.

### 4. ReBeL (Recursive Belief-based Learning)

**Unified RL+search for imperfect-information games via public belief states (PBS).**

ReBeL (Brown et al., 2020, NeurIPS) factors the game into a sequence of public belief states and uses a learned value function over these states to guide search. It converges to Nash equilibrium in any two-player zero-sum game.

- Defeated professional poker player Dong Kim in HU NLHE.
- Defeated benchmark bots BabyTartanian8 and Slumbot.
- Open-sourced for Liar's Dice.

**Multiplayer extension:** ReBeL's PBS formulation does not directly extend to multiplayer games with strong guarantees, because the belief update dynamics become more complex when multiple opponents' beliefs must be tracked simultaneously.

### 5. Student of Games

**General-purpose algorithm combining guided search, self-play learning, and game-theoretic reasoning (Schmid et al., 2023, *Science Advances*).**

- Achieves strong performance in chess, Go, HU NLHE poker, and Scotland Yard (multiplayer imperfect-info).
- Converges to perfect play as computation increases.
- Represents the most domain-general approach, but has not been specifically benchmarked on 6-max NLHE.

### 6. Actor-Critic RL (AlphaHoldem and Beyond)

**End-to-end reinforcement learning without explicit game-theoretic structure.**

AlphaHoldem (2022) achieves superhuman HU NLHE performance via:
- Pseudo-siamese architecture learning from state to action.
- Novel actor-critic loss function improving learning stability.
- Self-play against historical agent pool for diversity.
- Trained in 3 days on a single PC.

For multiplayer, actor-critic approaches face the challenge that the reward signal is noisier (outcomes depend on 5 other players' actions). Recent work (2022) explores multiplayer poker via actor-critic RL with promising but not superhuman results.

### 7. LLM-Based Approaches (Emerging, 2025)

**Large language models as poker decision-makers.**

- **PokerBench** (2025): Benchmark of 11,000 scenarios shows all SOTA LLMs underperform optimal play, though fine-tuning helps.
- **SpinGPT** (2025): LLM fine-tuned via SFT + RL on solver data for 3-player Spin & Go, matching solver actions 78% of the time.
- **PokerGPT**: End-to-end lightweight solver for multi-player Hold'em via LLM.
- **LLM Battle** (October 2025): Nine frontier LLMs played $10/$20 NLHE cash game over 5 days. Top models demonstrated competent deep-stack play without specialized training.

**Assessment:** LLM approaches are conceptually interesting for opponent modeling and strategic reasoning but fundamentally limited by their inability to perform the precise probability calculations that CFR-based methods excel at. They are likely complementary (opponent modeling layer) rather than competitive as primary solvers.

### 8. Quantal Response Equilibrium (QRE)

**Commercial solver innovation replacing Nash (GTO Wizard, April 2025).**

QRE (McKelvey & Palfrey, 1995) is a solution concept with bounded rationality: players make small mistakes proportional to a rationality parameter lambda. In poker solving:

- Produces well-defined strategies even in "ghost nodes" (spots that Nash says should never be reached).
- Optimizes against imperfect opponents rather than assuming perfect play.
- Reduced average flop exploitability by 25% vs. Nash in GTO Wizard benchmarks.
- Particularly valuable for multiway spots where Nash equilibrium behavior at low-frequency nodes is poorly defined.

**Relevance:** QRE may be more practically relevant than Nash for multiplayer poker because real opponents are imperfect. However, it introduces a free parameter (lambda) that must be calibrated to the opponent population.

---

## Key Challenge: No Nash Equilibrium Guarantee in Multiplayer

### Why Nash Doesn't Directly Apply

In two-player zero-sum games, the minimax theorem guarantees:
1. A Nash equilibrium exists and is computable.
2. All Nash equilibria have the same value to each player.
3. Nash strategies are interchangeable (mixing strategies from different equilibria remains a Nash equilibrium).
4. Playing a Nash equilibrium guarantees at least breaking even against any opponent.

**None of these hold with 3+ players:**

1. **Existence:** Nash equilibria exist (Nash's theorem) but are non-constructive.
2. **Non-uniqueness:** Infinitely many Nash equilibria exist with different values to each player.
3. **Non-interchangeability:** Combining strategy components from different equilibria can create exploitable strategies.
4. **No safety guarantee:** A Nash equilibrium strategy can lose money if other players collude or even if they independently play a different equilibrium.

### Computational Hardness

- Finding an exact Nash equilibrium in multiplayer games is PPAD-complete.
- Finding an epsilon-Nash (additive approximation) is also PPAD-hard for constant epsilon.
- Computing an epsilon-approximate Nash in an n-player game requires 2^Omega(n) oracle queries to payoff tensors.
- For 6-player poker specifically: no polynomial-time algorithm is known.

### Alternative Solution Concepts

**Correlated Equilibrium (CE):**
- Players' strategies may be correlated via a mediator.
- Computable in polynomial time via linear programming (for normal-form games).
- Extensive-Form Correlated Equilibrium (EFCE) and Extensive-Form Coarse Correlated Equilibrium (EFCCE) are the relevant variants for sequential games.

**EFCCE:**
- Can be approached via no-regret dynamics (each player running independent CFR).
- DREAM converges to EFCCE in general games.
- More tractable than Nash but provides weaker guarantees.

**Computing optimal CE in extensive-form games:**
- Social-welfare-maximizing EFCCE/NFCCE can be computed via column generation (polynomial time in two-player, no-chance-move games).
- NP-hard in general multiplayer games with chance moves.
- Two-sided column generation and correlation DAG methods (Farina et al., 2022) outperform prior approaches.

### Practical Implications for 6-Max

Pluribus sidesteps the equilibrium question entirely: it uses MCCFR to compute an approximate strategy (not provably a Nash equilibrium) and validates it empirically. The strategy Pluribus plays has no formal name in equilibrium theory -- it is best described as "the output of MCCFR run for N iterations on an abstracted game, refined by heuristic real-time search."

This pragmatic approach works because:
- Human opponents are far from any equilibrium.
- The strategy only needs to be "good enough" against the actual opponent population.
- Empirical validation (win-rate over many hands) is more informative than theoretical exploitability bounds that are undefined for multiplayer games.

---

## Abstraction Techniques

### Card Abstraction

**Goal:** Reduce the number of distinct private information states by clustering strategically similar hands.

**Dominant approach:** Potential-aware abstraction with Earth Mover's Distance (PAAEMD):
1. For each hand on each street, compute an equity histogram -- the probability distribution over possible hand strengths on the next street.
2. Measure distance between hands using Earth Mover's Distance (EMD) between their equity histograms.
3. Cluster hands into k buckets using k-means with EMD as the distance metric.

**EMD advantages:** Captures the full distribution of future equity, not just expected value. Two hands with the same average equity but different distributions (e.g., a made hand vs. a strong draw) are correctly distinguished.

**Recent improvements:**
- KrwEmd (2024): clusters signal observation information sets using EMD to address over-abstraction.
- Imperfect-recall abstractions: allow the abstract game to "forget" certain action history details, reducing the game tree while preserving key strategic structure.

### Action Abstraction

**Goal:** Discretize the continuous bet-sizing space into a manageable number of options.

**Standard approach:** Express bets as fractions of the pot:
- Common sizes: 0.25x, 0.33x, 0.5x, 0.67x, 1.0x, 1.5x, 2.0x pot, and all-in.
- Pluribus used ~200 sizes in the blueprint and 4-8 sizes in real-time search.
- The choice of which sizes to include is critical and often requires domain expertise.

**Scaling to 6 players:**
- The branching factor per street scales as O(b^n) where b is the number of bet sizes and n is the number of active players.
- With 6 bet sizes and 5 active opponents: 6^5 = 7,776 possible action combinations per decision point, vs. 6^1 = 6 in HU.
- This forces much more aggressive abstraction in multiplayer, typically reducing to 2-4 bet sizes per player in most decision contexts.

### Abstraction Scaling Challenges

The exponential blowup in multiplayer games means:
- **Blueprint quality degrades faster with abstraction.** Errors from coarse abstraction compound across multiple opponent decision points.
- **Generating best-response strategies is infeasible.** Computing an abstract game best response for a multiplayer game is intractable -- a single CFR best-response run on a 3-player game can take months.
- **Abstraction refinement during search is essential.** The blueprint's coarse abstraction must be refined in real-time for the specific subgame encountered, as Pluribus does.

---

## Exploitative Play

### When and How to Deviate from Equilibrium

**The fundamental tension:** In multiplayer poker, strict equilibrium play (even if computable) is not guaranteed to maximize profit. Against imperfect opponents, exploitative deviations capture excess value.

**Key findings from recent research:**

**Machine learning outperforms GTO against humans.** ML-based approaches that learn from opponent behavior generate more profit than a GTO game style against human opponents (Survey on GTO Poker, 2024). This is especially true in multiplayer where theoretical approaches struggle.

**Safe exploitation.** Listy et al. (2023) develop algorithms for exploiting opponents while guaranteeing at least the value of the game. The agent adjusts its strategy based on observed opponent tendencies while maintaining a floor on expected value.

**"Beyond GTO" (Yang et al., 2025).** An adaptive model that:
1. Learns GTO behavior from self-play via MCCFR.
2. Continuously tracks opponents' tendencies in real-time.
3. Shifts strategy to capture excess value while maintaining provable safety against counter-exploitation.
4. MCCFR performs best in HU situations; remains strongest in most multiway situations.

**Evolved RNNs for exploitation.** Poker agents using evolved recurrent neural networks can adapt to unseen opponents and exploit weak strategies far more effectively than Nash-based agents (Li et al., 2018). The RNN captures opponent patterns that static equilibrium strategies cannot.

**Opponent modeling in multiplayer (2024).** Recent work on opponent modeling in multiplayer imperfect-information games shows that state-of-the-art Nash-based approaches lack the ability to model and exploit opponents effectively. The ability to build and update opponent models during play is critical for multiplayer performance.

### The Collusion Problem

**Express collusion:** Two or more agents sharing private information and coordinating strategy. Detectable via information-theoretic approaches (mutual information between agent actions). Colluding partners play more aggressively when either has a strong hand.

**Implicit collusion:** Even without communication, multiplayer Nash equilibria can involve coordinated punishment strategies. A coalition can withstand punishments if and only if its members have highly aligned interests.

**Detection methods:** Domain-independent mutual information analysis provides good indication of collusive behavior, and extends to partially observable sequential games like poker.

**For Myosu's subnet:** anti-collusion measures are critical. Evaluation games must randomize seat assignments, use anonymous play, and potentially analyze strategy correlations across submitted agents.

---

## Open Problems

### 1. Theoretically Sound Multiplayer Subgame Solving
Pluribus's search heuristic works empirically but has no theoretical guarantees. Developing subgame solving techniques that are provably safe in n-player settings remains open.

### 2. Scalable Exploitability Metrics for Multiplayer
There is no clean analog of exploitability (the standard metric for 2-player solver quality) in multiplayer games. Evaluation must rely on empirical win-rates, which are noisy and require many hands for statistical significance.

### 3. Optimal Abstraction for Multiplayer
Most abstraction research targets 2-player games. The interaction between card abstraction and action abstraction in 6-player settings is poorly understood. How to optimally allocate abstraction budget across players and streets is an open question.

### 4. Convergence of CFR in General-Sum Games
CFR has no convergence guarantee to Nash equilibrium in games with 3+ players. It converges to EFCCE, which is a weaker solution concept. Whether practical variants can achieve tighter approximations in poker-sized games is unknown.

### 5. Neural Architecture for Multiplayer Poker
DeepCFR and DREAM have been validated primarily on 2-3 player games. Scaling neural function approximation to 6-player NLHE while maintaining training stability is an active research area (Kdb-D2CFR is a first step).

### 6. Bridging LLMs and Solvers
LLMs show surprising poker competence but lack precise probability reasoning. How to combine LLM-based opponent modeling with CFR-based strategy computation is unexplored.

### 7. Anti-Collusion in Decentralized Evaluation
For systems like Myosu where solver strategies are submitted and evaluated, preventing collusion between submissions while maintaining fair evaluation is an open design problem.

### 8. Dynamic Population Adaptation
The opponent population in any real poker environment (or evaluation subnet) shifts over time. Developing solvers that adapt their exploitation strategy as the meta evolves is an ongoing challenge.

---

## Relevance to Myosu

### Mapping to Autoresearch Config

The autoresearch config evaluates solver architectures on a CIFAR-100 proxy benchmark across a 20-game survey. For NLHE 6-max specifically:

**Method family ranking (supported by research):**
1. **MCCFR (external sampling) with linear/discounted weighting** -- the proven approach. Pluribus used this. Hyperparameter schedules (Zhang et al., 2024) can yield orders-of-magnitude convergence improvements.
2. **Blueprint + depth-limited search** -- essential for real-time play. Without this, a static blueprint's coarse abstraction limits performance.
3. **DeepCFR / Kdb-D2CFR** -- promising for eliminating manual abstraction but higher compute cost and training instability.
4. **DREAM** -- valuable for its convergence to EFCCE and model-free property, but less mature than tabular MCCFR for poker-scale games.
5. **Actor-critic RL** -- fast training on commodity hardware but weaker theoretical grounding for imperfect-information games.
6. **NFSP** -- historically important but superseded by RM-FSP and DREAM variants.

**Hyperparameter considerations:**
- `graph_iterations` and `darts_steps` have the most direct impact on ranking quality (per known learnings). For MCCFR, the number of iterations directly determines convergence quality.
- Card abstraction bucket count critically affects blueprint quality. Too few buckets = strategic information lost. Too many = computational blowup.
- Opponent model type matters for the search phase: multiple blueprint perturbations (Pluribus-style) vs. learned opponent model (exploitative).

**Evaluation design:**
- Variance reduction via AIVAT can reduce hands needed for statistical significance by 10-44x.
- Position rotation is essential: evaluate each solver from all 6 positions against diverse opponents.
- No single exploitability metric exists; use empirical win-rate as primary metric.

### Subnet Considerations

| Factor | Recommendation |
|--------|---------------|
| Compute budget | Pluribus-class blueprint computable on commodity hardware (64 cores, 128GB RAM, 8 days). Feasible for decentralized nodes. |
| Strategy representation | Blueprint strategies for 6-max are large (~10^12 information sets abstracted) but compressible. Neural representations (DeepCFR) may be more bandwidth-efficient for submission. |
| Evaluation protocol | Round-robin tournaments across all 6 positions with AIVAT variance reduction. Minimum 10,000 hands per position pair for statistical significance. |
| Anti-collusion | Randomize seat assignments, anonymous play, mutual information analysis on submitted strategies. |
| Solution concept | Do not require Nash equilibrium (undefined/intractable). Evaluate on empirical win-rate against a diverse opponent pool. |
| Adaptation | Reward strategies that perform well against the current population, not just against fixed benchmarks. |

---

## Key Papers & References

### Foundational

| Year | Authors | Title | Venue | Key Contribution |
|------|---------|-------|-------|-----------------|
| 2007 | Zinkevich et al. | Regret Minimization in Games with Incomplete Information | NeurIPS | Original CFR algorithm |
| 2014 | Tammelin | Solving Large Imperfect Information Games Using CFR+ | arXiv | CFR+ with non-negative regrets |
| 2017 | Brown & Sandholm | Safe and Nested Subgame Solving for Imperfect-Information Games | NeurIPS | Theoretical foundations of subgame solving |
| 2017 | Brown & Sandholm | Superhuman AI for Heads-Up No-Limit Poker: Libratus | Science | First superhuman HU NLHE agent |
| 2019 | Brown & Sandholm | Superhuman AI for Multiplayer Poker | Science | Pluribus, superhuman 6-max agent |
| 2019 | Brown & Sandholm | Solving Imperfect-Information Games via Discounted Regret Minimization | AAAI | DCFR, faster convergence for large games |

### Neural and Deep Learning Approaches

| Year | Authors | Title | Venue | Key Contribution |
|------|---------|-------|-------|-----------------|
| 2016 | Heinrich & Silver | Deep Reinforcement Learning from Self-Play in Imperfect-Information Games | NeurIPS | NFSP, first end-to-end deep RL for Nash approximation |
| 2019 | Brown et al. | Deep Counterfactual Regret Minimization | ICML | DeepCFR, neural function approximation for CFR |
| 2020 | Brown et al. | Combining Deep Reinforcement Learning and Search for Imperfect-Information Games | NeurIPS | ReBeL, RL+search via public belief states |
| 2020 | Steinberger | DREAM: Deep Regret Minimization with Advantage Baselines and Model-free Learning | arXiv | Model-free deep regret minimization |
| 2022 | Zhao et al. | AlphaHoldem: High-Performance AI for Heads-Up No-Limit Poker via End-to-End RL | AAAI | Actor-critic self-play on commodity hardware |
| 2023 | Wang et al. | Kdb-D2CFR: Solving Multiplayer IIGs with Knowledge Distillation-based DeepCFR | Knowledge-Based Systems | First DeepCFR for multiplayer games |

### Search and Solving Advances

| Year | Authors | Title | Venue | Key Contribution |
|------|---------|-------|-------|-----------------|
| 2018 | Brown et al. | Depth-Limited Solving for Imperfect-Information Games | NeurIPS | Practical cost reduction for subgame solving |
| 2022 | Guo et al. | DecisionHoldem: Safe Depth-Limited Solving with Diverse Opponents | arXiv | Diverse opponent modeling in subgame solving |
| 2023 | Liu et al. | Opponent-Limited Online Search for Imperfect Information Games | ICML | Orders of magnitude faster subgame solving |
| 2023 | Schmid et al. | Student of Games | Science Advances | Unified algorithm for perfect and imperfect-info games |
| 2024 | Zhang et al. | Faster Game Solving via Hyperparameter Schedules | arXiv | Orders-of-magnitude faster CFR convergence |

### Abstraction and Evaluation

| Year | Authors | Title | Venue | Key Contribution |
|------|---------|-------|-------|-----------------|
| 2013 | Ganzfried & Sandholm | Potential-Aware Automated Abstraction | AAAI | EMD-based card abstraction |
| 2014 | Ganzfried et al. | Potential-Aware Imperfect-Recall Abstraction with EMD | AAAI | PAAEMD, state-of-the-art card abstraction |
| 2018 | Burch et al. | AIVAT: A New Variance Reduction Technique for Agent Evaluation | AAAI | 85% variance reduction in poker evaluation |

### Equilibrium Theory for Multiplayer

| Year | Authors | Title | Venue | Key Contribution |
|------|---------|-------|-------|-----------------|
| 2020 | Farina et al. | Coarse Correlation in Extensive-Form Games | AAAI | EFCCE/NFCCE theory and algorithms |
| 2022 | Farina et al. | Optimal Correlated Equilibria in General-Sum Extensive-Form Games | EC | Two-sided column generation for correlated equilibria |
| 2023 | Zhang & Sandholm | Computing Optimal Nash Equilibria in Multiplayer Games | NeurIPS | Algorithms for optimal Nash in multiplayer settings |

### Recent and Applied

| Year | Authors | Title | Venue | Key Contribution |
|------|---------|-------|-------|-----------------|
| 2024 | Marchetti et al. | A Survey on Game Theory Optimal Poker | arXiv | Comprehensive survey of GTO poker methods |
| 2025 | Yang et al. | Beyond Game Theory Optimal: Profit-Maximizing Poker Agents for No-Limit Hold'em | arXiv | Adaptive GTO + exploitative hybrid agent |
| 2025 | GTO Wizard | Introducing Quantal Response Equilibrium | Blog | QRE replaces Nash in commercial solver |
| 2025 | PokerBench team | PokerBench: Training LLMs to Become Professional Poker Players | arXiv | LLM poker benchmark and fine-tuning |
| 2025 | SpinGPT team | SpinGPT: A Large-Language-Model Approach to Playing Poker Correctly | arXiv | LLM + RL for 3-player poker |
| 2025 | Nature Scientific Reports | Comparative Analysis of Extensive Form Zero Sum Game Algorithms for Poker-like Games | Nature SR | Comprehensive algorithm comparison on poker benchmarks |
