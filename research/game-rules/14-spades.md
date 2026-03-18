# Spades

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Spades |
| Variants | Partnership (4-player), Cutthroat (3-4 player individual), Solo Spades, Mirror Spades, Suicide Spades |
| Players | 4 (two partnerships) standard; 3-6 in variants |
| Information | Imperfect (hidden hands; bidding signals) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (between partnerships) |
| Solved Status | Unsolved; limited academic study |

## Overview

Spades is a trick-taking card game for four players in fixed partnerships, devised in the United States in the 1930s. It combines elements of Bridge-style bidding with a fixed trump suit (spades are always trump). Spades became widely popular through military and college play and is now one of the most played partnership card games in America.

- **Players:** 4 (in two partnerships; partners sit across from each other)
- **Duration:** 30--60 minutes (to 500 points)
- **Objective:** Win at least the number of tricks your partnership bid, while avoiding excessive overtricks (bags).

## Equipment

- One standard 52-card deck (no jokers)
- Score sheet or scoring app

### Card Rankings

Within each suit, cards rank from high to low:

**A, K, Q, J, 10, 9, 8, 7, 6, 5, 4, 3, 2**

Spades are always trump and beat any card of any other suit.

## Setup

1. Choose partnerships. Partners sit across from each other.
2. Select a first dealer (any method). Deal rotates clockwise after each hand.
3. The dealer shuffles and deals the entire deck, one card at a time, clockwise, starting to the left of the dealer.
4. Each player receives **13 cards**.

## Game Flow

Each hand consists of two phases: **Bidding** and **Play**.

### Phase 1: Bidding

Starting with the player to the dealer's left and proceeding clockwise, each player bids exactly once. A bid is a number from 0 to 13 representing the number of tricks that player expects to win.

- Bids are individual, but partner bids are **combined** for scoring purposes.
- There is exactly one round of bidding (no second bids, no communication between partners about bid strategy).
- A player may not pass -- every player must bid a number.

#### Special Bids

**Nil (bid of 0):** The player predicts they will win zero tricks. Scored separately from the partner's bid (see Scoring).

**Blind Nil:** A player declares Nil **before looking at their cards**. After bidding Blind Nil:
- The player looks at their hand.
- The Blind Nil bidder and their partner each exchange **2 cards** (pass 2 face-down, receive 2 from partner).
- Blind Nil is typically only allowed when the team is losing by 100+ points.

### Phase 2: Play

#### Leading

The player to the dealer's left leads the first trick. **Spades may not be led on the first trick** unless the leader has nothing but spades.

#### Following Suit

Each player, clockwise, plays one card to the trick:

1. **Must follow suit** if able (play a card matching the led suit).
2. If unable to follow suit, may play **any card**, including a spade (trump).

#### Winning a Trick

- If no spades were played, the highest card of the led suit wins.
- If one or more spades were played, the highest spade wins.
- The trick winner leads the next trick.

#### Breaking Spades

Spades cannot be led until one of the following occurs:

1. A player has played a spade on a previous trick (trumping because they were void in the led suit).
2. The leader has **only spades** remaining in their hand.

Once either condition is met, spades are "broken" and may be led freely.

#### 13 Tricks

Play continues until all 13 tricks have been played, then scoring occurs.

## Scoring / Winning

### Partnership Contract

The team's contract is the sum of both partners' bids (excluding Nil bids, which are scored separately).

### Making the Contract

If the team wins **at least** as many tricks as their combined bid:

- **Base score:** 10 points per trick bid (e.g., a bid of 7 earns 70 points if made).
- **Overtricks (bags):** Each trick won beyond the bid earns **1 point**.

### Failing the Contract

If the team wins **fewer** tricks than their combined bid:

- The team loses **10 points per trick bid** (e.g., a bid of 7 costs -70 points if failed).
- Overtricks are irrelevant when the contract fails.

### Bag Penalty (Sandbagging)

Overtricks (bags) accumulate across hands. When a team accumulates **10 bags**, they receive a **-100 point penalty** and the bag counter resets to zero.

Bags carry over between hands. For example, 4 bags in hand 1 and 7 bags in hand 2 = 11 bags total = -100 penalty, with 1 bag carried forward.

### Nil Scoring

| Bid | Success | Failure |
|-----|---------|---------|
| Nil | +100 points | -100 points |
| Blind Nil | +200 points | -200 points |

Nil scoring is **independent** of the partner's bid:

- A Nil bidder's partner still has their own contract to fulfill.
- If the Nil bidder takes tricks, those tricks **do** count toward the partner's contract.
- Example: Partner bids 4, Nil bidder accidentally wins 2 tricks. The Nil fails (-100), but the partner only needs 2 more tricks to make their bid of 4.

### Game End

The game ends when either team reaches **500 points** (or another agreed-upon target). If both teams exceed 500 in the same hand, the team with the higher score wins. If tied, play continues.

## Special Rules

### Minimum Bid Variant

Some groups require a combined team bid of at least 4.

### Double Nil

Both partners may bid Nil in the same hand. If both succeed, +200 points; if both fail, -200 points. If one succeeds and one fails, they cancel (net 0 for the Nil portion).

### Joker Variant (Dennis Barmore Rules)

- Add both jokers to the deck; remove 2 of clubs and 2 of diamonds.
- Big Joker is the highest trump; Little Joker is the second highest trump.
- Trump ranking: Big Joker > Little Joker > A♠ > K♠ > ... > 2♠.

### Three-Player Variant

- Remove 2 of clubs (51 cards). Each player gets 17 cards.
- No partnerships -- each player bids and scores individually.
- Scoring: 10 points per bid if made, -10 per bid if failed. Bags and bag penalty still apply.

### Two-Player Variant

- Each player draws cards alternately from the deck, choosing to keep or discard (face-down) each card drawn, until both players have 13 cards.
- Play proceeds as in standard 4-player Spades with individual scoring.

### Six-Player Variant

- Uses two 52-card decks minus low cards (102 cards total), forming three partnerships.
- If two identical cards appear in the same trick, the second one played beats the first.

## Key Strategic Concepts

- **Accurate bidding** is the most important skill. Underbidding leads to bags; overbidding leads to set penalties (-10 per trick bid).
- **Bag management** requires balancing between making your contract and avoiding overtricks. A team at 8 bags should aim for exact bids.
- **Counting trump** (tracking how many spades have been played) is essential for knowing when your spades are winners.
- **Nil defense/offense:** When your partner bids Nil, you must win tricks to protect them (lead suits they are void in, win tricks before they are forced to). When an opponent bids Nil, target their void suits and force them to win tricks.
- **Leading strategy:** Leading a suit your partner is strong in helps them win tricks. Leading a suit an opponent is void in forces them to trump or discard.
- **Card counting:** Tracking all 52 cards across 13 tricks determines which cards are winners in late-game play.
- **Communication through play:** Since no verbal communication is allowed, partners signal strength through their card choices (playing high or low in a suit).

## Common Terminology

| Term | Definition |
|------|-----------|
| **Trick** | A round where each player plays one card; the highest card wins |
| **Trump** | Spades -- always the trump suit in this game |
| **Bid** | The number of tricks a player commits to winning |
| **Contract** | The combined bid of a partnership |
| **Bag** | An overtrick (trick won beyond the bid) |
| **Sandbagging** | Accumulating bags, leading to the -100 penalty at 10 bags |
| **Set** | Failing to make the team's contract |
| **Nil** | A bid of zero tricks |
| **Blind Nil** | A Nil bid made before looking at one's cards |
| **Break spades** | Playing a spade for the first time (enabling spades to be led) |
| **Follow suit** | Playing a card of the same suit that was led |
| **Void** | Having no cards in a particular suit |
| **Sluff / Throw off** | Playing an unwanted card when unable to follow suit |
| **Book** | The first 6 tricks (some scoring variants count tricks from book 7 onward) |
| **Boston** | Winning all 13 tricks in a hand |

## State Space Analysis

### Information Sets
- Initial hands: C(52,13) per player.
- Each player knows: own hand, all bids, cards played so far.
- Hidden: other 3 players' hands (39 cards initially).
- As tricks are played, information accumulates (cards revealed, void suits identified).

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible deals | ~5.36 × 10^28 (same as bridge) |
| Bidding sequences | ~14^4 = 38,416 (each player 0-13) |
| Play sequences | Up to 13! (constrained by suit-following) |
| Game tree nodes | ~10^20-10^30 |
| Information sets | ~10^15-10^20 |

### Action Space
- **Bidding**: integer 0-13 (with nil as special 0).
- **Play**: choose from legal cards (1-13, constrained by suit-following and spades-breaking rules).
- Branching factor: ~3-13 per trick (depends on hand composition and remaining cards).

## Key Challenges for AI/Solver Approaches

### 1. Partnership Coordination Without Communication
Partners cannot communicate during bidding or play except through:
- Their bid (signals confidence in a specific trick count).
- Their card play (which cards they lead, which they play on partner's lead).
- Implicit signaling: leading a suit signals strength in that suit; playing high on partner's lead signals support.

### 2. Bid Calibration
Accurate bidding requires:
- Estimating tricks from hand strength alone.
- Accounting for partner's likely contribution (inferred from their bid).
- Balancing between overbidding (risk of missing bid) and underbidding (bag accumulation).
- The bags penalty creates a medium-term strategic consideration.

### 3. Nil Defense and Attack
Nil bids create asymmetric gameplay:
- The nil bidder's partner must try to win tricks to protect the nil.
- Opponents try to force the nil bidder to take a trick.
- Nil play involves leading low to pass through, partner covering with high cards.
- A significant portion of strategic depth comes from nil dynamics.

### 4. Spades-Breaking Timing
The restriction on leading spades until broken creates strategic considerations:
- Breaking spades early benefits players with many spades.
- Delaying the break benefits players with spade voids.
- The timing of spade-breaking is a strategic lever.

### 5. Bags Management
The 10-bag penalty creates a multi-hand strategic concern:
- Teams near 10 bags must avoid overtricks (even at the cost of some efficiency).
- This penalizes conservative bidding systematically.
- Long-term bag management requires planning across multiple hands.

## Known Solver Results

### Academic Work
- Very limited published research on Spades AI.
- Some work on trick-taking game AI applicable to Spades (overlap with Bridge, Hearts).
- No published Nash equilibrium computation for Spades.

### AI Approaches
- **Rule-based**: most existing Spades AI uses heuristics:
  - Bid based on high-card points + trump count.
  - Play follows standard trick-taking heuristics (second hand low, third hand high).
  - Nil strategies coded explicitly.
- **Monte Carlo simulation**: sample unknown hands, simulate play, evaluate actions.
- **RL-based**: some mobile game implementations use basic RL, but not at competitive levels.
- **No superhuman AI**: Spades has not received the research attention of bridge or poker.

### Online Spades Platforms
- Trickster Cards, Spades Plus, VIP Spades — all use simple heuristic AI.
- No competitive AI benchmark or tournament series.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2001 | Ginsberg (GIB) | MC + double-dummy applicable to trick-taking games |
| 2016 | Various card game AI surveys | Spades as part of trick-taking family |
| 2020 | RL for card games (various) | General RL techniques applicable to Spades |

## Relevance to Myosu

### Solver Applicability
Spades tests **partnership trick-taking** with simpler bidding than bridge:
- **CFR**: applicable as partnership vs partnership (2-team zero-sum). Simpler than bridge due to fixed trump suit and straightforward bidding.
- **Monte Carlo simulation**: proven approach for trick-taking games. MC + perfect-information solver for evaluating plays.
- **Neural networks**: can learn bidding functions and play policies from self-play.
- **Partnership modeling**: less complex than bridge (no convention system), but coordination still matters.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 3/5 | Partnership structure; moderate state space |
| Neural value network potential | 4/5 | Trick estimation and play quality learnable |
| Abstraction necessity | 2/5 | Moderate state space |
| Real-time solving value | 4/5 | MC simulation effective |
| Transferability of techniques | 4/5 | Shares with bridge, hearts, and other trick-taking games |

### Myosu Subnet Considerations
- **Widespread popularity**: Spades is one of the most popular card games in the US, particularly in casual settings.
- **Simpler than bridge**: provides an entry point for partnership trick-taking without bridge's bidding complexity.
- **Nil evaluation**: nil bid success rate is a clean metric for defensive/offensive play quality.
- **Bags tracking**: multi-hand strategy (bags management) tests long-term planning.
- **Game oracle**: trick resolution, scoring, and bags tracking are straightforward to implement.
- **Shared infrastructure**: trick-taking engine can be shared with Bridge (10) and Hearts (20).

### Recommended Approach for Myosu
1. MC simulation with perfect-information trick solver for play decisions.
2. Neural networks for bid prediction (input: hand features; output: trick estimate).
3. Specialized nil strategy module.
4. Evaluate via win rate and score differential over multi-hand sessions.
5. Share trick-taking infrastructure with Bridge and Hearts to reduce implementation effort.
