# Liar's Dice

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Liar's Dice (Perudo / Dudo) |
| Variants | Single-round (one die each), multi-die (standard), Perudo (South American rules) |
| Players | 2-6 (commonly 2-6) |
| Information | Imperfect (hidden dice values) |
| Stochasticity | Stochastic (dice rolls) |
| Zero-Sum | Yes (elimination game) |
| Solved Status | Partially solved (2-player, 1 die each = fully solved; larger variants unsolved) |

## Overview

Liar's Dice is a bluffing dice game of hidden information where players make increasingly bold claims about the combined dice values under all players' cups. The game has ancient origins and exists in many cultural variants, with **Perudo** (also called Dudo) being the most codified tournament version, originating in South America. The core mechanic -- bidding on hidden information and challenging bluffs -- makes it one of the purest bluffing games.

- **Players:** 2--6 (best with 3--6)
- **Duration:** 15--30 minutes
- **Objective:** Be the last player with dice remaining.

## Equipment

- **5 six-sided dice** per player (standard d6)
- **1 opaque cup** per player (to hide dice rolls)
- A flat playing surface

Total dice in play: 5 x number of players (e.g., 25 dice for 5 players).

## Setup

1. Each player takes 5 dice and a cup.
2. All players simultaneously shake their cups and slam them face-down on the table, peeking at only their own dice without revealing them to others.
3. If any die lands on top of another, that player re-rolls all their dice.
4. Determine the starting player (youngest, highest visible die, random, or winner of a single die roll-off).

## Game Flow

### Turn Structure

Play proceeds clockwise. On each turn, a player must do exactly one of the following:

1. **Raise the bid** -- make a higher bid than the current one.
2. **Challenge** -- call "Liar!" (or "Dudo!") to challenge the current bid.
3. **Call Exact** (optional rule) -- claim the current bid is exactly correct.

### Bidding

A bid consists of two components:

- **Quantity:** The claimed minimum number of dice showing a specific face value across ALL players' cups combined.
- **Face value:** Which pip value (1--6) is being claimed.

Example: "Four fives" means "there are at least four dice showing 5 among all players' dice."

#### Valid Bid Escalation

Each new bid must be strictly higher than the previous bid. A bid is higher if:

1. **Same face value, higher quantity:** "Three 4s" → "Four 4s"
2. **Higher face value, same or higher quantity:** "Three 4s" → "Three 5s" or "Three 6s" or "Four 5s"
3. **Lower face value, higher quantity:** "Three 4s" → "Four 2s" or "Four 3s"

You cannot bid the same quantity and face as the previous bid.

### Challenge Resolution

When a player challenges by calling "Liar!" (or "Dudo!"):

1. **All players lift their cups**, revealing all dice.
2. **Count** the total number of dice showing the bid's face value (including wilds, if using wild ones -- see below).
3. **Compare** the count to the bid's quantity:

| Actual Count vs. Bid | Result |
|----------------------|--------|
| Count >= bid quantity | Bid was valid. **Challenger loses 1 die.** |
| Count < bid quantity | Bid was wrong. **Bidder loses 1 die.** |

4. The loser removes one die permanently from the game.
5. All players re-roll their remaining dice under cups.
6. The loser of the challenge starts the next round's bidding.

### Exact Call (Optional Rule)

Instead of raising or challenging, a player may call "Exact!" (or "Spot on!"), claiming the current bid is precisely correct (not more, not fewer).

- If the count **exactly equals** the bid quantity: the caller **gains 1 die** back (up to the starting maximum of 5). All other players lose nothing.
- If the count does **not** exactly equal the bid: the caller **loses 1 die**.

The Exact call is high-risk, high-reward and is not used in all rule sets.

### Elimination

When a player loses their last die, they are eliminated from the game. Play continues until only one player remains.

## Wild Ones (Aces Wild)

In the standard Perudo/tournament version, **1s are wild** -- they count as any face value for counting purposes.

Example: If the bid is "Four 5s" and the dice show three 5s and two 1s, the actual count is **five** (three 5s + two 1s), so the bid is valid.

### Bidding on 1s Directly

When a player bids on 1s specifically, the wild rule creates special bid escalation rules:

- **Switching to 1s:** The quantity is **halved** (rounded up) relative to the last non-1 bid. Example: After "Six 3s", a valid 1s bid is "Three 1s" (ceil(6/2) = 3).
- **Switching from 1s:** The quantity is **doubled** plus one. Example: After "Three 1s", a valid non-1 bid must be at least "Seven" of any face (3 x 2 + 1 = 7).
- When 1s are bid, **1s are not wild** for that specific bid -- only actual 1s count.

### No-Wilds Variant

Some groups play without wild 1s. All dice count only at face value. This simplifies the math and removes the 1s bidding conversion rules.

## Palafico Round (Perudo)

When any player is reduced to **exactly 1 die**, a special **Palafico round** occurs:

1. The player with 1 die **starts the bidding**.
2. **1s are not wild** during this round.
3. Bids may only increase in **quantity**, not face value. The face value is locked to whatever the Palafico player initially bids.
4. A player's Palafico privilege triggers only **once** per game (if they gain a die back via Exact call and are reduced to 1 die again, no second Palafico).

## Scoring / Winning

- **Last player standing wins.** There is no point system.
- Dice are the life counter -- losing all dice means elimination.
- In tournament settings, placing order (1st, 2nd, 3rd eliminated, etc.) may be tracked for rankings.

## Special Rules

### Two-Player Endgame

When only two players remain with 1 die each, some variants change the bidding to be about the **sum of both dice** rather than face-value counts.

### Opening Bid Restrictions

- The opening bidder has no previous bid to beat, so they may bid any quantity (minimum 1) and any face value.
- Some groups require the minimum opening bid to be "one 2" (or "one" of any face).

### Simultaneous Challenges

If two players attempt to challenge simultaneously, the player whose turn it is (next in clockwise order after the bidder) has priority.

### Re-rolling

After every challenge resolution (and after every round in some variants), all remaining players re-roll their dice under cups.

## Key Strategic Concepts

- **Probability-based bidding:** With N total dice in play and wild 1s, the expected count of any face value is N/3 (each die has a 1/3 chance of being that value or a 1). Bids near N/3 are safe; bids well above are aggressive bluffs.
- **Information extraction:** The pattern of bids reveals information about what other players hold. If everyone is raising on 5s, there are probably many 5s and 1s out.
- **Bluff sizing:** Small raises (e.g., +1 quantity) put pressure on the next player without overcommitting. Large jumps signal either extreme confidence or a bold bluff.
- **Position awareness:** Players near a challenge (e.g., immediately after an aggressive bid) face the toughest decisions. Being far from the bidder in turn order gives time to gather information from intervening bids.
- **Endgame dynamics:** With fewer total dice, variance increases dramatically. Probabilities shift and bluffing becomes both more necessary and more punishable.
- **Palafico exploitation:** During Palafico rounds (no wilds), probability calculations change significantly since 1s no longer count double.
- **Exact call timing:** The Exact call is highest-EV when the bid quantity closely matches the expected count and many dice remain in play (reducing variance).

## Common Terminology

| Term | Definition |
|------|-----------|
| **Bid** | A claim about the minimum number of dice showing a specific face value |
| **Challenge / Dudo / Liar** | Calling a bluff -- forcing all dice to be revealed |
| **Exact / Spot On** | Claiming the bid is precisely correct (not more, not fewer) |
| **Wild** | 1s count as any face value (standard Perudo rule) |
| **Palafico** | Special round when a player has exactly 1 die remaining |
| **Cup** | The opaque container used to hide dice |
| **Face value** | The pip number showing on a die (1--6) |
| **Quantity** | The number of dice claimed in a bid |
| **Escalation** | Requirement that each bid be strictly higher than the previous |
| **Elimination** | Losing all dice and exiting the game |
| **Perudo** | The South American tournament version of Liar's Dice |
| **Round** | A sequence of bids ending in a challenge (or exact call) |

## State Space Analysis

### Information Sets
- Per player: knows own dice values (5 dice, 6 faces each = 6^5 = 7,776 outcomes).
- Hidden: all other players' dice.
- For N players with D dice each: total hidden state = 6^(D×(N-1)) possible opponent configurations.
- Information reduces as dice are lost (elimination) and potentially through bidding inference.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Dice outcomes (5 dice per player) | 6^5 = 7,776 per player |
| Total dice states (6 players, 5 dice) | 6^30 ≈ 2.2 × 10^23 |
| Bidding sequences per round | ~30 bids × variable length |
| Game tree (single round) | ~10^5-10^10 (depends on player count and dice) |
| Game tree (full game, 6 players) | ~10^30-10^50 (with elimination rounds) |
| Information sets (single round, 2-player, 5 dice) | ~10^8 |

### Action Space
- **Bid**: (quantity, face) pairs. Quantity: 1 to total dice on table. Face: 1-6. Constrained by previous bid.
- **Challenge**: binary.
- **Exact** (optional): binary.
- Effective choices per turn: ~5-30 (depending on current bid and remaining legal bids).

## Key Challenges for AI/Solver Approaches

### 1. Probability Estimation Under Hidden Information
The core challenge: estimating the probability that a bid is true given:
- Known own dice.
- Number of unknown dice.
- Binomial/multinomial distribution of face values (with or without wild 1s).
- Bayesian updating from opponent bidding behavior.

### 2. Bluffing Dynamics
Liar's Dice is a **pure bluffing game**:
- Players routinely bid higher than what their dice alone support, relying on opponents' dice.
- The decision to bluff depends on risk tolerance, opponent tendencies, and position in the bidding sequence.
- Equilibrium strategies involve mixed strategies (randomized bluffing frequencies).

### 3. Multi-Player Elimination Dynamics
With 3+ players, the game has elimination dynamics:
- Challenging is risky — the loser directly loses a die.
- Sometimes it's optimal to let an incorrect bid pass and let the next player challenge.
- ICM-like effects: when one player has 1 die, others may prefer to let that player be eliminated.

### 4. Wild 1s Complexity
The Perudo wild-1s rule roughly doubles the expected count of any face value:
- Without wilds: expected count of face X among N dice = N/6.
- With wilds: expected count = N/6 + N/6 = N/3 (since 1s also count).
- This shifts the probability calculations significantly and changes optimal bidding thresholds.

### 5. Palafico Subgame
The palafico rule (when a player has 1 die) creates a constrained subgame:
- No wild 1s.
- Only quantity increases allowed.
- This subgame has different equilibrium properties than the main game.

## Known Solver Results

### Solved Variants
- **2-player, 1 die each**: fully solved. The Nash equilibrium is known analytically.
  - Challenge threshold: bid is challenged when probability of being true drops below ~50%.
  - Bluffing frequency is well-defined.
- **2-player, N dice**: approximately solved via CFR for moderate N (up to ~5 dice each).
- **Multi-player**: not solved; heuristic approaches only.

### Academic Work
- **Lisý et al. (2015)**: game abstraction and CFR for Liar's Dice.
- **Lanctot et al.**: used Liar's Dice as a benchmark for MCCFR variants.
- Liar's Dice is a common testbed for imperfect-information game algorithms due to its clean structure and manageable size.

### CFR Results
- 2-player Liar's Dice with small die counts is a standard CFR benchmark.
- Near-exact Nash equilibria computed for 2-player, 1-die variants.
- Approximate equilibria for 2-player, 5-die variants with card (die) abstraction.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2009 | Lanctot et al., "Monte Carlo Sampling for Regret Minimization in Extensive-Form Games" | MCCFR benchmarked on Liar's Dice |
| 2015 | Lisý et al., "Online Monte Carlo Counterfactual Regret Minimization for Search in Imperfect Information Games" | Search-based approach for Liar's Dice |
| 2017 | Various | Liar's Dice as standard benchmark in IIG literature |

## Relevance to Myosu

### Solver Applicability
Liar's Dice is a **clean bluffing game** ideal for testing fundamental imperfect-information techniques:
- **CFR**: directly applicable and proven for 2-player variants. The game's compact state space makes CFR tractable.
- **MCCFR**: standard benchmark — external sampling and outcome sampling both tested extensively.
- **Neural networks**: can learn bidding policies, but the game may be simple enough for tabular methods.
- **Bayesian inference**: probability estimation from opponent bids is a natural application.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 5/5 | Standard CFR benchmark; tractable |
| Neural value network potential | 3/5 | Game may be simple enough for tabular |
| Abstraction necessity | 2/5 | Small state space for 2-player |
| Real-time solving value | 3/5 | Pre-computed strategies often sufficient |
| Transferability of techniques | 4/5 | Bluffing mechanics fundamental to many games |

### Myosu Subnet Considerations
- **Pedagogical value**: Liar's Dice is easy to understand, making it an excellent demonstration game for myosu's solver capabilities.
- **Clean bluffing benchmark**: isolates the bluffing/belief-tracking component of imperfect-information games from card/hand complexity.
- **Low compute requirements**: 2-player Liar's Dice can be approximately solved on modest hardware.
- **Game oracle**: dice roll verification, bid legality, and challenge resolution are trivially verifiable.
- **Scalability test**: performance from 2-player to 6-player tests how solver architectures handle increasing player counts.
- **Quick evaluation**: short games enable rapid strategy assessment.

### Recommended Approach for Myosu
1. CFR/MCCFR for 2-player variants (near-exact solutions feasible).
2. MCCFR with opponent modeling for multi-player variants.
3. Use as a validation game — any solver architecture should handle Liar's Dice competently.
4. Evaluate via exploitability (2-player) and win rate (multi-player).
5. Include as a "fast feedback" game in the solver evaluation pipeline — games complete quickly.
