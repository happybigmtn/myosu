# Pusoy Dos (Big Two / Filipino Poker)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Pusoy Dos (Filipino), Big Two (大老二), Dai Di (大帝) |
| Variants | Filipino rules, Taiwanese rules, Hong Kong Big Two, Choh Dai Di |
| Players | 4 (standard), 2-3 (variants) |
| Information | Imperfect (hidden hands) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (scoring system) |
| Solved Status | Unsolved; limited academic study |

## Overview

Pusoy Dos is a climbing/shedding card game originating from the Philippines, closely related to the Chinese game Big Two (大老二). The name literally means "Poker Two" in Filipino, referring to the 2 being the highest-ranked card. It is one of the most popular card games in the Philippines and Southeast Asia.

- **Players:** 4 (standard); playable with 2--3
- **Duration:** 10--20 minutes per round
- **Objective:** Be the first player to discard all 13 cards from your hand.

## Equipment

- One standard 52-card deck (no jokers)

### Card Rankings

Cards rank from highest to lowest:

**2 > A > K > Q > J > 10 > 9 > 8 > 7 > 6 > 5 > 4 > 3**

The 2 is the highest card; the 3 is the lowest.

### Suit Rankings

Within the same card rank, suits break ties. From highest to lowest:

**Diamonds (♦) > Hearts (♥) > Spades (♠) > Clubs (♣)**

Therefore the **2♦** is the single strongest card in the game, and the **3♣** is the weakest.

> Note: The suit order in Pusoy Dos differs from standard Big Two (which typically uses Spades > Hearts > Clubs > Diamonds). The Filipino suit order places Diamonds highest.

## Setup

1. Shuffle the deck and deal all 52 cards evenly to the 4 players, **13 cards each**.
2. Players look at their hands but keep them hidden from others.
3. The player holding the **3♣** (lowest card in the game) goes first in the first round. In subsequent rounds, the winner of the previous round leads.

## Game Flow

### Round Structure

A **round** (also called a trick or bout) is a sequence of plays that ends when all players but one have passed. The game consists of multiple rounds.

### Starting the Game

The player with the **3♣** must lead the first round. Their opening play must **include the 3♣** -- either as a single card or as part of a valid combination containing it.

### Turn Sequence

Play proceeds **clockwise**. On each turn, a player must either:

1. **Play** a valid combination that beats the current one on the table, OR
2. **Pass** (play nothing).

### Beating Rules

- A combination can **only be beaten by a combination of the same type and same number of cards**.
- Singles beat singles, pairs beat pairs, triples beat triples, five-card hands beat five-card hands.
- **Exception:** Among five-card hands, a higher-ranked hand type always beats a lower-ranked hand type regardless of card values (e.g., any flush beats any straight).

### Passing

- A player may pass even if they have a valid play available.
- **Once a player passes, they are out of the current round.** They cannot play again until a new round begins.

### Round End

When all players except one have passed consecutively, the remaining player (who made the last unchallenged play) **wins the round** and **leads the next round** with any card or combination of their choice.

### Game End

The game ends **immediately** when any player plays their last card(s). That player wins. Play does not continue to determine second, third, and fourth place (though some scoring variants do -- see Scoring).

## Actions

### Valid Combinations

#### Single (1 card)

Any individual card. Beaten by a higher-ranked single (by rank first, then suit).

Example: 5♥ beats 5♠ (same rank, higher suit). K♣ beats Q♦ (higher rank).

#### Pair (2 cards)

Two cards of the same rank. A pair is beaten by a pair of higher rank, or by a pair of the same rank containing a higher suit.

The **higher suit in the pair** determines the pair's suit ranking. Example: 7♦-7♣ beats 7♥-7♠ because ♦ > ♥.

#### Three of a Kind (3 cards)

Three cards of the same rank. Beaten by a higher-ranked triple. Suit is irrelevant since three of a kind automatically contains the determining suit.

#### Five-Card Hands (5 cards)

Five-card hands are ranked by **hand type** first, then by internal values. The hand types, from lowest to highest:

| Rank | Hand Type | Description | Tiebreaker |
|------|-----------|-------------|------------|
| 1 | **Straight** | 5 consecutive ranks, mixed suits | Highest card rank, then highest card suit |
| 2 | **Flush** | 5 cards of the same suit, non-sequential | Suit of the flush, then highest card |
| 3 | **Full House** | Triple + pair | Rank of the triple |
| 4 | **Four of a Kind** | Quad + 1 kicker | Rank of the quad |
| 5 | **Straight Flush** | 5 consecutive ranks, same suit | Highest card rank, then suit |

Any flush beats any straight. Any full house beats any flush. And so on.

##### Straight Details

- Valid straights: A-2-3-4-5 (highest, since 2 is high), 10-J-Q-K-A, 9-10-J-Q-K, etc.
- The lowest straight is 3-4-5-6-7.
- A straight wraps: A-2-3-4-5 is valid and is the highest straight (contains a 2).
- Comparison: the straight with the highest top card wins. If tied, the highest suit among the top cards wins.

> Note: Some rule sets do not allow wrapping (A-2-3-4-5) or rank it as the lowest straight. Clarify before play.

##### Flush Details

- All 5 cards must be the same suit.
- Between two flushes, the one with the **higher suit** wins (Diamonds > Hearts > Spades > Clubs), regardless of card ranks within the flush.
- If same suit (impossible in standard 4-player since there are only 13 cards per suit), compare highest card, then second highest, etc.

##### Four of a Kind Details

- Must be played as exactly 5 cards: the four matching cards plus any one kicker card.
- Cannot play four of a kind without the fifth card.

## Scoring / Winning

### Basic Scoring

The first player to empty their hand wins the round. Multiple scoring systems exist:

#### Placement Scoring

| Finish | Points |
|--------|--------|
| 1st out | 3 points (or 10) |
| 2nd out | 2 points (or 5) |
| 3rd out | 1 point (or 3) |
| Last (4th) | 0 points |

Play continues after the first player goes out until only one player remains.

#### Penalty Scoring (Big Two Standard)

Losers pay the winner based on remaining cards:

| Cards Remaining | Penalty per Card |
|-----------------|-----------------|
| 1--9 cards | 1 point per card |
| 10--12 cards | 2 points per card |
| 13 cards (all) | 3 points per card (39 total) |

#### Pusoy Dos Special Scoring

Winners score based on how strong their winning play was:

| Winning Play Contains | Points |
|-----------------------|--------|
| No 2s | 1 |
| One 2 | 2 |
| Two 2s | 4 |
| Three 2s | 8 |
| Four 2s | 16 |

## Special Rules

### Control / Free Lead

When a player wins a round (all others pass), they have "control" and may lead with **any** combination type. This is a key strategic moment -- leading with a type where you are strong forces opponents to match that type or pass.

### 2♦ Ending Rule (Optional)

Some Filipino rule sets include: if the winning (final) card played is the 2♦ (highest card in the game), the points are doubled.

### Passing Does Not Prevent Future Rounds

Passing only locks you out of the **current round**. When a new round begins (new lead), all players are eligible to play again.

### Three-Player Variant

- Deal 17 cards to each player. One card remains face-down and is not used, OR the player holding the 3♣ receives the extra card (18 cards).
- The 3♣ holder still leads first.

### Two-Player Variant

- Various dealing methods: 13 cards each (26 cards unused), or more cards per player depending on group preference.

## Key Strategic Concepts

- **Control is king.** Winning a round gives you a free lead with any combination type. Structuring your hand to maintain control is the primary strategic goal.
- **2s are endgame weapons.** 2s are the highest singles and dominate the late game. Holding a 2 ensures you can always regain control with a single.
- **Combination planning.** Before playing, plan how to decompose your 13 cards into combinations that can all be played out. Leaving orphan cards (singles that cannot be played out) is a common mistake.
- **Suit awareness.** In Pusoy Dos, the suit hierarchy (♦ > ♥ > ♠ > ♣) frequently determines winners when ranks tie. The 2♦ is effectively a guaranteed round winner as a single.
- **Five-card hand formation.** Grouping cards into strong five-card hands (especially straight flushes or four-of-a-kind) can clear 5 cards at once and overwhelm opponents.
- **Counting cards.** Tracking which high cards and 2s have been played tells you what threats remain.
- **Pass timing.** Sometimes passing early in a round (even with a playable card) preserves cards for a better strategic moment.

## Common Terminology

| Term | Definition |
|------|-----------|
| **Pusoy Dos** | Filipino name for Big Two; literally "Poker Two" |
| **Control** | Having won the last round and being free to lead any combination |
| **Lead** | Starting a new round with any legal combination |
| **Pass** | Declining to play in the current round |
| **Round / Bout / Trick** | A sequence of plays ending when all but one player pass |
| **Single** | A play of one card |
| **Pair** | Two cards of the same rank |
| **Triple / Trips** | Three cards of the same rank |
| **Straight** | Five consecutive-ranked cards |
| **Flush** | Five cards of the same suit |
| **Full House** | Three of a kind plus a pair |
| **Quad / Four of a Kind** | Four cards of the same rank plus a kicker |
| **Straight Flush** | Five consecutive cards of the same suit |
| **Kicker** | The extra card played with a four-of-a-kind to make five |
| **Shed** | To play cards from your hand (the goal is to shed all cards) |
| **Locked out** | Having passed in the current round and unable to play until a new round begins |

## State Space Analysis

### Information Sets
- Initial deal: C(52,13) per player.
- Each player sees only their own hand initially.
- As play progresses: cards played are public.
- Hidden: other players' remaining cards.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible deals | C(52,13) × C(39,13) × C(26,13) ≈ 5.36 × 10^28 |
| Legal combinations per turn | ~5-30 (depends on hand and required type) |
| Average game length | ~20-40 actions |
| Game tree nodes | ~10^25-10^40 |
| Information sets | ~10^20-10^30 |
| Branching factor | ~5-30 per turn |

### Action Space
- **Leading**: play any single, pair, triple, or five-card combination.
- **Following**: play same type with higher rank, or pass.
- The five-card poker combinations create a rich action space for 5-card plays.
- Must decide between using cards in combinations vs saving for later.

## Key Challenges for AI/Solver Approaches

### 1. Hand Decomposition
A 13-card hand can be decomposed into combinations in many ways:
- Which cards to group as pairs, triples, five-card hands, or singles.
- Different decompositions lead to different game plans.
- Optimal decomposition depends on the evolving game state.

### 2. Card Control (Initiative)
Maintaining "control" (the lead) is critical:
- The player who leads chooses the combination type, giving them initiative.
- Passing cedes control.
- Big twos (2s) are the highest singles — holding them provides guaranteed future control.
- Strategic timing of when to play high cards vs save them.

### 3. Four-Player Hidden Information
With 4 players and 13 cards each, 39 cards are hidden:
- Card counting becomes important as cards are played.
- Reading opponents' passes (they can't beat the current play) provides information.
- Inferring opponents' hand strength from their play patterns.

### 4. End-Game Planning
The late game (when players have few cards) requires precise planning:
- Can I play out my remaining cards before opponents?
- Which combination types do I need to lead?
- Opponents' remaining cards constrain their options.
- Some endgames are deterministic (can be solved by exhaustive search).

### 5. Penalty Avoidance
The scoring system penalizes remaining cards, with multipliers for 10+ cards:
- Even if winning is unlikely, minimizing remaining cards is important.
- Sometimes it's better to play suboptimal combinations early to reduce card count.
- 2s remaining double penalties — playing or holding 2s is a key decision.

## Known Solver Results

### Academic Work
- Limited published research specific to Big Two.
- Some overlap with general shedding game AI research.
- Endgame solving (perfect information, few remaining cards) is tractable.

### AI Approaches
- **Rule-based heuristics**: most implementations use:
  - Play smallest winning combination.
  - Save 2s for control.
  - Decompose hand to minimize expected remaining cards.
- **Monte Carlo simulation**: sample opponents' hands, simulate play.
- **Endgame solver**: when enough information is known, solve the remaining game exactly.
- **No published superhuman AI**: Big Two has not received significant research attention.

### Connection to Dou Di Zhu
- Shares the shedding mechanic and combination-based play.
- DDZ research (DouZero, DeltaDou) is transferable.
- Key differences: symmetric player count (no landlord/peasant), different combination types, different card rankings.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2021 | Zha et al. (DouZero) | Shedding game RL techniques transferable |
| 2020 | Various game AI surveys | Big Two included in shedding game family |

## Relevance to Myosu

### Solver Applicability
Pusoy Dos tests **symmetric multiplayer shedding** mechanics:
- **CFR**: limited applicability for 4-player (non-zero-sum structure due to placement scoring). Better for 2-player variants.
- **Deep RL**: the natural approach, following DDZ research. Policy networks for action selection.
- **Endgame solving**: tractable when few cards remain — hybrid approach of RL policy + endgame solver.
- **Hand decomposition**: requires specialized algorithms for evaluating different card groupings.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 2/5 | 4-player; placement scoring |
| Neural value network potential | 4/5 | Learnable from self-play |
| Abstraction necessity | 3/5 | Action space encoding needed |
| Real-time solving value | 4/5 | Endgame solver very valuable |
| Transferability of techniques | 5/5 | Direct transfer to/from DDZ and Tien Len |

### Myosu Subnet Considerations
- **Southeast Asian popularity**: Pusoy Dos is one of the most popular card games in the Philippines and wider Southeast Asia.
- **Shared infrastructure with DDZ and Tien Len**: shedding game framework can be shared across games 16, 17, and 18.
- **Combination validation**: the five-card poker-hand hierarchy adds complexity to the game oracle.
- **Regional rule variations**: the subnet must specify exact rules (which combination types allowed, suit ranking, first-play rules).
- **Endgame verification**: perfect-information endgames can be verified exactly, providing a ground-truth evaluation.
- **Card ranking uniqueness**: 2 as highest card (not ace) requires specific implementation.

### Recommended Approach for Myosu
1. Deep RL (following DouZero methodology) adapted for Pusoy Dos's symmetric structure.
2. Endgame solver for positions with few remaining cards.
3. Shared shedding game infrastructure with DDZ (16) and Tien Len (18).
4. Evaluate via win rate and average remaining cards in losses.
5. Use as the primary Southeast Asian market game.
