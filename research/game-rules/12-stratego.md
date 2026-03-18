# Stratego (Hidden Army Board Game)

## Game Identity

| Property | Value |
|----------|-------|
| Full Name | Stratego |
| Variants | Classic (10×10), Barrage (8×8 quick version), Ultimate Lightning |
| Players | 2 |
| Information | Imperfect (hidden piece identities until combat) |
| Stochasticity | Deterministic (no randomness after setup; setup itself is a strategic choice) |
| Zero-Sum | Yes |
| Solved Status | Unsolved; superhuman AI demonstrated (DeepNash, 2022) |

## Rules Summary

### Setup
- 10×10 board with two 2×1 impassable water obstacles in the center.
- Each player has 40 pieces with hidden identities (opponent cannot see piece ranks).
- Players independently arrange their 40 pieces on their half of the board (4 back rows).

### Piece Types and Ranks
| Piece | Count | Rank | Special |
|-------|-------|------|---------|
| Marshal | 1 | 10 | Highest rank |
| General | 1 | 9 | |
| Colonel | 2 | 8 | |
| Major | 3 | 7 | |
| Captain | 4 | 6 | |
| Lieutenant | 4 | 5 | |
| Sergeant | 4 | 4 | |
| Miner | 5 | 3 | Can defuse Bombs |
| Scout | 8 | 2 | Can move multiple squares in a straight line |
| Spy | 1 | 1 | Defeats Marshal when attacking |
| Bomb | 6 | - | Immobile; destroys any attacker except Miner |
| Flag | 1 | - | Immobile; capture = win |

### Movement
- One piece moves one square orthogonally per turn (except Scouts and immobile pieces).
- Scouts may move any number of squares in a straight line (like a rook in chess), but cannot jump.
- Bombs and Flag cannot move.
- No piece may enter water squares.
- A piece cannot move back and forth between the same two squares more than a set number of times (two-square rule).

### Combat
- When a piece moves into a square occupied by an opponent's piece, combat occurs.
- Both pieces are revealed.
- Higher rank wins; equal ranks destroy each other.
- Special cases:
  - Spy defeats Marshal only when Spy attacks (not when defending).
  - Miner defeats Bomb (defuses it).
  - All other pieces attacking a Bomb are destroyed (Bomb also destroyed in some rules).
- Losing piece is removed from the board. Winning piece occupies the square.

### Victory Conditions
- Capture the opponent's Flag.
- Opponent cannot make a legal move (all mobile pieces captured or blocked).

## State Space Analysis

### Information Sets
- **Setup phase**: C(40,40) orderings × strategic placement = ~40! / product of piece count factorials ≈ 1.01 × 10^17 possible setups per player.
- **Game phase**: each player knows their own piece positions/identities + opponent piece positions (but not identities) + combat results (which reveal identities).
- Information about opponent pieces is gradually revealed through combat and movement patterns.

### Game Tree Size
| Metric | Estimate |
|--------|----------|
| Possible setups per player | ~1.01 × 10^17 |
| Combined setup configurations | ~10^34 |
| Game tree nodes (play phase) | ~10^535 (estimated by Perolat et al., 2022) |
| Information sets | ~10^175 (estimated) |
| Average game length | ~200-400 moves |
| Branching factor | ~20-30 legal moves per turn |

### Action Space
- **Setup phase**: arrange 40 pieces on 40 squares (one-time decision per game).
- **Play phase**: choose one of ~20-30 legal moves per turn (one piece, one direction, one distance).
- For Scouts: additional options for longer moves.

## Key Challenges for AI/Solver Approaches

### 1. Massive Hidden Information (Initial Setup)
The ~10^17 possible setups per player mean that at the start of the game, almost nothing is known about the opponent's army composition and positioning. This is fundamentally different from card games where the deck is known:
- In poker, you know the deck composition — you just don't know which cards your opponent holds.
- In Stratego, the opponent's entire army structure is hidden, and information is revealed only through combat.

### 2. Long Game Horizon
Games last 200-400 moves, creating deep game trees that are challenging for search-based approaches. The long horizon means:
- Value estimation must be accurate at many depths.
- Credit assignment is difficult (a setup decision may only pay off 100+ moves later).
- Policies must balance exploration (probing opponent pieces) with exploitation (attacking with known advantages).

### 3. Information Asymmetry Accumulation
Information is gained asymmetrically through:
- **Combat**: reveals both pieces involved. The attacker chooses when to probe.
- **Movement patterns**: immobile pieces are likely Bombs or Flag. Pieces moving like Scouts are revealed as Scouts.
- **Bluffing**: moving a low-rank piece aggressively to suggest it's high-rank.
- A strong player must maintain a "belief state" about opponent piece identities and update it with each observation.

### 4. Setup Strategy
The initial placement is a pure strategy game (no randomness) with enormous strategy space. Good setup must:
- Protect the Flag with Bombs.
- Position offensive pieces for attack.
- Create a balanced defense.
- Avoid predictable patterns that opponents can exploit.

### 5. Bluffing and Deception
Stratego involves deep bluffing:
- Moving a weak piece aggressively to suggest strength.
- Retreating a strong piece to suggest weakness.
- Sacrificing pieces to gather information.
- These deceptive behaviors emerge naturally from equilibrium play.

## Known Solver Results

### DeepNash (Perolat et al., DeepMind, 2022)
The landmark AI result for Stratego:
- Achieved expert human level on the Gravon online platform, ranking in the top 3 of human players.
- Published in *Science*.
- Architecture: **Regularized Nash Dynamics (R-NaD)** — a new game-theoretic deep RL algorithm.
- Key innovations:
  1. **R-NaD**: a principled RL algorithm that converges to Nash equilibrium in two-player zero-sum games, combining regularization with the Nash dynamics (replicator dynamics of evolutionary game theory).
  2. **Model-free approach**: no search at test time. The policy network directly outputs actions.
  3. **Handles massive information sets**: processes the full game state (own pieces + belief about opponent) through a neural network.
  4. **Emergent bluffing**: DeepNash learned to bluff, feint, and use deceptive piece movements without explicit programming of these behaviors.
- Training: large-scale distributed training on TPUs.

### Earlier Work
- **Master of the Flag** (de Boer, 2007): strong Stratego program using MC simulation and heuristic evaluation.
- **Probe** (various): early AI using alpha-beta search with chance nodes for unknown pieces.
- **Information Set MCTS (ISMCTS)**: applied to Stratego with moderate success (Cowling et al., 2012).
- **Neural Fictitious Self-Play (NFSP)**: applied to simplified Stratego variants (Lanctot et al., 2017).

### Barrage Variant
- 8×8 board with fewer pieces — used as a simplified testbed.
- Solved to a greater degree than full 10×10 Stratego.
- DeepNash was first validated on Barrage before scaling to full Stratego.

## Key Papers

| Year | Paper | Contribution |
|------|-------|--------------|
| 2007 | de Boer, "Invincible: Master of the Flag" | MC + heuristic Stratego AI |
| 2012 | Cowling et al., "Information Set MCTS" | ISMCTS applicable to Stratego |
| 2017 | Lanctot et al., "A Unified Game-Theoretic Approach to Multi-Agent RL" | NFSP for imperfect info games |
| 2022 | Perolat et al., "Mastering the Game of Stratego with Model-Free Multi-Agent RL and Search" | DeepNash (*Science*) |

## Relevance to Myosu

### Solver Applicability
Stratego is a **tier-1 imperfect-information game** distinct from card games:
- **CFR**: theoretically applicable but computationally intractable for full Stratego due to game tree size.
- **Deep RL (R-NaD)**: the proven approach. Model-free policy networks trained via self-play with game-theoretic guarantees.
- **MCTS**: information set MCTS is viable but weaker than deep RL for full Stratego.
- **No search at test time**: DeepNash demonstrates that pure policy networks can achieve expert play without real-time search, an important architectural finding.

### Architecture Ranking Factors
| Factor | Score | Notes |
|--------|-------|-------|
| CFR applicability | 1/5 | Game tree too large for CFR |
| Neural value network potential | 5/5 | Essential; DeepNash is purely neural |
| Abstraction necessity | 2/5 | Neural networks handle raw state directly |
| Real-time solving value | 2/5 | DeepNash succeeds without search |
| Transferability of techniques | 4/5 | R-NaD is a general algorithm for two-player zero-sum |

### Myosu Subnet Considerations
- **Unique game type**: Stratego is the only board game in the 20-game list (aside from Backgammon), providing architectural diversity.
- **DeepNash is model-free**: no search means low inference latency — suitable for real-time play on modest hardware.
- **Training compute**: R-NaD training requires significant distributed compute. The subnet could distribute training across solver nodes.
- **Evaluation**: head-to-head matches with Elo rating. Gravon rating system provides a human-comparable benchmark.
- **Game oracle**: simple deterministic rules — combat resolution, movement validation, victory detection are all straightforward.
- **Bluffing as emergent behavior**: Stratego's bluffing dynamics make it an excellent showcase for game-theoretic AI — the solver must learn deception.
- **Setup evaluation**: the initial army placement can be evaluated separately (does this setup protect the Flag? Is it predictable?).

### Recommended Approach for Myosu
1. Implement R-NaD or similar game-theoretic deep RL for training.
2. Policy networks for direct action selection (no search needed at inference time).
3. Evaluate via Elo rating from round-robin tournaments.
4. Consider Barrage (8×8) as a faster-to-solve validation variant.
5. Stratego demonstrates that myosu's solver framework works beyond card games — important for credibility and breadth.
