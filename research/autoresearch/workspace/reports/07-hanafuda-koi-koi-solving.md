# Solving Strategies for Hanafuda Koi-Koi (花札 こいこい)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods and AI research, 2010--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Hanafuda Koi-Koi is a two-player, zero-sum, imperfect-information card game played with the traditional Japanese 48-card Hanafuda deck. Unlike poker, where the primary hidden information is hole cards and the action space is dominated by bet sizing, Koi-Koi's complexity arises from a unique combination of matching/capture mechanics, a combinatorial yaku (scoring combination) system, and the eponymous "koi-koi" continue-or-stop decision that functions as an embedded optimal stopping problem.

Academic research on Koi-Koi AI is sparse compared to poker or Go, but has accelerated since 2020. The most significant published work is Guan et al. (2023), "Learning to Play Koi-Koi Hanafuda Card Games with Transformers" (IEEE Transactions on Artificial Intelligence), which achieved a 53% win rate against experienced human players using a transformer encoder trained via Monte Carlo reinforcement learning. Earlier Japanese-language research explored policy gradient methods (JAIST, 2017), Monte Carlo reward shaping for strategy construction (IPSJ, 2021), and CFR applied to a reduced "Mini-Hanafuda" variant (CyberAgent AI Lab). A commercial implementation by SUZUKI PLAN (Battle Hanafuda, 2024--2025) uses Monte Carlo inference for its strongest difficulty level.

The game's moderate state space (estimated 10^15--10^20 game tree nodes per round, far smaller than NLHE) makes it a tractable target for both CFR-based equilibrium computation and neural RL approaches. However, the stochastic draw pile, the asymmetric information about opponent hand cards, and the complex yaku interaction system create challenges that prevent naive approaches from reaching strong play. The koi-koi decision is the strategic crux: a double-or-nothing optimal stopping problem whose correct solution depends on card counting, yaku progress tracking for both players, and probabilistic reasoning about the hidden deck.

No Nash equilibrium has been published for full Koi-Koi. This represents a clear opportunity for Myosu to be first.

---

## Game Complexity Analysis

### Deck and Card Structure

The Hanafuda deck contains 48 cards organized into 12 suits (months), each with exactly 4 cards. Cards are further categorized into four types with different strategic value:

| Card Type | Count | Per-Card Points | Strategic Role |
|-----------|-------|-----------------|----------------|
| Bright (光) | 5 | 20 | Highest-value yaku components |
| Animal/Seed (種) | 9 | 10 | Mid-tier yaku, versatile |
| Ribbon (短冊) | 10 | 5 | Multiple overlapping yaku |
| Plain/Chaff (カス) | 24 | 1 | Bulk yaku (10+ needed) |

The Sake Cup (September Chrysanthemum) is dual-typed: it counts as both Animal and Plain in most rulesets, making it the single most versatile card for yaku stacking.

### State Space Estimates

| Metric | Estimate | Notes |
|--------|----------|-------|
| Initial deal combinations | C(48,8) x C(40,8) x C(32,8) | ~10^14 initial configurations |
| Cards per hand (start) | 8 | Decreasing each turn |
| Turns per round | 16 (8 per player) | Fixed by hand size |
| Branching factor per turn | 1--8 (card choice) x 1--2 (field match) | Plus binary koi-koi decision |
| Game tree nodes (single round) | ~10^15--10^20 | Estimated |
| Information sets (single round) | ~10^12--10^15 | Hidden: opponent hand + draw pile order |
| Full game (12 rounds) | Multiplicative | But rounds are largely independent |

### Why Koi-Koi Is Moderately Hard

1. **Matching constraint.** Players cannot freely choose which cards to capture; they must match by month (suit). This constrains the action space but creates forced plays and field management dilemmas.

2. **Stochastic draw pile.** After playing from hand, the player draws a random card from the 24-card draw pile. This draw is the primary source of stochasticity and cannot be controlled.

3. **Imperfect information.** Each player sees their own hand (8 cards) and the field (8 cards). The remaining 24 cards are split between the opponent's hand (8) and the draw pile (16 at game start), with no information about which is where.

4. **Combinatorial yaku system.** Multiple overlapping scoring combinations must be tracked simultaneously. Some yaku are mutually exclusive (Bright tiers), some stack additively (Ribbons), and incremental cards beyond the minimum add bonus points.

5. **The koi-koi decision.** After completing a yaku, the player must decide whether to stop and bank points or continue. This embeds an optimal stopping problem inside each round.

6. **Multiplier effects.** Scores of 7+ points are doubled. An opponent's prior koi-koi call can further double the winner's score. These nonlinear payoffs make the risk-reward calculation discontinuous.

### Comparison to Other Games

| Game | State Space | Info Type | Solved Status | Notes |
|------|-------------|-----------|---------------|-------|
| Limit Hold'em HU | ~10^14 info sets | Imperfect | Essentially solved | Bowling et al. 2015 |
| **Koi-Koi** | **~10^12--10^15 info sets** | **Imperfect** | **Unsolved** | **No published equilibrium** |
| Dou Di Zhu | ~10^20+ | Imperfect | Superhuman AI | ISMCTS + RL |
| NLHE HU | ~10^161 nodes | Imperfect | Approximate Nash | Libratus, DeepStack |
| Mahjong (4-player) | ~10^50+ | Imperfect | Superhuman AI | Suphx (Microsoft) |

Koi-Koi's information set count is comparable to Limit Hold'em HU, suggesting that direct CFR computation (without heavy abstraction) may be feasible. This is a distinctive opportunity: the game may be small enough to solve more precisely than poker, yet complex enough to be strategically interesting.

---

## Koi-Koi Decision Theory: The Optimal Stopping Problem

### The Core Dilemma

The koi-koi call is the game's defining strategic element. It maps directly to the class of optimal stopping problems with adversarial risk:

- **Stopping (calling "Stop"):** Collect the sum of all completed yaku. The round ends. Guaranteed payoff.
- **Continuing (calling "Koi-Koi"):** The round continues. Potential to accumulate additional yaku and reach the 7-point doubling threshold. But if the opponent completes a yaku first, the koi-koi caller scores zero and the opponent may receive doubled points.

This differs from classical optimal stopping (e.g., the Secretary Problem) because:
1. The opponent is an adversarial agent, not a passive stochastic process.
2. Multiple koi-koi calls can be made per round, compounding risk.
3. The payoff structure is nonlinear (doubling at 7+ points).
4. The remaining card information is partially observable.

### Formal Structure

Let S_t be the player's current yaku score at decision point t. The player chooses between:

- **Stop:** receive payoff f(S_t), where f applies doubling rules
- **Continue:** receive f(S_{t+k}) if the player completes more yaku before the opponent, or 0 (and the opponent receives g(S_opp)) if the opponent completes first

The optimal policy depends on:
- S_t (current score and distance to doubling threshold)
- Opponent's visible captured cards (yaku progress inference)
- Cards remaining in hand and estimated draw pile composition
- Number of turns remaining
- Whether the opponent has previously called koi-koi (affects their multiplier)

### Key Heuristic Thresholds

Analysis from strategy guides and AI research suggests:

| Current Score | General Recommendation | Rationale |
|---------------|----------------------|-----------|
| 1--3 points | Call koi-koi (aggressive) | Low guaranteed value; potential for large upside |
| 4--6 points | Context-dependent | Depends on yaku progress and opponent threat |
| 7+ points | Stop (conservative) | Already at doubling threshold; risk of losing doubled score |
| 1--3 + opponent near yaku | Stop | Even small guaranteed points beat probable zero |

A rigorous solution requires solving the full game tree, as these heuristics ignore the interaction between card counting, field state, and opponent modeling.

### Multi-Round Dynamics

In multi-round Koi-Koi (typically 6 or 12 rounds), cumulative score affects risk tolerance. A player leading in total score should adopt a more conservative koi-koi policy (stop earlier, protect the lead), while a trailing player should call koi-koi more aggressively (the expected value of the game is negative under conservative play, so variance-seeking behavior is rational).

---

## Current Best Approaches

### 1. Transformer + Monte Carlo RL (Guan et al. 2023)

**The most significant published work on Koi-Koi AI.**

**Paper:** "Learning to Play Koi-Koi Hanafuda Card Games with Transformers," IEEE Transactions on Artificial Intelligence, vol. 4, no. 6, pp. 1449--1460, 2023.

**Architecture:**
- Transformer encoder backbone processes tokenized card state input
- Input encoding uses "deck tokens + special tokens" -- each card is tokenized with attributes (suit, type) without handcrafted features
- Special [K0] and [K1] tokens with unique one-hot encoding are added to extract overall state for the koi-koi stop/continue decision
- Multi-head self-attention learns card dependencies: some heads specialize in same-suit connections (matching), others learn cross-suit yaku patterns
- Preliminary card embeddings are produced via position-wise feed-forward layers with ReLU activations

**Training:**
- Monte Carlo reinforcement learning from self-play, starting from zero (no human data)
- Phased round reward: reward shaping that accounts for multi-round game dynamics
- No look-ahead search at inference time; the trained network directly outputs action probabilities

**Results:**
- 53% win rate vs. experienced human players in multi-round games
- +2.02 average point differential per game
- Attention analysis reveals the agent prioritizes high-value yaku while assessing opponent threat

**Strengths:**
- End-to-end learning without handcrafted features
- Attention mechanism provides interpretable strategy analysis
- Handles multi-round dynamics via phased reward
- Open-source implementation available (github.com/guansanghai/KoiKoi-AI)

**Weaknesses:**
- 53% win rate is solid but not superhuman
- No exploitability measurement against a Nash baseline
- Monte Carlo RL can converge to local optima; no theoretical convergence guarantee to Nash equilibrium
- Does not use search at inference time, leaving performance on the table

**Compute:** Not publicly documented for training. Inference is lightweight (single forward pass).

### 2. CFR on Mini-Hanafuda (CyberAgent AI Lab)

**The only published application of CFR to Hanafuda.**

**Approach:**
- Developed a reduced "Mini-Hanafuda" variant: 2 hand cards each, 1 field card, subset of the full deck
- Applied vanilla Counterfactual Regret Minimization to compute Nash equilibrium strategies
- Relaxed yaku formation requirements and added new yaku to ensure scoring opportunities in the reduced game

**Results:**
- Successfully computed exact Nash equilibrium for the mini variant
- Demonstrated that CFR is applicable to Hanafuda's game structure
- Identified that scaling to full Hanafuda would require abstraction techniques

**Significance:**
- Proof of concept that CFR works for the Hanafuda game family
- CyberAgent explicitly noted that future work should apply abstraction (card bucketing, information abstraction) to scale to full 48-card Koi-Koi
- No follow-up publication on full-size CFR has appeared as of 2026

**Strengths:**
- Exact equilibrium (not approximate) for the reduced game
- Validates the CFR pathway for Hanafuda

**Weaknesses:**
- Mini variant is dramatically simpler than full Koi-Koi
- Scaling path to full game is not demonstrated
- No head-to-head comparison with other approaches

### 3. Monte Carlo Methods with Reward Shaping (IPSJ, 2021)

**Japanese academic work on constructing multiple strategies via reward manipulation.**

**Paper:** "モンテカルロ法の報酬の変更による花札「こいこい」の戦略構築" (Strategy Construction in Hanafuda Koi-Koi by Modifying Monte Carlo Rewards), IPSJ Proceedings, 2021.

**Approach:**
- Uses Monte Carlo playouts (random simulation to end of round) to evaluate actions
- Constructs multiple distinct strategies by varying the reward function:
  - Aggressive (maximize yaku count, call koi-koi frequently)
  - Defensive (minimize opponent scoring, stop early)
  - Balanced (weighted combination)
- Each strategy is evaluated against random players and against the other strategies

**Results:**
- All Monte Carlo strategies significantly outperform random play
- Strategy effectiveness varies depending on the opponent's style
- No single reward function dominates across all opponents

**Strengths:**
- Simple, interpretable approach
- Demonstrates that reward shaping meaningfully affects Koi-Koi strategy
- Multiple strategies provide a portfolio that could be used for opponent exploitation

**Weaknesses:**
- Random playouts are a weak evaluation signal (no opponent modeling)
- No convergence to Nash equilibrium
- No comparison to learning-based approaches

### 4. Policy Gradient Reinforcement Learning (JAIST, 2017)

**Early work on applying RL to Koi-Koi.**

**Paper:** "花札の「こいこい」ゲームの強化学習によるコンピュータプレイヤ" (A Computer Player for Hanafuda Koi-Koi Using Reinforcement Learning), JAIST Repository, 2017.

**Approach:**
- Policy gradient method (REINFORCE family)
- Hand-designed features based on game domain knowledge:
  - Yaku progress vectors (how close to each possible yaku)
  - Card type distributions in hand, field, and captured piles
  - Opponent's visible captured cards
- Weighted linear combination of features as the state-action value function
- Trained against random and rule-based opponents

**Results:**
- Outperforms random players and simple rule-based opponents
- Feature engineering proved critical to learning speed

**Strengths:**
- Demonstrates RL viability for Koi-Koi
- Feature design provides interpretable state representation

**Weaknesses:**
- Linear function approximation limits representational capacity
- Hand-crafted features may miss subtle interactions
- Training against weak opponents does not guarantee strong play against strong opponents
- Superseded by Guan et al.'s transformer approach

### 5. Monte Carlo Inference in Commercial AI (SUZUKI PLAN, 2024--2025)

**Battle Hanafuda (Steam) uses Monte Carlo inference for its strongest AI difficulty.**

**Approach:**
- Monte Carlo-based estimation algorithm selects strategies maximizing win probability
- Difficulty levels 0--4, with level 4 (strongest) potentially incorporating machine learning
- AI is actively tuned during the Early Access period

**Significance:**
- Represents the state of practice in commercial Hanafuda AI
- Validates that Monte Carlo inference produces competent play in practice
- The developer has noted that Monte Carlo inference may be more effective than deep learning for Hanafuda's "sweet spot" action space (not too large, not too small)

---

## Imperfect Information Handling

### Card Counting and Inference

Koi-Koi's information structure makes card counting both feasible and strategically essential:

- **Known information:** Own hand (8 cards), field (8 cards), own captured cards, opponent's captured cards
- **Unknown at start:** 24 cards split between opponent hand (8) and draw pile (16)
- **Progressive revelation:** Every card played (from hand or drawn) reveals new information, narrowing the hidden set

With only 48 cards total, perfect card counting is tractable. By mid-game, a card-counting player can narrow the opponent's possible hand to a small set of combinations. This is the primary skill ceiling for human players and the baseline capability for any competitive AI.

**Bayesian inference** can be applied to weight the probability of each possible opponent hand based on observed play. If an opponent plays a low-value card when a high-value match was available, this provides information about their hand composition and strategic intent.

### Determinization and ISMCTS

Two well-studied approaches for imperfect information in card games are directly applicable to Koi-Koi:

**Perfect Information Monte Carlo (PIMC):**
- Sample possible opponent hands and draw pile orderings consistent with observed play
- Solve each "determinized" game as if it were a perfect-information game
- Average results across samples
- Known weaknesses: **strategy fusion** (recommending different moves for different samples when the player must choose a single action) and **non-locality** (locally optimal moves that are globally inferior)

**Information Set Monte Carlo Tree Search (ISMCTS):**
- Searches trees of information sets directly, avoiding strategy fusion
- Each node represents what the player knows, not a specific game state
- Demonstrated to outperform PIMC in card games with significant hidden information (Cowling, Powley, Whitehouse 2012)
- Successfully applied to Pokemon, Dou Di Zhu, Spades, and other card games
- Natural fit for Koi-Koi's draw pile uncertainty

### Opponent Modeling

Bayesian opponent modeling (Ganzfried 2016) provides a framework for exploiting non-equilibrium opponents:
- Maintain a distribution over possible opponent strategies
- Update beliefs based on observed actions (which cards they play, whether they call koi-koi)
- Shift play toward exploitative strategies as confidence in the opponent model increases

For Koi-Koi specifically, key signals for opponent modeling include:
- Which field cards the opponent captures (reveals yaku targets)
- Which cards the opponent places on the field (reveals hand limitations)
- Koi-koi decisions (reveals risk tolerance and score expectations)
- Timing of stops relative to score thresholds

---

## Relevant General-Purpose Algorithms

### Counterfactual Regret Minimization (CFR) and Variants

CFR is the foundational algorithm for two-player zero-sum imperfect-information games. Given Koi-Koi's estimated ~10^12--10^15 information sets per round, several CFR variants are applicable:

| Variant | Key Innovation | Applicability to Koi-Koi |
|---------|---------------|--------------------------|
| Vanilla CFR | Full tree traversal per iteration | May be feasible for abstracted game |
| Monte Carlo CFR (MCCFR) | Sample-based traversal | Handles stochastic draw pile naturally |
| CFR+ | Regret matching+, faster convergence | Direct application |
| Discounted CFR (DCFR) | Down-weights early iterations | Better final strategy quality |
| Deep CFR | Neural network approximation | Eliminates need for explicit abstraction |
| DDCFR+ (2024) | Learned discount schedules | Faster convergence in medium-sized games |

**Deep CFR** (Brown & Lerer, 2019) is particularly relevant: it replaces tabular storage with neural networks that generalize across similar information sets. This would allow solving full Koi-Koi without explicit abstraction, using the neural network's implicit generalization as a form of abstraction. The game's moderate size (much smaller than NLHE) suggests Deep CFR could converge to a strong strategy with feasible compute.

### ReBeL and Student of Games

**ReBeL** (Recursive Belief-based Learning, Brown et al. 2020) generalizes AlphaZero's self-play + search paradigm to imperfect-information games by operating on "public belief states" (probability distributions over possible game states). It converges to Nash equilibrium in any two-player zero-sum game and achieved superhuman play in NLHE with less domain knowledge than prior poker AI.

**Student of Games** (Schmid et al., Science Advances 2023) unifies search, self-play learning, and game-theoretic reasoning for both perfect and imperfect information games. It combines Growing-Tree CFR (GT-CFR) with sound self-play and learned counterfactual value/policy networks (CVPNs).

Both algorithms are applicable to Koi-Koi in principle. The game's smaller state space compared to NLHE means the public belief state representation would be more manageable, and the value networks would need to capture simpler patterns.

### Abstraction Considerations

Given Koi-Koi's moderate state space, the question is whether abstraction is needed at all:

- **Card abstraction:** Grouping cards by strategic similarity (e.g., all non-yaku-contributing plains are equivalent). The 48-card deck with distinct card identities (each card participates in specific yaku) means abstraction must be careful not to conflate strategically different cards.
- **Action abstraction:** The action space is already small (play 1 of up to 8 cards, choose 1 of up to 2 matches). No action abstraction is needed.
- **Information abstraction:** Grouping similar opponent-hand distributions. Given the tight deck (48 cards), this may not be necessary.

The CyberAgent Mini-Hanafuda CFR experiment suggests that full-game CFR without abstraction may be infeasible, but with modest card-type abstraction (treating strategically equivalent plains as identical), the information set count could be reduced to a tractable level.

---

## Open Problems

### 1. No Published Nash Equilibrium

No Nash equilibrium (exact or approximate) has been computed for full Koi-Koi under any standard ruleset. The game's moderate state space makes this a realistic near-term target. Computing and publishing the first Koi-Koi Nash equilibrium would be a novel contribution.

### 2. Rule Variation Standardization

Koi-Koi has numerous house rules that significantly affect strategy:
- Whether Hanami-zake and Tsukimi-zake are enabled
- Whether the Sake Cup dual-counts for Animal and Plain yaku
- Doubling threshold (7 points vs. other values)
- Exhaustive draw scoring (dealer wins, no points, or captured-card comparison)
- Teshi/Kuttsuki handling (redeal vs. automatic win)
- Monthly multiplier (rounds increase in value)

Any serious solver must fix a canonical ruleset. The Nintendo standard rules or the Board Game Arena implementation are natural candidates.

### 3. Multi-Round Meta-Strategy

Most AI research has focused on single-round play. Multi-round Koi-Koi introduces a meta-game layer:
- Risk tolerance should vary with cumulative score differential
- Dealer/non-dealer advantage alternates each round
- Monthly multipliers (if used) make later rounds exponentially more important

Solving the multi-round game optimally requires dynamic programming over the space of possible cumulative scores, a layer of complexity beyond single-round solving.

### 4. Koi-Koi Decision Optimization

The koi-koi call is under-studied as a standalone decision problem. A focused study could:
- Characterize the optimal stopping boundary as a function of game state
- Quantify the expected value lost by heuristic koi-koi policies vs. optimal
- Analyze how the doubling threshold affects optimal play

### 5. Draw Pile Order Inference

In physical play, the draw pile is shuffled randomly. But in multi-turn play, the sequence of draws provides no information about remaining order (each draw is independent conditional on remaining cards). An open question is whether there exist exploitable patterns in specific digital implementations (e.g., weak PRNGs) and whether the solver should explicitly reason about draw pile order or treat it as exchangeable.

### 6. Yaku Interaction Complexity

The yaku system creates complex incentive structures where capturing one card may advance multiple yaku simultaneously. Formalizing the "yaku progress vector" as a feature and understanding its interaction with capture decisions is an under-explored area with implications for both heuristic and neural approaches.

---

## Relevance to Myosu

### Solver Architecture Fit

| Factor | Score | Assessment |
|--------|-------|------------|
| CFR applicability | 5/5 | 2-player zero-sum, moderate state space; Deep CFR could solve without abstraction |
| Neural value network potential | 4/5 | Rich feature space from yaku progress; transformer attention well-suited |
| Abstraction necessity | 2/5 | Game may be tractable with minimal or no abstraction |
| Real-time solving value | 3/5 | ISMCTS viable for real-time; pre-computed blueprint may suffice |
| Transferability | 3/5 | Unique mechanics but general imperfect-info techniques apply |
| Novelty opportunity | 5/5 | No published equilibrium; Myosu could be first |

### Recommended Solver Pipeline for Myosu

1. **Phase 1 -- Baseline:** Implement ISMCTS with card counting as the baseline agent. This provides a strong, interpretable player with no training required.

2. **Phase 2 -- Blueprint via Deep CFR:** Apply Deep CFR (or MCCFR with neural function approximation) to compute an approximate Nash equilibrium for the full game under a fixed ruleset. The moderate state space suggests this is feasible with commodity hardware (multi-GPU training over days, not weeks).

3. **Phase 3 -- Real-time refinement:** Use the blueprint strategy as a prior for real-time subgame solving (ReBeL-style) at decision points. This is particularly valuable for the koi-koi decision, where the binary choice can be resolved with high-quality local search.

4. **Phase 4 -- Multi-round optimization:** Layer dynamic programming over the single-round solver to account for cumulative score dynamics and risk-adjusted koi-koi decisions.

### Subnet Design Considerations

- **Canonical ruleset:** Fix a specific Koi-Koi ruleset (recommended: Nintendo standard or Board Game Arena rules with Hanami/Tsukimi enabled). Publish the exact rules on-chain for verifiable evaluation.
- **Yaku verification oracle:** The scoring system is deterministic given captured cards. On-chain yaku verification is straightforward and can serve as the scoring oracle.
- **Strategy encoding:** A Koi-Koi strategy can be represented as a policy mapping information sets to action distributions. Given the moderate game size, the full policy table may be storable, unlike poker where only the blueprint + solver are feasible.
- **Cultural value:** Hanafuda is deeply embedded in Japanese gaming culture. Koi-Koi is one of the most widely recognized card games in Japan. Including it in Myosu broadens market reach beyond Western card games.
- **Difficulty calibration:** Koi-Koi's moderate complexity makes it an ideal intermediate target -- harder than fully solved games, easier than NLHE, and offering a clear path to a publishable equilibrium result.

---

## Key Papers and References

### Koi-Koi-Specific Research

| Year | Authors / Title | Venue | Contribution |
|------|----------------|-------|--------------|
| 2017 | JAIST: "花札の「こいこい」ゲームの強化学習によるコンピュータプレイヤ" | JAIST Repository | Policy gradient RL with hand-crafted features for Koi-Koi |
| ~2019 | CyberAgent AI Lab: "ミニ花札のAIを作ってみよう" | CyberAgent Tech Blog | CFR applied to Mini-Hanafuda; proof of concept for equilibrium computation |
| 2021 | "モンテカルロ法の報酬の変更による花札「こいこい」の戦略構築" | IPSJ Proceedings | Monte Carlo reward shaping for multiple Koi-Koi strategies |
| 2023 | Guan, Wang, Zhu, Qian, Wei: "Learning to Play Koi-Koi Hanafuda Card Games with Transformers" | IEEE TAI, vol. 4, no. 6, pp. 1449--1460 | Transformer + MC-RL; 53% win rate vs. humans; SOTA |
| 2024--25 | SUZUKI PLAN: Battle Hanafuda | Steam (commercial) | Monte Carlo inference for commercial Hanafuda AI |

### General Imperfect-Information Game Solving

| Year | Authors / Title | Venue | Relevance |
|------|----------------|-------|-----------|
| 2007 | Zinkevich et al.: "Regret Minimization in Games with Incomplete Information" | NeurIPS | CFR: foundational algorithm |
| 2012 | Cowling, Powley, Whitehouse: "Information Set Monte Carlo Tree Search" | IEEE TCIAIG | ISMCTS; solves strategy fusion for card games |
| 2015 | Bowling et al.: "Heads-up Limit Hold'em Poker Is Solved" | Science | Limit poker solved via CFR+; comparable state space to Koi-Koi |
| 2019 | Brown & Lerer: "Deep Counterfactual Regret Minimization" | ICML | Neural CFR; eliminates explicit abstraction |
| 2020 | Brown et al.: "Combining Deep RL and Search for Imperfect-Information Games" (ReBeL) | NeurIPS | Self-play + search on public belief states |
| 2023 | Schmid et al.: "Student of Games" | Science Advances | Unified algorithm for perfect + imperfect info games |
| 2025 | "Valet: A Standardized Testbed of Traditional Imperfect-Information Card Games" | arXiv | 21-game testbed; framework for cross-game evaluation |
| 2025 | "Deep (Predictive) Discounted CFR" | arXiv | Faster neural CFR convergence |

### Opponent Modeling and Card Counting

| Year | Authors / Title | Venue | Relevance |
|------|----------------|-------|-----------|
| 2016 | Ganzfried: "Bayesian Opponent Exploitation in Imperfect-Information Games" | IEEE | Bayesian exploitation framework for 2p0s games |
| 2022 | "Opponent Modeling in Multiplayer Imperfect-Information Games" | arXiv | Extends opponent modeling beyond 2-player |

### Related Card Game AI

| Year | Authors / Title | Venue | Relevance |
|------|----------------|-------|-----------|
| 2011 | Whitehouse et al.: "Determinization and ISMCTS for Dou Di Zhu" | IEEE CIG | ISMCTS for a Chinese card game; directly applicable techniques |
| 2019 | Sturtevant et al.: "Survey of AI for Card Games" | arXiv | Comprehensive survey covering PIMC, ISMCTS, CFR for card games |
| 2024 | "Research of AI in Imperfect Information Card Games" | ACE Journal | Recent survey of card game AI techniques |

---

## Appendix: Yaku Progress as a Feature Vector

For solver implementations, the game state can be compactly represented by tracking progress toward each yaku. This "yaku progress vector" is a natural feature for both heuristic evaluation and neural network input:

| Yaku | Required | Tracking State |
|------|----------|---------------|
| Goko (Five Brights) | 5 specific cards | Count of Brights captured (0--5) |
| Shiko (Dry Four) | 4 non-Rain Brights | Count of non-Rain Brights (0--4) |
| Ame-Shiko (Rainy Four) | Rain Man + 3 others | Rain Man captured (bool) + other Brights (0--4) |
| Sanko (Three Brights) | 3 non-Rain Brights | Count of non-Rain Brights (0--4) |
| Inoshikacho | 3 specific Animals | Boar + Deer + Butterflies (0--3) |
| Tane (Animals) | 5+ Animals | Animal count (0--9) |
| Akatan (Red Poetry) | 3 specific Ribbons | Jan + Feb + Mar poetry ribbons (0--3) |
| Aotan (Blue) | 3 specific Ribbons | Jun + Sep + Oct blue ribbons (0--3) |
| Tanzaku (Ribbons) | 5+ Ribbons | Ribbon count (0--10) |
| Kasu (Plains) | 10+ Plains | Plain count (0--24) |
| Hanami-zake | Curtain + Sake Cup | 2 specific cards (0--2) |
| Tsukimi-zake | Full Moon + Sake Cup | 2 specific cards (0--2) |

Both the player's own progress and the opponent's visible progress (from captured cards) should be tracked. The difference vector indicates relative advantage and informs both card selection and koi-koi decisions.

This compact representation (approximately 24 integers for both players combined) captures the strategically relevant state far more efficiently than raw card enumeration, and is amenable to both linear and neural evaluation functions.
