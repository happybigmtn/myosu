# Solving Strategies for Spades (Partnership Trick-Taking)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of solving methods, 2001--2026
**Status:** Research report (no implementation)

---

## Executive Summary

Spades is a four-player partnership trick-taking card game with a fixed trump suit (spades), imperfect information, and a unique scoring system that penalizes both underbidding (bags accumulation) and overbidding (set penalties). Despite its widespread popularity in the United States -- particularly in military, college, and online settings -- Spades has received remarkably little dedicated academic attention compared to its trick-taking relatives Bridge, Skat, and Hearts. No Nash equilibrium computation exists for Spades, no superhuman AI has been demonstrated, and the strongest published work specifically on Spades is the Bidding-in-Spades (BIS) algorithm (Cohensius, Meir, Oved, Stern; ECAI 2020), which addresses only the bidding phase and achieves performance superior to recreational human players but not expert-level play.

The game's strategic depth arises from the interaction of five distinct challenges: accurate trick estimation during bidding, partnership coordination without explicit communication, nil bid risk management, multi-hand bag penalty avoidance, and card play optimization under imperfect information. The partnership structure places Spades in the class of adversarial team games, where the appropriate solution concept is Team-Maxmin Equilibrium with Coordination (TMECor) rather than standard Nash equilibrium. Recent advances in team-game solving (Team-PSRO at NeurIPS 2023, PRR-TM in 2025, and warm-started CFR for TMECor in 2024) provide theoretical foundations but have not been applied to Spades at scale.

The most promising practical approaches draw from the broader trick-taking AI literature: Perfect Information Monte Carlo (PIMC) sampling with domain-specific inference for card play, neural network-based bidding models trained via supervised learning and reinforcement learning, and specialized modules for nil strategy and bag management. A 2024 general framework for trick-taking games (Edelkamp, KI 2024) and the Generative Observation Monte Carlo Tree Search (GO-MCTS) method using transformers (2024) represent the current frontier of applicable techniques. Spades' simpler bidding structure relative to Bridge and its shared trick-taking mechanics with Hearts and Skat make it an ideal candidate for transfer learning from more heavily studied games.

---

## Game Complexity Analysis

### Structural Properties

| Property | Value |
|----------|-------|
| Players | 4 (two fixed partnerships) |
| Cards | Standard 52-card deck, 13 per player |
| Trump | Spades (fixed, always trump) |
| Information | Imperfect: each player sees only own 13 cards |
| Bidding | Single round, each player bids 0--13 |
| Special bids | Nil (0 tricks), Blind Nil (before seeing hand) |
| Scoring | 10 pts/trick bid if made; -10 pts/trick bid if set; +1 pt per overtrick (bag); -100 pts at 10 accumulated bags |
| Game end | First team to 500 points |

### State Space Metrics

| Metric | Estimate | Notes |
|--------|----------|-------|
| Possible deals | ~5.36 x 10^28 | C(52,13) x C(39,13) x C(26,13); identical to Bridge |
| Bidding sequences | 14^4 = 38,416 | Each player bids 0--13; single round |
| Average play branching factor | ~3.6 | Constrained by must-follow-suit and spades-breaking rules |
| Game tree nodes (single hand) | ~10^15--10^20 | Lower than Bridge due to no auction complexity |
| Information sets (single hand) | ~10^12--10^17 | Reduced as tricks reveal cards |
| Multi-hand game states | Effectively unbounded | Score + bags create cross-hand state |

### Partnership Dynamics

Spades is an **adversarial team game**: two partnerships compete in a zero-sum setting where each team member holds private information and cannot communicate except through public game actions (bids and card plays). This maps to the TMECor solution concept, where teammates may coordinate strategies before play (analogous to pre-game agreements about bidding tendencies and signaling) but cannot exchange private information during play.

Unlike Bridge, Spades has no formal convention system. There is no bidding language with defined meanings -- a bid of "4" simply means "I expect to win 4 tricks." This drastically simplifies the cooperative communication problem relative to Bridge but does not eliminate it: partners still signal through card selection (leading a suit signals strength), through bid magnitude (a high bid from partner changes the play strategy), and through timing of spade-breaking.

The fixed trump suit eliminates Bridge's denomination selection problem entirely. There is no auction to determine trumps, no dummy exposure, and no declarer/defender asymmetry. All four players play independently with symmetric information access (each sees only their own 13 cards). This symmetry simplifies analysis but preserves the core partnership coordination challenge.

---

## Bidding Strategy

### The Bidding-in-Spades (BIS) Algorithm

The only peer-reviewed algorithm specifically targeting Spades bidding is BIS (Cohensius, Meir, Oved, Stern; ECAI 2020). Its architecture:

1. **Expected utility computation**: For each possible bid b in {0, 1, ..., 13}, estimate the expected utility U(b) considering the scoring function (10 pts/trick bid if made, -10 pts/trick bid if set, +1 pt per bag, -100 pts at 10 cumulative bags).

2. **Non-nil bid estimation**: Use Monte Carlo simulation to estimate trick-winning probability. Sample N random hands for the three unseen players (consistent with known information), simulate play using a fixed playing algorithm, and count tricks won across samples. The expected tricks distribution is then convolved with the scoring function to produce U(b) for each non-nil bid.

3. **Nil bid evaluation**: Train a supervised learning classifier on millions of real Spades games to predict the probability of a nil bid succeeding given the hand composition. The classifier uses hand features (high-card points, spade count, void suits, singleton honors) to output P(nil success). The nil expected utility is then: U(nil) = P(success) * 100 + (1 - P(success)) * (-100).

4. **Heuristic correction via machine learning**: Domain heuristics (e.g., counting sure tricks from high cards, adjusting for trump length) produce initial estimates that are then corrected using a regression model trained on real game data. This accounts for systematic biases in pure simulation (e.g., simulation assumes perfect play, real opponents make mistakes).

5. **Bid selection**: Choose argmax_b U(b), incorporating the team's current score and bag count into the utility function.

**Performance**: BIS beats rule-based bidding bots when all agents use the same playing component. When combined with a rule-based playing algorithm, it outperforms the average recreational human player. It does not approach expert-level bidding.

### Trick Estimation Fundamentals

Accurate trick estimation is the most critical skill in Spades. The key factors:

- **Sure tricks**: Aces and protected Kings in non-trump suits; high spades (Ace, King, Queen of spades are near-certain winners if spade count is adequate).
- **Probable tricks**: Queens with one guard, Jacks with two guards. These depend on positional play.
- **Length tricks**: In a long suit (5+ cards), after opponents exhaust their holdings, remaining cards become winners. Probability depends on suit distribution across the table.
- **Trump tricks**: Low spades can win tricks by ruffing (playing spades when void in the led suit). The probability depends on void creation -- having short side suits increases trump trick potential.
- **Positional adjustments**: Being last to play in a trick is advantageous (see all other cards before deciding). Leading position affects trick-winning probability by ~0.5--1 trick over a hand.

### Nil Decision Framework

The nil bid (+100/-100 points) is the single highest-variance decision in Spades. Key factors for nil viability:

| Factor | Favorable for Nil | Unfavorable for Nil |
|--------|-------------------|---------------------|
| Spade holdings | 2-spade or fewer, all low (2-7) | Any spade honor (A, K, Q, J) |
| Side suit honors | None, or protected by 3+ low cards | Bare Ace, bare King, or AK tight |
| Void suits | 1--2 voids (can sluff losers) | No voids (forced to follow and take tricks) |
| Partner's bid | High (partner can cover tricks) | Low (partner cannot protect nil) |
| Score situation | Team trailing by 100+ points (risk justified) | Team leading (risk unnecessary) |
| Position | Later in bidding order (more information) | First to bid (least information) |

Blind nil (+200/-200) is a desperation play, typically reserved for teams trailing by 100+ points. The expected value is negative for any hand distribution but becomes strategically correct when the alternative (slow recovery through normal bidding) has an even lower win probability given the score differential.

### Bid Calibration and Team Coordination

The team's combined bid determines the contract. Partners must implicitly coordinate:

- **Conservative pairing**: If both partners bid conservatively (say, each underbids by 0.5 tricks), the team accumulates ~1 bag per hand, reaching the -100 penalty every ~10 hands. This is sustainable but suboptimal.
- **Aggressive pairing**: If both bid aggressively, the team risks being set (losing 10 pts per trick bid) more frequently. A set rate above ~25% typically yields negative expected value.
- **Asymmetric roles**: One partner bids their true estimate while the other adjusts based on the first partner's bid. The second bidder (later in order) has more information and can calibrate -- bidding higher if the team's combined estimate seems low, or lower to avoid bags.

The BIS algorithm treats each player's bid independently (conditioned on prior bids in the round). A more sophisticated approach would model the team's joint bidding as a coordination problem, where the second bidder's optimal bid depends not just on their hand but on the information contained in the first bidder's choice.

---

## Play Strategy

### Card Play Under Imperfect Information

The play phase of Spades is a standard trick-taking game amenable to the same algorithmic approaches used for Hearts, Skat, and Bridge:

**Perfect Information Monte Carlo (PIMC)**: Sample N random deals consistent with all known information (own hand, cards played, bids, void suits observed), solve each deal using a perfect-information solver, aggregate results to select the best card. This is the workhorse approach for trick-taking AI. For Spades, the perfect-information solver can use alpha-beta search or exhaustive minimax on the remaining cards.

**Strengths of PIMC in Spades**: The average branching factor of ~3.6 (constrained by suit-following) makes perfect-information solving tractable even with 8--10 tricks remaining. The 52-card deck and 13-trick structure keep the search trees manageable. PIMC with 50--200 samples produces strong play at sub-second latency.

**Strategy fusion problem**: PIMC suffers from strategy fusion -- it treats each sampled world independently, assuming it can use different strategies in different worlds. In Spades, this manifests when a card is the best play in some possible worlds but catastrophic in others. The Extended PIMC (EPIMC) algorithm (Arjonilla et al., 2024) mitigates this by postponing perfect-information resolution deeper into the search, maintaining partial information sets. EPIMC has not been tested specifically on Spades but is directly applicable.

### Inference: Reconstructing Hidden Hands

The quality of PIMC depends critically on the quality of hand samples. Naive uniform sampling over all consistent hands produces unrealistic distributions. Better inference uses:

1. **Suit-following constraints**: When a player fails to follow suit, they are void in that suit. This eliminates large regions of the hand space.

2. **Bidding information**: A player who bid 5 is likely to hold more high cards than a player who bid 2. Bayesian updating on the bid narrows the hand distribution.

3. **Policy-based inference (Rebstock et al., 2019)**: Assume a model of how each player selects cards given their hand, then compute the likelihood of the observed play sequence under each possible hand assignment. This dramatically improves sampling quality. Policies can be trained on human game data via supervised learning (deep neural networks).

4. **Card counting**: Track all 52 cards across tricks. As the game progresses, the number of possible hand assignments shrinks exponentially. By trick 10, most card locations are determined.

### Signaling and Partner Communication

In competitive Spades, partners communicate solely through their card play:

- **Leading a suit**: Signals strength in that suit. Leading the Ace of hearts says "I control hearts; give me hearts tricks."
- **Playing high on partner's lead**: Confirms strength in the led suit and encourages continuation.
- **Playing low on partner's lead**: Suggests weakness or that the partner should shift to a different suit.
- **Discarding**: When unable to follow suit, the choice of discard signals preferences. Discarding a high card in a suit signals that suit is unimportant; discarding a low card preserves high cards for later.
- **Spade-breaking timing**: Breaking spades early signals spade strength and a desire to pull opponents' trumps.

For AI, modeling these signals requires a **theory of mind**: the agent must reason about what its partner is trying to communicate, not just what card maximizes immediate trick value. This is related to the Hanabi challenge (Bard et al., 2020), where cooperative agents must infer intentions from actions in a game with hidden information and limited communication.

### Defensive and Offensive Play Patterns

**Attacking a nil bidder**: When an opponent bids nil, the team should:
- Lead suits where the nil bidder is likely to hold high cards.
- Lead suits where the nil bidder is void, forcing them to ruff with spades (if they hold spades) or discard (revealing information).
- Play low cards in tricks led by the nil bidder's partner, forcing the nil bidder to win.

**Defending a partner's nil**: When your partner bids nil:
- Lead suits where partner is void (they can discard safely).
- Win tricks early to prevent opponents from leading dangerous suits.
- Play high cards to cover partner's potential losers.

**Bag avoidance**: When the team is near 10 cumulative bags:
- Deliberately lose tricks that would create overtricks.
- Shift to low-card leads to cede trick-winning opportunities.
- Consider underbidding by 1 to create a cushion against accidental overtricks.

---

## Current Best Approaches

### 1. PIMC + Rule-Based Play (Industry Standard)

**Architecture**: Monte Carlo sampling of hidden hands + heuristic or minimax play solver + rule-based bidding.

**Implementation**: NeuralPlay (commercial Spades app) uses Monte Carlo simulation AI at higher difficulty levels. The AI considers total bid, tricks remaining, team bids, and individual player bids to determine whether to try to win or lose tricks. Multiple difficulty levels provide graduated challenge.

**Strengths**: Fast, robust, easy to implement. Produces play quality competitive with average recreational players. The rule-based bidding component handles standard situations well.

**Weaknesses**: Cannot learn or adapt. Rule-based bidding misses subtle hand evaluation factors. No partner modeling beyond bid values. Nil strategy is hand-coded and rigid. No bag management optimization across hands.

**Compute**: Real-time on mobile devices. ~50--200 MC samples per decision, sub-second per move.

### 2. BIS (Bidding) + PIMC (Play) -- Academic SOTA

**Architecture**: The Bidding-in-Spades algorithm for bid selection; PIMC with heuristic play for card selection.

**Strengths**: Best published bidding performance for Spades. The ML-corrected heuristics handle nil evaluation well. Modular -- the bidding component can be attached to any play algorithm.

**Weaknesses**: Bidding and play are not jointly optimized. The BIS nil evaluator is trained on recreational-level games, not expert play. No partner modeling in the play phase. Limited to the play algorithm it is paired with.

**Compute**: MC simulation for bidding (~1000 samples) + PIMC for play. Several seconds per bid decision; sub-second per play decision.

### 3. Neural Network Play (Van Hus, 2020)

**Architecture**: Feedforward neural networks trained on game data to predict optimal card plays. Two architectures explored: "card-based" (input: 52 x 3 features per card -- location probabilities; output: card to play) and an alternative representation.

**Strengths**: Learns implicit patterns from data. Does not require explicit rule engineering. Can potentially capture signaling and partner coordination implicitly.

**Weaknesses**: Tested only at bachelor thesis level (Utrecht University). Performance did not consistently exceed PIMC baselines. Small training dataset. No self-play refinement. No bidding component.

**Compute**: Inference is fast (~1ms per decision on CPU). Training requires ~10K+ games.

### 4. GO-MCTS with Transformers (2024, Applicable)

**Architecture**: Generative Observation Monte Carlo Tree Search. A transformer model learns to predict observation sequences from self-play. MCTS is performed in observation space (what the agent sees) rather than state space (the true game state). The transformer generates plausible future observation sequences, and MCTS evaluates actions by sampling these futures.

**Application to Spades**: Demonstrated on Hearts, Skat, and The Crew (all trick-taking games). Directly applicable to Spades with minimal adaptation. The observation-space approach naturally avoids strategy fusion because the search never resolves hidden information into a specific state.

**Strengths**: Avoids strategy fusion and non-locality issues inherent in PIMC. The transformer captures sequential dependencies in card play. Population-based self-play training discovers strong policies without human data. Principled handling of imperfect information.

**Weaknesses**: Higher compute than PIMC (transformer inference per MCTS node). Not yet tested on Spades specifically. Partnership coordination may not emerge naturally from individual self-play.

**Compute**: Training: days of GPU time for self-play. Inference: ~1--10 seconds per decision depending on MCTS budget.

### 5. General Trick-Taking Framework (Edelkamp, KI 2024)

**Architecture**: A unified programming interface for multiple trick-taking games (Belote, Tarot, Doppelkopf, Spades, Hearts, Euchre, Schafkopf). Implements general and game-specific card recommenders for bidding, team building, game selection, and card play. Expert rules enhance play quality.

**Application to Spades**: Spades is one of the explicitly supported games. The framework provides a baseline AI and evaluation infrastructure. Expert rules for Spades would include nil strategy, bag management, and spade-breaking heuristics.

**Strengths**: Provides a ready-made environment for Spades AI research. Transfer of improvements across trick-taking games. Evaluation against other games' AIs.

**Weaknesses**: Rule-based at core; no learning component. Expert rules are hand-engineered. Limited to the strategic depth encoded by the designer.

**Compute**: Real-time play at human-adequate speed.

### 6. Team-Game Equilibrium Approaches (Theoretical)

**Algorithms**: Team-PSRO (McAleer et al., NeurIPS 2023), PRR-TM (2025), warm-started CFR for TMECor (2024).

**Application to Spades**: Spades' 2v2 partnership structure is a canonical adversarial team game. These algorithms compute approximate TMECor -- the strategy profile where the team maximizes its worst-case payoff against the best opposing strategy, allowing pre-game coordination but not in-game communication.

**Strengths**: Game-theoretically principled. Naturally handles partnership coordination. Converges to unexploitable strategies.

**Weaknesses**: Computational cost is prohibitive for full Spades. TMECor computation requires transforming the team game into a two-player game, which exponentially increases size. Team-PSRO has been tested only on small games (tabular Kuhn poker, Liar's Dice). No published application to any full trick-taking game. Abstractions would be necessary, and trick-taking game abstraction is understudied.

**Compute**: Impractical for full Spades without significant abstraction. Potential for abstracted versions of single-hand Spades.

---

## Open Problems

### 1. Joint Bidding and Play Optimization

All existing approaches treat bidding and play as separate problems. In practice, the optimal bid depends on the play strategy (a stronger player can bid more aggressively), and the optimal play strategy depends on the bid (a team that bid 7 plays differently than one that bid 4). Joint optimization -- training a single agent that makes both bidding and play decisions -- would capture these interactions. This requires end-to-end reinforcement learning across the full hand lifecycle.

### 2. Multi-Hand Strategy (Bag and Score Management)

Current Spades AI treats each hand independently. The bag penalty system (-100 points at 10 accumulated bags) creates a strategic consideration that spans multiple hands. A team at 8 bags should bid and play differently than a team at 2 bags. Similarly, a team at 480 points (near game end) should play differently than one at 200 points. No published work addresses this multi-hand planning problem in Spades. Approaches from RL (modeling the full game as an episodic MDP with score/bags as state) are directly applicable but unexplored.

### 3. Partnership Signaling Emergence

Can self-play RL agents develop effective signaling conventions in Spades? In Bridge, self-play produces opaque conventions that do not transfer to human partners. Spades' simpler structure (no explicit convention system, signals are purely through card choice) might make emergent signaling more natural and interpretable. This is an open question connecting to the broader emergent communication literature in multi-agent RL and the Hanabi challenge.

### 4. Nil Strategy Optimization

Nil bids create highly asymmetric gameplay that has not been formally optimized. The nil bidder and their partner essentially play a cooperative subgame against the opposing team's focused attack. Optimal nil defense (by the partner) and nil attack (by opponents) require reasoning about the nil bidder's exact hand composition, which changes as cards are played. This is a minimax problem with rapidly resolving information -- amenable to search but unexplored in published literature.

### 5. Exploitability Measurement

Like Bridge, there is no practical way to measure the exploitability of a Spades strategy. The team structure makes best-response computation intractable. Proxy metrics (win rate vs. baseline, score differential, bid accuracy, nil success rate) are used instead, but these do not provide theoretical guarantees about strategy quality.

### 6. Robust Opponent Modeling

Current approaches assume a fixed opponent model. In competitive Spades, opponents vary in skill, aggressiveness, and bidding tendencies. Adaptive opponent modeling -- detecting whether opponents are conservative or aggressive, whether they signal honestly or deceptively, whether they target your nil bids -- would improve performance but requires online learning during play.

### 7. Transfer from Bridge and Hearts AI

Bridge AI (NooK, PIMC + DDS, belief Monte Carlo search) and Hearts AI (PIMC, neural evaluation) have matured further than Spades AI. Systematic transfer -- adapting Bridge's bidding neural networks, Hearts' PIMC implementations, and Skat's policy-based inference to Spades -- is an obvious research direction but has not been published.

---

## Relevance to Myosu

### Solver Architecture Assessment

| Factor | Score | Rationale |
|--------|-------|-----------|
| CFR applicability | 2/5 | Team structure requires TMECor, not standard CFR. Small game abstractions may be feasible. |
| Neural value network potential | 4/5 | Trick estimation is highly learnable. Bidding models can be trained on simulation data. |
| Abstraction necessity | 2/5 | Moderate state space (smaller than Bridge). PIMC handles play without abstraction. |
| Real-time solving value | 5/5 | PIMC is the canonical real-time approach and is highly effective for card play. |
| Transferability of techniques | 5/5 | Shares infrastructure with Bridge (10), Hearts (20), and other trick-taking games. |

### Subnet Architecture Recommendations

1. **Dual-phase evaluation**: Score bidding and play separately. Bid accuracy (how close the bid is to tricks actually won) provides a clean metric. Play quality can be measured against PIMC with perfect information (upper bound on trick count).

2. **Nil evaluation as a differentiator**: Nil bid success rate (across both bidding decisions and play execution) is a clean, high-signal metric that tests both strategic judgment and tactical execution. A solver's nil success rate when the hand warrants nil, and its nil-attack success rate against opponent nils, are strong discriminators of quality.

3. **Bag management over multi-hand sessions**: Evaluate solvers over 20+ hand sessions (to 500 points) rather than single hands. This tests the multi-hand strategic planning that separates strong from weak Spades AI. Track bags accumulated, set frequency, and final score.

4. **Duplicate format for variance reduction**: Deal the same hands to all solver pairs. This eliminates deal-distribution variance and isolates decision quality, similar to duplicate bridge.

5. **Partnership as the evaluation unit**: Score partnerships, not individual players. This naturally captures partnership coordination quality. A solver that signals well to its partner but plays suboptimally alone is still valuable in partnership.

6. **Shared trick-taking infrastructure**: The trick resolution engine, card tracking, suit-following enforcement, and PIMC sampling framework can be shared across Bridge (game 10), Spades (game 14), and Hearts (game 20). This reduces implementation effort by ~60%.

### Compute Profile

| Component | Time Budget | Resource |
|-----------|-------------|----------|
| PIMC (100 samples) per play decision | ~0.1--1s | Single CPU core |
| Bid evaluation (1000 MC simulations) | ~1--5s | Single CPU core |
| Neural network bid inference | ~1--10ms | CPU or GPU |
| GO-MCTS with transformer (50 iterations) | ~1--10s | GPU + CPU |
| Full hand (bid + 13 tricks) | ~5--30s | Mixed |
| Full game (to 500 points, ~10--20 hands) | ~2--10min | Mixed |

Spades is computationally lightweight. A full evaluation tournament of 100 games (each to 500 points) can be completed in under a day on modest hardware. This makes Spades attractive for high-throughput solver evaluation in the subnet.

### Recommended Approach for Myosu

1. **PIMC with policy-based inference** for card play. Adapt the Rebstock et al. (2019) framework from Skat: train inference networks on self-play Spades data, use policy-conditioned sampling to generate realistic hand distributions, run PIMC over these samples.

2. **Neural bidding model** trained via supervised learning on self-play data, refined through RL. Input features: hand composition (high-card points, suit lengths, spade count, void suits, singleton honors), position in bidding order, partner's bid (if already made), team score, team bags, opponent score, opponent bags. Output: bid distribution over 0--13 + nil.

3. **Specialized nil module**: Separate neural network for nil/non-nil decision, trained on labeled nil outcomes. Higher risk tolerance when trailing in score.

4. **Multi-hand planning via RL**: Model the full game (to 500 points) as an MDP where each hand is a step. State includes score, bags, and hand-level features. The RL agent learns bag management and score-sensitive bid adjustment.

5. **Transfer from Bridge and Hearts**: Reuse PIMC infrastructure from Bridge (game 10), adapt inference networks from Hearts (game 20), and share the trick-taking engine across all three games.

---

## Key Papers and References

| Year | Authors | Title | Venue | Contribution |
|------|---------|-------|-------|--------------|
| 2001 | Ginsberg | GIB: Imperfect Information in a Computationally Challenging Game | JAIR | MC + double-dummy paradigm for trick-taking AI |
| 2009 | Long, Sturtevant, Buro, Furtak | Improving State Evaluation, Inference, and Search in Trick-Based Card Games | IJCAI | State inference + evaluation improvements for Skat |
| 2011 | Long, Sturtevant, Buro, Furtak | Understanding the Success of Perfect Information Monte Carlo Sampling in Game Tree Search | AAAI | Theoretical analysis of why PIMC works for trick-taking |
| 2012 | Cowling, Powley, Whitehouse | Information Set Monte Carlo Tree Search | IEEE TCIAIG | ISMCTS: searches information sets, avoids strategy fusion |
| 2016 | Sturtevant, White | Learning to Play the Game of Hearts using RL | U. Alberta | RL for Hearts; applicable techniques for trick-taking |
| 2018 | Charlesworth | Self-Play RL for Four-Player Imperfect Information Games | arXiv | PPO self-play for 4-player card game (Big 2) |
| 2019 | Bard et al. | The Hanabi Challenge: A New Frontier for AI Research | AI Journal | Cooperative imperfect-info benchmark; theory of mind |
| 2019 | Rebstock, Solinas, Buro, Sturtevant | Policy Based Inference in Trick-Taking Card Games | IEEE CoG | Neural policy-based hand inference for Skat; applicable to all trick-taking |
| 2019 | Stammler et al. | Survey of AI for Card Games and Application to Jass | arXiv | Comprehensive survey of card game AI; covers trick-taking family |
| 2019 | Elzayn, Hayhoe, Kumar, Fereydounian | Hidden Information, Teamwork, and Prediction in Trick-Taking Card Games | RLDM | RL for Whist-family partnership games |
| 2020 | Cohensius, Meir, Oved, Stern | Bidding in Spades | ECAI | BIS algorithm: MC + ML-corrected heuristics for Spades bidding |
| 2020 | Van Hus | Artificial Neural Networks for Playing Spades | Utrecht U. Thesis | Neural network card play for Spades |
| 2020 | Stettler et al. | Challenging Human Supremacy: MCTS and DL for Jass | Springer | MCTS vs. DNN comparison for partnership trick-taking |
| 2021 | Edelkamp | Knowledge-Based Paranoia Search in Trick-Taking | IEEE CoG | Forced-win search for Skat; applicable to trick-taking |
| 2023 | McAleer et al. | Team-PSRO for Learning Approximate TMECor | NeurIPS | Team-game equilibrium via cooperative RL |
| 2023 | Carminati et al. | Multi-Player Transformation Algorithm for ATGs | ICML | Team-to-2p game reduction for adversarial team games |
| 2024 | Arjonilla et al. | Perfect Information Monte Carlo with Postponing Reasoning (EPIMC) | ALA Workshop | Strategy fusion mitigation for imperfect-info games |
| 2024 | (Authors) | Transformer Based Planning in Observation Space for Trick-Taking | arXiv | GO-MCTS: transformer + MCTS in observation space; tested on Hearts, Skat, The Crew |
| 2024 | Edelkamp | A Framework for General Trick-Taking Card Games | KI 2024 | Unified framework supporting Spades, Hearts, Euchre, etc. |
| 2024 | (Authors) | Warm-Started CFR for TMECor in Adversarial Team Games | Neurocomputing | Accelerated team-game solving via seed strategies |
| 2025 | (Authors) | PRR-TM: Equilibria in Adversarial Team Games | IJMLC | Perfect-recall refinement with teammate modeling |
| 2025 | Edelkamp | Outer-Learning Framework for Multi-Player Trick-Taking Card Games | arXiv | Self-improving statistical engine for Skat; bootstrapping approach |

### Software and Resources

| Resource | URL | Notes |
|----------|-----|-------|
| OpenSpiel | github.com/google-deepmind/open_spiel | DeepMind's game framework; Hearts env included, extensible to Spades |
| RLCard | rlcard.org | RL toolkit for card games; extensible to trick-taking |
| NeuralPlay Spades | neuralplay.com/spades.html | Commercial Spades AI using MC simulation |
| BIS Paper | arxiv.org/abs/1912.11323 | Bidding-in-Spades algorithm, open access |
| Van Hus Thesis | studenttheses.uu.nl | Neural network Spades play, open access |
| Edelkamp Framework | (KI 2024 proceedings) | General trick-taking framework supporting Spades |
| GO-MCTS Paper | arxiv.org/abs/2404.13150 | Transformer-based planning for trick-taking games |
