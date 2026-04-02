# Solving Strategies for Dou Di Zhu (Fight the Landlord)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, 2011--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Dou Di Zhu (斗地主, "Fight the Landlord") is the most popular card game in China with over 600 million registered players, and has become one of the most actively studied imperfect-information games in AI research. The game presents a unique combination of challenges: asymmetric 1-vs-2 team dynamics (landlord against two cooperating peasants), a massive combinatorial action space of 27,472 possible card combinations, imperfect information from hidden hands, and the requirement for implicit cooperation between peasant agents who cannot communicate directly. These properties make it fundamentally different from poker-centric game-solving approaches and resistant to classical techniques like Counterfactual Regret Minimization (CFR), which loses all theoretical convergence guarantees in three-player settings.

The watershed moment for Dou Di Zhu AI came in 2021 with DouZero (Zha et al., ICML 2021), which demonstrated that a conceptually simple Deep Monte Carlo (DMC) approach---combining MC sampling with deep neural networks, action encoding via card matrices, and massively parallel self-play---could achieve superhuman performance without any human knowledge, search, or action abstraction. DouZero reached #1 on the Botzone leaderboard among 344+ agents using just a single server with four GTX 1080 Ti GPUs. Since DouZero, the field has seen rapid iteration: PerfectDou (NeurIPS 2022) introduced perfect-information distillation for dramatically improved sample efficiency; DouZero+ added opponent modeling and coach-guided learning; OADMCDou (IJCAI 2024) combined oracle guiding with adaptive DMC; AlphaDou (2024) integrated bidding for end-to-end play; and FPDou (2025) achieved a new state of the art with a 3x smaller model by applying Generalized Weakened Fictitious Play with a two-player reduction of the peasant coalition. The field remains highly active, driven by the game's cultural significance in China and its role as a benchmark for multi-agent imperfect-information game solving.

The dominant paradigm is model-free deep RL (DMC or PPO) with position-specific networks, though recent work increasingly incorporates search (MCTS), information distillation, and game-theoretic frameworks (fictitious play). CFR is largely inapplicable. Open problems include principled peasant cooperation, optimal bidding integration, exploitability measurement in three-player games, and scaling these techniques to the four-player double-deck variant.

---

## Game Complexity Analysis

### Three-Player Asymmetric Structure

Dou Di Zhu uses a 54-card deck (standard 52 plus two jokers). Each player receives 17 cards, with 3 remaining "kitty" cards awarded to the winning bidder who becomes the Landlord (20 cards). The two Peasants cooperate implicitly to defeat the Landlord. This 1-vs-2 asymmetry is fundamental:

- **Landlord advantage:** 3 extra cards, leads first, sees the kitty
- **Peasant advantage:** numerical superiority (34 cards combined vs 20), cooperative play
- **Information asymmetry:** all players see the kitty cards revealed, but only the Landlord knows which of their 20 cards came from the kitty

### State Space and Action Space

| Metric | Estimate | Notes |
|--------|----------|-------|
| Possible deals | ~10^28 | C(54,17) x C(37,17) x C(20,3) |
| Legal card combinations | 27,472 | All possible combination types across all cards |
| Legal actions per turn | 20--100+ | Highly variable depending on hand and game state |
| Average game length | 15--30 actions | Per player |
| Game tree nodes | ~10^30--10^50 | Estimated |
| Information sets | ~10^20--10^35 | Estimated |
| Average branching factor | 30--100+ | Much larger than poker (~3--10) |

### Why the Action Space Is Unique

Unlike poker where actions are bet sizes (a small discrete set), Dou Di Zhu actions are card combinations drawn from 14 distinct combination types: singles, pairs, triples, triple+single, triple+pair, sequences (5+ cards), pair sequences (3+ pairs), triple sequences ("airplanes"), airplanes with wings (singles or pairs), quadplex sets (four-of-a-kind + 2 singles or 2 pairs), bombs (four-of-a-kind), and rockets (both jokers). The combinatorial explosion comes from:

1. **Multiple valid decompositions** of any given hand into playable combination sets
2. **Variable-length sequences** (a 20-card hand might contain straights of length 5--12)
3. **Kicker selection** for airplanes and quadplex sets (choosing which cards to attach)
4. **Context dependence:** when following, only combinations of the same type and length that beat the current lead are legal; when leading, any combination type is valid

This creates a fundamentally different challenge than poker: the action space cannot be easily abstracted because removing even one card from a combination can break other planned combinations (e.g., using a card as a kicker destroys a potential straight).

### Comparison to Other Games

| Game | Action Space Size | Information Sets | Players | Key Difficulty |
|------|------------------|------------------|---------|----------------|
| HULHE | ~10^4 | ~3.2 x 10^14 | 2 | Solved (2015) |
| HUNLHE | ~10^4 | ~6.4 x 10^161 | 2 | Near-solved (Libratus, Pluribus) |
| Dou Di Zhu | ~2.7 x 10^4 per turn | ~10^20--10^35 | 3 (1v2) | Asymmetric teams, combo actions |
| Mahjong (4P) | ~10^2 per turn | ~10^48 | 4 | Tile drawing stochasticity |

Dou Di Zhu's per-turn action space rivals poker's total action vocabulary, but each decision requires selecting from a much larger set of structurally different combination types. Combined with three-player dynamics and cooperation requirements, it occupies a unique position in game complexity research.

---

## Historical Progression

### 2011--2016: Rule-Based and Early MCTS

**Determinization MCTS (Whitehouse, Powley, Cowling, 2011).** The earliest academic work on Dou Di Zhu AI applied Information Set Monte Carlo Tree Search (ISMCTS) to the game. The approach samples deterministic "worlds" consistent with observed information and runs MCTS on each. While theoretically sound, ISMCTS struggled with Dou Di Zhu's large action space and the difficulty of accurately modeling peasant cooperation during rollouts.

**Rule-based systems.** Early Chinese commercial Dou Di Zhu games used hand-crafted heuristic systems: card-counting, fixed combination priority tables, and simple bidding thresholds. These systems played at recreational level but lacked adaptability and strategic depth.

### 2019: Deep RL Enters the Arena

**Combinational Q-Learning (CQL, You et al., 2019).** The first deep RL approach specifically designed for Dou Di Zhu's combinatorial action space. CQL introduced a two-stage architecture:
1. **Decomposition Proposal Network (DPN):** evaluates possible hand decompositions (ways to partition remaining cards into playable combinations) and selects the optimal decomposition via Q-values
2. **Move Proposal Network (MPN):** within the selected decomposition, chooses the best action

This reduced the effective action space from thousands to tens at each stage, achieving a 30% win-rate improvement over naive DQN and A3C baselines. CQL demonstrated that standard RL algorithms (DQN, A3C) fail on Dou Di Zhu due to: (a) overestimation bias from iterating over the massive action space, (b) sparse binary rewards, and (c) variable-length action sets that make batch learning difficult.

**DeltaDou (Jiang et al., IJCAI 2019).** The first system to claim expert-level Dou Di Zhu play. DeltaDou combined:
- **Fictitious Play MCTS (FPMCTS):** an AlphaZero-style MCTS adapted for imperfect information, where other players are assumed to follow fixed (but learned) strategies
- **Policy-value network:** approximates policy and value at each decision node
- **Bayesian inference:** infers hidden cards from observed play history
- **Kicker network:** a specialized network for action abstraction in kicker selection

DeltaDou required 2 months of training on 68 CPUs, a significant compute investment that reflected the difficulty of combining search with learning in this domain.

### 2020: Multirole Modeling

**Li et al. (Complexity, 2020)** proposed a framework with three components: role modeling (CNN-based classification of game situations), card carrying (evaluation algorithm for combination planning), and role-specific decision making. The system trained separate models on landlord-winning and peasant-winning historical data, producing play that qualitatively resembled human decision-making. While not state-of-the-art, this work highlighted the importance of role-specific strategies.

### 2021: The DouZero Revolution

**DouZero (Zha et al., ICML 2021)** transformed the field by demonstrating that a simple, scalable approach could dominate all previous systems without search, abstraction, or human knowledge.

**Combining Tree Search and Action Prediction (Zhang et al., IJCAI 2021).** Concurrent with DouZero, this work extended AlphaZero to multiplayer imperfect-information games via Action-Prediction MCTS (AP-MCTS). The method builds search trees on public information and predicts opponents' actions directly. Against experienced human players, AP-MCTS achieved a 65.65% win rate, roughly double the human baseline. Its Elo rating was 50--200 points above existing Dou Di Zhu AIs.

---

## DouZero Deep Dive

### Core Algorithm: Deep Monte Carlo (DMC)

DouZero's algorithm is a surprisingly simple combination of Monte Carlo sampling with deep Q-networks:

1. **Self-play generation:** parallel actor processes play games against themselves, generating complete game trajectories (state, action, reward triples)
2. **Q-value estimation:** for each state-action pair (s, a), the return is the final game outcome (win = +1, loss = -1), with no intermediate rewards or bootstrapping
3. **Network training:** a centralized learner updates the Q-network to minimize the MSE between predicted Q-values and observed Monte Carlo returns
4. **Action selection:** during play, the agent selects argmax_a Q(s, a) over legal actions

This avoids temporal-difference bootstrapping entirely, sidestepping overestimation bias that plagues DQN in large action spaces. The high variance inherent in MC methods is mitigated by massive parallelism---thousands of games per second across 48 CPU actors.

### State and Action Encoding

**State features** for each position include:
- Current hand cards (4 x 15 binary matrix, rows = suits conceptually but encoding card counts 0--4 for each of 15 ranks: 3 through 2 plus two jokers)
- Union of opponents' remaining cards
- Most recent moves in the action history
- Number of cards remaining for each opponent (one-hot)
- Number of bombs played so far (one-hot)

**Action encoding** represents each card combination as a 4 x 15 binary matrix (same format as hand encoding), enabling the network to generalize across structurally similar actions---e.g., the pattern for a straight (3-4-5-6-7) looks similar to (8-9-10-J-Q), allowing the network to transfer knowledge between them.

### Network Architecture

- **LSTM encoder:** processes the sequence of historical actions to capture game dynamics and implicitly infer hidden information
- **MLP head:** six fully-connected layers with hidden dimension 512, producing Q-values for given state-action pairs
- **Position-specific networks:** three separate networks for Landlord, Peasant-downstream (plays after Landlord), and Peasant-upstream (plays before Landlord), reflecting the asymmetric roles

### Training Details

| Parameter | Value |
|-----------|-------|
| Hardware | 1 server, 4 GTX 1080 Ti GPUs, 48 CPU cores |
| Training time to SOTA | ~10 days |
| Actors | 45 parallel self-play processes |
| Batch size | 32 |
| Learning rate | 1e-4 (Adam) |
| Samples generated | Billions (thousands per second) |
| Human knowledge | None (pure self-play from random initialization) |
| Search at inference | None (single forward pass) |

### Results

- **Botzone leaderboard:** #1 among 344 agents (Elo 1625.11)
- **vs DeltaDou:** surpassed within 10 days (DeltaDou required 2 months with human heuristics)
- **vs human players:** superhuman performance in Botzone matches
- **Open source:** github.com/kwai/DouZero (1.4k+ stars)

### Why DMC Works for Dou Di Zhu

1. **No overestimation:** MC returns are unbiased, unlike TD-bootstrapped Q-values that compound errors across the large action space
2. **Action generalization:** card matrix encoding lets the network share representations across the 27,472 possible actions
3. **Implicit opponent modeling:** the LSTM history encoder learns to infer opponents' likely holdings and strategies from observed play
4. **Scalable parallelism:** self-play games are independent and embarrassingly parallel, converting compute directly into data quality

---

## Current Best Approaches

### PerfectDou (NetEase, NeurIPS 2022)

**Method:** Actor-critic with perfect information distillation.

**Key innovation:** During training, the critic (value network) has access to all players' cards (perfect information), while the actor (policy network) sees only its own cards (imperfect information). The critic's knowledge is "distilled" into the actor through the advantage estimates used in PPO updates. At inference, only the actor is used.

**Technical details:**
- PPO with Generalized Advantage Estimation (GAE)
- Oracle value function uses minimum steps to play out all cards as a training signal
- Parallel training paradigm with position-specific networks
- Card and game features engineered to represent both perfect and imperfect information views

**Results:** With 1 billion training samples, PerfectDou beats DouZero (trained with 10 billion samples) with a winning percentage of 0.732 and average difference of 1.270 points. This represents dramatically superior sample efficiency.

**Strengths:** Best sample efficiency; strong absolute performance; no search needed at inference.
**Weaknesses:** Requires careful engineering of the perfect/imperfect information interface; PPO hyperparameter sensitivity.
**Open source:** github.com/Netease-Games-AI-Lab-Guangzhou/PerfectDou

### DouZero+ / Full DouZero+ (Zhao et al., IEEE CoG 2022 / 2023)

**Method:** Extensions to DouZero adding opponent modeling, coach-guided learning, and bidding.

**Key innovations:**
1. **Opponent modeling:** a separate network predicts opponents' likely hands from observed play, augmenting the state representation
2. **Coach network:** a "teacher" network trained with global information guides the student (playing) network, accelerating convergence
3. **Bidding network (Full version):** Monte Carlo simulation-based training for the landlord bidding phase, typically ignored by other systems

**Results:** Ranked #1 on Botzone among 400+ agents, surpassing the original DouZero.

**Strengths:** Addresses the bidding phase; opponent modeling improves play quality; coach network accelerates training.
**Weaknesses:** Added complexity over base DouZero; opponent model quality depends on training distribution.

### OADMCDou (IJCAI 2024)

**Method:** Oracle Guiding + Adaptive Deep Monte Carlo.

**Key innovations:**
1. **Oracle Guiding:** trains an agent with both perfect and imperfect information, then gradually reduces reliance on perfect information to transition to a standard agent (curriculum-based distillation)
2. **Adaptive DMC:** introduces gradient weight clipping to prevent extreme policy updates, stabilizing training
3. **Minimum Combination Search (MCS):** dynamic programming-based heuristic that prunes suboptimal actions by computing the minimum number of combinations needed to empty a hand

**Results:** Outperforms DouZero with a 95% confidence interval of 0.104 +/- 0.041 and a 28.6% reduction in loss.

**Strengths:** Principled oracle-to-standard transition; action pruning via MCS reduces effective action space.
**Weaknesses:** MCS is a heuristic that may prune viable actions in edge cases.

### AlphaDou (2024)

**Method:** End-to-end RL integrating bidding and card play.

**Key innovations:**
- Modified DMC framework that simultaneously estimates win rates and expected values
- Action pruning based on expectations; strategy selection based on win rates
- Full bidding phase integration (determining whether to bid and at what level)

**Results:** Win rate of 0.6167 against DouZero in bidding environments; 0.5970 win rate even in non-bidding test environments. State-of-the-art RL model as of mid-2024.

**Strengths:** First complete end-to-end system handling both bidding and play; flexible strategy generation.
**Weaknesses:** Performance gap between bidding-trained and non-bidding test environments suggests some overfitting to bidding dynamics.

### FPDou (2025)

**Method:** Generalized Weakened Fictitious Play (GWFP) with perfect-training-imperfect-execution.

**Key innovations:**
1. **Two-player reduction:** treats the two Peasants as a single player by sharing information during training, converting the three-player game to a two-player zero-sum game where GWFP has convergence guarantees
2. **Regularization term:** penalizes policy discrepancy between perfect and imperfect information views, mitigating the training-execution gap
3. **Unified learning step:** consolidates RL and supervised learning into a single update, eliminating the need for separate networks
4. **Alternating on-policy/off-policy updates:** preserves stationarity for best-response learning

**Results:** New state of the art with a 3x smaller model than competitors.

**Strengths:** Theoretically grounded in fictitious play; smallest model size; addresses non-stationarity explicitly.
**Weaknesses:** Two-player reduction assumes perfect peasant cooperation, which may not hold in practice against non-cooperative peasant agents.

### DouRN (2024)

**Method:** DouZero with residual neural networks replacing MLPs.

**Key innovations:** Adopts ResNet architecture to address the degradation problem of deep neural networks, enabling faster convergence. Adds a call scoring system for landlord bidding decisions.

**Results:** Significantly improved win rate within the same training time compared to DouZero.

**Strengths:** Drop-in architectural improvement; minimal added complexity.
**Weaknesses:** Incremental rather than fundamental advance.

---

## Landlord vs Peasant Strategy

### Landlord-Specific Optimization

The Landlord faces a fundamentally different strategic landscape:

**Advantages:**
- 3 extra cards from the kitty (20 vs 17), providing more combination options
- First move advantage (controls initial tempo)
- Kitty cards are seen by all, but the Landlord chose to bid knowing their hand quality

**Challenges:**
- Must defeat two coordinated opponents simultaneously
- If either Peasant empties their hand, the Landlord loses
- Bomb/rocket usage doubles stakes---high risk when losing means paying both Peasants

**AI approach:** All major systems train separate Landlord networks. The Landlord network tends to learn aggressive tempo-control strategies: playing dominant combination types early (e.g., leading with long sequences if holding several), using bombs to recapture initiative at critical moments, and planning endgame sequences that empty the hand in 1--2 final plays.

### Peasant-Specific Optimization

**Cooperation challenge:** The two Peasants cannot communicate but must cooperate. This is modeled differently across systems:

- **DouZero:** trains separate networks for Peasant-upstream and Peasant-downstream, learning implicit cooperation through self-play
- **PerfectDou:** during training, the critic sees both Peasants' hands, distilling cooperative knowledge into individual actors
- **JP-DouZero:** explicitly models the peasant coalition through a joint Q-network that evaluates state-action pairs from the collective peasant perspective
- **FPDou:** treats both Peasants as a single player during training (information sharing), then separates them at inference with a regularization penalty

**Cooperative patterns learned by AI systems:**
1. **Passing to partner:** recognizing when the partner is closer to winning and passing to let them lead
2. **Breaking Landlord leads:** even with weak cards, playing to prevent the Landlord from controlling tempo
3. **Signal plays:** leading with specific ranks to implicitly communicate hand structure to the partner
4. **Sacrifice plays:** using individually suboptimal combinations to enable the partner's endgame

### Bidding Strategy

The bidding phase determines who becomes Landlord (bidding 1, 2, or 3) and sets the base stake multiplier. Most early systems (including DouZero) skip bidding entirely, randomly assigning the Landlord role. Recent systems address this:

- **Full DouZero+:** trains a bidding network via Monte Carlo simulation, evaluating expected returns for each bid level
- **AlphaDou:** integrates bidding into the end-to-end RL framework, simultaneously learning when to bid and how to play
- **DouRN:** adds a call scoring system for bid/pass decisions

Key factors in optimal bidding: hand strength (bombs, rockets, long sequences), card connectivity (how many clean combinations the hand decomposes into), presence of high singles (2s, jokers for tempo control), and risk tolerance given the stake multiplier.

---

## Card Decomposition

### The Decomposition Problem

A central challenge unique to shedding games like Dou Di Zhu: given a hand of cards, what is the optimal way to partition them into playable combinations? The same hand can be decomposed many ways, and the choice of decomposition fundamentally constrains the available play sequence.

**Example:** Holding 3-3-3-4-4-4-5-5 could be decomposed as:
- Airplane (3-3-3-4-4-4) + Pair (5-5) = 2 plays
- Triple+Pair (3-3-3-5-5) + Triple (4-4-4) = 2 plays
- Triple+Single (3-3-3-4) + Triple+Single (4-4-4-5) + Single (5) = 3 plays

The first decomposition is strictly more efficient, but such optimization is NP-hard in general and becomes critical for endgame planning.

### Algorithmic Approaches

**Minimum Combination Search (MCS):** A dynamic programming heuristic used in OADMCDou that computes the minimum number of plays needed to empty a hand. MCS evaluates all possible first moves, recursively computes the minimum for the remainder, and returns the globally optimal decomposition. This is computationally feasible because hand sizes are bounded (max 20 cards) and the combination vocabulary, while large, is enumerable.

**Combinational Q-Learning (CQL):** Learns decomposition quality via RL rather than computing it analytically. The DPN stage selects among possible decompositions, while the MPN stage selects the best move within that decomposition. This learned decomposition may outperform MCS because it considers opponent dynamics, not just hand efficiency.

**Neural implicit decomposition:** Systems like DouZero and PerfectDou do not explicitly decompose the hand. Instead, the neural network implicitly learns decomposition quality through the Q-value or policy outputs. By evaluating all legal actions at each turn, the network effectively learns which plays preserve favorable hand structures for future turns.

### Combination Enumeration

Enumerating legal actions from a hand requires checking all 14 combination types with their constraints:
- Sequences exclude 2s and jokers, range 3--A only
- Pair sequences require 3+ consecutive pairs
- Airplanes require 2+ consecutive triples
- Kicker attachments must be distinct ranks
- A bomb (four-of-a-kind) is always legal regardless of the current lead type

Efficient enumeration algorithms typically iterate over the card count vector (15 positions, values 0--4) and pattern-match against each combination template. DouZero's action encoding pre-enumerates all 27,472 possible combinations and indexes them, making action selection a lookup followed by a legality check.

---

## Open Problems

### 1. Principled Peasant Cooperation

No current system has a theoretically grounded solution for peasant cooperation in the imperfect-information setting. FPDou's two-player reduction assumes perfect cooperation during training but cannot guarantee it at inference. JP-DouZero's joint Q-network helps but still relies on independent execution. The gap between cooperative training and decentralized execution remains the deepest open problem.

### 2. Exploitability Measurement

In two-player zero-sum games, exploitability is well-defined (distance from Nash equilibrium). In three-player games with team structure, there is no clean exploitability metric. How do we measure whether a Dou Di Zhu AI is "optimal" when the solution concept itself is unclear? Current evaluation is purely empirical (win rates against baselines).

### 3. Integrated Bidding and Play

Most SOTA systems either skip bidding or treat it as a separate module. True end-to-end optimization requires jointly learning when to bid, how high to bid, and how to play the resulting hand. AlphaDou and Full DouZero+ address this but performance gaps between bidding and non-bidding environments suggest incomplete integration.

### 4. Four-Player and Variant Generalization

The four-player variant uses a double deck (108 cards, 25 cards per player, 33 for landlord), variable-size bombs, and four jokers. No published work addresses this variant at scale, despite its popularity. The explosion in state/action space makes it significantly harder than the three-player game.

### 5. Real-Time Search Integration

DeltaDou (2019) and AP-MCTS (2021) showed that search improves play quality, but at significant computational cost. PerfectDou and FPDou achieve strong results without search. The question of whether search can be made efficient enough to complement fast policy networks at inference---as in AlphaGo/AlphaZero for perfect-information games---remains open.

### 6. Sample Efficiency

While PerfectDou and FPDou have improved sample efficiency, training still requires hundreds of millions to billions of games. Whether techniques from model-based RL, world models, or offline RL can reduce data requirements for Dou Di Zhu is unexplored.

### 7. Transfer Across Shedding Games

Dou Di Zhu, Tien Len (Vietnamese), Pusoy Dos (Filipino), and Big Two (Cantonese) share the climbing/shedding game structure with overlapping combination types. Whether representations learned on one game transfer to others could reduce training costs and inform a general shedding-game solver.

---

## Relevance to Myosu

### Solver Architecture Implications

| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 1/5 | Three-player asymmetric structure breaks CFR guarantees |
| Neural value network potential | 5/5 | Proven essential by DouZero, PerfectDou, FPDou |
| Action abstraction necessity | 2/5 | Card matrix encoding handles the space without abstraction |
| Real-time search value | 3/5 | Improves play but adds latency; pure policy is competitive |
| Transferability to other games | 4/5 | Shedding game techniques transfer to Tien Len, Big Two, Pusoy Dos |

### Subnet Design Considerations

1. **Combination validation oracle:** The game interface must correctly identify and validate all 14 combination types, including edge cases in airplane+wings kicker selection. This is the most complex validation logic of any Myosu game.

2. **Action encoding standardization:** The subnet should adopt DouZero's 4x15 (or 15x4) card matrix encoding as the standard representation, enabling solvers to share infrastructure.

3. **Asymmetric evaluation:** Solver submissions must be evaluated separately as Landlord and as both Peasant positions. A strong Landlord model may be a weak Peasant model and vice versa. The scoring function should weight all three positions.

4. **Bidding inclusion:** The full game includes bidding, which fundamentally changes strategy and expected values. The subnet should require solvers to handle the complete game flow.

5. **Baseline benchmark:** DouZero is open-source and provides a natural baseline. Solver submissions should demonstrate improvement over DouZero to be considered competitive.

6. **Compute constraints:** DouZero achieves SOTA with 4 GPUs in 10 days. Myosu's solver evaluation should set practical compute budgets that allow competitive submissions without requiring industrial-scale resources.

### Recommended Approach

The proven path for Myosu's Dou Di Zhu subnet:

1. **Primary method:** Deep Monte Carlo (DMC) or PPO-based actor-critic, following the DouZero/PerfectDou lineage
2. **Architecture:** Position-specific networks with LSTM history encoding and card matrix action representation
3. **Training paradigm:** Perfect-information distillation (PerfectDou-style) for sample efficiency, or fictitious play reduction (FPDou-style) for theoretical grounding
4. **Bidding:** Integrated end-to-end as in AlphaDou, or Monte Carlo simulation as in Full DouZero+
5. **Evaluation:** Separate Landlord and Peasant win rates against DouZero baseline, across thousands of games with random deals

---

## Key Papers & References

| Year | Paper | Venue | Contribution |
|------|-------|-------|--------------|
| 2011 | Whitehouse et al., "Determinization and ISMCTS for Dou Di Zhu" | IEEE CIG | First MCTS approach for DDZ |
| 2019 | You et al., "Combinational Q-Learning for Dou Di Zhu" | AAAI/arXiv | Two-stage DQN for combinatorial actions |
| 2019 | Jiang et al., "DeltaDou: Expert-level Doudizhu AI through Self-play" | IJCAI | FPMCTS + policy-value network, first expert-level |
| 2020 | Li et al., "Strategy of Playing Doudizhu Based on Multirole Modeling" | Complexity | Role-specific CNN strategies |
| 2021 | Zha et al., "DouZero: Mastering DouDizhu with Self-Play Deep RL" | ICML | Deep Monte Carlo, superhuman, open-source |
| 2021 | Zhang et al., "Combining Tree Search and Action Prediction for SOTA in DDZ" | IJCAI | AP-MCTS, 65.65% vs humans |
| 2022 | Zhao et al., "DouZero+: Improving DouDizhu AI by Opponent Modeling" | IEEE CoG | Opponent model + coach network |
| 2022 | Guan et al., "PerfectDou: Dominating DouDizhu with Perfect Info Distillation" | NeurIPS | Perfect-training-imperfect-execution, best sample efficiency |
| 2022 | NV-Dou, "Q-based Policy Gradient Optimization for Doudizhu" | Applied Intelligence | Mean actor-critic + neural fictitious self-play |
| 2023 | Zhao et al., "Full DouZero+: Opponent Modeling, Coach, and Bidding" | IEEE TGCGT | Bidding network via MC simulation |
| 2024 | Li et al., "Enhanced DDZ Strategy: Oracle Guiding + Adaptive DMC" (OADMCDou) | IJCAI | Oracle curriculum + gradient clipping + MCS pruning |
| 2024 | Liu et al., "AlphaDou: High-Performance End-to-End DDZ AI Integrating Bidding" | arXiv | End-to-end bidding + play via modified DMC |
| 2024 | "Improved Learning Efficiency of DMC for Imperfect-Info Card Games" | Applied Soft Computing | Opponent model + MCS, 25.5% training time |
| 2024 | DouRN, "Improving DouZero by Residual Neural Networks" | CyberC/arXiv | ResNet architecture, call scoring |
| 2024 | JP-DouZero, "Enhanced DDZ AI with Peasant Collaboration" | IEEE | Joint peasant Q-network + intrinsic rewards |
| 2025 | FPDou, "Mastering DouDizhu with Fictitious Play" | OpenReview | GWFP, two-player reduction, 3x smaller model, new SOTA |

### Frameworks and Tools

- **RLCard** (rlcard.org): Open-source toolkit for RL in card games, includes Dou Di Zhu environment (both full and simplified variants). Provides DQN, NFSP, and CFR baselines.
- **Botzone** (botzone.org): Online AI competition platform with Dou Di Zhu ("FightTheLandlord") as a featured game. Elo-based ranking system with 400+ registered agents.
- **DouZero codebase** (github.com/kwai/DouZero): Reference implementation of DMC for Dou Di Zhu. The de facto starting point for new research.
- **PerfectDou codebase** (github.com/Netease-Games-AI-Lab-Guangzhou/PerfectDou): Reference implementation of perfect information distillation.
