# Open Face Chinese Poker (OFC)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Open-Face Chinese Poker (OFC) |
| Variants | Pineapple OFC (most popular), Standard OFC, Turbo OFC, Progressive Pineapple |
| Players | 2-3 (commonly 2-3; up to 4 in some variants) |
| Information | Imperfect (unknown draw cards, opponent hands partially visible) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (scoring system) |
| Solved Status | Unsolved; limited academic study |

## Overview

Open Face Chinese Poker (OFC) is a points-based poker variant where players arrange 13 cards face-up across three rows to form poker hands. Unlike traditional poker, there is no betting -- scoring is entirely based on hand strength, royalties (bonuses), and scooping. OFC evolved from standard Chinese Poker in Finland around 2012 and rapidly gained popularity in the poker community.

- **Players:** 2--3 (standard), up to 4 in some variants
- **Duration:** 5--15 minutes per hand
- **Objective:** Arrange 13 cards into three rows (front, middle, back) to maximize points against opponents. Rows must be in ascending strength from front to back.

## Equipment

- One standard 52-card deck (no jokers)
- Scoring chips, tokens, or a point-tracking system
- A dealer button to track position

## Setup

1. Designate a dealer (button). The button rotates clockwise after each hand.
2. Agree on point value (e.g., $1/point) and any variant rules before play.
3. The dealer shuffles the deck.

## Game Flow

### Standard OFC

#### Initial Deal

The dealer deals **5 cards** face-down to each player, one at a time, starting to the left of the button.

#### Initial Placement

Starting with the player to the left of the button and proceeding clockwise, each player arranges their 5 cards face-up into their three rows. Once placed, cards cannot be moved.

#### Drawing Phase

After all players have set their initial 5 cards, play continues clockwise. On each subsequent turn, a player draws **1 card** from the deck, reveals it, and places it face-up in one of their three rows.

This continues for 8 more rounds (8 cards drawn per player), until each player has exactly 13 cards arranged as:

- **Front (Top):** 3 cards
- **Middle:** 5 cards
- **Back (Bottom):** 5 cards

### Pineapple OFC (Most Common Variant)

#### Initial Deal

Same as standard: 5 cards dealt face-down to each player.

#### Drawing Phase

Instead of drawing 1 card, each player receives **3 cards** per turn, places **2** into their rows, and **discards 1** face-down. This continues for 4 rounds (receiving 12 additional cards, placing 8, discarding 4), completing the 13-card layout.

## Actions

On each turn, a player must:

1. **Place** the dealt card(s) into one of their three rows (respecting row capacity: front holds 3, middle holds 5, back holds 5).
2. In Pineapple, **discard** one of the three received cards face-down.

There is no option to pass, trade, or rearrange previously placed cards.

## Row Strength Requirement

The three rows must satisfy a strict ordering:

```
Back hand >= Middle hand >= Front hand
```

Where `>=` means "at least as strong as" using standard poker hand rankings. The front row uses three-card hand rankings (only high card, pair, three-of-a-kind -- no straights or flushes).

If this ordering is violated, the hand is **fouled** (see below).

## Scoring / Winning

### Head-to-Head Row Comparison

Each player's three rows are compared independently against every other player's corresponding rows. For each opponent matchup:

| Result | Points |
|--------|--------|
| Win 2 of 3 rows | +1 point (net) |
| Lose 2 of 3 rows | -1 point (net) |
| Win all 3 rows (scoop) | +6 points |
| Lose all 3 rows (scooped) | -6 points |
| Tied row | 0 for that row |

The scoop bonus replaces the standard per-row scoring: winning all three rows awards 1+1+1+3 = 6 total (not 3).

### Fouling (Busting)

If a player's rows violate the strength ordering (front stronger than middle, or middle stronger than back), the hand is **fouled**. A fouled hand:

- Automatically loses all three rows to every non-fouled opponent.
- Pays 6 points (scoop penalty) to each non-fouled opponent.
- Collects no royalties.

### Royalties (Bonus Points)

Royalties are bonus points earned for making specific hands in each row. They are paid by each opponent regardless of who wins the row, as long as the player does not foul.

#### Back Row Royalties

| Hand           | Points |
|----------------|--------|
| Straight       | 2      |
| Flush          | 4      |
| Full House     | 6      |
| Four of a Kind | 10     |
| Straight Flush | 15     |
| Royal Flush    | 25     |

#### Middle Row Royalties

Middle royalties are **double** the back row values, plus a bonus for three-of-a-kind:

| Hand           | Points |
|----------------|--------|
| Three of a Kind| 2      |
| Straight       | 4      |
| Flush          | 8      |
| Full House     | 12     |
| Four of a Kind | 20     |
| Straight Flush | 30     |
| Royal Flush    | 50     |

#### Front Row Royalties

| Hand           | Points |
|----------------|--------|
| Pair of 6s     | 1      |
| Pair of 7s     | 2      |
| Pair of 8s     | 3      |
| Pair of 9s     | 4      |
| Pair of 10s    | 5      |
| Pair of Jacks  | 6      |
| Pair of Queens | 7      |
| Pair of Kings  | 8      |
| Pair of Aces   | 9      |
| Trip 2s        | 10     |
| Trip 3s        | 11     |
| Trip 4s        | 12     |
| Trip 5s        | 13     |
| Trip 6s        | 14     |
| Trip 7s        | 15     |
| Trip 8s        | 16     |
| Trip 9s        | 17     |
| Trip 10s       | 18     |
| Trip Jacks     | 19     |
| Trip Queens    | 20     |
| Trip Kings     | 21     |
| Trip Aces      | 22     |

#### Royalty Netting

Royalties are netted between players. If Player A earns 6 royalty points from back-row full house and Player B earns 9 royalty points from front-row pair of aces, the net transfer is 3 points from A to B (in addition to row-win scoring).

## Fantasyland

### Entry Conditions (Standard OFC)

A player who sets **Queens or better** (QQ+) as a pair in the front row **without fouling** enters Fantasyland on the next hand.

### Entry Conditions (Pineapple OFC -- Progressive)

| Front Row Hand     | Cards Dealt in Fantasyland |
|--------------------|---------------------------|
| Pair of Queens     | 14                        |
| Pair of Kings      | 15                        |
| Pair of Aces       | 16                        |
| Three of a Kind    | 17                        |

### Fantasyland Play

- The Fantasyland player receives all their cards (13--17 depending on variant) at once, face-down.
- They set their complete hand face-down before other players begin their turns.
- Other players play the hand normally (drawing one/three cards at a time).
- After all players finish, the Fantasyland player's hand is revealed.
- In Pineapple Progressive, if dealt more than 13 cards, the player keeps 13 and discards the rest face-down.

### Staying in Fantasyland

A player already in Fantasyland remains in Fantasyland on the next hand if they set (without fouling):

| Row    | Required Hand       |
|--------|---------------------|
| Front  | Three of a Kind     |
| Middle | Full House or better |
| Back   | Four of a Kind or better |

Any one of these conditions is sufficient to stay.

### Fantasyland Rules

- The dealer button does **not** advance during Fantasyland hands.
- Multiple players can be in Fantasyland simultaneously. They all set face-down, then are revealed after non-Fantasyland players complete their hands.

## Special Rules

### Surrendering (Optional)

Some games allow a player in Fantasyland to surrender their hand if it is weak, paying a fixed penalty (typically 4 points per opponent) rather than risking a foul or bad rows.

### Pineapple 2-7 Variant

- The middle row uses **2-7 lowball** rankings instead of standard poker rankings.
- The middle hand cannot exceed 10-high without fouling.
- Aces count as high for the middle row.
- Back must still beat front in standard poker rankings.
- Middle row royalties: 9-high = 1, 8-high = 2, 7-high = 4, perfect 7-5-4-3-2 = 8.
- Fantasyland entry: KK+ in front, or 7-5-4-3-2 in middle (both = 15 cards).

### Turbo OFC

- Initial deal: 5 cards (set all 5).
- Subsequent draws: 4 cards per turn, place all 4 (no discards).
- Two drawing rounds total (5 + 4 + 4 = 13 cards).
- Fantasyland: QQ+ in front = 13 cards dealt at once.
- Fastest OFC variant.

## Key Strategic Concepts

- **Row ordering discipline:** Every placement must consider future cards. Setting an overly strong middle makes it harder to make a valid back.
- **Fantasyland chasing:** The massive advantage of Fantasyland (seeing all cards at once) creates incentive to aggressively pursue QQ+ in front, even at foul risk.
- **Live card tracking:** Since all cards are face-up, tracking which cards remain in the deck is critical for probability calculations.
- **Foul avoidance vs. royalty maximization:** The tension between playing safe (avoiding a foul) and playing aggressively (maximizing royalties) is the core strategic decision.
- **Scoop defense:** When trailing, preventing a scoop (-6) is more important than winning individual rows (+1).
- **Position advantage:** Acting last provides information about opponents' placements before making decisions.

## Common Terminology

| Term | Definition |
|------|-----------|
| **Front / Top** | The 3-card row (weakest hand required) |
| **Middle** | The 5-card middle row |
| **Back / Bottom** | The 5-card bottom row (strongest hand required) |
| **Foul / Bust** | Invalid hand where row ordering is violated |
| **Scoop / Sweep** | Winning all three rows against one opponent (+6 points) |
| **Royalty / Bonus** | Extra points for specific hand strengths in each row |
| **Fantasyland** | Bonus state where all cards are dealt at once |
| **Live cards** | Cards still in the deck (not yet placed by any player) |
| **Set** | To place a card into a row |
| **Button** | Dealer position marker; determines action order |
| **Pineapple** | Variant where 3 cards are drawn and 1 discarded per turn |
| **Progressive** | Pineapple variant with graduated Fantasyland card counts |

## State Space Analysis

### Information Sets
- Standard OFC: all placed cards visible. Unknown: future deal cards only.
- Pineapple OFC: placed cards visible, but discarded cards create hidden information.
- Initial 5-card placement: C(52,5) possible deals × row placement options.
- Subsequent placement decisions depend on visible state and unknown remaining cards.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Initial 5-card deals | C(52,5) = 2,598,960 |
| Placement options per round | Variable (depends on remaining slots) |
| Pineapple: cards per round | 3 dealt, choose 2, discard 1 |
| Total game tree (Pineapple OFC, 2-player) | ~10^40-10^60 (estimated) |
| Information sets (Pineapple) | ~10^30-10^45 |

### Action Space
- **Initial placement**: place 5 cards into top (3 slots) + middle (5 slots) + bottom (5 slots). Combinatorial but bounded.
- **Subsequent Pineapple rounds**: choose 2 of 3 cards to keep, then place each in a legal row.
- **Constraint enforcement**: each placement must maintain the possibility of completing a valid (non-fouling) hand.
- Branching factor per round: ~6-15 (placement options × discard choice).

## Key Challenges for AI/Solver Approaches

### 1. Sequential Placement Under Uncertainty
Cards are placed one at a time (or in small batches), and placements are irrevocable. The solver must:
- Evaluate the long-term value of each placement.
- Balance competing objectives across three rows.
- Account for future card distributions.

### 2. Fantasyland Dynamics
Fantasyland creates a qualitative shift in the game:
- Achieving Fantasyland is worth a large expected value (EV).
- Aggressive top-row play (aiming for QQ+) increases foul risk but opens Fantasyland.
- Staying in Fantasyland creates a compounding advantage.
- The Fantasyland subgame (place 13+ cards optimally) is a combinatorial optimization problem solvable by exhaustive search.

### 3. Multi-Row Optimization
Placing a card in one row affects the possibilities for other rows:
- A strong card in the top limits options for middle/bottom.
- Must balance royalty potential across all three rows.
- The foul constraint creates hard dependencies between rows.

### 4. Opponent Card Visibility
In standard OFC, all placed cards are visible, enabling:
- Card counting (knowing which cards remain in the deck).
- Adjusting strategy based on opponent's developing hands.
- In Pineapple OFC, discarded cards add imperfect information.

### 5. Foul Risk Management
The catastrophic penalty for fouling creates a risk-management problem:
- Must ensure that the developing hand can always be completed without fouling.
- Sometimes the optimal play is to "abandon" a row to avoid fouling.
- Safety-first vs royalty-maximizing strategies.

## Known Solver Results

### Academic Work
- Limited published research on OFC-specific solving.
- Some work on optimal Fantasyland placement (combinatorial optimization).
- No published Nash equilibrium computation for OFC.

### Practical Solvers
- **OFC Pineapple calculators**: tools that compute expected value for specific placements using Monte Carlo simulation.
- **Fantasyland solvers**: brute-force search for optimal 13-card placement (tractable due to small state space once all 13 cards are known).
- **Training tools**: some commercial applications for OFC practice use heuristic evaluation.

### AI Approaches
- **Monte Carlo simulation**: the primary approach — sample unknown cards, evaluate placements.
- **Rule-based heuristics**: expert-encoded rules for placement priorities.
- **RL-based**: limited published work, but the sequential nature of OFC makes it a natural fit for RL.
- **No known superhuman AI**: OFC lacks the research attention of mainstream poker variants.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2014 | Various poker strategy articles | OFC strategy frameworks |
| 2018 | Tian & Zhang, "Learning to Play OFC Pineapple" | RL approach to OFC |
| 2020 | Online poker AI conferences | OFC as benchmark game |

## Relevance to Myosu

### Solver Applicability
OFC tests solver architectures on **sequential placement optimization**:
- **CFR**: applicable to simplified OFC (2-player, limited cards). Full OFC likely requires significant abstraction.
- **Monte Carlo simulation**: proven practical approach for OFC placement decisions.
- **Neural networks**: can learn placement value functions from self-play.
- **Combinatorial optimization**: Fantasyland placement is a clean optimization problem.
- **RL**: natural fit — each placement is a sequential decision with delayed reward.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 2/5 | Large game tree; heavy abstraction needed |
| Neural value network potential | 4/5 | Placement value estimation is learnable |
| Abstraction necessity | 4/5 | Needed for full-game solving |
| Real-time solving value | 4/5 | MC + neural evaluation for each placement |
| Transferability of techniques | 3/5 | Sequential placement is somewhat unique |

### Myosu Subnet Considerations
- **Niche but devoted player base**: OFC is popular in the poker community, particularly Pineapple variant.
- **Clean evaluation metric**: royalty points + scoop bonuses provide a well-defined scoring system.
- **Fantasyland solver as subproblem**: the optimal Fantasyland placement is tractable and can serve as a standalone evaluation task.
- **Game oracle**: row comparison, royalty calculation, and foul detection are deterministic and verifiable.
- **Card visibility**: the open-face nature means game states are largely public — simplifies verification and replay.
- **Sequential decision quality**: each placement can be independently evaluated for EV, providing fine-grained strategy assessment.

### Recommended Approach for Myosu
1. Monte Carlo simulation for real-time placement decisions.
2. Neural value networks trained via self-play for placement evaluation.
3. Exact solver for Fantasyland placement (brute-force is feasible).
4. Evaluate strategies by expected royalty income + scoop rate + foul rate.
5. Use OFC as a "placement optimization" benchmark distinct from traditional poker betting games.
