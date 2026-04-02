# Solving Strategies for Pusoy Dos (Big Two)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods for Big Two / Pusoy Dos and the climbing card game family, 2011--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Pusoy Dos (Big Two / 大老二) is a four-player climbing/shedding card game in which each player receives 13 cards and races to empty their hand by playing increasingly powerful card combinations. Despite being one of the most popular card games in Southeast Asia and Chinese-speaking regions, Big Two has received far less academic attention than poker or even its cousin Dou Di Zhu (斗地主). The game's combination of four-player dynamics, imperfect information (39 hidden cards), a combinatorial action space (~1,695 possible plays from a 13-card hand), and the strategic centrality of "control" (winning a round to lead the next) makes it a genuinely hard AI challenge that resists clean theoretical analysis.

The most significant published work directly targeting Big Two is Charlesworth (2018), which trained a Proximal Policy Optimization (PPO) agent via self-play to superhuman amateur level without any tree search. More recently, the Taiwanese research group behind Big2AI (Li et al. 2022) and Big2MDP (Li et al. 2024) developed MCTS/ISMCTS-based and MDP-based frameworks incorporating multi-opponent prediction, card superiority analysis, and free-play-right exploration. A 2024 paper in Applied Soft Computing (the ODMC framework) explicitly tested improved Deep Monte Carlo methods on Big Two alongside Dou Di Zhu, achieving comparable performance to standard DMC at 25% of the training cost by combining Minimum Combination Search (MCS) with an advanced opponent model. The dominant paradigm, however, remains transfer from Dou Di Zhu research: DouZero (ICML 2021), PerfectDou (NeurIPS 2022), DeltaDou, AlphaDou (2024), and the Combinatorial Q-Learning framework all address the same climbing-game action space problem and their techniques apply almost directly.

Open problems remain substantial. No published agent has been shown to reach expert human level in four-player Big Two. CFR-based methods lack convergence guarantees for four-player games. Optimal hand decomposition (partitioning 13 cards into playable groups) remains an NP-hard combinatorial subproblem that is typically handled heuristically or via learned networks. Endgame solving -- exact play when few cards remain and information becomes near-complete -- is tractable but has not been systematically integrated with a learning-based approach for Big Two. The climbing game family (Big Two, Dou Di Zhu, Tien Len, Guandan) shares enough structure that a unified shedding-game framework would yield significant research and engineering leverage.

---

## Game Complexity Analysis

### Structural Properties

| Property | Value |
|----------|-------|
| Players | 4 (standard) |
| Cards per player | 13 (full 52-card deck, no jokers) |
| Information type | Imperfect (39 hidden cards) |
| Stochasticity | Initial deal only |
| Sum type | Zero-sum under penalty scoring |
| Combination types | Singles, pairs, triples, five-card poker hands |
| Five-card hierarchy | Straight < Flush < Full House < Four-of-a-Kind < Straight Flush |
| Suit hierarchy | Diamonds > Hearts > Spades > Clubs (Filipino rules) |
| Highest card | 2 of Diamonds |

### Complexity Metrics

| Metric | Estimate | Notes |
|--------|----------|-------|
| Possible deals | C(52,13) x C(39,13) x C(26,13) ~ 5.36 x 10^28 | Each deal is a unique game instance |
| Distinct 13-card hands | C(52,13) ~ 6.35 x 10^11 | Per player |
| Total action space | ~1,695 distinct plays | From a full 13-card hand (Charlesworth 2018) |
| Legal actions per turn | ~5--30 | Depends on lead type and hand composition |
| Average game length | ~20--40 plays per player | Empirical |
| Game tree nodes | ~10^25 -- 10^40 | Rough estimate |
| Information sets | ~10^20 -- 10^30 | 39 hidden cards create massive uncertainty |
| Branching factor | ~5--30 | Heavily context-dependent |

### Why Big Two Is Hard

**1. Four-player imperfect information.** With 39 cards hidden at the start, the belief space over opponent hands is enormous. Unlike heads-up poker where one opponent holds 2 cards from a known deck, Big Two requires reasoning about three opponents each holding 13 cards. As play progresses, information accumulates (played cards are public), but the inference problem remains intractable for exact methods.

**2. Combinatorial action space.** A 13-card hand can generate hundreds of legal plays. The player must choose not only which combination type to play (single, pair, triple, five-card) but which specific cards to include. Different card selections for the same combination type leave different residual hands, creating a tree of consequences that cascades through the rest of the game. Charlesworth counted 1,694 possible non-pass actions; DouZero's Dou Di Zhu environment enumerates 27,472 possible card combinations (Dou Di Zhu has additional combination types like chains and rockets).

**3. Hand decomposition interdependence.** The central strategic problem is decomposing 13 cards into a sequence of playable combinations. Breaking a potential straight to use one card in a pair changes the entire game plan. This decomposition problem is NP-hard in the general case and interacts with game dynamics: the optimal decomposition depends on what opponents play, which is unknown.

**4. Control (initiative) dynamics.** Winning a round grants "free lead" -- the ability to choose any combination type. This is the primary strategic resource in the game. Planning multiple rounds ahead to maintain control (e.g., leading with a type where you hold the highest card, then winning control back with a 2) requires lookahead that interacts with the decomposition problem.

**5. Suit-aware tiebreaking.** Unlike Dou Di Zhu (where suits are irrelevant), Big Two uses suit rankings to break ties at every level: singles, pairs, and flushes. This makes the effective action space larger and prevents the aggressive abstraction that works for DDZ.

**6. No natural subgame decomposition.** Poker hands decompose into betting rounds with clean information boundaries. Big Two rounds do not decompose as cleanly: a player's decision in round 3 depends on what cards they expect to need in rounds 7--10, and the control dynamics create dependencies across the entire game.

---

## Current Best Approaches

### 1. Self-Play Deep RL with PPO (Charlesworth 2018)

**Paper:** "Application of Self-Play Reinforcement Learning to a Four-Player Game of Imperfect Information" (arXiv:1808.10442)

**Architecture.** A feedforward neural network with a shared 512-unit ReLU hidden layer branching into two 256-unit heads: a value head (scalar state value estimate) and a policy head (1,695-dimensional output over all possible plays including pass). The state representation encodes: the player's hand as 13 binary card indicators with suit encoding, which cards can form valid combinations, cards already played, and the current combination on the table.

**Training.** Pure self-play using PPO with four copies of the same agent playing against each other. Training was conducted on a single machine and achieved super-amateur-level play in days. No tree search at inference time -- the network outputs a policy directly.

**Strengths.**
- Conceptually simple; no hand-crafted heuristics or domain knowledge beyond the game rules.
- Demonstrates that end-to-end deep RL can learn the game from scratch.
- Fast inference (single forward pass).
- Open-source implementation available (TensorFlow).

**Weaknesses.**
- No opponent modeling or belief tracking -- treats all three opponents identically.
- The 1,695-dimensional flat action space does not decompose the combination selection problem, limiting scalability to richer action spaces.
- No endgame solver integration.
- Evaluated against amateur humans only; no benchmark against expert play.
- Single-agent self-play may not converge to robust strategies in 4-player settings (no Nash equilibrium guarantees).

**Compute.** Single machine with GPU, days of training.

### 2. Big2AI: MCTS + ISMCTS with Multi-Opponent Prediction (Li et al. 2022)

**Paper:** "Challenging Artificial Intelligence with Multi-Opponent and Multi-Movement Prediction for the Card Game Big2" (IEEE Xplore, 2022)

**Architecture.** A multi-component framework combining: (a) card superiority analysis (evaluating hand strength), (b) dynamic weight adjustment for play decisions, (c) game feature learning from historical data, and (d) multi-opponent movement prediction using both standard MCTS and Information Set MCTS (ISMCTS).

**ISMCTS component.** Instead of determinizing the game (sampling a specific assignment of hidden cards and solving the resulting perfect-information game), ISMCTS builds a single tree over information sets. Each node represents what the acting player knows, not the exact game state. This avoids the strategy fusion problem inherent in determinization-based approaches, where different determinizations suggest different actions for the same information state.

**Strengths.**
- First published framework combining ISMCTS with opponent modeling specifically for Big Two.
- Self-play capability with multiple AI personalities.
- Practical Android implementation demonstrating real-time play.
- Outperformed existing commercial Big Two AI opponents.

**Weaknesses.**
- MCTS depth is limited by the branching factor and hidden information; shallow searches may miss long-term strategic plans.
- Opponent model is heuristic rather than learned.
- Not compared against deep RL baselines (DouZero, PPO).

### 3. Big2MDP: Markov Decision Process Framework (Li et al. 2024)

**Paper:** "Markov Decision Process-Based Artificial Intelligence With Card-Playing Strategy and Free-Playing Right Exploration for Four-Player Card Game Big2" (IEEE, 2024)

**Architecture.** Models Big Two as an MDP where states encode the current hand, played cards, and game phase. The key innovations are: (a) simultaneous optimization of scoring (winning) and penalty minimization (reducing remaining cards when losing), and (b) explicit modeling of "free-playing right" -- the strategic value of winning control to choose the next combination type.

**Free-playing right exploration.** When a player has control, they can lead with any combination type. Big2MDP evaluates the strategic value of each lead type by predicting how opponents will respond and whether the player can maintain control through subsequent rounds. This addresses a core strategic element that flat RL approaches handle implicitly.

**Strengths.**
- First Big Two AI to explicitly model the scoring/penalty tradeoff.
- Addresses the free-play-right problem directly rather than leaving it to end-to-end learning.
- Outperforms Big2AI and other baselines in both win rate and penalty minimization.
- Android implementation with human testing.

**Weaknesses.**
- MDP formulation requires state discretization that may lose information.
- Not benchmarked against modern deep RL approaches.
- Limited to heuristic opponent models.

### 4. ODMC: Optimized Deep Monte Carlo with Minimum Combination Search (2024)

**Paper:** "Improved learning efficiency of deep Monte-Carlo for complex imperfect-information card games" (Applied Soft Computing, 2024)

**Architecture.** Extends the Deep Monte Carlo (DMC) framework from DouZero to both Dou Di Zhu and Big Two. Two key innovations: (a) an advanced opponent model that predicts not just card counts but the minimum composition of opponents' hands including suit information, and (b) Optimized DMC (ODMC) which uses Minimum Combination Search to filter suboptimal actions before neural network evaluation.

**Minimum Combination Search (MCS).** MCS is a dynamic-programming-based heuristic that computes the minimum number of plays needed to empty a hand. By pruning actions that lead to provably worse decompositions (more remaining plays), MCS reduces the effective action space that the neural network must evaluate. This is particularly effective in Big Two where suit information matters for determining which actions are dominated.

**Opponent model.** The advanced opponent model predicts the minimum composition of each opponent's remaining hand, including suit distribution. This is the first time such granular opponent modeling has been applied to Big Two, enabling the agent to reason about which specific cards opponents might hold.

**Results.** Achieves comparable performance to standard DMC with only 25.5% of the training time. This efficiency gain is critical for Big Two, where the four-player structure makes training more expensive than three-player DDZ.

**Strengths.**
- Directly addresses Big Two (not just DDZ transfer).
- Significant training efficiency improvement via action pruning.
- Sophisticated opponent model with suit-level hand prediction.
- Combines well-understood DMC foundations with targeted improvements.

**Weaknesses.**
- MCS is a heuristic; it may prune actions that are strategically valuable despite being suboptimal in isolation.
- Opponent model accuracy degrades early in the game when little information is available.

### 5. Transfer from Dou Di Zhu Research

The majority of applicable techniques come from the DDZ research pipeline, where significantly more effort has been invested. Key systems:

**DouZero (Zha et al., ICML 2021).** Enhanced Deep Monte Carlo with neural networks, action encoding, and parallel actors. Trained from scratch on a single server with 4 GPUs, reaching #1 on the Botzone DDZ leaderboard among 344 agents. The action encoding scheme compresses DDZ's 27,472 possible plays into a tractable representation. For Big Two, the same DMC + action encoding paradigm applies with modified combination rules.

**PerfectDou (Guan et al., NeurIPS 2022).** Introduces "perfect information distillation" -- training the policy network using a teacher that has access to all players' cards (oracle), then running the trained policy with imperfect information at test time. This perfect-training-imperfect-execution framework achieved state-of-the-art DDZ performance with an order of magnitude fewer training samples than DouZero. Directly applicable to Big Two: train with full hand visibility, deploy with hidden information.

**DeltaDou (Jiang et al., 2019).** Combines asymmetric MCTS on information sets, policy-value networks, Bayesian inference on hidden hands, and a pre-trained kicker network for action abstraction. The Bayesian inference component is particularly transferable to Big Two, where inferring opponent hand composition from observed play is critical.

**AlphaDou (Li et al., 2024).** End-to-end DDZ AI integrating the bidding phase. Uses a one-hot 4x13 card matrix for state encoding (13 columns for ranks 3--2, 4 rows for suits). While Big Two lacks a bidding phase, the card encoding and the variance-reduction techniques for training are directly transferable.

**DouZero+ (Zhao et al., 2023).** Adds opponent modeling and coach-guided learning to DouZero. The coach provides offline demonstrations to accelerate early training, while the opponent model adapts the policy to different play styles. Both techniques apply to Big Two.

**Combinatorial Q-Learning (You et al., 2019).** Addresses the action space explosion via a two-stage architecture: a Decomposition Proposal Network (DPN) selects a hand decomposition, then a Move Proposal Network (MPN) selects the specific play within that decomposition. Uses order-invariant max-pooling to extract relationships between cards. This two-stage approach is arguably more important for Big Two than DDZ because Big Two's suit-awareness creates a richer decomposition space.

---

## Hand Decomposition

### The Core Problem

Given 13 cards, partition them into a sequence of playable combinations (singles, pairs, triples, five-card hands) that can plausibly be played out to empty the hand. The quality of a decomposition depends on:

1. **Number of plays required** -- fewer plays means faster exit.
2. **Control potential** -- does the decomposition include combinations that can win rounds (high pairs, straights containing 2s, etc.)?
3. **Flexibility** -- does the decomposition leave options for adapting to opponent play?
4. **Orphan avoidance** -- does the decomposition leave isolated low cards that can never win a round?

### Algorithmic Approaches

**Dancing Links (Knuth's Algorithm X).** Card decomposition can be formulated as an exact cover problem: select a set of valid combinations such that every card appears in exactly one combination. The dancing links technique reduces computation from seconds to milliseconds for 13-card hands, making real-time enumeration of all valid decompositions feasible. This was demonstrated for DDZ hand decomposition and applies directly to Big Two with modified combination definitions.

**Minimum Combination Search (MCS).** A dynamic-programming heuristic that computes the minimum number of plays needed to empty a hand. MCS iterates over combination types in priority order (five-card hands first, then triples, pairs, singles) and greedily assigns cards to the highest-value combinations. While not globally optimal, MCS runs in near-linear time and provides a useful lower bound and pruning criterion.

**Learned Decomposition (DPN from Combinatorial Q-Learning).** Rather than enumerating decompositions algorithmically, the DPN learns to score decomposition proposals using Q-values. Given a hand, the DPN evaluates candidate decompositions and selects the one with the highest expected game outcome. This captures strategic considerations (e.g., sacrificing a strong five-card hand to preserve a high pair for control) that algorithmic methods miss.

**Suit-Aware Decomposition for Big Two.** Unlike DDZ, Big Two's suit rankings mean that decomposition must consider suits. Two kings of different suits are not interchangeable in a pair -- the pair's rank depends on the higher suit. A flush decomposition must select five cards of the same suit, which may conflict with straight potential. This suit-awareness significantly expands the decomposition search space compared to DDZ and makes Big Two's decomposition problem harder.

### Complexity

The number of valid decompositions of a 13-card Big Two hand is typically in the range of 10^2 to 10^4. Exhaustive enumeration is feasible via dancing links in under 10ms, but evaluating each decomposition's strategic quality requires game-theoretic reasoning that is not tractable exactly. Practical systems either use heuristic scoring (MCS + hand strength evaluation) or learned scoring (DPN).

---

## Climbing Game Family: Shared Techniques

### Family Members

| Game | Players | Suits Matter? | Key Differences from Big Two |
|------|---------|---------------|------------------------------|
| **Dou Di Zhu** (斗地主) | 3 (1 vs 2) | No | Asymmetric teams; bombs; chains of pairs/triples; rockets |
| **Tien Len** (Tiến lên) | 4 | Yes | No five-card poker hands; 2 is high; instant-win sequences |
| **Guandan** (掼蛋) | 4 (2v2 teams) | Limited | Team cooperation; wild cards; multi-round scoring |
| **Zheng Shangyou** (争上游) | 4 | No | Simpler combination rules; precursor to DDZ |
| **Pusoy Dos** | 4 | Yes (Filipino suit order) | Five-card poker hand hierarchy; suit-based tiebreaking |

### Transferable Techniques

**Deep Monte Carlo (DMC).** The core DouZero paradigm -- simulate full games to completion using current policy, train value/policy networks on collected trajectories -- transfers directly across the family. The game-specific components are: (a) the legal action generator, (b) the state representation, and (c) the reward function. Everything else (parallel actors, experience replay, network architecture) is game-agnostic.

**Action encoding.** All climbing games share the problem of mapping a variable-size set of legal card combinations to a fixed neural network input/output. DouZero's encoding scheme (one-hot matrices of card presence/absence, with separate channels for hand, played cards, and combination type) provides the template. Big Two and Tien Len require additional suit channels that DDZ omits.

**Perfect information distillation.** PerfectDou's perfect-training-imperfect-execution framework is game-agnostic and particularly valuable for the climbing family, where the imperfect information structure (hidden hands that gradually reveal through play) is shared across all members.

**Opponent modeling.** DouZero+'s opponent modeling and ODMC's minimum-composition prediction both address the fundamental problem of inferring hidden hands from observed play. The inference structure is similar across the family: when a player passes, they cannot beat the current combination, which constrains their possible hand.

**Endgame solving.** When players have few remaining cards (e.g., 3--5 each), the game becomes near-perfect-information: card counting reveals most of what opponents hold. At this point, exact minimax search is feasible and provably optimal. This endgame solver pattern applies across the entire family.

### Key Differences Affecting Transfer

**DDZ to Big Two.** DDZ is 3-player with asymmetric roles (landlord vs peasants). Big Two is 4-player symmetric. This means: (a) DDZ's role-conditioned networks (separate networks for landlord and peasant) do not directly apply; (b) Big Two has no natural cooperation structure, so multi-agent cooperation techniques from DDZ are irrelevant; (c) the 4-player setting increases the hidden information (39 hidden cards vs 34 in DDZ, given DDZ's 54-card deck with jokers dealt 20+17+17).

**DDZ to Big Two: suits.** DDZ ignores suits entirely. Big Two's suit hierarchy creates a larger effective action space and richer decomposition space. Any DDZ architecture transferred to Big Two must add suit-aware state representation.

**Tien Len to Big Two.** Tien Len lacks five-card poker hands, making its action space smaller and decomposition simpler. However, Tien Len's four-player symmetric structure and suit-awareness make it a closer structural match to Big Two than DDZ.

**Guandan to Big Two.** Guandan's team structure (2v2) introduces cooperative dynamics absent from Big Two. However, Guandan research (DanZero, GuanZero, OpenGuanDan) has produced multi-agent training infrastructure and four-player climbing-game environments that could be adapted.

---

## Equilibrium Concepts and Theoretical Limitations

### CFR in Four-Player Games

Counterfactual Regret Minimization (CFR) is the gold standard for two-player zero-sum imperfect-information games, with proven convergence to epsilon-Nash equilibrium. For games with three or more players, CFR loses all theoretical convergence guarantees. Empirical results on 3-player poker show that CFR-generated strategies can still perform well (winning the 2009 AAAI Computer Poker Competition 3-player event), but there is no theoretical foundation for this performance.

For four-player Big Two specifically: (a) the game is not strictly zero-sum under placement scoring (third place receives more than fourth); (b) the number of information sets makes full CFR intractable; (c) even if tractable, convergence to a meaningful equilibrium is not guaranteed. CFR-based approaches are therefore not the natural choice for Big Two, though DeepCFR variants (Kdb-D2CFR, 2023) have shown some scalability to 3--8 player poker.

### Deep RL and Self-Play

Self-play deep RL (PPO, DMC, policy gradient methods) is the dominant paradigm for climbing games. These methods do not converge to Nash equilibria in general multiplayer settings, but empirically produce strong policies. The key risk is cyclic strategies: agent A beats agent B which beats agent C which beats agent A. Population-based training (PSRO, Pipeline PSRO) can mitigate this by maintaining a diverse population of strategies.

### Student of Games (2023)

The Student of Games algorithm (Schmid et al., Science Advances 2023) unifies search, self-play learning, and game-theoretic reasoning for both perfect and imperfect information games. It combines Growing-Tree CFR (GT-CFR) with learned counterfactual value-policy networks. While demonstrated on poker and Scotland Yard, SoG has not been applied to climbing games. Its unified framework could potentially address Big Two's mix of combinatorial planning (decomposition), imperfect information (hidden hands), and multiplayer dynamics, but the computational cost of GT-CFR search in the large action space would be a challenge.

---

## Open Problems

### 1. Expert-Level Four-Player Big Two AI

No published system has been demonstrated at expert human level in standard four-player Big Two. Charlesworth's PPO agent beat amateur players; Big2AI and Big2MDP beat commercial app-level AI. The gap to expert play likely requires: (a) larger-scale training (DouZero-level GPU-days), (b) better hand decomposition, and (c) integrated endgame solving.

### 2. Optimal Hand Decomposition Under Uncertainty

Computing the optimal hand decomposition is NP-hard even with perfect information about all hands. Under imperfect information, the optimal decomposition depends on opponent hands and future play, making it a planning problem under uncertainty. Current approaches use heuristics (MCS) or learned scorers (DPN), but neither is provably good. A principled approach combining combinatorial optimization with belief-state planning is an open research question.

### 3. Endgame Solving Integration

Endgame solving (exact play when sufficient information is available) has been shown to dramatically improve poker AI (Libratus, Pluribus) but has not been systematically applied to climbing games. The challenge is defining the "endgame" boundary: in poker, it is a clean betting round; in Big Two, the transition from imperfect to near-perfect information is gradual as cards are played and inferred. Determining when to switch from learned policy to exact search is itself a research problem.

### 4. Exploitability Measurement

In two-player zero-sum games, exploitability (the gap to Nash equilibrium) can be computed exactly. For four-player Big Two, no tractable exploitability metric exists. Current evaluation relies on win rates against specific opponents, which does not measure strategic robustness. Developing approximate exploitability measures for multiplayer climbing games is an open theoretical problem.

### 5. Rule Variant Robustness

Big Two has many regional variants: Filipino (Pusoy Dos) with Diamonds-high suit order, Taiwanese with different straight-wrapping rules, Hong Kong (Choh Dai Di) with different scoring. A robust AI should handle rule variations without retraining from scratch. Transfer learning or rule-conditioned networks could address this, but no published work has explored it.

### 6. LLM-Based Reasoning

Recent work (2025) has shown that LLMs trained on game data can approach teacher-model performance in DDZ, with chain-of-thought reasoning enabling acceptable strategic play. Whether LLMs can handle Big Two's suit-aware decomposition and control dynamics is untested. The emerging "Theory of Mind" approach from GuanDan research (LLM agents inferring opponents' beliefs and intentions) is a promising but unexplored direction for Big Two.

---

## Relevance to Myosu

### Architecture Ranking

| Factor | Score | Rationale |
|--------|-------|-----------|
| CFR applicability | 2/5 | 4-player; no convergence guarantees; action space too large |
| Deep RL (DMC/PPO) applicability | 5/5 | Proven paradigm for climbing games; direct DDZ transfer |
| Neural value network potential | 4/5 | Learnable from self-play; suits add representation complexity |
| Action abstraction necessity | 4/5 | 1,695 actions manageable but suit-awareness prevents trivial abstraction |
| Real-time search value | 4/5 | Endgame solver and ISMCTS both add value over pure policy |
| Transferability across climbing family | 5/5 | Shared infrastructure with DDZ (game 16) and Tien Len (game 18) |

### Recommended Myosu Architecture

1. **Deep Monte Carlo (DMC) as the base**, following the DouZero paradigm. Train via self-play with parallel actors. Use a 4x13 one-hot card matrix with suit channels as the core state representation.

2. **Minimum Combination Search (MCS) for action pruning.** Before neural network evaluation, prune dominated actions using MCS. This reduces training cost by ~75% (per ODMC results) and is especially valuable for Big Two's suit-expanded action space.

3. **Perfect information distillation for training efficiency.** Use PerfectDou's oracle-teacher framework: train a teacher network with full hand visibility, then distill to a student that operates with imperfect information. This reduces sample complexity by ~10x.

4. **Endgame solver for late-game precision.** When card counting reveals sufficient information about opponents' hands (e.g., all players have 5 or fewer cards), switch from the learned policy to exact minimax search. The branching factor drops to ~3--8 actions in late game, making exact search feasible in real time.

5. **Shared shedding-game framework** with Dou Di Zhu (game 16) and Tien Len (game 18). The game environment, training infrastructure, action encoding, and opponent modeling modules should be parameterized by game rules, not reimplemented per game. The suit-handling layer needed for Big Two and Tien Len is absent from DDZ and should be designed as a composable module.

### Evaluation Metrics

- **Win rate** in self-play tournaments (4-player, random seating).
- **Average remaining cards** when not finishing first (penalty minimization).
- **Exploitability proxy**: win rate against a diverse population of strategies including heuristic baselines, PPO agents, and MCTS agents.
- **Endgame accuracy**: percentage of endgame positions solved optimally (verifiable via exhaustive search).

### Subnet Considerations

- **Southeast Asian market**: Pusoy Dos is the primary card game in the Philippines. Regional popularity makes it a natural entry point for the Myosu subnet in SEA markets.
- **Rule specification**: The subnet must fix exact rules (Filipino suit order, straight-wrapping policy, scoring system) to enable deterministic verification.
- **Verification via endgame solving**: Perfect-information endgames provide a ground-truth oracle for strategy evaluation, enabling the subnet to verify solver quality without relying solely on win rates.

---

## Key Papers and References

| Year | Authors / System | Title | Venue | Contribution |
|------|-----------------|-------|-------|--------------|
| 2011 | Whitehouse, Powley, Cowling | Determinization and ISMCTS for Dou Di Zhu | IEEE CIG 2011 | First ISMCTS application to a climbing card game |
| 2012 | Cowling, Powley, Whitehouse | Information Set MCTS | IEEE TCIAIG 2012 | ISMCTS algorithm definition; avoids strategy fusion |
| 2018 | Charlesworth | Self-Play RL for a Four-Player Game of Imperfect Information | arXiv:1808.10442 | First deep RL agent for Big Two; PPO self-play |
| 2019 | Jiang et al. (DeltaDou) | DeltaDou: Expert-level Doudizhu AI through Self-play | Unpublished | MCTS + Bayesian inference + kicker network for DDZ |
| 2019 | You et al. | Combinational Q-Learning for Dou Di Zhu | AAAI AIIDE 2019 / arXiv:1901.08925 | Two-stage DPN+MPN architecture for action decomposition |
| 2021 | Zha et al. (DouZero) | Mastering DouDizhu with Self-Play Deep RL | ICML 2021 | DMC + action encoding; #1 on Botzone (344 agents) |
| 2022 | Guan et al. (PerfectDou) | Dominating DouDizhu with Perfect Information Distillation | NeurIPS 2022 | Oracle-teacher distillation; 10x fewer samples than DouZero |
| 2022 | Li et al. (Big2AI) | Challenging AI with Multi-Opponent Prediction for Big2 | IEEE 2022 | First ISMCTS + opponent modeling framework for Big Two |
| 2022 | Lu et al. (DanZero) | Mastering GuanDan Game with RL | arXiv:2210.17087 | DMC for 4-player team climbing game |
| 2023 | Zhao et al. (DouZero+) | Improving DouDizhu AI by Opponent Modeling and Coach-guided Learning | arXiv:2204.02558 | Opponent modeling + coaching for DDZ |
| 2023 | Schmid et al. (Student of Games) | Unified learning algorithm for perfect and imperfect information games | Science Advances 2023 | GT-CFR + learned value networks; general-purpose |
| 2023 | Kdb-D2CFR | Solving Multiplayer IIGs with Knowledge Distillation-based DeepCFR | Knowledge-Based Systems 2023 | DeepCFR scaled to 3--8 player games |
| 2024 | OADMCDou | Enhanced DDZ Strategy Using Oracle Guiding and Adaptive DMC | IJCAI 2024 | Oracle guiding + gradient clipping for DDZ |
| 2024 | ODMC | Improved learning efficiency of DMC for imperfect-information card games | Applied Soft Computing 2024 | MCS + opponent model for Big Two and DDZ; 75% training reduction |
| 2024 | Li et al. (Big2MDP) | MDP-Based AI with Card-Playing Strategy for Big2 | IEEE 2024 | MDP + free-play-right exploration for Big Two |
| 2024 | Li et al. (AlphaDou) | High-Performance End-to-End Doudizhu AI Integrating Bidding | arXiv:2407.10279 | End-to-end DDZ with bidding; 4x13 card encoding |
| 2024 | Yanggong et al. (GuanZero) | Mastering Guandan with Deep RL and Behavior Regulating | arXiv:2402.13582 | Hybrid RL + behavior regularization for 4-player teams |
| 2025 | Various | LLM reasoning for card games (DDZ, Guandan) | Multiple venues | Chain-of-thought + Theory of Mind for card game agents |
| 2026 | OpenGuanDan | A Large-Scale Imperfect Information Game Benchmark | arXiv:2602.00676 | Open-source 4-player climbing game environment and baselines |
