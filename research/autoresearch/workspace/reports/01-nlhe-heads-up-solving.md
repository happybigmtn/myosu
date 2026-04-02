# Solving Strategies for No-Limit Hold'em Heads-Up (NLHE HU)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, 2007--2026
**Status:** Research report (no implementation)

---

## Executive Summary

No-Limit Hold'em Heads-Up (NLHE HU) is the canonical benchmark for imperfect-information game solving. The game's enormous state space (~10^161 game tree nodes, ~3.19 x 10^14 information sets without abstraction) places it well beyond exact solution, but a series of algorithmic breakthroughs from 2015 to the present have produced superhuman-level approximate Nash equilibrium strategies.

The dominant paradigm remains a two-phase approach: (1) offline computation of a coarse "blueprint" strategy via Counterfactual Regret Minimization (CFR) variants operating on an abstracted game, followed by (2) real-time subgame solving that refines the blueprint at decision time using depth-limited lookahead with neural network leaf evaluation. This paradigm was established by Libratus (2017) and DeepStack (2017), extended to multiplayer settings by Pluribus (2019), and generalized by ReBeL (2020) and Student of Games (2023). Recent work (2024--2026) has focused on accelerating CFR convergence (DDCFR, Hyperparameter Schedules, PDCFR+), improving neural integration (Deep DCFR+, Robust Deep MCCFR), exploring alternatives to Nash equilibrium (QRE, Preference-CFR), and questioning whether simpler policy gradient methods can match CFR-based approaches entirely (ICLR 2025).

The field is mature for two-player zero-sum settings but open problems remain: true exploitability is intractable to measure for the full game, multiplayer Nash equilibrium has questionable strategic value, opponent exploitation remains largely heuristic, and the gap between abstracted and full-game performance (abstraction pathologies) is still not fully understood.

---

## Game Complexity Analysis

### Information Set Structure

| Metric | Value | Source |
|--------|-------|--------|
| Raw game tree nodes | ~10^161 -- 10^165 | Johanson 2013 |
| Information sets (no abstraction) | ~3.19 x 10^14 | Johanson 2013 |
| Information sets (Libratus-level abstraction) | ~10^12 | Brown & Sandholm 2017 |
| Strategically distinct preflop hands | 169 (1,326 raw combos) | Standard |
| Distinct hand+board combos (flop) | ~2.6 million | Standard |
| Effective branching factor (after abstraction) | 2--5 actions per node | Typical |

### Why NLHE HU Is Hard

1. **Continuous action space.** Unlike limit hold'em where bet sizes are fixed, NLHE allows any integer chip amount up to a player's stack, making the raw game tree effectively infinite without discretization.

2. **Imperfect information.** Hidden hole cards create information asymmetry. Optimal strategy in a subgame depends on play in unreached parts of the game tree (unlike perfect-information games where subgames can be solved in isolation).

3. **Sequential decision-making under uncertainty.** Four betting rounds with interleaved chance events (card deals) create deep decision trees where early mistakes compound.

4. **Bluffing and deception.** Nash equilibrium strategies inherently involve mixed strategies where the same hand is played differently at different frequencies, requiring randomized action selection.

5. **Abstraction pathologies.** Coarsening the game for tractability can introduce errors that actually increase exploitability when the abstract strategy is mapped back to the full game.

### Comparison to Other Solved/Near-Solved Games

| Game | State Space | Information | Status |
|------|-------------|-------------|--------|
| Checkers | ~5 x 10^20 | Perfect | Solved (2007) |
| Chess | ~10^47 | Perfect | Superhuman AI |
| Go | ~10^170 | Perfect | Superhuman AI |
| Limit Hold'em HU | ~10^14 info sets | Imperfect | Essentially solved (2015) |
| **NLHE HU** | **~10^161 nodes** | **Imperfect** | **Approximate Nash, superhuman** |
| 6-max NLHE | Orders of magnitude larger | Imperfect | Superhuman (Pluribus, 2019) |

---

## Historical Progression

### 2007: CFR Is Born
Zinkevich, Johanson, Bowling, and Piccione introduce Counterfactual Regret Minimization (CFR), the foundational algorithm for solving extensive-form imperfect-information games. CFR decomposes the full-game regret into per-information-set counterfactual regrets that can be minimized independently, converging to a Nash equilibrium in two-player zero-sum games.

### 2014: CFR+ Accelerates Convergence
Tammelin introduces CFR+, which clips negative cumulative regrets to zero at each iteration. This simple modification dramatically accelerates convergence and eliminates the need for the regret-matching averaging step -- the current strategy at each iteration is already a good approximation.

### 2015: Cepheus Solves Limit Hold'em HU
Bowling, Burch, Johanson, and Tammelin (University of Alberta) essentially solve heads-up limit Texas hold'em using CFR+ with fixed-point compression. The strategy has exploitability < 0.986 mbb/hand. The computation required 68 days on 200 compute nodes, processing 4.531 x 10^13 information sets. Compression reduced memory from raw terabytes to ~11 TB for regrets and ~6 TB for the strategy, using ordered fixed-point arithmetic at ~13:1 compression ratios. Published in *Science*.

### 2017: DeepStack -- Neural-Guided Continual Re-solving
Moravcik et al. (University of Alberta / Czech Technical University) introduce DeepStack, which avoids precomputing a full-game strategy entirely. Instead, it treats every decision point as the root of a new subgame and solves it in real time using depth-limited lookahead. Beyond the depth limit, counterfactual values are estimated by neural networks trained on millions of randomly generated poker subgames before play begins. Architecture: 7 fully connected hidden layers, 500 nodes each, parametric ReLU activations. DeepStack defeated professional players in a statistically significant 44,852-hand study. Published in *Science*.

### 2017: Libratus -- Blueprint + Nested Subgame Solving
Brown and Sandholm (Carnegie Mellon University) build Libratus, a three-module system that defeats top human professionals in a 120,000-hand match. The three modules:
1. **Blueprint strategy:** Precomputed via Monte Carlo CFR (MCCFR) on an abstracted game tree.
2. **Nested subgame solving:** Real-time refinement at each decision point using safe subgame solving, which provably does not increase exploitability compared to the blueprint.
3. **Self-improvement:** Overnight patching of blueprint leaks detected during play by adding missing branches to the abstraction where opponents found off-tree actions.

Published in *Science* (2018). This architecture establishes the dominant paradigm that persists today.

### 2019: Pluribus -- Multiplayer Poker
Brown and Sandholm extend the approach to 6-player no-limit hold'em. Key innovations:
- Blueprint computed via linear MCCFR in 8 days on a 64-core server (12,400 CPU core hours, <512 GB memory).
- Depth-limited real-time search limited to ~2--3 moves ahead with ~6 action sizes.
- At leaf nodes, each player may choose between k different continuation strategies (not a single fixed strategy), reducing the exploitability of the depth-limited approach.
- Pluribus defeated elite professionals across 10,000+ hands. Published in *Science*.

### 2020: ReBeL -- RL + Search for Imperfect Information
Brown et al. (Facebook AI Research) introduce ReBeL (Regularized policy from Belief-space Learning), which unifies reinforcement learning and search for imperfect-information games. ReBeL operates on a "public belief state" (PBS) -- a probability distribution over all players' possible hidden information -- transforming the imperfect-information game into a higher-dimensional but perfect-information MDP. This allows standard RL+search techniques (similar to AlphaZero) to be applied. ReBeL achieves superhuman NLHE HU performance with far less domain-specific engineering than Libratus/Pluribus.

### 2022: AlphaHoldem -- End-to-End RL
Zhao et al. introduce AlphaHoldem, a lightweight NLHE HU AI trained entirely via end-to-end self-play reinforcement learning using a pseudo-siamese network architecture. It defeats Slumbot and DeepStack by 111.56 and 16.91 mbb/h respectively, using only a single GPU with 2.9ms per decision (1,000x faster than DeepStack). Training completes in 3 days on one PC. Published at AAAI 2022.

### 2023: Student of Games -- Unified Algorithm
Schmid et al. (EquiLibre Technologies, DeepMind, Sony AI) publish Student of Games, a general-purpose algorithm combining guided search, self-play learning, and game-theoretic reasoning. It performs strongly in both perfect-information games (chess, Go) and imperfect-information games (poker, Scotland Yard). Its search is sound: guaranteed to find an approximate Nash equilibrium via subgame re-solving. Published in *Science Advances*.

### 2024--2026: The Optimization Era
Focus shifts from architectural breakthroughs to convergence acceleration, neural integration, and alternative equilibrium concepts. Key developments detailed in sections below.

---

## Current Best Approaches

### Family 1: CFR Variants (Tabular)

The CFR family remains the gold standard for computing Nash equilibria in two-player zero-sum extensive-form games.

#### Vanilla CFR (2007)
- **Algorithm:** Iteratively traverses the game tree, computing counterfactual regrets for each action at each information set. Strategy is proportional to positive cumulative regret (regret matching). Average strategy converges to Nash equilibrium at O(1/sqrt(T)).
- **Strengths:** Simple, provably convergent, embarrassingly parallelizable across information sets.
- **Weaknesses:** Slow convergence. Requires full game tree traversal per iteration (prohibitive for NLHE without sampling).
- **Compute:** Linear in game size per iteration. Memory: O(|information sets| x |actions|).

#### CFR+ (2014)
- **Algorithm:** Same as CFR but clips negative cumulative regrets to zero and uses the current strategy (not the average) as the output.
- **Strengths:** Dramatically faster convergence than vanilla CFR. Used to solve limit hold'em.
- **Weaknesses:** Still requires full traversal without sampling.
- **Compute:** Same per-iteration cost as CFR, but needs far fewer iterations.

#### Monte Carlo CFR / MCCFR (2009)
- **Algorithm:** Samples a subset of the game tree per iteration rather than traversing it fully. Variants: external sampling (samples opponent actions and chance), outcome sampling (samples a single trajectory), chance sampling (samples only chance nodes).
- **Strengths:** Each iteration is much cheaper. Scales to games too large for full traversal. Used by Libratus and Pluribus for blueprint computation.
- **Weaknesses:** Higher variance per iteration. Convergence guarantees weaker than full CFR for some sampling schemes.
- **Compute:** Per-iteration cost proportional to sampled subtree size. External sampling is the most common in practice.

#### Discounted CFR / DCFR (2019)
- **Algorithm:** Weights earlier iterations less than later ones using a fixed discount schedule. Specifically, iteration t's contribution is weighted by t^alpha for positive regrets, t^beta for negative regrets, and t^gamma for the average strategy.
- **Strengths:** Faster convergence than CFR+ on poker games specifically. The discounting helps escape early bad decisions.
- **Weaknesses:** Fixed discount schedule may not be optimal for all game structures. PCFR+ can outperform DCFR on non-poker games.
- **Compute:** Same per-iteration cost as CFR, with trivial overhead for discount computation.

#### Predictive CFR+ / PCFR+ (2019)
- **Algorithm:** Leverages the predictability of counterfactual regrets across iterations, using an optimistic online mirror descent step to accelerate convergence.
- **Strengths:** Faster than DCFR on non-poker games. Stronger theoretical convergence guarantees.
- **Weaknesses:** Can be slower than DCFR specifically on poker games.
- **Compute:** Slight overhead for prediction computation.

#### PDCFR+ (2024, IJCAI Oral)
- **Algorithm:** Integrates PCFR+ and DCFR in a principled manner. Uses discounted weights for regret computation (mitigating dominated actions quickly) while simultaneously leveraging predictions for acceleration.
- **Strengths:** Combines the poker-specific advantages of DCFR with the general advantages of PCFR+.
- **Weaknesses:** Requires selecting discounting and prediction hyperparameters.
- **Compute:** Marginal overhead over DCFR or PCFR+ alone.

#### Dynamic Discounted CFR / DDCFR (2024, ICLR Spotlight)
- **Algorithm:** Replaces fixed discount schedules with a learned dynamic scheme. Formulates the iteration process as an MDP and uses policy optimization to learn the optimal discounting scheme.
- **Strengths:** Automatically adapts discounting to the specific game being solved. Outperforms PCFR+ on poker games. Generalizes well across game types.
- **Weaknesses:** Requires an initial training phase to learn the discounting policy. More complex implementation.
- **Compute:** Training the MDP policy adds upfront cost, but per-iteration cost remains similar.

#### Hyperparameter Schedules / HS (2024)
- **Algorithm:** Dynamically adjusts the hyperparameters governing DCFR or PCFR+ discount schemes during solving, rather than using fixed values. Proposed by Zhang, McAleer, and Sandholm.
- **Strengths:** Yields "orders-of-magnitude speed improvements" over static DCFR/PCFR+. Easy to implement (small code modifications). No game-specific tuning required. **Current state of the art for tabular game solving.**
- **Weaknesses:** Limited published evaluation on the largest game instances.
- **Compute:** Negligible overhead.

### Family 2: Deep Learning + CFR Hybrids

#### Deep CFR (2019)
- **Algorithm:** Replaces the tabular regret/strategy storage of CFR with neural networks. A value network approximates action advantages, and a policy network learns the average strategy. The value network is retrained from scratch each CFR iteration using sampled game tree traversals stored in a replay buffer.
- **Strengths:** Eliminates the need for hand-crafted abstractions -- operates on the full game. Scales to games where tabular CFR is memory-prohibitive.
- **Weaknesses:** Each iteration requires expensive neural network training (thousands of SGD steps). No convergence guarantees as strong as tabular CFR. Quality depends on network architecture and training hyperparameters.
- **Compute:** GPU-intensive. Typical setup: 4,000 SGD iterations per CFR step with batch size 10,000 (HUNL uses 32,000 SGD iterations, batch size 20,000).

#### Deep DCFR+ and Deep PDCFR+ (2025)
- **Algorithm:** Integrates the discounting and clipping mechanisms of DCFR+ and PDCFR+ into the neural CFR framework. DCFR+ applies discounting and clipping to cumulative advantages. PDCFR+ adds a prediction network that estimates advantages at each iteration.
- **Strengths:** Faster convergence than vanilla Deep CFR across eight benchmark games. Higher average rewards against rule-based agents in large poker games. DCFR+ is the fastest in the HUNL subgame benchmark.
- **Weaknesses:** Increased model complexity (additional prediction network for PDCFR+).
- **Compute:** Similar GPU requirements to Deep CFR with additional network overhead.

#### Supremus (2020)
- **Algorithm:** Combines improvements to deep counterfactual value networks with DCFR+, running fully on GPU. Extends DeepStack's approach with better value network training and faster iteration.
- **Strengths:** Beats Slumbot by 176 +/- 44 mbb/g (vs. DeepStack losing 63 mbb/g). 1,000 DCFR+ iterations in 0.8 seconds (6x faster than DeepStack).
- **Weaknesses:** Still requires careful value network architecture design.
- **Compute:** Single GPU, real-time inference.

#### Robust Deep MCCFR (2025)
- **Algorithm:** Systematic analysis of scale-dependent failure modes in neural MCCFR: nonstationary target distributions, action support collapse, variance explosion, and warm-starting bias. Proposes target networks with delayed updates, uniform exploration mixing, variance-aware training, and diagnostic monitoring.
- **Strengths:** First principled framework for understanding when neural CFR components help vs. hurt. Provides practical guidance: exploration mixing hurts in small games but helps in large ones.
- **Weaknesses:** Adds implementation complexity. No single configuration works across all scales.
- **Compute:** Similar to Deep CFR with monitoring overhead.

### Family 3: RL + Search

#### DeepStack (2017)
- **Algorithm:** Continual re-solving with neural value estimation. Every decision is the root of a new subgame solved in real time with depth-limited CFR. Value networks (trained offline on random subgames) estimate counterfactual values at the depth limit.
- **Strengths:** No precomputed blueprint needed. Adapts naturally to any game state. Sound re-solving guarantees.
- **Weaknesses:** Slow inference (~3 seconds per decision). Quality bounded by value network accuracy.
- **Compute:** Moderate GPU for inference. Training: millions of random subgame solves.

#### ReBeL (2020)
- **Algorithm:** Transforms imperfect-information games into a perfect-information "public belief state" MDP. Applies self-play RL with search (similar to AlphaZero) on this PBS space. The value and policy networks operate on belief distributions rather than concrete states.
- **Strengths:** General framework requiring minimal domain-specific engineering. Achieves superhuman NLHE HU. Extends naturally to other imperfect-information games.
- **Weaknesses:** PBS representation is high-dimensional for large games. Requires careful belief tracking.
- **Compute:** Significant training cost (self-play RL), but inference is fast.

#### Student of Games (2023)
- **Algorithm:** Combines guided search (Growing-Tree CFR for imperfect information, MCTS for perfect information), self-play learning, and game-theoretic reasoning. Uses subgame re-solving for sound play across game types.
- **Strengths:** Single algorithm handles both perfect and imperfect information games. Beats the strongest openly available NLHE HU agent and state-of-the-art Scotland Yard agent.
- **Weaknesses:** Performance on NLHE HU doesn't match specialized approaches like Libratus.
- **Compute:** Moderate. Not characterized for extreme scale.

#### AlphaHoldem (2022)
- **Algorithm:** End-to-end self-play RL with a pseudo-siamese architecture. Novel state representation for cards and betting. Multi-task self-play training loss. No CFR, no abstraction, no subgame solving.
- **Strengths:** Extremely fast inference (2.9ms/decision on single GPU). Trains in 3 days on one PC. Beats DeepStack and Slumbot.
- **Weaknesses:** No convergence-to-Nash guarantees. Exploitability unclear. Performance may degrade against specifically adversarial opponents.
- **Compute:** Minimal compared to CFR-based systems. Single GPU training and inference.

### Family 4: Neural Fictitious Self-Play (NFSP)

#### NFSP (2016)
- **Algorithm:** Combines fictitious self-play with deep RL. Two networks: Q-network (action values from RL data) and policy network (average strategy via supervised learning). Two corresponding replay buffers.
- **Strengths:** End-to-end, no domain-specific engineering. Approximates Nash equilibrium from self-play.
- **Weaknesses:** Convergence slower than CFR-based approaches. Divergence risk in complex games.
- **Compute:** Standard deep RL requirements.

#### RM-FSP (2023)
- **Algorithm:** Replaces NFSP's best-response computation with regret minimization. Outperforms NFSP in both exploitability and time efficiency.

### Family 5: Policy Gradient Methods (Emerging, 2024--2025)

#### Magnetic Mirror Descent / MMD (2023)
- **Algorithm:** A policy gradient method that maximizes expected value, regularizes the policy, and controls update magnitude. Demonstrated competitive with CFR in tabular settings.
- **Strengths:** Compatible with standard deep learning tooling. Simple implementation.
- **Weaknesses:** Less established than CFR family. Limited large-scale evaluation.

#### PPO and Generic PG for IIGs (2025, ICLR)
- **Finding:** Over 7,000 training runs, generic policy gradient methods like PPO are competitive with or superior to FP-, DO-, and CFR-based deep RL approaches for imperfect-information games. This challenges the assumption that CFR-specific algorithms are strictly necessary for the neural setting.
- **Significance:** If confirmed at scale, this could simplify the implementation stack significantly.

---

## Abstraction Techniques

Abstraction is the process of reducing the game's complexity to a tractable size while preserving strategic structure. Two dimensions: card (information) abstraction and action (bet) abstraction.

### Card Abstraction

#### Hand Isomorphism (Lossless)
Suit permutations that do not change strategic equivalence are collapsed. Waugh (2013) showed fast optimal hand indexing that accounts for suit isomorphisms. This is lossless -- no information is discarded. Reduces preflop hands from 1,326 to 169 strategically distinct classes.

#### Expected Hand Strength (EHS)
Simplest lossy abstraction: group hands by their expected equity against a uniform random opposing range. Fast to compute but discards distributional information -- two hands with the same average equity but very different equity distributions (e.g., a strong made hand vs. a draw) are treated identically.

#### Earth Mover's Distance (EMD) Clustering
Groups hands by the similarity of their full equity distributions using k-means clustering with Earth Mover's Distance (the minimum "work" to transform one distribution into another). Preserves much more strategic information than EHS. The leading abstraction algorithm for imperfect-information games (Ganzfried & Sandholm, AAAI 2014).

#### Potential-Aware Abstractions
Account for how hand strength changes on future streets, not just current equity. Potential-Aware EMD (PAEMD) combines EMD clustering with forward-looking evaluation. Recent work (2024) establishes "potential-aware outcome isomorphism" as the theoretical resolution bound for potential-aware abstraction algorithms.

#### Typical Bucket Counts
- Preflop: 8--169 buckets (often ~50--100)
- Flop: 200--5,000 buckets
- Turn: 200--5,000 buckets
- River: 200--5,000 buckets
- Libratus-level systems: ~10^12 information sets after abstraction

### Action Abstraction

The no-limit betting structure creates a continuous action space. Typical discretization:
- 3--8 bet sizes per street
- Common sizes: fold/check, call, pot fractions (0.25x, 0.33x, 0.5x, 0.67x, 1x, 1.5x, 2x, all-in)
- Preflop: fixed sizes (2.5bb open, 3-bet to 9bb, etc.)

#### The Off-Tree Problem
When an opponent bets a size not in the abstraction, the system must translate the action to a nearby in-abstraction bet size. Simple rounding is highly exploitable. Libratus addressed this by adding opponent action sizes dynamically (self-improvement module) and by using nested subgame solving to handle off-tree actions properly. DeepStack's continual re-solving naturally handles any bet size since it constructs a fresh subgame at every decision.

### Abstraction Pathologies
Waugh et al. showed that finer abstractions do not always improve performance -- "abstraction pathologies" can cause the strategy in a finer abstraction to be more exploitable than in a coarser one when mapped back to the full game. This is a fundamental challenge and motivates the shift toward abstraction-free approaches (Deep CFR, AlphaHoldem, ReBeL).

---

## Real-Time Solving

Real-time (or online) solving refers to refining the strategy during play at each decision point, rather than relying solely on a precomputed blueprint.

### Safe Subgame Solving (Brown & Sandholm, 2017)
- **Core idea:** When the game reaches a subgame, re-solve it with higher fidelity than the blueprint while provably not increasing exploitability.
- **Mechanism:** Construct a "gadget game" that includes the opponent's option to deviate to the blueprint strategy. The solution to the gadget game is guaranteed to be at least as good as the blueprint at the subgame root.
- **Nested solving:** Subgame solving can be repeated as play progresses deeper into the tree, compounding improvements.

### Depth-Limited Solving (Brown et al., 2018)
- **Core idea:** Limit the search to a fixed depth (typically one street) and use a learned value function to estimate the game value beyond the depth limit.
- **Key insight:** Far cheaper than solving to terminal nodes. GTO Wizard reports a 30,000x reduction in computation by solving one street at a time.
- **Implementation:** At the depth limit, a neural network takes the public state and player ranges as input and outputs estimated counterfactual values for all possible holdings.

### Safe Depth-Limited Solving with Diverse Opponents (DecisionHoldem, 2022)
- **Core idea:** Extends depth-limited solving by considering multiple possible opponent ranges at off-tree nodes, not just the range implied by the blueprint.
- **Performance:** Defeats Slumbot by >730 mbb/h and OpenStack (DeepStack reproduction) by >700 mbb/h.
- **Open source:** Code and tools publicly available.

### Continual Re-solving (DeepStack, 2017)
- **Core idea:** Every decision point is treated as the root of a new subgame. No precomputed blueprint is needed. Value networks provide leaf evaluations for the depth-limited lookahead.
- **Sound:** The strategy profile remains consistent across re-solves during a single hand.

### GTO Wizard AI (2024--2025)
- **Architecture:** Combines CFR with neural networks for depth-limited solving. Solves one street at a time.
- **Performance:** Reaches 0.22% accuracy in 6 seconds on 2 cores / 8 GB RAM (vs. PioSolver: 0.23% accuracy in 81 minutes on 16 cores / 128 GB RAM).
- **QRE upgrade (April 2025):** Switched from Nash Equilibrium to Quantal Response Equilibrium, reducing average flop exploitability from 0.17% to 0.12% of pot. QRE assumes opponents make mistakes and optimizes responses accordingly, eliminating "ghost lines" (spots Nash ignores because they "shouldn't happen").

---

## Alternative Equilibrium Concepts

### Quantal Response Equilibrium (QRE)
- **Concept:** Players choose actions proportional to exponentiated expected payoffs (logit model), rather than playing deterministically optimal. A "temperature" parameter controls rationality -- high temperature = uniform random, low temperature = Nash-like.
- **Advantage over Nash:** Produces well-defined strategies at every game node (no ghost lines). Better exploits imperfect opponents. Faster convergence on complex game trees.
- **Adoption:** GTO Wizard switched to QRE in April 2025, reporting 25% better exploitability metrics.

### Preference-CFR (2024, ICML 2025)
- **Concept:** Introduces "preference degree" and "vulnerability degree" parameters that allow shifting the strategy distribution within an acceptable exploitability threshold. Enables training aggressive, passive, tight, or loose strategy styles while maintaining near-Nash performance.
- **Application:** Allows generating diverse strategy profiles for testing, opponent modeling, or stylistic play.

### Beyond GTO: Profit-Maximizing Agents (2025)
- **Concept:** Pure GTO guarantees non-negative expected value but doesn't maximize profit against weak opponents. The approach computes a GTO baseline via MCCFR self-play, then adapts in real time by observing opponent tendencies and shifting toward exploitative play.
- **Finding:** MCCFR performs best in heads-up situations for both the defensive and exploitative components.

---

## Open Problems

### 1. True Exploitability Is Intractable
Computing the exact exploitability of a strategy in full NLHE HU requires solving the full game to find a best response -- which is computationally intractable. Current measurements use approximate best responses (Local Best Response / LBR) that provide only lower bounds. Best-known approximate Nash strategies for NLHE HU have estimated exploitability of ~20--50 mbb/hand (vs. 0.986 mbb/hand for solved limit hold'em).

### 2. Abstraction Pathologies
Finer abstractions do not always improve the mapped-back strategy's exploitability. There is no reliable way to predict when increasing abstraction resolution will help vs. hurt. Deep CFR and end-to-end RL approaches sidestep this but introduce their own approximation errors.

### 3. Scale-Dependent Neural CFR Behavior
Robust Deep MCCFR (2025) showed that the effectiveness of individual neural CFR components (exploration mixing, target networks, etc.) can completely reverse between small and large games. There is no universal configuration.

### 4. Multiplayer Extensions
Nash equilibrium in multiplayer games is not unique and playing a Nash equilibrium may not be strategically sound (opponents can collude or deviate profitably in correlated ways). CFR does not straightforwardly extend to more than two players. Pluribus succeeded empirically but without theoretical guarantees.

### 5. Opponent Exploitation
The GTO vs. exploitation tradeoff remains largely heuristic. There is no principled framework for dynamically balancing Nash-safe play with opponent-specific exploitation that has convergence guarantees. Bayesian opponent modeling and evolutionary approaches (LSTM, Pattern Recognition Trees) show promise but lack theoretical grounding.

### 6. LLM Integration
PokerBench (AAAI 2025) showed that state-of-the-art LLMs (GPT-4: 53.55% accuracy) significantly underperform GTO solvers at poker. Fine-tuned smaller models (Llama-3-8B) reach 78.26% but remain far from solver-level play. The role of LLMs in poker AI appears limited to natural language interfaces and high-level strategy explanation rather than actual solving.

### 7. Policy Gradient vs. CFR in the Neural Setting
ICLR 2025 work showing generic policy gradient methods matching CFR-based approaches opens fundamental questions about whether the CFR inductive bias is necessary when function approximation is used. If PPO-like methods suffice, the poker AI software stack could simplify dramatically.

---

## Relevance to Myosu Autoresearch Config

The autoresearch config (`champion_config.json`) maps to the solving landscape as follows:

### Method Families

| Config Family | Poker Solving Analog |
|---------------|---------------------|
| `random` (Random Search) | Random strategy sampling; consistently weakest in poker AI research |
| `gradient` (DARTS, AdamW) | Policy gradient methods (MMD, PPO for IIGs); emerging but unproven at scale |
| `augmented` (AugMax) | Augmented/regularized methods (QRE, Preference-CFR); recent frontier |
| `established` (established_method_1, _2) | CFR+ / DCFR / MCCFR variants; proven workhorses |
| `proposed` (proposed_approach, proposed_variant) | Deep CFR / ReBeL / RL+Search hybrids; high potential, higher variance |

### Hyperparameters

| Config Parameter | Poker Solving Interpretation |
|-----------------|------------------------------|
| `graph_iterations: 26` | CFR iterations; the most impactful parameter. DDCFR and HS show orders-of-magnitude speedups by dynamically adjusting iteration weighting. |
| `darts_steps: 22` | Gradient-based solver steps; maps to policy gradient training iterations. Small changes propagate through all evaluations. |
| `random_search_candidates: 40` | Number of random strategy samples; consistent underperformer in poker. |
| `bootstrap_samples: 1000` | Sufficient for ranking accuracy; mirrors poker finding that increasing sample count beyond a threshold adds runtime without improving quality. |
| `calibration_success_threshold: 0.72` | Analogous to exploitability threshold; determines when a strategy is "good enough." |
| `degenerate_tolerance: 1e-06` | Guards against collapsed strategies (action support collapse in neural CFR). |

### Abstraction Strategy

| Config Parameter | Poker Solving Interpretation |
|-----------------|------------------------------|
| `strategy: card_isomorphism` | Lossless suit-isomorphism abstraction; the minimum viable abstraction. Consider upgrading to `potential_aware_emd` for richer clustering. |
| `bucket_count: 200` | Moderate granularity; Libratus-level systems use orders of magnitude more. For a ranking proxy, 200 may be sufficient. |
| `opponent_model: average_strategy` | Standard CFR output. Alternatives: `best_response` (exploitative), `qre` (QRE-based), `diverse_opponents` (DecisionHoldem-style). |

### Recommendations for Config Optimization

1. **Prioritize `established` family methods.** MCCFR/DCFR with Hyperparameter Schedules is the current state of the art. The config's `established_method_1` having the highest mean primary_metric (0.3786) aligns with this.

2. **Consider dynamic iteration weighting.** `graph_iterations` is the most impactful parameter. DDCFR and HS research suggests adaptive schedules could significantly improve ranking accuracy without increasing iteration count.

3. **The `proposed` family's high variance mirrors Deep CFR/ReBeL behavior.** These methods require careful hyperparameter tuning (learning rate, network architecture, replay buffer size) to realize their potential. Reducing variance may require more seeds or stabilization techniques (target networks, gradient clipping).

4. **Random Search and DARTS being weakest performers is expected.** Random search has no convergence guarantees. DARTS (gradient NAS) is designed for architecture search, not game solving -- its weakness in poker-like ranking tasks is structurally expected.

5. **Abstraction strategy is ripe for experimentation.** Upgrading from `card_isomorphism` to `potential_aware_emd` or increasing `bucket_count` could improve ranking accuracy, particularly for postflop scenarios.

---

## Key Papers and References

| Year | Paper | Venue | Contribution |
|------|-------|-------|--------------|
| 2007 | Zinkevich et al., "Regret Minimization in Games with Incomplete Information" | NeurIPS | Introduced CFR algorithm |
| 2009 | Lanctot et al., "Monte Carlo Sampling for Regret Minimization in Extensive Games" | NeurIPS | MCCFR variants (external, outcome, chance sampling) |
| 2013 | Johanson, "Measuring the Size of Large No-Limit Poker Games" | arXiv | Game tree size analysis (~10^161 nodes) |
| 2014 | Tammelin, "Solving Large Imperfect Information Games Using CFR+" | arXiv | CFR+ variant, dramatically faster convergence |
| 2014 | Ganzfried & Sandholm, "Potential-Aware Imperfect-Recall Abstraction with EMD" | AAAI | Leading card abstraction algorithm |
| 2015 | Bowling et al., "Heads-Up Limit Hold'em Poker Is Solved" | *Science* | Essentially solved limit HU (Cepheus) |
| 2016 | Heinrich & Silver, "Deep RL from Self-Play in IIGs" | arXiv | Neural Fictitious Self-Play (NFSP) |
| 2017 | Moravcik et al., "DeepStack: Expert-Level AI in HUNL Poker" | *Science* | Neural-guided continual re-solving |
| 2017 | Brown & Sandholm, "Safe and Nested Subgame Solving for IIGs" | NeurIPS | Safe subgame solving theory |
| 2017 | Brown & Sandholm, "Superhuman AI for HU NL Poker: Libratus" | IJCAI / *Science* (2018) | Blueprint + nested subgame solving |
| 2018 | Brown et al., "Depth-Limited Solving for Imperfect-Information Games" | NeurIPS | Depth-limited solving with learned leaf evaluation |
| 2019 | Brown & Sandholm, "Superhuman AI for Multiplayer Poker" | *Science* | Pluribus (6-max NLHE) |
| 2019 | Brown & Sandholm, "Solving Imperfect-Information Games via Discounted Regret Minimization" | AAAI | DCFR |
| 2019 | Brown et al., "Deep Counterfactual Regret Minimization" | ICML | Deep CFR |
| 2019 | Farina et al., "Stable-Predictive Optimistic CFR" | ICML | PCFR+ |
| 2020 | Brown et al., "Combining Deep RL and Search for IIGs" | NeurIPS | ReBeL framework |
| 2020 | Steinberger, "Unlocking the Potential of Deep Counterfactual Value Networks" | arXiv | Supremus, GPU-accelerated Deep CFR |
| 2022 | Zhao et al., "AlphaHoldem: High-Performance AI for HUNL Poker via E2E RL" | AAAI | End-to-end RL, no CFR |
| 2022 | Bai et al., "DecisionHoldem: Safe Depth-Limited Solving with Diverse Opponents" | arXiv | Improved safe depth-limited solving |
| 2023 | Schmid et al., "Student of Games" | *Science Advances* | Unified algorithm for perfect and imperfect info games |
| 2023 | Li et al., "RM-FSP: Regret Minimization Optimizes NFSP" | Neurocomputing | Improved NFSP via regret minimization |
| 2023 | Sokota et al., "Magnetic Mirror Descent" | NeurIPS | Policy gradient competitive with CFR |
| 2024 | Zhang et al., "Dynamic Discounted CFR" | ICLR (Spotlight) | Learned dynamic discount schedules |
| 2024 | Zhang, McAleer & Sandholm, "Faster Game Solving via Hyperparameter Schedules" | arXiv | Orders-of-magnitude speedup for DCFR/PCFR+ |
| 2024 | Liu et al., "Minimizing Weighted Counterfactual Regret with Optimistic OMD" | IJCAI (Oral) | PDCFR+ |
| 2024 | arXiv 2401.06168, "A Survey on Game Theory Optimal Poker" | arXiv | Comprehensive survey of GTO poker methods |
| 2024 | arXiv 2411.01217, "Preference-CFR: Beyond Nash Equilibrium" | ICML 2025 | Stylistic strategy generation within exploitability bounds |
| 2025 | Zhuang et al., "PokerBench: Training LLMs to Become Professional Poker Players" | AAAI | LLM poker evaluation benchmark |
| 2025 | GTO Wizard, "Introducing Quantal Response Equilibrium" | Blog | QRE adoption in commercial solvers |
| 2025 | El Jaafari, "Robust Deep Monte Carlo CFR" | arXiv | Scale-dependent neural CFR analysis |
| 2025 | arXiv 2502.08938, "Reevaluating Policy Gradient Methods for IIGs" | ICLR | PPO competitive with CFR-based deep RL |
| 2025 | Yi et al., "Beyond GTO: Profit-Maximizing Poker Agents for NLH" | arXiv | GTO baseline + real-time exploitation |
| 2025 | arXiv 2511.08174, "Deep (Predictive) Discounted CFR" | arXiv | Deep DCFR+ and Deep PDCFR+ |
| 2025 | Scientific Reports 15:2917, "Comparative Analysis of EF Zero-Sum Game Algorithms" | Nature | 10-algorithm comparison on poker benchmarks |

---

## Summary of State of the Art (March 2026)

**For blueprint computation:** MCCFR with Hyperparameter Schedules (HS) on top of DCFR or PCFR+ represents the current fastest convergence. DDCFR is a strong alternative when game-specific schedule learning is feasible.

**For real-time play:** Safe depth-limited subgame solving with neural leaf evaluation (Libratus/DeepStack lineage) remains the dominant paradigm. DecisionHoldem's diverse-opponent extension improves off-tree robustness.

**For minimal engineering:** ReBeL and AlphaHoldem demonstrate that competitive performance is achievable without the full CFR+abstraction+subgame-solving stack, though with weaker theoretical guarantees.

**Emerging frontier:** QRE replacing Nash as the target equilibrium concept, policy gradient methods potentially replacing CFR in the neural setting, and Preference-CFR enabling controllable strategy diversity.
