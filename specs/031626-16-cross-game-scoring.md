# Specification: Cross-Game Scoring — Exploitability Metrics Normalization

Source: DESIGN.md 9.23 lobby, VO-01..07 validator oracle
Status: Draft
Date: 2026-03-30
Depends-on: GT-01..05 (traits), VO-01..07 (validator), 031626-13 (n-player design)
Blocks: Lobby display correctness for multi-game subnets

## Purpose

DESIGN.md 9.23 (lobby) shows an `exploit` column displaying solver quality
across games: `13.2 mbb/h` for poker, `4.1 mcpw` for backgammon. Each game
has a "natural unit" for exploitability, and these units are not directly
comparable. A miner with 13.2 mbb/h in poker is not "better" or "worse"
than a miner with 0.02 in liar's dice.

This spec defines:
1. How each game reports its exploitability metric (units, scale)
2. How the lobby displays these metrics correctly
3. How validators use game-specific metrics for weight calculation

## Current Truth

- the current validator implementation is still stage-0 narrow and lives under
  `crates/myosu-validator/src/validation.rs`
- the current shared game metadata surface lives under
  `crates/myosu-games/src/registry.rs`
- the repo does **not** yet ship a dedicated lobby screen module under
  `crates/myosu-tui/src/screens/`; the current user-facing shell still lives in
  `myosu-play`
- this spec is therefore a future normalization layer on top of live
  single-game stage-0 scoring, not an already-started implementation

## The problem

Exploitability measures how much a perfect adversary can extract from a
strategy per game. The raw number depends on the game's utility scale:

| Game | Utility scale | Natural unit | Good value | Random value |
|------|--------------|-------------|------------|--------------|
| NLHE HU | big blinds | mbb/h (milli-BB per hand) | <15 | ~300 |
| NLHE 6-max | big blinds | mbb/h (per position) | <25 | ~400 |
| PLO | big blinds | mbb/h | <30 | ~500 |
| Short Deck | big blinds | mbb/h | <20 | ~350 |
| Tournament | ICM equity | mICM/h (milli-ICM per hand) | <10 | ~200 |
| Liar's Dice | game outcome (±1) | exploit (0-2 scale) | <0.01 | ~1.0 |
| Backgammon | points/game | mcpw (milli-cubeless PPW) | <5 | ~500 |
| Teen Patti | stakes | mbb/h (same as poker) | <20 | ~300 |
| Riichi Mahjong | points | mpts/h (milli-points per hand) | TBD | TBD |
| Bridge | IMPs | mIMP/b (milli-IMP per board) | TBD | TBD |
| Gin Rummy | deadwood points | mdw/h (milli-deadwood per hand) | TBD | TBD |
| Hanafuda / Hwatu | game points | mpts/r (milli-points per round) | TBD | TBD |
| Dou Di Zhu | game outcome | exploit (team-adjusted) | TBD | TBD |
| Spades / Call Break | tricks | mtricks/h (milli-tricks per hand) | TBD | TBD |
| Pusoy Dos / Tien Len | game outcome | exploit (0-1 scale) | TBD | TBD |
| Stratego | game outcome (±1) | exploit (0-2 scale) | TBD | TBD |
| OFC | royalty points | mrp/h (milli-royalty per hand) | TBD | TBD |

## Design

### Per-game metric registration

Each `GameType` registers a metric descriptor in the `GameRegistry`:

```rust
pub struct ExploitMetric {
    pub unit: &'static str,          // display unit ("mbb/h", "mcpw", etc.)
    pub scale: ExploitScale,         // how to interpret the raw number
    pub display_precision: u8,       // decimal places in lobby
    pub random_baseline: f64,        // exploitability of uniform random strategy
    pub good_threshold: f64,         // below this = "good" solver
}

pub enum ExploitScale {
    /// Raw exploitability value. Lower is better. 0 = Nash.
    /// Used for small games (Liar's Dice, Stratego).
    Absolute,

    /// Milli-units per hand/round. Lower is better. 0 = Nash.
    /// Used for games with per-hand utility (poker, backgammon).
    MilliPerHand,

    /// Normalized to [0, 1] where 0 = Nash, 1 = random.
    /// Used when absolute scale varies too much across configs.
    Normalized,
}
```

### Validator weight calculation

The validator oracle computes raw exploitability and normalizes it for
weight submission:

```rust
fn compute_weight(exploit: f64, metric: &ExploitMetric) -> u16 {
    // Normalize: 0 = random strategy, MAX_WEIGHT = Nash
    let normalized = (1.0 - (exploit / metric.random_baseline).min(1.0)).clamp(0.0, 1.0);
    (normalized * u16::MAX as f64) as u16
}
```

Weights on-chain are always `u16` (0..65535) regardless of game. The
normalization ensures that Yuma Consensus treats all games equivalently:
a miner with weight 60000 in poker is comparable to weight 60000 in
mahjong, even though the raw exploitability numbers are different.

### Lobby display

DESIGN.md 9.23 shows the raw metric with unit suffix:

```
  id  game              miners  exploit     balance
  1   nlhe-hu           12      13.2 mbb/h  1000bb
  20  backgammon         3      4.1 mcpw    --
  15  liars-dice         2      0.02        --
```

The lobby formats each game's exploitability using its registered metric:

```rust
fn format_exploit(exploit: f64, metric: &ExploitMetric) -> String {
    if exploit < 1e-9 {
        return "solved".to_string();
    }
    format!("{:.prec$} {}", exploit, metric.unit, prec = metric.display_precision as usize)
}
```

Games with no active miners show `--`.

### N-player game metrics

Per 031626-13, n-player games use `max_exploitability` (maximum individual
exploitability across all players). This produces a single scalar that
fits the same framework:

```rust
// 2-player: standard exploitability (sum of best-response values)
let exploit_2p = profile.exploitability();

// n-player: max individual exploitability
let exploit_np = NPlayerExploitability::max_exploitability(&strategy, samples);
```

Both produce a non-negative f64 where 0 = equilibrium. The metric
descriptor handles display formatting.

## Scope

### AC-CS-01: Metric Registration in GameRegistry

- Where: `crates/myosu-games/src/registry.rs (extend)`
- How: Add `ExploitMetric` to each `GameType` variant. Register metrics
  for all 20 games (use TBD placeholders for games without empirical data).

- Required tests:
  - `registry::tests::all_game_types_have_metrics`
  - `registry::tests::random_baseline_positive`
  - `registry::tests::good_threshold_less_than_baseline`
- Pass/fail:
  - Every `GameType` variant returns a non-None `ExploitMetric`
  - `random_baseline` > 0 for all games
  - `good_threshold` < `random_baseline` for all games

### AC-CS-02: Normalized Weight Calculation in Validator

- Where: `crates/myosu-validator/src/validation.rs` plus
  `crates/myosu-games/src/registry.rs`
- How: Replace hardcoded poker normalization with metric-driven normalization.
  Validator fetches `ExploitMetric` from `GameRegistry` for its subnet's game
  type and uses it for weight calculation.

- Required tests:
  - `oracle::tests::weight_zero_for_random_strategy`
  - `oracle::tests::weight_max_for_nash_strategy`
  - `oracle::tests::weight_scales_linearly`
- Pass/fail:
  - Random strategy (exploit = baseline) → weight = 0
  - Nash strategy (exploit = 0) → weight = u16::MAX
  - Strategy with exploit = baseline/2 → weight ≈ u16::MAX/2

### AC-CS-03: Lobby Exploit Display

- Where: future lobby/UI surface, with current adjacent gameplay shell in
  `crates/myosu-play/src/main.rs` and shared metric metadata in
  `crates/myosu-games/src/registry.rs`
- How: Lobby screen fetches `ExploitMetric` per subnet and formats the
  display column accordingly. Right-align the exploit value, left-align
  the unit suffix.

- Required tests:
  - `lobby::tests::exploit_displays_with_correct_unit`
  - `lobby::tests::no_miners_shows_dash`
  - `lobby::tests::zero_exploit_shows_solved`
- Pass/fail:
  - NLHE subnet shows `13.2 mbb/h` (not raw float)
  - Subnet with no miners shows `--`
  - Subnet with exploit = 0 shows `solved`

## Decision log

- 2026-03-17: Use per-game metric descriptors rather than a universal
  normalized scale. Rationale: raw numbers with units are more meaningful to
  domain experts. A poker player understands "13.2 mbb/h." Normalizing to
  0-100 loses that domain meaning.
- 2026-03-17: Validator weights ARE normalized (u16). The normalization
  happens at the validator, not the display. This keeps Yuma Consensus
  game-agnostic while keeping the lobby display game-specific.
- 2026-03-17: TBD baselines for Stage 2-3 games. These will be filled in
  when each game engine is implemented and random strategy exploitability
  can be measured empirically.
