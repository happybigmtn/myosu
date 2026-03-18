# No-Limit Hold'em Tournament (NLHE MTT)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | No-Limit Texas Hold'em Tournament (Multi-Table / Sit & Go) |
| Variants | MTT (multi-table), SNG (sit & go), Spin & Go, satellite, bounty |
| Players | 2-10 per table; 2-10,000+ in tournament |
| Information | Imperfect (hidden hole cards, opponent stack sizes visible) |
| Stochasticity | Stochastic (card deals, blind level increases) |
| Zero-Sum | No — ICM makes chip equity non-linear |
| Solved Status | Unsolved; no superhuman tournament AI |

## Overview

No-Limit Hold'em Tournament (Multi-Table Tournament, MTT) is a poker competition where all players start with an equal number of chips and play until one player holds all the chips. Players are eliminated when they lose all their chips. Prize money is distributed according to a payout structure based on finishing position. The game mechanics follow standard NLHE rules, but the tournament wrapper introduces blind escalation, eliminations, table balancing, and payout structures that fundamentally alter strategy.

**Players:** Varies (2 to thousands); tables seat 6 (6-max) or 9 (full-ring)
**Deck:** Standard 52-card deck, no jokers
**Betting structure:** No-limit
**Duration:** Continues until one player remains (or a deal is made)

## Setup

### Starting Stacks
All players receive an identical number of tournament chips at the start. Common starting stacks:
- 10,000 chips (standard)
- 20,000 to 60,000 chips (deep-stacked events)
- Starting stack size relative to the initial big blind determines the depth of play.

### Tournament Chips vs. Real Money
Tournament chips have no direct cash value. A player with 50,000 chips is not worth exactly 5x a player with 10,000 chips in prize equity. The conversion from chips to prize money is nonlinear (see ICM below).

### Blind Structure
Blinds increase on a fixed schedule throughout the tournament. A blind structure defines:
- **Levels:** Timed intervals (typically 15-60 minutes for live; 3-15 minutes for online).
- **Blind amounts:** SB and BB for each level, escalating over time.
- **Antes:** Introduced at a specified level. Standard antes are typically 10-12.5% of the BB.

**Example blind schedule:**

| Level | SB | BB | Ante | Duration |
|-------|-----|------|------|----------|
| 1 | 25 | 50 | 0 | 40 min |
| 2 | 50 | 100 | 0 | 40 min |
| 3 | 75 | 150 | 0 | 40 min |
| 4 | 100 | 200 | 25 | 40 min |
| 5 | 150 | 300 | 50 | 40 min |
| 6 | 200 | 400 | 50 | 40 min |
| 7 | 300 | 600 | 75 | 40 min |
| 8 | 400 | 800 | 100 | 40 min |

### Ante Formats
- **Traditional ante:** Every player at the table posts an ante each hand. Creates significant dead money.
- **Big Blind ante (BBA):** The player in the big blind posts the ante for the entire table. This is the modern standard in most major tournament series. The BB posts both the big blind and the ante.
  - When BBA is less than the BB, the BB player posts BB + ante.
  - When BBA equals the BB (common in later levels), the BB effectively posts 2x BB.

### Seating
Players are randomly assigned to tables and seats. As players are eliminated, tables are broken and remaining players are consolidated to maintain balanced table sizes.

## Game Flow

### Hand-Level Play
Each hand follows standard NLHE rules (see 01-nlhe-hu.md or 02-nlhe-6max.md for details):
1. Blinds and antes are posted.
2. Two hole cards are dealt to each player.
3. Preflop betting round (UTG acts first).
4. Flop (3 community cards) + betting round.
5. Turn (1 community card) + betting round.
6. River (1 community card) + betting round.
7. Showdown (if applicable).

### Tournament-Level Flow
1. **Registration:** Players buy in and receive starting stacks.
2. **Early stages:** Low blinds relative to stacks. Deep-stacked play.
3. **Middle stages:** Blinds increase, short stacks emerge, eliminations accelerate.
4. **Bubble:** The point just before the payout threshold. Play tightens dramatically.
5. **In the money (ITM):** All remaining players are guaranteed a payout.
6. **Final table:** The last table remaining (typically 6 or 9 players).
7. **Heads-up:** The final two players. Button/blind rules switch to heads-up format (see 01-nlhe-hu.md).
8. **Winner:** The last player standing.

## Betting Rules

### No-Limit Structure
Identical to cash game NLHE:
- Minimum bet equals the big blind.
- Minimum raise increment equals the previous raise increment.
- Maximum bet is a player's entire stack (all-in).
- No cap on number of raises.

### All-In and Side Pots
Side pot rules are identical to cash game NLHE (see 02-nlhe-6max.md). Multiple side pots may be created when several players of different stack sizes are all-in.

### Eliminations from All-In Confrontations
- When a player loses all their chips, they are eliminated from the tournament.
- If multiple players are eliminated on the same hand at the same table, the player who started the hand with more chips finishes in the higher position (lower finishing number = better placement).
- If multiple players are eliminated on the same hand at different tables, they tie for the finishing position and split the associated prize money equally.

## Hand Rankings

Identical to standard NLHE. See 01-nlhe-hu.md for the complete table and tie-breaking rules.

## Showdown Rules

Identical to cash game NLHE. See 02-nlhe-6max.md for multiway showdown rules.

## Special Rules

### Table Balancing
- Tables must remain within one player of each other in count (e.g., if one table has 7 players and another has 5, a player is moved).
- Players are moved to balance tables, typically from the seat that will most disadvantage them the least (e.g., avoiding moving a player into the blinds if possible). The standard TDA rule moves a player from the position that would next receive the big blind.
- When a table breaks, players are randomly reassigned to open seats at remaining tables.

### Dead Button
When a player is eliminated from a blind position:
- The **dead button rule** applies: blinds are posted in their correct positions, but the button may not advance past a seat that should have the button.
- This can result in a "dead small blind" (no player in SB position; the SB amount goes into the pot as dead money from no player, or the SB is simply not posted and the pot starts with just the BB + antes).
- The key principle: no player misses the big blind, and no player pays the big blind twice in succession due to eliminations.

### Chip Race / Color Up
When a denomination is removed from play (because blinds have escalated beyond it):
1. All chips of the expiring denomination are collected.
2. Players receive the maximum number of higher-denomination chips their total converts to.
3. Remaining odd chips are "raced off": each player with odd chips receives one card per odd chip. The highest card wins one chip of the new denomination.
4. **A player cannot be eliminated by a chip race.** If a player loses all their chips in a race-off, they receive one chip of the minimum denomination.

### Late Registration and Re-Entry
- **Late registration:** Many tournaments allow new players to enter during the early levels (typically first 6-8 levels). Late entrants receive a starting stack.
- **Re-entry:** Some tournaments allow eliminated players to buy back in during the late registration period.
- **Rebuy:** In rebuy tournaments, players may purchase additional chips (usually up to a certain point in the tournament). This is a distinct format.

### Bubble Play
- **The bubble** is the situation when one more elimination will put all remaining players in the money.
- **Hand-for-hand:** During bubble play, all tables play one hand simultaneously, then pause until all tables complete the hand. This prevents stalling.
- If multiple eliminations occur on the bubble during the same hand-for-hand deal:
  - Same table: player with the larger starting stack for that hand finishes higher.
  - Different tables: players tie and split the relevant prize positions.

### Heads-Up Transition
When two players remain:
- Button placement follows the standard rule: the button/SB was the player who was in the BB when 3-handed play ended (or the button moves to the player who would receive it in normal rotation).
- Action order changes: BTN/SB acts first preflop, BB acts first postflop. See 01-nlhe-hu.md.

### Penalties
Tournament directors may issue penalties for violations:
- **Verbal warning:** Minor infractions.
- **Missed hands:** Player is dealt out for a specified number of hands.
- **Round penalty:** Player is dealt out for an entire blind level.
- **Disqualification:** Severe infractions (threats, collusion, etc.).

### Clock and Time Limits
- Each player has a reasonable time to act (varies by event, typically 30-60 seconds).
- Any player may request a clock be put on another player. The floor is called and the player is given a fixed time (typically 60 seconds) to act, after which their hand is declared dead.
- Online tournaments use automatic time banks.

### Deal-Making
At the final table, remaining players may agree to redistribute the remaining prize pool:
- **ICM chop:** Split remaining prizes based on ICM equity (see below).
- **Chip chop:** Split based on current chip counts (less common, disadvantages short stacks).
- All players must agree. The tournament may continue for the title/trophy even after a financial deal.

## Payout Structure

### Standard Payout Distribution
Approximately 10-15% of the field is paid. Top-heavy distribution, with the winner receiving the largest share.

**Example payout for a 100-player tournament (10% paid):**

| Place | Payout (% of pool) |
|-------|-------------------|
| 1st | 25% |
| 2nd | 15% |
| 3rd | 10% |
| 4th | 8% |
| 5th | 6.5% |
| 6th | 5.5% |
| 7th | 4.5% |
| 8th | 4% |
| 9th | 3.5% |
| 10th | 3% |

Exact percentages vary by operator. Larger fields typically pay a higher percentage of entrants but with a more top-heavy distribution.

### ICM (Independent Chip Model)
ICM converts tournament chip stacks into estimated prize equity:
- It considers all remaining players' stack sizes and the full payout structure.
- A player's ICM equity is the probability-weighted sum of all possible finishing positions.
- **Key property:** ICM equity is nonlinear. Doubling your chip stack does **not** double your prize equity. The first chip is always worth more than the last chip.
- **Implication:** Unlike cash games, risking your tournament life for a marginal edge is often incorrect because the cost of elimination (losing all equity) outweighs the gain of doubling up.

### Bubble Factor
The bubble factor quantifies the ICM tax on risk-taking:
- Bubble factor = (equity lost if you lose the hand) / (equity gained if you win the hand).
- In a cash game, bubble factor = 1.0 (symmetric risk/reward).
- Near the bubble with a medium stack, bubble factor may be 1.5-3.0+, meaning you need significantly more equity to justify calling an all-in.

## Key Strategic Concepts

### Survival vs. Chip Accumulation
Tournament strategy balances two competing objectives:
- **Accumulating chips** to win first place (the largest prize).
- **Surviving** to reach higher payout tiers.
The optimal balance depends on stack size, blind level, proximity to the money, and payout structure.

### Stack Size Categories (in big blinds)
| Stack | Category | Strategic Mode |
|-------|----------|----------------|
| 60+ BB | Deep | Full postflop poker; wide ranges, speculative hands have value |
| 30-60 BB | Medium | Standard open/3-bet/fold; postflop play with less room for error |
| 15-30 BB | Short | Re-steal and shove/fold situations emerge; limited postflop play |
| 10-15 BB | Danger zone | Push/fold with occasional open-raise; very limited postflop |
| < 10 BB | Critical | Pure push/fold strategy |

### ICM Pressure
- **Big stacks** exert ICM pressure on medium stacks by threatening elimination.
- **Medium stacks** face the most ICM pressure — they have too much to lose and must play tighter.
- **Short stacks** have less ICM pressure (they have the least equity to lose) and can take more risks.
- **On the bubble:** ICM effects are maximized. Even big stacks tighten up against other big stacks but exploit short stacks.

### Blind Escalation and Urgency
As blinds increase, stack-to-pot ratios decrease and action becomes more preflop-centric. Players must accumulate chips or be blinded out. This creates urgency to take risks, especially with shorter stacks.

### Changing Gears
Effective tournament players adjust their strategy as conditions change:
- **Early levels:** Play for postflop value. Avoid unnecessary risks. Build reads.
- **Middle levels:** Open up stealing, attack tight players, begin to accumulate.
- **Late stages / final table:** ICM considerations dominate. Exploit bubble pressure.
- **Heads-up:** Pure chip EV poker returns (ICM no longer applies with two players).

### Pay Jumps
Large pay jumps between positions (e.g., 3rd vs. 2nd, 2nd vs. 1st) should influence risk-taking. Players should take more risks to secure a significant pay jump and fewer risks when the next pay jump is small relative to their current equity.

### Independent vs. Table-Dependent Decisions
Unlike cash games, tournament decisions are not independent. Chips gained or lost affect your equity in the overall tournament, and the actions of players at other tables matter (e.g., a short stack at another table who might bust before you need to risk your chips).

## Common Terminology

| Term | Definition |
|------|------------|
| **MTT** | Multi-table tournament |
| **SNG** | Sit-and-go; a single-table (or small multi-table) tournament that starts when enough players register |
| **Buy-in** | The entry fee for the tournament (e.g., "$100+$10" means $100 to the prize pool, $10 to the house) |
| **Starting stack** | The number of chips each player receives at the start |
| **Blind level** | A period of play at a fixed blind/ante amount |
| **Level up** | When blinds increase to the next scheduled level |
| **ITM** | In the money; all remaining players are guaranteed a payout |
| **Bubble** | The situation where one more elimination reaches the payout threshold |
| **Bubble boy** | The last player eliminated before the money |
| **Stone bubble** | The exact point of being one elimination from the money |
| **Final table** | The last remaining table in a multi-table tournament |
| **Chip leader** | The player with the most chips at any given point |
| **Short stack** | A player with significantly fewer chips than average |
| **Average stack** | Total chips in play divided by remaining players |
| **M-ratio (Harrington's M)** | Stack size divided by (SB + BB + total antes). Indicates how many orbits a player can survive passively |
| **ICM** | Independent Chip Model; converts chip stacks to prize equity |
| **Bubble factor** | Ratio of equity lost (if you lose) to equity gained (if you win) |
| **cEV** | Chip expected value; EV measured in chips, ignoring ICM |
| **$EV** | Dollar expected value; EV measured in real money via ICM |
| **Push/fold** | Strategy at short stacks: go all-in or fold; no calling or raising |
| **Shove** | Go all-in |
| **Open-shove** | Go all-in as the first raiser (no prior action) |
| **Resteal** | Re-raising a late-position raise (often with a shove) to pick up the pot |
| **Fold equity** | The additional value from opponents potentially folding |
| **Hand-for-hand** | Synchronized play during the bubble |
| **Color up / chip race** | Removing lower denominations from play |
| **Table draw** | Random table/seat assignment |
| **Break** | Scheduled pause in play (typically every 2 hours) |
| **Late reg** | Late registration period |
| **Overlay** | When the guaranteed prize pool exceeds total buy-ins |
| **Chop** | Agreement among remaining players to divide the prize pool |
| **Deal** | Same as chop; a negotiated redistribution of remaining prizes |

## State Space Analysis

### Information Sets
- Inherits all NLHE information set complexity.
- Additional state variables: stack sizes of all players at all tables, blind level, proximity to bubble/payout jumps, ICM equity.
- The ICM component makes every decision dependent on the entire tournament state, not just the current table.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Single hand game tree | Same as NLHE (varies by players at table) |
| Tournament game tree | Intractable — must model sequence of hands across changing conditions |
| ICM decision space | Continuous (stack distributions), typically discretized |
| Effective state variables | Chip stack vector + blind level + payout structure + hand state |

### Action Space
- Same as NLHE per hand: fold/check/call/bet/raise.
- Meta-decisions: stack preservation, ICM-aware ranges, bubble play adjustments.
- Additional strategic dimensions in bounty tournaments (bounty value affects calling ranges).

## Key Challenges for AI/Solver Approaches

### 1. Non-Zero-Sum via ICM
This is the fundamental challenge. In a cash game, winning 100 chips from one opponent is equivalent regardless of who you win from. In a tournament:
- Winning 100 chips from a big stack is worth less than winning 100 from a short stack (relative to ICM equity).
- Busting a player near the bubble creates "ICM equity" for all surviving players.
- The utility function is a complex, non-linear function of all players' stack sizes.

### 2. Dynamic Game Parameters
- Blind levels increase, changing the effective stack depth.
- Table compositions change as players bust and tables merge.
- The payout structure creates varying incentive gradients (e.g., the jump from 10th to 9th may be worth more or less than 5th to 4th).

### 3. Multi-Table Reasoning
In MTTs, optimal play at your table depends on what's happening at other tables:
- Near the bubble, if a short stack at another table is likely to bust soon, you should tighten your calling ranges.
- This requires information about other tables, which is available in online poker but creates a massive state space.

### 4. Changing Opponent Pool
As players bust, the remaining field becomes stronger (weak players eliminated first, on average). The solver must adapt to evolving opponent quality.

### 5. Push/Fold Theory
At short stack depths (<15bb), the game reduces to a push-or-fold decision. This is the one regime where near-optimal tournament play is well-understood:
- **Nash Push/Fold Charts** (with ICM adjustments) provide close-to-optimal short-stack play.
- The SAGE (Sit And Go Endgame) system and ICMizer-style tools solve these endgame scenarios exactly.

## Known Solver Results

### ICM Solvers
- **ICMizer / HoldemResources Calculator**: compute ICM-adjusted push/fold ranges for SNG endgames. Essentially solved for the push/fold regime.
- **ICMIZER3**: extends to post-flop play with ICM-adjusted decision making, though with significant abstraction.
- **Chip EV + ICM adjustment**: most practical approaches solve for chip EV first, then apply ICM corrections.

### Academic Work
- **Nash equilibrium push/fold**: Miltersen & Sorensen (2007) computed Nash equilibria for SNG push/fold games with ICM.
- No published work has solved full post-flop tournament play with ICM.
- The non-zero-sum nature of ICM makes CFR convergence guarantees inapplicable without modification.

### Practical Approaches
- **Blueprint + ICM overlay**: compute a standard NLHE strategy, then adjust ranges and bet sizes based on ICM considerations.
- **Stack-depth indexed strategies**: pre-compute strategies for different effective stack depths, interpolating based on current tournament state.
- **Final table solvers**: specialized tools that solve final table scenarios with known stack distributions and payout structures.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2000 | Malmuth & Harville, "ICM foundations" | Independent Chip Model formulation |
| 2007 | Miltersen & Sorensen, "Computing Proper Equilibria of Zero-Sum Games" | Nash equilibria for SNG push/fold |
| 2015 | Kovacic, "Tournament Poker ICM" | Comprehensive ICM analysis |
| 2019 | Chen & Ankenman, "Mathematics of Poker" | Risk-premium and tournament theory |

## Relevance to Myosu

### Solver Applicability
Tournament NLHE tests whether architectures can handle **non-stationary, non-zero-sum** environments:
- **CFR**: not directly applicable due to non-zero-sum ICM structure. Requires modification (e.g., solving for chip EV then applying ICM correction, or reformulating as a team game).
- **Reinforcement learning**: potentially better suited than CFR for the non-stationary, multi-stage nature of tournaments.
- **Neural value estimation**: must learn ICM-dependent value functions, adding state dimensions.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 2/5 | Non-zero-sum ICM breaks standard CFR guarantees |
| Neural value network potential | 4/5 | Can learn ICM-aware valuations |
| Abstraction necessity | 5/5 | Must abstract tournament state as well as card state |
| Real-time solving value | 4/5 | ICM adjustments change optimal play continuously |
| Transferability of techniques | 3/5 | Tournament-specific; less transfer to non-tournament games |

### Myosu Subnet Considerations
- **Evaluation complexity**: tournament strategy quality requires large sample sizes due to high variance and the multi-hand nature of tournaments.
- **ICM verification oracle**: the subnet needs a correct ICM calculator to verify strategy quality in tournament contexts.
- **Format diversity**: MTTs, SNGs, and bounty tournaments each require different solver adaptations.
- **Practical relevance**: tournaments are the most popular competitive poker format, making this commercially important.
- **Partial observability of other tables**: in MTTs, information about other tables is available but adds state space complexity. The subnet must decide how much table-level information to include.

### Recommended Approach for Myosu
1. Solve for chip EV using standard NLHE techniques (CFR/MCCFR).
2. Apply ICM adjustments via a learned correction model (neural network trained on ICM equity differences).
3. For evaluation, use SNG formats (single-table tournaments) to control variance and isolate ICM decision quality.
4. Push/fold accuracy serves as a quick litmus test — any competitive agent must match known Nash push/fold charts.
5. Use tournament equity ($EV) as the primary evaluation metric, not chip counts.
