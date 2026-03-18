# Hearts / Cribbage (Trick-Avoidance and Pegging)

## Game Identity

This file covers two distinct games grouped by their shared theme of indirect scoring (penalty avoidance in Hearts, pegging optimization in Cribbage).

---

## Part A: Hearts

### Game Identity

| Property | Value |
|----------|-------|
| Full Name | Hearts (Black Lady / Black Maria) |
| Variants | Standard Hearts, Omnibus Hearts (bonus for J♦), Partnership Hearts, Cancellation Hearts |
| Players | 4 (standard), 3-7 (variants) |
| Information | Imperfect (hidden hands) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (penalty points are constant-sum) |
| Solved Status | Unsolved; moderate AI research |

### Rules Summary

#### Setup
- Standard 52-card deck (4-player). For 3 players, one card removed (typically 2♣).
- 13 cards dealt to each player.
- Card ranking per suit: A > K > Q > J > 10 > 9 > 8 > 7 > 6 > 5 > 4 > 3 > 2.
- No trump suit.

#### Passing Phase
- Before play, each player passes 3 cards to another player:
  - Round 1: pass left.
  - Round 2: pass right.
  - Round 3: pass across.
  - Round 4: no passing (hold).
  - Cycle repeats.

#### Play Phase
- Player with 2♣ leads first trick (some variants: lowest club).
- Must follow suit if possible. If unable, may play any card except:
  - Hearts cannot be played on the first trick (in most rule sets).
  - Hearts cannot be led until "broken" (a heart has been discarded on a previous trick).
- Highest card of the led suit wins the trick.
- No trump suit — only the led suit matters.

#### Scoring (Penalty Points)
| Card | Penalty Points |
|------|---------------|
| Each Heart (♥) | 1 point |
| Queen of Spades (Q♠) | 13 points |
| All other cards | 0 points |
| Total per hand | 26 points distributed |

#### Shooting the Moon
- If one player takes all 26 penalty points (all hearts + Q♠):
  - Option A: all other players receive 26 points.
  - Option B: the shooter deducts 26 points from their score.
- A high-risk, high-reward strategy.

#### Game End
- Game ends when any player reaches 100 points (or other agreed threshold).
- Player with lowest score wins.

### State Space Analysis

#### Information Sets
- Initial deal: C(52,13) per player.
- After passing: own hand modified, know 3 cards sent to target.
- As play progresses: tricks are public, void suits revealed.

#### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible deals | ~5.36 × 10^28 |
| Passing options | C(13,3) = 286 per player |
| Play sequences | 13 tricks, each 4 plays |
| Game tree nodes | ~10^20-10^30 |
| Information sets | ~10^15-10^20 |

#### Action Space
- **Passing**: choose 3 of 13 cards to pass (286 options).
- **Play**: choose from legal cards per trick (1-13, typically 3-5 options mid-game).
- Leading: any card (except unbroken hearts).

### Key Challenges for AI/Solver Approaches

#### 1. Penalty Avoidance (Inverse Objective)
Unlike most trick-taking games where taking tricks is good, Hearts penalizes taking certain cards:
- The goal is to avoid winning tricks containing hearts or Q♠.
- This inverts standard trick-taking heuristics.
- "Ducking" (playing low to avoid winning) is as important as winning tricks.

#### 2. Queen of Spades Dynamics
The Q♠ (13 points) is the most impactful single card:
- Holders try to dump it on someone else.
- Other players try to avoid winning tricks when Q♠ might appear.
- Creating spade voids early allows dumping Q♠ opportunistically.

#### 3. Shooting the Moon
The shoot-the-moon possibility creates a bifurcation in strategy:
- Must detect when an opponent might be shooting and play to prevent it.
- Must assess when shooting is viable and commit to it early.
- Partial shoot attempts (taking most but not all penalties) are catastrophic.

#### 4. Card Passing Strategy
The 3-card pass is a critical decision:
- Pass high hearts and Q♠ to avoid penalties.
- But also consider: passing creates information (recipient knows what you passed).
- Strategic passing: keep certain suits to maintain control.
- Pass direction matters (left vs right vs across).

#### 5. Multi-Player Score Tracking
Long-term strategy depends on all players' cumulative scores:
- When a player is near 100, others may cooperate to prevent them from busting.
- Score-aware play: sometimes it's beneficial to take some penalty points if it prevents an opponent from shooting the moon.

---

## Part B: Cribbage

### Game Identity

| Property | Value |
|----------|-------|
| Full Name | Cribbage |
| Variants | 5-card cribbage (original), 6-card cribbage (standard), 7-card cribbage, 3-player |
| Players | 2 (standard), 3-4 (variants) |
| Information | Imperfect (hidden crib cards, hidden hand during pegging) |
| Stochasticity | Stochastic (card deals, starter card) |
| Zero-Sum | Yes (race to 121) |
| Solved Status | Unsolved; strong AI exists |

### Rules Summary

#### Setup
- Standard 52-card deck.
- 6 cards dealt to each player (2-player standard).
- Each player discards 2 cards to the "crib" (4 cards total, belonging to the dealer).
- One card cut from the remaining deck as the "starter" (turned face-up).
- If starter is a Jack: dealer pegs 2 points ("his heels" / "nibs").

#### Card Values
- A = 1, 2-10 = face value, J/Q/K = 10.
- For sequences/runs: A-2-3-...-K in rank order.

#### Phase 1: Pegging (The Play)
- Starting with non-dealer, players alternately play cards face-up.
- Running total is maintained (cumulative value of played cards).
- **Running total cannot exceed 31**.
  - If a player cannot play without exceeding 31: "Go" (pass).
  - If opponent also cannot play: "Go" awards 1 point. Count resets to 0.
  - Reaching exactly 31: 2 points.
- Scoring during pegging:
  - **Pair**: playing a card matching the previous card = 2 points.
  - **Pair Royal**: third matching card = 6 points.
  - **Double Pair Royal**: fourth matching card = 12 points.
  - **Run**: 3+ cards in sequence (in any order of play) = points equal to run length.
  - **Fifteen**: running total reaches exactly 15 = 2 points.
  - **Last card**: player of the last card scores 1 point ("go").

#### Phase 2: Show (Counting Hands)
After pegging, both players count their hands (4 cards + starter = 5 cards):
- **Fifteens**: each combination of cards totaling 15 = 2 points.
- **Pairs**: each pair of same-rank cards = 2 points (multiple pairs: 4 for three-of-a-kind, 12 for four-of-a-kind).
- **Runs**: each unique run of 3+ consecutive ranks = points equal to run length.
- **Flush**: 4 cards of same suit in hand = 4 points (5 including starter = 5 points). Crib flush requires all 5 same suit.
- **Nobs**: Jack of same suit as starter in hand = 1 point.

Non-dealer counts first, then dealer counts, then dealer counts crib.
The order matters because the game can end mid-count (reaching 121).

#### Scoring
- Pegged on a cribbage board (track with pegs).
- Game to 121 points (or 61 in some variants).
- "Skunk": winning by 31+ points (double game). "Double skunk": winning by 61+ points.

### State Space Analysis

#### Information Sets
- Initial deal: C(52,6) per player = 20,358,520.
- After discard: C(6,2) = 15 discard options per player.
- Starter card: 1 of 40 remaining cards.
- During pegging: opponent's remaining hand is hidden.

#### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible 6-card hands | ~2.04 × 10^7 |
| Discard options | 15 per player |
| Starter card | 40 possibilities |
| Pegging sequence | ~4-8 plays per pegging round |
| Game tree per hand | ~10^8-10^12 |
| Full game tree (to 121) | ~10^30-10^50 |

#### Action Space
- **Discard**: choose 2 of 6 cards for crib (15 options).
- **Pegging play**: choose 1 of remaining cards (1-4 options, constrained by 31 limit).
- **Counting**: deterministic (no decision, just scoring).

### Key Challenges for AI/Solver Approaches

#### 1. Discard Decision
The 2-card discard is the most strategic decision in cribbage:
- Must optimize own hand for counting while considering:
  - Crib value (positive if dealer, negative if non-dealer).
  - Starter card probability distribution.
  - Cards that work well together (fifteens, runs, pairs).
- The discard affects both counting and pegging phases.

#### 2. Pegging Tactics
Pegging involves card-by-card play with partial information:
- Running total management (approaching 15 or 31).
- Pair and run traps (playing a card that invites opponent to pair it, then playing the third for pair royal).
- Defensive pegging (avoiding playing cards that give opponent scoring opportunities).
- Last-card advantage.

#### 3. Dealer vs Non-Dealer Asymmetry
- Dealer gets the crib (typically worth 4-5 points).
- Non-dealer counts first (can win by reaching 121 before dealer counts).
- Strategies differ by position.

#### 4. Score-Aware Play
As players approach 121:
- Pegging becomes more important (points scored immediately, before counting).
- "Board position" (relationship between player scores and progress) affects whether to play offensively or defensively.
- The "positional" style considers where on the board each player is relative to their expected scoring.

---

## Combined Relevance to Myosu

### Solver Applicability

**Hearts:**
- **CFR**: applicable as a 4-player competitive game, though the penalty-avoidance objective requires inverted evaluation.
- **Monte Carlo simulation**: MC + perfect-information play for evaluating card play decisions.
- **Neural networks**: can learn passing strategies and play policies.
- **Shoot-the-moon detection**: requires specialized threat assessment.

**Cribbage:**
- **Expectation-based discard optimization**: the discard decision can be evaluated via expected value across all possible starter cards.
- **Pegging AI**: requires real-time tactical evaluation of card play sequences.
- **Endgame solving**: pegging decisions near 121 can be solved exactly.
- **Board position modeling**: score-aware play strategies learned via RL or pre-computed tables.

### Architecture Ranking Factors

| Factor | Hearts | Cribbage | Notes |
|--------|--------|----------|-------|
| CFR applicability | 3/5 | 3/5 | Hearts: 4-player complicates; Cribbage: 2-player, moderate |
| Neural value network potential | 4/5 | 4/5 | Both learnable |
| Abstraction necessity | 2/5 | 2/5 | Moderate state spaces |
| Real-time solving value | 4/5 | 3/5 | Hearts: MC simulation; Cribbage: pegging tactics |
| Transferability of techniques | 4/5 | 3/5 | Hearts shares with Spades/Bridge; Cribbage more unique |

### Myosu Subnet Considerations

**Hearts:**
- **Ubiquitous familiarity**: included in Windows since 1992, making it one of the most widely known card games.
- **Penalty avoidance**: tests inverse-objective optimization — different algorithmic challenges than win-maximization.
- **Shared trick-taking infrastructure**: engine shared with Spades (14) and Bridge (10).
- **Shoot-the-moon creates drama**: makes the game spectator-friendly.
- **Passing evaluation**: a clean, measurable decision point for strategy quality.

**Cribbage:**
- **Niche but passionate community**: cribbage has a devoted following, particularly in North America and UK.
- **Discard evaluation**: expected value computation across starter cards provides a ground-truth evaluation.
- **Pegging as unique mechanic**: the running-total, play-and-count mechanic is not found in other games in the list.
- **Board position strategy**: score-aware play is a unique test of long-horizon planning.
- **American Cribbage Congress (ACC)**: organized competitive play provides evaluation standards.

### Recommended Approach for Myosu

**Hearts:**
1. MC simulation with trick-taking solver for card play evaluation.
2. Neural networks for passing and play-from-hand decisions.
3. Shoot-the-moon risk assessment as a specific evaluation metric.
4. Evaluate via average penalty points per hand and shoot-the-moon detection rate.
5. Share trick-taking infrastructure with Spades and Bridge.

**Cribbage:**
1. Expected value tables for discard decisions (pre-computed across all starter cards).
2. RL-trained pegging policy with board-position awareness.
3. Endgame solver for close-to-121 situations.
4. Evaluate via discard EV accuracy and pegging point differential.
5. Use as a unique-mechanic game that diversifies the portfolio beyond trick-taking and shedding.

## Key Papers (Combined)

| Year | Paper | Game | Contribution |
|------|-------|------|--------------|
| 2001 | Sturtevant & Korf, "Selection of Strategies in Hearts" | Hearts | Strategy analysis |
| 2006 | Yan et al., "Computing Strategies for Hearts" | Hearts | MC-based AI |
| 2010 | Various | Cribbage | Discard optimization algorithms |
| 2016 | Silver et al. (AlphaGo) | General | Techniques applicable to both |
| 2019 | Various game AI surveys | Both | State of card game AI |
