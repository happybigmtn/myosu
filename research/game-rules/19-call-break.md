# Call Break (Call Bridge / Lakdi)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Call Break (Call Bridge / Lakdi / Ghochi) |
| Variants | Standard 5-deal, 10-deal, fractional scoring |
| Players | 4 (individual, no partnerships) |
| Information | Imperfect (hidden hands) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (fixed trick count per deal) |
| Solved Status | Unsolved; no published academic AI |

## Overview

Call Break is a trick-taking card game popular in Nepal, Bangladesh, and India. It features fixed trump (Spades are always trump), individual bidding, and a mandatory trump-beating rule that distinguishes it from similar games like Spades. Each player plays independently -- there are no partnerships. The game is known by various names including Call Bridge, Lakdi, and Ghochi.

- **Players:** 4 (each playing individually)
- **Duration:** 30--60 minutes (5 deals)
- **Objective:** Win at least as many tricks as you bid (called) in each deal, while maximizing your cumulative score across 5 deals.

## Equipment

- One standard 52-card deck (no jokers)
- Score sheet or scoring app

### Card Rankings

Within each suit, cards rank from high to low:

**A, K, Q, J, 10, 9, 8, 7, 6, 5, 4, 3, 2**

**Spades are permanent trump.** Any Spade beats any card of any other suit.

## Setup

1. Choose a first dealer (any method). The deal rotates to the **right** (counter-clockwise) after each hand.
2. The dealer shuffles and deals all 52 cards, one at a time, counter-clockwise.
3. Each player receives **13 cards**.

## Game Flow

### Phase 1: Calling (Bidding)

Starting with the player to the dealer's **right** and proceeding **counter-clockwise**, each player announces their call:

- A call is a number representing the tricks the player commits to winning.
- **Minimum call: 2.** Maximum call: 12 (theoretically 13, but impractical).
- Each player calls exactly once. There is no auction -- calls are independent declarations.
- Players do not need to call higher than the previous player.
- Calls are simultaneous commitments, not competitive bids.

#### Variant: Minimum Total Rule

Some groups require the four calls to total at least **10**. If the total is less than 10, the hand is redealt or the dealer must increase their call to meet the threshold.

### Phase 2: Play

#### Leading

The player to the dealer's **right** leads the first trick. On subsequent tricks, the winner of the previous trick leads.

#### Following Suit

Each player, counter-clockwise, plays one card to the trick. Strict rules govern what may be played:

1. **Must follow suit** if able. Play a card of the same suit that was led.
2. If **unable to follow suit**, the player **must play a Spade (trump)** that is high enough to beat any Spade already in the trick.
3. If unable to follow suit **and** unable to play a Spade that beats existing Spades in the trick, the player may play **any card** (including a lower Spade or any off-suit card).

**When following suit, there is no requirement to beat the current winning card.** A player following suit may play any card of the led suit, high or low.

**When trumping (playing a Spade because void in the led suit), the player MUST play a Spade that beats any Spade already played to that trick**, if possible.

#### Winning a Trick

- If no Spades were played, the highest card of the led suit wins.
- If one or more Spades were played, the **highest Spade** wins.
- The trick winner collects the 4 cards face-down and leads the next trick.

#### Spades May Be Led

Unlike Spades (the game), there is **no restriction on leading Spades** at any time. A player may lead a Spade on any trick, including the first.

#### 13 Tricks

Play continues until all 13 tricks are played, then scoring occurs.

## Scoring / Winning

### Standard Scoring

After each deal, compare each player's trick count to their call:

#### Successful Call (winning exactly the call or one more)

If a player wins **at least** the number of tricks they called, **and no more than (call + 1)**, the call value is **added** to their cumulative score.

| Call | Tricks Won | Result |
|------|-----------|--------|
| 4 | 4 | +4.0 (success) |
| 4 | 5 | +4.0 (success -- one overtrick allowed) |
| 4 | 6+ | -4.0 (failure -- too many overtricks) |
| 4 | 3 or fewer | -4.0 (failure -- underbid) |

> The exact overtrick tolerance varies by rule set. The most common rule is: a player must win **exactly their call or one more** to succeed. Winning 2+ more tricks than called counts as a failure. Some variants allow any number of overtricks.

#### Failed Call

If a player fails (too few or too many tricks), the call value is **subtracted** from their cumulative score.

#### Bonus Call (High Call: 8+)

Calls of **8 or higher** are considered bonus calls:

- **Success:** Score **13 points** (instead of the call value).
- **Failure:** Lose the call value (not 13).

Some variants award **16 points** for successful bonus calls.

### Fractional Overtrick Scoring (Variant)

In many digital implementations:

- Successful call: score = call value + 0.1 per overtrick.
- Example: Call 4, win 5 tricks = 4.1 points.
- This distinguishes between players who barely made their call and those who won extra tricks.

### Game Duration

A standard game consists of exactly **5 deals**. The player with the highest cumulative score after 5 deals wins.

Some groups play:
- **10 deals** for a longer game.
- **Indefinite** play until a target score is reached.

## Special Rules

### Must-Beat Trump Rule (Key Distinguishing Rule)

The obligation to play a higher Spade when trumping is what makes Call Break strategically distinct. It prevents "wasting" low trumps to steal tricks cheaply:

- If the led suit is not Spades and you are void in the led suit:
  - You MUST play a Spade if you have one.
  - If another player has already played a Spade to this trick, your Spade MUST beat that Spade (if you have a high enough Spade).
  - If your only Spades are lower than the existing Spade in the trick, you may play any card.

### Leading Spades Restriction (Variant)

Some groups prohibit leading Spades on the very first trick. After the first trick, Spades may be led freely.

### No Communication

Since there are no partnerships, players may not communicate about their hands. All information comes from observing play.

### Tie-Breaking

If two or more players tie for the highest cumulative score after all deals, the winner is determined by:
- Whoever had fewer failed calls.
- If still tied, whoever had the higher call in the final deal.
- If still tied, a tiebreaker deal is played.

## Key Strategic Concepts

- **Conservative calling:** Since both underbidding and overbidding result in penalties, accurate assessment of hand strength is critical. The must-beat trump rule means your mid-range Spades may not be controllable.
- **Trump management:** Spades are always trump, so counting Spades is essential. With 13 Spades in the deck distributed among 4 players, the average is 3--4 per player. Holding high Spades (A, K, Q) ensures trick-winning power.
- **Must-beat constraint:** The forced escalation when trumping means you cannot hide your high Spades. If you trump with the Ace of Spades early, you lose your biggest weapon. Planning when to void a suit (and thus be forced to trump) is a key decision.
- **Voiding suits:** Deliberately playing out a short suit creates opportunities to trump future tricks in that suit, but the must-beat rule means you must commit high Spades.
- **Counting tricks:** With 13 total tricks and 4 players, the average is 3.25 tricks per player. Most calls range from 2--5.
- **Reading opponents:** Since all play is face-up, tracking what cards opponents play (and what they don't play) reveals hand strength. A player who follows suit low on a trick they could win is likely saving high cards for later.
- **Endgame precision:** In the last 3--4 tricks, exact card counting determines whether you can hit your call precisely.

## Common Terminology

| Term | Definition |
|------|-----------|
| **Call** | The number of tricks a player commits to winning (their bid) |
| **Trump** | Spades -- always the trump suit |
| **Trick** | A round where each player plays one card; the highest card wins |
| **Follow suit** | Playing a card of the same suit that was led |
| **Void** | Having no cards in a particular suit |
| **Trump in / Ruff** | Playing a Spade when unable to follow the led suit |
| **Must-beat** | The rule requiring trumped Spades to beat existing Spades in the trick |
| **Bonus call** | A call of 8 or higher, scoring 13 points if successful |
| **Overtrick** | A trick won beyond the called amount |
| **Deal** | One complete hand of 13 tricks |
| **Cumulative score** | The running total across all deals in a game |
| **Lead** | Playing the first card to a trick |
| **Break** | The game's common short name (Call Break) |

## State Space Analysis

### Information Sets
- Initial deal: C(52,13) per player.
- Each player knows: own hand, all calls, cards played so far.
- Hidden: other 3 players' hands (39 cards initially).
- As tricks are played, information accumulates (cards revealed, void suits identified).

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible deals | ~5.36 x 10^28 (same as Spades/Bridge) |
| Calling sequences | ~12^4 = 20,736 (each player 2-12) |
| Play sequences | Up to 13! (constrained by suit-following and must-beat) |
| Game tree nodes | ~10^20-10^30 |
| Information sets | ~10^15-10^20 |

### Action Space
- **Calling**: integer 2-12.
- **Play**: choose from legal cards (constrained by suit-following and must-beat trump rule).
- Branching factor: ~3-13 per trick (depends on hand and trump constraints).

## Key Challenges for AI/Solver Approaches

### 1. Must-Beat Trump Rule
The mandatory escalation when trumping distinguishes Call Break from Spades:
- Players cannot strategically play low trumps to cheaply steal tricks.
- Holding high spades becomes both an asset (guaranteed trick winners) and a liability (forced to commit them early).
- Planning when to void a suit must account for the forced-escalation constraint.

### 2. Individual Play (No Partnerships)
Unlike Spades, each player competes independently:
- No partner to coordinate with or rely on.
- All 3 opponents are adversaries.
- The 4-player individual structure creates complex non-cooperative dynamics.

### 3. Strict Overtrick Penalty
In the standard variant, winning more than (call + 1) tricks results in failure:
- Players must manage their hand to win exactly their call or one more.
- This creates "trick avoidance" situations mid-hand, unlike Spades where overtricks are only mildly penalized.
- Requires precise endgame planning to avoid accidental overtricks.

### 4. Five-Deal Game Structure
The 5-deal format creates multi-hand strategic considerations:
- Cumulative score tracking affects risk tolerance.
- Players behind may need to make aggressive calls in later deals.
- Bonus call (8+) for 13 points introduces high-variance strategic options.

## Known Solver Results

### Academic Work
- No published academic research specific to Call Break AI.
- The game's similarity to Spades means trick-taking game AI research is transferable.
- The must-beat trump rule and individual (non-partnership) play add unique complexity.

### AI Implementations
- South Asian mobile gaming apps include basic AI opponents, typically rule-based.
- No known competitive AI benchmark or tournament series.
- Monte Carlo simulation with trick-taking heuristics would be a natural approach.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2001 | Ginsberg (GIB) | MC + double-dummy for trick-taking games (applicable) |
| 2020 | Various card game AI surveys | General trick-taking AI techniques applicable |

## Relevance to Myosu

### Solver Applicability
Call Break tests **individual trick-taking** with unique trump constraints:
- **CFR**: limited by 4-player non-cooperative structure. Better for simplified 2-player abstractions.
- **Monte Carlo simulation**: proven approach for trick-taking games, directly applicable.
- **Neural networks**: can learn calling functions and play policies from self-play.
- **Must-beat modeling**: the forced trump escalation requires specialized game-tree representation.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 2/5 | 4-player individual, non-cooperative |
| Neural value network potential | 4/5 | Trick estimation and play quality learnable |
| Abstraction necessity | 2/5 | Moderate state space |
| Real-time solving value | 4/5 | MC simulation effective |
| Transferability of techniques | 4/5 | Shares with Spades, Bridge, and other trick-taking games |

### Myosu Subnet Considerations
- **South Asian market**: Call Break is enormously popular in Nepal, Bangladesh, and India. No competitive AI solver exists.
- **Unique must-beat rule**: provides a distinct strategic challenge not found in Spades or Bridge.
- **Individual play**: tests non-cooperative multi-agent dynamics in a trick-taking context.
- **Shared trick-taking infrastructure**: engine can be shared with Spades (14) and Bridge (10).
- **Game oracle**: trick resolution, must-beat validation, and scoring are straightforward to implement.
- **Low compute requirements**: moderate state space means solver nodes require modest hardware.

### Recommended Approach for Myosu
1. MC simulation with perfect-information trick solver for play decisions.
2. Neural networks for call prediction (input: hand features; output: trick estimate with must-beat awareness).
3. Evaluate via call accuracy and score differential over 5-deal sessions.
4. Share trick-taking infrastructure with Spades and Bridge to reduce implementation effort.
5. Use as the primary South Asian market entry game.
