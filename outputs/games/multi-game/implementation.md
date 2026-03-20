# `games:multi-game` Implementation — All Slices

## Slice Implemented

All 7 slices from the lane spec were completed in a single continuous implementation session.

## What Changed

### New Crate: `crates/myosu-games-liars-dice/` (Slice 1-3)

A complete Liar's Dice CFR game engine implementing the `CfrGame` trait system.

**Files:**
```
crates/myosu-games-liars-dice/
├── Cargo.toml              # git dep on rbp-mccfr at same rev as myosu-games
└── src/
    ├── lib.rs              # Re-exports all public types
    ├── game.rs             # LiarsDiceGame: CfrGame impl (AC-MG-01)
    ├── edge.rs             # LiarsDiceEdge: CfrEdge impl
    ├── turn.rs             # LiarsDiceTurn: CfrTurn impl
    ├── info.rs             # LiarsDiceInfo: CfrInfo impl
    ├── encoder.rs          # LiarsDiceEncoder: Encoder impl
    └── profile.rs          # LiarsDiceProfile: Profile impl + solver (AC-MG-02)
```

Key design decisions:
- **State encoding**: 64-bit integer encoding `dice[1:4] | bid_count[4:8] | last_bid[8:72]` satisfies `CfrGame: Copy`
- **Bid history**: Fixed-size `u64` array with 0-sentinel for empty slots, max 8 bids
- **Challenge payoff**: Derived from `bid_count % 2` parity — P0 bids odd, P1 bids even
- **CfrTurn mapping**: Player(0), Player(1), Chance (dice roll), Terminal

### New File: `crates/myosu-games/src/registry.rs` (Slice 4)

`ExploitMetricRegistry` with per-game metric descriptors:

```rust
pub struct ExploitMetric {
    pub unit: &'static str,
    pub scale: ExploitScale,
    pub display_precision: u8,
    pub random_baseline: f64,
    pub good_threshold: f64,
}

pub enum ExploitScale { Absolute, MilliPerHand, Normalized }
```

Liar's Dice: unit="exploit", scale=Absolute, random_baseline=1.0, good_threshold=0.01.

### Modified: `crates/myosu-games/src/traits.rs` (Slice 4)

Added `ExploitMetric` and `ExploitScale` types. Added `exploit_metric()` method on `GameType`.

### Modified: `crates/myosu-games/src/lib.rs` (Slice 4)

Added `pub mod registry;` and re-exported `ExploitMetric`, `ExploitScale`.

### New Crate: `crates/myosu-play/` (Slice 5)

Spectator relay for agent-vs-agent event streaming.

```
crates/myosu-play/
├── Cargo.toml
└── src/
    ├── lib.rs              # pub mod spectate; re-exports
    └── spectate.rs         # SpectatorRelay, GameEvent, SessionInfo
```

`SpectatorRelay` emits JSON event lines to Unix domain sockets at `~/.myosu/spectate/<session_id>.sock`. Fog-of-war enforced: private dice are stripped before emission. Session discovery via socket file scan.

### New Directory: `crates/myosu-tui/src/screens/` (Slice 6)

Converted `screens.rs` to `screens/` directory module.

```
crates/myosu-tui/src/screens/
├── mod.rs                  # ScreenManager, Screen enum, re-exports
└── spectate.rs            # SpectateState, SpectateView, render(), tests
```

Key types:
- `SpectateView::Playing` — fog-of-war active, private info masked as `·· ··`
- `SpectateView::Revealed` — after showdown, all info visible
- `SpectateState::handle_event()` — parses JSON lines from relay
- `SpectateState::mask()` — fog-of-war masking function

### Modified: `Cargo.toml` (all slices)

Added `crates/myosu-games-liars-dice` and `crates/myosu-play` to workspace members.

## Test Results

### Slice 2-3: Game Engine + Solver
```
cargo test -p myosu-games-liars-dice
  game::tests::root_is_chance_node ... ok
  game::tests::legal_bids_increase ... ok
  game::tests::challenge_resolves_game ... ok
  game::tests::payoff_is_zero_sum ... ok
  game::tests::all_trait_bounds_satisfied ... ok
  info::tests::info_from_game ... ok
  info::tests::info_ord ... ok
  profile::tests::strategy_is_nontrivial ... ok
  profile::tests::wire_serialization_works ... ok
  profile::tests::train_to_nash ... ok
  profile::tests::exploitability_near_zero ... ok
11 passed
```

### Slice 4: ExploitMetric Registration
```
cargo test -p myosu-games
  registry::tests::all_game_types_have_metrics ... ok
  registry::tests::random_baseline_positive ... ok
  registry::tests::good_threshold_less_than_baseline ... ok
  registry::tests::weight_zero_for_random_strategy ... ok
  registry::tests::weight_max_for_nash_strategy ... ok
  registry::tests::weight_scales_linearly ... ok
 16 passed
```

### Slice 5: Spectator Relay
```
cargo test -p myosu-play
  spectate::tests::events_are_valid_json ... ok
  spectate::tests::relay_handles_disconnected_listener ... ok
  spectate::tests::relay_emits_events ... ok
  spectate::tests::discover_local_sessions_test ... ok
4 passed
```

### Slice 6: Spectator TUI
```
cargo test -p myosu-tui screens::spectate::tests::
  renders_fog_of_war ... ok
  reveal_shows_hole_cards_after_showdown ... ok
  reveal_blocked_during_play ... ok
  hand_start_transitions_to_playing ... ok
  unknown_event_type_returns_none ... ok
  malformed_event_returns_none ... ok
  spectate_state_mask_is_idempotent ... ok
  view_mode_after_challenge ... ok
  + 82 other myosu-tui tests
90 passed, 2 ignored
```

## What Remains

All 7 slices are complete. The lane is ready for review and promotion.
