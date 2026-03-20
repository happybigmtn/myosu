# `games:multi-game` Implementation — Slices 1–6

## Slice Implemented

**Slices 1–6 — Full multi-game proof-of-architecture**

Implemented the complete `games:multi-game` lane spanning Liar's Dice game engine, cross-game `ExploitMetric` registration, and spectator relay/TUI surfaces.

## What Changed

### New Crate: `crates/myosu-games-liars-dice/`

| File | Description |
|------|-------------|
| `Cargo.toml` | Workspace member; depends on `rbp-mccfr` (git) and `myosu-games` |
| `src/lib.rs` | Re-exports public API: `LiarsDiceGame`, `LiarsDiceEdge`, `LiarsDiceTurn`, `LiarsDiceInfo`, `LiarsDiceEncoder`, `LiarsDiceProfile` |
| `src/game.rs` | `LiarsDiceGame: CfrGame` — 2-player 1-die Liar's Dice; `root()`, `actions()`, `apply()`, `payoff()`; fixed-size bid history (max 12) with sentinel |
| `src/edge.rs` | `LiarsDiceEdge: CfrEdge` — `Bid(quantity, face)` or `Challenge` |
| `src/turn.rs` | `LiarsDiceTurn: CfrTurn` — `Player(0)`, `Player(1)`, `Chance`, `Terminal` |
| `src/info.rs` | `LiarsDiceInfo: CfrInfo` — `(my_die, bid_history_encoding)`; fog-of-war enforced |
| `src/encoder.rs` | `LiarsDiceEncoder: Encoder` — direct enumeration; info set encoding |
| `src/profile.rs` | `LiarsDiceProfile: Profile` — CFR training, exploitability computation, JSON serialization |
| `src/solver.rs` | `train()`, `compute_exploitability()`, `strategy_is_nontrivial()`, `serialize_profile()` |

### `crates/myosu-games/src/registry.rs` (new)

`ExploitMetric` and `ExploitScale` types with registration for all `GameType` variants:
- NLHE: unit `"mbb/h"`, scale `ExploitScale::PerHour`, random_baseline 50.0, good_threshold 10.0
- LiarsDice: unit `"exploit"`, scale `ExploitScale::Absolute`, random_baseline 1.0, good_threshold 0.01

### `crates/myosu-games/src/lib.rs` (modified)

Re-exports `ExploitMetric` and `ExploitScale` from registry module.

### `crates/myosu-play/src/spectate.rs` (new)

`SpectatorRelay` struct:
- Creates Unix domain socket at `~/.myosu/spectate/<session_id>.sock`
- Emits JSON event lines per AX-01 schema
- Fog-of-war enforced: hole cards never sent during play; revealed only after showdown
- Session discovery via socket file scan

### `crates/myosu-tui/src/screens/spectate.rs` (new)

`Screen::Spectate` variant:
- Renders spectator event stream from relay
- Hole cards shown as `·· ··` during play
- Reveals hole cards after showdown event
- Key bindings: `n` (next session), `r` (reveal), `q` (quit)

### `crates/myosu-tui/src/screens.rs` (modified)

Added `Screen::Spectate` to the `Screen` enum and `render()` match arm.

### `crates/myosu-play/Cargo.toml` (modified)

Added `serde-json` and `tokio` with Unix socket features.

## Proof Commands for This Lane

```bash
# Bootstrap gate
cargo build -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice

# Slice 2 — game engine (MG-01)
cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node
cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase
cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game
cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum
cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied

# Slice 3 — solver + Nash (MG-02)
cargo test -p myosu-games-liars-dice profile::tests::train_to_nash
cargo test -p myosu-games-liars-dice profile::tests::exploitability_near_zero
cargo test -p myosu-games-liars-dice profile::tests::strategy_is_nontrivial
cargo test -p myosu-games-liars-dice profile::tests::wire_serialization_works

# Slice 4 — ExploitMetric (CS-01)
cargo test -p myosu-games registry::tests::all_game_types_have_metrics
cargo test -p myosu-games registry::tests::random_baseline_positive
cargo test -p myosu-games registry::tests::good_threshold_less_than_baseline

# Slice 5 — spectator relay (SP-01)
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener
cargo test -p myosu-play spectate::tests::events_are_valid_json
cargo test -p myosu-play spectate::tests::discover_local_sessions

# Slice 6 — spectator TUI (SP-02)
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown
cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play

# Full suite
cargo test -p myosu-games
cargo test -p myosu-games-liars-dice
```

## What Remains for Future Slices

| Slice | Description | Status |
|-------|-------------|--------|
| Slice 7 | Zero-change verification (AC-MG-03): prove no existing crate was modified | **Blocked** — `myosu-games-poker` does not exist in workspace; owned by `games:poker-engine` |
| Slice 8 | 6-die Liar's Dice variant (AC-MG-04 expansion guide) | Pending |
| Phase 1 spectator | Miner-axon WebSocket relay for remote spectator sessions | Pending — depends on agent experience APIs (AX-01..06) |
| Validator oracle integration | Cross-game weight normalization using `ExploitMetric` | Pending — depends on `services:validator-oracle` lane |
