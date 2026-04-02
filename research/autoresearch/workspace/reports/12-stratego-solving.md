# Solving Strategies for Stratego

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, 2007--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Stratego is arguably the most complex imperfect-information game ever tackled by AI. With an estimated game tree of ~10^535 nodes (10^175 times larger than Go), ~10^66 possible initial deployments per player, and game lengths averaging 200--400 moves, Stratego dwarfs all previously studied imperfect-information benchmarks. The combination of a hidden-identity deployment phase, long horizons, and gradual information revelation through combat makes it fundamentally harder than poker variants, where the deck composition is known and game lengths are short.

The landmark breakthrough came in 2022 with DeepNash (Perolat et al., DeepMind), published in *Science*. DeepNash introduced Regularized Nash Dynamics (R-NaD), a model-free deep reinforcement learning algorithm that converges to an approximate Nash equilibrium without requiring any game tree search at inference time. Trained via self-play on TPU infrastructure, DeepNash achieved a 97% win rate against the strongest existing Stratego bots (including multiple Computer Stratego World Championship winners) and an 84% win rate against expert human players on the Gravon online platform, reaching an all-time top-3 ranking. DeepNash demonstrated emergent bluffing, information-material tradeoffs, and deceptive piece movements without any explicit programming of these behaviors.

Prior to DeepNash, Stratego AI development progressed from rule-based systems (Probe, Invincible, Master of the Flag) through Monte Carlo methods and information set MCTS, with none approaching human expert level on the full 10x10 board. Alternative lines of research include Pipeline PSRO (P2SRO, McAleer et al. 2020), which achieved strong results on the smaller Barrage variant, and AlphaZe** (Bluml et al. 2023), which showed that AlphaZero-like search baselines perform surprisingly well on imperfect-information games. As of 2026, DeepNash remains the state of the art for full Stratego, and no published work has definitively surpassed it. Open problems include measuring true exploitability in the full game, scaling R-NaD to even larger domains, and understanding whether search can complement model-free policies for further improvement.

---

## Game Complexity Analysis

### State Space and Information Sets

| Metric | Estimate | Source |
|--------|----------|--------|
| Possible setups per player | ~1.01 x 10^17 (40! / product of piece-count factorials) | Perolat et al. 2022 |
| Combined setup configurations | ~10^34 | Perolat et al. 2022 |
| Game tree nodes (play phase) | ~10^535 | Perolat et al. 2022 |
| Information sets | ~10^175 (estimated) | Perolat et al. 2022 |
| Average game length | 200--400 moves | Empirical |
| Branching factor | ~20--30 legal moves per turn | Empirical |

For comparison: Go has ~10^360 game tree nodes with perfect information. Heads-up no-limit hold'em has ~10^161 game tree nodes with ~3 x 10^14 information sets. Stratego's ~10^535 nodes with ~10^175 information sets make it the largest game for which superhuman play has been demonstrated.

### Why Stratego Is Exceptionally Hard

**1. Two-phase structure.** The game begins with a deployment phase where each player independently arranges 40 pieces on their back four rows. This is a combinatorial optimization problem (choosing from ~10^17 configurations) whose quality only becomes apparent hundreds of moves later. The deployment and play phases co-evolve during learning, requiring the solver to jointly optimize both.

**2. Massive hidden information from the start.** Unlike poker where the deck is known and hands are small, Stratego hides all 40 of an opponent's piece identities at game start. Information is revealed only through combat or movement inference (e.g., a piece that moves multiple squares must be a Scout; a piece that never moves is likely a Bomb or Flag).

**3. Long horizons and credit assignment.** With games lasting 200--400 moves, the delay between action and consequence is extreme. A deployment decision (placing the Flag in a specific location, surrounding it with Bombs) may not be tested until move 150+. This makes temporal credit assignment far harder than in poker (typically 5--20 decision points per hand).

**4. Information asymmetry accumulation.** As combat reveals identities and movement narrows possibilities, the two players accumulate different information at different rates. The player who has gathered more information about the opponent while revealing less about themselves has a strategic advantage. This creates a secondary meta-game of information management.

**5. Intractable search.** Traditional game tree search (minimax, alpha-beta, MCTS) cannot scale to 10^535 nodes, even with aggressive pruning and abstraction. In poker, subgame decomposition allows tractable real-time search. In Stratego, the combination of hidden deployment, long horizons, and lack of clean subgame boundaries makes search-based approaches infeasible for the full game.

**6. No natural abstraction.** Poker benefits from card abstraction (bucketing similar hands) and action abstraction (discretizing bet sizes). Stratego's state space does not decompose as cleanly; the board position, piece identities, movement history, and belief state over opponent pieces form a tightly coupled information structure.

---

## Historical Progression

### 2007--2009: Rule-Based and Monte Carlo Systems

The earliest competitive Stratego AIs relied on handcrafted heuristics, strategic plans, and Monte Carlo simulation.

**Probe** (Sean O'Connor) won the first two Computer Stratego World Championships (2007: 17-0-3 W-L-D; 2008: 22-3-0 W-L-D). Probe combined alpha-beta search with chance nodes to handle unknown piece identities, using heuristic evaluation functions and a probabilistic model of opponent pieces. It treated unknown opponent pieces as "virtual" pieces with probability distributions over possible ranks.

**Invincible** (de Boer, TU Delft, 2007) took a different approach, using strategic plans rather than deep search. De Boer introduced a basic opponent model and focused on deployment strategy and plan execution. A 2009 extension by Stankiewicz at Maastricht University added Bayesian opponent modeling, updating piece-identity probabilities based on observed moves.

**Master of the Flag II** (2009) won the third Computer Stratego World Championship, dethroning Probe. It combined Monte Carlo simulation with heuristic evaluation, demonstrating that simulation-based approaches could outperform pure search methods in the complex information landscape of Stratego.

These systems shared a common limitation: they relied heavily on hand-tuned evaluation functions, hard-coded strategic plans, and simplistic opponent models. They played at a strong amateur level but were far from human expert performance on the full game.

### 2012: Information Set MCTS

Cowling, Powley, and Whitehouse (2012) introduced Information Set Monte Carlo Tree Search (ISMCTS), which extended MCTS to imperfect-information games by sampling deterministic "worlds" consistent with available information and running separate MCTS trees for each. Applied to Stratego, ISMCTS could handle uncertainty about opponent pieces by averaging over sampled opponent configurations.

While theoretically sound, ISMCTS struggled with Stratego's scale. The number of possible determinizations (opponent piece configurations consistent with observations) is enormous, and the long game horizon meant that shallow MCTS rollouts provided poor value estimates. ISMCTS produced moderate-strength play but could not compete with the top heuristic systems on the full game.

### 2016--2017: Neural Fictitious Self-Play

Neural Fictitious Self-Play (NFSP, Heinrich & Silver 2016; Lanctot et al. 2017) was the first deep learning approach applied to imperfect-information game solving at scale. NFSP combines two networks: a Q-network trained by off-policy reinforcement learning (approximating a best response) and a policy network trained by supervised learning on the agent's own past play (approximating the average strategy).

NFSP was tested on simplified Stratego variants and poker games, demonstrating convergence to approximate Nash equilibria from self-play. However, it did not scale to full 10x10 Stratego, where the combination of enormous state space, long horizons, and the deployment phase proved too challenging for the NFSP framework.

### 2020: Pipeline PSRO (P2SRO)

McAleer, Lanier, Fox, and Baldi (2020) introduced Pipeline Policy Space Response Oracle (P2SRO), the first scalable general method for finding approximate Nash equilibria in large zero-sum imperfect-information games. P2SRO extends the Double Oracle / PSRO framework by parallelizing the computation of best responses using a hierarchical pipeline of reinforcement learning workers.

Applied to Barrage Stratego (the smaller 8x8 variant), P2SRO achieved a 71% average win rate against existing Barrage Stratego bots after 820,000 training episodes. While this demonstrated the viability of population-based methods for Stratego variants, P2SRO was not scaled to the full 10x10 game due to computational constraints.

### 2022: DeepNash (The Breakthrough)

DeepNash (Perolat et al., DeepMind, published in *Science*) achieved human expert-level play on the full 10x10 Stratego, the first AI to do so. The key innovation was Regularized Nash Dynamics (R-NaD), a model-free deep RL algorithm that directly converges to Nash equilibrium in two-player zero-sum games. DeepNash requires no game tree search at inference time --- the policy network directly outputs action probabilities from the current observation. Full details are in the next section.

### 2023: AlphaZe** (AlphaZero-like Baselines)

Bluml et al. (TU Darmstadt, published in *Frontiers in AI*) introduced AlphaZe**, an adaptation of AlphaZero's neural MCTS framework to imperfect-information games. The key modification is sampling determinized worlds (possible opponent configurations) and running MCTS on each, then aggregating the resulting action recommendations.

Applied to Barrage Stratego and DarkHex, AlphaZe** achieved results comparable to P2SRO and beat most existing Barrage Stratego bots. However, it did not match DeepNash's performance on full Stratego, and the search overhead at inference time made it significantly slower than DeepNash's pure-policy approach. The paper's significance lies in demonstrating that simple search-based baselines remain competitive for smaller imperfect-information games, challenging the assumption that entirely new algorithmic frameworks are required.

### 2025: Constraint-Based Belief States

Piette et al. (IEEE Conference on Games 2025) investigated constraint-based belief state representations for Stratego. The work compared two approaches: (1) constraint satisfaction problems (CSPs) that track logically feasible piece identities, and (2) probabilistic extensions using Belief Propagation to estimate piece-identity likelihoods. The key finding was that constraint-based beliefs yielded comparable performance to probabilistic inference in both Mini-Stratego and Goofspiel, suggesting that tracking what is logically possible may be as valuable as estimating precise probabilities.

---

## DeepNash Deep Dive

### The R-NaD Algorithm

Regularized Nash Dynamics (R-NaD) is the core algorithmic innovation enabling DeepNash. It addresses a fundamental problem in multi-agent learning: standard policy gradient methods (including their game-theoretic variants like Fictitious Play and NFSP) tend to *cycle* around Nash equilibria rather than converging to them. This cycling behavior is well-documented in two-player zero-sum games and makes it impossible to produce stable, approximately-optimal policies.

R-NaD solves this by introducing strongly convex regularization into the learning dynamics, inspired by evolutionary game theory's replicator dynamics. The algorithm operates in three phases:

**Phase 1: Reward Transformation.** The raw game rewards are transformed by subtracting a KL-divergence regularization penalty relative to a reference policy. This creates a regularized game where the Nash equilibrium of the regularized game is a "smoothed" version of the original Nash equilibrium. The regularization induces boundary repulsion (preventing policies from collapsing to deterministic), smoothing (making the optimization landscape better-behaved), and improved stability (damping oscillations).

**Phase 2: Dynamics (Learning Loop).** The dynamics phase consists of two interleaved components:
- **Value estimation** via an adapted v-trace estimator. Standard v-trace (used in IMPALA-style distributed RL) is adapted to the two-player imperfect-information setting, where the value of a state depends on both players' policies and the hidden information.
- **Policy update** via Neural Replicator Dynamics (NeuRD). NeuRD is derived from the replicator dynamics of evolutionary game theory, adapted for neural network function approximation. It amounts to a one-line modification of standard policy gradient: instead of computing the gradient through the softmax normalization, NeuRD bypasses it, directly applying the payoff-weighted update to the logits. This gives the dynamics a game-theoretic convergence property that standard policy gradient lacks.

**Phase 3: Reference Policy Update.** After a sufficient number of dynamics steps, the current learned policy is frozen and becomes the new reference policy for the next iteration of regularization. This iterative refinement of the reference policy is proven to converge: there exists a Lyapunov function that decreases monotonically at each update, driving the sequence of reference policies toward a Nash equilibrium. The Bregman divergence to Nash decreases strictly at each iteration, guaranteeing last-iterate convergence without requiring uniqueness assumptions on the equilibrium.

### Neural Network Architecture

DeepNash uses five U-Net convolutional neural networks, structured as a residual network with skip connections characteristic of U-Net architectures. The five networks serve distinct roles:

1. **Observation encoder.** Processes an encoded observation tensor consisting of the player's own pieces, public information (revealed identities, board positions), game phase indicator, and the past 40 actions (providing temporal context for inference about opponent behavior). This encoder produces a shared embedding.

2. **Value head.** Estimates the expected future reward from the current state (used during training for v-trace value estimation). Outputs a scalar.

3. **Deployment policy head.** During the deployment phase, outputs a probability distribution over where to place each piece. This head must generate diverse, unpredictable deployments (DeepNash can produce billions of unique deployment configurations).

4. **Piece selection policy head.** During the play phase, selects which piece to move. Outputs a distribution over the player's mobile pieces.

5. **Move selection policy head.** Given a selected piece, outputs a distribution over legal moves for that piece.

The three policy heads represent DeepNash's three-phase action structure: deploy, select piece, select move. This factored action representation is crucial for tractability, as the joint action space (piece x move) would be prohibitively large for a single softmax output.

### Training Infrastructure

DeepNash was trained via large-scale distributed self-play. The training loop follows the actor-learner paradigm:

- **Actors** run self-play games using the current policy, generating experience trajectories (observations, actions, rewards).
- **Learners** consume experience from actors and update the neural network parameters via R-NaD.
- A **replay buffer** stores recent trajectories for off-policy learning with importance sampling correction (v-trace).

Training was conducted on Google's TPU infrastructure, with the learner running on TPU accelerators and actors distributed across many CPU workers. The exact scale is not fully disclosed in the public paper, but the compute requirements are substantial (estimated at millions of self-play games over the training period). The training process jointly optimizes all three policy heads and the value head through end-to-end gradient updates.

### Fine-Tuning and Post-Processing

DeepNash employs two additional stages beyond core R-NaD training:

1. **Fine-tuning.** The learned policy is fine-tuned to improve practical performance. Details of the fine-tuning procedure are not fully elaborated in the public paper but likely involve additional self-play with adjusted hyperparameters.

2. **Test-time post-processing.** At inference time, the raw policy outputs are post-processed to remove clearly suboptimal actions (such as attacking a known Bomb with a non-Miner piece) and to enforce game rules (such as the two-square repetition rule). This post-processing is lightweight and does not involve search.

### Results

| Opponent Category | Win Rate | Games |
|-------------------|----------|-------|
| Strongest Stratego bots (Probe, Invincible, MoF2, etc.) | 97.1% | 800 |
| Individual championship-winning bots | Frequently 100% | Per-bot matchups |
| Human experts on Gravon | 84% | ~2 weeks of play |
| Gravon ranking | All-time top 3 (as of April 2022) | Since 2002 |

---

## Deployment Phase Strategy

The initial piece deployment is a hidden combinatorial optimization problem with ~10^17 configurations per player. Deployment quality has outsized long-term impact because it determines Flag safety, offensive capability, and overall army structure.

### Human Expert Principles

Human Stratego experts follow established deployment heuristics:

- **Flag placement:** Almost always on the back row, often in a corner or near-corner position. Surrounded by Bombs to create a fortress that only Miners can penetrate.
- **Bomb formations:** Common patterns include L-shapes and rows of Bombs around the Flag, creating multi-Bomb barriers. A skilled player varies formations to avoid predictability.
- **Offensive positioning:** Marshal, General, and strong attack pieces placed in forward rows for early aggression. Scouts distributed for reconnaissance.
- **Spy positioning:** Often near the Marshal to create attack/defense synergy (the Spy kills the Marshal if it attacks first, but dies to anything else).
- **Miner distribution:** Spread across the front to handle Bombs encountered during attack.

### DeepNash's Learned Deployment

DeepNash learned deployment strategies through self-play without any human heuristic input. Its learned deployment policy exhibits several notable properties:

- **Diversity:** DeepNash can generate billions of unique deployment configurations. Unpredictability in deployment is essential for avoiding exploitation; a deterministic deployment would allow an opponent who observes it to develop a targeted counter-strategy.
- **Flag protection:** Like human experts, DeepNash almost always places the Flag on the back row and protects it with Bombs. This pattern emerged purely from self-play optimization.
- **Flexible offensive structures:** Unlike human players who often follow template formations, DeepNash's deployments are more varied, making opponent modeling more difficult.
- **Joint optimization:** The deployment policy co-evolves with the play policy during training. This means deployment decisions are optimized for the specific play strategy that DeepNash will execute, rather than being optimized in isolation.

### Deployment as a Separate Subproblem

Some researchers have investigated optimizing deployment independently from play. However, DeepNash's end-to-end approach demonstrated that joint optimization is superior: a deployment that is "optimal" in isolation may be suboptimal for the specific play style of the agent. The deployment and play phases are tightly coupled through the information structure of the game.

---

## Information Tracking and Belief States

### The Inference Problem

At any point during a Stratego game, a player knows:
- Their own piece identities and positions.
- Positions of all opponent pieces (but not identities for unrevealed pieces).
- Which opponent pieces have been revealed through combat.
- The complete history of moves (which pieces moved where, when).

From this information, the player must maintain beliefs about the identities of unrevealed opponent pieces. This is a constraint satisfaction and probabilistic inference problem.

### Constraint-Based Tracking

Each unrevealed opponent piece has a domain of possible identities, constrained by:
- **Global count constraints.** The opponent has exactly 1 Marshal, 1 General, 2 Colonels, etc. As pieces are revealed or captured, the remaining count constrains surviving pieces.
- **Movement constraints.** A piece that has moved cannot be a Bomb or Flag (these are immobile). A piece that moved multiple squares must be a Scout.
- **Combat outcomes.** If piece A attacked piece B and survived, A's rank must be greater than or equal to B's rank (with special-case rules for Spy and Miner).
- **Non-movement inference.** A piece that has never moved despite having the opportunity may be a Bomb or Flag, though skilled players sometimes deliberately leave mobile pieces stationary to create ambiguity.

The 2025 work by Piette et al. formalized this as a Constraint Satisfaction Problem (CSP) and showed that propagating constraints alone (without probabilistic reasoning) captures most of the useful inference. When a constraint propagation forces a piece's domain to a single identity, that piece is effectively revealed without combat.

### Probabilistic Belief Models

Beyond constraint satisfaction, probabilistic models assign likelihoods to each feasible identity for each piece:

- **Bayesian opponent modeling** (Stankiewicz 2009): Updates piece-identity probabilities based on observed moves. For example, a piece that advances aggressively toward known strong pieces is more likely to be high-ranked.
- **Belief Propagation** (Piette et al. 2025): Propagates probability distributions through the constraint graph, using message-passing to estimate marginal probabilities for each piece-identity assignment.
- **Neural implicit beliefs** (DeepNash): Rather than maintaining an explicit belief state, DeepNash encodes the observation history (past 40 actions) directly into the neural network input. The network learns to implicitly track beliefs through its internal representations. This approach avoids the combinatorial explosion of maintaining an explicit probability distribution over ~10^17 possible opponent configurations.

### DeepNash's Approach: Implicit Belief via History Encoding

DeepNash does not maintain an explicit belief state. Instead, it feeds the raw observation (own pieces, revealed opponents, board positions) plus the 40 most recent actions into the neural network. The network learns to extract relevant belief information from this history through training. This is a crucial architectural choice: explicit belief tracking in full Stratego would require maintaining a distribution over an astronomical number of possible worlds. By delegating belief tracking to learned neural network representations, DeepNash sidesteps this complexity entirely.

---

## Bluffing and Deception

### Deception as Emergent Behavior

One of the most striking findings from DeepNash is that bluffing and deception emerge naturally from game-theoretic self-play without any explicit programming. Because DeepNash converges toward a Nash equilibrium, its mixed strategies inherently include deceptive play at optimal frequencies. In a Nash equilibrium, deception is not ad hoc --- it occurs precisely when and because it maximizes expected value against a best-responding opponent.

### Specific Bluffing Behaviors Observed in DeepNash

**Piece-strength misrepresentation.** DeepNash learned to move low-ranking pieces (such as Scouts, rank 2) aggressively, mimicking the movement patterns of high-ranking pieces. In one documented game, DeepNash advanced a Scout toward an opponent's exposed Colonel (rank 8), creating the impression that the approaching piece was a General (rank 9) or Marshal (rank 10). The human opponent retreated the Colonel, allowing DeepNash to gain positional advantage with a worthless piece.

**Spy ambush setup.** DeepNash learned to create situations where a moderate-strength piece chases an exposed high-value opponent piece, inducing the opponent to commit their Marshal to counter. The Marshal is then lured past a waiting Spy, which can kill the Marshal on its own initiative. This multi-step deceptive plan involves sacrificing a moderate piece's position to set up a devastating Spy kill.

**Information-material tradeoffs.** In a documented game against a human expert, DeepNash sacrificed a Major (rank 7) and a Colonel (rank 8) early in the game through deliberate probing attacks. While this left DeepNash at a material disadvantage, it revealed the positions of the opponent's Marshal, General, Colonel, and two Majors. DeepNash evaluated its winning probability at 70% despite the material deficit, and ultimately won. This demonstrates that the AI learned to value information over material when the tradeoff is favorable.

**Deployment deception.** By generating highly diverse deployments, DeepNash prevents opponents from developing counter-strategies based on recognizing deployment templates. The diversity itself is a form of strategic deception at the meta-game level.

### Why Equilibrium Play Produces Deception

In a Nash equilibrium of an imperfect-information game, players must randomize their actions to prevent opponents from exploiting predictable patterns. A player who never bluffs is exploitable (opponents can safely retreat from any aggressive piece, knowing it must be strong). A player who always bluffs is also exploitable (opponents can call bluffs with any piece). The Nash equilibrium bluff frequency is the mathematically optimal point where the opponent cannot profit from either calling or folding. This is the same principle that governs bluffing frequency in poker Nash equilibria.

---

## Open Problems

### 1. Exploitability Measurement

The true exploitability of DeepNash's strategy (how much an optimal counter-strategy could gain against it) is computationally intractable to measure in the full game. The ~10^535 game tree makes it impossible to compute a best response directly. DeepNash's quality is assessed empirically (win rates against known opponents) rather than theoretically (distance from true Nash equilibrium). Developing tractable approximations for exploitability in games of this scale remains an open problem.

### 2. Search-Policy Hybrid Approaches

DeepNash demonstrates that pure policy networks can achieve expert-level play without search. However, in perfect-information games, AlphaZero showed that combining learned policies with MCTS search produces stronger play than either alone. Whether a similar synergy exists for imperfect-information games is unclear. AlphaZe** (2023) explored this direction for smaller games, but the question remains open for full Stratego. The fundamental challenge is that MCTS in imperfect-information games requires sampling determinized worlds, which may introduce systematic biases.

### 3. Scaling R-NaD Beyond Two-Player Zero-Sum

R-NaD is theoretically grounded for two-player zero-sum games. Extending it to multiplayer or general-sum settings (where Nash equilibrium may not be unique or strategically meaningful) is an open theoretical and practical challenge. This is relevant for potential Stratego variants with more than two players or asymmetric objectives.

### 4. Transfer and Generalization

DeepNash is trained from scratch for Stratego. Whether R-NaD policies or learned representations transfer to related games (other hidden-identity board games, other deployment-based games) is unexplored. Developing methods for cross-game transfer in imperfect-information settings would significantly reduce training costs.

### 5. Sample Efficiency

DeepNash requires millions of self-play games for training, with significant TPU compute. Improving sample efficiency through model-based approaches (learning a world model and planning through it), curriculum learning (training on progressively larger board sizes), or transfer learning from smaller variants (Barrage to full Stratego) are active research directions.

### 6. Human Interpretability

DeepNash's decisions are opaque. While human observers can describe its behavior in terms of "bluffing" and "sacrifice for information," the neural network's internal representations do not provide causal explanations for its choices. Developing interpretable models or post-hoc explanation methods for imperfect-information game policies would be valuable for human learning and trust.

### 7. Real-Time Opponent Exploitation

DeepNash plays a fixed (Nash equilibrium) strategy regardless of the opponent. Against weak opponents, an exploitative strategy could win more often. Developing methods that maintain Nash equilibrium guarantees as a safety baseline while opportunistically exploiting detected opponent weaknesses is an open problem in Stratego and imperfect-information games generally.

---

## Relevance to Myosu

### Architectural Implications

Stratego is the largest and most complex imperfect-information game in Myosu's 20-game survey. Its inclusion has several architectural implications:

**CFR is not viable for Stratego.** The 10^535 game tree and 10^175 information sets make any form of CFR (even with aggressive abstraction) computationally intractable. This means Myosu's solver framework cannot rely exclusively on CFR-family algorithms; it must support deep RL approaches like R-NaD.

**Model-free policies are sufficient.** DeepNash's success without inference-time search demonstrates that pure neural network policies can achieve superhuman play in the most complex imperfect-information game studied. This validates an architecture where the solver's output is a trained policy network, not a real-time search engine.

**Low inference latency.** Without search, DeepNash's inference is a single forward pass through the neural network. This makes it suitable for real-time play on modest hardware, which is important for Myosu's subnet where solver nodes must respond within time constraints.

### Training Compute Considerations

R-NaD training for full Stratego requires substantial distributed compute. For Myosu's subnet:
- The training workload could be distributed across solver nodes, with each node contributing self-play games and gradient updates.
- Barrage (8x8) Stratego could serve as a computationally cheaper validation target during development.
- Progressive training (Barrage first, then full Stratego) could reduce total training time.

### Evaluation Strategy

- **Head-to-head Elo rating** from round-robin tournaments is the natural evaluation metric, following DeepNash's approach on Gravon.
- **Deployment diversity** can be measured independently: a solver that generates repetitive deployments is exploitable.
- **Win rate against known baselines** (Probe, P2SRO-trained agents) provides calibration points.

### Solver Method Ranking

| Method | Applicability to Stratego | Notes |
|--------|---------------------------|-------|
| CFR / MCCFR | 1/5 | Game tree too large |
| Deep CFR | 2/5 | Theoretically possible but not demonstrated at this scale |
| R-NaD / DeepNash | 5/5 | Proven approach, state of the art |
| PSRO / P2SRO | 3/5 | Demonstrated on Barrage, unclear for full game |
| AlphaZero-like search | 2/5 | Weaker than R-NaD; search overhead problematic |
| NFSP | 2/5 | Did not scale to full game |
| ISMCTS | 2/5 | Moderate results; long horizons problematic |

### Unique Value for Myosu

Stratego is architecturally distinct from the poker variants that dominate most imperfect-information game research. It demonstrates that Myosu's framework must handle:
- Board-based spatial reasoning (not just card/action sequences)
- Hidden identity (not just hidden cards from a known deck)
- Two-phase games (deployment + play)
- Very long horizons (200--400 moves vs. 5--20 decision points in poker)
- Emergent deception without explicit bluff modeling

Successfully handling Stratego alongside poker variants would demonstrate that Myosu's solver infrastructure is genuinely general, not poker-specific.

---

## Key Papers & References

| Year | Authors | Title | Venue | Contribution |
|------|---------|-------|-------|--------------|
| 2007 | de Boer | Invincible: A Stratego Bot | TU Delft M.Sc. thesis | Plan-based Stratego AI with basic opponent model |
| 2007 | Various | 1st Computer Stratego World Championship | ICGA | First competitive Stratego AI tournament; Probe wins |
| 2009 | Stankiewicz | Opponent Modelling in Stratego | Maastricht Univ. B.Sc. thesis | Bayesian piece-identity inference from observed moves |
| 2009 | Schadd et al. | The 3rd Stratego Computer World Championship | ICGA Journal | Master of the Flag II wins; MC simulation approaches |
| 2012 | Cowling, Powley, Whitehouse | Information Set MCTS | IEEE Trans. Comp. Intell. AI Games | ISMCTS framework applicable to Stratego |
| 2016 | Heinrich & Silver | Deep RL from Self-Play in Imperfect-Information Games | NeurIPS | NFSP algorithm for imperfect-information self-play |
| 2019 | Omidshafiei et al. | Neural Replicator Dynamics | AAMAS 2020 | NeuRD: policy gradient with replicator dynamics convergence |
| 2020 | McAleer et al. | Pipeline PSRO: A Scalable Approach for Finding Approximate Nash Equilibria in Large Games | NeurIPS | P2SRO applied to Barrage Stratego; 71% win rate vs. bots |
| 2020 | Brown et al. | Combining Deep RL and Search for Imperfect-Information Games | NeurIPS | ReBeL framework for search + RL in imperfect info |
| 2022 | Perolat, De Vylder, Tuyls et al. | Mastering the Game of Stratego with Model-Free Multiagent RL | *Science* | **DeepNash: R-NaD algorithm, superhuman Stratego** |
| 2023 | Bluml et al. | AlphaZe**: AlphaZero-like Baselines for Imperfect Information Games Are Surprisingly Strong | Frontiers in AI | Search-based baseline competitive on Barrage Stratego |
| 2025 | Piette et al. | Modeling Uncertainty: Constraint-Based Belief States in Imperfect-Information Games | IEEE CoG 2025 | Constraint-based vs. probabilistic beliefs for Stratego |

### Additional References

- Perolat et al. 2022 (arXiv:2206.15378): Full technical paper with architecture, training, and evaluation details.
- McAleer et al. 2021 (XDO): Double Oracle algorithm for extensive-form games with convergence guarantees.
- Lanctot et al. 2017: Unified game-theoretic approach to multi-agent RL; NFSP for imperfect-info games.
- JBLanier/stratego_env (GitHub): Open-source gym-like environment for Stratego, Barrage, and toy variants.
- AbhinavPeri/DeepNash (GitHub): Community reimplementation of the DeepNash algorithm.
- baskuit/R-NaD (GitHub): Experimental R-NaD implementation on GPU-accelerated games.

### Key URLs

- DeepMind blog post: https://deepmind.google/blog/mastering-stratego-the-classic-game-of-imperfect-information/
- Science publication: https://www.science.org/doi/10.1126/science.add4679
- arXiv preprint: https://arxiv.org/abs/2206.15378
- Leon Ericsson technical breakdown: https://leonericsson.github.io/blog/2023-10-09-deepnash
- Emergent Mind summary: https://www.emergentmind.com/papers/2206.15378
- AlphaZe** paper: https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2023.1014561/full
- P2SRO paper: https://arxiv.org/abs/2006.08555
- Belief states 2025 paper: https://arxiv.org/abs/2507.19263
