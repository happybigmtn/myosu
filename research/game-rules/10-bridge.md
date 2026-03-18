# Contract Bridge

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Contract Bridge (Duplicate Bridge for competition) |
| Variants | Rubber Bridge, Duplicate Bridge, Minibridge, Chicago Bridge |
| Players | 4 (two partnerships: North-South vs East-West) |
| Information | Imperfect (hidden hands, bidding signals, dummy exposed) |
| Stochasticity | Stochastic (card deals; eliminated in Duplicate) |
| Zero-Sum | Yes (between partnerships) |
| Solved Status | Unsolved; AI below top human level in bidding |

## Overview

Contract Bridge is a partnership trick-taking card game for four players, widely considered the most strategically complex card game. Two partnerships (North-South vs. East-West) compete through two distinct phases: the **auction** (bidding for a contract) and the **play** (executing tricks to fulfill or defeat the contract). Bridge originated in the early 20th century as an evolution of Whist and Auction Bridge. It is played worldwide in clubs, tournaments, and online, with standardized rules governed by the World Bridge Federation.

**Players:** 4 (two partnerships of 2)
**Deck:** Standard 52-card deck, no jokers
**Objective:** Score the most points across multiple deals by bidding and making contracts, or by defeating the opponents' contracts

## Equipment

- One standard 52-card deck (no jokers)
- A scoresheet or scoring device
- A bidding box (in tournament play) or verbal bidding
- Two boards/trays for holding dealt hands (in duplicate play)

### Card Rankings
Within each suit, cards rank: A (high), K, Q, J, 10, 9, 8, 7, 6, 5, 4, 3, 2 (low).

### Suit Rankings (for bidding purposes only)
Suits rank: No Trump (highest) > Spades > Hearts > Diamonds > Clubs (lowest). Spades and Hearts are **major suits** (worth more per trick). Diamonds and Clubs are **minor suits**.

## Setup

### Partnership and Seating
Partners sit across from each other at a table. The four positions are designated North, East, South, and West. North-South form one partnership; East-West form the other.

### Dealing
1. The dealer shuffles and the player to the right cuts.
2. Deal one card at a time, face-down, clockwise, starting with the player to the dealer's left.
3. Each player receives 13 cards.
4. The dealer rotates clockwise after each deal.

### Vulnerability
In rubber bridge, a side becomes **vulnerable** after winning one game (reaching 100+ trick points below the line). Vulnerability increases bonuses and penalties.

In duplicate/Chicago bridge, vulnerability is predetermined for each deal.

## Game Flow

Each deal consists of two phases:

### Phase 1: The Auction (Bidding)

The dealer opens the auction. Proceeding clockwise, each player may:
- **Bid:** State a contract (level + denomination). Levels are 1-7, representing tricks beyond the "book" of 6. Denominations are: Clubs, Diamonds, Hearts, Spades, No Trump. A bid of "1 Heart" means a contract to win at least 7 tricks (6 + 1) with Hearts as trump.
- **Pass:** Decline to bid.
- **Double:** Double the last bid made by an opponent. Increases penalties for failure and bonuses for success.
- **Redouble:** After an opponent's double, the declaring side may redouble, further increasing stakes.

**Bidding rules:**
- Each bid must be higher than the previous bid (either a higher level, or the same level with a higher-ranked denomination).
- The auction ends when three consecutive passes follow a bid (or four passes if no bid was made, resulting in a passed-out deal — no play, redeal).
- The final bid becomes the **contract**.

**Declarer determination:**
The player from the winning partnership who first bid the denomination of the final contract becomes the **declarer**. Their partner becomes the **dummy**.

### Phase 2: The Play

1. The player to the declarer's left makes the **opening lead** (plays the first card face-up to the table).
2. After the opening lead, the dummy's hand is placed face-up on the table, organized by suit.
3. Play proceeds clockwise. Each player plays one card to each trick (4 cards per trick, 13 tricks per deal).
4. The declarer controls both their own hand and the dummy's hand.

**Trick rules:**
- You must follow suit if possible (play a card of the suit led).
- If you cannot follow suit, you may play any card (including a trump card).
- The trick is won by the highest trump card played, or if no trump was played, by the highest card of the suit led.
- The winner of each trick leads to the next trick.

**No Trump contracts:** There is no trump suit. The highest card of the suit led wins each trick.

## Actions

### During the Auction
| Action | Rules |
|--------|-------|
| **Bid** | Must be higher than the last bid. Format: Level (1-7) + Denomination (C/D/H/S/NT). |
| **Pass** | No obligation. Passing does not prevent bidding later in the auction. |
| **Double** | Only on an opponent's bid. Applies to the most recent bid only. Cancelled by any subsequent bid. |
| **Redouble** | Only after an opponent's double. Cancelled by any subsequent bid. |

### During the Play
| Action | Rules |
|--------|-------|
| **Follow suit** | Mandatory if you hold a card of the suit led. |
| **Ruff (Trump)** | Play a trump card when void in the suit led. |
| **Sluff (Discard)** | Play a non-trump card of a different suit when void in the suit led. |
| **Claim** | Declarer (or defender) may claim remaining tricks by showing their hand and stating a line of play. Opponents may accept or contest. |

## Scoring

### Trick Values (Below the Line / Contract Points)

| Denomination | Points per Trick |
|-------------|-----------------|
| Clubs | 20 per trick |
| Diamonds | 20 per trick |
| Hearts | 30 per trick |
| Spades | 30 per trick |
| No Trump | 40 for the first trick, 30 for each subsequent trick |

Only tricks bid and made count toward contract points (below the line in rubber bridge). Doubled contracts: trick values x2. Redoubled: trick values x4.

### Game and Slam Thresholds
- **Game:** A contract whose trick value totals 100+ points (e.g., 3NT = 40+30+30 = 100; 4H = 30x4 = 120; 5C = 20x5 = 100).
- **Partscore:** A contract worth less than 100 trick points.
- **Small Slam:** 6-level contract (12 tricks).
- **Grand Slam:** 7-level contract (all 13 tricks).

### Overtricks (Tricks Beyond the Contract)

| Condition | Not Vulnerable | Vulnerable |
|-----------|---------------|------------|
| Undoubled | Trick value (20 or 30) per overtrick | Trick value per overtrick |
| Doubled | 100 per overtrick | 200 per overtrick |
| Redoubled | 200 per overtrick | 400 per overtrick |

### Undertricks (Failing to Make the Contract)

| Undertricks | Not Vul, Undoubled | Vul, Undoubled | Not Vul, Doubled | Vul, Doubled | Not Vul, Redoubled | Vul, Redoubled |
|-------------|-------------------|---------------|-------------------|--------------|---------------------|----------------|
| 1st | 50 | 100 | 100 | 200 | 200 | 400 |
| 2nd | 50 | 100 | 200 | 300 | 400 | 600 |
| 3rd | 50 | 100 | 200 | 300 | 400 | 600 |
| 4th+ | 50 | 100 | 300 | 300 | 600 | 600 |

### Bonuses

| Bonus | Not Vulnerable | Vulnerable |
|-------|---------------|------------|
| **Partscore** (duplicate) | 50 | 50 |
| **Game** (duplicate) | 300 | 500 |
| **Small Slam** | 500 | 750 |
| **Grand Slam** | 1,000 | 1,500 |
| **Making a doubled contract** (insult bonus) | 50 | 50 |
| **Making a redoubled contract** | 50 | 50 |

### Rubber Bridge Specific Scoring

**Below the line:** Only bid-and-made trick points accumulate toward game (100 points).

**Above the line:** All other scores (overtricks, undertricks, slam bonuses, honors, rubber bonus).

**Rubber bonus:**
- Win the rubber 2 games to 0: 700 points
- Win the rubber 2 games to 1: 500 points

**Unfinished rubber:**
- Side with a game: 300 points
- Side with a partscore in an unfinished game: 50 points

**Honors (rubber bridge only):**
- Any player holding 4 of the 5 trump honors (A, K, Q, J, 10): 100 points (scored by that player's side, above the line)
- Any player holding all 5 trump honors: 150 points
- Any player holding all 4 aces in a No Trump contract: 150 points

### Duplicate Bridge Scoring
In duplicate, each deal is scored independently. Vulnerability is assigned by board number, not earned. Matchpoints and IMPs are the two main ranking methods:

**Matchpoints (MPs):** Your score on a board is compared to every other pair playing the same board. You receive 1 MP for each pair you beat, 0.5 for each tie. Overtricks are critical.

**International Match Points (IMPs):** The difference between your score and the comparison score is converted via the IMP table. Favors contract accuracy over overtricks.

| Point Difference | IMPs |
|-----------------|------|
| 0-10 | 0 |
| 20-40 | 1 |
| 50-80 | 2 |
| 90-120 | 3 |
| 130-160 | 4 |
| 170-210 | 5 |
| 220-260 | 6 |
| 270-310 | 7 |
| 320-360 | 8 |
| 370-420 | 9 |
| 430-490 | 10 |
| 500-590 | 11 |
| 600-740 | 12 |
| 750-890 | 13 |
| 900-1090 | 14 |
| 1100-1290 | 15 |
| 1300-1490 | 16 |
| 1500-1740 | 17 |
| 1750-1990 | 18 |
| 2000-2240 | 19 |
| 2250-2490 | 20 |
| 2500-2990 | 21 |
| 3000-3490 | 22 |
| 3500-3990 | 23 |
| 4000+ | 24 |

## Bidding System: Standard American Yellow Card (SAYC)

SAYC is the most common convention in North America. Key elements:

### Opening Bids

| Bid | Meaning |
|-----|---------|
| **1C / 1D** | 13-21 HCP, 3+ cards in the suit (open longer minor; with 3-3, open 1C) |
| **1H / 1S** | 13-21 HCP, 5+ cards in the suit |
| **1NT** | 15-17 HCP, balanced hand (no 5-card major, no singleton, no void) |
| **2C** | 22+ HCP (or very strong playing hand). Artificial, forcing. |
| **2D / 2H / 2S** | Weak two-bid: 5-11 HCP, 6-card suit of reasonable quality |
| **2NT** | 20-21 HCP, balanced |
| **3-level+** | Preemptive: 7+ card suit, limited high-card values |

### Responses to 1-of-a-Suit

| Response | Meaning |
|----------|---------|
| **New suit at 1-level** | 6+ HCP, 4+ cards, forcing 1 round |
| **New suit at 2-level** | 10+ HCP (2/1 game forcing in modern style), 4+ cards |
| **1NT** | 6-9 HCP, no 4-card major to show at 1-level ("dustbin 1NT") |
| **2NT** | 13-15 HCP, balanced, no 4-card major (forcing to game) |
| **3NT** | 16-17 HCP, balanced |
| **Single raise (1H - 2H)** | 6-9 HCP, 3+ card support |
| **Jump raise (1H - 3H)** | 10-12 HCP, 4+ card support (limit raise, invitational) |
| **Double raise to game (1H - 4H)** | Weak distributional hand with 5+ card support |

### Responses to 1NT

| Response | Meaning |
|----------|---------|
| **2C (Stayman)** | Asking for a 4-card major. Opener rebids 2D (no major), 2H, or 2S. |
| **2D / 2H** (Jacoby Transfer) | Transfer to 2H / 2S respectively. Shows 5+ in the major. |
| **2S** | Minor suit Stayman or range-finding (varies by partnership) |
| **2NT** | Invitational (8-9 HCP) |
| **3NT** | Game values (10-15 HCP), balanced |
| **4NT** | Quantitative slam invitation |

### Competitive Bidding

| Convention | Meaning |
|-----------|---------|
| **Takeout Double** | Double of opponent's suit bid at low level, showing opening values with support for unbid suits |
| **Overcall** | Bid a suit over opponent's opening: 8-16 HCP, 5+ card suit |
| **1NT Overcall** | 15-18 HCP, balanced, stopper in opponent's suit |
| **Negative Double** | Double by responder after partner opens and opponent overcalls, showing values and unbid suits |
| **Blackwood (4NT)** | Asks partner for number of aces (responses: 5C=0/4, 5D=1, 5H=2, 5S=3) |
| **Gerber (4C over NT)** | Asks for aces when 4NT would be natural |

## Winning Conditions

### Rubber Bridge
Win two games (reach 100+ below-the-line trick points twice) before the opponents. The rubber ends when one side wins two games. The side with the higher total score (above + below the line) wins the rubber.

### Duplicate Bridge
The pair or team with the best aggregate matchpoint percentage or IMP score across all boards wins the event.

### Chicago (Four-Deal Bridge)
Four deals are played, with vulnerability rotating per deal: Deal 1 (neither vul), Deal 2 (dealer's side vul), Deal 3 (dealer's side vul), Deal 4 (both vul). The side with the higher total score after four deals wins.

## Special Rules

### Revoke (Failure to Follow Suit)
If a player fails to follow suit when able, the opponents may claim a penalty after the revoke is established (trick completed). The revoking side transfers 1 trick to the non-offending side (2 tricks if the revoking side won the revoke trick and any subsequent trick).

### Lead Restrictions
- The opening lead is made face-down in duplicate bridge (to prevent leads out of turn).
- No restrictions on opening lead choice (any card is permitted).

### Dummy's Restrictions
- Dummy may not initiate play, suggest plays, or draw attention to irregularities.
- Dummy may warn partner about leading from the wrong hand or failing to follow suit.

### Insufficient Bid
A bid lower than the previous bid is insufficient. The bidder may correct it to a sufficient bid; penalties may apply (opponent may accept the insufficient bid, or the bidder may be restricted in subsequent calls).

### Claim and Concession
A player may claim remaining tricks by stating their intended line of play. If opponents accept, tricks are awarded. If contested, the claim is adjudicated by the director (tournament) or resolved by playing on.

## Key Strategic Concepts

### Hand Evaluation
High Card Points (HCP): A=4, K=3, Q=2, J=1. Distribution points add value for long suits and short suits. A combined partnership count of 25+ HCP typically produces game; 33+ suggests slam.

### The Bidding Conversation
Bidding is a coded conversation between partners. Each bid communicates information about hand shape and strength. The goal is to find the best denomination (suit fit of 8+ cards, or No Trump) and the appropriate level.

### Declarer Play Techniques
- **Finesse:** Leading toward a high card, hoping the intervening higher card is held by the opponent playing second.
- **Endplay/Throw-in:** Forcing an opponent to lead a suit that gives you a trick.
- **Squeeze:** Reducing an opponent's hand to the point where they cannot protect all their winning holdings.
- **Elimination:** Stripping an opponent of safe exit cards before throwing them in.

### Defensive Play
- **Signal:** Partners communicate through card choices (attitude: high = encouraging; count: high-low = even number; suit preference).
- **Opening Lead Selection:** Against NT: lead 4th-best from longest and strongest suit. Against suits: lead partner's bid suit, top of a sequence, or a singleton.

### Vulnerability Awareness
Vulnerability changes the risk/reward calculus. Vulnerable games and slams earn more, but undertrick penalties are higher. Non-vulnerable preempts are cheaper.

## Common Terminology

| Term | Definition |
|------|------------|
| **Declarer** | The player who plays the hand for the winning partnership |
| **Dummy** | Declarer's partner, whose hand is laid face-up on the table |
| **Defender** | The two players opposing the declarer |
| **Contract** | The final bid, determining the trump suit and number of tricks required |
| **Book** | The first 6 tricks (must be won before the contract counts) |
| **Trick** | A round of 4 cards, one from each player |
| **Trump** | The suit designated by the contract; trumps beat all non-trump cards |
| **Ruff** | Playing a trump card on a non-trump lead when void in the led suit |
| **Finesse** | Attempting to win a trick with a lower card by playing toward it |
| **Slam** | Bidding and making 12 tricks (small slam) or 13 tricks (grand slam) |
| **Game** | A contract worth 100+ trick points |
| **Partscore** | A contract worth less than 100 trick points |
| **Vulnerable** | A side that has won one game (or is assigned vulnerable by board) |
| **HCP** | High Card Points (A=4, K=3, Q=2, J=1) |
| **Stopper** | A holding that can win a trick in the opponent's suit (e.g., Ax, Kx, QJx) |
| **Void** | Having no cards in a suit |
| **Singleton** | Having exactly one card in a suit |
| **Doubleton** | Having exactly two cards in a suit |
| **Fit** | Combined holding of 8+ cards in a suit between partners |
| **Forcing bid** | A bid that partner is not allowed to pass |
| **Convention** | A bid with an artificial (non-natural) agreed meaning |
| **Overcall** | A bid made after an opponent has opened the bidding |
| **Preempt** | A high-level opening bid on a long suit with limited high cards |
| **Squeeze** | Forcing an opponent to discard a card they need to keep |
| **Endplay** | Forcing an opponent to lead to their disadvantage |

## State Space Analysis

### Information Sets
- Initial hands: C(52,13) × C(39,13) × C(26,13) = enormous.
- Bidding phase: variable-length auction sequence (up to ~80 possible auctions for a given hand).
- Play phase after dummy exposed: declarer sees 26 cards (own + dummy); defenders each see 13 + dummy.
- Information sets in play phase: reduced by dummy exposure, but defenders still face significant uncertainty.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible deals | ~5.36 × 10^28 |
| Bidding sequences | Variable; ~10^5-10^7 meaningful auctions |
| Play sequences (13 tricks) | Up to 13! orderings (constrained by suit-following) |
| Game tree nodes (bidding + play) | ~10^40+ |
| Information sets (play phase, per defender) | ~10^12-10^15 |

### Action Space
- **Bidding**: at each turn, ~38 possible calls (35 bids + pass + double + redouble, constrained by rules).
- **Play**: at each trick, 1-13 legal cards (must follow suit).
- Branching factor decreases as tricks are played.

## Key Challenges for AI/Solver Approaches

### 1. Two Distinct Phases with Different Requirements
**Bidding** is a cooperative communication problem under constraints:
- Partners must communicate hand information through a limited vocabulary (35 bid levels + pass/double/redouble).
- Conventions (bidding systems) encode agreements about what bids mean.
- Adversarial element: opponents may interfere (overcalls, doubles).

**Play** is a sequential decision problem:
- Declarer has near-perfect information (sees 26/52 cards).
- Defenders have imperfect information (infer partner's and declarer's hands from bidding and play).

### 2. Partnership Communication
Bridge is fundamentally a **cooperative imperfect-information game within a competitive framework**. The bidding system serves as a communication protocol, and optimizing it is akin to designing a signaling scheme under noisy, adversarial conditions.

### 3. Convention-Dependent Strategy
Bidding strategies are deeply entangled with the chosen convention system. A solver must either:
- Operate within a fixed convention system (limiting strategic space).
- Learn its own conventions (requiring co-training of partners).
- Learn to adapt to arbitrary convention systems (the hardest option).

### 4. Defense Coordination
The defending partnership must coordinate without explicit communication:
- Signaling through card play (attitude, count, suit-preference signals).
- Inferring partner's hand from their bids, leads, and plays.
- This is a decentralized cooperative control problem.

### 5. Opening Lead
The opening lead is made before dummy is exposed — a uniquely difficult decision:
- Must choose from 13 cards based only on own hand and the bidding auction.
- High-variance decision — a poor opening lead can gift the contract; a brilliant one can defeat it.

## Known Solver Results

### WBridge5 and GIB
- **GIB** (Ginsberg, 2001): one of the first competitive bridge AIs. Uses Monte Carlo simulation (dealing random hands consistent with bidding information) and double-dummy analysis.
- **WBridge5**: multiple-time world computer bridge champion. Uses heuristic bidding system + MC simulation for play.
- These programs play at a strong amateur level but below top human professionals, particularly in bidding.

### NooK
- World Computer Bridge Championship winner (multiple years).
- Similar MC simulation approach to GIB.
- Bidding remains the primary weakness.

### Deep Learning Approaches
- **Joint AI (Jack, Blue Chip Bridge)**: commercially available bridge programs using rule-based bidding with MC play.
- **Bridgit / BridgeNN**: neural network approaches to bidding, learning from large databases of expert play.
- **Alpha-Bridge**: various research attempts to apply AlphaZero-style approaches, with limited success due to the partnership nature.

### Recent Developments
- **NukkAI (2022)**: defeated 8 world champions in a controlled match (play only, with fixed bidding). Demonstrated superhuman play (trick-taking) but did not address bidding.
- Key insight: bridge play (given a contract) is more tractable than bridge bidding.

### Theoretical Results
- Double-dummy solvers (Bo Haglund's DDS): can solve the play phase perfectly when all 52 cards are known. O(seconds) per deal.
- Monte Carlo + double-dummy: the standard approach — sample random hands consistent with known information, solve each double-dummy, aggregate results.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2001 | Ginsberg, "GIB: Imperfect Information in a Computationally Challenging Game" | MC + double-dummy for bridge |
| 2005 | Levy, "Robots on the Bridge" | Survey of bridge AI |
| 2016 | Yeh & Lin, "Bridge Bidding with Deep RL" | Neural bidding |
| 2020 | Lockhart et al., "Computing Approx. Equilibria in Sequential Games with IIG" | Theoretical framework applicable to bridge |
| 2022 | NukkAI, "Superhuman Bridge Play" | Defeated world champions in play phase |
| 2023 | Tian et al., "Bidding in Bridge with Deep RL" | End-to-end bidding learning |

## Relevance to Myosu

### Solver Applicability
Bridge is a **unique challenge** combining cooperative communication with adversarial competition:
- **CFR**: applicable to the play phase as a two-team zero-sum game. Not directly applicable to bidding (cooperative communication).
- **Monte Carlo + double-dummy**: proven approach for play evaluation. Computationally cheap per sample.
- **Deep RL for bidding**: emerging approach, but partnership coordination remains difficult.
- **Convention learning**: co-training of partners to develop bidding conventions is an open research problem.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 2/5 | Partnership structure complicates standard CFR |
| Neural value network potential | 4/5 | Strong for play; bidding is harder |
| Abstraction necessity | 3/5 | Play phase is moderate size; bidding is the bottleneck |
| Real-time solving value | 5/5 | MC + double-dummy is the standard real-time approach |
| Transferability of techniques | 3/5 | Partnership mechanics are unique to bridge/spades |

### Myosu Subnet Considerations
- **Global player base**: bridge has an estimated 200+ million players worldwide. The competitive duplicate bridge community is highly organized.
- **Separation of bidding and play**: the subnet could evaluate these independently, since play-phase solving is more mature.
- **Convention system specification**: solvers must declare their bidding conventions, matching tournament bridge requirements.
- **Double-dummy oracle**: the double-dummy solver provides a ground-truth for play evaluation — how close does the AI's play come to double-dummy optimal?
- **Duplicate format**: eliminates deal variance, providing clean skill comparison. Natural fit for subnet evaluation.
- **Anti-cheating relevance**: bridge has significant online cheating problems. AI-verified optimal play could serve as a fairness tool.

### Recommended Approach for Myosu
1. Use MC + double-dummy for play-phase solving (proven, efficient).
2. Evaluate play quality via "double-dummy percentage" — fraction of double-dummy-optimal results achieved.
3. Treat bidding as a separate evaluation target — compare bid sequences against expert databases.
4. Consider bridge as a long-term challenge game — bidding AI is an open research problem with significant value.
5. Leverage the duplicate format for variance-free evaluation on the subnet.
