# Teen Patti (तीन पत्ती)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Teen Patti (तीन पत्ती, "Three Cards") |
| Variants | Classic, Muflis (lowball), AK47, Joker, Best of Four, 999 |
| Players | 3-7 (commonly 5-6) |
| Information | Imperfect (hidden cards, optional "blind" play) |
| Stochasticity | Stochastic (card deals) |
| Zero-Sum | Yes (pot-based, no rake in traditional) |
| Solved Status | Unsolved for general play; trivial GTO for simplified variants |

## Overview

Teen Patti ("Three Cards") is a South Asian gambling card game originating from the Indian subcontinent. It is derived from the British game Three Card Brag and shares ancestry with poker. The game is deeply embedded in Indian culture, commonly played during Diwali and other festivals. Players compete by betting on who holds the best three-card hand, with a unique blind/seen betting mechanic that creates asymmetric information dynamics.

**Players:** 3 to 6 (works with 2, optimal at 4-6)
**Deck:** Standard 52-card deck, no jokers (unless a variation requires them)
**Objective:** Win the pot by holding the best three-card hand at showdown, or by being the last player remaining after all others have folded

## Equipment

- One standard 52-card deck (no jokers in the base game)
- Chips or currency for betting
- A flat surface

## Setup

### Boot (Ante)
Before any cards are dealt, all players agree on:
1. **Boot amount:** A mandatory ante placed by every player into the pot before cards are dealt. This establishes the initial pot and sets the starting stake.
2. **Maximum pot limit (optional):** An agreed cap on the pot size. When the pot reaches this limit, all remaining players must show their cards.

### Dealing
1. A dealer is selected (by consensus or high card draw). The deal rotates clockwise after each hand.
2. The dealer shuffles and deals one card at a time, face-down, clockwise to each player, until every player has exactly three cards.
3. Players may choose to look at their cards (becoming "seen" / Chaal) or leave them face-down (playing "blind" / Andha). This decision can be made at any point during the hand.

### Starting Player
The player to the left of the dealer acts first. Play proceeds clockwise.

## Game Flow

### Turn Structure
On each turn, a player must either:
1. **Bet** (place chips into the pot to remain in the hand), or
2. **Fold** (surrender their cards and forfeit all chips already placed in the pot)

The hand continues clockwise until:
- All players but one have folded (the remaining player wins), or
- Only two players remain and one requests a **show**, or
- The pot limit is reached (if playing with one)

### Blind vs. Seen Status
The core mechanic of Teen Patti is the distinction between blind and seen players:

- **Blind (Andha):** The player has NOT looked at their cards. Blind players bet at half the rate of seen players.
- **Seen (Chaal):** The player HAS looked at their cards. A player may look at their cards at any time before their turn, but once seen, they cannot revert to blind status.

### Betting Amounts (Current Stake System)
The game tracks a "current stake" amount that governs minimum and maximum bets:

**If you are BLIND:**
- Minimum bet: 1x the current stake
- Maximum bet: 2x the current stake
- The current stake for the next player becomes whatever you bet

**If you are SEEN:**
- Minimum bet: 2x the current stake
- Maximum bet: 4x the current stake
- The current stake for the next player becomes half of whatever you bet

The current stake starts equal to the boot amount. This system ensures seen players always pay at least double what a blind player would pay.

**Example:** Boot is 10. Current stake = 10. A blind player bets 10 (1x). Next player is seen, so they must bet at least 2x10 = 20. They bet 20. Current stake becomes 20/2 = 10. Next blind player bets at least 10.

## Actions

### 1. Bet (Chaal / Andha)
Place chips into the pot according to the blind/seen rules above. This is the mandatory action to stay in the hand.

### 2. Fold (Pack)
Surrender your cards and exit the hand. All chips previously placed are lost.

### 3. Show (when exactly 2 players remain)
When only two players remain in the hand, either player may request a show. The rules depend on blind/seen status:

- **Both players seen:** Either player may request a show by paying the current seen bet amount (2x current stake). Both players reveal their cards, and the higher-ranked hand wins the pot.
- **One blind, one seen:** Only the seen player may request a show, by paying the current seen bet amount. The blind player cannot request a show.
- **Both players blind:** Either player may request a show by paying the current blind bet amount (1x current stake).
- If hands are exactly equal at a show, the player who requested (paid for) the show loses.

### 4. Sideshow (Compromise)
When there are three or more players remaining and you are a seen player, you may request a sideshow with the player who bet immediately before you. The sideshow rules:

1. You must first place a bet equal to the current seen bet amount (2x current stake).
2. You then request a "sideshow" (also called "compromise" or "backshow") with the previous player.
3. The previous player may **accept** or **refuse** the sideshow.
4. If **accepted:** Both players privately compare their cards. The player with the lower-ranked hand must fold. If hands are equal, the player who requested the sideshow folds.
5. If **refused:** Play continues normally. The requesting player has already placed their bet.
6. Only seen players may request or participate in sideshows.

## Hand Rankings

Hands are ranked from strongest to weakest. Within each rank, ties are broken by card values (A highest, 2 lowest).

| Rank | Name (Hindi) | English Equivalent | Description |
|------|-------------|-------------------|-------------|
| 1 | **Trail / Set** (तीन) | Three of a Kind | Three cards of the same rank. AAA is highest, 222 is lowest. |
| 2 | **Pure Sequence** (पक्की सीक्वेंस) | Straight Flush | Three consecutive cards of the same suit. A-K-Q is highest, 4-3-2 is lowest. A-2-3 is the second-highest pure sequence. |
| 3 | **Sequence** (सीक्वेंस) | Straight | Three consecutive cards, not all the same suit. A-K-Q highest, 4-3-2 lowest. A-2-3 is the second-highest sequence. |
| 4 | **Color** (रंग) | Flush | Three cards of the same suit, not in sequence. Compared by highest card, then second, then third. |
| 5 | **Pair** (जोड़ी) | Pair | Two cards of the same rank plus one different card. Higher pair wins. If pairs are equal, the kicker (third card) breaks the tie. A-A-K is highest, 2-2-3 is lowest. |
| 6 | **High Card** (हाई कार्ड) | High Card | None of the above. Compared by highest card, then second, then third. A-K-J of mixed suits is the best high card. |

### Ranking Details
- **Aces are always high** in basic Teen Patti. A-2-3 is a valid sequence (second-highest), but the ace plays high.
- **Suit ranking:** Suits have no inherent ranking. If two hands are identical in rank and card values across all three cards, they are considered equal.
- The highest possible hand is A-A-A (trail of aces).
- The lowest possible hand is 5-3-2 (not of the same suit).

## Scoring

Teen Patti uses a simple pot-based scoring system:
- The winner of each hand takes the entire pot.
- There is no complex scoring or point system across hands.
- Sessions are typically cash games where players buy in with chips/money and leave when they choose.
- Some groups play to a fixed number of rounds or until a target profit/loss is reached.

## Winning Conditions

A hand is won when:
1. **Last player standing:** All other players fold, and the remaining player wins the pot without showing cards.
2. **Showdown (Show):** When two players remain and a show is requested, the player with the higher-ranked hand wins.
3. **Pot limit reached:** If an agreed pot limit is reached, all remaining players must show, and the highest hand wins.

## Special Rules and Variations

### Major Variations

#### Muflis (Lowball)
All standard rules apply, but the **lowest** hand wins instead of the highest. Hand rankings are inverted: high card beats pair, pair beats flush, etc. 5-3-2 offsuit becomes the best hand, and A-A-A (trail) becomes the worst.

#### AK47
Cards with rank A, K, 4, and 7 of all suits become **wild cards** (jokers). A wild card can substitute for any card to complete a better hand. All other rules remain the same.

#### Best of Four
Each player receives **four** cards instead of three and must select the best three-card hand from them. The discarded fourth card is not revealed.

#### Lowest Joker (Lallan Kallan)
In each player's hand, the lowest-ranked card becomes a wild card (joker) for that player only. If a player has two cards of the lowest rank, both are wild.

#### Wild Draw / Community
- **Two-Card Community:** Each player receives two private cards. One or more community cards are dealt face-up in the center. Players make their best three-card hand using their private cards plus one community card.
- **Three Community Cards:** Each player receives two private cards. Three community cards are dealt. Players choose any one community card to combine with their two private cards.

#### Stud / Faceup
One or more of each player's three cards are dealt face-up (visible to all). Common variants include one card up and two down, or two up and one down.

#### 999 / Closest to 9
Each player's hand is valued based on how close the total value is to 9 or 999 (using digit sums). Face cards count as 0, aces as 1, and numbered cards at face value. Only the units digit matters (e.g., 15 = 5).

#### Bust Card Draw
A single card is drawn from the deck and revealed. Any player holding a card of that rank must fold immediately. Remaining players continue normally.

#### Kissing / Missing
Missing: A card is drawn and revealed. Any player whose hand does NOT contain a card of that rank receives a joker bonus (e.g., their hand is upgraded). Kissing: players with adjacent ranks to the revealed card receive a bonus.

### Pot-Limit and Fixed-Limit Variants
- **Pot Limit:** Maximum bet at any point equals the current pot size.
- **Fixed Limit:** Bets are set at fixed increments (e.g., always 1x boot, 2x boot).
- **Spread Limit:** Bets must fall within a defined range (e.g., between boot and 4x boot).

## Key Strategic Concepts

### Blind Play Advantage
Playing blind costs half as much per round as playing seen. A player who stays blind for several rounds extracts information (observing who folds, who raises) at a reduced cost. However, they cannot request sideshows and can only be shown by seen players, not by choice.

### Table Image and Betting Patterns
Since hand information is hidden and there are no community cards, reads depend almost entirely on betting patterns and player behavior. Frequent raises from a normally conservative player signal strength. Consistent minimum bets may signal a marginal hand.

### Sideshow as Information Tool
Requesting a sideshow forces a private comparison with one opponent. Even the act of accepting or refusing a sideshow conveys information. A refusal may indicate a strong hand (player is confident and doesn't want to eliminate an opponent cheaply) or a bluff (they don't want to be compared).

### Fold Timing
Because the pot grows rapidly (each player adds at least the current stake each round), the cost of staying in escalates quickly. Knowing when to fold a mediocre hand is the primary skill differentiator. The blind/seen asymmetry means that a seen player with a weak hand hemorrhages chips faster.

### Position
Acting later provides the advantage of observing other players' decisions before committing. The player to the right of the dealer (last to act) has the most information each round.

### Bluffing
With no community cards and no way to verify a hand until showdown, bluffing is central to the game. A player who consistently plays blind and bets confidently can force multiple opponents to fold without ever having a strong hand.

## Common Terminology

| Term (Hindi/Urdu) | English | Definition |
|-------------------|---------|------------|
| **Teen Patti** (तीन पत्ती) | Three Cards | The name of the game |
| **Boot** (बूट) | Ante | Mandatory bet placed by all players before dealing |
| **Andha** (अंधा) | Blind | A player who has not looked at their cards |
| **Chaal** (चाल) | Seen / Call | A player who has looked at their cards; also refers to placing a bet |
| **Pack** (पैक) | Fold | Surrendering your hand |
| **Show** (शो) | Showdown | Revealing hands when two players remain |
| **Sideshow** (साइडशो) | Compromise | Private comparison between two seen players |
| **Trail** (ट्रेल) | Three of a Kind | Three cards of the same rank |
| **Pure Sequence** | Straight Flush | Three sequential same-suit cards |
| **Sequence** | Straight | Three sequential cards of mixed suits |
| **Color** (कलर) | Flush | Three same-suit cards not in sequence |
| **Pair** (जोड़ी) | Pair | Two cards of the same rank |
| **High Card** | High Card | No combination; highest individual card plays |
| **Muflis** (मुफलिस) | Lowball | Variation where lowest hand wins |
| **Pot** (पॉट) | Pot | The total chips wagered by all players in a hand |
| **Stake** (दाव) | Stake | The current bet level governing minimum/maximum bets |
| **Hukam** (हुक्म) | Trump / Joker | Wild card in relevant variations |
| **Seen** | Seen | Same as Chaal; a player who has viewed their cards |

## State Space Analysis

### Information Sets
- Starting hands: C(52,3) = 22,100 possible three-card hands.
- Strategically distinct (accounting for suit isomorphism): ~455.
- With blind/seen status: each player's information state includes their play mode and the visible betting history.
- Information sets are relatively small compared to poker variants with community cards and multiple streets.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Starting hand combinations | 22,100 (raw), ~455 (strategic) |
| Betting rounds | Variable (no fixed streets; continuous betting) |
| Game tree nodes (6-player) | ~10^15-10^20 (estimated, depends on pot/round limits) |
| Information sets per player | ~10^8-10^12 (with betting histories) |

### Action Space
- Limited actions per decision: bet (1×/2× stake), fold, request show/sideshow.
- Small branching factor per node compared to NLHE.
- The blind/seen choice adds a one-time binary decision per player.

## Key Challenges for AI/Solver Approaches

### 1. Blind vs. Seen Decision
The choice of when to look at one's cards is a critical strategic decision:
- Playing blind reduces the cost of staying in the hand.
- But seen play allows informed decision-making.
- The optimal timing of "going seen" is a core strategic question.

### 2. Variable-Length Betting Rounds
Unlike poker with fixed streets, Teen Patti betting continues indefinitely until all but one player fold or a show occurs. This creates:
- Variable-depth game trees.
- Difficulty in defining a fixed-depth search horizon.
- Potential for very deep game trees in aggressive games.

### 3. Show/Sideshow Mechanics
The show and sideshow mechanics create unique strategic considerations:
- Requesting a show reveals information (you're seen, you believe you might be ahead).
- Sideshow requests can be declined, adding another strategic layer.
- The cost structure of shows affects calling ranges.

### 4. Multiplayer Pot Dynamics
With 5-7 players, pot contributions are asymmetric (blind vs seen bets differ). The multiplayer aspect creates coalition dynamics similar to other multi-player poker games.

### 5. Variant Explosion
The many popular variants (Muflis, AK47, Joker, etc.) each require separate solution strategies:
- **Muflis**: lowest hand wins (complete inversion of hand rankings).
- **AK47**: all aces, kings, fours, and sevens are wild.
- **Best of Four**: dealt 4 cards, choose best 3.

## Known Solver Results

### Academic Work
- Very limited academic research compared to Western poker variants.
- No published Nash equilibrium computation for full Teen Patti.
- Some game-theoretic analyses of simplified variants exist in Indian computer science literature.
- The game's structure (fixed-size hands, no community cards) makes it more amenable to exact solution than Hold'em for small player counts.

### Practical Observations
- GTO strategy for 2-player Teen Patti is tractable and can be computed with standard CFR.
- The blind/seen mechanic makes the game richer than a pure 3-card showdown game.
- Commercial Teen Patti apps use basic hand-strength heuristics rather than GTO strategies.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2013 | Jain & Gupta, "Game theoretic analysis of Teen Patti" | Basic equilibrium analysis |
| 2018 | Various Indian CS conference papers | Heuristic-based Teen Patti AI |
| 2020 | Online poker AI research | General imperfect-info game techniques applicable |

## Relevance to Myosu

### Solver Applicability
Teen Patti is interesting for myosu because it represents an **underserved market with simpler game mechanics**:
- **CFR**: directly applicable. The smaller hand space (455 strategic hands) and simpler betting structure make CFR tractable without heavy abstraction for 2-3 players.
- **Neural approaches**: overkill for the game's complexity but useful for multiplayer variants.
- **Blind/seen modeling**: a unique feature not found in Western poker, requiring specialized game-tree representation.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 5/5 | Small hand space, two-player variant tractable |
| Neural value network potential | 3/5 | Game is simple enough that tabular methods may suffice |
| Abstraction necessity | 2/5 | Less needed due to smaller state space |
| Real-time solving value | 3/5 | Useful for multiplayer, less critical for 2-player |
| Transferability of techniques | 3/5 | Blind/seen mechanic is unique; general techniques transfer |

### Myosu Subnet Considerations
- **Market opportunity**: Teen Patti is enormously popular in South Asia, with hundreds of millions of players. Solver technology for this game is virtually nonexistent.
- **Lower compute requirements**: the simpler game structure means solver nodes require less hardware, lowering the barrier to participation.
- **Hand evaluation oracle**: straightforward 3-card hand comparison. Much simpler than Hold'em evaluation.
- **Variant support**: the subnet should support multiple Teen Patti variants to maximize relevance. Each variant requires a separate hand ranker but shares the same game tree structure.
- **Cultural significance**: including Teen Patti positions myosu as globally inclusive, not just Western-poker-focused.

### Recommended Approach for Myosu
1. Implement CFR for 2-player Teen Patti as a quick win — solvable with modest compute.
2. Extend to 5-6 player with MCCFR and opponent modeling.
3. Model the blind/seen decision as an explicit game tree node.
4. Support major variants (Muflis, AK47, Best of Four) with pluggable hand evaluators.
5. Use as a low-complexity entry point for solver node operators on the subnet.
