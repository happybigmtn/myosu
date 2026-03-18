# Gin Rummy

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Gin Rummy |
| Variants | Standard Gin, Oklahoma Gin, Hollywood Gin, Straight Gin |
| Players | 2 |
| Information | Imperfect (hidden hand, unknown draw pile, discards visible) |
| Stochasticity | Stochastic (card deals, draw pile order) |
| Zero-Sum | Yes |
| Solved Status | Unsolved; strong AI exists but no Nash equilibrium computed |

## Overview

Gin Rummy is a two-player card game from the Rummy family, developed in the early 1900s in the United States. It became hugely popular in the 1940s, particularly in Hollywood. Players draw and discard cards to form melds (sets and runs) while minimizing the point value of unmatched cards (deadwood). The game's distinctive feature is the **knock** mechanic: a player may end the hand before achieving a perfect hand, as long as their deadwood count is sufficiently low. The opponent then has a chance to **lay off** cards against the knocker's melds, creating an undercut dynamic that rewards defensive play.

**Players:** 2
**Deck:** Standard 52-card deck, no jokers
**Objective:** Be the first player to reach the target score (typically 100 points) across multiple hands

## Equipment

- One standard 52-card deck (no jokers)
- Scoresheet or scoring method
- A flat surface

### Card Values (for Deadwood Counting)
| Card | Value |
|------|-------|
| A | 1 point |
| 2-10 | Face value (2 = 2, ..., 10 = 10) |
| J, Q, K | 10 points each |

### Card Rankings
Aces are always low. Cards rank K (high) through A (low). For runs, only A-2-3 is valid at the low end; Q-K-A does not wrap around.

## Setup

### Dealing
1. Determine first dealer by drawing cards; lower card deals (aces low). Alternate dealing each hand thereafter.
2. The dealer deals 10 cards one at a time, alternating, starting with the non-dealer.
3. The 21st card is placed face-up to begin the **discard pile**.
4. The remaining cards are placed face-down as the **stock pile**.

### First Turn Special Rule
The non-dealer may pick up the face-up card (the first discard) or pass. If the non-dealer passes, the dealer may pick it up or pass. If both pass, the non-dealer draws from the stock pile to begin normal play.

## Game Flow

### Turn Structure
Starting with the non-dealer (after the first-turn special rule), players alternate turns. Each turn consists of two actions:

**1. Draw:** Take one card from either:
- The top card of the **stock pile** (face-down, unknown), or
- The top card of the **discard pile** (face-up, known)

**2. Discard:** Place one card from your hand face-up on top of the discard pile.

After discarding, the player may optionally **knock** or **go gin** if their deadwood count qualifies (see Actions).

**Restriction:** A player cannot draw from the discard pile and immediately discard the same card in the same turn.

### Hand End
A hand ends when:
1. A player knocks or goes gin, or
2. The stock pile is reduced to 2 cards and neither player has knocked. The hand is a **draw** (no score, same dealer redeals).

## Actions

### 1. Knock (Deadwood ≤ 10)
After drawing and before discarding, a player may knock if their deadwood (unmatched card total) would be 10 points or less after discarding one card.

**Knock procedure:**
1. Discard one card face-down on the discard pile (signaling the knock).
2. Lay out your hand face-up, organized into melds and deadwood.
3. The opponent then lays out their hand, organizing into melds.
4. The opponent may **lay off** (see below) unmatched cards onto the knocker's melds.
5. Compare deadwood counts.

**Scoring after a knock:**
- If the knocker has lower deadwood: **Knocker wins.** Score = opponent's deadwood minus knocker's deadwood.
- If the opponent has equal or lower deadwood after lay-offs: **Undercut.** Opponent wins. Score = knocker's deadwood minus opponent's deadwood + **25-point undercut bonus** (awarded to the opponent).

### 2. Gin (Deadwood = 0)
If a player can arrange all 10 cards (after discarding one) into melds with zero deadwood, they declare **gin**.

**Gin bonus:**
- The gin player scores the opponent's total deadwood + **25-point gin bonus**.
- The opponent **cannot lay off** against a gin hand.

### 3. Big Gin (Deadwood = 0 with 11 cards)
If a player draws a card that completes all 11 cards (hand of 10 + drawn card) into melds with zero deadwood, they may declare Big Gin **without discarding**.

**Big Gin bonus:**
- Score = opponent's deadwood + **31-point big gin bonus** (some rule sets use 25 or 50).
- No lay-offs permitted.
- Big Gin is optional in many rule sets.

### 4. Lay Off
When the opponent (defender) responds to a knock:
- The defender organizes their own melds.
- The defender may add their unmatched cards to the **knocker's melds**, extending sets or runs. For example, if the knocker shows 7-8-9 of spades, the defender may lay off the 6 of spades or 10 of spades.
- Laid-off cards are removed from the defender's deadwood count.
- Lay-offs are **not permitted** against a gin hand.
- A defender cannot lay off cards to create new melds or extend their own melds with the knocker's cards.

## Melds

### Sets (Groups)
Three or four cards of the **same rank** but different suits.
- 7H-7D-7C (valid 3-card set)
- 7H-7D-7C-7S (valid 4-card set)

### Runs (Sequences)
Three or more **consecutive cards of the same suit**.
- 3H-4H-5H (valid 3-card run)
- 3H-4H-5H-6H-7H (valid 5-card run)
- A-2-3 of any suit (valid; ace is low)
- Q-K-A is **not valid** (ace cannot be high)

### Meld Rules
- A card can only belong to one meld. If a card could belong to either a set or a run, the player chooses the more advantageous assignment.
- There is no requirement to meld all possible cards. A player may keep cards unmelded strategically (though this increases deadwood).

## Scoring

### Hand Scoring

| Outcome | Points Awarded To | Amount |
|---------|-------------------|--------|
| **Knock (knocker wins)** | Knocker | Opponent's deadwood - Knocker's deadwood |
| **Undercut** | Defender | Knocker's deadwood - Defender's deadwood + 25 bonus |
| **Gin** | Gin player | Opponent's deadwood + 25 bonus |
| **Big Gin** | Big Gin player | Opponent's deadwood + 31 bonus |
| **Draw (stock exhausted)** | Neither | 0 (hand is void) |

### Game Scoring

The game ends when a player reaches the target score (typically **100 points**). The winner receives:
1. The point difference between the two players' scores.
2. A **game bonus** of 100 points.
3. A **box bonus** (also called "line bonus") of 25 points for each hand won.

**Example:** Player A wins with 115 points, having won 6 hands. Player B has 80 points, having won 4 hands.
- Point difference: 115 - 80 = 35
- Game bonus: 100
- Box bonus: (6 - 4) x 25 = 50
- Total: 35 + 100 + 50 = **185 points to Player A**

### Hollywood Scoring (Three-Game Variant)
In Hollywood scoring, three games are tracked simultaneously. A player's first hand win counts only in Game 1. Their second hand win counts in Games 1 and 2. Their third hand win (and all subsequent) counts in Games 1, 2, and 3.

**How it works:**
1. Draw three columns on the scoresheet (Game 1, Game 2, Game 3).
2. When a player wins their first hand, record the score in Game 1 only.
3. When a player wins their second hand, record the score in Games 1 and 2.
4. Third and subsequent hand wins go into all three games.
5. Each game ends independently when either player reaches 100 in that game.
6. Apply game bonus and box bonus to each game separately.

This effectively triples the action per session and rewards consistent play.

### Oklahoma Scoring Variant
In Oklahoma Gin, the first face-up card (the initial discard) determines the maximum deadwood allowed for knocking in that hand:
- If the upcard is a face card (J, Q, K) or 10: knock threshold is 10 (standard rules).
- If the upcard is a number card (2-9): the knock threshold equals the card's face value (e.g., upcard is 4, must have 4 or less deadwood to knock).
- If the upcard is an Ace: only gin is allowed (no knocking with deadwood).
- If the upcard is a spade: all scores for that hand are doubled.

## Winning Conditions

### Game Victory
The first player to accumulate 100 or more points (across multiple hands) wins the game. Final scoring (game bonus, box bonuses) is then calculated.

### Match Victory (Optional)
Some play a "match" as a series of games (e.g., best of 3 games). Each game is scored independently.

### Shutout / Schneider (Optional)
If the losing player has not won a single hand when the game ends, the winner's game bonus is doubled (200 instead of 100 in some rule sets) or the entire game score is doubled.

## Special Rules

### Stock Pile Exhaustion
When only 2 cards remain in the stock pile, the hand is void — no score is awarded. The same dealer redeals.

### Drawing from the Discard Pile
The discard pile is never shuffled back into the stock. Only the top card of the discard pile is ever available.

### Looking at Previous Discards
Players are **not** allowed to look through the discard pile. Only the top card is visible. Memorizing previously discarded cards is a key skill.

### Dead Cards
Once a card is buried in the discard pile (no longer on top), it is effectively dead for the remainder of the hand.

### Calling Attention to Errors
If a player draws out of turn or exposes a card, the opponent may choose to accept the irregularity or require a correction, depending on house rules.

## Variants

### Straight Gin (No Knocking)
Players can only end the hand by going gin (zero deadwood). No knocking is permitted. This dramatically changes strategy, making hands longer and emphasizing complete hand construction.

### Tedesco Gin
After the initial deal, the non-dealer may "pass" the upcard. If both players pass, the non-dealer draws from the stock. Additionally, the game target is often 200 points instead of 100.

### Partnership Gin (Four Players)
Two teams of two players each play separate two-player games. Scores are combined at the end of each hand.

### Around-the-Corner
Runs may wrap around: Q-K-A-2-3 is a valid run. This significantly changes meld possibilities.

## Key Strategic Concepts

### Deadwood Management
Minimize high-value deadwood cards (10-pointers: face cards, tens). Early discards should eliminate high-value cards that don't contribute to melds. Holding low cards (aces, twos, threes) as deadwood is much cheaper.

### Drawing Strategy
- **Stock vs. Discard:** Drawing from the discard pile reveals information to your opponent (they see what you took and can infer your melds). Drawing from stock keeps your hand hidden but is unpredictable.
- Only take from the discard pile if the card clearly fits into your hand plan.

### Card Tracking
Since only 52 cards exist and you can see your 10 cards plus all discards, tracking which cards remain in the stock pile and opponent's hand is critical. If 3 jacks have been discarded, the 4th jack is worthless for a set. If several cards of a suit sequence are gone, a run in that suit is impossible.

### Knock Timing
- Knock early with a low deadwood count to catch your opponent with high deadwood.
- Avoid knocking with 8-10 deadwood unless confident the opponent has more.
- Knocking with high deadwood risks undercut — the 25-point bonus penalty is steep.

### Defensive Play
- Watch opponent's draws from the discard pile to infer their melds.
- Discard cards that are unlikely to help your opponent (cards adjacent to or matching their known interests).
- Hold "safe" discards for later rounds when the opponent is close to knocking.
- Discard high cards early to limit your exposure if the opponent knocks before you're ready.

### Speculative Holding
Sometimes keeping a card that doesn't immediately meld but has multiple potential (can form a set OR a run) is better than discarding it for a card that only fits one way.

### Undercut Defense
If you suspect the opponent will knock soon, prioritize reducing your own deadwood even at the expense of meld building. An undercut + 25-point bonus is worth more than most hands.

## Common Terminology

| Term | Definition |
|------|------------|
| **Deadwood** | Cards not part of any meld; their point values are totaled |
| **Meld** | A valid combination: a set (3-4 of a kind) or a run (3+ consecutive same-suit) |
| **Set / Group** | Three or four cards of the same rank |
| **Run / Sequence** | Three or more consecutive cards of the same suit |
| **Knock** | Ending the hand by discarding face-down when deadwood is 10 or less |
| **Gin** | Knocking with zero deadwood; earns a bonus |
| **Big Gin** | All 11 cards (10 + drawn) form melds; declared without discarding |
| **Undercut** | Defender has equal or lower deadwood than the knocker; earns a bonus |
| **Lay off** | Defender adding unmatched cards to the knocker's melds to reduce deadwood |
| **Stock pile** | The face-down draw pile |
| **Discard pile** | The face-up pile where discarded cards are placed |
| **Upcard** | The initial face-up card dealt to start the discard pile |
| **Box bonus / Line bonus** | 25 points per hand won, added at game end |
| **Game bonus** | 100 points awarded to the game winner |
| **Hollywood scoring** | Three simultaneous games tracked on one scoresheet |
| **Oklahoma** | Variant where the upcard determines the knock threshold |
| **Schneider / Shutout** | Winning without the opponent having won any hand |
| **Going to the wall** | Drawing from the stock pile (as opposed to the discard pile) |

## State Space Analysis

### Information Sets
- Initial deal: C(52,10) = 15,820,024,220 possible hands per player.
- With known discard history: information narrows as game progresses.
- Key hidden information: opponent's hand, remaining stock cards.
- As cards are drawn from stock and discarded, both players gain information about remaining cards.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible initial hands | ~1.58 × 10^10 |
| Turns per game | ~10-20 (typical) |
| Draw decision per turn | 2 (discard pile or stock) |
| Discard decision per turn | 10-11 (cards in hand) |
| Knock decision per turn | Binary (if eligible) |
| Game tree nodes | ~10^20-10^30 |
| Information sets | ~10^15-10^20 |

### Action Space
- **Draw phase**: binary choice (discard pile top card vs stock).
- **Discard phase**: choose 1 of 10-11 cards.
- **Knock decision**: binary (knock or continue), subject to deadwood threshold.
- **Lay-off phase** (opponent of knocker): which cards to lay off on knocker's melds.
- Effective branching factor: ~22 per turn (2 × 11) + knock decision.

## Key Challenges for AI/Solver Approaches

### 1. Draw Source Decision
Whether to draw from the discard pile (public, known card) or stock (private, unknown card) is a critical decision revealing information:
- Drawing from the discard pile reveals that you want that specific card (or want to prevent opponent from getting it).
- Drawing from stock reveals nothing.
- Deceptive play: sometimes drawing from the discard pile to mislead.

### 2. Card Tracking and Inference
A strong player tracks:
- All discards (visible to both players).
- Which cards opponent has drawn from the discard pile (reveals preferences).
- Cards drawn from stock are unknown, but the stock's composition can be inferred.
- Opponent's likely meld structure based on draw/discard patterns.

### 3. Knock vs. Continue Decision
When eligible to knock, the decision involves:
- Current deadwood differential (estimated, since opponent's hand is hidden).
- Risk of opponent reaching Gin if you continue.
- Risk of undercut if you knock with marginal deadwood.
- Number of remaining stock cards (endgame pressure).

### 4. Deadwood Minimization vs. Gin Pursuit
Two strategic poles:
- Conservative: minimize deadwood, knock early, avoid undercuts.
- Aggressive: pursue Gin for the bonus, risking that opponent knocks first.
- Optimal strategy depends on score differential, remaining stock, and opponent tendencies.

### 5. Lay-Off Decisions
When the opponent knocks, deciding which cards to lay off can be non-trivial:
- Laying off a card from a potential meld in your hand may increase your deadwood.
- Must evaluate the net effect on deadwood count.

## Known Solver Results

### Academic Work
- **Dahl (2001)**: one of the first competitive Gin Rummy AI programs.
- **GinRummyAI competition** (various): annual competitions at AAAI and other venues.
- No published Nash equilibrium for full Gin Rummy.
- CFR-based approaches have been explored for simplified variants.

### AI Approaches
- **Rule-based**: strong heuristic players exist, using:
  - Deadwood minimization.
  - Card counting.
  - Knock threshold optimization.
- **Monte Carlo simulation**: sample opponent hands consistent with observations, evaluate each action by simulation.
- **Reinforcement learning**: Q-learning and policy gradient methods applied with moderate success.
- **Deep RL**: neural network-based approaches have been explored but not published at top venues.

### AAAI Gin Rummy Competition
- Annual computer Gin Rummy tournament since 2019.
- Best agents use a mix of CFR for simplified game models, MC simulation, and heuristic evaluation.
- No agent has demonstrated definitively superhuman play.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2001 | Dahl, "A Gin Rummy Playing Program" | Early competitive AI |
| 2019 | AAAI Gin Rummy Competition | Standardized evaluation framework |
| 2020 | Seitz et al., "MCCFR for Gin Rummy" | CFR variant for gin rummy |
| 2021 | Various competition entries | Deep RL approaches |

## Relevance to Myosu

### Solver Applicability
Gin Rummy tests solver performance on **sequential draw-discard games** with information leakage:
- **CFR**: applicable as a 2-player zero-sum game, but the game tree size requires significant abstraction (card grouping, game phase abstraction).
- **Monte Carlo simulation**: the primary practical approach — sample unknown cards, evaluate actions.
- **Card counting models**: tracking discard history and opponent draw patterns is essential and provides a testbed for inference algorithms.
- **Neural networks**: can learn draw/discard policies and knock timing from self-play.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 3/5 | Feasible with abstraction; moderate state space |
| Neural value network potential | 4/5 | Rich state representation from card tracking |
| Abstraction necessity | 3/5 | Some needed for tractability |
| Real-time solving value | 3/5 | MC simulation effective |
| Transferability of techniques | 4/5 | Draw-discard mechanics shared with other rummy variants |

### Myosu Subnet Considerations
- **Widespread familiarity**: Gin Rummy is widely known and played, particularly in North America.
- **Moderate complexity**: easier than NLHE but harder than trivial games — good mid-tier benchmark.
- **Information leakage modeling**: the way draw/discard decisions leak information is a useful testbed for inference capabilities.
- **Game oracle**: hand evaluation (meld detection, deadwood calculation) is deterministic and well-specified. Lay-off resolution requires careful implementation.
- **Deterministic verification**: given a complete game record, the oracle can verify all scoring deterministically.
- **Knock decision quality**: the knock/continue decision is a clean, measurable strategic choice for evaluation.

### Recommended Approach for Myosu
1. MC simulation with card counting as the primary approach.
2. CFR for pre-computing blueprint strategies on abstracted game trees.
3. Neural networks for draw source and discard selection.
4. Evaluate strategies on knock timing quality and overall win rate.
5. Use Gin Rummy as a gateway game for solver operators familiar with traditional card games.
