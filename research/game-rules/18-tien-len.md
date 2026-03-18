# Tien Len (Thirteen / Vietnamese Cards)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Tien Len (Tiến Lên, "Advance"), also known as Thirteen |
| Variants | Tien Len Mien Nam (Southern), Tien Len Mien Bac (Northern), Killer/Instant Win variants |
| Players | 4 (standard), 2-6 (variants) |
| Information | Imperfect (hidden hands) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (scoring system) |
| Solved Status | Unsolved; no published academic AI |

## Overview

Tien Len (Tiến Lên, meaning "Go Forward" or "Advance") is a climbing/shedding card game from Vietnam and the most popular card game in the Vietnamese diaspora worldwide. It is related to Big Two but has distinct Vietnamese rules, including a unique suit hierarchy, special "bomb" mechanics for defeating 2s, and instant win conditions. The game is also known as Thirteen (referring to the 13 cards dealt per player).

- **Players:** 4 (standard); playable with 2--3
- **Duration:** 10--20 minutes per round
- **Objective:** Be the first player to play all 13 cards from your hand.

## Equipment

- One standard 52-card deck (no jokers)

### Card Rankings

Cards rank from highest to lowest:

**2 > A > K > Q > J > 10 > 9 > 8 > 7 > 6 > 5 > 4 > 3**

The **2** is the highest rank; the **3** is the lowest.

### Suit Rankings

Within the same card rank, suits break ties. From highest to lowest:

**Hearts (♥) > Diamonds (♦) > Clubs (♣) > Spades (♠)**

Therefore the **2♥** is the single strongest card in the game, and the **3♠** is the weakest.

> Note: The Vietnamese suit order differs from Big Two and Pusoy Dos. Hearts are highest; Spades are lowest.

## Setup

1. Shuffle the deck and deal all 52 cards evenly to 4 players, **13 cards each**.
2. Players examine their hands privately.
3. In the very first game, the player holding the **3♠** (lowest card) plays first. In subsequent games, the loser of the previous game plays first (or the winner -- varies by house rule).

## Game Flow

### Round Structure

A **round** (also called a trick or bout) is a sequence of plays starting from a lead and ending when all other players pass. The game consists of multiple rounds until one player empties their hand.

### Starting the Game

The first player leads by playing any single card or legal combination. In the very first hand of a session, the opening play must **include the 3♠**.

### Turn Sequence

Play proceeds **clockwise**. On each turn, a player must either:

1. **Play** a combination of the same type and same card count that beats the current play, OR
2. **Pass** (play nothing).

### Beating Rules (Standard)

A combination is beaten by a higher combination of the **same type and same number of cards**:

- Singles beat singles (by rank, then suit).
- Pairs beat pairs (by the higher card in the pair -- rank first, then suit).
- Triples beat triples (by rank).
- Sequences beat sequences of the **same length** (by the highest card -- rank first, then suit).
- Double sequences beat double sequences of the **same length** (by the highest pair).

### Passing

- A player may pass even if they have a valid play.
- **Once a player passes, they are locked out of the current round** and cannot play again until a new round begins.
- **Exception:** A player who passed may still play a "bomb" to beat a 2 (see Special Bomb Rules below).

### Round End

When all players except one have passed, the remaining player wins the round and leads the next round with any combination.

### Game End

The first player to play all their cards wins and drops out. Play continues among remaining players. The **last player** with cards in hand is the loser.

In the gambling variant, the loser pays a fixed stake to each other player.

## Actions

### Valid Combinations

#### Single (1 card)

Any individual card. Beaten by a higher-ranked single (by rank, then by suit).

#### Pair (2 cards)

Two cards of the same rank. Beaten by a higher pair. The higher card in each pair (by suit) determines which pair wins when ranks are equal.

#### Triple (3 cards)

Three cards of the same rank. Beaten by a triple of higher rank.

#### Four of a Kind (4 cards)

All four cards of the same rank. Beaten by a higher four of a kind. Also has special bomb properties (see below).

#### Sequence (3+ cards)

Three or more consecutive-ranked cards (suits may be mixed).

- Minimum length: **3 cards** (e.g., 3-4-5).
- **2s cannot appear in sequences.** The highest possible sequence top card is Ace.
- Valid: 3-4-5, 7-8-9-10-J, 10-J-Q-K-A.
- Invalid: Q-K-A-2, K-A-2-3 (no wrapping through 2).
- Sequences must be beaten by sequences of the **same length** with a higher top card.

#### Double Sequence (3+ consecutive pairs)

Three or more consecutive pairs. Example: 3-3-4-4-5-5.

- Minimum length: **3 pairs** (6 cards).
- 2s cannot appear.
- Beaten by a double sequence of the same length with higher pairs.
- Also has special bomb properties (see below).

## Special Bomb Rules (Two-Killers)

Certain combinations can beat 2s even though they are a different combination type. These are called **bombs** or **chops**:

| Bomb Combination | Beats |
|-----------------|-------|
| **Four of a Kind** | Any single 2 |
| **3 consecutive pairs** (double sequence of 3 pairs) | Any single 2 |
| **4 consecutive pairs** (double sequence of 4 pairs) | Any pair of 2s |
| **5 consecutive pairs** (double sequence of 5 pairs) | Any triple of 2s (three 2s played as a triple) |

### Bomb Rules in Detail

1. A bomb can **only be played in response to the specific 2-combination it beats**. A four-of-a-kind cannot beat a random single card -- it only activates its bomb power against a single 2.
2. After a bomb is played, it establishes a **new combination type** for the round. Subsequent players must beat the bomb with a higher bomb of the same type or pass.
3. A player who has previously passed in the current round **may still play a bomb** to beat a 2. This is the only exception to the "locked out after passing" rule.
4. Four of a kind played as a bomb against a single 2 can be beaten by a higher four of a kind.
5. A 3-pair double sequence bomb can be beaten by a higher 3-pair double sequence.

### 2s as Combinations

- A single 2 can be led or played to beat any other single card.
- A pair of 2s can be played to beat any other pair.
- Three 2s can be played as a triple to beat any other triple.
- All four 2s played together form a four of a kind (the highest possible).

## Instant Win Conditions

Before any cards are played, if a player holds one of the following combinations in their dealt hand, they may reveal it for an **automatic win** (no play required):

| Instant Win | Description |
|------------|-------------|
| **Four 2s** | All four 2s in one hand |
| **Six Pairs** | Six pairs of any rank (12 of 13 cards are paired) |
| **Three Consecutive Triples** | Three triples in sequence (e.g., 4-4-4, 5-5-5, 6-6-6) |
| **Dragon (12-card sequence)** | A complete 3-through-Ace straight: 3-4-5-6-7-8-9-10-J-Q-K-A |

If multiple players hold instant-win hands, priority from highest to lowest:
**Four 2s > Dragon > Three Consecutive Triples > Six Pairs**

Instant wins must be from the **dealt hand** -- they cannot be arranged through trading or play.

## Scoring / Winning

### Standard (No Gambling)

- First player out wins.
- Last player remaining loses.
- No points are tracked; the game is played for bragging rights or drinks.

### Stake-Based Scoring

The loser pays a fixed stake to each other player. Some variants scale the penalty:

- Last player with 13 cards (never played anything): double or triple penalty.
- Holding the 2♥ (highest card) when losing: additional penalty.

### Tournament Scoring

Assign points by finish order: 1st = 3 points, 2nd = 2, 3rd = 1, 4th (loser) = 0.

## Special Rules

### Smashing / 2-Killing Priority

When a 2 is played, the next player in turn order has first right to bomb it. If they pass, the opportunity moves clockwise. However, any player (even one who previously passed) may bomb a 2 -- this is the one exception to the pass-lockout rule.

### Suit as Tiebreaker

In all combinations, when ranks are equal, the **highest suit** among the cards determines the winner. For pairs, the pair containing the highest-suited card wins. For sequences, the highest card's suit determines it.

### Three-Player Variant

- Deal 17 cards each. The remaining card is set aside face-down (unused).
- The player with the 3♠ still goes first.
- Some variants give 17 cards to each with the last card revealed and given to the 3♠ holder (18 cards for one player).

### Two-Player Variant

- Deal 13 cards each. Remaining 26 cards are unused.
- Some variants deal more cards per player.

### Southern Vietnamese vs. Northern Vietnamese Rules

Minor regional variations exist in:
- Whether passing permanently locks you out (South: yes; North: some variants allow re-entry).
- Whether 4-of-a-kind beats any combination (some Northern variants) vs. only beating single 2s.
- Whether a straight can include a 2 (universally: no).

## Key Strategic Concepts

- **2-management:** 2s are both powerful (highest cards) and vulnerable (can be bombed). Timing when to play 2s -- using them to regain control without giving opponents a bomb opportunity -- is the central strategic tension.
- **Bomb awareness:** Track which four-of-a-kinds and long double sequences remain. Holding a bomb gives you insurance against opponents' 2s and lets you regain control at critical moments.
- **Card decomposition:** Before the first play, plan how to partition your 13 cards into playable combinations. Avoid orphan cards that cannot be shed.
- **Control chains:** Maintaining lead (control) by playing combinations that are hard to beat ensures you can shed cards on your own terms.
- **Sequence building:** Long sequences (5+ cards) clear many cards at once and are difficult to beat since the opponent needs an equal-length sequence.
- **Endgame calculation:** When few cards remain per player, counting cards to determine what opponents hold becomes critical.
- **Instant win recognition:** Always check your dealt hand for instant-win conditions before play begins.

## Common Terminology

| Term | Definition |
|------|-----------|
| **Tien Len (Tiến Lên)** | "Go Forward" -- the Vietnamese name for the game |
| **Thirteen** | English name (13 cards per player) |
| **Bomb / Chop** | A combination that can beat a 2-based play |
| **Dragon** | A 12-card sequence from 3 through Ace (instant win) |
| **Control** | Leading a new round with any combination |
| **Round / Trick** | A sequence of plays ending when all others pass |
| **Pass** | Declining to play in the current round |
| **Locked out** | Having passed and being unable to play until the next round |
| **Single** | A play of one card |
| **Pair / Double** | Two cards of the same rank |
| **Triple / Trips** | Three cards of the same rank |
| **Sequence / Run** | Three or more consecutive-ranked cards |
| **Double Sequence** | Three or more consecutive pairs |
| **Smash / Kill** | Using a bomb to beat a 2 |
| **Instant win** | A dealt hand that wins without any play |
| **Shed** | To play cards from your hand |

## State Space Analysis

### Information Sets
- Initial deal: C(52,13) per player ≈ 6.35 × 10^11.
- Players see their own hand only at start.
- Cards played become public.
- Hidden: opponents' remaining cards.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible deals | ~5.36 × 10^28 (same as other 4×13 deals) |
| Legal combinations per turn | ~5-25 (depends on hand and required type) |
| Average game length | ~20-40 actions |
| Game tree nodes | ~10^25-10^40 |
| Information sets | ~10^20-10^30 |
| Branching factor | ~5-25 per turn |

### Action Space
- **Leading**: play any legal single, pair, triple, sequence, or pair sequence.
- **Following**: play same type with higher rank, or pass (with chop exceptions for 2s).
- Sequences of varying lengths provide additional action space.
- Pair sequences add a combination type not found in Big Two.

## Key Challenges for AI/Solver Approaches

### 1. Chop Mechanics
The ability to beat 2s with pair sequences and quads creates unique dynamics:
- Building pair sequences is strategically valuable beyond their raw card strength.
- Holding back pair sequences to beat opponents' 2s is a key tactical consideration.
- The chop mechanic reduces the dominance of 2s compared to Big Two.

### 2. Variable-Length Sequences
Unlike Big Two (which has fixed 5-card combinations), Tien Len allows sequences of 3+ cards:
- Longer sequences are harder to beat (fewer hands with long sequences in the same rank range).
- Deciding between playing a long sequence (harder to beat, uses more cards) vs shorter sequences (more flexible) is a strategic choice.
- Pair sequences of varying lengths add another dimension.

### 3. Four-Player Dynamics
Same challenges as other 4-player shedding games:
- 39 hidden cards at game start.
- Player elimination order affects scoring.
- Positional play (who leads, who follows) creates tactical considerations.

### 4. 2s as Premium Cards
The four 2s are the highest singles and among the highest pairs:
- Holding 2s guarantees future control (can win any single or pair play).
- But 2s can be "chopped" by pair sequences — so they're not invincible.
- Deciding when to play 2s vs save them is a key strategic question.
- Remaining 2s incur extra scoring penalties.

### 5. Instant Win Detection
The solver must recognize instant win hands at the start of each deal:
- Dragon (13-card straight) is trivially detectable.
- Six pairs requires checking for 6 pairs in 13 cards.
- Four threes: simple count check.
- These are rare but must be handled correctly.

## Known Solver Results

### Academic Work
- No published academic papers specifically on Tien Len AI.
- The game's similarity to Big Two and DDZ means that research on those games is transferable.
- The chop mechanic and variable-length sequences add complexity not present in related games.

### AI Implementations
- Vietnamese card game apps include basic AI opponents, typically rule-based.
- No known competitive AI benchmark.
- The game's similarity to other shedding games suggests DouZero-style approaches would be effective.

### Transferable Research
- **DouZero (DDZ)**: deep RL for shedding games, directly applicable architecture.
- **Big Two research**: combination-based play strategies transfer.
- **General shedding game theory**: card control and hand decomposition principles apply.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2021 | Zha et al. (DouZero) | Shedding game RL framework, directly transferable |
| 2020 | General shedding game AI | Applicable techniques |

## Relevance to Myosu

### Solver Applicability
Tien Len tests **shedding game variants** with unique mechanics:
- **CFR**: same limitations as DDZ and Big Two for 4-player play.
- **Deep RL**: the natural approach, adapting DouZero-style architecture.
- **Chop-aware planning**: the solver must reason about pair sequence / quad potential for beating 2s.
- **Sequence planning**: variable-length sequence optimization adds a planning dimension.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 2/5 | 4-player, placement scoring |
| Neural value network potential | 4/5 | Same as DDZ/Big Two |
| Abstraction necessity | 3/5 | Action encoding for variable-length combinations |
| Real-time solving value | 4/5 | Endgame solving valuable |
| Transferability of techniques | 5/5 | Direct transfer to/from DDZ and Pusoy Dos |

### Myosu Subnet Considerations
- **Vietnamese market**: Tien Len is the national card game of Vietnam, played by millions. No competitive AI exists.
- **Shared shedding framework**: combination validation, shedding mechanics, and RL training pipeline shared with DDZ (16) and Pusoy Dos (17).
- **Chop rule implementation**: the oracle must correctly handle pair sequence and quad chops against 2s.
- **Instant win verification**: must detect and verify instant win hands at deal time.
- **Regional rule variations**: Southern vs Northern Vietnamese rules differ in details. The subnet must specify a canonical version.
- **Sequence validation**: variable-length sequences and pair sequences require robust validation logic.

### Recommended Approach for Myosu
1. Adapt DouZero architecture for Tien Len's combination types.
2. Specific modeling of chop dynamics (pair sequence / quad utility for beating 2s).
3. Shared infrastructure with DDZ and Pusoy Dos.
4. Evaluate via win rate and placement distribution.
5. Use as the primary Vietnamese market game, paired with other shedding games for infrastructure efficiency.
