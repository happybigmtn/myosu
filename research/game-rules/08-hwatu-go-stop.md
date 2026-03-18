# Hwatu Go-Stop (화투 고스톱)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Go-Stop (고스톱), played with Hwatu (화투, "flower fight") cards |
| Variants | 2-player Go-Stop, 3-player Go-Stop (standard), Min-Hwa-Tu, Seotda |
| Players | 2-3 (standard: 3) |
| Information | Imperfect (hidden hand cards, unknown draw pile) |
| Stochasticity | Stochastic (card deals, draw pile) |
| Zero-Sum | Yes (among active players) |
| Solved Status | Unsolved; very limited academic study |

## Overview

Go-Stop (고스톱) is a Korean fishing card game played with Hwatu (화투, "flower cards"), a 48-card deck derived from Japanese Hanafuda. It is the most popular card game in Korea, played casually during holidays (especially Seollal and Chuseok) and as a gambling game year-round. Two or three players capture cards from a central field by matching months, then form scoring combinations. The game's signature mechanic is the "Go" or "Stop" decision: when a player reaches the minimum score threshold, they choose whether to stop and collect winnings or continue ("Go") for a multiplied score at the risk of another player winning first.

**Players:** 2 or 3 (3-player is standard; 2-player variant exists)
**Deck:** 48 Hwatu cards + 2 bonus/joker cards (50 cards total in most sets)
**Objective:** Be the first player to accumulate enough points to call "Stop" and win the hand

## Equipment

### The Hwatu Deck (48 + 2 Cards)
Hwatu cards mirror the Hanafuda structure: 12 suits (months), 4 cards per month, but with Korean artistic styling. Cards are divided into four categories:

| Month | Plant (Korean) | Gwang/Bright (광) | Animal/Yul (열) | Ribbon/Tti (띠) | Junk/Pi (피) |
|-------|---------------|-------------------|-----------------|-----------------|--------------|
| **1 (Jan)** | Pine (솔) | **Crane** (학) | | Red Poetry Ribbon (홍단) | Plain x2 |
| **2 (Feb)** | Plum Blossom (매화) | | Nightingale (꾀꼬리) | Red Poetry Ribbon (홍단) | Plain x2 |
| **3 (Mar)** | Cherry Blossom (벚꽃) | **Curtain** (막) | | Red Poetry Ribbon (홍단) | Plain x2 |
| **4 (Apr)** | Wisteria (등) | | Cuckoo (뻐꾸기) | Plain Red Ribbon | Plain x2 |
| **5 (May)** | Iris (난초) | | Eight-Plank Bridge (다리) | Plain Red Ribbon | Plain x2 |
| **6 (Jun)** | Peony (모란) | | Butterflies (나비) | Blue Ribbon (청단) | Plain x2 |
| **7 (Jul)** | Bush Clover (싸리) | | Boar (멧돼지) | Plain Red Ribbon | Plain x2 |
| **8 (Aug)** | Pampas Grass (억새) | **Full Moon** (달) | Geese (기러기) | | Plain x2 |
| **9 (Sep)** | Chrysanthemum (국화) | | Sake Cup (술잔) | Blue Ribbon (청단) | Plain x2 |
| **10 (Oct)** | Maple (단풍) | | Deer (사슴) | Blue Ribbon (청단) | Plain x2 |
| **11 (Nov)** | Paulownia (오동) | **Phoenix** (봉황) | | | Double-Junk (쌍피) + Plain |
| **12 (Dec)** | Willow/Rain (비) | **Rain Man** (비광) | Swallow (제비) | Red Ribbon | Double-Junk (쌍피) |

### Card Type Totals
- **Gwang (Bright):** 5 cards — Crane (Jan), Curtain (Mar), Moon (Aug), Rain Man (Dec), Phoenix (Nov)
- **Yul (Animal):** 9 cards
- **Tti (Ribbon):** 10 cards — 3 Red Poetry (홍단: Jan/Feb/Mar), 3 Blue (청단: Jun/Sep/Oct), 3 Plain Red (Apr/May/Jul), 1 Willow Red (Dec)
- **Pi (Junk):** 24 cards — includes 2 Double-Junk cards (쌍피) from November and December

### Bonus/Joker Cards (2 additional cards)
Most Hwatu sets include 2 extra cards not belonging to any month:
- **Bonus card counting as 2 Junk** (이끗)
- **Bonus card counting as 3 Junk** (삼끗)

These bring the total deck to 50 cards. Bonus cards serve as extra junk for scoring purposes and sometimes have special capture effects (see Special Rules).

### Special Card Notes
- **Double-Junk (쌍피):** The November plain and December plain cards each count as 2 junk cards for scoring purposes.
- **Sake Cup (September):** Counts as both an Animal card AND 1 junk card simultaneously.
- **Rain Man (December Bright):** Gwang card but treated specially — including it in Bright combinations reduces their value (see Scoring).

## Setup

### 3-Player Game (Standard)
1. Determine the first dealer (선, Seon) by drawing cards; earliest month becomes dealer.
2. The dealer shuffles and deals:
   - 7 cards to each of the 3 players (21 cards total)
   - 6 cards face-up to the field (바닥, Badak)
   - Remaining 23 cards form the draw pile (더미, Deomi)
3. Bonus cards in the initial deal are handled specially (see Special Rules).

### 2-Player Game
1. Deal 10 cards to each player.
2. 8 cards face-up to the field.
3. Remaining 20 cards form the draw pile.

### Field Setup Check
- If all 4 cards of a month appear on the field, the dealer captures all 4 immediately and draws a replacement card.
- If 3 cards of a month are on the field, they are stacked together (see "Stacking" under Special Rules).

## Game Flow

### Turn Structure
Play proceeds counter-clockwise from the dealer. On each turn:

**Step 1 — Play a card from hand:**
- Play one card to the field.
- If it matches one field card of the same month, stack it on top (both will be captured in Step 2 if the drawn card also matches, or at the end of the step if only 2 cards match).
- Actually: if it matches exactly one field card, both are captured immediately to your scoring pile.
- If it matches two field cards, choose which one to capture.
- If it matches three field cards (all three on the field), capture all three plus your played card (a "sweep" of the month).
- If no match, the card is placed on the field.

**Step 2 — Draw from the deck:**
- Draw the top card from the draw pile and reveal it.
- Apply the same matching rules as Step 1.
- If the drawn card matches a card you just placed on the field in Step 1 (because it didn't match anything), you capture both — this is called **Ppuk** (뻑) or "kiss," and results in a penalty to you or a bonus depending on the rule set.
- If no match, the drawn card is placed on the field.

**Step 3 — Check score and decide Go/Stop:**
- After capturing, count your points from scoring combinations (jokbo).
- If your total is at or above the minimum point threshold (typically 3 points for 3 players, 2 points for 2 players):
  - **Go (고):** Continue playing. Your score will be increased and/or multiplied.
  - **Stop (스톱):** End the hand. You win and collect payment from all opponents.
- If below the threshold, play passes to the next player.

### The Go/Stop Decision
This is the core strategic decision:
- Calling **Go** signals you want more points. Each Go call adds bonus points and eventually multiplies your score.
- But if another player reaches the threshold and calls Stop before your next turn, you lose — and pay extra penalties for having called Go.
- You must call Go or Stop on any turn where you meet or exceed the threshold. You cannot "silently" exceed the threshold.

## Actions

### Capture
Match a card from your hand or the draw pile to a field card of the same month. Both cards go to your scoring pile.

### Steal (뺏기 / Ppaetgi)
Certain events allow you to steal junk cards from opponents:
- Completing a **bomb** (폭탄) steals 1 junk from each opponent.
- Certain joker captures steal junk from opponents (house-rule dependent).

### Shake (흔들기 / Heundeulgi)
If you hold 3 cards of the same month in your hand, you may reveal them ("shake") before playing. This doubles your final score if you win the hand. You may shake multiple months for multiplicative doubling.

### Bomb (폭탄 / Poktan)
If you hold 3 cards of a month in your hand AND the 4th card is on the field:
1. Play all 3 cards at once, capturing all 4 cards of that month.
2. Steal 1 junk card from each opponent's scoring pile.
3. You may optionally shake before bombing.
4. After bombing, your next 1-2 turns may skip Step 1 (play only Step 2, drawing from the deck) since you used 3 hand cards at once.

## Scoring

Points come from combinations of captured cards. The scoring categories (족보, Jokbo) are:

### Gwang (Bright) Combinations

| Combination | Cards | Points |
|------------|-------|--------|
| **Ogwang** (오광 / Five Brights) | All 5 Gwang cards | 15 |
| **Sagwang** (사광 / Four Brights) | Any 4 Gwang, NOT including Rain Man | 4 |
| **Bi-Sagwang** (비사광 / Rainy Four Brights) | Rain Man + any 3 other Gwang | 4 (some rule sets: 3) |
| **Samgwang** (삼광 / Three Brights) | Any 3 Gwang, NOT including Rain Man | 3 |
| **Bi-Samgwang** (비삼광 / Rainy Three Brights) | Rain Man + any 2 other Gwang | 2 |

Only the highest qualifying Gwang combination scores. They do not stack.

### Animal (열끗 / Yeolkkeut) Scoring

| Combination | Cards | Points |
|------------|-------|--------|
| **Godori** (고도리 / Five Birds) | Cuckoo (Apr) + Geese (Aug) + Nightingale (Feb) | 5 |
| **Base Animals** | Any 5 Animal cards | 1 (+1 per additional Animal beyond 5) |

Godori stacks with the base Animal count: capturing the 3 bird cards plus 2 other Animals = 5 (Godori) + 1 (5 Animals base) = 6 points.

### Ribbon (띠 / Tti) Scoring

| Combination | Cards | Points |
|------------|-------|--------|
| **Hongdan** (홍단 / Red Poetry) | Jan + Feb + Mar red poetry ribbons | 3 |
| **Cheongdan** (청단 / Blue Ribbons) | Jun + Sep + Oct blue ribbons | 3 |
| **Chodan** (초단 / Grass Ribbons) | Apr + May + Jul plain red ribbons | 3 |
| **Base Ribbons** | Any 5 Ribbon cards | 1 (+1 per additional Ribbon beyond 5) |

Named ribbon combinations (Hongdan, Cheongdan, Chodan) stack with each other and with the base ribbon count.

### Junk (피 / Pi) Scoring

| Combination | Cards | Points |
|------------|-------|--------|
| **Base Junk** | 10 or more junk cards (counting double-junk as 2) | 1 (+1 per additional junk beyond 10) |

Remember: Double-Junk cards (쌍피) count as 2, the Sake Cup counts as 1 junk in addition to being an Animal, and bonus joker cards count as 2 or 3 junk respectively.

### Go Bonuses and Multipliers

When a player calls Go and later wins:

| Number of Go Calls | Bonus |
|--------------------|-------|
| 1 Go | +1 point |
| 2 Go | +2 points |
| 3 Go | Score is doubled (x2) |
| 4+ Go | Score is doubled again for each additional Go beyond 3 |

### Penalty Multipliers (Applied to Losers)

These multiply the amount each loser must pay:

| Penalty | Condition | Multiplier |
|---------|-----------|------------|
| **Gwang-bak** (광박 / Bright Wipeout) | Loser captured zero Gwang cards AND winner has any Gwang scoring combination | x2 |
| **Pi-bak** (피박 / Junk Wipeout) | Loser captured fewer than 5 junk AND winner scored from junk | x2 |
| **Meongttaerigi** (먹튀) | Loser had previously called Go in this hand | Loser pays the third player's share too |

These multipliers stack multiplicatively. A player who is both Gwang-bak and Pi-bak pays x4.

### Shake Multiplier
Each shake performed by the winner doubles the final payment. Two shakes = x4, etc.

## Winning Conditions

### Winning a Hand
A player wins when they:
1. Reach the minimum point threshold (typically 3 points for 3-player, varies by house rules).
2. Call "Stop" on their turn.
3. Collect payment from all other players based on their final score, including Go bonuses and penalty multipliers.

### Payment Calculation
Each losing player pays the winner:
```
Payment = (Winner's Base Points + Go Bonus) x Shake Multiplier x Penalty Multipliers
```

### Special Win Conditions
- **Four-of-a-Month in Hand:** If dealt all 4 cards of a single month, the player wins immediately (총통, Chongtong). Some rule sets require this to be declared before play begins.
- **Exhaustive Draw:** If the draw pile is empty and no one has called Stop, the hand is a draw (or redealt). Some rule sets penalize the player(s) who called Go.

### Natpeok (나뻑 / No-Score Penalty)
If a player calls Go but then fails to increase their score before another player wins (or the hand ends), they are penalized. The typical penalty is doubling their payment to the winner.

## Special Rules

### Ppuk (뻑 / Kiss)
When a card you play from your hand does not match anything on the field, but the card you draw from the deck DOES match the card you just placed — you capture both, but this is considered a "kiss" (뻑). The penalty for ppuk varies:
- Some rule sets: the player pays 1 junk to the field (or each opponent).
- Some rule sets: no penalty, just a capture.

### Stacking (Three of a Month on Field)
If three cards of the same month are on the field, they are stacked. When the fourth card appears (from hand or draw pile), the player captures all four. This prevents opponents from capturing partial sets.

### Bonus Card Handling
When a bonus/joker card appears:
- **In the initial field deal:** The dealer collects it immediately and flips a replacement card from the draw pile to the field.
- **Dealt to a player's hand:** The player places it face-up in their scoring pile at the start of any turn and draws a replacement from the draw pile.
- **Drawn from the deck:** The player adds it to their scoring pile and draws another card from the deck to continue their turn.

### September Ssangpi Rule (Optional)
In some rule sets, if the two Plain cards of September are both on the field or both captured by the same player, they can count as a Double-Junk (effectively 2 junk for 1 card) instead of their normal individual values.

### Minimum Point Threshold Variations
- **3-player standard:** 3 points minimum to call Stop.
- **2-player standard:** 2 points (some rule sets use 5 or 7).
- **After Go:** Some rule sets increase the threshold by 1 for each Go called (e.g., after 1 Go, need 4 points to Stop).

## Key Strategic Concepts

### Go/Stop Judgment
The Go/Stop decision is the fundamental skill. Factors to consider:
- Your current score vs. the doubling threshold (3 Go = double).
- How close opponents are to their own scoring combinations.
- How many cards remain in the draw pile (fewer cards = less opportunity to improve).
- Whether you are vulnerable to Gwang-bak or Pi-bak penalties if an opponent wins.

### Junk Accumulation
Junk cards are individually worthless but critical in volume. Reaching 10 junk is often the easiest scoring path. Stealing junk through bombs and special captures accelerates this. Denying opponents junk below 5 inflicts Pi-bak.

### Gwang Hoarding
Capturing Gwang cards serves dual purposes: building toward high-scoring Gwang combinations and protecting yourself from Gwang-bak penalty. Even a single Gwang card in your scoring pile prevents the x2 Gwang-bak multiplier.

### Godori Awareness
The three bird cards (Feb Nightingale, Apr Cuckoo, Aug Geese) are worth 5 points together. Tracking their location and denying opponents the set is a constant concern.

### Bomb Timing
A bomb (3 in hand + 1 on field) is powerful: it captures 4 cards at once, steals junk from opponents, and the preceding shake doubles your potential winnings. But it also reveals information about your hand composition.

### Card Counting
With 48 base cards and full visibility of the field and your captures, tracking which months have been fully played and which cards remain is essential for predicting draw pile contents and opponents' hands.

## Common Terminology

| Term (Korean) | Romanization | Definition |
|--------------|-------------|------------|
| 화투 | Hwatu | "Flower cards" — the Korean card deck |
| 고스톱 | Go-Stop | The game itself; also the core decision mechanic |
| 고 | Go | Decision to continue playing for more points |
| 스톱 | Stop | Decision to end the hand and collect winnings |
| 광 | Gwang | Bright cards (the 5 highest-value cards) |
| 열끗 | Yeolkkeut | Animal/10-point cards |
| 띠 | Tti | Ribbon cards |
| 피 | Pi | Junk/plain cards |
| 쌍피 | Ssang-pi | Double-junk cards (count as 2 junk) |
| 바닥 | Badak | The field (face-up cards in the center) |
| 선 | Seon | The dealer / first player |
| 족보 | Jokbo | Scoring combinations |
| 홍단 | Hongdan | Red Poetry ribbon combination |
| 청단 | Cheongdan | Blue ribbon combination |
| 초단 | Chodan | Grass/plain ribbon combination |
| 고도리 | Godori | "Five Birds" — the three bird Animal cards |
| 폭탄 | Poktan | Bomb — playing 3 cards of a month at once |
| 흔들기 | Heundeulgi | Shake — revealing 3-of-a-month for a score multiplier |
| 뻑 | Ppuk | Kiss — drawn card matches your just-placed card |
| 광박 | Gwang-bak | Bright wipeout penalty (loser has 0 Gwang) |
| 피박 | Pi-bak | Junk wipeout penalty (loser has <5 junk) |
| 나뻑 | Natpeok | Penalty for calling Go but failing to improve |
| 총통 | Chongtong | Instant win from holding 4 cards of one month |
| 먹 | Meok | Capture (to eat/take cards) |
| 설 | Seol | Lunar New Year — a traditional Go-Stop occasion |

## State Space Analysis

### Information Sets
- 3-player deal: C(48,7) × C(41,7) × C(34,7) possible initial configurations.
- Player knows: own hand, field cards, own captures, opponents' visible captures.
- Unknown: opponents' hands, draw pile order.
- Information reduces over time as cards are played and captured.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Initial deal combinations | ~10^18 |
| Cards per hand | 7 (3-player), 10 (2-player) |
| Turns per round | 7 per player (3-player) |
| Branching factor per turn | 1-7 (hand card choice) × 1-3 (field match choice) + Go/Stop |
| Game tree nodes | ~10^15-10^25 (estimated) |
| Information sets | ~10^12-10^18 |

### Action Space
- **Hand play**: choose 1 of remaining hand cards.
- **Match choice**: when multiple field cards share the month of the played card.
- **Go/Stop decision**: binary, available after reaching point threshold.
- Small per-turn branching, but Go/Stop creates high-impact binary decisions.

## Key Challenges for AI/Solver Approaches

### 1. Three-Player Default Format
The standard 3-player format is non-zero-sum in the same way as other multiplayer games:
- Two players may independently or incidentally collude against the leader.
- Payment structure (all pay the winner) creates asymmetric incentives.

### 2. Go/Stop Risk Management
The exponential multiplier from Go calls creates extreme risk-reward dynamics:
- Calling Go at 3 points, then stopping at 7 points yields (7-3) × 2 = 8 payment.
- But if an opponent reaches 3 points and stops, the Go caller loses.
- Optimal Go/Stop strategy depends on opponent hand estimation, remaining draw pile, and field state.

### 3. Special Capture Mechanics
The bonus mechanics (ppuk, ttadak, ssul) create tactical considerations beyond simple matching:
- Setting up the field for a ttadak (double capture) next turn.
- Clearing the field for a ssul bonus.
- These mechanics reward short-term tactical planning.

### 4. Bonus Pi Economy
Bonus pi cards taken from opponents (via ttadak, ssul, etc.) are a parallel economy. Managing this is important for reaching the 10-pi threshold.

### 5. Card Counting in a Reduced Information Setting
With 48 cards and only partial visibility, card counting is valuable but imperfect:
- Knowing which months are "dead" (all 4 cards accounted for) eliminates uncertainty.
- Inferring opponent hand composition from their play patterns.
- Estimating draw pile composition for Go/Stop decisions.

## Known Solver Results

### Academic Work
- Very limited published research.
- Some Korean university theses on Go-Stop AI using heuristic approaches.
- No published Nash equilibrium computation.
- No known CFR-based solver for Go-Stop.

### AI Implementations
- Korean mobile Go-Stop apps include AI opponents, typically rule-based:
  - Match highest-value cards first.
  - Go/Stop thresholds based on simple point/risk heuristics.
  - No known advanced AI (MCTS or RL) deployed in production.

### Relation to Hanafuda/Koi-Koi
- Shares the same 48-card deck structure.
- Scoring system differs significantly.
- Go-Stop's Go/Stop mechanic is analogous to but more extreme than Koi-Koi's Koi-Koi mechanic (exponential vs linear scaling).
- Techniques developed for Koi-Koi are partially transferable.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2012 | Kim et al. (Korean) | Heuristic Go-Stop AI |
| 2016 | Lee & Park (Korean) | MCTS application to Go-Stop |
| 2019 | Hwang et al. | Reinforcement learning for simplified Go-Stop |

## Relevance to Myosu

### Solver Applicability
Go-Stop is a **high-impact, underserved game** with unique mechanics:
- **CFR**: applicable to 2-player variant. 3-player is more challenging (non-zero-sum).
- **MCTS with information set sampling**: natural approach for handling hidden cards and draw pile.
- **RL**: the Go/Stop decision's exponential payoff structure may benefit from learned risk assessment.
- **Feature engineering**: capturing yaku progress, field state, and pi count creates a rich feature space.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 3/5 | 2-player yes; 3-player limited by non-zero-sum |
| Neural value network potential | 4/5 | Rich features from scoring system |
| Abstraction necessity | 2/5 | Moderate state space, less abstraction needed |
| Real-time solving value | 3/5 | MCTS suitable for real-time play |
| Transferability of techniques | 3/5 | Shared with Koi-Koi; Go/Stop mechanic is unique |

### Myosu Subnet Considerations
- **Korean market**: Go-Stop is one of the most popular card games in Korea, played by tens of millions. No competitive AI solver exists.
- **Shared deck with Koi-Koi**: the 48-card Hwatu/Hanafuda deck means card representation code can be shared with game 07.
- **Rule standardization needed**: many house rules exist. The subnet must define a canonical rule set.
- **Go/Stop verification**: the Go/Stop decision and exponential multiplier must be implemented correctly in the oracle.
- **Bonus mechanics verification**: ppuk, ttadak, and ssul rules must be precisely implemented.
- **Paired with Koi-Koi**: these two games share infrastructure, making them efficient to support together.

### Recommended Approach for Myosu
1. Share card representation and matching logic with Koi-Koi (game 07).
2. Implement CFR for 2-player Go-Stop; MCCFR/RL for 3-player.
3. The Go/Stop decision is the key differentiator — focus solver evaluation on this decision quality.
4. Use expected value analysis for Go/Stop decisions as a quick evaluation metric.
5. Support both 2-player and 3-player variants (2-player for benchmarking, 3-player for practical play).
