# No-Limit Hold'em 6-Max (NLHE 6-Max)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | No-Limit Texas Hold'em, 6-Player (6-Max) |
| Variants | Cash game (100bb-200bb), various stack depths |
| Players | 6 (can be 2-6 with empty seats) |
| Information | Imperfect (hidden hole cards) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Not strictly (multiway pots, implicit collusion possible) |
| Solved Status | Unsolved; superhuman play demonstrated (Pluribus, 2019) |

## Overview

No-Limit Hold'em 6-Max is a community card poker game played with a maximum of six players per table. Each player receives two private hole cards and shares five community cards dealt to the board. The objective is to win chips by making the best five-card poker hand or forcing all opponents to fold. The 6-max format is the dominant online cash game format and increasingly common in live settings.

**Players:** 2 to 6
**Deck:** Standard 52-card deck, no jokers
**Betting structure:** No-limit (any bet up to a player's entire stack)

## Setup

### Deck and Cards
Standard 52-card deck. Cards are ranked A (high) through 2 (low). Aces may also play low in the straight A-2-3-4-5. Suits have no ranking.

### Positions
With six players, positions are named (in order of preflop action):

| Position | Abbreviation | Preflop Order | Postflop Order |
|----------|-------------|---------------|----------------|
| Under the Gun | UTG | 1st | Varies |
| Hijack | HJ | 2nd | Varies |
| Cutoff | CO | 3rd | Varies |
| Button | BTN | 4th (last preflop*) | Last |
| Small Blind | SB | 5th* | 1st |
| Big Blind | BB | 6th* (last preflop) | 2nd |

*Preflop, SB and BB act after the button but are forced bets. The BB is the last to act preflop (unless there is a raise, in which case action continues until all players have acted on the final raise).

With fewer than six players, positions are removed starting from UTG:
- 5 players: HJ, CO, BTN, SB, BB
- 4 players: CO, BTN, SB, BB
- 3 players: BTN, SB, BB
- 2 players: See heads-up rules (01-nlhe-hu.md)

### Blinds
- **Small Blind (SB):** Posted by the player immediately left of the button. Typically half the big blind.
- **Big Blind (BB):** Posted by the player two seats left of the button. This sets the minimum bet size for the hand.
- Common stakes notation: "1/2" means SB = 1, BB = 2. "5/10" means SB = 5, BB = 10.

### Dealing
1. Both blinds are posted.
2. Dealer burns one card.
3. Starting with the SB and proceeding clockwise, each player receives one card face-down.
4. A second round of dealing gives each player their second hole card (same order: SB first, BTN last).
5. Each player now has exactly two private hole cards.

## Game Flow

### 1. Preflop
- Action begins with UTG (the player to the left of the BB) and proceeds clockwise.
- Each player may fold, call the big blind, or raise.
- Blinds are live: SB must complete to the current bet or fold; BB may check if no raise, or call/raise/fold if facing a raise.
- Betting continues around the table until all active players have contributed equally or folded.

### 2. Flop
- Dealer burns one card, then deals three community cards face-up.
- Action begins with the first active player to the left of the button (this will be SB if still in, otherwise BB, and so on clockwise).
- Players may check (if no bet) or bet. If a bet is made, subsequent players may fold, call, or raise.

### 3. Turn
- Dealer burns one card, then deals one community card face-up (fourth card).
- Betting proceeds identically to the flop.

### 4. River
- Dealer burns one card, then deals one final community card face-up (fifth card).
- Betting proceeds identically to the flop and turn.

### 5. Showdown
- If two or more players remain after river betting, hands are revealed.
- The best five-card hand wins the pot. If hands are identical, the pot is split.

## Betting Rules

### No-Limit Structure
- A player may bet any amount from the minimum up to their entire stack.
- **Minimum bet:** Equal to the big blind on any street.
- **Minimum raise:** The raise increment must be at least equal to the largest prior raise increment in the current betting round.
  - Example: Blinds 1/2. UTG raises to 6 (raise increment = 4, above the BB amount of 2). HJ's minimum re-raise is to 10 (6 + 4). CO's minimum re-raise (4-bet) is to 14 (10 + 4).
  - Example: On the flop, Player A bets 8. Player B raises to 24 (increment = 16). Player C's minimum re-raise is to 40 (24 + 16).
- **Maximum bet:** A player's entire stack (all-in).
- There is no cap on the number of raises in no-limit.

### All-In Rules
- A player may go all-in at any point for any amount up to their stack.
- **Under-raise all-in:** If an all-in is less than a full raise increment, it does not reopen the betting to players who have already acted on the current raise level.
  - Example: Blinds 1/2. BTN raises to 6. SB goes all-in for 9 (increment of 3, less than the required 4). BB may fold, call 9, or raise (since BB has not yet acted on BTN's raise). But if BTN had already been the one facing the all-in after acting, BTN could only fold or call.
- **Full raise all-in:** If the all-in constitutes at least a full raise, it reopens action to all players.

### Side Pots
When a player goes all-in and other players continue betting:
1. A **main pot** is created containing the all-in player's contribution matched by each remaining player (up to that amount).
2. A **side pot** is created from any additional bets. The all-in player cannot win the side pot.
3. Multiple side pots can exist if multiple players go all-in for different amounts.
4. At showdown, side pots are awarded first (from the most recent to the earliest), then the main pot.

**Side pot calculation:**
- Player A goes all-in for 50. Player B calls. Player C raises to 200. Player B calls 200.
- Main pot: 50 x 3 = 150 (all three eligible).
- Side pot: (200 - 50) x 2 = 300 (only B and C eligible).
- Player A can win at most 150. Players B and C contest both pots.

### Pot Size
The pot consists of all chips bet in the current hand including the blinds and all previous betting rounds. This total is relevant for strategic calculations (pot odds) but does not restrict bet sizing in no-limit.

## Hand Rankings

Identical to standard poker hand rankings. See 01-nlhe-hu.md for the complete table.

| Rank | Hand | Example |
|------|------|---------|
| 1 | Royal Flush | A{s}K{s}Q{s}J{s}T{s} |
| 2 | Straight Flush | 8{h}7{h}6{h}5{h}4{h} |
| 3 | Four of a Kind | 9{d}9{s}9{h}9{c}A{s} |
| 4 | Full House | K{s}K{h}K{d}6{c}6{s} |
| 5 | Flush | A{c}T{c}7{c}4{c}2{c} |
| 6 | Straight | Q{s}J{h}T{d}9{c}8{s} |
| 7 | Three of a Kind | 8{s}8{h}8{d}A{c}J{s} |
| 8 | Two Pair | A{s}A{h}7{d}7{c}K{s} |
| 9 | One Pair | J{s}J{h}A{d}9{c}4{s} |
| 10 | High Card | A{s}K{h}T{d}7{c}3{s} |

### Tie-Breaking Rules
- **Straight Flush / Straight:** Highest top card wins. A-2-3-4-5 is the lowest (ace plays low).
- **Four of a Kind:** Higher quad rank wins, then kicker.
- **Full House:** Higher trips first, then higher pair.
- **Flush:** Compare highest to lowest, first difference wins.
- **Three of a Kind:** Higher trips, then compare kickers.
- **Two Pair:** Higher top pair, then second pair, then kicker.
- **One Pair:** Higher pair, then compare kickers.
- **High Card:** Compare highest to lowest.
- **Identical hands:** Pot is split equally among tied players.

### Best Five-Card Hand
Each player constructs their best five-card hand from seven available cards (two hole cards + five community cards). A player may use both, one, or zero of their hole cards.

## Showdown Rules

- If all action is complete on the river:
  - The last aggressor (player who made the final bet or raise) shows first.
  - If no bet was made on the river (all checked), the first active player to the left of the button shows first.
- Remaining players may show or muck (in clockwise order).
- A player who can beat the shown hand must show to claim the pot (some rooms auto-award).
- If all remaining players are all-in, all hands are revealed before remaining community cards are dealt.

### Multiway Showdown
When three or more players reach showdown:
- Each player's best five-card hand is compared.
- The best hand wins the pot. If multiple players tie, the pot is split equally among them.
- Side pots are resolved separately; a player can only win pots they are eligible for.

## Special Rules

### Missed Blinds
- If a player misses one or both blinds (by being absent), they must post the missed blind(s) upon returning before being dealt in.
- A missed small blind is "dead" (goes directly to the pot and does not count toward the player's bet).
- A missed big blind is "live" (counts toward the player's bet in the first round).

### New Player Seating
- A new player joining the table may either wait for the big blind to reach them or post an amount equal to the big blind to be dealt in immediately.
- A new player may not be seated in the small blind or on the button.

### Button Movement
- The button moves one position clockwise after each hand.
- **Dead button rule:** If the player who would receive the button has been eliminated or left, the button moves to the next eligible position, but the blinds are posted in their normal positions. This can result in a "dead" small blind (no player in the SB position) to maintain proper blind progression.

### Straddle (Cash Games)
- A straddle is a voluntary blind bet, typically 2x the BB, posted by UTG before cards are dealt.
- The straddle acts as a third blind. Action preflop starts with the player left of the straddler.
- The straddler acts last preflop (similar to the BB's option).
- Not permitted in all rooms. Some rooms allow button straddles or Mississippi straddles.

### Rake
In cash games, the house takes a percentage of each pot (the "rake"), typically 2.5%-5% up to a fixed cap. This does not exist in tournament play (buy-in fees cover the house).

## Key Strategic Concepts

### Position Advantage
Position is the most important structural factor. Players acting later in a betting round have more information. The BTN is the most profitable seat; the blinds are the least profitable because they act last preflop but first postflop.

### Opening Ranges by Position
Starting hand selection widens as position improves:
- **UTG:** Tightest range (~15-20% of hands). Only strong starting hands.
- **HJ:** Slightly wider (~18-23%).
- **CO:** Significantly wider (~25-30%).
- **BTN:** Widest opening range (~40-50%). Benefits from guaranteed position postflop.
- **SB:** Opens wide when folded to (~40-50%) but faces positional disadvantage postflop.

### 3-Betting
Re-raising preflop is more frequent in 6-max than in full-ring because ranges are wider. 3-bets serve both as value bets (with premium hands) and bluffs (to deny equity and win pots immediately).

### Stealing Blinds
With only two players in the blinds, late-position players frequently raise with the intent of picking up the dead money. Blind defense frequency is a critical counter-adjustment.

### Multiway Pot Dynamics
When three or more players see a flop, hand equities run closer together, bluffing becomes less effective, and hand strength requirements increase compared to heads-up pots.

### Stack Depth
Standard cash game buy-in is 100 BB. At deeper stacks (200+ BB), implied odds increase and speculative hands (suited connectors, small pairs) gain value. At shorter stacks (40-60 BB), play becomes more preflop-centric.

### Aggression and Initiative
The preflop raiser has "initiative" and can leverage this with continuation bets. Maintaining aggression across streets applies pressure and forces opponents into difficult decisions.

## Common Terminology

| Term | Definition |
|------|------------|
| **UTG** | Under the Gun; first to act preflop |
| **HJ** | Hijack; two seats right of the button |
| **CO** | Cutoff; one seat right of the button |
| **BTN** | Button; dealer position, last to act postflop |
| **SB** | Small Blind; forced half-bet left of the button |
| **BB** | Big Blind; forced full bet two left of the button |
| **Open** | First voluntary raise preflop |
| **3-bet** | A re-raise of the initial open-raise |
| **4-bet / 5-bet** | Subsequent re-raises (4th bet, 5th bet in the sequence) |
| **C-bet** | Continuation bet; a bet by the preflop aggressor on the flop |
| **Double barrel** | C-betting the flop and following with a turn bet |
| **Triple barrel** | Betting flop, turn, and river |
| **Squeeze** | 3-betting after an open-raise and one or more callers |
| **Isolate** | Raising to narrow the field (typically against a limper) |
| **Steal** | Raising from late position to win the blinds uncontested |
| **Defend** | Calling or raising from the blinds against a steal attempt |
| **Float** | Calling a bet (often on the flop) with intent to take the pot away later |
| **Probe bet** | Betting into the preflop aggressor when they check back (out of position) |
| **Check-back** | Declining to bet when last to act |
| **Pot odds** | Ratio of the current pot to the cost of a call |
| **Implied odds** | Additional chips expected to be won on future streets if a drawing hand hits |
| **Equity** | The percentage of the pot a hand expects to win at showdown |
| **Fold equity** | Additional value from the probability that opponents fold |
| **SPR** | Stack-to-pot ratio after the flop; determines postflop commitment dynamics |
| **Effective stack** | The smallest stack among involved players; limits the maximum amount at risk |
| **Rake** | The house fee taken from each pot in cash games |

## State Space Analysis

### Information Sets
- Pre-flop: 169 strategically distinct starting hands per player × 6 positions × branching from prior actions.
- The multiplayer structure causes a combinatorial explosion in the number of possible action histories.
- Estimated information sets (full game): orders of magnitude beyond NLHE HU — no published tight bound, but roughly 10^170+ for the raw game tree.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Raw game tree nodes | >10^170 (far beyond HU) |
| Information sets (no abstraction) | Intractable |
| Information sets (with abstraction) | ~10^12 (Pluribus-level blueprint) |
| Branching factor increase vs HU | ~3-5× per street due to more players acting |

### Action Space
- Same continuous bet space as NLHE HU, requiring discretization.
- Additional complexity: must model responses from up to 5 remaining opponents per decision point.
- Pluribus used 200 bet sizes in the blueprint and 4-8 in subgame solving.

## Key Challenges for AI/Solver Approaches

### 1. Non-Zero-Sum Structure
With 3+ players, the game is no longer two-player zero-sum. Nash equilibria are:
- Not unique (infinitely many Nash equilibria exist).
- Not necessarily interchangeable (mixing strategies from different equilibria can be exploitable).
- Not guaranteed to be the "best" strategy — can be exploited by colluding opponents.
- Computationally harder: no known polynomial-time algorithm for finding Nash equilibria in general-sum games (PPAD-hard).

### 2. Implicit Collusion and Coalition Effects
Even without explicit collusion, multi-player equilibria can involve coordinated punishment strategies. This complicates the strategy space — a solver must account for the possibility that opponents may (even unintentionally) play in correlated ways.

### 3. Blueprint Scalability
The game tree is exponentially larger than HU. Blueprint strategies must be heavily abstracted, and the quality of abstraction becomes even more critical.

### 4. Real-Time Subgame Solving in Multiway Pots
Subgame solving with more than 2 players is theoretically unsound in general — resolving a subgame for one player changes the incentives for others. Pluribus addressed this with a novel approach but it's not a complete theoretical solution.

### 5. Opponent Modeling Without Exploitation Guarantees
In multiplayer games, there's no clean theory for safely exploiting weak opponents without becoming exploitable to others. The explore-exploit tradeoff is multi-dimensional.

## Known Solver Results

### Pluribus (Brown & Sandholm, 2019)
The landmark result for multiplayer poker AI:
- Defeated elite human professionals in 10,000 hands of 6-max NLHE.
- Published in *Science*.
- Key innovations:
  1. **Blueprint strategy via MCCFR with Linear CFR weighting**: trained on an abstracted game using Monte Carlo CFR with linear weighting (later iterations get more weight).
  2. **Depth-limited search with modified action abstraction**: at decision time, Pluribus constructs a subgame and searches 4 streets ahead.
  3. **Opponent modeling via "search policy"**: instead of tracking specific opponent tendencies, Pluribus assumes each opponent will play according to a modified version of the blueprint (with perturbations to model different play styles).
  4. **No neural networks**: Pluribus used no deep learning, running on a single machine with 128GB RAM and 2 CPUs. Blueprint computed in 8 days on a 64-core server.

### Theoretical Limitations
- No exploitability bound: unlike HU solvers, there's no meaningful exploitability metric for multiplayer Nash equilibria.
- The strategy Pluribus plays is not a Nash equilibrium — it's an approximation with no formal guarantees, validated only empirically against human experts.

### Other Notable Work
- **Slumbot** (unrestricted NLHE bots, Annual Computer Poker Competition): various entries have competed in multi-player formats.
- **Student of Games** (Schmid et al., 2023): a general game-playing agent combining search with learned models, evaluated on poker among other games.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2019 | Brown & Sandholm, "Superhuman AI for Multiplayer Poker" | Pluribus, first superhuman 6-max agent (*Science*) |
| 2017 | Brown & Sandholm, "Safe and Nested Subgame Solving for Imperfect-Information Games" | Theoretical foundations for subgame solving |
| 2019 | Brown & Sandholm, "Solving Imperfect-Information Games via Discounted Regret Minimization" | DCFR, improved convergence for large games |
| 2020 | Farina et al., "Stable-Predictive Optimistic CFR" | Faster CFR convergence |
| 2023 | Schmid et al., "Player of Games" | General agent framework including multiplayer poker |

## Relevance to Myosu

### Solver Applicability
NLHE 6-max is the **critical multiplayer benchmark**. It tests whether solver architectures can handle:
- **Multiplayer CFR scaling**: Blueprint computation for 6 players is feasible but expensive.
- **Search without theoretical guarantees**: Pluribus-style subgame search is heuristic in multiplayer settings.
- **Blueprint + search architectures**: the two-phase approach (offline blueprint, online search) is the proven paradigm.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 4/5 | Works but no convergence guarantees to Nash in n-player |
| Neural value network potential | 4/5 | Pluribus didn't use NN, but NN approaches viable |
| Abstraction necessity | 5/5 | Even more critical than HU |
| Real-time solving value | 5/5 | Essential for competitive play |
| Transferability of techniques | 5/5 | Multiplayer search applies to many games |

### Myosu Subnet Considerations
- **Validation challenge**: no clean exploitability metric for multiplayer strategies. Evaluation must be empirical (win rates over many hands against diverse opponents).
- **Compute requirements**: Pluribus ran on commodity hardware — this is important for decentralized solver nodes.
- **Strategy representation**: blueprint strategies for 6-max are larger than HU but still compressible.
- **Anti-collusion**: the subnet must address the possibility of colluding solver submissions in multiplayer evaluation games.
- **Position-dependent strategy**: strategies vary significantly by position, adding a dimension to evaluation.

### Recommended Approach for Myosu
1. Use MCCFR (external sampling or public-chance sampling) with linear weighting for blueprint computation.
2. Implement Pluribus-style depth-limited search for real-time play.
3. Evaluate strategies via head-to-head round-robin tournaments across positions.
4. Use variance-reduction techniques (AIVAT, or dealing identical cards across matches) for faster convergence of evaluation metrics.
5. Consider NLHE 6-max as the primary "hard" poker variant for ranking — its multiplayer nature makes it strictly more challenging than HU.
