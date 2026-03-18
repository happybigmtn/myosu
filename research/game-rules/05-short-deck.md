# Short Deck Hold'em (Six Plus / 6+)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Short Deck Hold'em (6-Plus Hold'em) |
| Variants | Triton rules (3-bet pot ante), PokerStars rules (button blind), various ante structures |
| Players | 2-9 (commonly 6-max) |
| Information | Imperfect (hidden hole cards) |
| Stochasticity | Stochastic (card deals from reduced deck) |
| Zero-Sum | Yes (HU); general-sum (multiway) |
| Solved Status | Unsolved; no published superhuman AI |

## Overview

Short Deck Hold'em (also called Six Plus Hold'em or 6+) is a variant of Texas Hold'em played with a 36-card deck created by removing all 2s, 3s, 4s, and 5s. The reduced deck dramatically changes hand probabilities, resulting in modified hand rankings (flushes beat full houses) and a much more action-heavy game. The game was popularized by the Triton Poker Series and is now offered at most major poker venues and online platforms.

**Players:** 2 to 9 (commonly 6-max)
**Deck:** 36 cards (6 through A in each of four suits)
**Betting structure:** No-limit (most common) or ante-only with button blind

## Setup

### Deck Composition
The deck contains 36 cards:
- Ranks: 6, 7, 8, 9, T, J, Q, K, A (nine ranks)
- Suits: Clubs, Diamonds, Hearts, Spades (four suits)
- Total: 9 x 4 = 36 cards

### Blind / Ante Structure
Short Deck uses one of two common structures:

**Structure A — Button Blind with Antes (Triton / High Stakes Standard):**
- Every player posts an ante each hand.
- The player on the button posts an additional "button blind" (typically 2x or 3x the ante).
- There is no small blind or big blind in this format.
- Preflop action begins with the player to the left of the button (UTG) and proceeds clockwise. The button acts last preflop.
- The button blind is live: the button player may check if no raise has been made (after calling the button blind), or raise.

**Structure B — Standard Blinds (PokerStars / Online Standard):**
- Standard small blind / big blind structure, identical to NLHE.
- Every player also posts an ante (typically equal to the BB).
- Preflop action begins with UTG; blinds act last.

This document describes both structures. The hand mechanics are identical; only the forced bets and preflop action order differ.

### Dealing
1. Antes (and blinds/button blind) are posted.
2. Dealer burns one card.
3. Each player receives two hole cards face-down (dealing order: left of button first, button last).

## Game Flow

### 1. Preflop

**Under Structure A (Button Blind):**
- All players post antes. The button posts the button blind.
- Action starts with the player to the left of the button.
- Players may fold, call the button blind, or raise.
- Action proceeds clockwise. The button acts last.
- If no one raises, the button may check (their blind is live).

**Under Structure B (Standard Blinds):**
- Identical to NLHE. Action starts with UTG, proceeds clockwise. BB acts last.

### 2. Flop
- Dealer burns one card, deals three community cards face-up.
- Action starts with the first active player to the left of the button.
- Players may check or bet. Standard betting rules apply.

### 3. Turn
- Dealer burns one card, deals one community card face-up.
- Action identical to flop.

### 4. River
- Dealer burns one card, deals the final community card face-up.
- Action identical to flop and turn.

### 5. Showdown
- If two or more players remain after the river betting round, hands are compared.
- Best five-card hand wins using Short Deck hand rankings (see below).
- Ties split the pot.

## Betting Rules

### No-Limit Structure (Standard)
- Identical to NLHE no-limit rules.
- Minimum bet equals the button blind (Structure A) or big blind (Structure B).
- Minimum raise increment equals the previous raise increment.
- Maximum bet is a player's entire stack.
- No cap on raises.

### All-In and Side Pot Rules
Identical to NLHE. See 02-nlhe-6max.md for details.

### Ante Mechanics (Structure A)
- Dead antes: all antes go into the pot before the hand. They are dead money (they do not count toward any player's bet).
- The button blind is live: it counts as a bet that must be called or raised.
- Preflop pot calculation: total antes + button blind = starting pot.
  - Example: 6 players, ante 1,000, button blind 3,000. Starting pot = 6,000 + 3,000 = 9,000.

## Hand Rankings

This is the critical section. Short Deck hand rankings differ from standard poker due to the altered probabilities of a 36-card deck.

### Modified Ranking (Most Common — PokerStars, Triton, GGPoker)

| Rank | Hand | Description | Example |
|------|------|-------------|---------|
| 1 | Royal Flush | A-K-Q-J-T of the same suit | A{s}K{s}Q{s}J{s}T{s} |
| 2 | Straight Flush | Five consecutive cards of the same suit | 9{h}8{h}7{h}6{h}A{h}* |
| 3 | Four of a Kind | Four cards of the same rank | 8{s}8{h}8{d}8{c}K{s} |
| 4 | **Flush** | Five cards of the same suit | A{d}J{d}9{d}7{d}6{d} |
| 5 | **Full House** | Three of a kind + a pair | Q{s}Q{h}Q{d}T{c}T{s} |
| 6 | Straight | Five consecutive cards, mixed suits | T{s}9{h}8{d}7{c}6{s} |
| 7 | Three of a Kind | Three cards of the same rank | J{s}J{h}J{d}A{c}K{s} |
| 8 | Two Pair | Two pairs + one kicker | A{s}A{h}K{d}K{c}Q{s} |
| 9 | One Pair | One pair + three kickers | K{s}K{h}A{d}Q{c}J{s} |
| 10 | High Card | No combination | A{s}K{h}Q{d}J{c}8{s} |

*A-6-7-8-9 is the lowest possible straight (see Aces and Straights below).

### Why Flush > Full House
With only 9 cards per suit (instead of 13), flushes are significantly harder to make:
- In a standard 52-card deck, C(13,5) = 1,287 flush combinations per suit.
- In a 36-card deck, C(9,5) = 126 flush combinations per suit.
- Full houses become relatively easier because the condensed rank set (9 ranks) makes pairing more likely.
- The probability of a flush in Short Deck is approximately 1 in 205 (five-card hands), while a full house is approximately 1 in 36.

### Three of a Kind vs. Straight (Variant Rule)
There are two schools of thought:
- **Standard (most common):** Straights rank above three of a kind (ranks 6 and 7 as shown above). This is used by PokerStars, GGPoker, and most online platforms.
- **Alternative:** Three of a kind ranks above straights. This is mathematically defensible (straights are more common than trips in a 36-card deck) but rarely used in practice.

This document uses the standard ranking (straights > trips) unless explicitly noted.

### Aces and Straights
- Aces play high (A-K-Q-J-T) and low.
- **The lowest straight is A-6-7-8-9** (the ace substitutes for the removed 5, creating a "wheel" equivalent).
- A-6-7-8-9 is a valid straight. A-6 alone is not a straight draw; the intermediate cards must be present.
- There is **no** A-2-3-4-5 straight (those cards don't exist in the deck).

### Complete Straight List (Low to High)
| Straight | Cards |
|----------|-------|
| Wheel (lowest) | A-6-7-8-9 |
| Six-high | 6-7-8-9-T |
| Seven-high | 7-8-9-T-J |
| Eight-high | 8-9-T-J-Q |
| Nine-high | 9-T-J-Q-K |
| Broadway (highest) | T-J-Q-K-A |

### Tie-Breaking Rules
Same principles as standard poker:
- **Straight Flush / Straight:** Highest top card wins. A-6-7-8-9 is the lowest straight.
- **Four of a Kind:** Higher quad wins, then kicker.
- **Flush:** Compare highest to lowest, first difference wins.
- **Full House:** Higher trips first, then pair.
- **Three of a Kind:** Higher trips, then kickers.
- **Two Pair:** Higher top pair, then second pair, then kicker.
- **One Pair:** Higher pair, then kickers.
- **High Card:** Compare highest to lowest.

### Best Five-Card Hand
Same as Hold'em: each player uses the best five cards from their two hole cards and five community cards. A player may use both, one, or neither hole card.

## Showdown Rules

Identical to NLHE. See 02-nlhe-6max.md.

## Special Rules

### No Low Cards
The absence of 2s, 3s, 4s, and 5s means:
- The lowest possible card is a 6.
- The lowest possible pair is a pair of 6s.
- The lowest possible kicker is a 6.

### Increased Hand Strength
With a compressed deck:
- Players hit strong hands more frequently.
- Pocket pairs flop sets more often (~17% vs ~12% in standard Hold'em) because there are fewer cards in the deck.
- Premium starting hands (AA, KK) are dealt more frequently (1 in 105 for AA vs 1 in 221 in standard Hold'em).
- Connected hands make straights much more often.

### Outs and Probabilities
With 36 cards instead of 52:
- Deck size after the deal (2 hole cards): 34 remaining.
- After the flop (2 hole + 3 board): 31 remaining.
- After the turn: 30 remaining.
- Flush draw outs: Typically 5 remaining suited cards (from 9 per suit minus the 4 you see). Compare to 9 remaining in standard Hold'em from 13 per suit.
- This means flush draws complete less often, confirming the rarity of flushes.

### Equity Convergence
Similar to PLO, hand equities run closer together in Short Deck. A pair of aces has roughly 63% equity against a random hand preflop (compared to ~85% in standard Hold'em). This leads to more multiway action and larger pots.

## Key Strategic Concepts

### Hand Strength Recalibration
Players transitioning from standard Hold'em must recalibrate hand strength:
- **Top pair** is weaker because opponents make two pair, trips, and straights more frequently.
- **Sets** are common. Flopping a set happens ~17% of the time with a pocket pair.
- **Straights** are very common. Any four-to-a-straight board is dangerous.
- **Flushes** are premium. A flush is very difficult to make and should be valued highly.
- **Overpairs** decline in value relative to standard Hold'em because they get outdrawn more often.

### Starting Hand Selection
- **Premium:** AA, KK, QQ, AKs, AQs. These are stronger than in standard Hold'em in terms of raw preflop equity but face more variance postflop.
- **Connected hands:** Suited connectors (T9s, 98s, 87s) are very strong because the compressed deck makes straights common. Connected hands often have 40-45% equity against premium pairs.
- **Suited hands:** Suited hands gain value because flushes are rare and powerful. Suited aces are premium.
- **Low pairs (66-88):** Weaker than in standard Hold'em. Set-over-set is more common, and low sets are vulnerable.

### Positional Play
Position functions the same as in NLHE. Late position is advantageous. In the button-blind ante format, the button is especially valuable because the dead money from antes increases the reward for stealing.

### Pot Odds Adjustment
With a 36-card deck, drawing odds are calculated differently:
- **Flush draw (5 outs on the flop):** ~16% per street (vs ~19% in standard Hold'em).
- **Open-ended straight draw (8 outs on the flop):** ~26% per street (vs ~17% in standard Hold'em with the same number of outs—but in Short Deck, straight draws typically have more outs due to deck compression).
- **Gutshot (4 outs on the flop):** ~13% per street (vs ~8.5% in standard Hold'em).

### Aggression and Action
The compressed deck and ante structure produce a fundamentally more aggressive game:
- Larger preflop pots (from antes) incentivize wider preflop play.
- Closer equities mean more flops are seen.
- Boards are more connected, creating more action-inducing textures.
- Conservative play is punished more heavily because the ante drain is significant.

### Variance
Short Deck is extremely high-variance due to:
- Closer equities preflop and postflop.
- More frequent coolers (set-over-set, flush-over-straight, etc.).
- Larger pots relative to effective stacks.
- Bankroll requirements are substantially higher than standard NLHE.

## Common Terminology

| Term | Definition |
|------|------------|
| **Short Deck** | The 36-card variant with 2-5 removed |
| **Six Plus / 6+** | Alternative names for Short Deck Hold'em |
| **Button blind** | The forced bet posted by the button player (Structure A) |
| **Ante** | Forced bet posted by every player each hand |
| **Wheel** | The lowest straight: A-6-7-8-9 |
| **Broadway** | The highest straight: T-J-Q-K-A |
| **Flush over boat** | Reminder that flushes beat full houses in Short Deck |
| **Wrap** | Though more commonly a PLO term, Short Deck's connected boards create many multiway straight draws |
| **Set-over-set** | Two players each having three of a kind with different ranks; more common in Short Deck |
| **Cooler** | An unavoidable clash of very strong hands |
| **Equity convergence** | The phenomenon of hand equities being closer together in Short Deck |
| **Dead money** | Antes in the pot that no player has a claim to; incentivizes aggression |
| **Nit** | A player who plays too tightly; punished heavily in Short Deck due to ante drain |
| **Rundown** | A coordinated board with many straight possibilities |
| **Nut flush draw** | Drawing to the best possible flush; extremely valuable in Short Deck |

## State Space Analysis

### Information Sets
- Starting hands: C(36,2) = 630 raw combinations. Strategically distinct: ~171 (accounting for suit isomorphism).
- Reduced deck means fewer possible boards: C(34,5) = 278,256 flops (vs C(50,5) for standard deck).
- Smaller information set space than standard NLHE, but still enormous.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Starting hand combinations | 630 (raw), ~171 (strategic) |
| Possible flops (HU, after dealing 4 cards) | C(32,3) = 4,960 |
| Game tree nodes (HU) | ~10^60+ (estimated, smaller than NLHE HU) |
| Reduction factor vs NLHE | ~50-100× smaller card state space |

### Action Space
- Same structure as NLHE: fold/check/call/bet/raise.
- No-limit allows any bet size; typically discretized.
- Ante structure (vs blind structure) changes pre-flop dynamics significantly.

## Key Challenges for AI/Solver Approaches

### 1. Altered Equity Distributions
With 36 cards:
- Hand equities cluster more tightly — pairs vs overcards are closer to 50-50.
- Flushes are significantly rarer, changing draw dynamics.
- Straights are more common (cards are closer together in rank).
- Pairs are more common (higher probability of matching ranks).
- These shifts mean NLHE card abstractions cannot be directly reused.

### 2. Ante Structure Dynamics
The universal ante structure (common in Short Deck) creates:
- Larger initial pots relative to stack sizes.
- More aggressive pre-flop play (more incentive to fight for antes).
- Different positional advantages compared to blind-based formats.

### 3. Card Removal Effects (Blockers)
With a 36-card deck, each known card removes a larger fraction of the remaining deck (1/36 vs 1/52). Blocker effects are more pronounced:
- Holding one suit reduces flush possibility more (1/9 vs 1/13 of each suit removed).
- Board texture reads are more precise with fewer possible hands.

### 4. Solver Transferability
Existing NLHE solvers can be adapted, but:
- Hand evaluation functions must be rewritten for the modified rankings.
- Card abstraction algorithms need retraining on the 36-card equity landscape.
- Pre-computed lookup tables (e.g., two-plus-two evaluator) must be regenerated.

### 5. Limited Training Data
Short Deck is a newer game with less available hand history data. This limits:
- Empirical validation of solver strategies.
- Opponent modeling from historical play.
- Academic study (few published papers).

## Known Solver Results

### Commercial Solvers
- **PioSOLVER**: supports Short Deck hand evaluation.
- **GTO Wizard**: offers Short Deck solutions.
- **SimplePostflop**: Short Deck mode available.
- Solutions are available for specific post-flop spots but no comprehensive strategy has been published.

### Academic Work
- No published academic papers specifically on solving Short Deck.
- The reduced game tree size (vs NLHE) suggests that HU Short Deck could be more thoroughly solved than NLHE HU with equivalent compute.
- CFR-based approaches are directly applicable with modified hand evaluator.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2017 | (Triton Poker Series inception) | Popularized Short Deck as a competitive format |
| 2019 | Brown & Sandholm, Pluribus | Techniques applicable to Short Deck multiway |
| 2020 | Various poker theory articles | Practical Short Deck strategy development |

## Relevance to Myosu

### Solver Applicability
Short Deck is a **"NLHE variant with reduced complexity"** — useful for testing solver scalability:
- **CFR/MCCFR**: directly applicable with modified hand evaluator. Smaller game tree means faster convergence.
- **Neural networks**: DeepStack/ReBeL approaches work with any hand evaluator — swap in Short Deck evaluation.
- **Abstraction**: less abstraction needed due to smaller card space, but the tighter equity distributions demand finer granularity.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 5/5 | Standard CFR, smaller game tree |
| Neural value network potential | 4/5 | Same as NLHE, less data needed |
| Abstraction necessity | 3/5 | Smaller space but tighter distributions |
| Real-time solving value | 4/5 | Valuable, same techniques as NLHE |
| Transferability of techniques | 4/5 | Direct transfer from NLHE with evaluator swap |

### Myosu Subnet Considerations
- **Validation simplicity**: smaller game tree means exploitability can be more tightly bounded than NLHE.
- **Hand evaluation oracle**: must implement the modified Short Deck hand rankings correctly. This is a non-trivial implementation detail.
- **Benchmarking utility**: Short Deck's smaller state space makes it useful as an intermediate benchmark between solved games and full NLHE.
- **Growing popularity**: Short Deck is played in high-stakes live games (Triton Series) and online, giving it commercial relevance.
- **Ante structure handling**: the solver framework must support ante-based formats, not just blind-based. This has implications for the pre-flop solution structure.

### Recommended Approach for Myosu
1. Adapt NLHE solver pipeline with Short Deck hand evaluator.
2. Use as an intermediate complexity benchmark — between solved limit HU and unsolved NLHE 6-max.
3. The smaller state space allows for tighter exploitability bounds, making it a better validation target.
4. Pre-compute more complete blueprints than feasible for NLHE, enabling quality comparison of real-time solving approaches.
5. Ante structure support is a requirement for the solver framework (reusable for other ante-based games).
