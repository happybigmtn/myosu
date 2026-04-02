# Solving Strategies for NLHE Tournament (ICM)

**Date:** 2026-03-30
**Scope:** Comprehensive survey of ICM-integrated tournament solving methods, 2000--2026
**Status:** Research report (no implementation)

---

## Executive Summary

No-Limit Hold'em Tournament poker (MTTs and SNGs) is fundamentally distinct from cash game NLHE because the Independent Chip Model (ICM) introduces a non-linear mapping from chip stacks to prize equity, transforming the game from a zero-sum contest into a non-zero-sum multi-agent problem. This single property invalidates most convergence guarantees of standard CFR, breaks the minimax theorem, and creates a setting where even "perfect" GTO play against imperfect opponents can lose dollar expected value ($EV). No superhuman tournament poker AI exists as of 2026.

The practical state of the art divides into two regimes. For short-stack push/fold situations (below ~15 big blinds), ICM-aware Nash equilibrium solvers (ICMIZER, HRC) compute near-optimal preflop strategies for fields up to hundreds of players, using fictitious play over ICM-valued game trees. For deep-stack postflop play under ICM, the 2024 breakthrough by GTO Wizard (Beardsell & Inariba) enables fast, accurate ICM-adjusted postflop solving by combining a novel closed-form ICM approximation with neural-guided subgame solving, reducing computation from hours to seconds for two-player postflop spots even with thousands of remaining players. Extensions to progressive knockout (PKO), satellite, and mystery bounty formats have followed. Future Game Simulation (FGS), implemented in ICMIZER 3, extends ICM by looking multiple hands ahead and accounting for blind escalation and positional rotation.

Major open problems remain: multi-way postflop ICM solving (only 3-way is currently tractable), full MTT trajectory optimization (solving across the entire tournament rather than spot-by-spot), non-zero-sum equilibrium refinement (Nash equilibrium is strategically unstable under ICM), scalable opponent exploitation under ICM pressure, and bridging the gap between the Malmuth-Harville ICM model's known biases and empirically accurate finishing probabilities.

---

## Game Complexity Analysis

### How ICM Transforms the Game

In cash game NLHE, every chip has identical marginal value. This makes the game zero-sum: one player's gain is another's loss, and CFR converges to a Nash equilibrium with a guaranteed non-losing strategy. Tournaments destroy this property through three mechanisms:

1. **Non-linear chip utility.** The Malmuth-Harville ICM model assigns finishing probabilities based on stack ratios, then maps these to a payout structure. Because payouts are top-heavy and diminishing (1st place gets far more than 2x of 2nd), the marginal value of each additional chip decreases. Doubling a 50BB stack does not double prize equity.

2. **Leakage.** When two players contest a pot in a tournament, part of the value "leaks" to non-participants. The sum of $EVs for the two players in the hand is not constant -- it depends on how aggressively they play. More aggressive strategy pairs leak more $EV to bystanders. This makes the pairwise interaction non-zero-sum even in a heads-up pot within a larger tournament.

3. **Global state dependence.** Every decision depends on the chip stacks of all players in the tournament (not just at the current table), the blind level, proximity to payout thresholds, and the remaining payout structure. A hand that is a clear call in chips can be a clear fold in dollars.

### Effective State Space

| Metric | Cash Game | Tournament |
|--------|-----------|------------|
| Per-hand game tree | ~10^161 nodes | Same |
| Utility function | Linear (chip count) | Non-linear (ICM over all stacks) |
| State variables per decision | Hand + stacks at table | Hand + all stacks + blind level + payout structure |
| Zero-sum property | Yes (two-player) | No |
| Stationarity | Yes | No (blinds escalate, players eliminated) |
| Independence between hands | Yes | No (stack changes affect future ICM equity) |

The tournament adds a continuous, high-dimensional "meta-state" (the chip distribution across all remaining players) on top of the already-intractable per-hand game tree. Exact computation of ICM equity for N players requires O(N!) operations for the Malmuth-Harville model, making naive implementations infeasible beyond ~15 players and even optimized versions infeasible beyond ~25-30 players for exact solutions.

---

## ICM Theory: Mathematical Foundations

### The Malmuth-Harville Model

The standard ICM was independently developed by David Harville (1973, for horse racing) and Mason Malmuth (1987, for poker). The model computes the probability of each player finishing in each position using a recursive formula:

- P(player i finishes 1st) = s_i / S, where s_i is player i's stack and S is the total chips in play.
- P(player i finishes 2nd) = sum over all j != i of [P(j finishes 1st) * P(i finishes 1st | j eliminated)], where the conditional probability is computed by removing j's stack and normalizing.
- This recurses for each finishing position.

Prize equity for player i = sum over all positions k of [P(i finishes kth) * Prize(k)].

### Known Limitations of Malmuth-Harville

1. **Skill-blindness.** ICM assumes all players are equally skilled. It treats the tournament as a sequence of random eliminations weighted only by stack size.

2. **Position and blind ignorance.** ICM does not account for positional advantage or blind costs. A 20BB stack about to post the big blind has different real equity than a 20BB stack on the button, but ICM treats them identically.

3. **Large-stack underestimation.** Kim (2025, arXiv:2506.00180) provides the first large-scale empirical validation of ICM using 10,000+ tournament results. The study finds ICM systematically underestimates the performance of large stacks (mean residual +5.59 x 10^-3, p=0.004) and overestimates the performance of short stacks (mean residual -4.44 x 10^-3, p<0.001). Medium stacks showed no significant bias.

4. **Inadequacy for 3+ player finishing order.** Diaconis and Ethier (2022, *Statistical Science*) use gambler's ruin analysis to show that the Harville-Malmuth finishing probabilities are "inadequate" for the 3-player case, diverging from the true gambler's ruin probabilities. They propose a regression adjustment that yields better approximations. Their Harville-Malmuth formulas only coincide with gambler's ruin estimates in the 2-player case.

5. **Computational intractability.** Exact ICM calculation for N players involves N! permutations. Naive implementations fail beyond ~15 players. HoldemResources Calculator's optimized algorithm handles up to ~500 players, but large MTT fields (1000+) require further approximation.

### Alternative Models

**Dependent Chip Model (DCM).** Besalu (2021, arXiv:2102.07738) proposes a recursive game-tree simulation that tracks hand-by-hand elimination probabilities. DCM awards more to top finishers and less to bottom finishers compared to ICM, which can change optimal decisions in marginal spots. However, DCM is computationally more expensive and has not been widely adopted.

**Ben Roberts Model.** An alternative probability assignment that performed competitively against ICM in empirical tests (losing ~1% of buy-in against a perfect strategy, similar to Harville).

**Malmuth-Weitzman Model.** An adjustment by Mark Weitzman to smooth known ICM biases, particularly for large fields. Not widely implemented in commercial tools.

**Future Game Simulation (FGS).** Developed by ICMIZER, FGS extends ICM by simulating multiple hands forward (depths 1-6), accounting for blind escalation, positional rotation, and attrition effects. FGS treats the tournament as an "evolving war campaign" rather than a single-battle analysis. FGS is strictly more accurate than ICM but requires substantially more computation. ICMIZER 3 uses a CFR+ engine for FGS calculations.

---

## Current Best Approaches

### Regime 1: Push/Fold Solving (< 15 BB)

At short stack depths, the game collapses to a binary decision: shove all-in or fold. This dramatically reduces the game tree and makes ICM-aware equilibrium computation tractable.

**ICMIZER 3.** Computes ICM/FGS Nash equilibrium for preflop push/fold with up to 500 remaining players. Uses CFR+ as the underlying engine. Handles arbitrary preflop action sequences (limps, raises, reraises). Supports MTT, SNG, satellite, and PKO formats. The FGS model at depth 4-6 provides meaningfully better results than basic ICM.

**HoldemResources Calculator (HRC).** Computes Nash equilibrium ranges for push/fold and 3-bet/4-bet scenarios. Known for high accuracy in SNG endgame analysis. Developed a novel approximation algorithm for large-field MTT ICM that compresses the field to a representative set of ~20 "hidden" stacks with adjusted payout structures, maintaining accuracy within ~3% of exact ICM.

**Gilpin, Sandholm & Sorensen (2008).** The foundational academic work on computing approximate jam/fold equilibria for 3-player SNG endgames with ICM payoffs. Uses fictitious play initialized with ICM values in an inner loop, with value iteration in an outer loop to adjust state values. This establishes that push/fold ICM games are computationally solvable to near-exact equilibrium.

**Key property:** Push/fold under ICM is the one regime where tournament poker is "essentially solved" for practical purposes. Commercial tools produce near-optimal strategies that serve as a litmus test for any competitive agent.

### Regime 2: Deep-Stack Postflop ICM Solving

This is where the 2024 breakthrough occurred. Historically, postflop ICM solving was either prohibitively slow (full CFR with ICM utility) or highly approximate (solving for chip EV then applying ad-hoc ICM corrections).

**GTO Wizard's ICM Breakthrough (2024).** Engineers Philippe Beardsell and Wataru Inariba developed a technique described as "one of the most significant theoretical breakthroughs in tournament poker since the invention of the ICM." The key innovation is a method to compute ICM-adjusted utilities efficiently -- reportedly a closed-form or near-closed-form approximation of ICM values that avoids the O(N!) enumeration problem. This enables:

- Postflop ICM solving in seconds for 2-player spots, even with thousands of remaining players
- Custom ICM solving for any tournament format: freezeout, KO, PKO, satellite, mystery bounty
- Integration with the GTO Wizard AI neural subgame solver
- ~25% reduction in exploitability compared to previous approaches (when combined with QRE)
- Nearly 600 pre-computed final table ICM solutions as of early 2025

The technique is described as "orders of magnitude more accurate than other approximate methods while requiring only a fraction of the computational time." Exact algorithmic details have not been publicly disclosed.

**3-Way Postflop ICM Solving.** As of 2025, GTO Wizard supports 3-way postflop ICM solving with full control over stack sizes, bet sizes, ranges, and rake. This represents the current frontier -- multiway (4+ player) postflop ICM solving remains intractable.

### Regime 3: Blueprint + ICM Overlay

For practical play across an entire tournament, the most common approach is:

1. Compute a chip EV blueprint strategy using standard CFR/MCCFR on an abstracted NLHE game.
2. At decision time, adjust the blueprint based on ICM considerations: tighten ranges under ICM pressure, widen ranges when short-stacked (less ICM equity to lose), and factor in bubble dynamics.
3. For critical spots, invoke a real-time ICM subgame solver.

This is essentially an adaptation of the Libratus/Pluribus architecture with ICM corrections layered on top. The weakness is that ICM corrections applied post-hoc to a chip EV strategy may not converge to the true ICM-optimal strategy, since ICM changes not just the value of outcomes but the structure of optimal play.

---

## Tournament-Specific Dynamics

### Bubble Factor and Risk Premium

**Bubble factor** = (equity lost if you lose) / (equity gained if you win). In a cash game, bubble factor is always 1.0. In a tournament:

- On the bubble with a medium stack: bubble factor can reach 1.5--3.0+
- Near a satellite ticket threshold: bubble factor can approach infinity (additional chips have zero value once you secure a seat)
- At the final table with large pay jumps: bubble factor fluctuates with each elimination

**Risk premium** quantifies the extra equity beyond pot odds needed to justify a call under ICM. It ranges from ~0.10 (early tournament, minimal ICM) to ~0.50+ (stone bubble, covered by a big stack). Dara O'Kearney's work popularized risk premium as a practical heuristic for tournament players.

### rMDF: Risk-Adjusted Minimum Defense Frequency

A 2024 conceptual framework that adjusts the standard Minimum Defense Frequency (MDF) for ICM pressure. In cash games, MDF determines how often you must defend against bets to prevent opponent profitability. Under ICM:

- Chips lost hurt more than chips gained help
- MDF must decrease (defend less) as ICM pressure increases
- rMDF incorporates a Risk Premium parameter (0.10--0.50) to adjust defense frequencies in real time

This is a heuristic bridge between theoretical MDF and practical tournament decision-making, not a solver-derived formula.

### Final Table Dynamics

The final table concentrates ICM effects because:

1. Every elimination creates a pay jump for all survivors
2. Stack distributions become more asymmetric
3. The transition from 3-handed to heads-up eliminates ICM entirely (two-player = pure chip EV)
4. Paradoxically, the final table bubble (e.g., 10th vs 9th place) often has a small pay jump, creating less ICM pressure than the money bubble

Solver analysis reveals that ICM pressure at the final table fundamentally restructures ranges -- it does not merely tighten them. Hand categories re-order in priority: hands with high all-in equity but low playability (suited connectors) lose value relative to hands with high card strength (broadway cards).

### Satellite Tournaments

Satellites create extreme ICM dynamics because the payout is binary: either you win a seat (all seats are equal value) or you win nothing. Once a player's stack exceeds the average needed to secure a seat, additional chips have near-zero marginal value. This makes satellites the purest expression of ICM strategy and a valuable test case for solvers.

### Progressive Knockout (PKO) Tournaments

PKO adds a second value function: each player carries a bounty that is partially awarded to whoever eliminates them. GTO Wizard's 2024 approach models PKO by combining three components:

1. **Immediate bounty EV:** the expected bounty value from eliminating an opponent this hand
2. **Future bounty EV:** a proportional model estimating bounties gained in future hands based on stack trajectory
3. **ICM prize equity:** standard ICM calculation for the remaining prize pool

The interaction between bounty incentives and ICM survival pressure creates scenarios where correct strategy diverges dramatically from either pure chip EV or pure ICM play.

---

## Abstraction Challenges Unique to Tournaments

### Variable Stack Depths

Cash game solvers can pre-compute strategies for a fixed effective stack depth. Tournaments require strategies across a continuously varying range of depths (from 200+ BB in early levels to <10 BB at critical moments), and the optimal strategy at each depth depends on the tournament state (ICM) rather than purely on the depth itself.

**Current approach:** Stack-depth indexed strategies that pre-compute solutions at representative depths (5, 10, 15, 20, 30, 50, 100 BB) and interpolate. The ICM correction varies at each depth, so a separate ICM overlay is needed per depth bracket.

### Changing Blind Structures

Blinds escalate on a fixed schedule, creating non-stationarity. A strategy optimal at one blind level becomes suboptimal at the next. FGS addresses this by looking ahead, but full integration of blind schedule dynamics into the solving process remains an open problem.

### Table Composition Changes

As players bust, tables merge and compositions change. The opponent distribution shifts (weaker players eliminated earlier on average), and stack distributions at a given table can change suddenly when a new player is moved in. Cash game solvers assume a fixed game structure; tournament solvers must handle dynamic game parameters.

### Multi-Table Information

In MTTs, information about other tables is partially observable (stack sizes are public, cards are not). Near the bubble, the optimal strategy at your table depends on short stacks at other tables who might bust first. Incorporating this cross-table reasoning into a solver would require modeling the tournament as a single massive extensive-form game, which is computationally infeasible. Current approaches treat each table independently and apply ICM corrections based on the global chip distribution.

---

## Equilibrium Theory Under ICM

### Why Nash Equilibrium Breaks

In two-player zero-sum games, Nash equilibrium guarantees a non-losing strategy. ICM breaks this in three ways:

1. **Leakage makes the game non-zero-sum.** The pot contested by two players distributes value to bystanders. Playing Nash against a Nash opponent in an ICM spot may leak more $EV to the field than playing a different (non-equilibrium) strategy.

2. **More actions can decrease EV.** In standard game theory, giving a player more options cannot decrease their equilibrium payoff. Under ICM, adding bet sizes can force opponents to respond more aggressively, increasing leakage for both players.

3. **Multiplayer instability.** In 3+ player games, Nash equilibrium does not guarantee non-exploitability. Two players can coordinate (even implicitly) to exploit a third player who is playing Nash.

### Quantal Response Equilibrium (QRE)

GTO Wizard introduced QRE to poker solving in 2025. Instead of assuming opponents play perfectly (Nash), QRE assumes opponents make mistakes with probabilities inversely related to the cost of the mistake. This produces:

- Strategies that are optimal against imperfect opponents rather than perfect ones
- Better convergence in "ghost lines" (uncommon action sequences rarely reached in Nash solving)
- ~25% reduction in exploitability of flop solutions compared to Nash-based approaches
- Natural integration with ICM (where opponent mistakes are common and costly)

QRE is the default algorithm for custom spots in GTO Wizard AI as of 2025, while pre-solved libraries continue to use Nash.

### Exploitability Measurement

Unlike cash game solvers where a best-response calculation measures exploitability, tournament exploitability is harder to define and measure because:

- The game is non-zero-sum, so max-exploit by one player involves assumptions about all other players
- Tournament outcomes are sampled over many hands with high variance
- ICM values are themselves approximations, introducing measurement uncertainty

---

## Open Problems

### 1. Full MTT Trajectory Optimization
No solver currently optimizes play across an entire tournament trajectory. All existing approaches solve spot-by-spot, applying ICM corrections independently at each decision. A true tournament solver would jointly optimize early-game chip accumulation, middle-game survival, bubble play, and final table strategy as a unified policy.

### 2. Multiway Postflop ICM (4+ Players)
Current postflop ICM solving is limited to 3-way spots. At a 6-max or 9-max table, multiway pots involve 4+ players, and ICM-aware solving for these spots remains computationally infeasible.

### 3. Non-Zero-Sum Equilibrium Refinement
Nash equilibrium is the wrong solution concept for ICM games. Better solution concepts (correlated equilibrium, team-maxmin, QRE) exist but are either computationally harder or not yet well-understood in the tournament context. The field needs a principled equilibrium concept that accounts for leakage and multiplayer instability.

### 4. Accurate Large-Field ICM
The Malmuth-Harville model has known biases (large-stack underestimation, short-stack overestimation). While Kim (2025) quantified these biases, no widely-adopted replacement model exists. The Dependent Chip Model (DCM) and gambler's ruin corrections are research proposals that have not been integrated into commercial solvers.

### 5. Opponent Exploitation Under ICM
Cash game exploitation (detecting and exploiting opponent tendencies) is well-studied. Tournament exploitation is harder because: (a) ICM makes exploitation risky (high bubble factor means failed exploitation costs more), (b) table compositions change frequently, limiting sample sizes, and (c) the optimal exploitation strategy depends on all players' tendencies, not just the target's. Research shows that in ICM spots, reads need to be correct ~96% of the time for max-exploit to be profitable versus a GTO-like baseline.

### 6. Blind Structure Integration
Current solvers treat each hand independently. Integrating the blind schedule (future increases creating urgency for short stacks) into the value estimation would improve accuracy but significantly increases computation.

### 7. Bounty and Non-Standard Format Modeling
While PKO solving has advanced, formats like mystery bounties, re-entry tournaments, and satellite qualifiers each introduce unique utility functions that require format-specific solver adaptations.

---

## Relevance to Myosu

### Architectural Implications

Tournament NLHE is a stress test for solving architectures because it combines several properties that are individually challenging and collectively unique:

| Property | Challenge for Solver |
|----------|---------------------|
| Non-zero-sum | CFR convergence guarantees lost |
| Non-stationary | Strategies must adapt to changing blinds and field |
| Non-linear utility | ICM transforms the objective function |
| Multi-agent | 2-10 players per table, hundreds-thousands in tournament |
| Variable horizons | Tournament length is stochastic |
| Mixed regimes | Deep-stack postflop and short-stack push/fold coexist |

### Recommended Approach

1. **Chip EV foundation.** Build a strong NLHE solver (CFR/MCCFR for blueprint, neural subgame solving for refinement) that operates in chip EV space. This is the well-understood base.

2. **ICM correction layer.** Train a neural network to predict ICM equity adjustments given the tournament state (stack distribution, payout structure, blind level). Apply these corrections at decision time to modify the chip EV strategy.

3. **Push/fold oracle.** For stacks below 15 BB, use exact ICM Nash push/fold computation (the solved regime). This serves as both a strategy component and an evaluation metric.

4. **Evaluation protocol.** Use SNG formats for evaluation to control variance. Primary metric: $EV (tournament equity), not chip count. Push/fold accuracy against known Nash charts serves as a fast litmus test for agent competence.

5. **FGS integration.** For higher accuracy, implement a look-ahead model (FGS-style) that accounts for blind escalation and positional costs over the next 1-3 orbits.

### Evaluation Considerations for the Subnet

- Tournament strategy quality requires large sample sizes (thousands of tournaments minimum) due to inherent variance
- An ICM verification oracle is needed to judge decision quality in specific spots
- Different tournament formats (MTT, SNG, PKO, satellite) stress different aspects of the solver and may warrant separate evaluation tracks
- Push/fold accuracy is a cheap, fast, and reliable quality signal

---

## Key Papers and References

### Foundational ICM Theory

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 1973 | Harville | "Assigning Probabilities to the Outcomes of Multi-Entry Competitions" | Original probability model for finishing order |
| 1987 | Malmuth | Application to poker tournaments | ICM formulation for poker |
| 2009 | Gilbert & McGlothlin | "The Independent Chip Model and Risk" (arXiv:0911.3100) | Risk aversion analysis in ICM context |
| 2022 | Diaconis & Ethier | "Gambler's Ruin and the ICM" (*Statistical Science*) | Shows ICM is inadequate for 3-player finishing probabilities; proposes regression adjustment |
| 2025 | Kim | "Empirical Validation of the ICM" (arXiv:2506.00180) | First large-scale empirical validation; quantifies large-stack underestimation and short-stack overestimation biases |

### Alternative Models

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 2021 | Besalu | "The Dependent Chip Model (DCM)" (arXiv:2102.07738) | Recursive game-tree alternative to ICM; awards more to top finishers |

### Tournament Equilibrium Computation

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 2008 | Gilpin, Sandholm & Sorensen | "Computing Approximate Jam/Fold Equilibrium for 3-Player SNGs" (AAMAS) | Fictitious play + value iteration for ICM push/fold Nash |
| 2024 | Beardsell & Inariba (GTO Wizard) | ICM breakthrough (unpublished/proprietary) | Fast, accurate postflop ICM solving for arbitrary tournament formats |

### Core Poker AI (Applicable to Tournament Extension)

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 2007 | Zinkevich et al. | "Regret Minimization in Games with Incomplete Information" (NIPS) | CFR algorithm |
| 2017 | Moravcik et al. | "DeepStack" (*Science*) | Neural continual re-solving |
| 2017 | Brown & Sandholm | "Libratus" (*Science*) | Blueprint + nested subgame solving |
| 2019 | Brown & Sandholm | "Pluribus" (*Science*) | 6-player NLHE, MCCFR + depth-limited search |
| 2020 | Brown et al. | "ReBeL" (NeurIPS) | RL + search for imperfect info games |
| 2022 | Liu et al. | "DecisionHoldem" | Safe depth-limited solving with diverse opponents |
| 2024 | Sonawane & Chheda | "A Survey on Game Theory Optimal Poker" (arXiv:2401.06168) | Comprehensive survey of GTO solving techniques |
| 2025 | Yao et al. | "Beyond GTO: Profit-Maximizing Poker Agents" (arXiv:2509.23747) | GTO + real-time exploitation; MCCFR + opponent modeling |
| 2025 | Zhuang et al. | "PokerBench" (AAAI 2025) | LLM evaluation benchmark for poker; 11,000 scenarios |

### Equilibrium Concepts

| Year | Authors | Title | Contribution |
|------|---------|-------|-------------|
| 2025 | GTO Wizard | "Introducing QRE" (blog) | Quantal Response Equilibrium for poker; handles bounded rationality |
| 2025 | Various | "Generalized QRE: Existence and Efficient Learning" (arXiv:2507.09928) | Theoretical foundations for generalized QRE |

### Commercial Tools

| Tool | Developer | Capabilities |
|------|-----------|-------------|
| ICMIZER 3 | ICMPoker | ICM/FGS Nash push/fold, up to 500 players, CFR+ engine |
| HRC | HoldemResources | ICM Nash push/fold and 3-bet/4-bet, large-field approximation |
| GTO Wizard | GTO Wizard Inc. | Postflop ICM solving, PKO, satellite, QRE, neural subgame solver |
| GTO LAB | GTO LAB | ICM and postflop GTO trainer for MTT |

---

## Summary of the State of the Art

| Aspect | Status (2026) |
|--------|---------------|
| Push/fold ICM (< 15 BB) | Essentially solved; commercial tools compute near-exact Nash |
| Postflop ICM (2-player) | Fast and accurate via GTO Wizard 2024 breakthrough |
| Postflop ICM (3-way) | Solvable with current tools, but computationally expensive |
| Postflop ICM (4+ way) | Unsolved / intractable |
| Full tournament trajectory | Unsolved; all approaches are spot-by-spot |
| Large-field ICM accuracy | Good approximations exist; exact Malmuth-Harville infeasible beyond ~30 players |
| ICM model accuracy | Known biases (Kim 2025); no widely-adopted replacement |
| Superhuman tournament AI | Does not exist |
| Equilibrium concept | Nash is inadequate under ICM; QRE shows promise but is early-stage |
