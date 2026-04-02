# Solving Strategies for Hwatu Go-Stop (화투 고스톱)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods and AI research, 2012--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Hwatu Go-Stop (고스톱) is a three-player (standard), zero-sum, imperfect-information fishing card game played with the Korean Hwatu (화투) deck of 48 cards plus two bonus/joker cards. It is by far the most popular card game in South Korea, played by tens of millions during holidays and year-round as a gambling game. Unlike its Japanese sibling Koi-Koi (game 07 in this survey), Go-Stop is primarily a three-player game, features exponential score multipliers through its eponymous "Go" calls, and includes additional tactical mechanics (bombs, shakes, junk-stealing) that significantly expand the strategy space. The three-player default format places Go-Stop in the harder class of multiplayer imperfect-information games where Nash equilibrium is PPAD-hard to compute and CFR convergence guarantees do not hold.

Published academic research on Go-Stop AI is extremely sparse. The most significant work comes from NHN Corporation (operator of Korea's Hangame platform), whose AI team presented at NHN FORWARD 2020 on developing a Go-Stop AI using supervised learning from human game records followed by reinforcement learning through self-play. NHN focused on the 2-player variant Matgo (맞고) to reduce complexity, achieving an RL agent that outperformed the supervised learning baseline by +0.4 points per game over 10,000 games. A Korean patent (KR102628188B1, filed 2021 by NHN Cloud) describes a deep-learning-based system for incomplete information game service incorporating state determination, win rate estimation, score estimation, and yaku achievement probability training. Beyond NHN's industrial work, a handful of Korean university theses (Kim et al. 2012, Lee & Park 2016, Hwang et al. 2019) have explored heuristic and simplified RL approaches, but none have been published in international venues or achieved significant solver depth.

The gap between Go-Stop's cultural significance and its solver maturity is enormous. No Nash equilibrium has been computed for any variant. No CFR-based solver exists. No ISMCTS implementation has been benchmarked against human experts. The game shares the 48-card Hwatu/Hanafuda deck with Koi-Koi, enabling substantial code and infrastructure reuse, but the three-player dynamics, exponential Go multiplier, and richer tactical mechanics require fundamentally different solver strategies. Go-Stop represents a high-impact, underserved target where Myosu could establish first-mover dominance with relatively modest compute investment.

---

## Game Complexity Analysis

### Three-Player Dynamics

Go-Stop's standard three-player format creates game-theoretic challenges absent in two-player Koi-Koi:

1. **Non-zero-sum coalitional pressure.** When one player leads in captured cards, the other two have implicit incentive to deny that player key captures. This emergent "cooperation against the leader" does not arise from explicit communication but from the payment structure: all losers pay the winner, so preventing anyone from winning is strategically valuable.

2. **Asymmetric payment structure.** The winner collects from both losers independently. Penalty multipliers (Gwang-bak, Pi-bak) apply per-loser, meaning one loser might pay x1 and the other x4. This asymmetry means players must track not just their own score but the vulnerability of each opponent.

3. **Three-way information partitioning.** In a 3-player deal: 7 hand cards each (21 total), 6 field cards, 23 in the draw pile. Each player sees only 13 of 50 cards (own hand + field). The unknown space includes two opponents' hands (14 cards total) plus the draw pile (23 cards), a much larger hidden space than Koi-Koi's single-opponent format.

4. **No Nash equilibrium convergence guarantee.** CFR and its variants converge to Nash equilibrium only in two-player zero-sum games. In three-player settings, CFR may oscillate or converge to non-equilibrium fixed points. This is the fundamental theoretical obstacle.

### Bonus Scoring and Multiplier System

The scoring system creates highly nonlinear payoff landscapes:

| Mechanic | Effect | Strategic Impact |
|----------|--------|-----------------|
| Go calls (1--2) | +1, +2 points | Linear bonus; moderate risk |
| Go calls (3+) | Score doubled per Go beyond 2 | Exponential; creates extreme variance |
| Gwang-bak (광박) | Loser pays x2 if zero Brights | Defensive incentive to capture any Bright |
| Pi-bak (피박) | Loser pays x2 if <5 junk | Incentive to accumulate junk defensively |
| Shake (흔들기) | Winner's score x2 per shake | Declared pre-capture; reveals information |
| Multiplier stacking | Gwang-bak x Pi-bak x Shake | Can reach x8 or higher combined |

The exponential Go multiplier is the single most important strategic lever. A player at 3 points calling Go three times to reach 9 points receives (9 + 3) x 2 = 24 base payment from each loser, potentially multiplied further by penalty conditions. But if an opponent reaches 3 points and Stops during those three turns, the Go-caller loses and pays extra for each Go called. This creates a risk-reward profile more extreme than Koi-Koi's doubling at 7 points.

### State Space Estimates

| Metric | 3-Player Estimate | 2-Player (Matgo) Estimate |
|--------|-------------------|--------------------------|
| Initial deal combinations | C(50,7) x C(43,7) x C(36,7) x C(29,6) | C(50,10) x C(40,10) x C(30,8) |
| Deal space magnitude | ~10^18 | ~10^14 |
| Cards per hand (start) | 7 | 10 |
| Turns per player | 7 | 10 |
| Branching per turn | 1--7 (card) x 1--3 (match) + Go/Stop | 1--10 (card) x 1--3 (match) + Go/Stop |
| Game tree nodes | ~10^15--10^25 | ~10^15--10^20 |
| Information sets | ~10^12--10^18 | ~10^12--10^15 |
| Bonus card handling | Adds stochastic branching | Same |
| Bomb/Shake decisions | Adds action branching | Same |

The 2-player Matgo variant has a state space comparable to Koi-Koi (~10^12--10^15 information sets), making it a tractable CFR target. The 3-player standard format is significantly larger and faces the additional theoretical barrier of multiplayer equilibrium computation.

### Comparison to Related Games

| Game | Players | Info Sets | Solved Status | Key Difference from Go-Stop |
|------|---------|-----------|---------------|----------------------------|
| Koi-Koi | 2 | ~10^12--10^15 | Unsolved (transformer RL SOTA) | No bombs/shakes, linear koi-koi |
| Matgo (2P Go-Stop) | 2 | ~10^12--10^15 | Unsolved (NHN RL only) | Equivalent deck, different scoring |
| Limit Hold'em HU | 2 | ~10^14 | Essentially solved | Different game structure entirely |
| DouDizhu | 3 | ~10^20+ | Superhuman AI (DouZero) | Cooperative/competitive; different card structure |
| Mahjong (4P) | 4 | ~10^48+ | Superhuman AI (Suphx) | Much larger state space; tile-based |
| **Go-Stop (3P)** | **3** | **~10^12--10^18** | **Unsolved; very limited AI** | **Exponential multiplier; 3-way dynamics** |

---

## Go/Stop Decision Theory: Risk Assessment with Multiple Opponents

### The Core Decision

The Go/Stop decision in Go-Stop is an optimal stopping problem with two key differences from Koi-Koi's "koi-koi" decision:

1. **Multiple adversaries.** The risk of an opponent reaching the scoring threshold is approximately doubled with two opponents instead of one. If each opponent has probability p of scoring on their next turn, the probability of at least one opponent scoring before the Go-caller's next turn is 1 - (1-p)^2, which is significantly higher than p alone.

2. **Exponential multiplier.** While Koi-Koi doubles at a fixed threshold (7 points), Go-Stop's multiplier grows exponentially with consecutive Go calls. The first two Go calls add +1 and +2 points respectively, but the third Go doubles the entire score, and each subsequent Go doubles again. This creates a convex payoff curve that rewards aggressive play when the probability of opponents scoring is low.

### Formal Framework

Let S_t be the player's current score at decision point t. Let p_i(t) denote the probability that opponent i reaches the scoring threshold and calls Stop before the player's next turn. The expected value of the Go/Stop decision:

- **Stop:** EV = f(S_t) x (penalty multipliers), paid by both opponents
- **Go:** EV = [1 - (1-p_1)(1-p_2)] x (-g(S_t, Go_count)) + [(1-p_1)(1-p_2)] x E[f(S_{t+k})]

Where f applies Go bonuses and g represents the penalty for losing after calling Go. The critical insight is that Go becomes rational when:

    [(1-p_1)(1-p_2)] x E[f(S_{t+k}) - f(S_t)] > [1-(1-p_1)(1-p_2)] x [f(S_t) + penalty]

### Key Factors Affecting Go/Stop Optimality

| Factor | Favors Go | Favors Stop |
|--------|-----------|-------------|
| Low opponent yaku progress | Yes | |
| Many cards remaining in draw pile | Yes | |
| Player near doubling threshold (3rd Go) | Yes | |
| Opponent has called Go previously | Yes (they pay more if they lose) | |
| Multiple scoring paths available | Yes | |
| Opponents have many captured junk (>5) | | Yes (no Pi-bak bonus) |
| Player has no Bright protection | | Yes (Gwang-bak risk) |
| Fewer than 5 junk captured by player | | Yes (Pi-bak vulnerability) |
| Few cards remaining in hand | | Yes (limited improvement) |
| Opponent close to Godori/Gwang combo | | Yes (high opponent threat) |

### The Natpeok Trap

The Natpeok (나뻑) penalty creates a strategic trap: if a player calls Go but fails to increase their score before the round ends, they pay an additional penalty. This means Go is not just "continue at risk" but "continue at risk with a sunk cost." The Natpeok penalty transforms the Go decision from a pure optimal stopping problem into one with an asymmetric penalty for stalling, making it more punishing than Koi-Koi's koi-koi call.

### Multiplayer Stop Dynamics

In three-player Go-Stop, a distinctive dynamic emerges: if Player A is near the threshold and calls Go, Players B and C have a shared incentive to reach the threshold themselves and call Stop, even at a low score. A Stop at exactly 3 points is suboptimal in isolation but may be highly rational if it prevents Player A from reaching a x4 or x8 multiplied score. This defensive stopping behavior has no analog in two-player games and is a key area where heuristic AI agents underperform.

---

## Current Best Approaches

### 1. NHN Supervised + Reinforcement Learning (2020)

**The most significant published work on Go-Stop AI.**

**Source:** NHN FORWARD 2020 conference presentation, "똑똑한 고스톱 AI 만들기" (Building a Smart Go-Stop AI); coverage in AI Times Korea (2020).

**Approach:**
- **Phase 1 -- Supervised Learning:** Collected game records from human players on the Hangame platform. Trained a deep neural network to predict human card-play decisions given game state (imitation learning). Applied the same supervised learning methodology used for NHN's Baduk AI "Handol" (한돌).
- **Phase 2 -- Reinforcement Learning:** Used the supervised learning model as initialization. Applied self-play reinforcement learning to improve beyond human-level play patterns. The RL agent explores all possible plays and identifies optimal card selection based on win-rate maximization.
- **Focus:** 2-player Matgo (맞고) variant, chosen because the 1v1 format reduces variables and makes training more tractable than 3-player Go-Stop.

**Results:**
- The RL model outperformed the supervised learning model, achieving +0.4 average score differential per game over 10,000 games.
- The supervised learning model trained on general player data achieved the highest win rate against human opponents in evaluation.
- Skilled human evaluators rated the AI as "very high level."
- Validation required 10,000+ games due to Go-Stop's high variance (compared to ~400 games sufficient for Baduk AI evaluation).

**Strengths:**
- Large-scale human data from Hangame platform provides a strong behavioral prior
- RL self-play improvement beyond human patterns
- Industrial-grade infrastructure and testing methodology
- Demonstrated that the supervised-to-RL pipeline (proven for Go/Baduk) transfers to Hwatu games

**Weaknesses:**
- Focused on 2-player Matgo, not the standard 3-player Go-Stop
- No exploitability measurement or Nash equilibrium analysis
- No published model architecture details or training hyperparameters
- Not published as a peer-reviewed paper; conference presentation only
- Proprietary; no open-source implementation

**Compute:** Not disclosed. NHN has significant GPU infrastructure through NHN Cloud.

### 2. Deep Learning Patent System (NHN Cloud, 2021)

**Patent:** KR102628188B1, "딥러닝 기반 불완전 정보 게임 서비스 제공 방법 및 그 장치" (Method and Device for Providing Deep Learning-Based Incomplete Information Game Service). Filed 2021-06-04 by NHN Cloud Co., Ltd.

**Architecture described in claims:**
- **State determination training:** Deep learning model learns to classify the current board state
- **Win rate estimation training:** Predicts probability of winning given current state and action
- **Score estimation training:** Predicts expected final score
- **Yaku (jokbo) achievement probability training:** Predicts likelihood of completing each scoring combination
- The system acquires training data containing Go-Stop game board states for multiple possible actions

**Significance:**
- Represents the most detailed published description of a Go-Stop AI architecture
- Multi-head prediction approach (win rate + score + yaku probability) is more sophisticated than simple action prediction
- The yaku achievement probability head specifically addresses Go-Stop's combinatorial scoring system
- Patent scope covers the service provision method, suggesting deployment intent

**Limitations:**
- Patent claims describe architecture but not empirical results
- No published comparison with baselines
- Proprietary; filed for commercial protection, not academic dissemination

### 3. Korean Academic Theses (2012--2019)

**Scattered Korean university work on Go-Stop AI, not available in international venues.**

| Year | Authors | Contribution | Approach |
|------|---------|-------------|----------|
| 2012 | Kim et al. | Heuristic Go-Stop AI | Rule-based card selection with hand-tuned priorities |
| 2016 | Lee & Park | MCTS application to Go-Stop | Monte Carlo Tree Search for action selection |
| 2019 | Hwang et al. | RL for simplified Go-Stop | Reinforcement learning on a reduced game variant |

These theses are cited in game rules documentation but have not been independently verified through international databases. Their limited availability suggests they remain as university internal publications (possibly in RISS, the Korean academic repository). The 2016 MCTS work by Lee & Park is the most interesting, as MCTS is a natural fit for Go-Stop's stochastic draw pile, but no details of the implementation, game size, or results are available in English.

### 4. Open-Source Go-Stop Environments

**GitHub: reidlindsay/gostop** -- A Python implementation of Go-Stop with game state management, hand handling, and scoring logic. Not an AI agent but a potential environment for RL training. Includes core game logic (gamestate.py, hand.py) suitable for wrapping as a Gym-like environment.

**GitHub: jwcrown/GoStop** -- A terminal-based Go-Stop implementation.

**GitHub: sunduk/freegostop** -- A free Go-Stop game implementation.

**Velog: skyfishbae** -- A personal project blog documenting the process of building a Go-Stop AI using reinforcement learning. Serves as a tutorial-level walkthrough rather than a research contribution, but demonstrates grassroots interest in the problem.

None of these include trained AI agents or benchmark results.

### 5. Commercial Go-Stop AI (Mobile Apps)

Korean mobile Go-Stop apps (available on Google Play and iOS) include AI opponents, but these are universally rule-based systems:
- Match highest-value cards first
- Go/Stop thresholds based on simple point comparisons (e.g., "Go if score < 5, Stop if >= 5")
- No card counting or opponent modeling
- No learned components

These represent the practical floor of Go-Stop AI quality. Any solver-based approach should dramatically outperform them.

---

## Comparison with Koi-Koi Solving

Go-Stop and Koi-Koi share the same 48-card Hanafuda/Hwatu deck and the fundamental fishing card mechanic (play a card, match by month, draw from the pile). This enables significant infrastructure sharing but masks deep strategic differences:

### Shared Mechanics

| Shared Element | Reusability |
|---------------|-------------|
| 48-card deck representation (12 months x 4 cards) | 100% shared |
| Card type classification (Bright/Animal/Ribbon/Junk) | ~95% shared (minor naming differences) |
| Monthly matching logic | ~90% shared (same core mechanic) |
| Field management | ~85% shared (similar placement rules) |
| Capture mechanics | ~80% shared (same month-matching capture) |
| Scoring combination (yaku/jokbo) detection | ~60% shared (overlapping but different sets) |

### Fundamental Differences Affecting Strategy

| Dimension | Koi-Koi | Go-Stop | Impact on Solving |
|-----------|---------|---------|-------------------|
| Players | 2 | 3 (standard) | 3P breaks CFR convergence guarantees |
| Continue mechanic | Koi-koi: additive, doubles at 7 | Go: exponential from 3rd call | Much higher variance in Go-Stop |
| Penalty multipliers | None (or minimal) | Gwang-bak x2, Pi-bak x2, stacking | Creates defensive sub-objectives |
| Bombs (폭탄) | Not present | 3-in-hand + 1-on-field = mass capture + steal | Adds action branching and information revelation |
| Shakes (흔들기) | Not present | Reveal 3-of-a-month for x2 multiplier | Information vs. payoff tradeoff |
| Bonus/Joker cards | Not present (standard) | 2 extra cards (counting as 2 and 3 junk) | Deck size 50 vs 48; extra stochasticity |
| Junk stealing | Not present | Via bombs, ppuk penalties | Creates a parallel junk economy |
| Ppuk (뻑) penalty | Not present | Kiss penalty (drawn card matches placed card) | Unique tactical consideration |
| Natpeok penalty | Not present | Penalty for Go without score increase | Asymmetric cost to Go calls |
| Dual-count cards | Sake Cup (Animal+Plain) | Sake Cup (Animal+Junk); double-junk cards | Slightly different dual-counting rules |

### Strategy Space Expansion

The combination of bombs, shakes, the junk-stealing economy, and the exponential Go multiplier makes Go-Stop's strategy space qualitatively larger than Koi-Koi's, even though the underlying card mechanics are similar. A Koi-Koi solver must primarily solve: (1) which card to play, (2) which field card to capture, and (3) koi-koi or stop. A Go-Stop solver must additionally reason about: (4) whether to bomb (and when), (5) whether to shake (trading information for multiplier), (6) junk economy management (stealing/protecting), (7) penalty avoidance across two opponents, and (8) the fundamentally different three-player dynamics.

### Transferability Assessment

Techniques developed for Koi-Koi (report 07) are partially transferable:
- **Card counting:** Fully transferable. Same inference process, larger hidden space.
- **Yaku progress tracking:** ~60% transferable. Many yaku overlap but Go-Stop has Korean-specific sets.
- **ISMCTS framework:** Transferable in principle but must be extended for 3-player information sets.
- **Transformer architecture (Guan et al.):** Network architecture transferable; tokenization scheme reusable. Training pipeline needs redesign for 3-player dynamics and Go multiplier.
- **CFR:** Not directly transferable from 2-player to 3-player. Requires multiplayer extensions (kdb-D2CFR or similar).

---

## Applicable Solver Technologies from Other Domains

### Multiplayer CFR Extensions

Standard CFR does not converge to Nash equilibrium in multiplayer (>2 player) games. Several recent advances address this:

**kdb-D2CFR (2023):** Knowledge distillation-based DeepCFR for multiplayer imperfect-information games. Trains a two-player CFR model first, then transfers knowledge to a multiplayer model via knowledge distillation. Published in Knowledge-Based Systems. This is the first DeepCFR-based method for multiplayer games and directly applicable to 3-player Go-Stop.

**Quadratic Programming for Nash Equilibrium (Ganzfried, 2026):** Solves a quadratically-constrained program based on nonlinear complementarity formulation from the sequence-form game representation. Provides exact Nash equilibrium computation for multiplayer imperfect-information games, but computational cost scales poorly with game size. May be applicable to abstracted Go-Stop.

**Successful Nash Equilibrium Agent for 3-Player Games (Ganzfried & Nowak, 2018):** Demonstrated that approximate Nash equilibrium can be computed for 3-player imperfect-information games (specifically 3-player Kuhn poker) and outperformed non-equilibrium baselines.

### Information Set MCTS (ISMCTS)

ISMCTS (Cowling, Powley, Whitehouse 2012) searches trees of information sets directly, avoiding the strategy fusion problem inherent in determinization. Demonstrated strong performance in card games including DouDizhu, Lord of the Rings card game, Spades, and the fishing card game Scopone.

For Go-Stop specifically:
- Handles the 3-player stochastic draw pile naturally through sampling
- Does not require Nash equilibrium computation
- Provides anytime decision quality (can be cut off at any compute budget)
- Successfully applied to Scopone (Watanabe et al.), another fishing card game with similar capture mechanics

**Determinization approaches** (PIMC) are simpler but suffer from strategy fusion, which is particularly problematic in Go-Stop due to the high branching factor from bombs and shakes creating correlated actions.

### DouDizhu Techniques

DouDizhu (斗地主) is the closest solved analog to Go-Stop: a three-player imperfect-information card game with cooperative/competitive dynamics. Key techniques from DouDizhu research:

**DouZero (ICML 2021):** Pure self-play deep RL using Deep Monte-Carlo (DMC) methods. Achieved superhuman play without CFR or MCTS. The approach is notable for its simplicity: deep neural networks trained via Monte Carlo rollouts with action encoding. DouZero's success suggests that similar DMC-based approaches could work for Go-Stop without requiring multiplayer CFR.

**AlphaDou (2024):** End-to-end system integrating bidding and play, eliminating human-crafted knowledge. Uses neural networks to estimate both win rate and expected value simultaneously.

**OADMC-Dou (IJCAI 2024):** Oracle Guiding and Adaptive Deep Monte-Carlo, improving DouZero's training efficiency. Relevant because Go-Stop's high variance (requiring 10,000+ games for evaluation) similarly benefits from training efficiency improvements.

### Student of Games / Player of Games (DeepMind, 2023)

Student of Games (Schmid et al., Science Advances 2023) unifies guided search, self-play learning, and game-theoretic reasoning for both perfect and imperfect information games. It combines Growing-Tree CFR with counterfactual value-and-policy networks (CVPNs). Demonstrated strong performance on chess, Go, poker, and Scotland Yard.

Applicability to Go-Stop: Student of Games is designed as a general-purpose algorithm. Its CVPN architecture could be trained on Go-Stop with the game tree serving as the search space. However, the algorithm has not been tested on three-player games in its published form.

### Embedding CFR (2025)

Embedding CFR (November 2025) pre-trains information set features into a low-dimensional continuous embedding space, then performs regret accumulation and strategy updates in this embedding space. Achieves faster exploitability convergence than cluster-based abstraction. Relevant because Go-Stop's information sets have rich structure (card types, yaku progress, penalty status) that could benefit from learned embeddings rather than hand-crafted abstraction.

### GPU-Accelerated CFR (2024--2025)

GPU-parallelized CFR implementations (Park et al., ICLR 2025) achieve up to 400x speedup over CPU baselines by representing CFR as dense/sparse matrix operations on CUDA. A related 2025 paper applied GPU-accelerated CFR to solve Pasur, an Iranian fishing card game structurally similar to Go-Stop (monthly matching, capture from field). The Pasur solver is the closest published analog: a fishing card game solved via CFR with GPU acceleration.

---

## Open Problems

### 1. No Published Solver for Any Go-Stop Variant

No Nash equilibrium, approximate or exact, has been computed for any variant of Go-Stop (2-player Matgo or 3-player standard). Even the 2-player Matgo variant, which has a comparable state space to Koi-Koi (~10^12--10^15 information sets), has not been solved via CFR. This is a clear opportunity for a first-mover contribution.

### 2. Three-Player Equilibrium Computation

Computing Nash equilibrium in 3-player Go-Stop faces fundamental theoretical barriers. CFR does not converge in multiplayer games. Recent approaches (kdb-D2CFR, quadratic programming) offer partial solutions but have not been validated on games of Go-Stop's complexity. The question of what solution concept to target (Nash equilibrium, correlated equilibrium, or simply strong empirical play via RL) is open.

### 3. Rule Standardization

Go-Stop has extensive house rule variation that materially affects strategy:
- Ppuk (뻑) penalty: ranges from "no penalty" to "pay 1 junk to each opponent"
- Minimum point threshold: 3 points (common) or 7 points (some variants)
- Bonus card handling: varies by region and platform
- September Ssangpi rule (optional double-junk)
- Shake/bomb rules: some variants restrict or modify these
- Natpeok penalty: varies in severity

Any solver must fix a canonical ruleset. The Hangame rules or the rules from a major mobile Go-Stop app would be natural choices for practical relevance.

### 4. Go/Stop Decision Optimization

The Go/Stop decision with three players and exponential multipliers has not been formally characterized. An optimal stopping analysis specific to Go-Stop's payoff structure would be a standalone contribution, even without solving the full game. This would quantify: when is the third Go (doubling) worth the risk? How does the presence of two opponents change the optimal stopping boundary compared to 2-player Matgo?

### 5. Bomb and Shake Decision Theory

The bomb (폭탄) and shake (흔들기) mechanics create unique information-revelation tradeoffs:
- **Bombs** reveal three cards of a month from the player's hand, providing opponents with substantial information about the player's remaining hand. In exchange, the player captures four cards and steals junk from each opponent.
- **Shakes** declare three-of-a-month for a score multiplier, revealing information before the capture even occurs.

The information cost of these actions has not been formally analyzed. A solver must weigh the immediate tactical gain against the information loss and subsequent strategic disadvantage.

### 6. Junk Economy Modeling

The parallel economy of junk cards (피) -- accumulated through normal play, stolen through bombs and special captures, and serving both as a scoring category (10+ junk = points) and a defensive concern (fewer than 5 junk = Pi-bak penalty) -- creates a resource management sub-problem. No published work has formally modeled this economy or its interaction with the Go/Stop decision.

### 7. Opponent Modeling in Three-Player Settings

With three players, opponent modeling becomes both more important and more complex. Signals include:
- Which cards each opponent captures (yaku target inference)
- Whether opponents call Go or Stop (risk tolerance)
- Bomb usage patterns (hand composition inference)
- Junk accumulation rates (Pi-bak vulnerability assessment)

Recent work on opponent modeling in multiplayer IIGs (2022) extends Bayesian exploitation frameworks to three-player settings, but has not been applied to Go-Stop.

---

## Relevance to Myosu

### Solver Architecture Fit

| Factor | Score | Assessment |
|--------|-------|------------|
| CFR applicability | 3/5 | 2-player Matgo: yes. 3-player: limited by non-zero-sum. kdb-D2CFR is the best current path. |
| Neural value network potential | 4/5 | Rich feature space from yaku progress, junk economy, penalty status, Go count |
| Abstraction necessity | 2/5 | Moderate state space; may be solvable with minimal abstraction (2P) or moderate abstraction (3P) |
| Real-time solving value | 4/5 | ISMCTS ideal for real-time play; critical for Go/Stop decisions under time pressure |
| Transferability from Koi-Koi | 3/5 | Shared deck and matching logic; diverges on multiplayer dynamics and scoring |
| Novelty opportunity | 5/5 | No published solver; massive cultural significance; Myosu would be first |

### Recommended Solver Pipeline

1. **Phase 1 -- Shared Infrastructure:** Build a Go-Stop game environment reusing Koi-Koi's 48-card deck representation and matching logic. Extend to support 50-card deck (bonus cards), bombs, shakes, and the full Go-Stop scoring system. Target both 2-player Matgo and 3-player Go-Stop variants.

2. **Phase 2 -- 2-Player Matgo Baseline via CFR:** Apply MCCFR (or GPU-accelerated CFR, following the Pasur precedent) to compute an approximate Nash equilibrium for 2-player Matgo. The state space is comparable to Koi-Koi and should be tractable. This produces the first published equilibrium for any Go-Stop variant and validates the game implementation.

3. **Phase 3 -- 3-Player ISMCTS Agent:** Implement ISMCTS for 3-player Go-Stop as a practical strong agent. ISMCTS handles the three-player dynamics naturally (no Nash convergence required; it optimizes expected utility under sampled opponent hands). Use card counting and yaku progress tracking as evaluation features.

4. **Phase 4 -- 3-Player Deep RL (DouZero-style):** Train a deep RL agent for 3-player Go-Stop using Deep Monte-Carlo methods (following DouZero's approach for DouDizhu). This avoids the multiplayer CFR convergence problem entirely. The neural network learns card selection, Go/Stop decisions, and bomb/shake timing end-to-end from self-play.

5. **Phase 5 -- Multiplayer CFR via Knowledge Distillation:** Optionally, apply kdb-D2CFR to transfer knowledge from the 2-player Matgo Nash equilibrium to a 3-player approximation. This provides a more theoretically grounded strategy than pure RL, at the cost of additional complexity.

### Subnet Design Considerations

- **Korean market.** Go-Stop is played by tens of millions of Koreans. It is the single most popular card game in the country. No competitive AI solver exists. The market opportunity is enormous.
- **Shared deck with Koi-Koi.** The 48-card Hwatu/Hanafuda deck means card representation, matching logic, and basic scoring detection can be shared with game 07. Estimated 60--80% code reuse at the game engine level.
- **Rule standardization.** The canonical ruleset must be defined precisely and published. Recommended: 3-player standard with 3-point minimum, Gwang-bak and Pi-bak penalties, bonus cards enabled, Natpeok penalty enabled.
- **Verification oracle.** Scoring is deterministic given captured cards and Go/Stop decisions. On-chain verification is straightforward: replay the capture sequence and compute yaku + multipliers.
- **High variance requires statistical rigor.** As NHN found, Go-Stop requires 10,000+ games for reliable performance evaluation. The evaluation harness must account for this.
- **Paired deployment.** Go-Stop and Koi-Koi should be deployed as paired games sharing infrastructure, reducing the marginal cost of supporting both.

---

## Key Papers and References

### Go-Stop-Specific Research

| Year | Authors / Source | Venue | Contribution |
|------|-----------------|-------|--------------|
| 2012 | Kim et al. (Korean) | Korean university thesis | Heuristic Go-Stop AI with rule-based card selection |
| 2016 | Lee & Park (Korean) | Korean university thesis | MCTS application to Go-Stop |
| 2019 | Hwang et al. (Korean) | Korean university thesis | Reinforcement learning for simplified Go-Stop |
| 2020 | NHN AI Team | NHN FORWARD 2020 conference | Supervised learning + RL for 2-player Matgo; +0.4 score/game improvement |
| 2021 | NHN Cloud Co., Ltd. | Korean Patent KR102628188B1 | Deep learning architecture for Go-Stop: state/winrate/score/yaku prediction |

### Koi-Koi Research (Transferable)

| Year | Authors / Title | Venue | Relevance |
|------|----------------|-------|-----------|
| 2023 | Guan, Wang, Zhu, Qian, Wei: "Learning to Play Koi-Koi Hanafuda Card Games with Transformers" | IEEE TAI, vol. 4, no. 6 | Transformer + MC-RL architecture transferable to Go-Stop |
| ~2019 | CyberAgent AI Lab: "ミニ花札のAIを作ってみよう" | Tech blog | CFR proof-of-concept for Hanafuda family |

### Korean Card Game AI

| Year | Authors / Title | Venue | Relevance |
|------|----------------|-------|-----------|
| 2022 | Lee, Kim, Kim: "Analysis and simulator implementation of Mighty" | J. Korea Society of Computer and Information | Korean card game simulator for AI research; methodology applicable to Go-Stop |

### Multiplayer Imperfect-Information Game Solving

| Year | Authors / Title | Venue | Relevance |
|------|----------------|-------|-----------|
| 2018 | Ganzfried & Nowak: "Successful Nash Equilibrium Agent for a 3-Player Imperfect-Information Game" | Games (MDPI) | 3-player equilibrium computation |
| 2019 | Brown & Sandholm: "Superhuman AI for Multiplayer Poker" (Pluribus) | Science | MCCFR + real-time search for 6-player poker |
| 2021 | Zha et al.: "DouZero: Mastering DouDizhu with Self-Play Deep RL" | ICML | DMC for 3-player card game; directly applicable methodology |
| 2023 | Schmid et al.: "Student of Games" | Science Advances | Unified algorithm for perfect + imperfect info games |
| 2023 | Kdb-D2CFR: "Solving Multiplayer IIGs with knowledge distillation-based DeepCFR" | Knowledge-Based Systems | First DeepCFR method for multiplayer games |
| 2024 | OADMC-Dou (IJCAI 2024) | IJCAI | Oracle-guided training efficiency for 3-player card game |
| 2024 | AlphaDou: "High-Performance End-to-End Doudizhu AI" | arXiv | End-to-end 3-player card game AI without handcrafted knowledge |
| 2025 | Keshavarzi & Navidi: "Comparative analysis of extensive form zero sum game algorithms" | Scientific Reports | Comprehensive benchmark of CFR variants including 3--5 player scalability |
| 2025 | Embedding CFR | arXiv | Pre-trained information set embeddings for faster convergence |
| 2025 | GPU-Accelerated CFR | ICLR 2025 | 400x speedup via CUDA; applied to Pasur fishing card game |
| 2025 | Baghal: "Solving Pasur Using GPU-Accelerated CFR" | arXiv | CFR for a fishing card game; closest structural analog to Go-Stop solving |
| 2026 | Ganzfried: "QP Approach for Nash Equilibrium in Multiplayer IIGs" | Games (MDPI) | Exact multiplayer Nash computation via quadratic programming |

### ISMCTS and Determinization

| Year | Authors / Title | Venue | Relevance |
|------|----------------|-------|-----------|
| 2011 | Whitehouse, Cowling, Powley: "Determinization and ISMCTS for Dou Di Zhu" | IEEE CIG | ISMCTS for 3-player card game |
| 2012 | Cowling, Powley, Whitehouse: "Information Set Monte Carlo Tree Search" | IEEE TCIAIG | Foundational ISMCTS paper; solves strategy fusion |
| ~2018 | Watanabe et al.: "IS-MCTS for Scopone" | (cited in survey) | ISMCTS for Italian fishing card game |

### Toolkits and Environments

| Year | Resource | URL | Relevance |
|------|----------|-----|-----------|
| 2019 | RLCard: RL Toolkit for Card Games | rlcard.org | Environment framework; Go-Stop not included but architecture applicable |
| -- | reidlindsay/gostop | github.com/reidlindsay/gostop | Python Go-Stop implementation; potential RL environment base |

---

## Appendix: Jokbo (족보) Progress as a Feature Vector

For solver implementations, the Go-Stop game state can be compactly represented by tracking progress toward each scoring combination for all three players. This feature vector is analogous to Koi-Koi's yaku progress vector but includes Go-Stop-specific elements:

| Jokbo | Required Cards | Tracking State | Points |
|-------|---------------|---------------|--------|
| Ogwang (오광, Five Brights) | All 5 Gwang | Gwang count (0--5) | 15 |
| Sagwang (사광, Four Brights) | 4 Gwang, no Rain Man | Non-Rain Gwang (0--4) | 4 |
| Bi-Sagwang (비사광) | Rain Man + 3 Gwang | Rain Man (bool) + others (0--4) | 3--4 |
| Samgwang (삼광, Three Brights) | 3 Gwang, no Rain Man | Non-Rain Gwang (0--4) | 3 |
| Bi-Samgwang (비삼광) | Rain Man + 2 Gwang | Rain Man (bool) + others (0--4) | 2 |
| Godori (고도리, Five Birds) | Cuckoo + Geese + Nightingale | Bird count (0--3) | 5 |
| Base Animals | 5+ Animal cards | Animal count (0--9) | 1 + extras |
| Hongdan (홍단, Red Poetry) | Jan + Feb + Mar poetry ribbons | Red poetry count (0--3) | 3 |
| Cheongdan (청단, Blue) | Jun + Sep + Oct blue ribbons | Blue ribbon count (0--3) | 3 |
| Chodan (초단, Grass) | Apr + May + Jul plain ribbons | Plain red count (0--3) | 3 |
| Base Ribbons | 5+ Ribbon cards | Ribbon count (0--10) | 1 + extras |
| Base Junk | 10+ junk (with double-counting) | Effective junk count (0--30+) | 1 + extras |

**Additional Go-Stop-specific features beyond yaku:**
- Go call count per player (0, 1, 2, 3+)
- Gwang-bak vulnerability per player (bool: has any Bright?)
- Pi-bak vulnerability per player (int: junk count < 5?)
- Shake declarations per player (count)
- Bomb availability (3-of-month in hand + 1 on field)
- Cards remaining in hand per player
- Cards remaining in draw pile
- Effective payment multiplier estimate per player

This compact representation (approximately 40--50 integers for all three players combined) captures the strategically relevant state. It is amenable to both linear evaluation functions (for ISMCTS rollout evaluation), neural network input (for Deep RL or Deep CFR), and feature engineering for opponent modeling.
