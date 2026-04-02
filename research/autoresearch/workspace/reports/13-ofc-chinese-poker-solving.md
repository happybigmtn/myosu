# Solving Strategies for Open-Face Chinese Poker (OFC)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, 2014--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Open-Face Chinese Poker (OFC) and its dominant variant, Pineapple OFC, occupy a unique niche in the game-solving landscape. Unlike traditional poker variants where the core challenge is betting strategy under hidden card information, OFC is fundamentally a **sequential card placement optimization problem**: players must irrevocably assign cards to three rows (front/3 cards, middle/5, back/5) over multiple turns, maximizing royalty bonuses and row-win scoring while avoiding the catastrophic penalty of fouling (violating the constraint that back >= middle >= front in hand strength). The game tree is estimated at 10^40--10^60 nodes for 2-player Pineapple OFC, with 10^30--10^45 information sets arising primarily from hidden discards and unknown draw cards.

The dominant solving approach remains **Monte Carlo simulation** with fast hand evaluation, a method that has powered every practical OFC solver since Kachushi (2014). The key insight is that each placement decision can be evaluated by sampling completions of the remaining deck, simulating the remaining turns, and computing expected value across royalties, row wins, scoops, foul penalties, and Fantasyland qualification. More recent work (Stanford CS224R, 2024) has applied deep reinforcement learning methods (DQN, PPO) and MCTS with RAVE heuristics, finding that **MCTS significantly outperforms pure RL approaches** in OFC, achieving 89% win rates against baseline agents. The commercial Oleg Ostroumov solver (released publicly April 2024, previously $30k+) represents the state of the art in practical OFC solving, supporting Classic, Pineapple Progressive, and Ultimate variants.

No published Nash equilibrium computation exists for full OFC. The game's sequential placement structure, combined with the foul constraint creating hard dependencies between rows, makes the standard CFR approach poorly suited without heavy abstraction. Fantasyland placement (arranging 13--17 known cards optimally) is a solved subproblem tractable via brute-force search. The core open problem remains full-game optimal play during the sequential drawing phase, where the interaction between Fantasyland-chasing aggression, foul risk management, and opponent-adaptive card tracking creates a decision space that resists clean decomposition.

---

## Game Complexity Analysis

### Sequential Placement Structure

OFC's complexity arises not from betting decisions (there are none) but from the **irrevocable sequential placement** of cards into a constrained three-row structure. Each placement decision has cascading consequences:

1. **Initial 5-card placement**: C(52,5) = 2,598,960 possible deals. For each deal, the player must distribute 5 cards across three rows. The number of legal initial placements is approximately 232 distinct configurations per deal (Kachushi analysis), considering the 3-5-5 row capacity constraint.

2. **Subsequent Pineapple draws**: Each turn deals 3 cards; the player keeps 2 and discards 1 face-down. The placement decision involves choosing which 2 of 3 cards to keep, then assigning each to a legal row. Branching factor per round: approximately 6--15 options (placement choices x discard choice).

3. **Foul constraint propagation**: Every placement must maintain the *possibility* of completing a valid hand. Placing a strong card in the front row commits the player to finding even stronger cards for middle and back -- a constraint that narrows future options with each turn.

### Information Structure

| Property | Standard OFC | Pineapple OFC |
|----------|-------------|---------------|
| Placed cards | Fully visible to all | Fully visible to all |
| Discarded cards | N/A | Hidden (face-down) |
| Remaining deck | Unknown | Unknown |
| Information type | Near-perfect | Imperfect |

Standard OFC is nearly a perfect-information game: all placed cards are visible, and the only unknown is the order of cards remaining in the deck. Pineapple OFC introduces genuine imperfect information through face-down discards. Each player discards 4 cards over the course of a hand, creating hidden information about which specific cards have been removed from play. While this is a small amount of hidden information compared to Hold'em, it meaningfully affects probability calculations -- a player tracking live cards cannot distinguish between a card that was discarded and one that remains in the deck.

### Game Tree Estimates

| Metric | Estimate | Notes |
|--------|----------|-------|
| Possible starting hands (per player) | C(52,10) ~ 1.58 x 10^10 | All cards a single player will see |
| Placements per starting hand | ~200+ | Initial 5-card distribution alone |
| Total game tree (2P Pineapple OFC) | ~10^40--10^60 | Estimated; no rigorous computation |
| Information sets (2P Pineapple OFC) | ~10^30--10^45 | Primarily from hidden discards |
| Branching factor per turn | ~6--15 | Placement options x discard choice |
| Turns per hand | 5 (initial + 4 Pineapple draws) | Per player |

For comparison: heads-up no-limit Hold'em has ~10^161 game tree nodes but benefits from clean subgame decomposition and betting abstraction. OFC's tree is orders of magnitude smaller, but the sequential placement constraint and multi-row optimization resist the abstraction techniques that make Hold'em tractable.

### Scoring Complexity

OFC's scoring system creates a multi-objective optimization problem:

- **Row wins**: +1/-1 per row won/lost against each opponent, with scoop bonus (+6/-6 for winning/losing all three rows).
- **Royalties**: Bonus points for specific hand strengths per row (e.g., front pair of Aces = 9 pts, middle flush = 8 pts, back straight = 2 pts). Middle row royalties are doubled relative to back row.
- **Foul penalty**: -6 points per opponent (equivalent to being scooped), plus forfeiture of all royalties. This creates a discontinuous penalty that dominates the scoring landscape.
- **Fantasyland qualification**: Achieving QQ+ in the front row (without fouling) triggers Fantasyland on the next hand, which is worth substantial expected value (estimated 8--15 points per hand advantage in Pineapple).

The interaction between these scoring components means that expected value calculation for each placement must integrate over: (a) probability of completing each row to various hand strengths, (b) probability of fouling, (c) probability of entering/staying in Fantasyland, (d) opponent row strengths, and (e) royalty collection across all three rows.

---

## Current Best Approaches

### 1. Monte Carlo Simulation (Primary Practical Method)

Monte Carlo simulation has been the backbone of every published OFC solver since 2014. The approach is conceptually simple:

**Algorithm:**
1. For each legal placement option at the current decision point:
   a. Sample N completions of the remaining deck (accounting for visible cards on all boards).
   b. For each sample, simulate the remaining turns using a rollout policy (often random or heuristic-guided).
   c. Evaluate the final board state: compute row wins, royalties, foul status, and Fantasyland qualification.
   d. Average the scores across all N samples to estimate expected value.
2. Select the placement with the highest expected value.

**Hand evaluation**: All practical OFC solvers use fast poker hand evaluation algorithms, typically variants of Cactus Kev's evaluator, which maps 5-card hands to integer strength values via lookup tables. These have been extended to handle 3-card front-row evaluation (high card, pair, trips only). Holz specifically uses a modified Cactus Kev algorithm for both 3-card and 5-card evaluation.

**Strengths:**
- No domain-specific strategy knowledge required; EV calculation captures all strategic considerations implicitly.
- Easily parallelizable; solver speed scales linearly with CPU cores (demonstrated by OFCSolver on GitHub).
- Handles the multi-objective scoring naturally by computing full scoring at rollout terminals.
- Adapts to visible information (opponent boards, known dead cards) by constraining the sampling distribution.

**Weaknesses:**
- Rollout quality limits accuracy. Random rollouts (as in Kachushi) assume the player and opponents play randomly in future turns, which systematically misevaluates positions where skilled future play matters. Kachushi's author noted this explicitly: the bot "models itself in future turns to be completely random, meaning it cannot think very far ahead."
- Convergence requires large sample counts. Practical solvers run thousands to hundreds of thousands of rollouts per decision, creating latency constraints for real-time play.
- Does not account for opponent strategy adaptation; treats opponent future play as random or fixed.

**Notable implementations:**
- **Kachushi** (2014, Haskell): Pure Monte Carlo with Cactus Kev hand evaluation. Tested over 10,000 four-player games. Treats opponents as random -- a major simplification.
- **Holz** (GitHub, Rust): EV simulator for standard OFC. Uses modified Cactus Kev for fast evaluation. Standard OFC only, not Pineapple.
- **PyOFC** (GitHub, Python): Command-line OFC AI with real-time Monte Carlo simulation. Reported to beat beginner and some intermediate human players.
- **OFCSolver** (GitHub): Pineapple OFC Monte Carlo solver with linear core scaling. Companion project PythonOFCSimulator handles late-game EV computation.

### 2. Monte Carlo Tree Search with RAVE (Best Research Results)

The Stanford CS224R project (2024) demonstrated that MCTS with Rapid Action Value Estimation (RAVE) significantly outperforms pure RL methods for OFC. This represents the strongest published research result on OFC solving.

**Algorithm:**
MCTS builds a search tree rooted at the current game state, using the UCT (Upper Confidence bounds applied to Trees) selection policy to balance exploration and exploitation. At each node, the algorithm selects the most promising action, simulates to a terminal state, and backpropagates the result.

RAVE augments standard MCTS by generalizing across subtrees: the value of an action in one state is used to inform the value estimate of the same action in sibling states. The key assumption -- that the value of placing a specific card in a specific row is roughly similar across nearby game states -- is well-suited to OFC, where card placement values are relatively stable within a given hand configuration.

**Results:**
- 89% win rate against baseline agents.
- 11.2 average points per game.
- Significantly outperformed Q-Learning, Deep Q-Learning, and PPO.

**Why MCTS works well for OFC:**
- The moderate branching factor (~6--15 per turn) is manageable for tree search.
- The game has only 5 decision points per player per hand, creating shallow trees relative to games like Go or Stratego.
- RAVE's generalization across states is well-suited to card placement, where the value of "place Ace of spades in the back row" is informative across many game states.
- MCTS handles the stochastic nature of future draws naturally through sampling.

**Limitations:**
- Computationally expensive relative to pure Monte Carlo; the tree search overhead limits the number of rollouts per decision.
- The Stanford work used relatively simple rollout policies; combining MCTS with learned value functions could improve quality but adds training complexity.
- Tested against baseline agents, not against strong human players or established commercial solvers.

### 3. Deep Reinforcement Learning (Emerging but Underperforming)

Multiple RL approaches have been applied to OFC, with mixed results.

**Tan & Xiao (2018), "Mastering Open-Face Chinese Poker by Self-Play":**
The earliest published deep RL work on OFC. Used self-play reinforcement learning where the agent plays against copies of itself and iteratively improves its policy. This work established the baseline for RL approaches to OFC and is cited by most subsequent research.

**Stanford CS224R (2024), Multi-method comparison:**
Tested Q-Learning, Deep Q-Learning (DQN), PPO, and MCTS on OFC Pineapple.

| Method | Win Rate | Avg Points/Game | Notes |
|--------|----------|----------------|-------|
| MCTS + RAVE | 89% | 11.2 | Best overall |
| PPO | ~70-75% | ~6-8 | Self-play training |
| DQN | ~60-70% | ~4-6 | Moderate variance |
| Q-Learning | ~55-65% | ~3-5 | Tabular; limited by state space |
| Random | ~25% | baseline | Control |

PPO showed promise through self-play training but suffered from high variance and sensitivity to reward shaping. The researchers noted that OFC's three-tiered structure makes standard RL state representation challenging -- the agent must simultaneously track the state of three developing poker hands, the remaining deck composition, and opponent boards.

**Key challenges for RL in OFC:**
- **State representation**: Encoding three partial poker hands, visible opponent cards, and remaining deck composition into a fixed-size feature vector is non-trivial. No standard card encoding (one-hot, rank-suit matrices) captures the multi-row constraint naturally.
- **Reward shaping**: The delayed, multi-component reward (row wins + royalties + foul penalty + Fantasyland bonus) makes credit assignment difficult. The catastrophic foul penalty creates a discontinuity that RL optimizers struggle with.
- **Sample efficiency**: OFC games produce only 5 decision points per player per hand, making data collection slow relative to games with more decisions per episode.

### 4. Commercial Solvers (State of the Art in Practice)

**Oleg Ostroumov OFC Solver** (ofc.olegsolvers.com, April 2024):
Released publicly after being available exclusively to high-stakes players for $30,000+. Supports Classic OFC, Pineapple Progressive, and Ultimate with Jokers. Key features:
- "Optimal" strategy that matches existing OFC Classic solvers.
- Training mode with top moves list.
- Non-UTG 5-card moves approximate in free version; precise in paid version.
- Pineapple solver updates added post-launch with improved precision.

Ostroumov previously created the world's first commercial Hold'em solver and sold it for $500k. The OFC solver's commercial provenance and matching of known optimal strategies for Classic OFC suggests it represents the practical state of the art.

**PokerBotAI PokerX** (2025):
Commercial bot with OFC support added in 2025. Uses the "TriBrain Engine" combining a neural network trained on 7+ billion synthetic hands, expert algorithms for specific situations, and hand history data. Specific OFC training involved millions of OFC hands and fine-tuning with experienced OFC players. Technical details are proprietary.

### 5. Heuristic / Rule-Based Systems

Expert-encoded heuristic systems remain competitive for OFC, particularly for initial placement decisions.

**Yakovenko's initial placement framework** (2014):
Nikolai Yakovenko (poker pro and AI researcher, co-author of Poker-CNN) published strategy analysis for the first 5-card placement in OFC. Key heuristic principles:
- Prioritize flexibility over immediate hand strength.
- Place cards that preserve multiple strong completion paths across all three rows.
- When holding a Queen+ pair, aggressively place it in the front row for Fantasyland qualification, even at increased foul risk.
- Track live card counts to assess completion probability for flush draws, straight draws, and pair-up opportunities.

**Isabelle "No Mercy" Mercier's OFC strategy framework** (ofcstrategy.com):
Comprehensive heuristic strategy covering all decision points, with emphasis on Fantasyland qualification, royalty optimization, and foul avoidance. While not a computational solver, this framework represents the strongest published human-expert heuristic for OFC play.

---

## Placement Strategy: Row-by-Row Analysis

### Back Row (Bottom, 5 cards)

The back row must be the strongest of the three hands. Strategic priorities:
- **Primary goal**: Build a hand strong enough to dominate the opponent's back row while qualifying for royalties (straight = 2 pts, flush = 4 pts, full house = 6 pts, quads = 10 pts).
- **Fantasyland stay condition**: Quads or better in the back row allows re-entry to Fantasyland.
- **Common error**: Over-committing to back row strength early, leaving insufficient cards for middle and front rows.
- **Key heuristic**: A flush draw in the back row is extremely valuable because it simultaneously provides royalties (4 pts) and row-win probability. Flush draws should generally be preserved unless a clearly superior alternative exists.

### Middle Row (5 cards)

The middle row sits between the constraints of back and front. Royalties are doubled relative to back row values.
- **Primary goal**: Build a hand strong enough to beat the front row while remaining weaker than the back.
- **Fantasyland stay condition**: Full house or better allows Fantasyland re-entry.
- **Royalty value**: Because middle royalties are doubled (flush = 8 pts, full house = 12 pts), a strong middle row can generate enormous scoring swings.
- **Key tension**: A strong middle row constrains the back row (which must be at least as strong) and the front row (which must be weaker). Over-investment in the middle row is the most common source of fouls.

### Front Row (Top, 3 cards)

The front row is the most strategically consequential despite being the smallest. Only three hand types are possible: high card, pair, trips.
- **Primary goal**: Qualify for Fantasyland (QQ+ pair) or maximize royalty value (6s+ pair = 1--9 pts, trips = 10--22 pts).
- **Fantasyland as the dominant strategic consideration**: The expected value of entering Fantasyland (estimated 8--15 points per hand advantage) far exceeds the royalty value of most front-row improvements. This means chasing QQ+ in the front row is almost always correct when it does not create excessive foul risk.
- **Key tradeoff**: Placing a high pair (e.g., Aces) on the front row generates 9 royalty points but severely constrains the middle and back rows. Placing Queens on the front is safer because the middle row need only exceed a pair of Queens.
- **Live card tracking**: If 2+ copies of a specific rank are already visible on opponent boards, the probability of pairing that rank in the front drops dramatically. This makes real-time card counting essential for front-row placement decisions.

### Cross-Row Optimization

The fundamental challenge of OFC placement is that the three rows are interdependent through the foul constraint. A card placed in one row affects the feasibility space for all other rows. Effective solvers must evaluate placements holistically rather than row-by-row.

**Monte Carlo approaches handle this naturally**: by simulating complete hands for each placement option, they capture the full interaction between rows. Heuristic approaches struggle with cross-row optimization because the interactions are combinatorial and context-dependent.

---

## Fantasyland: Strategy and Game-Theoretic Implications

### Entry Strategy

Fantasyland entry (QQ+ pair in front, non-fouling) is the single most important strategic element in Pineapple OFC. The advantage of seeing all cards at once and placing the complete hand privately is estimated at 8--15 points per hand in Pineapple.

**Progressive entry rewards:**

| Front Row | Cards Dealt in Fantasyland |
|-----------|---------------------------|
| QQ | 14 |
| KK | 15 |
| AA | 16 |
| Trips | 17 |

More cards dealt means more flexibility for optimal placement and higher probability of staying in Fantasyland, creating a compounding advantage.

**Optimal Fantasyland-chasing heuristics:**
- When holding a single Queen+ early in the hand, placing it in the front row is often correct even when alternative placements would improve immediate row strength. The probability of pairing up (hitting one of the remaining 3 cards of that rank) must be weighed against the massive Fantasyland EV.
- Risk calculation: Foul probability if chasing Fantasyland = f. Fantasyland EV = V_FL. Current hand EV without Fantasyland chase = V_safe. Chase is correct when: (1-f) * V_FL > V_safe + f * 6 (foul penalty). In practice, chasable positions arise frequently enough that aggressive Fantasyland pursuit is a dominant strategy at all skill levels.

### Fantasyland Placement (Solved Subproblem)

Once in Fantasyland, the player receives 13--17 cards and must arrange 13 optimally (discarding extras). This is a **pure combinatorial optimization problem** with no sequential uncertainty:

- **13 cards**: C(13,3) * C(10,5) * C(5,5) / constraint validation = manageable search space. Must check all valid (non-fouling) arrangements and select the one maximizing expected score against opponents.
- **14--17 cards**: C(n,13) possible subsets of 13 cards to keep, multiplied by arrangement options per subset. Still tractable via brute-force with pruning.

Published Fantasyland solvers (ofcsolver.com, OFC Fantasyland Calculator) solve even 17-card Fantasyland optimally in under 1 second. The optimization objective combines: row-win probability against opponents' developing hands, royalty maximization, and Fantasyland stay probability (trips in front, full house+ in middle, or quads+ in back).

### Fantasyland Stay (Compounding Advantage)

Stay conditions (any one sufficient):
- Front: Three of a kind
- Middle: Full house or better
- Back: Four of a kind or better

Staying in Fantasyland creates a **compounding advantage**: the player sees all cards each hand while opponents play sequentially. A player who stays in Fantasyland for N consecutive hands accumulates an expected advantage of approximately N * (8--15) points. This makes Fantasyland stay optimization a critical subproblem.

**Game-theoretic implication**: The Fantasyland mechanic creates strong positive feedback loops. A player who enters Fantasyland gains a substantial edge, which makes staying in Fantasyland easier (because optimal placement is trivial with full information), which further compounds the advantage. This means Fantasyland entry probability is disproportionately important in long-term EV calculations.

---

## Fouling: Risk Management and Probability

### The Foul as Discontinuous Penalty

Fouling (violating back >= middle >= front hand strength ordering) results in:
- Loss of all three rows to every non-fouling opponent (equivalent to being scooped: -6 points per opponent).
- Forfeiture of all royalties.
- No Fantasyland qualification.

This creates a **discontinuous loss function**: a hand that is one card away from fouling may be worth +10 points (strong royalties, Fantasyland), while the fouled hand is worth -6 points. The ~16 point swing makes foul probability the dominant factor in placement EV calculations.

### Foul Probability Computation

For a given partial board state, foul probability can be estimated via Monte Carlo simulation:
1. Sample remaining cards from the deck (excluding visible cards on all boards).
2. Simulate remaining placements using a rollout policy.
3. Count the fraction of simulations that result in a foul.

**Practical risk threshold**: Expert players and solvers typically accept foul risk up to approximately 15--25% when the potential payoff includes Fantasyland qualification. Without Fantasyland upside, foul risk tolerance drops to approximately 5--10%.

**EV formula incorporating foul risk** (from Card Player magazine analysis):
```
EV = P(no_foul) * E[score | no_foul] + P(foul) * (-6 * num_opponents)
```
Where the conditional EV given no foul includes royalties, row wins, and Fantasyland qualification value.

---

## Comparison to Regular Chinese Poker Solving

Regular (closed) Chinese Poker and OFC differ fundamentally:

| Property | Regular Chinese Poker | Open-Face Chinese Poker |
|----------|----------------------|------------------------|
| Card visibility | All 13 cards seen at once | Sequential revelation |
| Placement | Simultaneous, private | Sequential, public |
| Decision points | 1 (arrange 13 cards) | 5 per hand (Pineapple) |
| Foul risk | Low (see all cards) | High (commit before seeing all) |
| Information type | Perfect (own hand) | Imperfect (future draws, discards) |
| Optimal play | Combinatorial optimization | Sequential decision under uncertainty |
| Solver approach | Brute-force / enumeration | Monte Carlo / MCTS / RL |

Regular Chinese Poker is a single-shot combinatorial optimization problem solvable by exhaustive enumeration or integer programming. The player sees all 13 cards and must find the arrangement maximizing expected score. This is computationally tractable and effectively solved for practical purposes.

OFC transforms this into a multi-step sequential decision problem where the player commits to placements before seeing future cards. The irrevocability of placements and the foul constraint create a fundamentally different optimization landscape. Fantasyland in OFC effectively gives the player one hand of regular Chinese Poker -- which is precisely why it is so valuable.

---

## Open Problems

### 1. Nash Equilibrium Computation

No published work computes a Nash equilibrium for any variant of OFC. The game tree size (10^40--10^60) is within the range where CFR variants have been applied to poker (HULHE: ~10^17 information sets; OFC: ~10^30--10^45), but OFC's structure resists standard CFR application:
- The sequential placement structure does not decompose into clean subgames.
- The foul constraint creates hard dependencies across game states.
- Pineapple discards introduce hidden information that increases information set count.
- Standard poker abstractions (card bucketing, action abstraction) do not transfer cleanly to OFC's placement-based action space.

### 2. Opponent Modeling in OFC

All published OFC solvers treat opponents as either random or fixed-strategy. In practice, OFC strategy is heavily influenced by opponent tendencies:
- Observing an opponent chase Fantasyland aggressively changes the optimal defensive strategy.
- Card blocking (placing cards that reduce opponent completion probabilities) is a real strategic consideration that no published solver exploits.
- Opponent foul probability affects the value of defensive play (if the opponent will likely foul, aggressive play becomes less necessary).

### 3. Optimal Pineapple Discard Strategy

The discard decision in Pineapple OFC is understudied. The discard serves dual purposes:
- Remove the weakest card for the player's own hand development.
- Potentially hide information from opponents (the discard is face-down).
- In theory, a player could strategically discard cards that would help opponents, at a cost to their own hand development. The EV tradeoff of "information hiding" vs. "hand quality" via discard choice is not well characterized.

### 4. Scalable RL for OFC

While the Stanford CS224R work showed MCTS outperforming pure RL, the RL approaches tested were relatively basic (DQN, tabular Q-Learning, basic PPO). More sophisticated approaches remain unexplored:
- **AlphaZero-style MCTS + neural value network**: Combining MCTS search with a learned value function (as in AlphaGo/AlphaZero) could dramatically improve both search quality and sample efficiency.
- **Transformer-based state encoding**: The multi-row, variable-length card configuration of OFC boards could benefit from attention-based architectures that capture cross-row dependencies.
- **Population-based training**: Methods like PSRO could discover diverse OFC strategies rather than converging to a single policy.

### 5. Real-Time Solving Under Computational Constraints

Practical OFC play requires decisions within seconds. The tradeoff between computation time and solution quality is poorly characterized. How many Monte Carlo rollouts are needed for near-optimal play? How does MCTS + RAVE performance degrade as the computation budget decreases? These questions have practical implications for deploying OFC solvers in real-time play environments.

### 6. Multi-Player OFC

Most research focuses on 2-player OFC. With 3+ players, the game becomes significantly more complex:
- More visible cards provide more information for card counting.
- Scoring is pairwise (each player vs. each opponent), creating a more complex optimization objective.
- Coalition dynamics may emerge in 3-player games, though OFC's zero-sum scoring structure limits this.

---

## Relevance to Myosu

### Architecture Fit

OFC presents a **distinct solving challenge** compared to the betting-based poker variants that dominate the Myosu game survey. While NLHE and PLO solving centers on bet sizing, range construction, and opponent exploitation through CFR, OFC solving centers on sequential placement optimization, multi-objective scoring, and constraint satisfaction (foul avoidance).

| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 2/5 | Game tree too large for vanilla CFR; structure resists standard poker abstractions |
| Neural value network potential | 4/5 | Placement value estimation is naturally learnable from self-play data |
| MCTS applicability | 5/5 | Moderate branching factor, shallow depth, stochastic -- ideal MCTS domain |
| Abstraction necessity | 4/5 | Full-game solving requires abstraction; Fantasyland subgame does not |
| Real-time solving value | 4/5 | MC + learned evaluation enables fast per-placement decisions |
| Transferability | 2/5 | Sequential placement is somewhat unique; card tracking skills transfer |

### Subnet Architecture Recommendations

1. **MCTS + learned value network** as the primary solving method. The Stanford results showing MCTS + RAVE outperforming pure RL, combined with the moderate branching factor and shallow game depth, make this the highest-ROI approach.

2. **Exact Fantasyland solver** as a required submodule. Fantasyland placement is tractable via brute-force and represents a clean, verifiable optimization problem.

3. **Monte Carlo EV estimation** for real-time play. Fast hand evaluation (Cactus Kev variants) combined with deck sampling provides a reliable fallback when MCTS computation budget is limited.

4. **Card tracking as a first-class feature**. Unlike Hold'em where card removal effects are secondary, OFC card tracking (knowing which cards are live based on all visible boards) is a primary strategic input that must be tightly integrated into the evaluation pipeline.

### Evaluation Metrics

OFC provides unusually clean evaluation metrics for a Myosu subnet:
- **Points per hand** (aggregate of row wins, royalties, scoops, foul penalties).
- **Foul rate**: Percentage of hands where the agent fouls. Expert foul rate is approximately 5--10%.
- **Fantasyland entry rate**: Frequency of qualifying for Fantasyland. Higher is better, but must be weighed against foul rate.
- **Fantasyland stay rate**: Once in Fantasyland, how often the agent stays. Expert rates vary by variant but 30--50% is strong.
- **Royalty income per hand**: Average royalty points earned, measuring hand quality.
- **Scoop rate**: Frequency of winning all three rows against an opponent.

These metrics are all deterministic and verifiable from game logs, making OFC well-suited to Myosu's oracle-based verification model.

---

## Key Papers and References

### Academic Research

| Year | Authors / Title | Contribution |
|------|----------------|--------------|
| 2014 | Kachushi (blog post) | First published OFC AI; Monte Carlo with Cactus Kev hand evaluation; Haskell implementation |
| 2014 | Yakovenko, "Strategy for the First Five Cards in OFC" (PokerNews) | Expert heuristic framework for initial placement; later co-authored Poker-CNN (AAAI 2016) |
| 2017 | Govindaiah, "Solving Pineapple: An Application of MCTS and Investigation of Selection Heuristics" (Princeton senior thesis) | First MCTS application to Pineapple OFC; pruning methods; tested against commercial AI |
| 2018 | Tan & Xiao, "Mastering Open-Face Chinese Poker by Self-Play Reinforcement Learning" | First deep RL approach to OFC; self-play training framework |
| 2023 | ArXiv 2312.09455, "Integration of Robotics, CV, and Algorithm Design: A Chinese Poker Self-Playing Robot" | Computer vision + algorithm integration for physical Chinese Poker play |
| 2024 | Stanford CS224R project, "Advancing Multi-Agent Reasoning in Open-Face Chinese Poker" | Comprehensive comparison of DQN, PPO, MCTS+RAVE; MCTS achieves 89% win rate |
| 2025 | Nature Scientific Reports, "Comparative Analysis of Extensive Form Zero Sum Game Algorithms for Poker-like Games" | Broader framework comparison including OFC-relevant methods |

### Practical Solvers and Tools

| Name | Type | URL | Notes |
|------|------|-----|-------|
| Oleg Ostroumov OFC Solver | Commercial | ofc.olegsolvers.com | State of the art; Classic + Pineapple + Ultimate |
| PokerBotAI PokerX | Commercial bot | pokerbotai.com | Neural network + expert system; OFC support added 2025 |
| OFCSolver | Open-source (GitHub) | github.com/neery1218/OFCSolver | Monte Carlo Pineapple solver; linear core scaling |
| Holz | Open-source (GitHub) | github.com/dtrifuno/holz | Rust EV simulator; standard OFC only |
| PyOFC | Open-source (GitHub) | github.com/wesny/pyofc | Python CLI client + MC AI |
| OFC Fantasyland Solver | Web tool | ofcsolver.com | Brute-force optimal Fantasyland placement |
| Pineapple Prodigy | Hackathon project (Devpost) | devpost.com/software/pineapple-prodigy | Probability + game theory simulation framework |

### Strategy Resources

| Resource | Author | Notes |
|----------|--------|-------|
| ofcstrategy.com | Isabelle "No Mercy" Mercier | Comprehensive heuristic strategy; tutorial series |
| Red Chip Poker OFC Odds Charts | Red Chip Poker | Pineapple OFC probability reference tables |
| Card Player EV Calculations article | Card Player Magazine (2014) | Practical EV calculation methodology with worked examples |
| Wizard of Vegas OFC thread | Community | Mathematical discussion of optimal OFC strategy |

### Foundational Methods (Not OFC-Specific)

| Year | Authors / Title | Relevance to OFC |
|------|----------------|-----------------|
| 2007 | Zinkevich et al., "Regret Minimization in Games with Incomplete Information" | CFR framework; limited applicability to OFC due to game structure |
| 2014 | Tammelin, "Solving Large Imperfect Information Games Using CFR+" | CFR+ speedup; relevant if abstracted OFC variants are attempted |
| 2016 | Yakovenko et al., "Poker-CNN: A Pattern Learning Strategy for Making Draws and Bets" (AAAI) | CNN-based poker hand evaluation; applicable to OFC state encoding |
| 2017 | Moravcik et al., "DeepStack: Expert-Level AI in No-Limit Poker" | Deep learning + continual re-solving; architecture concepts transferable to OFC |
| 2019 | Brown & Sandholm, "Solving Imperfect-Information Games via Discounted Regret Minimization" | Discounted CFR; potentially applicable to abstracted OFC |
