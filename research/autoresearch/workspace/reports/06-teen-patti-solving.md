# Solving Strategies for Teen Patti (3-Card Indian Poker)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, game-theoretic analysis, and applicability of modern algorithms
**Status:** Research report (no implementation)

---

## Executive Summary

Teen Patti ("Three Cards") is a South Asian gambling card game derived from British Three Card Brag, played by hundreds of millions across the Indian subcontinent. Despite its enormous player base, the game has received almost zero formal game-theoretic or computational treatment in the academic literature. No published Nash equilibrium computation exists for the full multiplayer game with blind/seen dynamics, and commercial Teen Patti apps rely on hand-strength heuristics rather than equilibrium strategies.

This absence represents both a gap and an opportunity. Teen Patti's state space is dramatically smaller than Texas Hold'em -- 22,100 raw 3-card hands (~455 strategically distinct) versus 2.6 million flop+hand combinations in NLHE -- making it tractable for exact solution in the 2-player case using standard CFR. The game's unique blind/seen asymmetry, variable-length betting rounds without fixed streets, and sideshow mechanic introduce modeling challenges absent from Western poker variants, but none of these are computationally prohibitive.

The recommended approach for Myosu: solve 2-player Teen Patti exactly via Discounted CFR (DCFR), then extend to 3-6 players with Monte Carlo CFR (MCCFR) and opponent modeling. The blind/seen decision should be modeled as an explicit game tree node. The game's low compute requirements make it an ideal entry point for subnet solver operators, and its cultural footprint provides a market that no existing solver technology serves.

---

## Game Complexity Analysis

### Hand Space

| Metric | Value | Notes |
|--------|-------|-------|
| Raw 3-card combinations C(52,3) | 22,100 | Full deck, order-independent |
| Strategically distinct hands (suit isomorphism) | ~455 | Collapsing equivalent suit permutations |
| Trail (three of a kind) | 52 combos (0.24%) | 13 ranks x 4 suit combos |
| Pure Sequence (straight flush) | 48 combos (0.22%) | 12 straights x 4 suits |
| Sequence (straight, mixed suit) | 720 combos (3.26%) | Minus the 48 pure sequences |
| Color (flush, non-sequential) | 1,096 combos (4.96%) | C(13,3) x 4 minus pure sequences |
| Pair | 3,744 combos (16.94%) | 13 ranks x C(4,2) x 11 kickers x 4 |
| High Card | 16,440 combos (74.38%) | Remainder |

The hand space is roughly 50x smaller than NLHE's preflop space (1,326 combos) combined with any single board texture, and roughly 5 orders of magnitude smaller than NLHE's full information-set count. This means tabular CFR -- no neural networks, no abstraction -- can represent the complete hand-level strategy space in memory.

### Game Tree Characteristics

| Property | Teen Patti | Kuhn Poker | Leduc Poker | NLHE HU |
|----------|-----------|------------|-------------|---------|
| Deck size | 52 | 3 | 6 | 52 |
| Cards per player | 3 | 1 | 1 (+1 community) | 2 (+5 community) |
| Raw hand combos | 22,100 | 3 | 6 | ~10^14 info sets |
| Strategic hands | ~455 | 3 | 6 | ~10^12 (abstracted) |
| Community cards | 0 | 0 | 1 | 5 |
| Betting streets | 0 (continuous) | 1 | 2 | 4 |
| Blind/seen choice | Yes | No | No | No |
| Sideshow mechanic | Yes | No | No | No |

### Why Teen Patti Is Simultaneously Simple and Hard

**Simple dimensions:**
1. Small hand space (455 strategic classes vs. millions in NLHE)
2. No community cards -- hand strength is fixed at deal time, no "board texture" concept
3. Limited action space per decision: bet (1x or 2x stake), fold, show, sideshow
4. Hand evaluation is a trivial 3-card comparison -- no complex 5-of-7 selection

**Hard dimensions:**
1. Variable-length betting without fixed streets -- the game tree has no natural depth bound
2. Blind/seen asymmetry creates two distinct player types with different cost structures occupying the same seat
3. Multiplayer dynamics (3-7 players) break two-player zero-sum guarantees
4. Sideshow creates private bilateral comparisons mid-hand that branch the game tree
5. The "when to look" decision is a continuous-time strategic choice with no analogue in standard poker theory

---

## Blind vs. Seen Dynamics: Game-Theoretic Analysis

The blind/seen mechanic is Teen Patti's most distinctive feature and the primary source of modeling complexity. It has no direct analogue in Western poker or in the standard game-theory benchmark games (Kuhn, Leduc, NLHE).

### The Mechanic

A player begins each hand "blind" (has not looked at their cards). At any point before acting, they may choose to become "seen" (look at their cards). This is irreversible. The consequences:

| Property | Blind Player | Seen Player |
|----------|-------------|-------------|
| Minimum bet | 1x current stake | 2x current stake |
| Maximum bet | 2x current stake | 4x current stake |
| Can request show | Only vs. other blind | Yes (must pay seen rate) |
| Can request sideshow | No | Yes (only with adjacent seen player) |
| Information | None about own hand | Full knowledge of own hand |

### Game-Theoretic Implications

**Cost asymmetry as information rent.** A blind player pays half the per-round cost of a seen player. This is effectively an "information rent" -- the seen player pays a premium for the privilege of knowing their hand. The equilibrium question is: when does the expected value of seeing your hand exceed the cost of doubling your betting obligations?

**Blind play as a commitment device.** A blind player credibly signals that their actions are uncorrelated with hand strength (since they don't know their hand). This has a bluff-like effect: opponents cannot extract information from a blind player's betting patterns. In game-theoretic terms, blind play collapses the player's information partition to a single set, making their strategy profile trivially mixed from the opponents' perspective.

**The "when to look" decision as optimal stopping.** The decision of when to transition from blind to seen can be modeled as an optimal stopping problem. The player accumulates information about opponents (fold patterns, bet sizes, remaining player count) at reduced cost while blind, and must choose the moment at which private information about their own hand becomes more valuable than the cost savings of blind play.

**Modeling approach.** The cleanest representation adds a "look" action at each blind player's decision node. The game tree branches: one subtree for "remain blind" (player still has a single information set covering all possible hands), one subtree for "look" (player's information set splits into one per hand). This doubles the branching factor at each blind player's node but keeps the representation standard for CFR.

### Formal Comparison to Known Games

The blind/seen mechanic is structurally closest to **phantom games** (e.g., Phantom Go, Phantom Tic-Tac-Toe, Kriegspiel), where a player can optionally choose to observe certain information at a cost. It is also related to **costly information acquisition** models in mechanism design, where agents decide whether to pay to learn their own type before participating.

No existing benchmark game in the imperfect-information game solving literature incorporates this mechanic. This makes Teen Patti a genuinely novel test case, not merely a rescaled version of Kuhn or Leduc poker.

---

## Current Best Approaches

### Approach 1: Tabular CFR for 2-Player Teen Patti

**Algorithm:** Vanilla CFR or CFR+ with full enumeration

**How it works:** Enumerate all 22,100 x 22,099/2 opponent hand matchups (minus dealt cards). Represent the game tree explicitly with blind/seen decisions as action nodes. Run regret-matching updates across all information sets until convergence.

**Strengths:**
- Guaranteed convergence to Nash equilibrium in 2-player zero-sum setting
- No abstraction needed -- the game is small enough for exact tabular representation
- Produces a provably optimal strategy, not an approximation
- Memory footprint is manageable: ~455 strategic hand classes x betting history states

**Weaknesses:**
- Limited to 2-player games (no convergence guarantees for 3+ players)
- Variable-length betting rounds require a depth bound or pot-limit cap to keep the tree finite
- Does not model opponent tendencies (plays Nash, not exploitative)

**Compute estimate:** Minutes to hours on a single CPU core, depending on tree depth bound. Comparable to solving Leduc poker, which is routine.

**Key reference:** Zinkevich et al., "Regret Minimization in Games with Incomplete Information" (NIPS 2007)

### Approach 2: Discounted CFR (DCFR)

**Algorithm:** DCFR applies time-dependent discount factors to both positive and negative cumulative regrets, weighting recent iterations more heavily than early ones.

**How it works:** Same tree traversal as vanilla CFR, but regret updates are multiplied by discount factors that decay contributions from early iterations. This accelerates convergence by preventing early exploratory noise from persisting in the strategy profile.

**Strengths:**
- State-of-the-art convergence speed for poker-like games (outperforms CFR+ on Kuhn, Leduc, and Royal Poker benchmarks)
- Directly applicable to Teen Patti with zero modification
- Lowest exploitability among tested algorithms in the 2025 Keshavarzi & Navidi comparative study

**Weaknesses:**
- Still limited to 2-player zero-sum for theoretical guarantees
- Requires tuning of discount parameters (though defaults work well for poker-like games)

**Compute estimate:** Faster convergence than vanilla CFR by roughly 2-10x. 2-player Teen Patti solvable in minutes.

**Key reference:** Brown & Sandholm, "Solving Imperfect-Information Games via Discounted Regret Minimization" (AAAI 2019); Keshavarzi & Navidi, "Comparative analysis of extensive form zero sum game algorithms for Poker like games" (Scientific Reports, 2025)

### Approach 3: Monte Carlo CFR (MCCFR) for Multiplayer

**Algorithm:** MCCFR samples game tree trajectories instead of performing full traversals, enabling scaling to larger games and more players.

**How it works:** Instead of updating every information set per iteration, MCCFR samples a subset of chance outcomes and player actions. External sampling visits all opponent actions but samples chance; outcome sampling samples everything. The sampled counterfactual values are unbiased estimators of the true values.

**Strengths:**
- Scales to 3-7 player Teen Patti where full traversal is infeasible
- Per-iteration cost is dramatically lower than full CFR
- Well-studied in multiplayer poker settings (3-8 players in kdb-D2CFR work)

**Weaknesses:**
- Higher variance per iteration (convergence requires more iterations but each is cheaper)
- No convergence guarantee for 3+ player games (this is a fundamental limitation of all Nash-finding algorithms for multiplayer games, not specific to MCCFR)
- In practice, multiplayer MCCFR strategies are strong but not provably optimal

**Compute estimate:** Hours to days for 5-6 player Teen Patti, depending on tree depth and sampling scheme.

**Key references:** Lanctot et al., "Monte Carlo Sampling for Regret Minimization in Extensive Games" (NIPS 2009); kdb-D2CFR for multiplayer scaling (Knowledge-Based Systems, 2023)

### Approach 4: Deep CFR / Neural Function Approximation

**Algorithm:** Replace tabular strategy storage with neural networks that generalize across similar information sets.

**How it works:** Train neural networks to predict counterfactual values for each information set, using self-play data generated by CFR-like traversals. The networks generalize across similar game states, enabling handling of games too large for tabular storage.

**Strengths:**
- Enables scaling to the full multiplayer game with all variants
- Neural generalization may capture hand-similarity patterns that tabular methods treat independently
- Foundation for real-time subgame solving (DeepStack/Libratus paradigm)

**Weaknesses:**
- Overkill for 2-player Teen Patti (tabular methods are sufficient and exact)
- Training instability and hyperparameter sensitivity
- Approximation errors may exceed the gains from generalization in a game this small
- Significantly higher engineering and compute cost

**Compute estimate:** GPU-hours for training. Justified only for multiplayer variants with 5+ players.

**Key reference:** Brown et al., "Deep Counterfactual Regret Minimization" (ICML 2019)

### Approach 5: ReBeL (Recursive Belief-based Learning)

**Algorithm:** Combines self-play reinforcement learning with search over public belief states (PBS), treating imperfect-information games via a perfect-information abstraction over beliefs.

**How it works:** ReBeL maintains a probability distribution (PBS) over possible game states given public information. At each decision point, it runs depth-limited search using a value network trained by self-play. The PBS is updated using Bayesian inference from observed actions.

**Strengths:**
- General-purpose framework that handles any 2-player zero-sum game
- The PBS formulation naturally handles the blind/seen asymmetry (blind players have a uniform belief over their own hand; seen players have a point belief)
- Facebook/Meta released open-source implementation with Liar's Dice support
- Provably converges to Nash equilibrium

**Weaknesses:**
- Engineering complexity is substantially higher than tabular CFR
- Requires training a value network, which is unnecessary for a game this small
- No multiplayer extension with theoretical guarantees

**Compute estimate:** GPU-hours for training. Not recommended for 2-player Teen Patti but relevant as a demonstration of generality.

**Key reference:** Brown et al., "Combining Deep Reinforcement Learning and Search for Imperfect-Information Games" (NeurIPS 2020)

### Approach 6: LLM-Based Agents (Emerging, 2024-2025)

**Algorithm:** Fine-tune or prompt large language models to play poker-like games using natural language game state descriptions.

**How it works:** Systems like PokerGPT and SpinGPT convert game state into text prompts and use fine-tuned LLMs to generate actions. Training data comes from expert play or solver-generated strategies. PokerGPT uses RLHF on real game records; SpinGPT uses supervised fine-tuning on solver outputs followed by RL.

**Strengths:**
- Natural language interface for explainability and interaction
- Can handle arbitrary game variants without re-engineering the solver
- Low barrier to creating a "reasonable" agent

**Weaknesses:**
- Fundamentally cannot compute or verify equilibrium strategies
- Performance is bounded by training data quality, not by game-theoretic guarantees
- PokerBench (2025) shows that even GPT-4 makes systematic errors in poker decision-making
- Not suitable for competitive or high-stakes solver computation

**Relevance to Teen Patti:** LLM agents could serve as opponents for testing or as natural-language interfaces to a proper solver, but should not be the solver itself.

**Key references:** PokerGPT (arXiv 2401.06781, 2024); SpinGPT (arXiv 2509.22387, 2025); PokerBench (arXiv 2501.08328, 2025)

---

## The Sideshow Mechanic: Modeling Challenges

The sideshow is unique to Teen Patti and Three Card Brag. When three or more players remain, a seen player can request a private comparison with the previous seen bettor. The previous player may accept or refuse.

### Game Tree Impact

Sideshows create **bilateral subgames** embedded within the multiplayer game:
1. The requesting player pays the seen bet rate
2. The previous player decides accept/refuse (a binary choice with strategic implications)
3. If accepted, the weaker hand folds (deterministic outcome given both hands)
4. If refused, play continues with no information revealed

This creates a branching factor increase at every seen player's node (bet, bet+sideshow_request, or fold). The accept/refuse decision for the target player adds another binary branch. The private comparison is a deterministic function of two known hands, so no additional uncertainty is introduced -- but the accept/refuse decision itself is an information-carrying signal.

### Strategic Analysis

**Requesting sideshows:** Optimal for mid-strength hands. Strong hands prefer to stay in and win the full pot. Weak hands should fold rather than pay to compare. Mid-range hands (pairs, strong high cards) benefit from eliminating one opponent at reduced cost.

**Refusing sideshows:** Signals confidence (or bluff). A player with a strong hand may refuse to keep more opponents in and grow the pot. A player with a weak hand may refuse to avoid being exposed.

**Forced acceptance rule:** In many rulesets, the third consecutive sideshow request to the same player must be accepted. This caps the "refuse as bluff" strategy and should be modeled as a constraint in the game tree.

### Modeling Recommendation

Treat the sideshow as an additional action type at seen player nodes, with the accept/refuse response as a subsequent decision node for the target player. The private comparison outcome branches deterministically based on hand rankings. This increases tree size linearly per sideshow-eligible node, which is manageable given the game's otherwise small state space.

---

## Comparison to Solved / Benchmark Games

### Teen Patti vs. Kuhn Poker

| Dimension | Kuhn Poker | Teen Patti (2P) |
|-----------|-----------|-----------------|
| Deck | 3 cards | 52 cards |
| Cards per player | 1 | 3 |
| Strategic hand classes | 3 | ~455 |
| Information sets (2P) | 12 | ~10^4 - 10^6 (estimate, depends on tree depth) |
| Community cards | 0 | 0 |
| Betting rounds | 1 (fixed) | Variable (continuous) |
| Blind/seen | No | Yes |
| Nash equilibrium | Known analytically | Computable, not yet published |

Kuhn Poker's Nash equilibrium is known in closed form: Player 1 loses at a rate of -1/18 per hand. The equilibrium involves specific mixing frequencies (e.g., Player 2 bets with a Jack 1/3 of the time as a bluff). Teen Patti's equilibrium would follow similar principles -- bluffing with weak hands at computed frequencies, value-betting with strong hands -- but the strategy profile would be vastly larger and require numerical computation.

**Key insight:** Teen Patti occupies a complexity sweet spot between Kuhn Poker (trivially solvable, too simple to be interesting) and NLHE (requires massive abstraction and neural approximation). It is complex enough to exhibit rich strategic behavior (bluffing, blind play, sideshows) but small enough for exact computation.

### Teen Patti vs. Leduc Poker

Leduc Poker (6 cards, 2 betting rounds, 1 community card) is a standard CFR benchmark. Teen Patti is larger in hand space but lacks community cards, making the information-set structure different. Leduc has ~10^2 information sets; Teen Patti 2-player with bounded betting has ~10^4 - 10^6, depending on the depth bound. Both are well within tabular CFR's capacity.

### Teen Patti vs. Three Card Poker (Casino Game)

Casino Three Card Poker is a player-vs-dealer game with fixed rules: the dealer qualifies with queen-high or better, and the optimal strategy is simply "raise with Q-6-4 or better, fold otherwise." This has been fully solved with a house edge of ~3.4%. It is a decision problem, not a strategic game -- there is no opponent to model, no bluffing, no information dynamics. It bears almost no resemblance to Teen Patti's multiplayer strategic game despite the shared 3-card hand structure.

### Teen Patti vs. Three Card Brag

Three Card Brag is Teen Patti's direct ancestor and shares the blind/seen mechanic, though with rule differences (e.g., in Brag, if all but one player folds against a blind player, the pot remains and the blind player keeps their hand for the next round). No published game-theoretic analysis of Three Card Brag exists either, making the two games equally underserved by the research community.

---

## Open Problems

### 1. First Published Nash Equilibrium for 2-Player Teen Patti

No paper has computed and published the Nash equilibrium strategy for even the simplest 2-player variant. This is a straightforward application of DCFR with a bounded betting tree and would constitute a genuine contribution to the literature. The main modeling decision is how to handle the unbounded betting rounds (pot limit or round limit).

### 2. Optimal Blind-to-Seen Transition Timing

The "when to look" decision lacks any formal analysis. An optimal stopping formulation would characterize the expected value of transitioning as a function of: pot size, number of remaining opponents, betting history observed while blind, and cost differential. This is an open theoretical question with no existing treatment.

### 3. Multiplayer Equilibrium Concepts

For 3+ player Teen Patti, Nash equilibrium is no longer unique and may not be strategically meaningful (players can form implicit coalitions). Alternative solution concepts -- correlated equilibrium, quantal response equilibrium (QRE), or coarse correlated equilibrium -- may be more appropriate. The 2025 work on QRE for multiplayer Kuhn Poker (quadratic programming approach) is directly applicable.

### 4. Sideshow Equilibrium Analysis

The sideshow's accept/refuse dynamic creates a signaling game embedded within the main game. The equilibrium properties of this sub-mechanism (when to request, when to accept, when refusal is credible) have not been formally analyzed.

### 5. Variant-Specific Solutions

Each major variant (Muflis/lowball, AK47/wildcards, Best of Four) requires a separate solution. Muflis simply inverts the hand ranking -- the solver is identical but the evaluation function flips. AK47 introduces wild cards that expand the effective hand space. Best of Four adds a combinatorial selection step (choose 3 from 4). None have been analyzed.

### 6. Cross-Cultural Rule Normalization

Teen Patti rules vary significantly across regions, friend groups, and online platforms. There is no canonical ruleset comparable to the TDA rules for poker. A formal game-theoretic treatment requires first specifying exactly which rule variant is being solved. This normalization work is a prerequisite for meaningful solver comparison.

---

## Relevance to Myosu

### Strategic Value

Teen Patti represents Myosu's highest-value "low-hanging fruit" for several reasons:

1. **Unserved market.** No solver technology exists. Commercial apps use random number generators and basic heuristics. A provably optimal strategy engine would be the first of its kind.

2. **Massive player base.** Teen Patti Gold alone claims 6 million monthly active users. The total addressable market across all platforms and in-person play in South Asia is orders of magnitude larger.

3. **Low compute requirements.** 2-player solving requires only CPU time. Even multiplayer MCCFR is feasible on commodity hardware. This lowers the barrier for subnet solver operators compared to NLHE.

4. **Novel research contribution.** Publishing the first Nash equilibrium for Teen Patti (even the 2-player case) would be a genuine academic contribution, generating visibility for the Myosu project.

5. **Gateway game.** Teen Patti's simpler structure makes it an ideal onboarding game for the subnet, allowing operators to validate their solver infrastructure before tackling NLHE.

### Architecture Implications

| Component | Teen Patti Requirement | Complexity vs. NLHE |
|-----------|----------------------|---------------------|
| Hand evaluator | 3-card ranking (trivial) | ~100x simpler |
| Game tree representation | Variable-depth with blind/seen nodes | Novel structure, but small |
| Strategy storage | Tabular for 2P; neural for 5P+ | ~10^6x smaller (2P) |
| Abstraction | Not needed for 2P; minimal for MP | Eliminates major NLHE pain point |
| Real-time solving | Unnecessary for 2P; useful for MP | Much less critical |

### Recommended Implementation Roadmap

**Phase 1 (Quick Win):** Implement DCFR for 2-player Teen Patti with pot-limit betting cap. Target: exact Nash equilibrium in days of development, minutes of compute. Publish results.

**Phase 2 (Multiplayer):** Extend to 3-6 players with MCCFR. Model the blind/seen decision and sideshow as explicit game tree nodes. Accept that multiplayer strategies are approximate, not provably optimal.

**Phase 3 (Variants):** Add pluggable hand evaluators for Muflis, AK47, and Best of Four. The solver infrastructure is identical; only the evaluation function and (for Best of Four) the dealing/selection step change.

**Phase 4 (Exploitation):** Layer opponent modeling (ODCFR-style) on top of the Nash baseline. This is where the real-money value lies -- exploiting sub-optimal opponents rather than playing Nash against them.

---

## Key Papers & References

### Foundational CFR Literature

| Year | Authors | Title | Relevance |
|------|---------|-------|-----------|
| 2007 | Zinkevich et al. | Regret Minimization in Games with Incomplete Information | CFR algorithm -- the foundation for all poker solving |
| 2014 | Tammelin | Solving Large Imperfect Information Games Using CFR+ | CFR+ -- faster convergence, used to solve Limit Hold'em |
| 2019 | Brown & Sandholm | Solving Imperfect-Information Games via Discounted Regret Minimization | DCFR -- current state-of-the-art for poker-structure games |

### Landmark Poker AI Systems

| Year | System | Contribution |
|------|--------|-------------|
| 2017 | DeepStack (Moravcik et al.) | First superhuman NLHE HU agent; continual re-solving with neural networks |
| 2017 | Libratus (Brown & Sandholm) | Defeated top humans in NLHE HU; blueprint + subgame solving paradigm |
| 2019 | Pluribus (Brown & Sandholm) | First superhuman multiplayer (6-player) poker agent; MCCFR blueprint |
| 2020 | ReBeL (Brown et al., Meta AI) | Generalized RL+Search to imperfect-information games via public belief states |

### Recent Advances (2024-2026)

| Year | Authors/Title | Contribution |
|------|---------------|-------------|
| 2024 | Sonawane & Chheda, "A Survey on Game Theory Optimal Poker" (arXiv 2401.06168) | Comprehensive GTO poker survey covering abstraction, betting models, bot strategies |
| 2024 | MCCFVFP (NeurIPS 2024) | MC-based algorithm 20-50% faster than best MCCFR variants |
| 2024 | PokerGPT (arXiv 2401.06781) | LLM-based multiplayer poker solver; demonstrates feasibility of language model approach |
| 2025 | Keshavarzi & Navidi, Scientific Reports | Comparative analysis of 10 algorithms on Kuhn/Leduc/Royal Poker; DCFR wins on exploitability |
| 2025 | ODCFR (Knowledge-Based Systems) | Deep CFR with opponent modeling; overcomes Nash conservatism for increased profit |
| 2025 | SpinGPT (arXiv 2509.22387) | LLM for 3-player Spin & Go poker; supervised + RL training pipeline |
| 2025 | "Beyond GTO" (arXiv 2509.23747) | Profit-maximizing agents that outperform Nash in practice against human opponents |
| 2025 | QP approach for multiplayer NE (MDPI Games) | Quadratic programming for exact Nash in small multiplayer imperfect-info games |
| 2025 | kdb-D2CFR (Knowledge-Based Systems) | Knowledge-distillation-based DeepCFR for 3-8 player games |

### Teen Patti Specific

| Year | Source | Contribution |
|------|--------|-------------|
| ~2013 | Jain & Gupta (Indian CS literature) | Basic game-theoretic analysis (limited availability) |
| 2018 | Various Indian CS conference papers | Heuristic-based Teen Patti AI (not equilibrium-based) |
| -- | Wizard of Odds (wizardofodds.com/games/teen-patti/) | House-edge analysis of casino Teen Patti variant |
| -- | esrrhs/teenpatti_algorithm (GitHub) | Lookup-table hand evaluation with sort-based encoding |

### Relevant Theory

| Year | Authors | Title | Relevance |
|------|---------|-------|-----------|
| 2013 | Szafron et al. | A Parameterized Family of Equilibrium Profiles for Three-Player Kuhn Poker | Multiplayer equilibrium structure in small card games |
| 2018 | Ganzfried & Nowak | Successful Nash Equilibrium Agent for a 3-Player Imperfect-Information Game | Practical 3-player Nash computation |
| 2020 | Sklansky | The Theory of Poker | Optimal bluffing frequency theory (1:2 bluff-to-value ratio) |
| 2024 | RENES (IJCAI 2024) | Reinforcement Nash Equilibrium Solver for variable-size games |

---

## Appendix: Hand Probability Distribution

For reference, the complete Teen Patti hand probability table:

| Hand Type | Count | Probability | Odds Against |
|-----------|-------|-------------|-------------|
| Trail (Three of a Kind) | 52 | 0.235% | 424:1 |
| Pure Sequence (Straight Flush) | 48 | 0.217% | 459:1 |
| Sequence (Straight) | 720 | 3.258% | 29.7:1 |
| Color (Flush) | 1,096 | 4.959% | 19.2:1 |
| Pair | 3,744 | 16.941% | 4.9:1 |
| High Card | 16,440 | 74.389% | 0.34:1 |
| **Total** | **22,100** | **100%** | -- |

Note: Trail outranks Pure Sequence despite being marginally more probable (52 vs 48 combos). This is a deliberate ranking choice in Teen Patti rules, unlike standard poker where hand rankings correlate inversely with probability.
