# myosu-games-liars-dice

Liar's Dice game engine implementation for the myosu game-solving chain.

## Overview

This crate provides a complete CFR implementation for the 1-die-each variant of Liar's Dice:

- **2 players**, each with **1 die** (6 faces)
- **Variable-length bid history** (up to 12 bids before forced challenge)
- **~24,576 information sets** — converges in seconds to exact Nash

## Game Rules

1. Both players roll their die privately (only they see their die)
2. Player 0 starts, placing the first bid `(quantity, face)`
3. Players alternate bidding; each bid must **strictly increase** the claim
   - Same quantity: must claim a higher face
   - Higher quantity: any face allowed
4. A player may **Challenge** instead of bidding
5. If challenged, the bid is resolved: count total dice showing the claimed face
   - If count >= bid quantity: challenger loses, bidder wins
   - If count < bid quantity: challenger wins, bidder loses
6. Payoff: +1 for win, -1 for loss, 0 for tie

## Architecture

Implements the CFR trait system from `myosu-games`:

| Module | Type | Trait |
|--------|------|-------|
| `game.rs` | `LiarsDiceGame` | `CfrGame` |
| `edge.rs` | `LiarsDiceEdge` | `CfrEdge` |
| `turn.rs` | `LiarsDiceTurn` | `CfrTurn` |
| `info.rs` | `LiarsDiceInfo` | `CfrInfo` |
| `encoder.rs` | `LiarsDiceEncoder` | `Encoder` |
| `profile.rs` | `LiarsDiceProfile` | `Profile` |

## Proof of Architecture

This crate validates that the CFR trait system in `myosu-games` is **game-agnostic**:

- Zero changes required to `myosu-games` or `myosu-games-poker`
- All traits satisfied via enum dispatch (no `dyn Trait` needed)
- Exploitability converges to < 0.001 mbb/h

## References

- AC-MG-01: CfrGame implementation
- AC-MG-02: Solver + Nash convergence proof
- AC-MG-03: Zero-change architecture verification
