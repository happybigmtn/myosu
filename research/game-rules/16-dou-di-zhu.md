# Dou Di Zhu (Fight the Landlord)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Dou Di Zhu (斗地主, "Fight the Landlord") |
| Variants | Classic 3-player, 4-player (with partner), Laizi (wild card), Happy DDZ |
| Players | 3 (standard), 4 (variant) |
| Information | Imperfect (hidden hands, 3 hidden "kitty" cards until landlord reveal) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (2 peasants vs 1 landlord) |
| Solved Status | Unsolved; active research; AI competitions (DouZero, 2021) |

## Overview

Dou Di Zhu (斗地主, literally "Fight the Landlord") is a climbing card game and the most popular card game in China. It features asymmetric gameplay: one player (the Landlord) plays solo against the other two players (the Peasants), who form a temporary alliance. The game combines hand management, bluffing, and tactical card play with a rich system of legal combinations.

- **Players:** 3 (standard); 4-player variant exists
- **Duration:** 5--15 minutes per hand
- **Objective:** Be the first to play all cards from your hand. The Landlord plays against two Peasants.

## Equipment

- One standard 54-card deck: 52 standard cards plus 2 jokers (one Red/Big Joker, one Black/Small Joker)
- Scoring chips or a point-tracking system

### Card Rankings (Individual)

From highest to lowest:

**Red Joker > Black Joker > 2 > A > K > Q > J > 10 > 9 > 8 > 7 > 6 > 5 > 4 > 3**

Suits are irrelevant in Dou Di Zhu. Cards of the same rank are equal regardless of suit.

## Setup

1. Select a dealer (any method). The dealer shuffles the 54-card deck.
2. Deal **17 cards** to each player, one at a time.
3. Place the remaining **3 cards** face-down in the center as the "kitty" (底牌).

## Game Flow

### Phase 1: Landlord Selection (Bidding)

The bidding determines who becomes the Landlord and sets the base stake multiplier.

1. A starting bidder is determined (often the player who received a specific face-up card during the deal, or rotation from previous hand, or random).
2. Starting with the designated player and proceeding counter-clockwise, each player may either **bid** or **pass**.
3. Valid bids are **1, 2, or 3** (representing the stake multiplier). Each subsequent bid must be higher than the previous bid.
4. Bidding ends when:
   - A player bids **3** (the maximum -- they immediately become Landlord).
   - Two consecutive players pass after a bid has been made (the last bidder becomes Landlord).
   - All three players pass (the hand is redealt).
5. The Landlord picks up the 3 kitty cards (now visible to all players) and adds them to their hand, giving the Landlord **20 cards** while each Peasant has **17 cards**.

### Phase 2: Card Play

1. **The Landlord plays first** and may play any single card or any legal combination.
2. Play proceeds **counter-clockwise**. Each subsequent player may either:
   - **Play** a combination of the same type and same number of cards that beats the current lead (higher rank), OR
   - **Pass** (play nothing).
3. **Exception:** Bombs and Rockets can beat any combination regardless of type (see Combinations below).
4. When two consecutive players pass, the last player who played leads a new round with any card or combination.
5. Play continues until one player empties their hand.

## Actions

On each turn, a player either:

1. **Plays** a legal combination that beats the current one on the table.
2. **Passes** (choosing not to play, even if they could).

Passing is always permitted. There is no obligation to play even if you hold a beating combination.

## Card Combinations

### Basic Types

| # | Combination | Description | Example |
|---|-------------|-------------|---------|
| 1 | **Single** | One card | 7 |
| 2 | **Pair** | Two cards of the same rank | 8-8 |
| 3 | **Triple** | Three cards of the same rank | K-K-K |
| 4 | **Triple + Single** | Triple plus one unrelated card | 6-6-6-8 |
| 5 | **Triple + Pair** | Triple plus one pair | 6-6-6-8-8 |
| 6 | **Sequence (Straight)** | 5+ consecutive singles (3 through A only) | 3-4-5-6-7 |
| 7 | **Pair Sequence** | 3+ consecutive pairs (3 through A only) | 7-7-8-8-9-9 |
| 8 | **Triple Sequence (Airplane)** | 2+ consecutive triples (3 through A only) | 4-4-4-5-5-5 |
| 9 | **Airplane + Singles** | Triple sequence + one single card per triple | 4-4-4-5-5-5-7-9 |
| 10 | **Airplane + Pairs** | Triple sequence + one pair per triple | 4-4-4-5-5-5-7-7-9-9 |
| 11 | **Quadplex + 2 Singles** | Four of a kind + two different single cards | 6-6-6-6-8-9 |
| 12 | **Quadplex + 2 Pairs** | Four of a kind + two different pairs | 6-6-6-6-8-8-9-9 |
| 13 | **Bomb** | Four cards of the same rank | J-J-J-J |
| 14 | **Rocket** | Both jokers together | Red Joker + Black Joker |

### Sequence Restrictions

- **2s and Jokers may never appear in any sequence, pair sequence, or triple sequence.** The valid range for sequences is 3 through A only.
- Minimum straight length: **5 cards** (e.g., 3-4-5-6-7).
- Minimum pair sequence length: **3 pairs** (e.g., 3-3-4-4-5-5).
- Minimum triple sequence length: **2 triples** (e.g., 3-3-3-4-4-4).

### Attachment Rules for Airplane + Singles/Pairs

- Attached cards must all be **different ranks** from each other and from the triples.
- **2s may be attached** as singles or in pairs.
- **Jokers may be attached** as singles.
- The two jokers together **cannot** be attached as a "pair" to an airplane (they are not a true pair of the same rank).
- A bomb (four of the same rank) cannot be used as an attached pair.

### Quadplex Set Rules

- The quadplex is **not a bomb** -- it is played as a regular combination and does not beat other types.
- The two attached singles must be **different ranks**.
- The two attached pairs must be **different ranks**.

### Beating Rules

A combination can be beaten by:

1. A **higher-ranked combination of the same type and card count.** Comparison is by the rank of the primary component (the triple in a triple+single, the sequence's lowest card in a straight, etc.).
2. A **Bomb** beats any non-bomb, non-rocket combination.
3. A **higher Bomb** beats a lower Bomb (ranked by the four-of-a-kind's rank).
4. A **Rocket** beats everything, including any Bomb. The Rocket is the highest possible play.

## Scoring / Winning

### Win Condition

- If the **Landlord** plays all cards first: the Landlord wins.
- If **either Peasant** plays all cards first: both Peasants win (the other Peasant does not need to empty their hand).

### Point Calculation

The base payment is the bid value (1, 2, or 3) multiplied by applicable multipliers:

```
Payment = Bid x 2^(number of bombs played) x 2^(number of rockets played) x Spring_Multiplier
```

#### Bomb/Rocket Multiplier

Each time **any player** plays a Bomb or Rocket during the hand, the payment **doubles**.

| Bombs + Rockets Played | Multiplier |
|------------------------|------------|
| 0 | x1 |
| 1 | x2 |
| 2 | x4 |
| 3 | x8 |
| n | x2^n |

#### Spring Bonus (春天)

- **Landlord Spring:** If the Landlord wins and neither Peasant played a single card (the Landlord led every round and was never beaten), the payment is **doubled** again.
- **Peasant Spring (Anti-Spring):** If a Peasant wins and the Landlord played only their very first lead (never got to lead again after their opening play), the payment is doubled.

#### Payment Direction

- **Landlord wins:** Each Peasant pays the Landlord the full calculated payment.
- **Peasants win:** The Landlord pays each Peasant the full calculated payment.

#### Example

Bid = 3, two Bombs played during the hand, Landlord Spring applies:
- Payment = 3 x 2^2 x 2 (spring) = 3 x 4 x 2 = **24 points** per Peasant.

## Special Rules

### Redeal

If all three players pass during the bidding phase, the hand is void and redealt with the next dealer.

### Visible Kitty

The 3 kitty cards are revealed to all players after the Landlord picks them up. This information is public.

### Peasant Cooperation

The two Peasants cannot show each other their cards or explicitly communicate hand information. However, they can cooperate through their play choices (e.g., one Peasant might pass to let their ally take the lead).

### Four-Player Variant

- Uses a **double deck** (108 cards: two standard 52-card decks + 4 jokers -- 2 red, 2 black).
- Each player receives **25 cards**; the Landlord gets **33 cards** (25 + 8 kitty cards).
- Bombs can be **4 or more** cards of the same rank (more cards = higher bomb regardless of rank).
- The Rocket requires **all 4 jokers**.
- Quadplex sets and single-card attachments are not used.
- Bombs of 6+ cards or Rockets double the payment; 4-5 card bombs do not affect the multiplier.

## Key Strategic Concepts

- **Landlord bid assessment:** Bid high only with strong hands containing multiple bombs, rockets, or connected sequences. The kitty cards can dramatically improve a marginal hand.
- **Tempo control:** The player who leads controls the combination type. Leading with your strongest combination type (e.g., sequences if you have long runs) forces opponents to respond on your terms.
- **Bomb timing:** Playing a bomb doubles the stakes but also reveals information. Holding bombs for endgame can be more valuable than playing them early.
- **Peasant coordination:** Although Peasants cannot communicate explicitly, good Peasant play involves recognizing which Peasant is closer to going out and supporting them (passing to let them lead, breaking the Landlord's leads).
- **Card counting:** With only 54 cards and all kitty cards revealed, tracking which cards have been played is feasible and critical.
- **Endgame planning:** Structuring your hand to leave a single clean combination at the end (e.g., a single bomb or a sequence that finishes your hand) is the hallmark of expert play.

## Common Terminology

| Term | Definition |
|------|-----------|
| **Landlord (地主)** | The solo player who plays against both Peasants |
| **Peasant (农民)** | The two allied players opposing the Landlord |
| **Kitty (底牌)** | The 3 face-down reserve cards awarded to the Landlord |
| **Bomb (炸弹)** | Four cards of the same rank; beats any non-bomb/rocket |
| **Rocket (火箭)** | Both jokers played together; the highest possible play |
| **Airplane (飞机)** | A triple sequence (2+ consecutive triples) |
| **Straight/Chain (顺子)** | A sequence of 5+ consecutive singles |
| **Pair Sequence (连对)** | 3+ consecutive pairs |
| **Spring (春天)** | Bonus when the losing side played no cards (or only responded once) |
| **Quadplex Set (四带二)** | Four of a kind + 2 singles or 2 pairs (not a bomb) |
| **Pass (不出/过)** | Choosing not to play on your turn |
| **Lead (出牌)** | Starting a new round after the previous play was unchallenged |
| **Bid (叫分)** | The auction to become Landlord (1, 2, or 3) |

## State Space Analysis

### Information Sets
- Landlord's initial hand: C(54,20) = enormous.
- Peasant's hand: C(54,17) each.
- Hidden information: opponents' remaining cards, except as revealed by play.
- The 3 kitty cards are revealed to all when the landlord picks them up (in most rules).

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible deals | C(54,17) × C(37,17) × C(20,3) ≈ 10^28 |
| Legal combinations per turn | Variable, ~20-100+ (many combo types) |
| Average game length | ~15-30 actions |
| Game tree nodes | ~10^30-10^50 (estimated) |
| Information sets | ~10^20-10^35 |
| Branching factor | ~30-100+ (large due to combination variety) |

### Action Space
- Highly variable per turn:
  - If leading: can play any legal combination type.
  - If following: must match combination type with higher rank, or pass.
  - Some positions have 100+ legal actions (e.g., many possible chain/airplane combinations).
- The combination-based play creates a much larger action space per decision than typical card games.

## Key Challenges for AI/Solver Approaches

### 1. Asymmetric Teams (1 vs 2)
The landlord vs peasants structure creates:
- Peasant coordination: the two peasants should cooperate but cannot communicate.
- Information asymmetry: the landlord knows the kitty cards; peasants see them but don't know which are original and which are kitty-derived.
- Asymmetric hand sizes: landlord has 20 cards vs peasants' 17 each.

### 2. Large Action Space
The combination-based play creates a very large action space:
- Chains of different lengths.
- Airplanes with different kicker choices.
- Multiple valid decompositions of the same hand into playable combinations.
- This is fundamentally different from trick-taking games where each player plays exactly one card.

### 3. Combination Recognition and Planning
Optimal play requires:
- Decomposing the hand into an efficient set of combinations.
- Planning the order of play to maximize the chance of being the first to empty the hand.
- Evaluating the trade-off between playing suboptimal combinations now vs keeping cards for better combinations later.

### 4. Bomb Dynamics
Bombs (and rockets) act as "wild" combinations that beat anything:
- Holding a bomb provides a guaranteed future lead (can recapture initiative).
- Playing a bomb doubles the stakes — risk/reward decision.
- Estimating opponents' bomb holdings is critical for risk management.

### 5. Peasant Coordination
The two peasants must cooperate without explicit communication:
- Signaling through card play (e.g., playing high to lead for partner).
- Sacrificing hand efficiency to help partner escape.
- Knowing when to play and when to pass to support partner.

## Known Solver Results

### DouZero (Zha et al., 2021)
The landmark AI result for Dou Di Zhu:
- Achieved superhuman performance against state-of-the-art AI systems and strong human players.
- Architecture: **Deep Monte Carlo (DMC)** — a model-free RL approach using Monte Carlo methods with deep neural networks.
- Key innovations:
  1. Trains separate networks for each position (landlord, peasant-down, peasant-up).
  2. Uses action encoding that handles the combinatorial action space.
  3. Parallel self-play with experience replay.
  4. No search at test time — pure policy network.
- Published at ICML 2021.
- Open source: [github.com/kwai/DouZero](https://github.com/kwai/DouZero).

### DeltaDou (ByteDance, 2024)
- Further advancement using a combination of supervised learning and reinforcement learning.
- Monte Carlo Tree Search at test time for improved decision making.
- Builds on DouZero foundations.

### Other Work
- **CCP**: combinatorial game-theoretic approach.
- **Various Chinese AI labs**: significant industry research due to DDZ's massive popularity in China.
- **Botzone platform**: online AI competition platform with DDZ as a featured game.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2019 | You et al., "Combinational Q-Learning for Dou Di Zhu" | Q-learning with combo action space |
| 2021 | Zha et al., "DouZero: Mastering DouDiZhu with Self-Play Deep RL" | Superhuman DDZ AI (ICML 2021) |
| 2022 | Zhao et al., "DouzeroPlus" | Enhanced DouZero with planning |
| 2024 | Jiang et al., "DeltaDou" | MCTS-enhanced DDZ AI |

## Relevance to Myosu

### Solver Applicability
Dou Di Zhu is a **flagship shedding game** with unique structure:
- **CFR**: limited applicability due to asymmetric teams and large action space. Not the right tool for DDZ.
- **Deep RL (DMC/PPO)**: the proven approach (DouZero). Model-free RL handles the large action space well.
- **MCTS + neural networks**: DeltaDou shows search can improve upon pure policy networks.
- **Action encoding**: handling the combinatorial action space (chains, airplanes, etc.) requires specialized encoding.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 1/5 | Asymmetric teams, huge action space |
| Neural value network potential | 5/5 | Essential; DouZero is proof |
| Abstraction necessity | 3/5 | Action space needs encoding, not traditional abstraction |
| Real-time solving value | 4/5 | MCTS improves on pure RL (DeltaDou) |
| Transferability of techniques | 4/5 | Shedding game techniques transfer to Pusoy Dos, Tien Len |

### Myosu Subnet Considerations
- **Massive market**: DDZ has 600+ million registered players in China. It's the most-played card game in the world by some metrics.
- **DouZero is open source**: provides a baseline that solver submissions can be benchmarked against.
- **Action encoding challenge**: the subnet's game interface must handle the combinatorial action space correctly.
- **Asymmetric evaluation**: landlord and peasant strategies must be evaluated separately — a strong landlord strategy may not be a strong peasant strategy.
- **Combination validation oracle**: must correctly identify and validate all legal combination types. The airplane + wings combos are particularly tricky to validate.
- **Cultural significance**: including DDZ is essential for engagement in the Chinese-speaking market.

### Recommended Approach for Myosu
1. Deep RL (DMC or PPO) as the primary solving approach, following DouZero architecture.
2. Separate models for landlord and peasant roles.
3. MCTS for test-time search (DeltaDou-style) for enhanced performance.
4. Use DouZero as the baseline benchmark — solver submissions must outperform it.
5. Evaluate across both roles (landlord win rate + peasant win rate) for comprehensive assessment.
