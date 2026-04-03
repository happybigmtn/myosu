# Specification: Poker Variant Family — NLHE 6-max, PLO, Short Deck, Tournament/ICM

Historical note: this is a future expansion spec built on the live NLHE HU
wrapper, not a current stage-0 implementation target. The repo does not yet
ship dedicated six-max, PLO, short-deck, or tournament modules.

Source: OS.md games 2-5, DESIGN.md 9.10-9.13
Status: Draft
Date: 2026-03-30
Depends-on: PE-01..04 (NLHE HU engine), AP-01..03 (abstraction pipeline), GT-01..05 (traits)
Blocks: Stage 1 game expansion

## Purpose

Define the implementation path for the four poker variants that follow NLHE
heads-up. All four share the core poker engine (deal → streets → showdown)
and MCCFR solver approach, but each introduces distinct challenges:

| Variant | Key difference from NLHE HU | State space impact |
|---------|---------------------------|-------------------|
| NLHE 6-max | 2-6 players, position-dependent | ~100x HU (per position) |
| PLO | 4 hole cards (vs 2) | ~10x NLHE (hand abstraction) |
| Short Deck (6+) | 36-card deck, altered hand rankings | ~0.3x NLHE |
| Tournament/ICM | Stack-dependent utility (non-linear) | ~5x NLHE (stack abstraction) |

These four represent the $6B+ poker solver market. PioSolver charges $250+
for NLHE. MonkerSolver's entire business is PLO. No ICM-aware CFR solver
exists. Short Deck has limited solver support.

## Current Truth

- the current poker surface lives in `crates/myosu-games-poker/` and remains
  centered on NLHE HU plus shared solver/artifact/renderer utilities
- no variant-specific modules such as `sixmax.rs`, `plo.rs`, `shortdeck.rs`, or
  `tournament.rs` exist yet
- the current renderer reference remains
  `crates/myosu-games-poker/src/renderer.rs`
- this spec is therefore a future family-expansion plan layered on the current
  heads-up poker base

## Architecture

All variants use the 2-player `CfrGame` trait (per 031626-13 decision):

- **NLHE 6-max**: position-indexed 2-player model. Each position (UTG, MP, CO,
  BTN, SB, BB) trains an independent solver against "the field." This is
  industry standard (PioSolver, GTO+).
- **PLO, Short Deck, Tournament**: genuinely 2-player (or modeled as such).

All variants share:
- Card types from `rbp-cards` (Rank, Suit, Card, Hand)
- Street structure (preflop → flop → turn → river)
- Pot and stack management
- Showdown evaluation (with variant-specific hand rankings)

Each variant needs:
- Its own `CfrGame` implementation (different deal, different actions)
- Its own `Encoder` (different abstraction granularity)
- Its own `GameParams` variant in the registry
- Its own `GameRenderer` for the TUI

## Scope

In scope:
- PV-01: NLHE 6-max game engine (position-indexed model)
- PV-02: PLO game engine (4-card Omaha with pot-limit betting)
- PV-03: Short Deck game engine (36-card, modified rankings)
- PV-04: Tournament/ICM game engine (stack-dependent utility)
- PV-05: Shared poker abstraction framework (extend AP-01..03)
- PV-06: Variant-specific TUI renderers (all 4 game screens)

Out of scope:
- Mixed games (alternating variants) — future
- Straddles, antes, forced bets beyond standard — configurable later
- Pot-Limit NLHE or other hybrid betting structures
- NLHE heads-up — already covered by PE-01..04

---

## Acceptance Criteria

### AC-PV-01: NLHE 6-max Engine

- Where: `crates/myosu-games-poker/src/sixmax.rs (new)`
- How: Extend the NLHE engine for 2-6 players. Key differences from HU:

  **Position model**: The solver trains one strategy per position. At query
  time, the miner receives the hero's position + game state and returns the
  strategy for that position.

  ```rust
  pub struct Nlhe6maxGame {
      num_players: u8,       // 2-6, configurable
      position: Position,    // which position this solver instance covers
      // ... standard poker state
  }

  pub enum Position { Utg, Mp, Co, Btn, Sb, Bb }
  ```

  **Preflop ranges**: 6-max preflop has ~170 distinct starting hands per
  position (vs ~170 for HU but with position-dependent ranges). The
  abstraction pipeline clusters these independently per position.

  **Solver structure**: One `CfrGame` instance covers the full 6-max game
  tree with position as part of the state. The miner trains a single MCCFR
  solver that covers all positions. At query time, the validator or player
  specifies a position, and the strategy for that seat is extracted from
  the single trained profile. This avoids 6x memory/training cost compared
  to training independent per-position solvers.

  **GameParams**:
  ```rust
  GameParams::Nlhe6max {
      num_players: u8,
      stack_bb: u32,
      ante_bb: Option<u32>,
  }
  ```

- Required tests:
  - `sixmax::tests::deal_correct_for_n_players`
  - `sixmax::tests::blind_posting_correct_by_position`
  - `sixmax::tests::legal_actions_include_all_in`
  - `sixmax::tests::pot_splits_correctly_with_side_pots`
  - `sixmax::tests::position_indexed_info_sets_distinct`
  - `sixmax::tests::folded_player_excluded_from_showdown`
- Pass/fail:
  - 6-player deal produces 6 hands + community cards
  - SB/BB posted correctly for each player count (2-6)
  - Side pots computed correctly when short stacks are all-in
  - Info sets for BTN and UTG are distinct even with same cards (position matters)

### AC-PV-02: PLO Engine

- Where: `crates/myosu-games-poker/src/plo.rs (new)`
- How: Pot-Limit Omaha with 4 hole cards. Key differences from NLHE:

  **4 hole cards**: Player receives 4 cards, must use exactly 2 for their
  final hand. This creates C(4,2) = 6 possible 2-card combinations per
  player, dramatically increasing the hand abstraction space.

  **Pot-limit betting**: Maximum raise = current pot size. This bounds the
  game tree but doesn't reduce it as much as fixed-limit. The `CfrEdge`
  action space includes fold, check, call, and pot-fraction raises
  (1/3 pot, 1/2 pot, 2/3 pot, pot).

  **Hand evaluation**: Uses the same `rbp-cards` evaluator but with the
  Omaha constraint (exactly 2 from hand + exactly 3 from board). Need to
  wrap robopoker's evaluator or implement the combinatorial selection.

  **Abstraction challenge**: PLO's hand space is C(52,4) = 270,725
  starting hands (vs 1,326 for NLHE). Clustering must be more aggressive.
  Earth Mover's Distance between hand equity distributions is standard.

  **Compute warning**: PLO clustering is ~200x more expensive than NLHE
  (270K vs 1.3K hands, pairwise EMD is O(n²)). This is a batch/offline
  operation — the abstraction pipeline runs once to produce artifact files,
  then miners load the pre-computed artifacts at startup. Clustering PLO
  may take hours on a single core; parallelism across hands is trivial.

  **GameParams**:
  ```rust
  GameParams::Plo {
      num_hole_cards: u8,    // 4 (standard) or 5 (PLO-5)
      stack_bb: u32,
  }
  ```

- Required tests:
  - `plo::tests::deal_four_hole_cards`
  - `plo::tests::pot_limit_max_raise_correct`
  - `plo::tests::hand_evaluation_uses_exactly_two_from_hand`
  - `plo::tests::abstraction_clusters_270k_hands`
  - `plo::tests::plo5_deals_five_hole_cards`
- Pass/fail:
  - Player dealt exactly 4 hole cards
  - Maximum raise equals current pot size (not unlimited)
  - Hand evaluation uses best 2-from-hand + 3-from-board combination
  - Abstraction pipeline clusters PLO hands into configurable bucket count

### AC-PV-03: Short Deck Engine

- Where: `crates/myosu-games-poker/src/shortdeck.rs (new)`
- How: NLHE with a 36-card deck (remove 2-5 of each suit). Key differences:

  **Modified deck**: 36 cards (6 through A). Probability distributions
  change — flush is rarer (fewer suited cards per rank), full house is
  more common.

  **Modified hand rankings**: Flush beats full house. Three-of-a-kind
  beats straight. A-6-7-8-9 is a valid straight (A wraps low to 6, not 5).

  **Smaller state space**: ~30% of NLHE. Faster training, faster convergence.
  Good first expansion target after HU because the engine is almost identical.

  **Implementation**: Fork the NLHE engine. Change the deck construction
  and hand ranking table. Everything else (betting, streets, showdown
  logic) is inherited.

  ```rust
  pub struct ShortDeckGame {
      // Same as NlheGame but with deck: Deck::short_deck()
      // and evaluator: ShortDeckEvaluator
  }
  ```

- Required tests:
  - `shortdeck::tests::deck_has_36_cards`
  - `shortdeck::tests::no_cards_below_six`
  - `shortdeck::tests::flush_beats_full_house`
  - `shortdeck::tests::ace_six_straight_valid`
- Pass/fail:
  - Deck contains exactly 36 cards (6-A of each suit)
  - Hand evaluator ranks flush above full house
  - A♠ 6♠ 7♥ 8♦ 9♣ is evaluated as a straight

### AC-PV-04: Tournament/ICM Engine

- Where: `crates/myosu-games-poker/src/tournament.rs (new)`
- How: NLHE with stack-dependent utility (Independent Chip Model).

  **ICM utility**: In tournaments, chips won ≠ money won. A player with
  50% of chips does not have 50% of prize pool equity. ICM converts chip
  stacks to prize pool equity using a recursive probability model.

  ```rust
  pub fn icm_equity(stacks: &[u32], payouts: &[f64]) -> Vec<f64> {
      // Malmuth-Harville model:
      // P(player i finishes kth) = stack_i / total * P(remaining finish k+1..n)
      // Equity = sum over all finish positions of P(finish k) * payout[k]
  }
  ```

  **Key difference from cash game**: The solver's utility function is
  `icm_equity(new_stacks, payouts) - icm_equity(old_stacks, payouts)`
  instead of `chips_won`. This makes the game tree non-zero-sum in chips
  but zero-sum in equity — CFR still converges.

  **Blind level progression**: Tournaments have increasing blinds. The
  solver trains at specific blind-to-stack ratios (effective stack depth).
  The abstraction pipeline indexes by blind level.

  **State additions**:
  ```rust
  pub struct TournamentGame {
      // Standard NLHE state plus:
      blind_level: u8,
      players_remaining: u16,
      total_players: u16,
      payouts: Vec<f64>,           // payout structure (top 15% typically)
      all_stacks: Vec<u32>,        // all remaining players' stacks
  }
  ```

  **GameParams**:
  ```rust
  GameParams::NlheTournament {
      total_players: u16,
      starting_stack: u32,
      blind_schedule: Vec<(u32, u32, u32)>,  // (sb, bb, ante) per level
      payouts: Vec<f64>,
  }
  ```

- Required tests:
  - `tournament::tests::icm_equity_sums_to_total_prize_pool`
  - `tournament::tests::icm_chip_leader_has_less_than_proportional_equity`
  - `tournament::tests::utility_is_equity_delta_not_chips`
  - `tournament::tests::blind_increase_changes_strategy`
  - `tournament::tests::bubble_factor_increases_near_payout_threshold`
- Pass/fail:
  - ICM equity values sum to total prize pool (within f64 precision)
  - Player with 60% of chips has <60% of equity (ICM pressure)
  - Solver utility = ICM equity change, not chip change
  - Strategy at 20bb effective differs from strategy at 50bb effective

### AC-PV-05: Shared Poker Abstraction Extensions

- Where: future shared abstraction surface, with current adjacent artifact
  reality in `crates/myosu-games-poker/src/artifacts.rs`
- How: Extend the abstraction pipeline (AP-01..03) for variant-specific needs:

  **PLO hand clustering**: C(52,4) = 270,725 starting hands. Use Earth
  Mover's Distance (EMD) between equity distributions against random
  boards. Cluster into 200-2000 buckets (configurable).

  **Short Deck reclustering**: Same algorithm as NLHE but with 36-card
  deck. Rerun clustering with modified card set.

  **Tournament stack abstraction**: In addition to card abstraction, ICM
  adds a stack dimension. Discretize effective stacks into bins (5bb,
  10bb, 15bb, 20bb, 30bb, 50bb, 100bb+). Each bin has independent card
  abstraction.

  **6-max position bucketing**: Each position trains independently, so
  abstraction is per-position. Preflop ranges differ dramatically
  (UTG opens ~15% vs BTN opens ~45%).

  All abstraction outputs use the same file-based format from AP-01
  (no database dependency). Each variant produces its own abstraction
  artifact directory.

- Required tests:
  - `abstraction::tests::plo_clustering_270k_hands`
  - `abstraction::tests::shortdeck_clustering_36card`
  - `abstraction::tests::tournament_stack_bins`
- Pass/fail:
  - PLO clustering produces valid buckets for all 270,725 starting hands
  - Short Deck clustering uses only cards 6+
  - Tournament abstraction indexes by (stack_bin, card_bucket)

### AC-PV-06: Variant TUI Renderers

- Where: future game-specific renderer surfaces, with current adjacent poker
  reference in `crates/myosu-games-poker/src/renderer.rs`
- How: Implement `GameRenderer` for each variant per DESIGN.md mockups:

  | Variant | DESIGN.md section | Key rendering difference |
  |---------|-------------------|------------------------|
  | NLHE 6-max | 9.10 | 6 seat rows, position labels, side pots |
  | PLO | 9.11 | 4 hole cards shown (vs 2) |
  | Short Deck | 9.13 | "deck 36 cards (6+)" field, same layout otherwise |
  | Tournament | 9.12 | ICM panel, blind level, players remaining |

  Each renderer implements:
  ```rust
  impl GameRenderer for Nlhe6maxRenderer { ... }
  impl GameRenderer for PloRenderer { ... }
  impl GameRenderer for ShortDeckRenderer { ... }
  impl GameRenderer for TournamentRenderer { ... }
  ```

  Pipe output formats follow DESIGN.md section 12 (agent protocol).

- Required tests:
  - `tui::tests::sixmax_renders_6_seats`
  - `tui::tests::plo_renders_4_hole_cards`
  - `tui::tests::shortdeck_shows_deck_size`
  - `tui::tests::tournament_shows_icm`
- Pass/fail:
  - 6-max renderer shows all occupied seats with position labels
  - PLO renderer shows 4 hole cards for hero
  - Short Deck renderer includes "deck 36 cards (6+)" in state panel
  - Tournament renderer shows ICM equity, blind level, and players remaining

---

## Implementation order

1. **PV-03 Short Deck** — smallest delta from NLHE HU. Proves variant pattern.
2. **PV-01 NLHE 6-max** — highest market value. Position-indexed model is new.
3. **PV-02 PLO** — highest solver gap. Abstraction pipeline is the bottleneck.
4. **PV-04 Tournament/ICM** — most complex. ICM utility function is novel.
5. **PV-05 abstractions** — runs parallel with PV-01..04 as needed.
6. **PV-06 renderers** — after engines, before launch.

## Decision log

- 2026-03-17: Short Deck first because lowest risk — validates variant pattern
  with minimal new code. PLO/6-max have higher market value but higher risk.
- 2026-03-17: 6-max uses position-indexed 2-player model per 031626-13 decision.
  Does not use NPlayerGame trait.
- 2026-03-17: PLO abstraction uses EMD, same as academic literature (Waugh et al.).
- 2026-03-17: Tournament utility uses Malmuth-Harville ICM. Nash ICM
  (Tysen-Streib) is an optimization — baseline is sufficient for Stage 1.
