# Specification: Liar's Dice Engine

Source: Reverse-engineered from crates/myosu-games-liars-dice (game.rs, solver.rs)
Status: Draft
Depends-on: 001-game-trait-framework

## Purpose

The Liar's Dice engine implements a minimal two-player imperfect-information
game that proves the game trait framework supports multiple games without
requiring changes to existing game code. It provides a complete CFR game tree,
solver, and state representation for a simplified Liar's Dice variant. Its
existence validates that the miner, validator, and gameplay surfaces can operate
on any game that implements the framework traits.

The primary consumer is the system itself: Liar's Dice serves as the
second-game proof artifact for multi-game extensibility.

## Whole-System Goal

Current state: The engine is fully implemented with a complete CFR game tree,
solver, and integration into the miner and play binaries. It coexists with
poker on the same chain and same node-owned loop.

This spec adds: Nothing new. This documents the existing behavioral contract.

If all ACs land: The system demonstrably supports two independent
imperfect-information games through the same trait framework, proving that a
third game can be added without modifying poker or Liar's Dice code.

Still not solved here: Wire protocol encoding for Liar's Dice network transport,
artifact-backed blueprints, and on-chain game type coordination remain potential
future work.

## Scope

In scope:
- Two-player, one-die-per-player Liar's Dice game tree
- CFR trait implementations (CfrGame, CfrTurn, CfrEdge, CfrInfo)
- Chance node dealing, bidding, and challenge resolution
- Payoff computation
- Information set separation (public vs. secret state)
- MCCFR solver for strategy computation

Out of scope:
- Multi-die or multi-player Liar's Dice variants
- Wire protocol encoding for network transport
- Artifact-backed blueprint inference
- TUI rendering specifics for Liar's Dice
- On-chain registration of Liar's Dice subnets

## Current State

The crate exists at crates/myosu-games-liars-dice with approximately 2,100 lines
of code. It implements a minimal game variant: two players, one die each, six
faces per die.

The game tree starts with a Chance node that rolls both dice (36
equally-likely outcomes). Play alternates between P1 and P2, who may bid
(claiming a count and face value) or challenge the last bid. A bid must be
strictly higher than the previous bid. Challenge resolves the game: if the
actual count meets or exceeds the claimed count, the claimant wins; otherwise
the challenger wins. Payoff is +1.0 for the winner and -1.0 for the loser.

The information set cleanly separates public state (whose turn, last claim) from
secret state (the acting player's die value). This separation is required for
correct MCCFR convergence.

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|---|---|---|---|
| Game tree | LiarsDiceGame with Chance/P1/P2/Terminal states | Reuse | Complete minimal game implementation |
| CFR traits | CfrGame, CfrTurn, CfrEdge, CfrInfo implementations | Reuse | Proper trait compliance proven by solver convergence |
| Solver | MCCFR solver for Liar's Dice strategy computation | Reuse | Produces convergent strategies |
| Information sets | public() and secret() view separation | Reuse | Required for correct imperfect-information solving |
| Payoff | +1/-1 terminal evaluation | Reuse | Zero-sum two-player payoff |

## Non-goals

- Scaling to tournament-style multi-player Liar's Dice.
- Providing wire-encoded network transport for Liar's Dice strategies.
- Building artifact-backed blueprints analogous to the poker blueprint system.
- Optimizing the solver for large game trees (the game is intentionally small).
- Replacing poker as the primary game vertical.

## Behaviors

The game begins with a Chance node that produces all 36 possible die-roll
combinations with equal probability (1/36 each). After dealing, P1 acts first.

On each player's turn, the available actions are: all bids strictly higher than
the last bid (ordered by count then face), plus a challenge action if a bid has
been made. The first player to act may only bid (no challenge with no prior
bid).

A bid specifies a count (1 or 2) and a face (1 through 6). Higher means either
a higher count, or the same count with a higher face.

Challenge resolution counts how many dice across both players show the claimed
face value (or higher, depending on variant). If the actual count meets or
exceeds the claim, the claimant wins. Otherwise the challenger wins.

The information set for a player includes the turn indicator, the last claim
(public), and their own die value (secret). The opponent's die value is hidden.
This separation ensures the solver cannot exploit hidden information.

The solver uses MCCFR over this game tree to compute approximate Nash
equilibrium strategies. Because the game tree is small (36 chance outcomes, at
most ~20 actions per decision point), the solver converges quickly.

## Acceptance Criteria

- The game tree enumerates all 36 chance outcomes with equal probability.
- Players can only bid strictly higher than the last bid.
- Challenge is only available when a prior bid exists.
- Terminal payoff is +1.0 for the winner and -1.0 for the loser.
- Information sets correctly hide the opponent's die value from the acting
  player.
- The MCCFR solver produces a convergent strategy (exploitability decreases over
  iterations).
- The Liar's Dice game operates through the same game trait interface as poker
  without requiring modifications to the poker crate.
