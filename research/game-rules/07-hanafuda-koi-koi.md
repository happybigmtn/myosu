# Hanafuda Koi-Koi (花札 こいこい)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Koi-Koi (こいこい), played with Hanafuda (花札, "flower cards") |
| Variants | Koi-Koi, Hana-Awase, Hachi-Hachi, Sakura, Sudda |
| Players | 2 (Koi-Koi); 3 (Hana-Awase); up to 7 (Hachi-Hachi) |
| Information | Imperfect (hidden hand cards, unknown draw pile) |
| Stochasticity | Stochastic (card deals, draw pile) |
| Zero-Sum | Yes (2-player Koi-Koi) |
| Solved Status | Unsolved; limited academic study |

## Overview

Koi-Koi is a two-player Japanese card game played with Hanafuda ("flower cards"), a traditional 48-card deck organized by the twelve months of the year. Players capture cards from a central tableau by matching them with cards from their hand, aiming to form scoring combinations called yaku. The name "Koi-Koi" (こいこい, "come come") refers to the decision a player makes after completing a yaku: stop and collect points, or call "Koi-Koi" to continue playing for a higher score at the risk of the opponent completing their own yaku first.

**Players:** 2
**Deck:** 48-card Hanafuda deck (12 suits of 4 cards each)
**Objective:** Score the most points across a series of rounds by forming yaku from captured cards

## Equipment

### The Hanafuda Deck (48 Cards)
Each of the 12 suits represents a month and is identified by a plant or flower. Each suit contains exactly 4 cards, divided into the following types:

| Month | Plant (Japanese) | Bright (20 pt) | Animal/Seed (10 pt) | Ribbon (5 pt) | Plain (1 pt) |
|-------|-----------------|-----------------|---------------------|---------------|---------------|
| **January** | Pine (松 Matsu) | **Crane** (松に鶴) | | Red Poetry Ribbon (赤短) | Plain x2 |
| **February** | Plum Blossom (梅 Ume) | | Bush Warbler (梅に鶯) | Red Poetry Ribbon (赤短) | Plain x2 |
| **March** | Cherry Blossom (桜 Sakura) | **Curtain** (桜に幕) | | Red Poetry Ribbon (赤短) | Plain x2 |
| **April** | Wisteria (藤 Fuji) | | Cuckoo (藤に不如帰) | Plain Red Ribbon | Plain x2 |
| **May** | Iris (菖蒲 Ayame) | | Eight-Plank Bridge (菖蒲に八橋) | Plain Red Ribbon | Plain x2 |
| **June** | Peony (牡丹 Botan) | | Butterflies (牡丹に蝶) | Blue Ribbon (青短) | Plain x2 |
| **July** | Bush Clover (萩 Hagi) | | Boar (萩に猪) | Plain Red Ribbon | Plain x2 |
| **August** | Pampas Grass (芒 Susuki) | **Full Moon** (芒に月) | Geese (芒に雁) | | Plain x2 |
| **September** | Chrysanthemum (菊 Kiku) | | **Sake Cup** (菊に盃) | Blue Ribbon (青短) | Plain x2 |
| **October** | Maple (紅葉 Momiji) | | Deer (紅葉に鹿) | Blue Ribbon (青短) | Plain x2 |
| **November** | Willow (柳 Yanagi) | **Rain Man** (柳に小野道風) | Swallow (柳に燕) | Red Ribbon (柳に短冊) | Lightning (柳に雷) |
| **December** | Paulownia (桐 Kiri) | **Phoenix** (桐に鳳凰) | | | Plain x3 |

### Card Type Totals
- **Bright (光 Hikari):** 5 cards (Crane, Curtain, Full Moon, Rain Man, Phoenix)
- **Animal/Seed (種 Tane):** 9 cards
- **Ribbon (短冊 Tanzaku):** 10 cards (3 Red Poetry, 3 Blue, 3 Plain Red, 1 Willow Red)
- **Plain/Chaff (カス Kasu):** 24 cards

### Special Card: Sake Cup (September Chrysanthemum)
The Sake Cup is categorized as an Animal card (10 pt) but in many rule sets it **also** counts as a Plain card simultaneously. This means it can contribute to both Animal-based yaku and Plain-based yaku.

## Setup

### Dealing
1. Determine the first dealer (Oya, 親) by each player drawing a card; the player drawing the card of the earlier month becomes dealer. If from the same month, draw again.
2. The dealer shuffles and deals as follows:
   - 8 cards face-down to the opponent
   - 8 cards face-up to the table (the field / 場 Ba)
   - 8 cards face-down to the dealer
   - The remaining 24 cards form the face-down draw pile (山札 Yamafuda)
3. Cards may be dealt in groups of 2 or 4 at a time rather than one at a time, depending on house rules.

### Checking for Misdeals
Before play begins, check for special conditions:
- If all four cards of a single month are on the field, the dealer collects all four immediately.
- If three cards of a single month are on the field, they are stacked together. The player who plays the fourth matching card captures all four.
- Some rule sets call for a redeal if the field contains four pairs (8 cards, all paired), as this leaves no capture opportunities.

### Who Goes First
The non-dealer (子 Ko) plays first.

## Game Flow

### Round Structure
A game consists of a fixed number of rounds (typically 12, one per month, or 6, or 3). After each round, the dealer role alternates. At the end of all rounds, the player with the higher total score wins.

### Turn Structure
On each turn, a player performs two actions in sequence:

**Step 1 — Play a card from hand:**
- Select one card from your hand and play it to the field.
- If it matches the month of one card on the field, place it on top of that field card to capture both. Move the pair to your scoring area.
- If it matches two cards on the field, choose which one to capture.
- If it matches three cards on the field (rare, since the fourth is in your hand), you capture all three plus your played card.
- If it matches nothing, the card stays on the field as a new available card.

**Step 2 — Draw from the deck and resolve:**
- Draw the top card from the draw pile and reveal it.
- If it matches the month of one field card, capture both and add them to your scoring area.
- If it matches two field cards, choose which one to capture.
- If it matches three field cards, capture all three plus the drawn card.
- If it matches nothing, it remains on the field.

**Step 3 — Check for Yaku:**
- After both captures, check if you have completed any new yaku with your captured cards.
- If you have a yaku, you must decide: **Stop** (end the round and score) or **Koi-Koi** (continue playing for more points).
- If you call Koi-Koi, the round continues. If you later complete additional yaku, you may stop or call Koi-Koi again.
- If you call Koi-Koi and your opponent completes a yaku before you can stop, your opponent scores and your yaku are worthless.

### Round End Conditions
1. **A player calls Stop:** That player scores all their completed yaku.
2. **All cards exhausted (Exhaustive draw):** If both players exhaust all cards from their hands and neither has called Stop, the round is a draw (no points awarded). Some rule sets award points to the dealer or the player with more captured card points in this case.

## Scoring — Yaku (役)

Yaku are scoring combinations formed from captured cards. Multiple yaku can be scored simultaneously. All applicable yaku are summed.

### Bright Yaku (光 — mutually exclusive; score only the highest qualifying)

| Yaku | Cards Required | Points |
|------|---------------|--------|
| **Goko** (五光 / Five Brights) | All 5 Bright cards | 15 |
| **Shiko** (四光 / Dry Four Brights) | Any 4 Brights, NOT including Rain Man | 10 |
| **Ame-Shiko** (雨四光 / Rainy Four Brights) | Rain Man + any 3 other Brights | 8 |
| **Sanko** (三光 / Three Brights) | Any 3 Brights, NOT including Rain Man | 6 |

### Animal/Seed Yaku (種)

| Yaku | Cards Required | Points |
|------|---------------|--------|
| **Inoshikacho** (猪鹿蝶 / Boar-Deer-Butterfly) | Boar (Jul) + Deer (Oct) + Butterflies (Jun) | 5 (+1 per additional Animal) |
| **Tane** (タネ / Animals) | Any 5 Animal cards | 1 (+1 per additional Animal beyond 5) |

Inoshikacho and Tane stack: if you have the three Inoshikacho cards plus 2 more Animals, you score 5 (Inoshikacho) + 1 (Tane for 5 Animals) = 6.

### Ribbon Yaku (短冊)

| Yaku | Cards Required | Points |
|------|---------------|--------|
| **Akatan-Aotan no Chofuku** (赤短・青短の重複) | All 3 Red Poetry Ribbons + all 3 Blue Ribbons | 10 (+1 per additional Ribbon) |
| **Akatan** (赤短 / Red Poetry Ribbons) | Pine (Jan) + Plum (Feb) + Cherry (Mar) red poetry ribbons | 5 (+1 per additional Ribbon) |
| **Aotan** (青短 / Blue Ribbons) | Peony (Jun) + Chrysanthemum (Sep) + Maple (Oct) blue ribbons | 5 (+1 per additional Ribbon) |
| **Tanzaku** (短冊 / Ribbons) | Any 5 Ribbon cards | 1 (+1 per additional Ribbon beyond 5) |

Akatan, Aotan, and Tanzaku all stack with each other. The combined yaku (Akatan-Aotan no Chofuku) stacks with Tanzaku as well.

### Plain/Chaff Yaku (カス)

| Yaku | Cards Required | Points |
|------|---------------|--------|
| **Kasu** (カス / Plains) | 10 or more Plain cards | 1 (+1 per additional Plain beyond 10) |

If the Sake Cup counts as both Animal and Plain (per house rules), it contributes to both Tane and Kasu simultaneously.

### Viewing Yaku (花見・月見 — Optional, commonly played)

| Yaku | Cards Required | Points |
|------|---------------|--------|
| **Hanami-zake** (花見酒 / Blossom Viewing) | Curtain (Mar Bright) + Sake Cup (Sep Animal) | 5 |
| **Tsukimi-zake** (月見酒 / Moon Viewing) | Full Moon (Aug Bright) + Sake Cup (Sep Animal) | 5 |

These are widely used but considered optional. Some rule sets disable them or reduce their value. When both are achieved simultaneously, they are typically worth 5 + 5 = 10 (some rule sets make this combination worth a flat 10 instead of stacking).

### Scoring Multiplier: Koi-Koi Doubling
Many rule sets apply a doubling rule:
- If a player's final point total for the round is **7 or more**, the score is **doubled**.
- If the winner's opponent had previously called Koi-Koi during this round, the score is **doubled** again (for a total of 4x if both conditions apply).

### Monthly Multiplier (Optional)
Some rule sets multiply the round score by the current month number (Round 1 = x1, Round 12 = x12), increasing stakes as the game progresses.

## Winning Conditions

### Round Victory
A player wins the round by calling "Stop" after completing at least one yaku. They score the sum of all their completed yaku, with applicable multipliers.

### Game Victory
After the agreed number of rounds, the player with the highest total score wins. If scores are tied, the player who was dealer in the final round typically loses (opponent wins), though tie-breaking rules vary.

### Special: Empty Hand Win
If a player captures all cards played from their hand (all 8 cards matched and captured), some rule sets award a bonus or automatic round win.

## Special Rules

### Koi-Koi Risk/Reward
Calling Koi-Koi is the defining decision of the game. After completing a yaku:
- **Stop:** Safely collect points for all current yaku. End the round.
- **Koi-Koi:** Continue playing. Your score can grow as you complete more yaku, but if your opponent completes a yaku first and calls Stop, you score nothing for the round (and your opponent may receive a bonus for your Koi-Koi call).

A player may call Koi-Koi multiple times in a single round. Each Koi-Koi call increases risk but may also trigger a doubling bonus.

### Teshi and Kuttsuki (Hand-Based Misdeals)
- **Teshi (手四):** If a player is dealt all four cards of any single month in their hand, the round is typically redealt (or the player automatically wins, depending on house rules).
- **Kuttsuki (くっつき):** If a player's entire hand consists of paired months (e.g., 4 months x 2 cards each), the round is redealt (some rule sets award a win instead).

### Lucky Hands (House Rules)
Some families play with special instant-win hands dealt at the start, such as holding all four of a prestigious month or receiving a certain number of Brights.

## Key Strategic Concepts

### Capture Priority
Not all field cards are equally valuable. Prioritize capturing high-value cards (Brights, key Animals, Poetry/Blue Ribbons) even at the cost of leaving lower-value cards for your opponent. The Sake Cup is especially versatile if it counts toward both Animal and Plain yaku.

### Field Management
When a card from your hand matches nothing on the field, it becomes available for your opponent to capture. Avoid placing high-value cards on the field if possible. If forced, place a card that is less likely to contribute to your opponent's yaku progress.

### Koi-Koi Timing
Call Koi-Koi when:
- Your current yaku score is low (1-2 points) and you have strong prospects for more yaku.
- Your opponent appears far from completing any yaku.
- The doubling threshold (7+ points) is within reach.

Call Stop when:
- Your score is already high or at the doubling threshold.
- Your opponent is close to completing a yaku.
- Few cards remain in the draw pile (less opportunity to improve).

### Card Counting
With only 48 cards total and full visibility of the field and your own captures, tracking which cards of each month have appeared reveals exactly what remains in the draw pile and opponent's hand. This is the primary skill ceiling.

### Yaku Stacking
Multiple yaku within the same category stack (e.g., Akatan + Aotan + Tanzaku). Plan captures to achieve multiple simultaneous yaku rather than stopping at the first completed combination.

## Common Terminology

| Term (Japanese) | Romaji | English | Definition |
|----------------|--------|---------|------------|
| 花札 | Hanafuda | Flower Cards | The 48-card deck |
| こいこい | Koi-Koi | Come Come | Decision to continue playing after completing a yaku |
| 役 | Yaku | Hand / Combination | A scoring combination of captured cards |
| 場 | Ba | Field | The face-up cards in the center of the table |
| 山札 | Yamafuda | Draw Pile | The face-down draw pile |
| 親 | Oya | Dealer / Parent | The current dealer |
| 子 | Ko | Non-dealer / Child | The non-dealer |
| 光 | Hikari | Bright / Light | The 5 highest-value cards (20 pt each) |
| 種 | Tane | Seed / Animal | The 9 mid-value cards (10 pt each) |
| 短冊 | Tanzaku | Ribbon / Strip | The 10 ribbon cards (5 pt each) |
| カス | Kasu | Chaff / Plain / Dregs | The 24 lowest-value cards (1 pt each) |
| 赤短 | Akatan | Red Poetry | The three red poetry ribbon cards (Jan/Feb/Mar) |
| 青短 | Aotan | Blue | The three blue ribbon cards (Jun/Sep/Oct) |
| 猪鹿蝶 | Inoshikacho | Boar-Deer-Butterfly | The three specific Animal cards |
| 五光 | Goko | Five Brights | All 5 Bright cards captured |
| 三光 | Sanko | Three Brights | Any 3 non-Rain Bright cards |
| 花見酒 | Hanami-zake | Blossom Viewing Sake | Curtain + Sake Cup yaku |
| 月見酒 | Tsukimi-zake | Moon Viewing Sake | Full Moon + Sake Cup yaku |
| 手四 | Teshi | Four-in-Hand | Holding all 4 cards of one month |
| くっつき | Kuttsuki | All Paired | Hand of entirely paired months |

## State Space Analysis

### Information Sets
- Initial deal: each player knows their 8 cards + 8 field cards. 24 cards unknown (in draw pile or opponent's hand).
- As play progresses, captures reveal cards, narrowing the uncertainty.
- Effective information sets: player's hand × field state × opponent's captured cards × draw pile composition (inferred).

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Deck combinations | C(48,8) × C(40,8) × arrangement = enormous |
| Cards per hand | 8 (decreasing each turn) |
| Turns per round | 8 per player (16 total) |
| Branching factor per turn | 1-8 (hand card choice) × 1-2 (field match choice) + draw randomness |
| Game tree nodes (single round) | ~10^15-10^20 (estimated) |
| Information sets | ~10^12-10^15 |

### Action Space
- **Hand play**: choose 1 of up to 8 cards from hand.
- **Field match choice**: when multiple field cards match the played/drawn card, choose which to capture.
- **Koi-Koi decision**: binary (stop/continue) when a yaku is completed.
- Relatively small branching factor per decision, but the stochastic draw adds complexity.

## Key Challenges for AI/Solver Approaches

### 1. Complex Scoring System
The yaku system creates non-trivial scoring combinatorics. The solver must track progress toward multiple overlapping scoring combinations simultaneously.

### 2. Koi-Koi Risk-Reward Tradeoff
The central decision — whether to call Koi-Koi — involves:
- Current score vs potential additional score.
- Opponent's visible captured cards (indicator of their yaku progress).
- Remaining cards in hand and estimated draw pile composition.
- The doubling penalty for the opponent if they complete a yaku after you call Koi-Koi.

### 3. Imperfect Information about Draw Pile
The draw pile order is unknown, creating stochastic uncertainty. The solver must reason probabilistically about future draws while accounting for cards visible in the field and captures.

### 4. Card Counting and Inference
As cards are played and captured, both players gain information about the remaining cards. A strong player (or solver) tracks which cards remain and infers the opponent's hand composition.

### 5. Multi-Month Games
Koi-Koi is traditionally played over 6 or 12 rounds (months). Cumulative score across rounds adds a meta-game element — risk tolerance may change based on overall score differential.

## Known Solver Results

### Academic Work
- Limited published research on Hanafuda AI.
- Some Japanese university papers on Koi-Koi AI using Monte Carlo Tree Search (MCTS) and heuristic evaluation.
- No published Nash equilibrium computation.
- The game's moderate state space suggests that approximate equilibrium computation is feasible with modern techniques.

### AI Implementations
- Several open-source Koi-Koi AI implementations exist, mostly using:
  - Rule-based heuristics (play toward best yaku).
  - MCTS with random playouts.
  - Minimax with heuristic evaluation (treating unknown cards probabilistically).
- No known implementation using CFR or neural-guided search.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2010 | Kato & Takeuchi (Japanese) | MCTS for Koi-Koi |
| 2015 | Various Japanese CS proceedings | Heuristic Koi-Koi AI |
| 2018 | Takizawa et al. | Evaluation function design for Hanafuda |

## Relevance to Myosu

### Solver Applicability
Koi-Koi offers a **unique game structure** not found in Western card games:
- **CFR**: applicable to the 2-player zero-sum version. The moderate state space (much smaller than NLHE) makes CFR feasible without extreme abstraction.
- **MCTS with information set sampling**: natural fit for the stochastic draw pile.
- **Heuristic value functions**: the yaku system lends itself to feature-based evaluation (progress toward each yaku, blocking opponent yaku).

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 4/5 | 2-player zero-sum, moderate state space |
| Neural value network potential | 4/5 | Rich feature space from yaku progress |
| Abstraction necessity | 2/5 | Game may be tractable without heavy abstraction |
| Real-time solving value | 3/5 | MCTS viable for real-time play |
| Transferability of techniques | 3/5 | Unique mechanics, but general techniques apply |

### Myosu Subnet Considerations
- **Cultural reach**: Hanafuda is deeply rooted in Japanese gaming culture. Including it broadens myosu's appeal to the Japanese market.
- **Moderate complexity**: easier to solve than poker, harder than solved games — good intermediate target.
- **Yaku verification oracle**: the scoring system is deterministic and can be verified on-chain.
- **Rule variation challenge**: many house rules exist for Koi-Koi. The subnet must specify an exact rule set for consistent evaluation.
- **Visual appeal**: Hanafuda's beautiful artwork and seasonal themes create opportunities for engaging UI/UX.
- **Novelty**: no commercial or academic Koi-Koi solver exists at a high level, creating an opportunity for myosu to be first.

### Recommended Approach for Myosu
1. Implement CFR for 2-player Koi-Koi with exact card tracking.
2. Use information set MCTS for real-time play with stochastic draw pile.
3. Feature-based neural evaluation incorporating yaku progress vectors.
4. Standardize on a specific Koi-Koi rule set (e.g., Nintendo standard rules) for consistent evaluation.
5. Use as a showcase game demonstrating myosu's breadth beyond Western card games.
