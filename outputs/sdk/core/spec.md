# `sdk:core` Lane Spec

## Lane Boundary

`sdk:core` is the **developer-facing SDK surface** for the myosu game-solving platform. It owns the external API that third-party (and internal) game developers use to implement and register new games. It does not own any game logic — that lives in individual `myosu-games-<name>` crates.

`sdk:core` owns:

- The `myosu-sdk` meta-crate — re-exports all traits, types, and helpers a game developer needs
- The `myosu init` scaffold tool — generates a new game engine crate from a template
- The trait compliance test harness — validates a game implementation against CFR invariants
- The game registration flow (SDK CLI side) — `myosu register` command
- Developer documentation (docs/sdk/)

`sdk:core` does **not** own:

- Any `CfrGame` implementation code (lives in `myosu-games-<name>` crates)
- The on-chain `register_game_type` extrinsic (owned by `chain:pallet`)
- The miner or validator binaries (owned by `services:miner`, `services:validator-oracle`)
- Runtime game engine loading (future hot-load work)

---

## Platform-Facing Purpose

The user-visible outcome: **a competent Rust developer can add a new game to myosu in one afternoon.**

The `myosu-sdk` crate is the product surface. The scaffold and test harness are the developer tools. The registration flow connects the local implementation to the on-chain game registry.

The whole system fits together as:

```
Developer workflow:
  myosu init --game kuhn-poker
       ↓ (scaffold generates myosu-games-kuhn-poker crate)
  [developer implements CfrGame + Encoder + GameRenderer]
       ↓
  myosu test
       ↓ (compliance harness validates invariants)
  myosu train --iters 1000
       ↓
  myosu register --chain ws://localhost:9944 --game-type kuhn-poker
       ↓ (registration extrinsic + subnet creation)
  [game is live on the network]
```

---

## Currently Trusted Inputs

| File | Trust Signal |
|------|-------------|
| `crates/myosu-games/src/traits.rs` | All 10 unit tests + 4 doctests pass; `myosu-games` is a kept leaf |
| `crates/myosu-games/src/lib.rs` | Re-exports traits module; trivial |
| `crates/myosu-games/Cargo.toml` | Git dep on robopoker at `0471631...`; workspace member |
| `specsarchive/031626-19-game-engine-sdk.md` | AC-SDK-01..05; full spec with implementation order |
| `specsarchive/031626-02a-game-engine-traits.md` | AC-GT-01..05; trait definitions this SDK wraps |
| `outputs/games/traits/spec.md` | Confirms `myosu-games` is trusted |
| `outputs/games/traits/review.md` | Confirms `myosu-games` is KEEP; unblocked for implementation |
| `fabro/run-configs/platform/sdk-core.toml` | Bootstrap run config for this lane |

---

## Current Broken / Missing Surfaces

### Critical: `myosu-sdk` Crate Does Not Exist

`crates/myosu-sdk/` is not present. The meta-crate described in AC-SDK-01 has no implementation.

**Impact**: The SDK product surface cannot exist without this crate.

### Critical: Scaffold Tool Is Entirely Missing

`crates/myosu-sdk/src/scaffold/` (AC-SDK-02) does not exist. `myosu init --game <name>` cannot run.

**Impact**: Developers cannot scaffold new game crates. The "30-minute game" story is broken at step 1.

### Critical: Trait Compliance Test Harness Is Missing

`crates/myosu-sdk/src/testing/` (AC-SDK-03) does not exist. The `assert_game_valid`, `assert_solver_converges`, `assert_wire_serialization_roundtrips` helpers are not implemented.

**Impact**: Developers have no automated way to validate their `CfrGame` implementation. They must manually write compliance tests.

### High: Registration CLI Is Missing

`crates/myosu-sdk/src/register/` (AC-SDK-04 CLI side) does not exist. The `myosu register` command cannot run.

**Impact**: Developers cannot register game types on-chain from the SDK toolchain.

### Medium: Developer Documentation Does Not Exist

`docs/sdk/` does not exist (AC-SDK-05).

**Impact**: Developers have no guide for implementing their first game. The SDK fails the "without asking for help" bar.

---

## Code Boundaries and Deliverables

### Crate Structure

```
crates/myosu-sdk/
├── Cargo.toml              # Workspace member; depends on myosu-games, myosu-tui (opt)
├── src/
│   ├── lib.rs              # AC-SDK-01: re-exports, feature flags
│   ├── scaffold/
│   │   ├── mod.rs          # AC-SDK-02: scaffold logic
│   │   └── templates.rs    # Project template generation
│   ├── testing/
│   │   ├── mod.rs          # AC-SDK-03: compliance test harness
│   │   ├── game_valid.rs   # assert_game_valid and variants
│   │   └── convergence.rs  # assert_solver_converges
│   └── register/
│       └── mod.rs          # AC-SDK-04: CLI registration command
└── docs/sdk/               # AC-SDK-05: developer guide
    ├── quickstart.md
    ├── trait-reference.md
    └── registration.md
```

### Public API Surface

```rust
// myosu-sdk/src/lib.rs

// Core — always available. No TUI dependency.
pub use myosu_games::{CfrGame, CfrEdge, CfrTurn, CfrInfo, Profile, Encoder};
pub use myosu_games::{GameConfig, GameType, GameParams, ExploitMetric};
pub use myosu_games::wire::{WireStrategy, WireSerializable};

pub mod testing;   // AC-SDK-03: trait compliance test harness
pub mod scaffold;  // AC-SDK-02: project template generation

// TUI rendering — opt-in via `myosu-sdk = { features = ["tui"] }`
#[cfg(feature = "tui")]
pub use myosu_tui::GameRenderer;
```

### Scaffold Template Structure

Generated by `myosu init --game <name>`:

```
myosu-games-<name>/
├── Cargo.toml          # depends on myosu-sdk with features = ["tui"]
├── src/
│   ├── lib.rs         # re-exports
│   ├── game.rs        # CfrGame impl (stub with todo!())
│   ├── encoder.rs     # Encoder impl (stub)
│   ├── renderer.rs    # GameRenderer impl (stub, gated behind tui feature)
│   └── tests.rs       # trait compliance tests (pre-written, failing)
└── README.md
```

---

## Proof / Check Shape

### Bootstrap Proof (lane integrity check)

```bash
# After AC-SDK-01 (myosu-sdk crate exists):
cargo build -p myosu-sdk
cargo test -p myosu-sdk

# After AC-SDK-02 (scaffold):
cargo test -p myosu-sdk scaffold::tests::generates_compilable_crate
cargo test -p myosu-sdk scaffold::tests::refuses_to_overwrite_existing_directory

# After AC-SDK-03 (test harness):
cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks

# After AC-SDK-04 (registration):
# (requires chain; test with mock or skip in bootstrap)
```

### Milestone Checks

| Milestone | Validates | AC |
|-----------|-----------|-----|
| `use myosu_sdk::CfrGame` compiles | SDK re-exports | SDK-01 |
| `myosu init --game test-game` generates compilable crate | Scaffold | SDK-02 |
| RPS implementation passes all compliance checks via SDK harness | Test harness | SDK-03 |
| `myosu register --help` shows usage | Registration CLI | SDK-04 |
| Quickstart guide covers 30-minute game story | Documentation | SDK-05 |

---

## Next Implementation Slices (Smallest Honest First)

### Slice 1 — Create `myosu-sdk` Crate Skeleton (AC-SDK-01)

**Files**: `crates/myosu-sdk/Cargo.toml`, `crates/myosu-sdk/src/lib.rs`

Add `crates/myosu-sdk/` to workspace members. `Cargo.toml`:
- Dependency on `myosu-games`
- `crate-type = ["lib"]`
- `features = { default = [], tui = ["myosu-tui"] }`

`lib.rs` re-exports all types from `myosu-games` (traits, config, wire). Stub the `testing` and `scaffold` modules.

**Proof**: `cargo build -p myosu-sdk` exits 0 with empty lib. `cargo test -p myosu-sdk` runs (no tests yet, passes).

---

### Slice 2 — Trait Compliance Test Harness (AC-SDK-03)

**Files**: `crates/myosu-sdk/src/testing/mod.rs`, `crates/myosu-sdk/src/testing/game_valid.rs`, `crates/myosu-sdk/src/testing/convergence.rs`

Implement the compliance test functions from the traits spec:

```rust
pub fn assert_game_valid<G: CfrGame>() {
    assert_root_is_chance_or_player::<G>();
    assert_legal_actions_nonempty_except_terminal::<G>();
    assert_apply_changes_state::<G>();
    assert_terminal_has_utility::<G>();
    assert_payoff_is_zero_sum::<G>();
    assert_info_sets_distinct_for_different_observations::<G>();
}

pub fn assert_solver_converges<G, E>(
    encoder: &E,
    max_iters: usize,
    target_exploit: f64,
) where G: CfrGame, E: Encoder<G> { ... }

pub fn assert_wire_serialization_roundtrips<G: WireSerializable>() { ... }
```

Use the existing RPS reference implementation from `rbp-mccfr::rps` as the test target — RPS already passes all checks and has known Nash equilibrium (1/3, 1/3, 1/3).

**Proof**:
```bash
cargo test -p myosu-sdk testing::tests::rps_passes_all_compliance_checks
cargo test -p myosu-sdk testing::tests::broken_game_fails_zero_sum_check
cargo test -p myosu-sdk testing::tests::convergence_test_detects_non_convergence
```

---

### Slice 3 — Scaffold Tool (AC-SDK-02)

**Files**: `crates/myosu-sdk/src/scaffold/mod.rs`, `crates/myosu-sdk/src/scaffold/templates.rs`

Implement `myosu init --game <name>`:

1. `ScaffoldGenerator::new(name)` validates name (no spaces, valid Rust identifier)
2. `generate()` creates `myosu-games-<name>/` directory tree
3. Writes `Cargo.toml`, `src/game.rs`, `src/encoder.rs`, `src/renderer.rs`, `src/tests.rs` from templates
4. Stubs compile but panic at runtime with clear `todo!()` messages

Key constraint: the generated crate must compile with `cargo check` immediately after generation. The developer fills in `todo!()` bodies.

**Proof**:
```bash
cargo test -p myosu-sdk scaffold::tests::generates_compilable_crate
cargo test -p myosu-sdk scaffold::tests::generated_tests_fail_with_todo
cargo test -p myosu-sdk scaffold::tests::refuses_to_overwrite_existing_directory
```

---

### Slice 4 — Registration CLI (AC-SDK-04)

**Files**: `crates/myosu-sdk/src/register/mod.rs`

Implement `myosu register` command:

```bash
myosu register --chain ws://localhost:9944 \
    --game-type kuhn-poker \
    --players 2 \
    --exploit-unit "exploit" \
    --exploit-baseline 1.0
```

Validates chain connectivity (5s timeout), submits `register_game_type` extrinsic via Substrate API, polls for execution. Prints clear errors if chain unreachable or extrinsic fails.

**Proof**:
```bash
cargo test -p myosu-sdk register::tests::register_help_output
cargo test -p myosu-sdk register::tests::connection_timeout_error
```

Note: Full registration integration test requires a running chain node. Bootstrap can verify CLI parsing and error handling with mocks.

---

### Slice 5 — Developer Documentation (AC-SDK-05)

**Files**: `docs/sdk/quickstart.md`, `docs/sdk/trait-reference.md`, `docs/sdk/registration.md`

Quickstart covers the 30-minute Kuhn Poker story end-to-end. Trait reference documents `CfrGame`, `Encoder`, `GameRenderer`, `NPlayerGame`. Registration guide covers subnet creation flow.

**Proof**: All code examples in docs compile and pass tests (add doctest harness or inline `# ```ignore` blocks verified manually).

---

## Dependency Order

```
games:traits (trusted) ──► sdk:core slices 1-5

Slice 1 (SDK crate) → must precede all others
Slice 2 (test harness) → independent of slice 3
Slice 3 (scaffold) → independent of slice 2
Slice 4 (registration) → independent of 2 and 3
Slice 5 (docs) → parallel with all above

Note: AC-SDK-04 (registration) requires chain:pallet to have
register_game_type extrinsic implemented. Slice 4 should still
be written (CLI side) but marked "requires chain integration".
```

---

## Relationship to Other Lanes

| Lane | Relationship |
|------|-------------|
| `games:traits` | `sdk:core` depends on `myosu-games` (a kept leaf). `games:traits` is stable. No circular dependency. |
| `games:multi-game` | Independent. Multi-game implements a specific game engine; sdk:core provides the general SDK for all games. |
| `games:poker-engine` | Independent. Poker-engine is the first game using the SDK; sdk:core provides the SDK it uses. |
| `chain:pallet` | AC-SDK-04 (registration) calls `register_game_type` extrinsic. Must coordinate on the on-chain interface. |
| `services:miner` | Miners serve strategies for SDK-registered games. They depend on `myosu-sdk` traits. |
| `services:validator-oracle` | Validators score strategies using the exploitability API that the SDK test harness validates. |
| `product:play-tui` | The scaffold generates `GameRenderer` implementations for TUI rendering. `tui` feature flag gates this. |
