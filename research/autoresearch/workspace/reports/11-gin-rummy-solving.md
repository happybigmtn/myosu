# Gin Rummy: Solving Strategies Research Report

## Executive Summary

Gin Rummy occupies a unique position in the imperfect-information game landscape: a two-player zero-sum draw-discard game with a game tree estimated at 10^20-10^30 nodes and information sets on the order of 10^15-10^20. Unlike poker, where the hidden information is a small number of hole cards, Gin Rummy conceals 10-card hands plus the entire stock pile ordering, creating an information asymmetry that makes direct CFR application computationally prohibitive without heavy abstraction. No Nash equilibrium has been computed for full Gin Rummy.

The bulk of dedicated Gin Rummy AI research emerged from the 2021 EAAI Undergraduate Research Challenge, which produced 13 peer-reviewed papers at AAAI-21 covering opponent hand estimation, knock decision optimization, heuristic agents, neural network approaches, and MCCFR-derived strategies. Since then, the field has advanced through broader imperfect-information game techniques (ReBeL, RL-CFR, GO-MCTS, NFSP) and through cross-domain transfer from Mahjong AI (Suphx) and rummy-family research (SPADENet, Indian Rummy MinDist). The most recent work (2025) explores adversarial co-evolution of PPO agents against LLM opponents, achieving 99%+ win rates through curriculum learning and knowledge distillation.

The practical state of the art for Gin Rummy AI combines Monte Carlo simulation for opponent hand inference, heuristic or learned discard policies, MCCFR-derived knock strategies, and neural network hand evaluation. No agent has demonstrated definitively superhuman play. The game remains unsolved in the game-theoretic sense, and the gap between current agents and optimal play is unknown, making it an active research target well-suited to Myosu's solver architecture evaluation.

## Game Complexity Analysis

### Hidden Information Structure

Gin Rummy's hidden information is substantially more complex than Texas Hold'em poker:

| Dimension | Gin Rummy | NLHE Heads-Up |
|-----------|-----------|---------------|
| Hidden cards per player | 10 | 2 |
| Possible initial hands | ~1.58 x 10^10 | 1,326 |
| Hand-vs-hand combinations | ~2.17 x 10^18 | ~8.5 x 10^5 |
| Unknown stock pile cards | 31 (ordered) | N/A (5 community) |
| Information revealed per turn | 1 draw choice + 1 discard | 1 bet/raise/call/fold |

The critical asymmetry: after the initial deal, 31 cards are in the stock pile in an unknown order, 10 are in the opponent's hand, and only 1 (the initial upcard) is public. As the game progresses, discards become visible, and the stock pile shrinks, but drawing from stock reveals nothing while drawing from the discard pile leaks information about the drawer's hand.

### Information Set Explosion

The number of non-abstract information sets in Gin Rummy is prohibitively large compared to poker. In NLHE, the 2-card hand creates 169 strategically distinct hole card combinations. In Gin Rummy, the 10-card hand creates C(52,10) = 15,820,024,220 possible holdings before accounting for game history (discard sequence, draw source choices, remaining stock composition). The full information set must encode:

- The player's current 10-card hand
- The complete sequence of discards by both players
- Which cards each player drew from the discard pile (public) vs. stock (private)
- The current stock pile size
- Whether the player is eligible to knock

This creates an information set space estimated at 10^15 to 10^20, compared to roughly 10^14 for NLHE (before abstraction).

### Game Tree Properties

Typical Gin Rummy hands last 10-20 turns. Each turn has a branching factor of approximately 22 (2 draw choices x 11 discard choices) plus a binary knock decision when eligible. The game tree is further complicated by the stochastic stock pile ordering and the lay-off phase after a knock. Total game tree size is estimated at 10^20-10^30 nodes.

A key structural difference from poker: Gin Rummy has no betting rounds. All information leakage occurs through draw-source and discard choices, making the signaling channel narrower but more persistent (every turn leaks information, whereas poker has only 4 betting rounds).

### Meld Assignment Complexity

Determining optimal deadwood for a 10-card hand requires solving a combinatorial optimization problem: partitioning cards into melds (sets of 3-4 same-rank cards, runs of 3+ consecutive same-suit cards) to minimize unmelded card values. This is a variant of the set packing problem, which is NP-hard in general. However, for the fixed hand size of 10-11 cards, exhaustive enumeration is tractable (the number of possible meld arrangements for a 10-card hand is bounded and small enough for brute-force search), making deadwood calculation fast in practice. Efficient implementations use backtracking search with pruning.

## Current Best Approaches

### 1. Monte Carlo Simulation with Card Counting

**The dominant practical approach.** Sample possible opponent hands consistent with observed game history, evaluate each candidate action via forward simulation, and select the action with highest expected value.

**Algorithm:**
1. Maintain a belief distribution over opponent hands based on: own hand, discards observed, cards opponent drew from discard pile, cards remaining in stock.
2. For each candidate action (draw source, discard choice, knock decision), sample N opponent hands from the belief distribution.
3. For each sample, simulate the game forward (often with heuristic rollout policies) to estimate action value.
4. Select the action maximizing expected value across samples.

**Strengths:** Handles imperfect information naturally through sampling; scales to full game without abstraction; integrates opponent modeling through belief updates.

**Weaknesses:** Quality depends on belief model accuracy; computationally expensive for real-time play with many samples; rollout policy quality limits lookahead accuracy; susceptible to non-locality (a locally optimal draw/discard sequence may be globally inferior).

**Compute:** Moderate. 1000-10000 samples per decision is typical, with each sample requiring a heuristic game-completion rollout. Real-time play is achievable on modern hardware.

**Representative work:** Vento (Stanford, 2020) applied online planning with Monte Carlo rollouts, finding that algorithms not assuming knowledge of opponent's hand outperformed those with such assumptions. Decisions achievable in milliseconds.

### 2. MCCFR (Monte Carlo Counterfactual Regret Minimization)

**Applied selectively to tractable subgames**, most notably the knock decision. Full CFR over the complete Gin Rummy game tree is computationally infeasible without severe abstraction, because the game tree grows exponentially with draw/discard decisions. However, the knock decision has a much smaller action space (binary: knock or continue), and ending the game by knocking limits tree growth to linear depth.

**Algorithm (for knock decision):**
1. Abstract the knock decision into an extensive-form game conditioned on current deadwood, estimated opponent deadwood, stock pile size, and game phase.
2. Apply MCCFR (outcome sampling or external sampling variant) to iterate through the abstracted game tree.
3. Converge to an approximate Nash equilibrium knock strategy.

**Key result:** Goldstein et al. (AAAI 2021) used MCCFR to derive Nash equilibrium knocking strategies, finding that optimal knock thresholds are dynamic -- they decrease as the game progresses (knock with lower deadwood later in the hand to reduce undercut risk). The analysis also revealed that conflicting expert knock advice arises partly from differences between Gin Rummy rule variants (standard vs. Oklahoma).

**Strengths:** Provably converges to Nash equilibrium in two-player zero-sum games; produces unexploitable knock strategies.

**Weaknesses:** Only tractable for isolated subgames (knock decision); cannot handle the full draw/discard game tree without extreme abstraction; requires careful information set abstraction.

**Compute:** Moderate for knock-only subgames. Full-game MCCFR is currently infeasible.

### 3. Neural Network Approaches

Multiple neural architectures have been applied to Gin Rummy decision-making:

**a) Deterministic Neural Networks (Nguyen et al., AAAI 2021)**
- Convolutional neural network operating on hand representations
- Opponent hand estimated using Bayesian reasoning and Monte Carlo simulation
- Separate models for draw, discard, and knock decisions
- Trained on self-play data

**b) Random Forest for Opponent Hand Estimation (AAAI 2021)**
- Trained a random forest on self-play game states to estimate per-card probabilities of being in the opponent's hand
- Agent using these estimates won 61% of games against baseline heuristic agent without opponent modeling

**c) SPADENet (IEEE CAI 2024)**
- Multi-input deep neural network architecture for skill-based card games
- Captures player hand combined with visible game state
- Applied to online rummy variants (Points, Pool, Deals)
- Achieved 18% F1 improvement over prior best on Pool-6P variant (0.767 to 0.908)
- Generalizable across rummy game variants

**d) Data-Driven Hand Evaluation (Truong & Neller, AAAI 2021)**
- Employed CNNs with Monte Carlo simulation and Bayesian reasoning
- Computes both offensive scores (proximity to gin/knock) and defensive scores (vulnerability to opponent knock)
- Used for holistic hand-state evaluation

**Strengths:** Can learn complex patterns from self-play; fast inference at decision time; can capture subtle card relationships.

**Weaknesses:** No convergence guarantees to Nash equilibrium; quality bounded by training data/environment; black-box decision-making complicates analysis.

### 4. Deep Reinforcement Learning

**a) PPO with Self-Play**
Proximal Policy Optimization trained through self-play, reaching a level outperforming amateur human players. The agent learns draw, discard, and knock policies end-to-end from reward signals.

**b) TD-Rummy vs. EVO-Rummy (Kotnik & Kalita, ICML 2003)**
Foundational comparison of temporal-difference learning vs. coevolution for training gin rummy agents. Both used neural network value functions trained through self-play. Coevolution produced superior results, suggesting that the fitness landscape for gin rummy value functions benefits from population-based search.

**c) Adversarial Co-Evolution with LLMs (2025)**
The most recent approach uses LLMs (Llama 3, Gemma, GPT) as zero-shot strategic opponents via chain-of-thought prompting, with a 3-phase curriculum:
1. Phase 1: PPO agent trains against random opponents
2. Phase 2: Self-play refinement
3. Phase 3: Adversarial training against LLM opponents

Achieves 99.12% win rate against LLM opponents using a multi-process PPO pipeline with custom action masking (Stable Baselines 3 + PyTorch). Key innovation: distilling the broad strategic "common sense" of LLMs into a compact, fast RL policy.

**d) Dynamic Strategies (IntelliSys 2021)**
RL-based agent with strategy switching based on game history. Discard and draw algorithms adapted dynamically based on previous game results. Win rate improved from 50% (random) to 57.85% (static RL) to 67.735% (dynamic strategies) over 100,000 games.

**Strengths:** End-to-end learning; no hand-crafted features required for policy; can discover non-obvious strategies.

**Weaknesses:** High variance in gin rummy outcomes requires very large training budgets; no exploitability guarantees; sensitive to reward shaping and hyperparameters.

### 5. ISMCTS (Information Set Monte Carlo Tree Search)

Not yet directly published for Gin Rummy, but highly applicable. ISMCTS (Cowling, Powley, Whitehouse, 2012) addresses the fundamental limitations of Perfect Information Monte Carlo (PIMC) search:

**Strategy fusion problem:** PIMC evaluates each determinization independently, potentially playing different strategies in different sampled worlds. ISMCTS grows a single tree across all sampled worlds, averaging information.

**Non-locality problem:** PIMC selects locally optimal moves that may be globally inferior. ISMCTS mitigates this through unified tree statistics.

ISMCTS has been successfully applied to Spades, Dou Di Zhu, Hanabi, and other imperfect-information card games. Its application to Gin Rummy would involve sampling consistent opponent hands and stock orderings at each decision node, running MCTS simulations, and building a single information-set tree.

### 6. GO-MCTS (Generative Observation MCTS)

**Emerging approach (Rebstock et al., 2024).** Uses a transformer model to generate observation sequences, performing MCTS entirely in observation space rather than state space. Applied to trick-taking card games (Hearts, Skat, The Crew).

Relevance to Gin Rummy: the observation-space formulation naturally handles the partial observability of opponent hands and stock pile, and the transformer architecture can learn the temporal dependencies in draw/discard sequences. This approach avoids strategy fusion and non-locality by construction.

### 7. Heuristic Agents

Still competitive with learned approaches. The EAAI 2021 challenge produced several strong heuristic agents:

**Heisenbot (AAAI 2021):** Rule-based agent using domain knowledge from expert strategy guides. Competitive with neural approaches in tournament play.

**Highly-Parameterized Ensemble (Nagai et al., DePauw, AAAI 2021):** Genetic and grid search optimization over hyperparameters controlling draw, discard, and knock decisions. Ensemble approach for discard selection.

**Myopic Meld Distance (Goldman et al., AAAI 2021):** Hand evaluation based on "edit distance" to valid meld configurations, combined with opponent modeling and conservative knock assessment.

## Draw/Discard Decision Theory

### Information Leakage Model

Every draw-discard turn in Gin Rummy is a signaling game:

**Drawing from the discard pile reveals:**
- The specific card taken (public information)
- The player wants that card (or wants to deny it to the opponent)
- Likely meld directions (if opponent draws 7H, they may be building a hearts run or a 7s set)

**Drawing from stock reveals:**
- Nothing about the drawn card (private information)
- The player did NOT want the top discard (mild negative inference)

**Discarding reveals:**
- The specific card released (public information)
- The player does not need that card for melds (or is making a deceptive play)
- Cards adjacent to or of the same rank as discards may be safe to discard

### Optimal Draw Policy

The draw decision balances information gain against information leakage:

1. **Take from discard pile when:** The card immediately completes or substantially advances a meld, AND the information leaked is acceptable given the game state.
2. **Take from stock when:** The discard pile card is not useful, OR taking it would reveal critical meld information to the opponent, OR you want to maintain maximum uncertainty for the opponent.
3. **Deceptive draws:** Occasionally drawing from the discard pile for non-meld purposes (blocking, misdirection) can exploit opponents who model draw behavior. However, this is expensive (the drawn card may not reduce deadwood).

### Discard Selection

The discard decision is the most complex per-turn choice (11 candidates from an 11-card hand):

**Offensive considerations:**
- Discard the card that contributes least to meld potential
- Preserve cards with multiple meld paths (e.g., 7H can join a 7-set or a hearts run)
- Discard high-value deadwood early to reduce knock exposure

**Defensive considerations:**
- Avoid discarding cards the opponent likely needs (inferred from their draw history)
- "Salting" -- discarding cards similar to those the opponent has shown interest in, to misdirect
- Hold safe discards (cards known not to help opponent) for late-game when opponent may be close to knocking

**Card Fitness (Gallucci et al., AAAI 2021):** A per-card metric estimating the card's contribution to the hand's overall meld potential, accounting for both current melds and probabilistic future meld completion. Used to rank discard candidates.

### Opponent Hand Inference

Multiple approaches have been validated:

| Method | Accuracy | Win Rate vs. Baseline |
|--------|----------|----------------------|
| Bayesian updating | Moderate | Not isolated |
| Random Forest | Good (per-card probability) | 61% |
| CNN on game state | Good | Competitive |
| Heuristic card counting | Moderate | Competitive |

The Bayesian approach maintains a prior distribution over opponent hands and updates based on observed draws and discards. The random forest approach trains on self-play data to predict per-card presence probabilities. Both significantly improve discard selection and knock timing.

## Knock Decision Optimization

### The Knock Decision as Optimal Stopping

The knock decision is structurally an optimal stopping problem: at each turn where the player's deadwood is at most 10, they must decide whether to "stop" (knock) or "continue" (keep playing for lower deadwood or gin).

**Factors influencing the optimal knock threshold:**

1. **Current deadwood:** Lower is always better for knocking, but the marginal benefit decreases.
2. **Estimated opponent deadwood:** If opponent is likely to have high deadwood, knock even with marginal deadwood (8-10). If opponent may have low deadwood, avoid knocking to prevent undercut.
3. **Turn number / Stock pile size:** As the game progresses, knock sooner. Late-game knocking with moderate deadwood is safer because the opponent has had fewer turns to improve. Conversely, knocking with 10 deadwood on turn 3 risks severe undercut.
4. **Score context:** If the game score is close, conservative knocking (low deadwood only) reduces variance. If behind, aggressive knocking (higher deadwood) or gin pursuit may be warranted.
5. **Undercut penalty:** The 25-point undercut bonus is severe. It makes marginal knocks (8-10 deadwood) risky unless the opponent's hand is clearly worse.

### MCCFR-Derived Knock Strategies

Goldstein et al. (2021) computed approximate Nash equilibrium knock policies using MCCFR. Key findings:

- **Dynamic threshold:** The optimal knock deadwood threshold decreases as the hand progresses. Early in the hand, knocking at 10 is acceptable; by mid-game, the threshold drops to 5-7; late-game, only gin or very low deadwood knocks are optimal.
- **Variant sensitivity:** Different rule variants (standard vs. Oklahoma) produce substantially different optimal knock thresholds, explaining apparent contradictions in expert knock advice.
- **Knocking policy dominates performance:** Among all decision components (draw, discard, knock), the knock policy has the largest impact on agent win rate.

### Gin Pursuit vs. Early Knock

Two strategic poles with different risk/reward profiles:

**Conservative (knock early):** Minimize deadwood, knock as soon as eligible with reasonable deadwood. Catches opponents with high deadwood. Lower variance. Better when ahead in game score.

**Aggressive (pursue gin):** Hold out for zero deadwood to earn the 25-point gin bonus and deny lay-offs. Higher variance. Risk that opponent knocks first. Better when behind in game score or when stock pile is deep (more turns to improve).

The optimal balance depends on the opponent's tendencies: against aggressive opponents who knock early with high deadwood, playing conservatively and aiming for undercuts is profitable. Against defensive opponents who hold for gin, knocking early with moderate deadwood catches them with high deadwood.

## Open Problems

### 1. Full-Game Nash Equilibrium
No Nash equilibrium has been computed for full Gin Rummy. The information set space (~10^15-10^20) makes direct CFR application infeasible without abstraction, and no adequate abstraction scheme has been validated. Unlike poker, where card abstraction (bucketing) is well-understood, Gin Rummy's meld-based hand structure makes abstraction non-trivial -- two hands with similar deadwood can have vastly different meld potential.

### 2. Exploitability Measurement
There is no established exploitability metric for Gin Rummy agents. In poker, exploitability is measured as the expected loss per hand against a best-response opponent. Gin Rummy's larger information set space makes best-response computation intractable, leaving no way to measure how far current agents are from optimal play.

### 3. Abstraction for CFR
What is the right abstraction for Gin Rummy? Poker abstractions bucket hands by equity; Gin Rummy requires abstractions that capture meld potential, deadwood structure, and defensive vulnerability simultaneously. No principled abstraction framework exists.

### 4. Long-Horizon Credit Assignment
Gin Rummy hands span 10-20 turns with high outcome variance. A single draw/discard decision's impact on the final outcome is diluted across many subsequent decisions and stochastic events (stock pile draws). This makes RL training slow to converge and credit assignment noisy.

### 5. Superhuman Play Verification
No agent has demonstrated definitively superhuman performance. Unlike poker (where Libratus/Pluribus beat top professionals) or Mahjong (where Suphx exceeded 99.99% of ranked players), Gin Rummy lacks a benchmark for superhuman play. The EAAI competition provides agent-vs-agent rankings but no human professional baseline.

### 6. Multi-Hand Strategy
Most research focuses on single-hand play. The full game (first to 100 points) introduces meta-strategic considerations: when to play aggressively vs. conservatively based on game score, how to adjust knock thresholds based on point differential, and whether to pursue shutout bonuses. No research has addressed optimal multi-hand strategies.

### 7. Deceptive Play
Drawing from the discard pile for deception (misdirecting the opponent about meld intentions) is discussed in expert strategy guides but has not been formalized or evaluated in AI research. The information-theoretic cost/benefit of deceptive draws is unknown.

### 8. Real-Time Solving vs. Blueprint
Poker AI uses a blueprint strategy computed offline and refined with real-time solving at each decision point. Gin Rummy's larger information sets make both the blueprint and real-time components more expensive. Whether a DeepStack/ReBeL-style approach is tractable for Gin Rummy remains an open question -- Vento (2020) noted that a DeepStack-like implementation would be too time-intensive.

## Relevance to Myosu

### Solver Architecture Evaluation

Gin Rummy is a strong mid-tier benchmark for Myosu's solver evaluation framework:

1. **CFR feasibility boundary:** Gin Rummy sits at the boundary of CFR tractability. Full-game CFR is infeasible, but subgame CFR (knock decisions) works. This tests the solver architecture's ability to decompose games into tractable and intractable components and apply different methods to each.

2. **Monte Carlo simulation testing:** The dominant practical approach (MC sampling of opponent hands) is well-suited to evaluation. Solver quality can be measured by MC sample efficiency, belief model accuracy, and rollout policy quality.

3. **Information leakage modeling:** The draw-source decision creates a unique information channel not present in poker (where all actions are bets/folds). This tests whether the solver architecture can model and exploit asymmetric information flows.

4. **Neural network value function:** The rich hand state (10 cards, meld structure, discard history) provides a good testbed for neural hand evaluation networks, which must capture both offensive (meld potential) and defensive (deadwood exposure) value.

### Architecture Ranking Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| CFR applicability | 3/5 | Feasible for subgames; requires heavy abstraction for full game |
| Neural value network | 4/5 | Rich state space; CNN/transformer architectures show promise |
| Abstraction necessity | 4/5 | Significant abstraction needed for tractability |
| Real-time solving value | 3/5 | MC simulation effective; DeepStack-style solving open question |
| Technique transferability | 4/5 | Draw-discard mechanics shared across rummy family; opponent modeling transfers broadly |

### Recommended Solver Stack for Myosu

1. **Primary method:** Monte Carlo simulation with belief tracking for draw/discard decisions
2. **Knock policy:** MCCFR-derived blueprint with dynamic deadwood threshold
3. **Opponent modeling:** Random forest or neural network for opponent hand estimation, updated from observed draw/discard history
4. **Hand evaluation:** Neural network trained via self-play for offensive/defensive scoring
5. **Fallback:** Heuristic agent using card fitness metrics and expert-derived rules

### Evaluation Metrics

- **Win rate** against calibrated opponents (heuristic baseline, self-play RL, random)
- **Knock decision quality** (measured against MCCFR-optimal knock policy)
- **Opponent hand estimation accuracy** (per-card probability calibration)
- **Deadwood efficiency** (average deadwood at knock vs. opponent deadwood at knock)
- **Information exploitation** (how effectively the agent uses discard-pile draws to infer opponent hand)

## Key Papers & References

### Foundational

| Year | Authors | Title | Venue | Contribution |
|------|---------|-------|-------|-------------|
| 2003 | Kotnik, Kalita | TD-Rummy vs. EVO-Rummy: Temporal Difference Learning in Self-Play Training | ICML | First rigorous comparison of RL methods for gin rummy; coevolution outperformed TD learning |
| 2012 | Cowling, Powley, Whitehouse | Information Set Monte Carlo Tree Search | IEEE TCIAIG | ISMCTS algorithm solving strategy fusion and non-locality in imperfect-information games |

### EAAI 2021 Gin Rummy Challenge (AAAI-21)

| Authors | Title | Key Contribution |
|---------|-------|-----------------|
| Goldstein, Astudillo Guerra, Haigh, Cruz Ulloa, Blum | Extracting Learned Discard and Knocking Strategies from a Gin Rummy Bot | MCCFR-derived Nash equilibrium knock strategies; dynamic knock thresholds |
| (Princeton team) | Random Forests for Opponent Hand Estimation in Gin Rummy | Random forest opponent hand prediction; 61% win rate vs. baseline |
| (Multiple teams) | Opponent Hand Estimation in Gin Rummy Using Deep Neural Networks and Heuristic Strategies | Bayesian + CNN opponent hand estimation |
| Nguyen, Doan, Neller | A Deterministic Neural Network Approach to Playing Gin Rummy | CNN-based draw/discard/knock policies |
| Truong, Neller | A Data-Driven Approach for Gin Rummy Hand Evaluation | CNN + MC + Bayesian offensive/defensive hand scoring |
| Nagai, Shrivastava, Ta, Bogaerts, Byers | A Highly-Parameterized Ensemble to Play Gin Rummy | Genetic algorithm hyperparameter optimization for ensemble agent |
| Goldman, Knutson, Mahtab, Maloney, Mueller, Freedman | Evaluating Gin Rummy Hands Using Opponent Modeling and Myopic Meld Distance | Meld distance metric + opponent modeling + conservative knock |
| (Heisenbot team) | Heisenbot: A Rule-Based Game Agent for Gin Rummy | Expert-knowledge heuristic agent competitive with learned agents |
| Gallucci et al. | Estimating Card Fitness for Discard in Gin Rummy | Per-card fitness metric for discard ranking |
| (Knocking paper) | Knocking in the Game of Gin Rummy | Analysis of knock timing as key strategic decision |

### Recent and Cross-Domain

| Year | Authors | Title | Venue | Relevance |
|------|---------|-------|-------|-----------|
| 2020 | Brown, Sandholm et al. | ReBeL: Combining Deep RL and Search for Imperfect-Information Games | NeurIPS | General framework for self-play RL + search; applicable to gin rummy subgames |
| 2020 | Li et al. | Suphx: Mastering Mahjong with Deep RL | arXiv | Draw-discard game AI; global reward prediction, oracle guiding, run-time policy adaptation |
| 2020 | Vento | Online Planning Strategies for Gin Rummy | Stanford AA228 | MC rollouts for gin rummy; millisecond decision-making |
| 2020 | Zha et al. | RLCard: A Toolkit for RL in Card Games | IJCAI | Open-source toolkit with gin rummy environment, DQN/NFSP/CFR implementations |
| 2021 | Nguyen et al. | Dynamic Strategies and Opponent Hands Estimation for RL in Gin Rummy | IntelliSys | Dynamic strategy switching; 67.7% win rate |
| 2024 | Jain, Garg et al. | SPADENet: Skill-based Player Action Decision for Card Games | IEEE CAI | Multi-input DNN for rummy variants; 18% F1 improvement |
| 2024 | Rebstock, Solinas, Sturtevant, Buro | GO-MCTS: Transformer-Based Planning in Observation Space | arXiv | Observation-space MCTS avoiding strategy fusion; applicable to draw-discard games |
| 2024 | Li et al. | RL-CFR: Improving Action Abstraction for Imperfect-Information Games | ICML | RL-guided action abstraction for CFR; outperforms ReBeL on NLHE |
| 2025 | (Adversarial co-evolution) | Adversarial Co-Evolution of RL and LLM Agents in Gin Rummy | GitHub | PPO + LLM curriculum learning; 99% win rate against LLM opponents |
| 2025 | (Indian Rummy) | Quantitative Rule-Based Strategy Modeling in Classic Indian Rummy | arXiv | MinDist metric for hand evaluation; algorithmic rummy strategy design |

### Toolkits and Implementations

| Resource | URL | Notes |
|----------|-----|-------|
| RLCard | https://rlcard.org/ | Open-source RL toolkit with Gin Rummy env; DQN, NFSP, CFR |
| OpenSpiel | https://github.com/google-deepmind/open_spiel | DeepMind's game research framework; gin rummy supported |
| EAAI Gin Rummy Challenge | http://cs.gettysburg.edu/~tneller/games/ginrummy/eaai/ | Challenge framework, agent interface, evaluation harness |
| GinRummyAlgorithm | https://github.com/rxng8/GinRummyAlgorithm | RL + NN approaches for gin rummy |
| Adversarial Co-Evolution | https://github.com/Nikelroid/adversarial-coevolution | PPO + LLM framework for gin rummy |
