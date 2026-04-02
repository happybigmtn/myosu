# Solving Strategies for Liar's Dice (Perudo / Dudo)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, 2005--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Liar's Dice (also known as Perudo, Dudo, or Bluff) is one of the cleanest benchmark games in imperfect-information game research. Its structure -- hidden dice values, sequential bidding with escalation constraints, and a binary challenge mechanism -- isolates the core bluffing problem without the combinatorial complexity of card games. The 2-player, 1-die variant is small enough for exact Nash equilibrium computation via linear programming; the standard 2-player, 5-die variant with wild aces generates approximately 10^8 information sets and roughly 10^26 game states (comparable to legal chess positions), placing it at a sweet spot where approximate methods are required but tractable on modest hardware.

The solving trajectory for Liar's Dice mirrors the broader field of imperfect-information game AI. Exact methods (linear programming, sequence-form LP) handle the smallest variants. Vanilla CFR solves moderate-sized instances but hits exponential blowup in the game graph due to repeated information set visits. The breakthrough for full 2-player Dudo came from Neller and Hnath (2011) with Fixed-Strategy Iteration CFR (FSICFR), which converts CFR's exponential tree recursion into polynomial-time dynamic programming, enabling approximate Nash equilibrium computation for the complete 2-player game. Monte Carlo CFR variants (external sampling, outcome sampling) reduce per-iteration cost and scale further. Facebook AI's ReBeL (Brown and Bakhtin, 2020) -- whose only open-source implementation is for Liar's Dice -- demonstrated that deep RL combined with search over public belief states converges to near-zero exploitability in small variants (1 die, 4--6 faces; 2 dice, 3 faces). Most recently, SP-PSRO (McAleer et al., ICLR 2024) showed significant exploitability reduction on Liar's Dice with near-monotonic convergence. The multi-player case (3--6 players) remains largely unsolved: Nash equilibrium computation is PPAD-hard, and no algorithm guarantees convergence to equilibrium with more than two players.

For Myosu, Liar's Dice serves as an ideal smoke-test game. It validates the core solver pipeline (CFR convergence, exploitability measurement, belief tracking) with low compute overhead and rapid iteration cycles. Any architecture that cannot competently solve 2-player Liar's Dice has fundamental issues.

---

## Game Complexity Analysis

### Information Sets by Variant Size

| Variant | Dice Config | Faces | Wild 1s | Information Sets (est.) | Game States (est.) | Tractability |
|---------|------------|-------|---------|------------------------|-------------------|--------------|
| 2P, 1 die each | 1v1 | 6 | Yes | ~200 | ~10^3 | Exact LP |
| 2P, 1 die each | 1v1 | 6 | No | ~120 | ~10^2 | Exact LP |
| 2P, 2 dice each | 2v2 | 6 | Yes | ~10^4 | ~10^6 | Tabular CFR |
| 2P, 5 dice each | 5v5 | 6 | Yes | ~10^8 | ~10^26 | FSICFR / MCCFR |
| 3P, 5 dice each | 5v5v5 | 6 | Yes | ~10^13 | ~10^39 | Approximate only |
| 6P, 5 dice each | 5^6 | 6 | Yes | >10^25 | ~10^50 | Heuristic only |

The state space explosion stems from three compounding factors:

**1. Hidden dice configurations.** Each player's roll is private. With D dice and F faces, there are F^D possible rolls per player. For 5 six-sided dice: 6^5 = 7,776 per player. For N players: 6^(5N) total configurations.

**2. Bidding sequence length.** Legal bids range from (1, 2) to (total_dice, 6), yielding up to 5*N*6 = 30N possible bid values. Each round is a variable-length sequence of escalating bids terminated by a challenge. The number of possible bid sequences is exponential in the number of players and the bid space.

**3. Multi-round elimination dynamics.** The full game spans multiple rounds as players lose dice. Each round begins with a fresh dice roll and reset bidding. The number of rounds depends on outcomes, creating a variable-depth game tree. Player asymmetry (different dice counts) further expands the state space.

### Why Wild Aces Matter for Complexity

With standard Perudo rules, 1s are wild -- they count as any face value. This doubles the probability mass: the expected count of any non-1 face value among N unknown dice shifts from N/6 to N/3. This has two effects on solving:

1. **Probability calculations change.** Challenge thresholds shift. The probability that "K dice show face X" follows a binomial with p=1/3 rather than p=1/6.
2. **Bid space has special structure.** Bidding on 1s directly invokes halving/doubling conversion rules, creating non-trivial strategic branching that increases the information set count.

---

## Historical Progression

### Phase 1: Exact Solutions via Linear Programming (pre-2009)

The earliest analytical work on Liar's Dice used classical game theory. Ferguson and Ferguson (UCLA) developed game-theoretic models for Liar's Dice variants, reducing small instances to linear programs.

**Koller-Megiddo Sequence Form (1992).** The foundational technique for solving two-player zero-sum extensive-form games reduces the problem to a linear program whose size is polynomial in the number of information sets (not game tree nodes). For 2-player, 1-die Liar's Dice, this yields an LP with a few hundred variables, solvable in milliseconds.

**Lanctot and Long, "Solving Bluff" (2005).** This early study applied LP-based equilibrium computation to small variants of Bluff (a Liar's Dice variant), demonstrating that Nash equilibria necessarily involve mixed strategies with non-trivial bluffing frequencies. They showed that converting the game to normal form produces matrices exponential in the game tree size, confirming that sequence-form LP is the correct approach for small variants, while any tabular approach becomes infeasible for games beyond 2--3 dice per player.

**Key result for 1v1, 1-die Dudo:** The Nash equilibrium is analytically known. The equilibrium strategy involves:
- Bidding truthfully with high probability when holding high dice values.
- Bluffing (overbidding) at precisely calibrated frequencies when holding low values.
- Challenging when the estimated probability of the bid being true drops below approximately 50%, adjusted for position.

### Phase 2: CFR and Its Limitations (2007--2011)

**Vanilla CFR (Zinkevich et al., 2007).** Counterfactual Regret Minimization was the first scalable algorithm for imperfect-information games. It iteratively traverses the game tree, accumulating counterfactual regret at each information set and updating strategies via regret matching. The average strategy provably converges to a Nash equilibrium in two-player zero-sum games at rate O(1/sqrt(T)).

Applied to Liar's Dice, vanilla CFR faces a critical scaling problem: the game graph has exponentially many paths visiting the same information sets due to the sequential bidding structure. Each training iteration requires a complete tree traversal, and the number of node visits grows exponentially with the depth (length of bid sequences). For the full 2-player, 5-dice Dudo game, vanilla CFR cannot complete a single iteration in reasonable time.

**Neller and Lanctot, "An Introduction to CFR" (2013).** This tutorial paper used a simplified Liar Die (single-die, single-claim variant) to demonstrate CFR and proposed 1v1 Dudo as an exercise. The paper established Liar's Dice as a standard pedagogical and benchmark game for the CFR community.

**FSICFR: The Breakthrough for Full Dudo (Neller and Hnath, 2011).** Fixed-Strategy Iteration CFR resolved the exponential blowup by decomposing each CFR iteration into two passes through a directed acyclic graph (DAG):

1. **Forward pass:** Accumulate visit counts and reach probabilities for each player while holding all strategies fixed.
2. **Backward pass:** Compute utilities and update regrets using the accumulated counts.

The key insight: by holding strategies fixed within each iteration, the exponential tree recursion collapses into a polynomial-time graph traversal (dynamic programming). FSICFR's time complexity is O(|V| * avg_outdegree) per iteration, compared to O(|visits| * avg_outdegree) for vanilla CFR, where |visits| >> |V| due to exponential path counts.

**Result:** FSICFR converges for all 2-player Dudo configurations (any combination of player/opponent dice counts) before vanilla CFR can complete a single iteration for 7+ total dice. This made computation of approximate Nash equilibria for the full 2-player game tractable for the first time.

### Phase 3: Monte Carlo CFR Variants (2009--2015)

**MCCFR (Lanctot et al., NeurIPS 2009).** Monte Carlo Counterfactual Regret Minimization replaces exhaustive game tree traversal with trajectory sampling. Three principal sampling schemes:

| Variant | What Is Sampled | Per-Iteration Cost | Variance | Convergence |
|---------|----------------|-------------------|----------|-------------|
| Chance Sampling | Chance nodes only | Medium | Low | O(1/sqrt(T)) |
| External Sampling | Chance + opponent actions | Low | Medium | O(1/sqrt(T)) |
| Outcome Sampling | Single full trajectory | Minimal | High | O(1/sqrt(T)) with importance weighting |

MCCFR variants require more iterations than vanilla CFR but dramatically lower cost per iteration. For Liar's Dice, external sampling is particularly natural: sample the dice rolls and opponent bids, then update regrets along the traversal player's decision path.

**Lanctot et al. (2009)** benchmarked MCCFR on Liar's Dice, showing that external sampling converges faster in wall-clock time than vanilla CFR for all but the smallest variants. Outcome sampling showed higher variance but the lowest per-iteration cost, making it suitable for real-time search.

**OOS: Online Outcome Sampling for Search (Lisy, Lanctot, and Bowling, AAMAS 2015).** OOS extended MCCFR to online search, computing strategies in real-time during play. OOS is the first imperfect-information search algorithm guaranteed to converge to a Nash equilibrium in two-player zero-sum games. Tested on Liar's Dice:
- At 0.1s/move, OOS statistically significantly beats both variants of Information Set MCTS (ISMCTS).
- At 1s/move, OOS performance degrades relative to ISMCTS due to ISMCTS's better exploitation of the longer time budget.
- Unlike ISMCTS, OOS's exploitability provably decreases with more search time.

### Phase 4: Deep Learning and RL+Search (2016--2024)

**Deep CFR (Brown, Lerer, Gross, Sandholm, ICML 2019).** Deep CFR replaces CFR's tabular regret storage with deep neural networks, using an advantage network to estimate counterfactual values and a strategy network to approximate the average strategy. This eliminates the need for game abstraction by generalizing across similar information sets via neural function approximation. While primarily demonstrated on poker, Deep CFR's architecture is directly applicable to Liar's Dice, particularly for variants where the information set count exceeds tabular memory limits.

**NFSP: Neural Fictitious Self-Play (Heinrich and Silver, 2016).** NFSP combines a best-response network (trained via Q-learning) with an average-strategy network (trained via supervised learning on the agent's own play). Convergence to approximate Nash equilibrium is demonstrated from self-play. NFSP has been applied to Liar's Dice variants, with recent work (Stanford CS224R, 2024) comparing NFSP and CFR on the related game Liar Bar, finding that NFSP's deep self-play framework outperforms CFR in complex multi-player scenarios and remains nearly unexploitable in 4-player settings.

**ReBeL: Combining Deep RL and Search (Brown and Bakhtin, NeurIPS 2020).** ReBeL is the most significant algorithmic advance for Liar's Dice solving. Facebook AI chose Liar's Dice as the sole open-source implementation of ReBeL specifically because the game's adjustable size makes it ideal for research.

ReBeL's architecture:
1. **Public Belief State (PBS).** At each point in the game, ReBeL constructs a probability distribution over all possible private states (dice rolls) for each player, conditioned on the public history of bids. This PBS replaces the traditional game state.
2. **Value Network.** A neural network (2 hidden layers, 256 units each) maps PBS to expected values for each player. Trained with Adam optimizer (lr=3e-4, halved every 400 epochs) on 25,600 examples per epoch with batch size 512, for 1000 total epochs.
3. **Subgame Solving.** At each decision point, ReBeL builds a depth-limited subgame rooted at the current PBS and solves it using CFR for 1024 iterations. Leaf values are estimated by the value network.
4. **Self-Play Training.** The value network is trained via self-play, with each training epoch generating data from 60 CPU threads while a single GPU trains the network.

**ReBeL exploitability results on Liar's Dice variants:**

| Variant | Dice | Faces | Exploitability (ReBeL) | Comparison |
|---------|------|-------|----------------------|------------|
| 1x4f | 1 | 4 | Near-zero | Competitive with tabular CFR |
| 1x5f | 1 | 5 | Near-zero | Competitive with tabular CFR |
| 1x6f | 1 | 6 | Near-zero | Competitive with tabular CFR |
| 2x3f | 2 | 3 | Low | Slightly higher than tabular |

ReBeL demonstrated that learned value functions can effectively replace tabular computation for imperfect-information search, with the Liar's Dice implementation serving as a clean proof of concept. Checkpoints for all four variants are publicly available in the facebookresearch/rebel repository.

**Solly / DeepNash for Liar's Poker (2025).** While technically a different game, the closely related "Mastering Liar's Poker via Self-Play and Reinforcement Learning" (arXiv:2511.03724) demonstrated that R-NaD (Regularized Nash Dynamics, the algorithm behind DeepNash for Stratego) can master bluffing games at elite human level. Solly is the first AI agent to achieve elite performance in Liar's Poker, winning >50% of hands against experienced human players. The methodology -- model-free actor-critic deep RL with self-play -- is directly transferable to Liar's Dice.

**SP-PSRO (McAleer et al., ICLR 2024).** Self-Play Policy Space Response Oracle ensures exploitability does not increase between iterations (unlike vanilla PSRO). On Liar's Dice:
- SP-PSRO achieves significantly lower exploitability than PSRO and APSRO in early iterations.
- Near-monotonic exploitability reduction.
- Often converges in just a few iterations, compared to many more for PSRO.

---

## Current Best Approaches

### Tier 1: Exact Methods (Small Variants Only)

**Sequence-Form Linear Programming.** For 2-player, 1-die Liar's Dice (with or without wild aces), the game has few enough information sets (~100--200) that the Nash equilibrium can be computed exactly via LP in milliseconds. This provides ground-truth benchmarks for all approximate methods.

- **Strengths:** Exact Nash equilibrium, zero exploitability, fast computation.
- **Weaknesses:** Does not scale. LP size is polynomial in information sets, but information sets grow exponentially with dice count.
- **Compute:** Seconds for 1v1; hours for 2v2; infeasible for 3+ dice per player.

### Tier 2: FSICFR (Full 2-Player Dudo)

**Fixed-Strategy Iteration CFR** remains the best approach for computing approximate Nash equilibria for the complete 2-player game (any dice count asymmetry: 1v5, 2v4, 3v3, 5v5, etc.).

- **Strengths:** Handles the exponential path problem that blocks vanilla CFR. Polynomial per-iteration cost. Provable convergence. No neural networks or GPUs needed.
- **Weaknesses:** Memory scales with the DAG size (all information sets must be stored). Does not extend naturally to 3+ players. No function approximation means no generalization.
- **Compute:** Full 2-player, 5-dice Dudo converges in hours to days on a single CPU.

### Tier 3: MCCFR Variants (Flexible, Scalable)

**External Sampling MCCFR** is the workhorse for medium-scale Liar's Dice solving. It samples chance outcomes (dice rolls) and opponent actions, updating only the traversal player's regrets.

- **Strengths:** Low per-iteration cost. Scales to larger variants than tabular CFR. Can be parallelized. Natural fit for Liar's Dice where chance sampling (dice rolls) dramatically reduces per-iteration work.
- **Weaknesses:** Higher variance than full-width traversal. Requires more iterations for the same exploitability. Convergence rate still O(1/sqrt(T)).
- **Compute:** Minutes for small variants; hours for 5v5. GPU not required.

**CFR+ and DCFR.** CFR+ (Tammelin, 2014) and Discounted CFR (Brown and Sandholm, 2019) accelerate convergence by non-uniform weighting of iterations. CFR+ uses regret-matching+ with linear averaging; DCFR discounts early iterations more aggressively. DCFR and Predictive CFR+ (PCFR+) are the fastest known algorithms for two-player zero-sum games. Recent work on hyperparameter schedules (2024) achieves additional orders-of-magnitude speedups on top of DCFR/PCFR+.

### Tier 4: Deep RL + Search (ReBeL and Beyond)

**ReBeL** is the state of the art for Liar's Dice variants where tabular methods are insufficient but exact exploitability computation is still feasible (for validation). Its combination of learned value functions and real-time subgame solving generalizes across game sizes without retraining.

- **Strengths:** Learns transferable value functions. Real-time search at play time. Provable convergence in 2-player zero-sum setting. Clean open-source implementation.
- **Weaknesses:** Requires GPU for training. Subgame solving adds latency at inference time. Exploitability higher than tabular CFR for the same game size (function approximation error). Not demonstrated on 3+ player Liar's Dice.
- **Compute:** Training: single GPU + 60 CPU threads for ~1000 epochs. Inference: 1024 CFR iterations per decision.

**Student of Games (SoG, Schmid et al., Science Advances 2023).** A unified algorithm for perfect and imperfect information games, using Growing-Tree CFR (GT-CFR) for local search and sound self-play for network training. While not specifically benchmarked on Liar's Dice, SoG's general-purpose architecture (tested on poker, chess, Go, Scotland Yard) is directly applicable and represents the frontier of general game-solving algorithms.

---

## Bluffing Theory

### Fundamental Bluffing Principles

Liar's Dice is a pure bluffing game -- equilibrium strategies must involve mixed strategies with non-trivial bluffing and calling frequencies. A player who never bluffs is exploitable (opponents simply challenge any bid not supported by prior information). A player who never challenges is exploitable (opponents bid arbitrarily high).

### Optimal Bluff Frequencies

In equilibrium, bluffing frequencies are calibrated to make the opponent indifferent between challenging and raising. Key results from solved small variants:

**1. Challenge Threshold.** A bid should be challenged when the estimated probability of it being true drops below approximately 50%. This threshold arises from the symmetric payoff structure: both the bidder (if caught) and the challenger (if wrong) lose exactly one die.

**2. N/3 Rule with Wild Aces.** When 1s are wild, the expected count of any non-1 face value among N unknown dice is N/3 (probability 1/3 per die: 1/6 for the face value + 1/6 for wild 1s). Bids near N/3 are "safe" (>50% likely to be true); bids significantly above N/3 are either well-informed or bluffs.

**3. Position-Dependent Bluffing.** Players earlier in the bidding sequence bluff at higher rates because they face less information from prior bids. Players immediately after an aggressive bid face a forced-challenge or forced-raise decision, reducing their strategic flexibility.

**4. Bluffing as Information Manipulation.** Overbidding a face value you do not hold can serve dual purposes: it may win the round if unchallenged, and it corrupts opponents' Bayesian inference about your dice values.

### Bayesian Inference and Belief Updating

Sophisticated play involves updating beliefs about opponent dice based on their bidding patterns:

- **Bid content inference.** If a player bids "four 5s," they likely hold some 5s and/or 1s. The frequency and confidence of bids on a particular face value are signals.
- **Bid sequence inference.** A player who consistently raises on a face value (rather than switching) likely has strong holdings in that face.
- **Challenge aversion inference.** A player who fails to challenge a suspect bid may hold dice that support it, or may be bluffing support for a later raise.

In equilibrium, these inferences are precisely balanced: bluffing frequencies are set so that opponents cannot profitably deviate by over-weighting or under-weighting bid signals. This is what makes the Nash equilibrium mixed.

---

## Multi-Player Scaling

### Fundamental Challenges

Multi-player Liar's Dice (3--6 players) introduces qualitatively different strategic dynamics:

**1. Equilibrium non-uniqueness.** In two-player zero-sum games, all Nash equilibria yield the same payoff and CFR converges to one. In multi-player games, Nash equilibria may have different payoffs, multiple equilibria may coexist, and no efficient algorithm is known that converges to any Nash equilibrium. Computing a Nash equilibrium for three or more players is PPAD-hard.

**2. Challenge dynamics.** With 3+ players, the "let someone else challenge" problem arises. If Player B makes a dubious bid, Player C (next in turn) faces a dilemma: challenge and risk losing a die, or raise and pass the risk to Player D. In equilibrium, challenge probabilities must be mixed across all eligible challengers.

**3. Elimination and ICM effects.** When one player has few dice remaining, other players may tacitly cooperate to eliminate them (by not challenging each other's dubious bids while keeping pressure on the short-stacked player). This creates implicit coalition dynamics that no existing algorithm handles cleanly.

**4. State space explosion.** Adding a third player with 5 dice multiplies the hidden state space by 6^5 = 7,776. The bid sequence space grows multiplicatively with each additional player. The full 6-player game has >10^50 game tree nodes.

### Current Multi-Player Approaches

**NFSP with deep self-play.** Recent work (Stanford CS224R, 2024) on the related game Liar Bar showed that NFSP with a history-aware neural architecture maintains near-zero exploitability in 4-player settings. The dual-network architecture (best-response + average-strategy) scales more gracefully than tabular CFR to multi-player games.

**R-NaD / DeepNash-style training.** The Liar's Poker work (arXiv:2511.03724, 2025) demonstrated that R-NaD (model-free actor-critic with regularized Nash dynamics) can produce strong multi-player bluffing play. Solly achieved >50% win rate and positive equity against elite humans in multi-player Liar's Poker. This approach does not guarantee Nash equilibrium convergence but produces empirically strong policies.

**Population-based methods.** PSRO variants (SP-PSRO, Fusion-PSRO, Conflux-PSRO) maintain a population of policies and compute best responses against population mixtures. These can handle multi-player games but require training many independent policies and solving increasingly large meta-games.

### Practical Implications

For Myosu, multi-player Liar's Dice should be treated as a benchmark for approximate methods rather than exact solving. Win rate against a field of opponents (including CFR-trained, heuristic, and random agents) is a more meaningful metric than exploitability, since exploitability is not well-defined for >2 player games.

---

## Open Problems

### 1. Exact Exploitability for Larger Variants

Exploitability can be computed in time linear in the number of game states, but this is infeasible for games with 4+ dice per player (~10^26 states). No efficient exploitability approximation method exists that provides tight bounds for medium-scale Liar's Dice.

### 2. Multi-Player Equilibrium Convergence

No algorithm is known to converge to a Nash equilibrium in multi-player Liar's Dice. Correlated equilibria, team-maxmin strategies, and other solution concepts may be more appropriate but are less studied for this game.

### 3. Optimal Abstraction for Dice Games

Unlike poker, where hand strength bucketing provides natural card abstraction, Liar's Dice lacks a standard abstraction hierarchy. Neller's "memory of m previous claims" abstraction is effective but ad hoc. Principled, lossless, or near-lossless abstraction for dice games remains unexplored.

### 4. Palafico and Special-Rule Subgames

The Palafico round (no wilds, locked face value, triggered when a player has 1 die) creates a constrained subgame with different equilibrium properties. No published work specifically analyzes Palafico equilibria or how Palafico strategy interacts with main-game strategy.

### 5. Real-Time Belief Tracking at Scale

As the game progresses and players lose dice, the belief state (probability distribution over opponent dice) must be updated based on observed bids and challenge outcomes. Efficient belief tracking for 3+ players with 5 dice each is computationally expensive and may require neural approximation.

### 6. Transfer Across Game Sizes

Can a value network trained on 2-dice Liar's Dice transfer to 5-dice Liar's Dice? ReBeL's architecture is size-specific (tied to the number of faces and dice). Learning representations that generalize across game sizes is an open research direction.

### 7. Human-Like Bluffing

Equilibrium strategies produce optimal bluffing frequencies against worst-case opponents, but human opponents deviate from equilibrium. Exploitative strategies that adapt to human bluffing patterns (over-bluffing, under-challenging, tell-based inference) could significantly outperform Nash strategies against humans, as demonstrated by Solly in Liar's Poker.

---

## Relevance to Myosu

### Smoke-Test Role

Liar's Dice is one of three smoke-test games in the autoresearch configuration (alongside NLHE HU and Backgammon). It validates:

1. **CFR convergence.** Any CFR implementation should converge on 2-player, 1-die Liar's Dice within seconds and match the known exact equilibrium.
2. **Exploitability measurement.** The small variants have computable exploitability, providing ground-truth validation of the evaluation pipeline.
3. **Bluffing mechanics.** Liar's Dice isolates bluffing from card/hand complexity. If the solver cannot learn to bluff in Liar's Dice, it will fail at poker.
4. **Rapid iteration.** Games complete in seconds. Solver training completes in minutes to hours. The fast feedback loop accelerates architecture iteration.

### Architecture Validation

| Solver Component | Liar's Dice Test |
|-----------------|-----------------|
| Tabular CFR | Exact match on 1-die variant |
| MCCFR (external sampling) | Convergence on 5-dice variant |
| Neural value network | ReBeL-style training and evaluation |
| Exploitability computation | Ground truth on small variants |
| Belief state tracking | PBS construction and updating |
| Multi-player handling | 3--4 player win rate measurement |

### Recommended Evaluation Protocol

1. **Correctness gate:** 2-player, 1-die, no wilds. Compare solver output against exact LP solution. Exploitability must be < 0.001.
2. **Scale gate:** 2-player, 5-dice, wild aces. Run MCCFR for 10^6 iterations. Measure exploitability trend (should be monotonically decreasing).
3. **Quality gate:** ReBeL-style training on 1-die, 6-face variant. Compare exploitability against published ReBeL checkpoints.
4. **Generality gate:** 3-player, 3-dice variant. Measure win rate against uniform random and simple heuristic opponents. Solver should achieve >60% win rate against random.

---

## Key Papers and References

### Foundational

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 1992 | Koller, Megiddo | "The Complexity of Two-Person Zero-Sum Games in Extensive Form" | Sequence-form LP for solving extensive-form games; polynomial in information sets |
| 2005 | Lanctot, Long | "Solving Bluff" | LP-based equilibrium computation for small Liar's Dice; demonstrated necessity of mixed strategies |
| 2007 | Zinkevich et al. | "Regret Minimization in Games with Incomplete Information" | Vanilla CFR; foundational algorithm for imperfect-information game solving |

### CFR Variants

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 2009 | Lanctot et al. | "Monte Carlo Sampling for Regret Minimization in Extensive Games" (NeurIPS) | MCCFR family; external and outcome sampling; benchmarked on Liar's Dice |
| 2011 | Neller, Hnath | "Approximating Optimal Dudo Play with Fixed-Strategy Iteration CFR" | FSICFR; first tractable computation of approximate Nash for full 2-player Dudo |
| 2013 | Neller, Lanctot | "An Introduction to Counterfactual Regret Minimization" | Tutorial paper; established Liar's Dice (Dudo) as standard CFR exercise |
| 2014 | Tammelin | "Solving Large Imperfect Information Games Using CFR+" | CFR+ with regret-matching+; order-of-magnitude speedup over vanilla CFR |
| 2019 | Brown, Sandholm | "Solving Imperfect-Information Games via Discounted Regret Minimization" (AAAI) | DCFR; fastest convergence for two-player zero-sum games |
| 2024 | Farina et al. | "Faster Game Solving via Hyperparameter Schedules" | Further speedups on DCFR/PCFR+ via adaptive weighting |

### Deep Learning and RL+Search

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 2016 | Heinrich, Silver | "Deep Reinforcement Learning from Self-Play in Imperfect-Information Games" | NFSP; first deep RL approach for approximate Nash in imperfect-information games |
| 2019 | Brown, Lerer, Gross, Sandholm | "Deep Counterfactual Regret Minimization" (ICML) | Deep CFR; neural function approximation eliminates need for game abstraction |
| 2020 | Brown, Bakhtin et al. | "Combining Deep Reinforcement Learning and Search for Imperfect-Information Games" (NeurIPS) | ReBeL; open-source Liar's Dice implementation; PBS + value network + subgame solving |
| 2023 | Schmid et al. | "Student of Games" (Science Advances) | Unified algorithm for perfect and imperfect information games; GT-CFR + sound self-play |
| 2024 | McAleer et al. | "Toward Optimal Policy Population Growth in Two-Player Zero-Sum Games" (ICLR) | SP-PSRO; near-monotonic exploitability reduction on Liar's Dice |
| 2025 | Various | "Outbidding and Outbluffing Elite Humans: Mastering Liar's Poker via Self-Play and RL" (arXiv) | R-NaD for bluffing games; elite human-level play; directly transferable methodology |

### Search-Based

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 2012 | Cowling, Powley, Whitehouse | "Information Set Monte Carlo Tree Search" | ISMCTS; MCTS extension for imperfect-information games |
| 2015 | Lisy, Lanctot, Bowling | "Online Monte Carlo Counterfactual Regret Minimization for Search in Imperfect Information Games" (AAMAS) | OOS; first search algorithm with Nash convergence guarantee; benchmarked on Liar's Dice |

### Open-Source Implementations

| Repository | URL | Description |
|-----------|-----|-------------|
| facebookresearch/rebel | github.com/facebookresearch/rebel | ReBeL for Liar's Dice (Python + C++); value network checkpoints for 4 variants |
| google-deepmind/open_spiel | github.com/google-deepmind/open_spiel | Liar's Dice environment; CFR, MCCFR, and other algorithm implementations |
| thomasahle/snyd | github.com/thomasahle/snyd | Liar's Dice endgame analysis; CFR with neural value network (PyTorch) |
| erikbrinkman/cfr | github.com/erikbrinkman/cfr | Rust CFR library applicable to Liar's Dice |
| Neller FSICFR | modelai.gettysburg.edu/2013/cfr/ | Reference FSICFR implementation for Dudo |
