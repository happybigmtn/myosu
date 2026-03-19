# `play:tui` Lane Specification

## Purpose and User-Visible Outcome

`play:tui` is the **interactive NLHE poker gameplay lane** for Myosu. It delivers:

1. **`myosu-play` binary** — the consumer-facing CLI; `myosu-play --train` for local practice, `myosu-play --chain ws://...` for miner-connected play
2. **NLHE `GameRenderer` implementation** — draws the game-state panel (board, hole cards, stacks, pot, street, hand strength) and log (prose action history with right-aligned pot)
3. **Training mode** — heads-up NLHE practice with a blueprint (mmap MCCFR) or heuristic bot; no chain required
4. **Solver advisor** — displays the trained solver's action distribution for the hero's pending decision as the "EQUILIBRIUM" section
5. **`--pipe` mode** — plain-text state/output protocol for agent piping (`agent_a | myosu-play --pipe | agent_b`)

**User-visible behavior**: A player runs `myosu-play --train`, plays NLHE hands against a blueprint bot, and sees both the bot's actions and the solver's recommended action frequencies for each decision point.

---

## Lane Boundary

```
                         ┌─────────────────────────────────────────────────────────┐
                         │               play:tui (THIS LANE)                      │
                         │                                                          │
  upstream               │  ┌─────────────────┐  ┌─────────────────────────┐      │
  tui:shell ──────────► │  │ GameRenderer    │  │ TrainingTable          │      │
  (82 tests, trusted)    │  │ (NlheRenderer)  │  │ (game loop + bot       │      │
                         │  │                 │  │  dispatch)             │      │
                         │  └────────┬────────┘  └───────────┬─────────────┘      │
                         │           │                       │                     │
  upstream               │  ┌────────┴───────────────────────┴────────┐          │
  games:traits ─────────►│  │ CfrGame, Profile, GameConfig,          │          │
  (14 tests, trusted)    │  │ GameType, StrategyQuery/Response       │          │
                         │  └────────────────────────────────────────┘          │
                         │  ┌──────────────────┐  ┌──────────────────────┐       │
                         │  │ BlueprintBackend │  │ SolverAdvisor        │       │
                         │  │ (mmap artifact) │  │ (EQUILIBRIUM panel)  │       │
                         │  └──────────────────┘  └──────────────────────┘       │
                         │  ┌──────────────────────────────────────────────┐    │
                         │  │ myosu-play binary (main.rs)                   │    │
                         │  │ ├── --train  (local bot)                      │    │
                         │  │ ├── --chain   (miner axon)                    │    │
                         │  │ └── --pipe    (agent protocol)                │    │
                         │  └──────────────────────────────────────────────┘    │
                         │                                                          │
  untrusted              │  ┌──────────────────────────────────────────────┐    │
  miner (future) ───────►│  │ HTTP client — miner axon /strategy          │    │
                         │  └──────────────────────────────────────────────┘    │
                         └─────────────────────────────────────────────────────────┘
```

**Trusted upstream inputs:**
- `tui:shell` (82 tests pass) — shell, screens, input, events, theme, pipe mode, `GameRenderer` trait
- `games:traits` (14 tests pass) — `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response`

**Untrusted inputs** (validated at use site):
- Blueprint artifacts from disk (SHA-256 hash verified before mmap)
- Miner HTTP responses (serde-validated before use)

**Trusted downstream outputs:** None — `play:tui` is a terminal lane

---

## Current Implementation Status

| Surface | Status | Evidence |
|---------|--------|---------|
| `crates/myosu-tui/` shell | **TRUSTED** | 82 tests pass (`cargo test -p myosu-tui`) |
| `crates/myosu-games/` traits | **TRUSTED** | 14 tests pass (`cargo test -p myosu-games`) |
| `crates/myosu-play/` binary | **MISSING** | No crate at this path |
| `crates/myosu-games-poker/` NLHE renderer | **MISSING** | No crate at this path |
| `TrainingTable` | **MISSING** | No `training.rs` |
| `BlueprintBackend` | **MISSING** | No `blueprint.rs` |
| `SolverAdvisor` | **MISSING** | No `advisor.rs` |
| Chain discovery + miner client | **MISSING** | Future slice (depends on `chain:runtime`) |

---

## Broken or Missing Surfaces

### 1. `myosu-play` Binary — Completely Absent

`crates/myosu-play/` does not exist. The spec at `specsarchive/031626-05-gameplay-cli.md` defines 6 modules (`main.rs`, `chain.rs`, `discovery.rs`, `game_loop.rs`, `bot.rs`, `recorder.rs`); none are implemented.

**Impact**: No consumer-facing product exists.
**Required**: Bootstrap the binary with `--train` mode first; wire `NlheRenderer` into the `tui:shell`; prove the render loop.

### 2. NLHE `GameRenderer` — No Concrete Implementation

`crates/myosu-games-poker/` does not exist. The `GameRenderer` trait (in `myosu-tui`) is the only per-game customization point, but no `NlheRenderer` proves the architecture works end-to-end.

Spec TU-08 (from `specsarchive/031626-07-tui-implementation.md`) defines the expected output:
- **State panel**: `board`, `you`, `solver`, `pot`, `EQUILIBRIUM` (optional)
- **Declaration**: `THE SYSTEM AWAITS YOUR DECISION` / `BOT THINKING` / `SHOWDOWN`
- **Log**: prose format (`solver raises to 6bb · pot 9`)
- **Solver advisor**: action distribution as `raise 53% · call 35% · fold 12%`

**Impact**: The `GameRenderer` contract is unproven for actual poker.
**Required**: `NlheRenderer` with hardcoded mock states first, then wired to robopoker's `Game`.

### 3. Training Mode — `TrainingTable` Absent

`crates/myosu-play/src/training.rs` does not exist. Training mode is the Phase 0 product — local play without chain/miner, proving the game loop before any network dependency.

Spec TU-09 defines:
- `TrainingTable` wrapping robopoker's `Game`
- `BotBackend` trait: `BlueprintBackend` → `HeuristicBackend` fallback
- `/deal`, `/board`, `/stack`, `/showdown` training commands
- 200–500ms bot thinking delay

**Impact**: No way to play locally.
**Required**: `TrainingTable` + `BotBackend` + practice chip tracking.

### 4. Blueprint Loading — `BlueprintBackend` Absent

`crates/myosu-play/src/blueprint.rs` does not exist. Blueprint artifacts enable the trained-solver opponent experience.

Spec TU-10 defines:
- Artifact discovery (`MYOSU_BLUEPRINT_DIR`, default `~/.myosu/blueprints/`)
- Manifest validation (schema v1, SHA-256 hashes)
- Memory-mapped strategy lookup (< 1μs per query)
- Error messages: `~ bot strategy: heuristic · blueprint not found`

**Impact**: Only heuristic fallback bot available without blueprint artifacts.
**Required**: `BlueprintBackend` with mmap lookup + graceful fallback.

### 5. Solver Advisor — `SolverAdvisor` Absent

`crates/myosu-play/src/advisor.rs` does not exist. The advisor is the key differentiating feature: players see what the trained solver recommends for their exact spot.

Spec TU-11 defines:
- `SolverAdvisor` trait: `advise(recall, hero_seat) → Option<Vec<(Action, f64)>>`
- EQUILIBRIUM section: `raise 53% · call 35% · fold 12%`
- Toggle: `/advisor on/off`
- Default: ON in training mode

**Impact**: Players cannot learn from the solver during hands.
**Required**: `SolverAdvisor` backed by `BlueprintBackend::action_distribution()`.

### 6. Chain Discovery + Miner Client — Future Slice

`crates/myosu-play/src/discovery.rs` and `bot.rs` (chain variant) are future work — blocked on `chain:runtime` being available.

---

## Code Boundaries and Deliverables

| File | Responsibility |
|------|---------------|
| `crates/myosu-play/src/main.rs` | CLI entry, mode dispatch (`--train`/`--chain`/`--pipe`) |
| `crates/myosu-play/src/training.rs` | `TrainingTable`, `BotBackend` trait, practice chips |
| `crates/myosu-play/src/blueprint.rs` | `BlueprintBackend`, artifact discovery, mmap lookup |
| `crates/myosu-play/src/advisor.rs` | `SolverAdvisor`, EQUILIBRIUM formatting |
| `crates/myosu-play/src/recorder.rs` | Hand history JSON, session stats |
| `crates/myosu-games-poker/src/lib.rs` | Crate entry |
| `crates/myosu-games-poker/src/renderer.rs` | `NlheRenderer` (GameRenderer impl) |
| `crates/myosu-games-poker/src/truth_stream.rs` | Prose log formatting, pot tracking |

---

## Proof / Check Shape for the Lane

The lane is **proven** when all of the following pass:

```
cargo build -p myosu-play --train
cargo test -p myosu-play training::tests::hand_completes_fold
cargo test -p myosu-play training::tests::hand_completes_showdown
cargo test -p myosu-play blueprint::tests::load_valid_artifact   # or mock
cargo test -p myosu-play advisor::tests::format_distribution_text
cargo test -p myosu-games-poker
Hero plays 1 complete hand via myosu-play --train: deal → action → showdown → result
Solver advisor shows EQUILIBRIUM section on hero's turn
```

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1: `myosu-play` Binary Skeleton + Shell Wiring
**Files**: `crates/myosu-play/` (new crate scaffold)
**What**: Bare `main.rs` with `--train` flag; creates `NlheRenderer` (hardcoded/hacky); wires into `Shell` from `myosu-tui`; proves render loop compiles and displays.
**Proof gate**: `cargo build -p myosu-play` exits 0; `myosu-play --train` renders the 5-panel layout without panic.

### Slice 2: `NlheRenderer` with Hardcoded States
**Files**: `crates/myosu-games-poker/src/renderer.rs` (new crate)
**What**: `GameRenderer` impl with pre-set game states (preflop, flop, hero turn, bot turn, showdown). No robopoker dependency yet. Parse input: `f/c/r X` shorthands. `pipe_output()` for agent mode.
**Proof gate**: `cargo test -p myosu-games-poker renderer::tests::render_preflop_state`

### Slice 3: `TrainingTable` + `HeuristicBackend`
**Files**: `crates/myosu-play/src/training.rs`
**What**: `TrainingTable` wrapping robopoker's `Game`; `HeuristicBackend` (equity-based, always available); practice chip tracking; `/deal`, `/board`, `/stack` commands; 200–500ms bot delay.
**Proof gate**: `cargo test -p myosu-play training::tests::hand_completes_showdown`

### Slice 4: `BlueprintBackend` with Graceful Fallback
**Files**: `crates/myosu-play/src/blueprint.rs`
**What**: Artifact discovery + manifest validation; mmap strategy lookup; fallback to `HeuristicBackend` on any load error. Falls back to heuristic immediately if no artifact found.
**Proof gate**: `cargo test -p myyosu-play blueprint::tests::lookup_returns_valid_distribution`

### Slice 5: `SolverAdvisor` (EQUILIBRIUM Panel)
**Files**: `crates/myosu-play/src/advisor.rs`
**What**: `SolverAdvisor` backed by `BlueprintBackend::action_distribution()`. Format: `raise 53% · call 35% · fold 12%`. `/advisor on/off` toggle. Default ON in training mode.
**Proof gate**: `cargo test -p myosu-play advisor::tests::format_distribution_text`

### Slice 6: Hand History + Session Stats
**Files**: `crates/myosu-play/src/recorder.rs`
**What**: JSON hand history to `{data-dir}/hands/hand_{N}.json`; session summary with win rate, BB/hand, showdown %.
**Proof gate**: `cargo test -p myosu-play recorder::tests::record_hand`

### Slice 7: Chain Discovery + Miner Client (Future)
**Files**: `crates/myosu-play/src/discovery.rs`, `crates/myosu-play/src/bot.rs`
**What**: Query chain for best miner by incentive score; HTTP `POST /strategy` to miner axon; fallback to random on timeout; best-of-N miner selection.
**Depends on**: `chain:runtime` lane

---

## Dependency on Other Lanes

| Lane | Type | What Is Used |
|------|------|-------------|
| `tui:shell` | Hard upstream | `Shell`, `ScreenManager`, `InputLine`, `EventLoop`, `Theme`, `GameRenderer` trait |
| `games:traits` | Hard upstream | `CfrGame`, `Profile`, `GameConfig`, `GameType`, `StrategyQuery/Response` |
| `robopoker` | Hard upstream (external repo) | `Game`, `Recall`, `Action` — absolute path deps; needs git migration |
| `chain:runtime` | Soft upstream (future) | Miner discovery via on-chain incentive scores |
| `chain:pallet` | Soft upstream (future) | `GameType` byte encoding convention must match |

---

## Phase Ordering

```
Phase A (Shell Integration — depends on tui:shell):
  Slice 1 → Slice 2

Phase B (Local Gameplay — depends on Phase A + games:traits):
  Slice 3 → Slice 4 → Slice 5 → Slice 6

Phase C (Chain-Connected — depends on chain:runtime):
  Slice 7
```

---

## Portability Note

Both `games:traits` and the upcoming `myosu-play` depend on robopoker crates via **absolute filesystem paths** (`/home/r/coding/robopoker/crates/...`). This blocks CI, contributors, and any clean publish. The `games:traits` lane review (at `outputs/games/traits/review.md`) documents this as its highest-priority Slice 1 fix. `play:tui`'s Slice 1 must also verify this is resolved before robopoker APIs are called.
