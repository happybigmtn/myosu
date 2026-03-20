# Implementation: games:multi-game Lane

## Summary

This document records the implementation of the `games:multi-game` lane, which adds Liar's Dice as a second CFR game engine to validate the robopoker trait system's generality, along with cross-game scoring infrastructure.

## Completed Slices

### Slice 1: Create myosu-games-liars-dice Crate Skeleton

**Files created:**
- `crates/myosu-games-liars-dice/Cargo.toml` - Crate manifest with dependencies on robopoker (rbp-core, rbp-mccfr, rbp-transport)
- `crates/myosu-games-liars-dice/src/lib.rs` - Module declarations
- `crates/myosu-games-liars-dice/src/turn.rs` - LiarsDiceTurn enum
- `crates/myosu-games-liars-dice/src/edge.rs` - LiarsDiceEdge enum
- `crates/myosu-games-liars-dice/src/info.rs` - BidHistory, LiarsDicePublic, LiarsDiceSecret, LiarsDiceInfo
- `crates/myosu-games-liars-dice/src/game.rs` - LiarsDiceGame state machine
- `crates/myosu-games-liars-dice/src/encoder.rs` - LiarsDiceEncoder
- `crates/myosu-games-liars-dice/src/profile.rs` - LiarsDiceProfile, LiarsDiceSolver

**Key decisions:**
- Added `rbp-transport` as direct dependency to access `Support` trait (required by `CfrEdge`)
- Used `Self::VariantName` syntax to avoid E0170 pattern binding errors

### Slice 2: Game Engine Implementation

**LiarsDiceGame** (`game.rs`):
- Fixed dice roll [1,1] at root for deterministic CFR traversal
- `acting: u8` tracks current player (0, 1) or terminal (2)
- `challenger: Option<u8>` stored before terminal to compute payoff
- `BidHistory` as fixed-size array [Option<(u8,u8)>; 12] for Copy constraint
- Zero-sum payoff: P0 payoff + P1 payoff = 0 at every terminal

**LiarsDiceTurn** (`turn.rs`):
- Variants: `Chance`, `Player0`, `Player1`, `Terminal`
- Implements `CfrTurn` with `chance()` and `terminal()` methods

**LiarsDiceEdge** (`edge.rs`):
- Variants: `Bid { quantity, face }`, `Challenge`
- Implements `CfrEdge` and `Support`

**BidHistory** (`info.rs`):
- Fixed 12-slot array with sentinel None values
- `Copy`, `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`

**LiarsDiceInfo** (`info.rs`):
- Public component: bid history
- Private component: player's die value
- Implements `CfrInfo` with `public()` and `secret()` methods

**Tests passing:**
- `game::tests::root_is_chance_node`
- `game::tests::legal_bids_increase`
- `game::tests::challenge_resolves_game`
- `game::tests::payoff_is_zero_sum`
- `game::tests::all_trait_bounds_satisfied`

**Bug fixed:** `choices()` in `LiarsDicePublic` iterated `q in (last_qty + 1)..=6`, which excluded same-quantity bids with higher faces. Fixed to `q in last_qty..=6`.

### Slice 3: Encoder and Profile/Solver

**LiarsDiceEncoder** (`encoder.rs`):
- Trivial encoding: info set is (die, bid_history)
- Implements `Encoder` trait

**LiarsDiceProfile** (`profile.rs`):
- HashMap-based storage for regrets, weights, evalues, counts
- Implements `Profile` trait

**LiarsDiceSolver** (`profile.rs`):
- Implements `Solver` with `LinearWeight`, `DiscountedRegret`, `VanillaSampling`
- `batch_size() = 1`, `tree_count() = 1024`

**Tests passing:**
- `profile::tests::profile_default`
- `profile::tests::train_short`

### Slice 4: ExploitMetric Registration

**Files modified:**
- `crates/myosu-games/src/traits.rs` - Added `ExploitMetric`, `ExploitScale`, `exploit_metric_for()`
- `crates/myosu-games/src/lib.rs` - Re-exported new types

**ExploitMetric** struct:
```rust
pub struct ExploitMetric {
    pub unit: &'static str,          // display unit
    pub scale: ExploitScale,         // Absolute, MilliPerHand, Normalized
    pub display_precision: u8,        // decimal places
    pub random_baseline: f64,         // random strategy exploitability
    pub good_threshold: f64,         // below this = "good"
}
```

**ExploitScale** enum:
- `Absolute` - Raw exploitability (Liar's Dice)
- `MilliPerHand` - milli-units per hand (poker)
- `Normalized` - [0, 1] where 0=Nash, 1=random

**Metrics registered:**
- NLHE HU: unit="mbb/h", scale=MilliPerHand, baseline=300.0, threshold=15.0
- NLHE 6-max: unit="mbb/h", scale=MilliPerHand, baseline=400.0, threshold=25.0
- Liar's Dice: unit="exploit", scale=Absolute, baseline=1.0, threshold=0.01

**Tests passing:**
- `traits::tests::all_game_types_have_metrics`
- `traits::tests::random_baseline_positive`
- `traits::tests::good_threshold_less_than_baseline`

## Blocked Slices

### Slice 5: SpectatorRelay in myosu-play

**Status:** BLOCKED

`myosu-play` crate does not exist. It is commented out in `Cargo.toml` workspace members as "Stage 5".

The SpectatorRelay requires:
- `crates/myosu-play/src/spectate.rs` - SpectatorRelay, session discovery, socket management
- JSON event emission to `~/.myosu/spectate/<session_id>.sock`
- Fog-of-war enforcement at relay

### Slice 6: Spectator TUI Screen

**Status:** BLOCKED (depends on Slice 5)

`crates/myosu-tui/src/screens/spectate.rs` does not exist. The `Screen::Spectate` variant exists in `screens.rs` but has no renderer.

Requires:
- Event stream rendering
- Fog-of-war view (hole cards as `·· ··` during play)
- Hole card reveal after showdown

## Verification

**Build commands:**
```bash
cargo build -p myosu-games-liars-dice  # PASS
cargo build -p myosu-games              # PASS
cargo build -p myosu-tui               # PASS
```

**Test commands:**
```bash
cargo test -p myosu-games-liars-dice  # 7 tests PASS
cargo test -p myosu-games              # 13 tests PASS
```

**Note:** The review.md references tests at `registry::tests::*` but implementation uses `traits::tests::*` per the spec. The review.md also references `solver::tests::*` tests that don't exist in the current implementation.

## Key Architectural Decisions

1. **Fixed-size BidHistory array** - Satisfies `CfrGame: Copy` constraint without heap allocation
2. **Direct rbp-transport dependency** - Required to access sealed `Support` trait
3. **Separate challenger tracking** - Terminal state stores who challenged for zero-sum payoff computation
4. **Per-game ExploitMetric** - Enables cross-game scoring normalization while preserving game-specific display units

## Dependencies

- robopoker at `04716310143094ab41ec7172e6cea5a2a66744ef`
  - rbp-core: Probability, Utility
  - rbp-mccfr: CfrGame, CfrEdge, CfrTurn, CfrInfo, CfrPublic, CfrSecret, Profile, Encoder, Solver
  - rbp-transport: Support (sealed trait)
