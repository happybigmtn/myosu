# Specification: Multi-Game Architecture — Liar's Dice Proof + Expansion Guide

Source: Master spec AC-FG-01
Status: Draft
Date: 2026-03-30
Depends-on: GT-01..05 (trait system), PE-01..04 (poker engine proves pattern)

## Purpose

Prove that myosu's architecture supports multiple games — not just poker — by
implementing Liar's Dice as a second game engine and documenting the expansion
path for backgammon, mahjong, bridge, and other candidates.

Liar's Dice is chosen because: (1) it's an imperfect-information game suitable
for CFR, (2) the state space is small enough to compute exact Nash equilibria
(exploitability = 0 is verifiable), (3) it validates that `CfrGame`, `CfrEdge`,
`CfrTurn`, `CfrInfo`, `Profile`, and `Encoder` traits generalize beyond poker.

**Key design constraint**: adding a new game must require ZERO changes to the
chain pallet, miner binary, or validator binary. A new game = new `GameEngine`
impl + new `Encoder` + new `GameType` enum variant.

## Whole-System Goal

Current state:
- The additive second-game proof is already real locally: Liar's Dice has its
  own crate, local play surface, miner/validator compatibility, and owned
  two-subnet coexistence proof beside poker
- Game engine traits already support both poker and Liar's Dice without chain
  rewrites
- The remaining gap is not whether multi-game works at all; it is how far the
  repo wants to standardize the future expansion guide beyond the current
  executable proof

This spec now serves two purposes:
- record the already-landed Liar's Dice proof honestly
- preserve the future expansion guidance for additional games beyond poker and
  Liar's Dice

If all ACs land:
- Liar's Dice implements `CfrGame`, `CfrInfo`, `Profile`, `Encoder`
- A Liar's Dice solver trains to Nash equilibrium
- Exploitability computation verifies convergence (= 0 for exact solution)
- Architecture doc explains how to add backgammon, mahjong, bridge
- No existing code was modified to add the new game

Still not solved here:
- Actual backgammon/mahjong/bridge implementations (future specs)
- Multi-game miner (serving multiple games simultaneously)
- Cross-game abstraction sharing
- Game-specific validator oracle tuning

12-month direction:
- 5+ game implementations in production subnets
- Game developer SDK for third-party game engines
- Automated game registration via governance

## Why This Spec Exists As One Unit

- The Liar's Dice implementation and expansion documentation together prove
  one claim: "myosu is a multi-game platform"
- The implementation proves it technically (the traits work for non-poker)
- The documentation proves it strategically (the path to 5+ games is clear)
- Neither alone is sufficient — code without docs doesn't guide, docs without
  code doesn't convince

## Scope

In scope:
- Liar's Dice game engine implementing robopoker CFR traits
- Liar's Dice encoder (trivial — direct enumeration, no abstraction needed)
- Liar's Dice profile and solver
- Exact Nash equilibrium verification (exploitability = 0)
- GameType::LiarsDice registry variant
- Architecture expansion guide for 6 candidate games
- Verification that no existing code changes are needed

Out of scope:
- Backgammon, mahjong, bridge implementations — those are future specs
- Liar's Dice gameplay CLI (optional enhancement, not core)
- Multi-game miner support
- Game-specific validator tuning

## Current State

- `myosu-games-liars-dice/` already exists with game, solver, renderer,
  protocol, and wire modules
- `myosu-play` can already load and render a local Liar's Dice surface
- The local owned chain harness already proves poker and Liar's Dice as
  distinct subnets coexisting on the same stage-0 chain

## What Already Exists

| Sub-problem | Existing code / flow | Reuse / extend / replace | Why |
|-------------|----------------------|--------------------------|-----|
| CFR traits | `rbp-mccfr` re-exports (GT-01) | reuse | Game-agnostic by design |
| RPS reference | `rbp-mccfr::rps` | reference | Shows how small games implement traits |
| Poker pattern | `myosu-games-poker` (PE-*) | reference | Pattern for wrapping a game engine |
| Exploitability | `Profile::exploitability()` | reuse | Works for any CfrGame |
| Live second-game proof | `crates/myosu-games-liars-dice/` + `myosu-play` + local chain harness | extend | The additive architecture has already been proven on a second real game |

## Non-goals

- Optimal Liar's Dice UX — CLI gameplay is optional, not required
- Large-variant Liar's Dice (6 dice per player) — start with 1 die each
- Multiplayer (3+) Liar's Dice — 2-player for trait validation

## Ownership Map

| Component | Status | Location |
|-----------|--------|----------|
| Liar's Dice game + protocol | Live | crates/myosu-games-liars-dice/src/game.rs, crates/myosu-games-liars-dice/src/protocol.rs |
| Liar's Dice solver | Live | crates/myosu-games-liars-dice/src/solver.rs |
| Liar's Dice renderer | Live | crates/myosu-games-liars-dice/src/renderer.rs |
| Liar's Dice wire surface | Live | crates/myosu-games-liars-dice/src/wire.rs |
| Expansion guide | Not yet split into a standalone doc | future documentation surface |

---

## Acceptance Criteria

### AC-MG-01: Liar's Dice Game Engine

- Where: `crates/myosu-games-liars-dice/`
- How: Implement `CfrGame` for 2-player Liar's Dice with 1 die each (6 faces):

  **Rules**: Each player rolls one die (hidden). Players alternate bidding
  quantities of a face value ("I bid two 3s"). A bid claims that the total
  count of that face across all dice is at least the bid amount. Each bid
  must raise the quantity or the face value. A player can challenge ("liar!")
  instead of bidding. If challenged, dice are revealed: if the bid was true,
  the challenger loses; if false, the bidder loses.

  **State**: `LiarsDiceGame { dice: [u8; 2], bids: Vec<Bid>, current_player: u8 }`
  where `Bid { quantity: u8, face: u8 }`.

  **Types**:
  - `LiarsDiceGame: CfrGame` (Clone + Copy for small state)
  - `LiarsDiceEdge: CfrEdge` — `Bid(quantity, face)` or `Challenge`
  - `LiarsDiceTurn: CfrTurn` — `Player(0)`, `Player(1)`, `Chance`, `Terminal`
  - `LiarsDiceInfo: CfrInfo` — (my die, bid history) — player can't see opponent's die

  State space: 36 die outcomes × 4,095 bid sequences ≈ **147,420 terminal
  states** and **~24,576 information sets**. (max_bids = max_quantity(2) ×
  num_faces(6) = 12 distinct ordered bids; 2^12 - 1 = 4,095 strictly
  increasing subsequences ending in challenge.) Still trivially solvable —
  converges in seconds, comparable to Kuhn poker (~30 info sets).

- Whole-system effect: proves CfrGame works for non-poker games.
- State: game state (two dice, bid history).
- Wiring contract:
  - Trigger: solver creates game via `LiarsDiceGame::root()`
  - Callsite: solver training loop
  - State effect: game transitions through bids to terminal
  - Persistence effect: N/A (ephemeral game instances)
  - Observable signal: `cargo test -p myosu-games-liars-dice` passes
- Required tests:
  - `cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node`
  - `cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase`
  - `cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game`
  - `cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum`
  - `cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied`
- Pass/fail:
  - Root state is a chance node (dice roll)
  - After dice dealt, it's Player 0's turn
  - Legal bids must exceed previous bid
  - Challenge after a truthful bid → challenger gets payoff -1
  - Challenge after a bluff → bidder gets payoff -1
  - `LiarsDiceGame` satisfies `CfrGame` (Clone + Copy + Send + Sync)
- Blocking note: this is the architectural proof. If Liar's Dice can't
  implement CfrGame, the multi-game claim is false.
- Rollback condition: CfrGame: Copy constraint is impossible for Liar's Dice
  state (bid history is variable-length).

### AC-MG-02: Liar's Dice Solver and Nash Verification

- Where: `crates/myosu-games-liars-dice/ (extend)`
- How: Implement `Encoder` and `Profile` for Liar's Dice. Train to convergence
  (exact Nash for this small game). Verify exploitability = 0.

  The encoder is trivial: info set = (my die face, bid history). No abstraction
  needed because the state space is small enough to enumerate exactly.

  Train for 10,000+ iterations. Verify:
  - Exploitability < 0.001 (effectively zero)
  - Strategy is non-trivial (not always challenge, not always bid)
  - Strategy probabilities sum to 1.0 at every info set

- Whole-system effect: proves the solver + exploitability pipeline works for
  a game with a known exact solution.
- State: Liar's Dice profile (regrets, strategies).
- Wiring contract:
  - Trigger: test harness creates and trains solver
  - Callsite: test suite
  - State effect: solver converges to Nash
  - Persistence effect: N/A (test only)
  - Observable signal: exploitability assertion passes
- Required tests:
  - `cargo test -p myosu-games-liars-dice solver::tests::train_to_nash`
  - `cargo test -p myosu-games-liars-dice solver::tests::exploitability_near_zero`
  - `cargo test -p myosu-games-liars-dice solver::tests::strategy_is_nontrivial`
  - `cargo test -p myosu-games-liars-dice solver::tests::wire_serialization_works`
- Pass/fail:
  - After 10,000 iterations, exploitability < 0.001
  - Strategy at info set (die=3, no bids yet) is mixed (not deterministic)
  - All action probabilities in [0.0, 1.0] and sum to ~1.0
  - WireStrategy serialization round-trips for Liar's Dice types
- Blocking note: exact Nash verification is the strongest possible proof
  that the trait system and exploitability computation work correctly.
- Rollback condition: convergence requires > 100,000 iterations (indicates
  implementation error in the game rules).

### AC-MG-03: Zero-Change Verification

- Where: `crates/myosu-games-liars-dice/tests/ (new)`
- How: Integration test proving no existing crate was modified:

  ```rust
  #[test]
  fn adding_liars_dice_required_no_changes_to_existing_crates() {
      // This test passes by construction: if Liar's Dice compiles
      // and all poker tests still pass, no existing code was broken.
      // The test documents the architectural invariant.
  }
  ```

  Run the full poker test suite alongside Liar's Dice to prove no regression.
  Verify that `myosu-games`, `myosu-games-poker`, and the chain pallet
  have no diff in their source files.

- Required tests:
  - `cargo test -p myosu-games` — all pass (no changes)
  - `cargo test -p myosu-games-poker` — all pass (no changes)
  - `cargo test -p myosu-games-liars-dice` — all pass (new)
- Pass/fail:
  - All existing tests pass without modification
  - Liar's Dice tests pass
  - `git diff crates/myosu-games/src/ crates/myosu-games-poker/src/` is empty
- Blocking note: the zero-change property is the architectural claim.
- Rollback condition: existing traits need modification for Liar's Dice.

### AC-MG-04: Multi-Game Expansion Guide

- Where: `docs/multi-game-expansion.md (new)`
- How: Document the expansion path for each candidate game:

  | Game | CFR Suitable? | State Space | Key Challenge | Estimated Effort |
  |------|--------------|-------------|---------------|------------------|
  | **PLO** | Yes | 10x NLHE | 4 hole cards, larger branching | Medium (reuse NLHE patterns) |
  | **Short Deck** | Yes | 0.3x NLHE | 36-card deck, different hand rankings | Small (variant of NLHE) |
  | **Backgammon** | Partially | Large | Dice + continuous decisions, doubling cube | Large (new game mechanics) |
  | **Mahjong (Riichi)** | Partially | Massive | 4-player, tile draws, scoring rules | Very large (complex state) |
  | **Bridge** | Yes | Large | Partnership, bidding conventions | Large (4-player + bidding) |
  | **Liar's Dice (6 dice)** | Yes | Medium | Scale from 1-die proof | Small (scale existing) |

  For each game, document:
  1. How to implement `CfrGame` (state representation, actions, terminal payoffs)
  2. How to implement `Encoder` (what abstraction is needed)
  3. Estimated information set count and memory requirements
  4. Exploitability computation feasibility
  5. Subnet configuration (tempo, max neurons, etc.)
  6. Prerequisite research or libraries

- Required tests: N/A (documentation)
- Pass/fail:
  - Document covers all 6 candidate games
  - Each game has concrete type signatures for CfrGame impl
  - Each game has an estimated info set count
  - Document reviewed and approved
- Blocking note: guides future development and investor conversations.
- Rollback condition: N/A.

---

## Decision Log

- 2026-03-16: Liar's Dice with 1 die each — smallest possible variant that
  exercises all traits. 6-die variant is a scaling test, not an architecture test.
- 2026-03-16: Zero-change verification as an explicit AC — the architectural
  claim "no chain changes needed" must be tested, not just asserted.
- 2026-03-16: CfrGame: Copy may be a problem for variable-length bid history —
  can work around with fixed-size array (max 12 bids for 1-die game) or
  bitpacked representation.

## Milestone Verification

| # | Scenario | Validates | ACs |
|---|----------|-----------|-----|
| 1 | Liar's Dice game runs from dice roll to challenge | Game engine | MG-01 |
| 2 | Solver trains to exploitability < 0.001 | Nash convergence | MG-02 |
| 3 | All existing tests pass without changes | Architecture | MG-03 |
| 4 | Expansion guide covers 6 candidate games | Documentation | MG-04 |
| 5 | Full pipeline: train Liar's Dice → serialize → exploit → score | Integration | MG-01, MG-02 |
