# `games:multi-game` Lane Spec

## Lane Boundary

`games:multi-game` is the **multi-game orchestration lane** for the myosu game-solving chain. It does not own a single crate — it owns the cross-cutting coordination surfaces that span multiple game engines:

- **`myosu-games-liars-dice`** (greenfield): The Liar's Dice proof-of-architecture — a second CFR game engine that validates the trait system's generality
- **Cross-game scoring integration**: The `ExploitMetric` registration path in `myosu-games` that makes game-specific exploitability displayable and validator-weights comparable
- **Spectator relay wiring**: The event emission surface in `myosu-play` and the spectator TUI screen in `myosu-tui` that let humans watch agent-vs-agent gameplay

`games:multi-game` does **not** own:

- `myosu-games` trait definitions (owned by `games:traits`)
- `myosu-games-poker` (owned by `games:poker-engine`)
- Any miner or validator binary (owned by `services:miner` and `services:validator-oracle`)
- The `games:variant-family` lane (NLHE 6-max, PLO, Short Deck — future)
- `docs/multi-game-expansion.md` (owned by the expansion guide AC-MG-04)

---

## Platform-Facing Purpose

The multi-game lane delivers the proof that myosu is **not a one-game platform**. The user-visible outcomes are:

- A second game engine (Liar's Dice) can be trained to Nash equilibrium (exploitability = 0), proving the CFR trait system works for non-poker games
- The lobby can display exploitability with correct per-game units (`mbb/h` for poker, exploitability index for Liar's Dice) because each game registers its metric descriptor
- A spectator can connect to any active session and watch agents play with fog-of-war (no hole cards revealed during play)

---

## How Surfaces Fit Together

```
┌─────────────────────────────────────────────────────────────┐
│                  games:traits (myosu-games)                 │
│  CfrGame, CfrEdge, CfrTurn, CfrInfo, Profile, Encoder       │
│  GameType, GameConfig, StrategyQuery, StrategyResponse      │
│  ExploitMetric (per game) — added by multi-game lane      │
└──────────┬──────────────────────────┬──────────────────────┘
           │                          │
           ▼                          ▼
┌──────────────────────┐    ┌─────────────────────────────────┐
│ myosu-games-poker   │    │     myosu-games-liars-dice      │
│ (games:poker-engine)│    │       (games:multi-game)        │
│                     │    │  AC-MG-01: CfrGame impl         │
│  PE-01..04          │    │  AC-MG-02: Solver + Nash proof  │
└──────────────────────┘    │  AC-MG-03: Zero-change verify  │
                            └──────────────┬──────────────────┘
                                           │
                                           ▼
                              ┌─────────────────────────┐
                              │  Cross-Game Scoring     │
                              │  (AC-CS-01: ExploitMetric│
                              │   registration in registry)
                              └─────────────────────────┘

┌─────────────────────────┐    ┌─────────────────────────────────┐
│  myosu-play (spectate) │    │       myosu-tui (spectate)       │
│  AC-SP-01: relay sock  │    │  AC-SP-02: spectator TUI screen  │
└─────────────────────────┘    └─────────────────────────────────┘
```

**Liar's Dice game** (`src/game.rs`): `LiarsDiceGame`, `LiarsDiceEdge`, `LiarsDiceTurn`, `LiarsDiceInfo`, `LiarsDiceProfile`, `LiarsDiceEncoder`. State space: 36 die outcomes × 4,095 bid sequences ≈ 147,420 terminal states, ~24,576 information sets. Converges in seconds to exact Nash.

**Liar's Dice solver** (`src/profile.rs`, `src/solver.rs`): `LiarsDiceProfile` implementing `Profile`, training to exploitability < 0.001, serializing via `Encoder`.

**Cross-game scoring** (`myosu-games/src/traits.rs` extension): Each `GameType` variant returns an `ExploitMetric` with `unit`, `scale`, `display_precision`, `random_baseline`, `good_threshold`. The validator oracle normalizes to `u16` weights using the metric.

**Spectator relay** (`myosu-play/src/spectate.rs`): `SpectatorRelay` emitting JSON event lines to `~/.myosu/spectate/<session_id>.sock`. Fog-of-war enforced at relay (hole cards never sent during play).

**Spectator TUI** (`myosu-tui/src/screens/spectate.rs`): `Screen::Spectate` rendering the event stream. Hole cards shown as `·· ··` during play, revealed after showdown.

---

## Currently Trusted Inputs

| File | Trust Signal |
|------|-------------|
| `crates/myosu-games/Cargo.toml` | Git dep on robopoker at `04716310143094ab41ec7172e6cea5a2a66744ef`; workspace member; `cargo test -p myosu-games` passes |
| `crates/myosu-games/src/traits.rs` | All tests pass; `GameType` already has `LiarsDice` variant with `from_bytes`/`to_bytes`; `GameParams` already has `LiarsDice` variant |
| `crates/myosu-games/src/lib.rs` | Re-exports all CFR traits from `rbp_mccfr`; re-exports `Probability`, `Utility` from `rbp_core` |
| `specsarchive/031626-06-multi-game-architecture.md` | AC-MG-01..04; Liar's Dice 1-die-each variant; zero-change architectural claim |
| `specsarchive/031626-16-cross-game-scoring.md` | AC-CS-01..03; `ExploitMetric` descriptor design; validator weight normalization formula |
| `specsarchive/031626-17-spectator-protocol.md` | AC-SP-01..03; local relay for Phase 0; fog-of-war design; JSON event format reusing AX-01 schema |
| `outputs/games/poker-engine/spec.md` | Reference pattern for lane spec structure; same `games:*` namespace conventions |

---

## Current Broken / Missing Surfaces

### Critical: `myosu-games-liars-dice` Crate Does Not Exist

`crates/myosu-games-liars-dice/` is not present in the workspace. All source files (`game.rs`, `edge.rs`, `turn.rs`, `info.rs`, `encoder.rs`, `profile.rs`) are unimplemented.

**Impact**: The Liar's Dice proof-of-architecture cannot exist without this crate. This is the primary blocker for AC-MG-01.

### Missing: `myosu-games-liars-dice` Not in Workspace Members

`Cargo.toml` workspace members only list `crates/myosu-games` and `crates/myosu-tui`. The new crate must be added.

### Missing: `ExploitMetric` Registration in `myosu-games`

The cross-game scoring spec (AC-CS-01) calls for `ExploitMetric` on each `GameType` variant. `crates/myosu-games/src/traits.rs` has no `ExploitMetric` type. The `GameType` enum has the `LiarsDice` variant but no associated metric.

**Impact**: Cross-game scoring cannot work for Liar's Dice until metric registration exists.

### Missing: `SpectatorRelay` in `myosu-play`

`crates/myosu-play/src/spectate.rs` does not exist. The spectator relay per AC-SP-01 is unimplemented.

**Impact**: No local spectator sessions can be observed.

### Missing: Spectator TUI Screen

`crates/myosu-tui/src/screens/spectate.rs` does not exist. The spectator screen per AC-SP-02 is unimplemented.

**Impact**: The TUI has no spectator view.

---

## Code Boundaries and Deliverables

### Crate Structure (Liar's Dice)

```
crates/myosu-games-liars-dice/
├── Cargo.toml              # Workspace member; depends on rbp-mccfr (git), myosu-games
└── src/
    ├── lib.rs              # Re-exports public API
    ├── game.rs             # LiarsDiceGame: CfrGame impl (root, actions, terminal payoff)
    ├── edge.rs             # LiarsDiceEdge: CfrEdge impl (Bid(quantity, face) | Challenge)
    ├── turn.rs             # LiarsDiceTurn: CfrTurn impl (Player(0), Player(1), Chance, Terminal)
    ├── info.rs             # LiarsDiceInfo: CfrInfo impl ((my_die, bid_history))
    ├── encoder.rs          # LiarsDiceEncoder: Encoder impl (trivial — direct enumeration)
    └── profile.rs          # LiarsDiceProfile: Profile impl + solver harness + Nash verification
```

### Spectator Relay Structure

```
crates/myosu-play/src/spectate.rs   # SpectatorRelay, session discovery, socket management
crates/myosu-tui/src/screens/spectate.rs  # Screen::Spectate variant and renderer
```

### ExploitMetric Extension Structure

```
crates/myosu-games/src/traits.rs    # Add ExploitMetric, ExploitScale, impl on GameType
crates/myosu-games/src/lib.rs        # Re-export ExploitMetric
```

### Public API Surface

```rust
// myosu-games-liars-dice lib.rs
pub use game::LiarsDiceGame;
pub use edge::LiarsDiceEdge;
pub use turn::LiarsDiceTurn;
pub use info::LiarsDiceInfo;
pub use encoder::LiarsDiceEncoder;
pub use profile::LiarsDiceProfile;

// myosu-games extensions
pub use myosu_games::ExploitMetric;
pub fn exploit_metric_for(game_type: &GameType) -> Option<&ExploitMetric>;
```

---

## Proof / Check Shape

### Bootstrap Proof (lane integrity check)

```bash
# After myosu-games-liars-dice exists:
cargo build -p myosu-games-liars-dice
cargo test -p myosu-games-liars-dice

# After ExploitMetric added to myosu-games:
cargo build -p myosu-games
cargo test -p myosu-games

# After spectator relay:
cargo build -p myosu-play
cargo test -p myosu-play

# After spectator TUI:
cargo build -p myosu-tui
cargo test -p myosu-tui
```

### Milestone Checks

| Milestone | Validates | AC |
|-----------|-----------|-----|
| Liar's Dice game runs from dice roll to challenge | Game engine | MG-01 |
| Solver trains to exploitability < 0.001 | Nash convergence | MG-02 |
| All existing tests pass without changes to existing crates | Architecture | MG-03 |
| Expansion guide covers 6 candidate games | Documentation | MG-04 |
| `ExploitMetric` registered for all `GameType` variants | Cross-game scoring | CS-01 |
| Validator normalizes using `ExploitMetric` | Cross-game scoring | CS-02 |
| Lobby displays exploitability with correct units | Cross-game scoring | CS-03 |
| Spectator relay emits JSON events | Spectator | SP-01 |
| Spectator TUI renders fog-of-war view | Spectator | SP-02 |
| Spectator session discovery lists active sessions | Spectator | SP-03 |

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1 — Create `myosu-games-liars-dice` Crate Skeleton

**Files**: `crates/myosu-games-liars-dice/Cargo.toml`, `crates/myosu-games-liars-dice/src/lib.rs`

Add crate to workspace members. Cargo.toml:
- Dependency on `rbp-mccfr` (git) at same rev as `myosu-games` (`04716310143094ab41ec7172e6cea5a2a66744ef`)
- Dependency on `myosu-games`
- `crate-type = ["lib"]`

`lib.rs` re-exports stub types initially.

**Proof**: `cargo build -p myosu-games-liars-dice` exits 0 with empty lib.

---

### Slice 2 — `game.rs` + `edge.rs` + `turn.rs` + `info.rs`: Liar's Dice Game Engine

**Files**: `crates/myosu-games-liars-dice/src/{game,edge,turn,info}.rs`

Implement 2-player Liar's Dice with 1 die each (6 faces):

- `LiarsDiceGame: CfrGame` — state: `dice: [u8; 2]`, `bids: Vec<Bid>`, `current_player: u8`
- `LiarsDiceEdge: CfrEdge` — `Bid(quantity: u8, face: u8)` or `Challenge`
- `LiarsDiceTurn: CfrTurn` — `Player(0)`, `Player(1)`, `Chance`, `Terminal`
- `LiarsDiceInfo: CfrInfo` — `(my_die: u8, bid_history: Vec<Bid>)` — player cannot see opponent's die

**Key constraint**: `CfrGame: Copy` is required. Use fixed-size array for bid history (max 12 bids for 1-die game) with a "sealed" sentinel to distinguish empty slots. This avoids heap allocation and satisfies `Copy`.

**Proof**:
```bash
cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node
cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase
cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game
cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum
cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied
```

---

### Slice 3 — `encoder.rs` + `profile.rs`: Solver and Nash Verification

**Files**: `crates/myosu-games-liars-dice/src/{encoder,profile}.rs`

Implement `Encoder` for Liar's Dice (trivial — info set = (my die, bid history), direct enumeration) and `Profile` impl. Train to convergence and verify exploitability < 0.001.

**Proof**:
```bash
cargo test -p myosu-games-liars-dice solver::tests::train_to_nash
cargo test -p myosu-games-liars-dice solver::tests::exploitability_near_zero
cargo test -p myosu-games-liars-dice solver::tests::strategy_is_nontrivial
cargo test -p myosu-games-liars-dice solver::tests::wire_serialization_works
```

---

### Slice 4 — `ExploitMetric` Registration (AC-CS-01)

**File**: `crates/myosu-games/src/traits.rs`

Add `ExploitMetric` struct and `ExploitScale` enum per `031626-16`. Implement `exploit_metric_for(&GameType) -> Option<&ExploitMetric>` for all variants.

For Liar's Dice: unit = "exploit", scale = `ExploitScale::Absolute`, random_baseline = 1.0, good_threshold = 0.01, display_precision = 2.

**Proof**:
```bash
cargo test -p myosu-games registry::tests::all_game_types_have_metrics
cargo test -p myosu-games registry::tests::random_baseline_positive
cargo test -p myosu-games registry::tests::good_threshold_less_than_baseline
```

---

### Slice 5 — Spectator Relay (AC-SP-01)

**File**: `crates/myosu-play/src/spectate.rs`

Implement `SpectatorRelay` emitting JSON event lines to `~/.myosu/spectate/<session_id>.sock`. Fog-of-war enforced at relay: hole cards never sent during play. Session discovery via socket file scan.

**Proof**:
```bash
cargo test -p myosu-play spectate::tests::relay_emits_events
cargo test -p myosu-play spectate::tests::relay_handles_disconnected_listener
cargo test -p myosu-play spectate::tests::events_are_valid_json
cargo test -p myosu-play spectate::tests::discover_local_sessions
```

---

### Slice 6 — Spectator TUI Screen (AC-SP-02)

**File**: `crates/myosu-tui/src/screens/spectate.rs`

Implement `Screen::Spectate` rendering spectator view with fog-of-war. Key bindings: `n` (next session), `r` (reveal after showdown), `q` (quit).

**Proof**:
```bash
cargo test -p myosu-tui spectate::tests::renders_fog_of_war
cargo test -p myosu-tui spectate::tests::reveal_shows_hole_cards_after_showdown
cargo test -p myosu-tui spectate::tests::reveal_blocked_during_play
```

---

### Slice 7 — Zero-Change Verification (AC-MG-03)

**File**: `crates/myosu-games-liars-dice/tests/zero_change.rs`

Integration test proving no existing crate was modified.

**Proof**:
```bash
cargo test -p myosu-games
cargo test -p myosu-games-poker
cargo test -p myosu-games-liars-dice
git diff crates/myosu-games/src/ crates/myosu-games-poker/src/  # must be empty
```

---

## Dependency Order

```
games:traits ──(stable)──► games:multi-game slices 1-7
     │
     └── games:poker-engine already bootstrapped independently ✓

Slice 1 (crate skeleton)
Slice 2 (game engine) → must precede slice 3
Slice 3 (solver + Nash) → must precede slice 7
Slice 4 (ExploitMetric) → independent of slices 2-3
Slice 5 (spectator relay) → independent
Slice 6 (spectator TUI) → depends on slice 5
Slice 7 (zero-change verify) → depends on slices 2-3
```

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | `games:multi-game` extends `games:traits` with `ExploitMetric` registration and `LiarsDice` in `GameType`/`GameParams`. Relationship is additive, not breaking. |
| `games:poker-engine` | Both consume `myosu-games` traits. Poker-engine is the first concrete game; multi-game is the proof that the architecture generalizes. No direct dependency. |
| `services:miner` | Will eventually serve Liar's Dice strategy queries. Not needed for the proof-of-architecture. |
| `services:validator-oracle` | Will use `ExploitMetric` for cross-game weight normalization (AC-CS-02). The metric registration in slice 4 is prerequisite. |
| `product:play-tui` | Spectator TUI is part of play-tui. Slice 5 and 6 integrate with the play TUI. |
