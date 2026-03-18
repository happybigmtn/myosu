# Pot-Limit Omaha (PLO)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Pot-Limit Omaha Hold'em |
| Variants | PLO4 (4 cards), PLO5 (5 cards), PLO6 (6 cards), Hi/Lo split |
| Players | 2-9 (commonly 6-max) |
| Information | Imperfect (hidden hole cards) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (HU); general-sum (multiway) |
| Solved Status | Unsolved; no superhuman AI demonstrated |

## Overview

Pot-Limit Omaha is a community card poker game where each player receives four private hole cards (instead of two in Hold'em) and must use **exactly two** of them combined with **exactly three** of the five community cards to form their best five-card hand. The pot-limit betting structure caps the maximum bet at the current size of the pot. PLO is the second most popular poker variant worldwide after No-Limit Hold'em.

**Players:** 2 to 9 (commonly 6-max)
**Deck:** Standard 52-card deck, no jokers
**Betting structure:** Pot-limit (maximum bet/raise equals the current pot size)

## Setup

### Deck and Cards
Standard 52-card deck. Cards are ranked A (high) through 2 (low). Aces may play low in the straight A-2-3-4-5. Suits have no ranking.

### Positions and Blinds
Positions are identical to Hold'em. In a 6-max game: UTG, HJ, CO, BTN, SB, BB. Blind structure uses a small blind and big blind, identical to Hold'em.

### Dealing
1. Both blinds are posted.
2. Dealer burns one card.
3. Starting with the SB and proceeding clockwise, each player receives one card face-down.
4. This is repeated three more times, for a total of **four hole cards** per player.
5. Each player now has exactly four private hole cards.

## Game Flow

The game flow is identical to Hold'em in structure: preflop, flop, turn, river, showdown. The critical difference is the number of hole cards and the hand construction rule.

### 1. Preflop
- Action begins with UTG and proceeds clockwise.
- Each player may fold, call the big blind, or raise (up to the pot).
- Blinds are live, with the same rules as Hold'em.

### 2. Flop
- Dealer burns one card, then deals three community cards face-up.
- First active player left of the button acts first.

### 3. Turn
- Dealer burns one card, then deals one community card face-up.
- Same action order as flop.

### 4. River
- Dealer burns one card, then deals one final community card face-up.
- Same action order as flop and turn.

### 5. Showdown
- Remaining players reveal hands.
- Each player **must use exactly two hole cards and exactly three community cards** to form their best five-card hand.
- Best hand wins. Ties split the pot.

## Betting Rules

### Pot-Limit Structure
The defining feature of PLO's betting structure is that the maximum bet or raise is capped at the size of the pot.

### Pot Calculation Formula

**When no bet has been made on the current street:**
- Maximum bet = total pot (all chips from previous streets + blinds + any antes).

**When facing a bet:**
- Maximum raise = Pot before the bet + the bet + the cost to call
- Equivalently: 3 x (the bet) + (pot before the bet)

**Step-by-step:**
1. Start with the pot before any action on the current street.
2. Add all bets and calls made so far on the current street.
3. Add the amount needed to call the current bet.
4. The result is the maximum raise **size** (the raise increment).
5. The total amount placed into the pot = call amount + raise size.

**Preflop pot calculation example (Blinds 5/10):**
- Pot before action = 15 (SB 5 + BB 10).
- UTG wants to raise the pot. UTG must first call 10, making the pot 25, then raise 25. Total bet = 35.
- Shortcut: 3 x 10 (the amount to call, treated as a bet) + 5 (pot before the "bet," i.e., the SB) = 35.

**Postflop pot calculation example:**
- Pot entering the flop = 70.
- Player A bets 40. Pot is now 110.
- Player B wants to pot-raise. Call amount = 40. Pot after call = 150. Maximum raise = 150. Total bet = 40 (call) + 150 (raise) = 190.
- Shortcut: 3 x 40 + 70 = 190.

**Re-raise example:**
- Pot = 70. Player A bets 40 (pot = 110). Player B raises to 190 (pot = 300). Player C wants to pot-re-raise.
- Call amount = 190. Pot after call = 490. Maximum raise = 490. Total = 190 + 490 = 680.
- Shortcut: 3 x 190 + (70 + 40) = 570 + 110 = 680.

### Minimum Bet and Raise
- **Minimum bet:** Equal to the big blind.
- **Minimum raise:** The raise increment must be at least equal to the previous bet or raise increment (same as NL Hold'em).
  - Example: BB = 10. Player A bets 10. Player B's minimum raise increment is 10, so minimum raise is to 20. Player C's minimum raise increment is 10, so minimum raise is to 30.

### All-In Rules
- A player may go all-in at any time for their remaining stack, even if that amount exceeds the pot (the excess is irrelevant; they simply bet their stack).
- If a player's stack is less than the minimum bet or a pot-sized bet, they can go all-in for their remaining chips.
- Under-raise all-in rules are the same as in NL Hold'em (does not reopen action if less than a full raise).
- Side pots are created as in Hold'em when players have unequal stacks.

### Declaring "Pot"
In live games, a player may verbally declare "pot" and the dealer will calculate and announce the maximum bet. This is a binding declaration.

## Hand Rankings

Hand rankings are identical to standard poker. The critical difference is in **hand construction**, not hand ranking.

| Rank | Hand | Example (using 2 hole cards + 3 board cards) |
|------|------|-----------------------------------------------|
| 1 | Royal Flush | Hole: A{s}K{s}7{d}2{c} / Board: Q{s}J{s}T{s}4{h}8{d} |
| 2 | Straight Flush | Hole: 9{h}8{h}K{d}3{c} / Board: 7{h}6{h}5{h}A{s}Q{d} |
| 3 | Four of a Kind | Hole: Q{s}Q{h}Q{d}3{c} / Board: Q{c}7{s}2{d}A{h}9{c} |
| 4 | Full House | Hole: K{s}K{h}9{d}4{c} / Board: K{d}9{s}9{c}2{h}6{d} |
| 5 | Flush | Hole: A{c}J{c}K{d}5{s} / Board: 8{c}6{c}3{c}T{h}2{d} |
| 6 | Straight | Hole: J{s}T{h}4{d}3{c} / Board: Q{d}9{c}8{s}2{h}A{c} |
| 7 | Three of a Kind | Hole: 8{s}8{h}A{d}3{c} / Board: 8{d}K{s}6{c}2{h}5{d} |
| 8 | Two Pair | Hole: A{s}J{h}7{d}3{c} / Board: A{d}J{c}5{s}8{h}2{d} |
| 9 | One Pair | Hole: A{s}K{h}7{d}3{c} / Board: A{d}9{c}6{s}4{h}2{d} |
| 10 | High Card | Hole: A{s}K{h}7{d}3{c} / Board: Q{d}J{c}9{s}5{h}2{d} |

### The Two-and-Three Rule (Critical)
This is the most important rule in PLO and the source of the most common beginner mistakes:

- A player **must use exactly 2** of their 4 hole cards.
- A player **must use exactly 3** of the 5 community cards.
- **No exceptions.** A player cannot use 1 or 3 or 4 hole cards.

**Common mistake examples:**
- Board: A{s}K{s}Q{s}J{s}T{s}. Player holds 9{s}2{h}3{d}4{c}. The player does **NOT** have a flush. They must use exactly two hole cards. Their best hand uses 9{s} + one other card with three board cards.
- Board: 7{h}7{d}7{c}7{s}A{d}. Player holds K{s}Q{h}J{d}T{c}. The player does **NOT** have four of a kind. They can only use three board cards. Best hand: 7{h}7{d}7{c} (from board) + K{s}Q{h} (from hand) = three sevens with K-Q kickers. But another player with A{s}2{h}3{d}4{c} would make 7{h}7{d}7{c} + A{s}2{h}, which is also trips but with a better kicker via the board ace used differently. Actually, with four sevens on the board, everyone has quad sevens plus their highest hole card as kicker.

**Correction on the above:** When the board has four sevens, each player uses two hole cards + three board cards. They use three 7s from the board + their two highest hole cards. So the hand is 7-7-7-K-Q vs 7-7-7-A-2. The first player wins because K-Q > A-2... Actually no: 7-7-7 + K,Q vs 7-7-7 + A,2. The full five-card hand is three of a kind (7-7-7) with kickers. A-2 beats K-Q because the ace is the highest kicker. Second player wins.

### Tie-Breaking
Same rules as Hold'em. Compare from the highest-ranking component downward.

## Showdown Rules

- Same as Hold'em: last aggressor shows first; if no river bet, first player left of button shows first.
- Each player must clearly show all four hole cards for the hand to be valid.
- The dealer (or software) determines the best possible five-card hand from each player's four hole cards combined with the board, enforcing the two-and-three rule.

## Special Rules

### The Two-and-Three Rule in Detail
Because players have four hole cards, there are C(4,2) = 6 possible two-card combinations from the hole cards, and C(5,3) = 10 possible three-card combinations from the board. This yields 6 x 10 = 60 possible five-card hands per player. The best of these 60 is the player's hand.

### No "Playing the Board"
Unlike Hold'em, a player cannot "play the board." At least two hole cards must always be used. If the board is A-K-Q-J-T (a broadway straight), a player without two cards that contribute to a straight or better cannot claim that straight.

### Wraps (PLO-Specific Draws)
With four hole cards, PLO features "wraps" — straight draws with more outs than are possible in Hold'em. A 20-out wrap (the maximum) occurs when four consecutive or near-consecutive hole cards surround the board cards.

### Five-Card PLO (PLO-5)
A variant where players receive five hole cards instead of four. The two-and-three rule still applies: exactly two hole cards, exactly three community cards. This variant is growing in popularity but is a distinct game.

## Key Strategic Concepts

### Equity Runs Closer Together
With four hole cards generating six two-card combinations, hand equities are much closer in PLO than in Hold'em. The best preflop hand (AAKKds) has roughly 65% equity against a random hand, compared to ~85% for AA in Hold'em. This makes PLO a more postflop-oriented game.

### Position Is Even More Important
Because equities are closer and pots are built quickly via pot-sized bets, positional advantage is amplified. Playing out of position in PLO is significantly more costly than in Hold'em.

### Nut-Oriented Play
Because multiple players often have strong draws, PLO rewards playing for the nuts. Drawing to non-nut hands (e.g., the second-best flush draw, a low straight draw) is dangerous because opponents frequently hold the nut draw.

### Connectedness and Suitedness
Starting hand quality depends heavily on how cards work together:
- **Double-suited:** Two pairs of suited cards (e.g., A{s}K{s}Q{h}J{h}) — most valuable.
- **Connected:** Cards that form straight possibilities (e.g., T-9-8-7).
- **Dangler:** A card that doesn't connect with the other three, reducing hand quality.

### Hand Categories (Preflop)
- **Premium:** AAxx (double-suited), high rundowns (KQJT), big double-suited broadway hands.
- **Strong:** Single-suited aces with connectors, medium rundowns (9876ds).
- **Speculative:** Low rundowns, single-suited hands with some connectivity.
- **Trash:** Disconnected, unsuited hands with danglers (e.g., K{s}7{h}3{d}2{c}).

### Pot Control
Because pot-limit betting causes pots to grow exponentially (a pot-sized bet on each street creates a pot approximately 20x the original by the river), hand selection and early-street decisions are critical. A poor call on the flop cascades into a massive pot by the river.

### Variance
PLO is significantly higher variance than Hold'em. Closer equities and larger pots (relative to stack sizes) create larger swings. Bankroll requirements are higher.

### Blockers
With four hole cards, blockers (holding cards that reduce the probability of an opponent having a specific hand) are more strategically important. For example, holding the A{s} when three spades are on the board means no opponent can have the nut flush.

## Common Terminology

| Term | Definition |
|------|------------|
| **PLO** | Pot-Limit Omaha |
| **PLO-4** | Standard PLO with four hole cards (to distinguish from PLO-5) |
| **PLO-5** | Five-card Omaha variant |
| **Wrap** | A straight draw with 9 or more outs, only possible with four hole cards |
| **Rundown** | Four consecutive or near-consecutive hole cards (e.g., 9-8-7-6) |
| **Double-suited** | Hole cards containing two suited pairs (e.g., A{s}K{s}Q{h}J{h}) |
| **Single-suited** | One pair of suited cards among the four hole cards |
| **Rainbow** | No two hole cards share a suit |
| **Dangler** | A hole card that doesn't connect with the other three |
| **Nut draw** | A draw to the best possible hand |
| **Blocker** | Holding a card that reduces the likelihood of an opponent having a specific hand |
| **Pot** (verb) | To bet or raise the maximum allowed (the pot size) |
| **Two-and-three** | The rule requiring exactly two hole cards and three board cards |
| **Top set** | Three of a kind using the highest board card paired with a pocket pair |
| **Bottom set** | Three of a kind using the lowest board card |
| **Backdoor** | A draw requiring two remaining cards to complete (e.g., needing both turn and river) |
| **Nuttedness** | How often a hand makes the absolute best possible hand (the "nuts") |
| **Cooler** | A situation where two very strong hands collide unavoidably |
| **Scoop** | Winning the entire pot (relevant in hi-lo variants; in PLO-hi, every winner scoops) |

## State Space Analysis

### Information Sets
- Starting hands: C(52,4) = 270,725 raw combinations. After suit isomorphism: ~16,432 strategically distinct starting hands.
- Compare to Hold'em's 169 distinct starting hands — PLO is ~97× more complex pre-flop.
- Post-flop: each player's 4 hole cards produce C(4,2) = 6 possible two-card sub-hands to evaluate against the board. The effective hand space is 6× richer per player per street.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Starting hand combinations | 270,725 (raw), ~16,432 (strategic) |
| Post-flop hand evaluations per player | 6 sub-hands per board |
| Game tree nodes (HU, pot-limit) | ~10^80+ (estimated, no tight bound published) |
| Information sets (full, no abstraction) | Orders of magnitude beyond NLHE |
| Abstraction challenge multiplier vs NLHE HU | ~100× in card abstraction alone |

### Action Space
- Pot-limit constrains the maximum bet size, reducing the action space compared to no-limit.
- Typical discretization: fold/check, call, 1/3 pot, 1/2 pot, 2/3 pot, pot.
- Fewer extreme bet sizes than NLHE, but the hand complexity compensates.

## Key Challenges for AI/Solver Approaches

### 1. Starting Hand Complexity
With 270,725 raw starting hands (vs 1,326 in Hold'em), card abstraction is vastly more difficult. Equity distributions are more clustered — hands are closer in value, making abstraction errors more costly.

### 2. Equity Realization and Distribution
PLO hands run much closer in equity than Hold'em hands. Pre-flop equities rarely exceed 65-35, even for the best hands against the worst. This means:
- Less can be "decided" pre-flop; post-flop play is more critical.
- Strategies must be more nuanced about hand playability vs raw equity.
- The concept of "nutted" vs "non-nutted" ranges is central — hands that can make the nuts are disproportionately valuable.

### 3. Multi-Street Nut Advantage
In PLO, holding hands that can make the current and future nuts is critical. The solver must reason about:
- Current nut draws (e.g., flush draws, straight draws).
- Blockers (holding cards that prevent opponents from having specific hands).
- Board texture dynamics across streets.

### 4. Computational Intractability
The combination of larger starting hand space, pot-limit bet sizing, and multi-street play makes PLO significantly harder than NLHE for tabular solvers. No published work has produced a competitive PLO agent via pure CFR without extreme abstraction.

### 5. Hi/Lo Split Variant
PLO Hi/Lo adds an entire second dimension — the solver must reason about both high and low hands simultaneously, with scooping (winning both halves) being the primary strategic objective.

## Known Solver Results

### Commercial Solvers
- **PioSOLVER** (extended to PLO): can solve specific post-flop spots with given ranges, but pre-flop solutions are intractable without heavy abstraction.
- **MonkerSolver**: specifically designed for PLO. Pre-computes PLO solutions with simplified bet trees. The first commercial solver to offer full PLO pre-flop solutions with abstraction.
- **GTO Wizard**: cloud-based solver offering PLO solutions, using heavy abstraction.

### Academic Work
- No published academic work has demonstrated superhuman PLO play.
- CFR-based approaches struggle with the starting hand complexity — the abstraction required is so aggressive that solution quality degrades.
- Neural network approaches (DeepStack/ReBeL-style) are theoretically applicable but not yet demonstrated for PLO.

### Key Technical Results
- **Equity calculation**: PLO equity evaluation is ~6× slower than Hold'em per comparison due to the "must use exactly 2" constraint. Lookup-table-based evaluators exist but are larger.
- **Abstraction**: k-means clustering on equity distributions is the standard approach, but PLO requires finer granularity (more buckets) for equivalent solution quality.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2013 | Johanson et al., "Measuring the Size of Large No-Limit Poker Games" | Game tree size methodology (applicable to PLO) |
| 2019 | Brown & Sandholm, "Solving Imperfect-Information Games via Discounted Regret Minimization" | DCFR applicable to PLO blueprint computation |
| 2020 | Steinberger, "DREAM: Deep Regret minimization with Advantage baselines and Model-free learning" | Deep CFR variant potentially applicable to PLO |
| 2021 | Li et al., "Double Neural CFR" | Scalable deep CFR for large games |

## Relevance to Myosu

### Solver Applicability
PLO tests the **scalability limits** of solver architectures. It is the natural "next step" beyond NLHE:
- Same game structure, but dramatically larger information set space.
- Forces evaluation of whether architectures can handle richer hand spaces without unacceptable abstraction loss.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 3/5 | Feasible with extreme abstraction; quality suffers |
| Neural value network potential | 4/5 | Likely necessary for competitive play |
| Abstraction necessity | 5/5 | Dominant bottleneck |
| Real-time solving value | 5/5 | Critical given abstraction limitations |
| Transferability of techniques | 4/5 | Tests scaling of NLHE techniques |

### Myosu Subnet Considerations
- **Compute intensity**: PLO solving requires significantly more resources than NLHE. Solver nodes need adequate hardware specifications.
- **Evaluation difficulty**: comparing PLO strategies is harder because variance is higher (equities run closer), requiring more hands for statistically significant results.
- **Hand evaluation oracle**: must enforce the "exactly 2 hole cards" constraint. Non-trivial to implement efficiently.
- **Market demand**: PLO is the second most popular online poker variant after Hold'em, making it commercially relevant.
- **Abstraction quality as differentiator**: in PLO, the quality of card abstraction may matter more than the solving algorithm. This creates an interesting dimension for subnet competition.

### Recommended Approach for Myosu
1. Use Deep CFR or neural-guided MCCFR for blueprint computation (tabular CFR is likely insufficient).
2. Implement PLO-specific card abstraction with blocker-aware features.
3. Real-time subgame solving is essential — blueprint quality alone won't suffice.
4. Evaluate strategies with high sample counts (10,000+ hands minimum for statistical significance due to high variance).
5. Consider PLO as a "stress test" for solver architectures — performance here indicates scalability to other complex card games.
