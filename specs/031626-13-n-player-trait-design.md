# Design Decision: N-Player Game Support in CFR Trait System

Source: Gap analysis — DESIGN.md defines 8 games with 3-4 players; GT-01..05 assumes 2-player
Status: Draft
Date: 2026-03-30
Depends-on: GT-01..05 (game engine traits), MG-01..04 (multi-game architecture)
Blocks: All non-2-player game implementations (Stage 1-3)

## Problem

Robopoker's `CfrTurn` trait returns variants for `Chance`, `Terminal`, and
`Player(usize)`. The MCCFR implementation in `rbp-mccfr` iterates over exactly
two players (index 0 and 1) when computing counterfactual regret. The
`Profile::exploitability()` method computes best-response for exactly 2 players
and sums them.

DESIGN.md now specifies mockups for 8 games with >2 players:

| Game | Players | Structure | OS.md # |
|------|---------|-----------|---------|
| NLHE 6-max | 2-6 | Free-for-all | 2 |
| Dou Di Zhu | 3 | Asymmetric 2v1 (landlord vs peasants) | 16 |
| Riichi Mahjong | 4 | Free-for-all | 9 |
| Bridge | 4 | 2v2 partnership | 10 |
| Spades | 4 | 2v2 partnership | 14 |
| Call Break | 4 | Free-for-all | 19 |
| Pusoy Dos | 4 | Free-for-all | 17 |
| Tien Len | 4 | Free-for-all | 18 |

This is 40% of the game portfolio. The trait system must accommodate them
without breaking 2-player games or the existing solver pipeline.

## Current Truth

- the live repo still centers on 2-player stage-0 games and 2-player solver
  assumptions
- there is no in-repo n-player solver path or additive `NPlayerGame` trait
  layer yet
- this document remains an honest Stage 1-3 design decision rather than a
  partially implemented stage-0 surface

## Decision

**Tiered approach: keep 2-player CFR as the primary solver, add n-player
support as a trait extension, not a replacement.**

### Tier 1: 2-player CFR (unchanged)

Games 1, 3, 4, 5, 6, 7, 8, 11, 13, 15, 20 remain 2-player. They use
robopoker's `CfrGame` + `Profile` + `Encoder` exactly as spec'd in GT-01..05.
No changes needed.

Liar's Dice (15) is 2-player in MG-01. Gin Rummy (11) is 2-player. OFC (13) is
2-player. Teen Patti (6) is 2-player in solver mode (fold equity against one
opponent).

### Tier 2: Reducible n-player games (adapter pattern)

Some n-player games reduce to 2-player for solving purposes:

**Partnership games (Bridge, Spades):** A 2v2 game where partners share a
utility function is strategically equivalent to a 2-player game where each
"player" is a team. The solver sees Team A vs Team B. At query time, the
individual player's information set maps to the team's strategy.

```rust
struct PartnershipAdapter<G: NPlayerGame> {
    inner: G,
}

impl CfrGame for PartnershipAdapter<BridgeGame> {
    // Player(0) = NS partnership, Player(1) = EW partnership
    // Information set includes only the individual hand (not partner's)
    // Utility is shared between partners
}
```

**Landlord games (Dou Di Zhu):** A 2v1 game where the two peasants share
utility. Reduce to landlord vs peasant-team.

```rust
struct LandlordAdapter<G: NPlayerGame> {
    inner: G,
}

impl CfrGame for LandlordAdapter<DdzGame> {
    // Player(0) = landlord, Player(1) = peasant team
    // Info set: the ACTING peasant's hand + kitty (not partner's hand)
    // Both peasant seats map to Player(1) but with seat-specific info sets
    // Utility: landlord wins = +1, peasants win = +1 (shared)
}
```

**Important subtlety**: the peasant team's info set must NOT be the union
of both peasants' hands. That would give the team perfect coordination
(seeing both hands), which is stronger than what individual peasants
actually have. Instead, the info set is the acting peasant's own hand +
the revealed kitty + play history. The other peasant's remaining cards
are hidden. This matches DouZero's modeling approach and produces a
realistic equilibrium.

This preserves Nash equilibrium guarantees because the team shares utility,
even though they have asymmetric information. CFR handles this correctly —
the team is one "player" with multiple decision points that have different
information available at each.

### Tier 3: True n-player games (new trait + solver)

Free-for-all games with 3+ independent utility functions cannot be reduced to
2-player. MCCFR does not converge to Nash for n>2 — it converges to a
**coarse correlated equilibrium** (CCE), which is still useful but weaker.

Games in this tier: Riichi Mahjong, Call Break, Pusoy Dos, Tien Len.

(NLHE 6-max is NOT in this tier — see "NLHE 6-max special case" below.
It uses position-indexed 2-player modeling.)

**New trait for n-player games:**

This intentionally does NOT extend `CfrGame` despite similar method
signatures. The differences are structural: no `Copy` bound (n-player
states are too large), `num_players()` method, `utility()` takes a
player index, and the solver implementation is fundamentally different
(External Sampling MCCFR vs standard MCCFR). Forced inheritance would
require `Copy` workarounds that pollute every n-player implementation.

```rust
pub trait NPlayerGame: Clone + Send + Sync {
    type State: Clone + Serialize;
    type Action: Clone + Serialize + Eq + Hash;
    type Info: Clone + Serialize + Eq + Hash;

    fn num_players(&self) -> usize;
    fn initial_state(&self) -> Self::State;
    fn current_player(state: &Self::State) -> NPlayerTurn;
    fn legal_actions(state: &Self::State) -> Vec<Self::Action>;
    fn apply(state: &Self::State, action: &Self::Action) -> Self::State;
    fn is_terminal(state: &Self::State) -> bool;
    fn utility(state: &Self::State, player: usize) -> f64;
    fn information_set(state: &Self::State, player: usize) -> Self::Info;
}

pub enum NPlayerTurn {
    Player(usize),   // 0..num_players
    Chance,
    Terminal,
}
```

**Solver for n-player:** External Sampling MCCFR generalizes to n players.
Each iteration samples one player's decisions and updates that player's
regrets while treating opponents' strategies as fixed. Convergence is to CCE
rather than Nash, but:
- CCE contains all Nash equilibria as special cases
- For many games, CCE exploitability is comparable to 2-player Nash
- Superhuman AI for Mahjong (Suphx) and Dou Di Zhu (DouZero) use similar approaches

**New exploitability metric for n-player:**

Standard exploitability (sum of best-response values) doesn't apply to n>2.
Instead, use **individual exploitability**: for each player i, compute the
best-response value of an adversary replacing player i while all other players
follow the strategy. The individual exploitability is the maximum over all
players.

```rust
pub trait NPlayerExploitability {
    type Game: NPlayerGame;

    fn individual_exploitability(
        strategy: &NPlayerProfile<Self::Game>,
        player: usize,
        sample_count: usize,
    ) -> f64;

    fn max_exploitability(
        strategy: &NPlayerProfile<Self::Game>,
        sample_count: usize,
    ) -> f64 {
        (0..strategy.num_players())
            .map(|p| Self::individual_exploitability(strategy, p, sample_count))
            .fold(f64::NEG_INFINITY, f64::max)
    }
}
```

### How this affects GT-01..05

**No changes to existing traits.** The 2-player `CfrGame` + `Profile` +
`Encoder` system remains the primary path. The n-player extensions are
additive:

| Trait | 2-player (existing) | N-player (new) |
|-------|--------------------|--------------------|
| Game state | `CfrGame` | `NPlayerGame` |
| Turn type | `CfrTurn` | `NPlayerTurn` |
| Strategy | `Profile` | `NPlayerProfile` |
| Exploitability | `Profile::exploitability()` | `NPlayerExploitability::max_exploitability()` |
| Encoder | `Encoder` | `NPlayerEncoder` |
| Solver | MCCFR (2-player) | External Sampling MCCFR (n-player) |

The `GameRegistry` (GT-03) already has `num_players: u8`. It dispatches to the
correct trait system based on player count.

### How this affects the validator

The validator oracle (VO-01..07) currently calls `Profile::exploitability()`.
For n-player subnets, it calls `NPlayerExploitability::max_exploitability()`
instead. The dispatch is by `GameType` — the validator knows which game it's
scoring.

The weight calculation remains the same: `weight = max(0, 1 - exploit / baseline)`.
The exploit metric just comes from a different function.

### How this affects the TUI

DESIGN.md already handles this correctly. The `GameRenderer` trait is
agnostic to player count — each game renders its own state panel. The
multiplayer seating conventions (section 5: "4 (mahjong): east (you),
south, west, north") are game-specific rendering logic.

### How this affects the miner

A miner for an n-player game trains using External Sampling MCCFR instead
of the 2-player variant. The axon endpoint returns the same
`StrategyResponse` — action distributions for a given information set.
The miner binary selects the solver based on `GameType`.

## NLHE 6-max special case

NLHE 6-max is listed as 2-6 players. In practice, poker variants are solved
by treating each decision point as a 2-player subgame (acting player vs "the
field"). This is how PioSolver and MonkerSolver work. The solver trains
strategies for each position (UTG, MP, CO, BTN, SB, BB) independently.

**Recommendation:** NLHE 6-max uses the 2-player `CfrGame` trait with a
position-indexed abstraction, NOT the n-player trait. Each position's
decision is modeled as a 2-player game (hero vs rest-of-table). This is
standard in poker solvers and avoids the CCE convergence limitations.

## Implementation timeline

| Phase | Games | Trait | Solver |
|-------|-------|-------|--------|
| Phase 0 | NLHE HU | `CfrGame` (2p) | MCCFR |
| Stage 1 | NLHE 6-max, PLO, Short Deck, Tournament | `CfrGame` (2p) | MCCFR |
| Stage 2 | Bridge, Spades, Dou Di Zhu | `CfrGame` via adapter | MCCFR |
| Stage 2 | Riichi, Call Break, Pusoy Dos, Tien Len | `NPlayerGame` | Ext. Sampling MCCFR |

## Open questions

1. **Should `NPlayerGame` extend `CfrGame` or be independent?** Current
   recommendation: independent. The `Copy` bound on `CfrGame` doesn't suit
   n-player games with variable state, and the solver implementations are
   fundamentally different. Bridge them via `GameRegistry` dispatch.

2. **Is CCE exploitability sufficient for validator scoring?** For the
   incentive mechanism, we need miners to be rankable. CCE exploitability
   provides a total ordering, but the gap between "good CCE" and "Nash" is
   unknown for most games. Empirically, this works — DouZero and Suphx use
   similar metrics. Formal analysis is future work.

3. **Should partnership adapters handle asymmetric information?** In Bridge,
   partners can't see each other's hands. The adapter must model this: the
   info set for the partnership "player" is the acting individual's hand,
   not the combined hands. This is correct but subtle — need to verify
   CFR convergence with asymmetric team information sets.

## Decision log

- 2026-03-17: Tiered approach chosen over "rewrite all traits for n-player."
  Rationale: 60% of games are 2-player or reducible. Rewriting would delay
  Phase 0 for games that won't ship until Stage 2-3.
- 2026-03-17: Partnership adapter pattern chosen for Bridge/Spades/DDZ.
  Rationale: preserves Nash guarantees, reuses existing solver.
- 2026-03-17: NLHE 6-max classified as 2-player (position-indexed).
  Rationale: industry standard — PioSolver, MonkerSolver all model 6-max
  as independent per-position decisions.
- 2026-03-17: External Sampling MCCFR chosen for true n-player.
  Rationale: proven for n-player imperfect info (OpenSpiel, DouZero).
  Converges to CCE, which is sufficient for ranking miners.
