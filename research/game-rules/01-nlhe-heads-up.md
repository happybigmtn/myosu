# No-Limit Hold'em Heads-Up (NLHE HU)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | No-Limit Texas Hold'em, Heads-Up |
| Variants | Cash game, fixed-stack (e.g., 100bb, 200bb) |
| Players | 2 |
| Information | Imperfect (hidden hole cards) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes |
| Solved Status | Essentially solved (Cepheus, 2015) for limit variant; approximate Nash for no-limit |

## Overview

No-Limit Hold'em Heads-Up is a two-player poker variant where each player receives two private hole cards and shares five community cards. The objective is to win chips by making the best five-card hand or forcing the opponent to fold. Heads-up is widely considered the purest form of poker, as every hand is contested directly between two players with no multiway dynamics.

**Players:** Exactly 2
**Deck:** Standard 52-card deck, no jokers
**Betting structure:** No-limit (any bet up to a player's entire stack)

## Setup

### Deck and Cards
Standard 52-card deck. Cards are ranked A (high) through 2 (low). Aces may also play low in the straight A-2-3-4-5 (the "wheel"). Suits have no ranking.

### Positions and Blinds
There are two positions:
- **Button / Small Blind (BTN/SB):** Posts the small blind. Also acts as the dealer.
- **Big Blind (BB):** Posts the big blind, which is exactly 2x the small blind.

The button alternates between players after every hand.

### Dealing
1. Each player posts their respective blind.
2. The dealer burns one card, then deals one card face-down to the BB, then one to the BTN/SB.
3. A second round of dealing gives each player their second hole card (BB first, then BTN/SB).
4. Each player now has exactly two private hole cards.

**Note on dealing order:** In heads-up, the BTN/SB receives the last card dealt. This is consistent with the rule that the button always receives the last card.

## Game Flow

A hand proceeds through up to four betting rounds, each separated by community card deals.

### 1. Preflop
- **First to act:** BTN/SB acts first preflop. This is the critical heads-up exception to the standard rule. In all other formats, the player left of the BB acts first preflop.
- **Options:** Fold, call (match the big blind), or raise (minimum raise to 2x BB).
- If BTN/SB calls (a "limp"), BB may check or raise.
- If BTN/SB raises, BB may fold, call, or re-raise.
- Betting continues until both players have acted and all bets are matched, or one player folds.

### 2. Flop
- Dealer burns one card, then deals three community cards face-up in the center.
- **First to act:** BB acts first on all postflop streets.
- Options: Check or bet.
- If a bet is made, opponent may fold, call, or raise.

### 3. Turn
- Dealer burns one card, then deals one community card face-up (fourth community card).
- Betting proceeds identically to the flop. BB acts first.

### 4. River
- Dealer burns one card, then deals one final community card face-up (fifth community card).
- Betting proceeds identically to the flop and turn. BB acts first.

### 5. Showdown
- If betting on the river completes with both players still in, hands are revealed and the best five-card hand wins.
- If at any point one player folds, the remaining player wins the pot without showing cards.

## Betting Rules

### No-Limit Structure
- A player may bet any amount from the minimum bet up to their entire chip stack at any time.
- **Minimum bet:** Equal to the big blind on any street.
- **Minimum raise:** The raise increment must be at least equal to the previous raise increment. The total bet must be at least the previous bet plus the previous raise size.
  - Example: Blinds 50/100. BB is 100. BTN raises to 300 (raise increment = 200). BB's minimum re-raise is to 500 (300 + 200).
  - Example: On the flop, Player A bets 150. Player B's minimum raise is to 300 (raise increment = 150). Player A's minimum re-raise is to 450 (300 + 150).
- **Maximum bet:** A player's entire remaining chip stack (an "all-in").
- There is no cap on the number of raises.

### All-In Rules
- A player may go all-in at any time for any amount up to their stack.
- If a player goes all-in for less than the minimum raise amount, this does **not** reopen the betting to the previous aggressor. The opponent may only call or fold.
  - Example: Blinds 50/100. BTN raises to 300. BB goes all-in for 400 (raise increment of only 100, less than the previous 200 increment). BTN may only call 100 more or fold. BTN cannot re-raise.
- If a player goes all-in for an amount that **does** constitute a full raise (increment >= previous raise increment), the opponent may fold, call, or re-raise.

### Side Pots
In heads-up play with only two players, side pots do not arise. The all-in player can win at most the amount they wagered from the opponent, plus their own chips. Any excess chips the opponent bet beyond what the all-in player can match are simply returned.

## Hand Rankings

Hands are ranked from strongest to weakest. Within each rank, ties are broken by the highest relevant cards (kickers).

| Rank | Hand | Description | Example |
|------|------|-------------|---------|
| 1 | Royal Flush | A-K-Q-J-T of the same suit | A{s}K{s}Q{s}J{s}T{s} |
| 2 | Straight Flush | Five consecutive cards of the same suit | 9{h}8{h}7{h}6{h}5{h} |
| 3 | Four of a Kind | Four cards of the same rank + one kicker | 7{s}7{h}7{d}7{c}K{s} |
| 4 | Full House | Three of a kind + a pair | Q{s}Q{h}Q{d}4{c}4{s} |
| 5 | Flush | Five cards of the same suit, not sequential | A{d}J{d}8{d}6{d}2{d} |
| 6 | Straight | Five consecutive cards, mixed suits | T{s}9{h}8{d}7{c}6{s} |
| 7 | Three of a Kind | Three cards of the same rank + two kickers | 5{s}5{h}5{d}K{c}9{s} |
| 8 | Two Pair | Two different pairs + one kicker | J{s}J{h}4{d}4{c}A{s} |
| 9 | One Pair | Two cards of the same rank + three kickers | T{s}T{h}A{d}8{c}5{s} |
| 10 | High Card | No matching ranks, no straight, no flush | A{s}Q{h}9{d}6{c}3{s} |

### Tie-Breaking Rules
- **Straight Flush / Straight:** Highest top card wins. A-2-3-4-5 is the lowest straight (the "wheel"); the ace plays low.
- **Four of a Kind:** Higher quad rank wins. If equal, higher kicker wins.
- **Full House:** Higher trips rank wins. If equal, higher pair wins.
- **Flush:** Compare cards from highest to lowest; first difference determines winner.
- **Three of a Kind:** Higher trips wins. If equal, compare kickers highest first.
- **Two Pair:** Higher top pair wins. If equal, higher second pair wins. If still equal, higher kicker wins.
- **One Pair:** Higher pair wins. If equal, compare kickers highest first.
- **High Card:** Compare cards highest to lowest.
- **Identical hands:** Pot is split equally (a "chop").

### Best Five-Card Hand
Each player constructs their best five-card hand from the seven available cards (two hole cards + five community cards). A player may use both, one, or neither of their hole cards combined with community cards.

## Showdown Rules

- The last aggressor (the player who made the final bet or raise on the river) must show first.
- If there was no betting on the river (both checked), the player first to act (BB in heads-up) shows first.
- The losing player may muck (discard without showing) rather than reveal their hand.
- If both players are all-in with no further action possible, both hands are revealed immediately (before remaining community cards are dealt).
- A player may voluntarily show their hand at any time, though this is not required.

## Special Rules

### Heads-Up Button/Blind Assignment
This is the defining structural difference for heads-up play:
- The BTN posts the small blind and acts first preflop.
- The BB acts first on all postflop streets (flop, turn, river).
- The BTN receives the dealer button and the last dealt card.

### Chopping Blinds
Chopping (both players agreeing to take back their blinds without playing) is generally not applicable in heads-up since every hand is contested.

### Straddle
Straddles are typically not used in heads-up play, though house rules may vary.

### Disconnection (Online)
Online heads-up games typically have timebanks. If a player disconnects, they are given a grace period to reconnect. If they fail to reconnect, their hand is folded (or checked if no bet is pending in some implementations).

## Key Strategic Concepts

### Position is Asymmetric
The BTN/SB has a positional disadvantage preflop (acts first) but a positional advantage postflop (acts last on flop, turn, and river). Since three of the four betting rounds favor the button, the button is the advantaged position overall.

### Wider Ranges
With only one opponent, starting hand requirements are dramatically wider than in full-ring or 6-max games. The BTN should open-raise a very high percentage of hands (often 70-90%). The BB should defend against raises at a high frequency.

### Aggression
Because fold equity is available against a single opponent, aggression is rewarded more than in multiway pots. Bluffing frequency is significantly higher.

### Stack Depth
The ratio of effective stack size to the big blind (stack-to-pot ratio) fundamentally changes strategy. Deep-stacked play (100+ BB) allows for more postflop maneuverability. Short-stacked play (< 20 BB) simplifies to push/fold strategies.

### Exploitative Play
With only one opponent, players can rapidly adjust to tendencies. If the opponent folds too frequently, increase bluff frequency. If they call too much, tighten value ranges and reduce bluffs. This dynamic adjustment cycle is the core of heads-up strategy.

### Game-Theory Optimal (GTO) Baseline
In the two-player zero-sum setting, Nash equilibrium strategies exist and are well-studied. Limit Hold'em heads-up has been essentially solved by computer programs (Cepheus). No-limit heads-up remains an active research area, with strong approximations from AI systems like Libratus and Pluribus.

## Common Terminology

| Term | Definition |
|------|------------|
| **BTN / Button** | The dealer position; posts the small blind in heads-up |
| **BB / Big Blind** | The non-dealer position; posts the big blind |
| **Limp** | Calling the big blind preflop rather than raising (from the BTN/SB) |
| **Open-raise** | The first raise preflop |
| **3-bet** | A re-raise of the initial preflop raise |
| **4-bet** | A re-raise of the 3-bet |
| **C-bet (Continuation bet)** | A bet on the flop by the preflop raiser |
| **Check-raise** | Checking to the opponent, then raising after they bet |
| **Donk bet** | A bet from the non-aggressor into the preflop raiser (BB betting into BTN on the flop) |
| **Effective stack** | The smaller of the two players' stacks; determines the maximum amount that can be won |
| **Pot odds** | The ratio of the current pot to the cost of a call |
| **Equity** | The percentage of the pot a hand expects to win at showdown |
| **Fold equity** | The additional value gained from the probability an opponent folds |
| **Push/fold** | A simplified strategy used at short stack depths where the only decisions are all-in or fold |
| **Cooler** | A situation where two very strong hands collide and significant action is unavoidable |

## State Space Analysis

### Information Sets
- Pre-flop: 169 strategically distinct starting hands (1,326 raw combinations).
- Flop: ~2.6 million distinct hand+board combinations (before abstraction).
- Full game tree (no abstraction): approximately 10^161 decision nodes (Johanson, 2013).
- With card abstraction (bucketing): reduced to ~10^12 to 10^14 information sets depending on granularity.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Raw game tree nodes | ~10^161 |
| Information sets (no abstraction) | ~3.19 × 10^14 |
| Information sets (with abstraction) | ~10^12 (Libratus-level) |
| Terminal nodes | ~10^161 |
| Effective branching factor | Variable, 2-5 typical actions per node after abstraction |

### Action Space
- Continuous in theory (any bet size).
- Typically discretized to 3-8 bet sizes per street for tractability.
- Common abstraction: fold/check, call, pot-fraction bets (0.33×, 0.5×, 0.67×, 1×, 1.5×, all-in).
- Pre-flop raising typically uses fixed sizes (2.5bb open, 3-bet to 9bb, etc.).

## Key Challenges for AI/Solver Approaches

### 1. Enormous Game Tree
The no-limit betting structure creates a combinatorial explosion. Unlike limit hold'em where bet sizes are fixed, NLHE allows any integer chip amount, making the raw game tree effectively infinite without discretization.

### 2. Card Abstraction
Grouping strategically similar hands is critical for tractability. Imperfect abstraction introduces "abstraction pathologies" where the strategy in the abstract game performs worse than the abstract game's equilibrium would suggest when mapped back to the full game.

Key abstraction techniques:
- **Expected Hand Strength (EHS)**: simple metric, loses distributional information.
- **Earth Mover's Distance (EMD)**: clusters hands by full equity distribution similarity.
- **Potential-Aware abstractions**: account for how hand strength changes on future streets.

### 3. Bet Abstraction
Reducing continuous bet sizes to discrete choices. The "off-tree" problem: when an opponent makes a bet size not in the abstraction, the solver must translate this to a nearby abstraction bet size, losing exploitability.

### 4. Exploitability vs. Exploitation
Nash equilibrium play guarantees non-negative expected value but does not maximally exploit weak opponents. The exploration-exploitation tradeoff is central to practical play.

### 5. Depth-Limited Solving
Real-time solving at each decision point with a limited look-ahead depth. Requires accurate leaf node evaluation (typically a pre-computed blueprint strategy or neural network value estimator).

## Known Solver Results

### Limit Hold'em (Solved)
- **Cepheus** (Bowling, Burch, Johanson, Tammelin, 2015): essentially solved limit hold'em heads-up. Exploitability < 0.986 mbb/hand (thousandths of a big blind per hand). Published in *Science*.
- Method: CFR+ (Counterfactual Regret Minimization Plus) with compression. 4.531 × 10^13 information sets. 68 days of computation on 200 nodes.

### No-Limit Hold'em (Approximately Solved)
- **Libratus** (Brown & Sandholm, 2017): defeated top human professionals in 120,000 hands of NLHE HU. Three-module system:
  1. Pre-computed blueprint strategy via Monte Carlo CFR with card/bet abstraction.
  2. Real-time subgame solving with safe nested endgame solving.
  3. Self-improvement module to patch blueprint leaks detected during play.
  - Published at IJCAI 2017 and *Science* 2018.

- **Pluribus** precursor techniques applied to HU as well (Brown & Sandholm, 2019).

- **DeepStack** (Moravcik et al., 2017): neural network-guided depth-limited solving. Trained value networks to estimate counterfactual values at public tree nodes. Defeated professional players in a statistically significant sample.
  - Key innovation: continual re-solving — treats every decision point as the root of a new subgame.
  - Published in *Science*.

- **ReBeL** (Brown et al., 2020, Facebook AI): combines reinforcement learning and search. Treats imperfect-information games through a perfect-information "public belief state" MDP. Achieves superhuman play in NLHE HU with less domain-specific engineering.

### Exploitability Bounds
- Best known NLHE HU strategies have estimated exploitability of ~20-50 mbb/hand (compared to 0.986 mbb/hand for limit HU).
- True exploitability is computationally intractable to measure for the full game.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2007 | Zinkevich et al., "Regret Minimization in Games with Incomplete Information" | Introduced CFR algorithm |
| 2014 | Tammelin, "Solving Large Imperfect Information Games Using CFR+" | CFR+ variant, faster convergence |
| 2015 | Bowling et al., "Solving Limit Hold'em" | Essentially solved limit HU (*Science*) |
| 2017 | Moravcik et al., "DeepStack: Expert-Level AI in HLHU Poker" | Neural-guided continual re-solving (*Science*) |
| 2017 | Brown & Sandholm, "Superhuman AI for HU NL Poker: Libratus" | Blueprint + nested subgame solving |
| 2019 | Brown & Sandholm, "Superhuman AI for Multiplayer Poker" | Pluribus (6-max), search in multiplayer |
| 2020 | Brown et al., "Combining Deep RL and Search for IIG" | ReBeL framework |

## Relevance to Myosu

### Solver Applicability
NLHE HU is the **canonical benchmark** for imperfect-information game solving. It directly validates:
- **CFR / CFR+**: The workhorse algorithm. Linear memory CFR+ is the gold standard for two-player zero-sum games.
- **Monte Carlo CFR (MCCFR)**: External sampling, outcome sampling, and public-chance sampling variants all developed primarily for poker.
- **Neural network value estimation**: DeepStack's approach of training value networks on random subgames is directly applicable.
- **Real-time subgame solving**: Libratus's safe nested endgame solving with "reach" margin vectors.
- **Depth-limited solving with learned leaf evaluations**: Core technique transferable to any game with sequential decisions under uncertainty.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 5/5 | Ideal two-player zero-sum structure |
| Neural value network potential | 5/5 | DeepStack/ReBeL proven |
| Abstraction necessity | 4/5 | Required for tractability |
| Real-time solving value | 5/5 | Libratus's key innovation |
| Transferability of techniques | 5/5 | Foundation for all other poker variants |

### Myosu Subnet Considerations
- NLHE HU serves as the **baseline validator**: any solver architecture must handle this game competently before being trusted on harder games.
- Strategy representation: compressed blueprint strategies can be stored on-chain as compact models.
- Verification: exploitability of submitted strategies can be bounded via best-response computation (computationally expensive but feasible for abstracted games).
- Oracles: hand history verification is straightforward — standard poker hand evaluation is deterministic and well-specified.

### Recommended Approach for Myosu
1. Use CFR+/MCCFR for blueprint computation.
2. Pair with neural value networks for real-time depth-limited solving.
3. Evaluate submitted strategies via exploitability estimation (lower-bound best-response sampling).
4. Use as calibration game — known exploitability results provide ground truth for solver quality metrics.
