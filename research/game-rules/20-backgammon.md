# Backgammon

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Backgammon |
| Variants | Standard (with doubling cube), Money game, Match play, Hypergammon, Nackgammon |
| Players | 2 |
| Information | Perfect (all pieces visible; imperfect prediction of future dice rolls) |
| Stochasticity | Stochastic (dice rolls) |
| Zero-Sum | Yes |
| Solved Status | Unsolved; superhuman AI exists (TD-Gammon, 1992; GNU Backgammon, Palamedes) |

## Overview

Backgammon is one of the oldest known board games, with origins dating back approximately 5,000 years to Mesopotamia. It is a two-player race game combining strategic positioning with dice-driven movement. The doubling cube, introduced in the 1920s in New York, adds a game-theory layer of stake management. Backgammon has a rich competitive tournament scene governed by standardized match-play rules.

- **Players:** 2
- **Duration:** 5--30 minutes per game (longer for matches)
- **Objective:** Move all 15 of your checkers into your home board and then bear them off (remove from the board). The first player to bear off all 15 checkers wins.

## Equipment

### Board

The board consists of **24 narrow triangles** called **points**, alternating in two colors. The points are grouped into four quadrants of six:

```
 13  14  15  16  17  18 | BAR | 19  20  21  22  23  24
                        |     |
 12  11  10   9   8   7 | BAR |  6   5   4   3   2   1
```

- **Points 1--6:** Player's home board (inner board)
- **Points 7--12:** Player's outer board
- **Points 13--18:** Opponent's outer board
- **Points 19--24:** Opponent's home board
- **The Bar:** A raised ridge dividing the board in half. Hit checkers are placed here.

Point numbering is relative: your 1-point is your opponent's 24-point, your 2-point is their 23-point, etc.

### Checkers

Each player has **15 checkers** (also called men, stones, or pieces) in a distinct color.

### Dice

- **2 standard six-sided dice** (shared or one pair per player)
- **1 doubling cube** -- a six-faced die marked with 2, 4, 8, 16, 32, 64 (not rolled; used as a stake marker)

## Setup

### Initial Checker Placement

Each player places their 15 checkers in the following starting position:

| Point | Your Checkers | Opponent's Checkers |
|-------|---------------|---------------------|
| 24    | 2             | -                   |
| 13    | 5             | -                   |
| 8     | 3             | -                   |
| 6     | 5             | -                   |
| 19    | -             | 2                   |
| 12    | -             | 5                   |
| 17    | -             | 3                   |
| 1 (opponent's view: 24) | - | 5 |

Both players move in **opposite directions**: you move from point 24 toward point 1 (your home board); your opponent moves from point 1 (their perspective) toward their home board.

The doubling cube starts at the center of the bar, on the 64 face (representing a stake of 1), not owned by either player.

## Game Flow

### Determining First Player

Each player rolls **one die**. The player with the higher number goes first and uses both dice for their opening move. If both players roll the same number, they re-roll until the numbers differ.

### Turn Structure

On each turn, a player:

1. **Rolls two dice.**
2. **Moves checkers** according to the dice values.
3. (Optionally, before rolling) **Offers a double** using the doubling cube.

### Movement Rules

The numbers on the two dice represent **two separate moves**. A player may:

- Move **one checker** by the value of one die and **a different checker** by the value of the other die.
- Move **one checker** by the total of both dice, provided the intermediate point (after the first die's movement) is also a legal landing point.

#### Legal Landing Points

A checker may move to a point that is:

1. **Open** -- not occupied by 2 or more opposing checkers.
2. Occupied by **one or more of the player's own checkers** (stacking).
3. Occupied by **exactly one opposing checker** (a blot -- see Hitting).

A point occupied by 2+ opposing checkers is **blocked** and may not be landed on or passed through (when using a single checker to move the combined total of both dice).

#### Doubles

When both dice show the same number, the player plays **four moves** of that number (not two). For example, double 3s = four moves of 3 each.

#### Mandatory Movement

- A player **must use both dice** if legally possible.
- If only one die can be used, the player **must play the higher number** (if both are individually playable but not both).
- If neither die can be played, the turn is forfeited.
- With doubles, the player must play as many of the four moves as legally possible.

### Hitting

A single checker on a point is called a **blot**. If an opposing checker lands on a blot:

1. The blot is **hit** and placed on the **bar**.
2. The moving checker occupies that point.

### Entering from the Bar

A player with one or more checkers on the bar **must enter all of them** before making any other move. Entry works as follows:

1. Roll the dice.
2. A checker enters on the point in the **opponent's home board** corresponding to the die value. For example, rolling a 3 enters the checker on the opponent's 3-point (your 22-point).
3. The destination point must be open (not blocked by 2+ opposing checkers).
4. If both dice can enter checkers, both must be played.
5. If neither die allows entry (both corresponding points are blocked), the player **forfeits their entire turn**.
6. If one die allows entry and the other does not, the player enters one checker and forfeits the unusable die (if the remaining die cannot be used for a regular move after entering).

### Bearing Off

Once **all 15** of a player's checkers are in their home board (points 1--6), that player may begin **bearing off**:

1. Roll the dice.
2. A die value **matching** a point number allows removing a checker from that point. For example, rolling a 4 bears off a checker from the 4-point.
3. A die value **higher** than the highest occupied point may be used to bear off a checker from the **highest occupied point**. For example, if the highest checker is on the 3-point and a 5 is rolled, a checker may be borne off from the 3-point.
4. A player is **not required** to bear off if a legal move within the home board is available. They may choose to move a checker forward instead.
5. If a checker is **hit** during bearing off, that player must re-enter the hit checker from the bar and move it all the way back into the home board before resuming bearing off.

### Doubling Cube

The doubling cube tracks the current stake of the game (starting at 1).

#### Offering a Double

- A player may propose doubling the stakes **at the start of their turn, before rolling**.
- Only the player who **does not own the cube** may propose a double. At the start of the game, either player may double (the cube is neutral). After a double is accepted, only the player who accepted (the cube owner) may propose the next redouble.

#### Responding to a Double

The opponent may:

1. **Accept (Take):** The stakes double, and the opponent takes possession of the cube.
2. **Decline (Drop/Pass):** The opponent concedes the game immediately, losing the current stake value (before the proposed double).

#### Redoubling

After accepting a double, the cube owner may later propose a **redouble**, doubling the stakes again. The original doubler must then accept or decline. There is no limit to the number of redoubles.

## Scoring / Winning

### Single Game

If the loser has borne off at least one checker:

**Points won = doubling cube value x 1**

### Gammon (Double Game)

If the loser has **not borne off any checkers**:

**Points won = doubling cube value x 2**

### Backgammon (Triple Game)

If the loser has not borne off any checkers **AND** has at least one checker on the bar or in the winner's home board:

**Points won = doubling cube value x 3**

### Match Play

Tournament backgammon is played as a **match to N points** (e.g., first to 7 points). Players alternate as first roller. The match score determines doubling strategy.

#### Crawford Rule

When a player first reaches a score of **(match length - 1)** -- exactly one point away from winning the match:

- The **immediately following game** is the **Crawford game**.
- During the Crawford game, **neither player may use the doubling cube**.
- After the Crawford game, the doubling cube is available again for all subsequent games.

The Crawford rule prevents the trailing player from immediately doubling to create a "free" gamble when they have nothing to lose.

#### Post-Crawford Strategy

After the Crawford game, if the match continues:

- The trailing player should double at the earliest opportunity (they have nothing to lose from increased stakes).
- The leading player may use the **free drop**: declining a double costs only 1 point, which is irrelevant when ahead. This gives the leader an option to re-roll unfavorable opening positions.

## Special Rules

### Optional Rules (Common in Social Play)

#### Automatic Doubles

If both players roll the same number to determine the first player, the stakes automatically double (cube moved to 2). Some groups limit automatic doubles to one per game.

#### Beavers

When a player is doubled, they may immediately **beaver** (redouble) while retaining ownership of the cube. The original doubler must then play at the higher stake. The original doubler may then **raccoon** (redouble again), and so on.

#### Jacoby Rule

Gammons and backgammons **count only as a single game** unless the doubling cube has been used at least once during the game. This prevents long, uncompetitive games where the winning player waits for a gammon rather than offering a double.

### Cocked Dice

If a die lands tilted against a checker, the edge of the board, or any other obstruction (not flat on the playing surface), both dice must be re-rolled.

### Illegal Moves

If a player makes an illegal move and the opponent notices before the next roll, the illegal move must be corrected. If the opponent rolls before noticing, the illegal move stands.

## Key Strategic Concepts

- **Race vs. prime vs. blitz:** Three main strategic archetypes. A **racing** game focuses on advancing quickly. A **priming** game builds consecutive blocked points (a "prime") to trap opposing checkers. A **blitz** attacks aggressively to send opponents to the bar.
- **The 5-point and the bar-point (7-point):** Controlling your own 5-point (the "golden point") and the 7-point (bar point) are among the highest strategic priorities in the opening. Together they form the start of a prime.
- **Pip count:** The total number of pips (points) each player must move to bear off all checkers. Lower pip count = winning the race. Pip counting informs doubling decisions.
- **Doubling cube theory:** Offering a double when your winning probability exceeds ~75% is generally correct in money play. Accepting a double when your winning probability exceeds ~25% is correct (since you lose 1 point by dropping vs. an expected loss of less than 1 point by taking).
- **Anchor:** A point held in the opponent's home board. The deeper the anchor (closer to the opponent's 1-point), the safer but less aggressive it is.
- **Timing:** Having enough moves to maintain structure while waiting for favorable rolls. Running out of timing (forced to break advantageous positions) is a common endgame problem.
- **Back game:** A defensive strategy where a player with two or more anchors deep in the opponent's home board waits for a hit opportunity during the opponent's bearing off.
- **Gammon threat:** In match play, the potential for a gammon (2 points) significantly affects doubling cube decisions, especially near the end of a match.
- **Crawford game awareness:** In the Crawford game, the trailing player cannot double and must play for a gammon to gain extra points. Post-Crawford, the trailing player should always double immediately.

## Common Terminology

| Term | Definition |
|------|-----------|
| **Point** | One of the 24 triangular spaces on the board |
| **Checker / Man / Stone** | A playing piece (15 per player) |
| **Blot** | A single checker on a point, vulnerable to being hit |
| **Hit** | Landing on an opponent's blot, sending it to the bar |
| **Bar** | The center divider where hit checkers are placed |
| **Enter** | Moving a checker from the bar into the opponent's home board |
| **Home board / Inner board** | Points 1--6, where checkers must be to bear off |
| **Outer board** | Points 7--12 |
| **Bear off** | Remove a checker from the board (the final phase) |
| **Prime** | A wall of consecutive blocked points (6 in a row is a full prime) |
| **Anchor** | A point held in the opponent's home board |
| **Double** | Offering to multiply the game stakes by 2 |
| **Take** | Accepting a double |
| **Drop / Pass** | Declining a double, conceding the game |
| **Redouble** | The cube owner offering to double the stakes again |
| **Beaver** | Immediately redoubling after being doubled, retaining cube ownership |
| **Gammon** | Winning when opponent has borne off zero checkers (2x stakes) |
| **Backgammon** | Gammon with opponent having a checker on the bar or in winner's home board (3x stakes) |
| **Crawford game** | The game after a player reaches match point; no doubling allowed |
| **Post-Crawford** | All games after the Crawford game |
| **Pip count** | Total remaining distance for all of a player's checkers |
| **Cube ownership** | The player who accepted the last double and can offer the next redouble |
| **Cocked die** | A die not resting flat; requires re-roll of both dice |
| **Slot** | Deliberately placing a blot on a key point, hoping to cover it next turn |
| **Cover** | Moving a second checker to a point with a blot, making it safe |
| **Split** | Moving one of two back checkers to separate points |
| **Builder** | A checker positioned to cover a key point on the next roll |
| **Running game** | A strategy focused on advancing checkers as quickly as possible |

## State Space Analysis

### Information Sets
- Backgammon is a **stochastic perfect-information game**: both players see the entire board, but future dice rolls are unknown.
- No hidden information (unlike card games). The only uncertainty is the dice.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Board positions | ~10^20 possible checker configurations |
| Average moves per turn | ~20 legal moves (varies; doubles create more) |
| Average game length | ~50-80 turns (each player) |
| Game tree nodes | ~10^140 (estimated by Berliner) |
| Expected positions per game | ~10^3 (after dice rolls and moves) |
| Branching factor (after dice) | ~20 legal moves; ~400 counting dice randomness |

### Action Space
- **Checker movement**: given a dice roll, enumerate all legal move combinations.
  - Non-double rolls: 2 dice values, each used for one move. Often 10-30 legal move combinations.
  - Double rolls: 4 moves of the same value. Can create many legal combinations.
- **Doubling cube**: binary decision at start of turn (offer double or not).
  - If doubled: opponent's binary decision (take or drop).
- Total actions per decision point: ~20-30 (checker moves) + doubling decision.

## Key Challenges for AI/Solver Approaches

### 1. Doubling Cube Strategy
The doubling cube is widely considered the most difficult aspect of expert backgammon:
- **When to double**: requires accurate assessment of winning probability and gammon/backgammon chances.
- **When to take/drop**: requires the same assessment from the opponent's perspective.
- **Market window**: the range of equity where doubling is correct (typically when equity is ~66-76%, depending on gammon risk and cube ownership).
- **Recube vig**: the value of owning the cube (ability to re-double) affects take/drop decisions.
- Match play adds another dimension (match equity tables, Crawford rule).

### 2. Positional Evaluation
Strong play requires nuanced position evaluation:
- **Racing positions**: when both sides are bearing off without contact — primarily about pip count.
- **Holding positions**: maintaining an anchor in opponent's board.
- **Priming positions**: building consecutive blocked points to trap opponent checkers.
- **Blitzing positions**: aggressive attacking to hit and contain opponent checkers.
- **Back game positions**: deliberate defensive strategy, holding multiple anchors in opponent's home board.
- Each position type requires different strategic treatment.

### 3. Dice Variance
The stochastic element creates high variance:
- A single unlucky roll can reverse a winning position.
- Long-term correct play is required — short-term results are noisy.
- Evaluation must consider the full distribution of dice outcomes, not just expected values.

### 4. Gammon/Backgammon Assessment
Accurately estimating gammon probability is critical for:
- Cube decisions (gammons double the cube value impact).
- Game play (when to play for gammon vs secure the win).
- Match play (gammon equity depends on match score).

### 5. Match Play Equity
In match play, the value of a point depends on the match score:
- Match equity tables (e.g., Kazaross XG2) provide the probability of winning the match from each score.
- This affects doubling cube strategy (at what match scores to double, take, or drop).
- The Crawford rule creates a special game with altered dynamics.

## Known Solver Results

### TD-Gammon (Tesauro, 1992)
The pioneering neural network backgammon AI:
- Trained via temporal difference learning (TD(λ)) through self-play.
- Achieved expert-level play.
- Architecture: simple neural network with hand-crafted features + raw board inputs.
- Published in *Communications of the ACM* and *Machine Learning*.
- Key innovation: one of the first demonstrations of neural network self-play achieving expert performance in a complex game.
- Estimated playing strength: approximately top 3-5 human level.

### GNU Backgammon (GnuBG)
- Open-source backgammon program.
- Neural network-based position evaluation (trained by Gerald Tesauro and later contributors).
- Considered world-class strength, approaching or matching the best human players.
- Includes rollout capabilities for precise position evaluation.
- Widely used for analysis and training by human experts.

### eXtreme Gammon (XG)
- Commercial backgammon software.
- Neural network evaluation with extensive rollout capabilities.
- Considered the gold standard for backgammon analysis.
- Plays at or above the best human level.

### Palamedes
- Won the first Computer Backgammon Olympiad.
- Uses neural network evaluation similar to TD-Gammon's approach.

### Modern Developments
- **AlphaZero-style approaches**: have been applied to backgammon but with limited published results (backgammon was included in early AlphaZero papers).
- **Neural network + MCTS**: combining learned evaluation with tree search.
- Current best programs (XG, GnuBG) are considered superhuman in checker play but doubling cube decisions remain imperfect.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 1992 | Tesauro, "Practical Issues in Temporal Difference Learning" | TD-Gammon (*Machine Learning*) |
| 1994 | Tesauro, "TD-Gammon, A Self-Teaching Backgammon Program, Achieves Master-Level Play" | Refinement and analysis |
| 1995 | Tesauro, "Temporal Difference Learning and TD-Gammon" | *Communications of the ACM* |
| 2000 | Tesauro, "On-line Resource Allocation Using Decompositional RL" | Further TD application |
| 2016 | Silver et al., "Mastering the game of Go with deep neural networks and tree search" | AlphaGo, with backgammon as comparison point |

## Relevance to Myosu

### Solver Applicability
Backgammon is the **canonical stochastic perfect-information game**:
- **CFR**: not the standard approach (perfect information eliminates the need for information-set reasoning). Minimax/expectimax with evaluation is standard.
- **Neural network value estimation**: the proven approach since TD-Gammon (1992). The first major success of neural self-play.
- **Temporal difference learning**: TD(λ) is the foundational algorithm for backgammon AI.
- **Rollout-based evaluation**: Monte Carlo simulation of positions for accurate equity estimation.
- **Doubling cube as separate decision**: cube decisions can be treated as a wrapper around checker-play evaluation (given equity, decide whether to double).

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 1/5 | Perfect information — CFR not the right tool |
| Neural value network potential | 5/5 | Proven since 1992; the original application |
| Abstraction necessity | 1/5 | Board state is compact enough for direct representation |
| Real-time solving value | 4/5 | Rollouts improve evaluation; MCTS applicable |
| Transferability of techniques | 3/5 | TD learning transfers; board game structure is unique in this list |

### Myosu Subnet Considerations
- **Historical significance**: backgammon was the first game where neural network self-play achieved expert performance. Including it honors this legacy.
- **Doubling cube as unique mechanic**: the only game in the list with an explicit doubling/stakes mechanism. Tests risk management in a pure form.
- **Perfect information**: the only (mostly) perfect-information game in the list besides Stratego's post-reveal phases. Provides architectural diversity.
- **Evaluation precision**: GNU Backgammon's rollout-based equity estimates provide ground truth for solver quality assessment.
- **Match play evaluation**: match equity tables provide an objective framework for evaluating cube decisions at different match scores.
- **Low state space**: the board position space (~10^20) is tractable for neural networks without abstraction.
- **Game oracle**: board legality, dice roll validation, and scoring are straightforward.

### Recommended Approach for Myosu
1. Neural network position evaluation via TD learning or policy gradient self-play.
2. Rollout-based equity estimation for high-precision evaluation.
3. Separate cube decision model (input: equity estimate + match score; output: double/take/drop).
4. Evaluate via equity loss per decision (comparison to XG/GnuBG rollout evaluations).
5. Backgammon serves as the "stochastic perfect-information" anchor in the game portfolio — architecturally distinct from the card games.
