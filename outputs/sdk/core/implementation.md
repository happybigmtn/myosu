# `sdk:core` Implementation

## Lane
`core-implement`

## Goal
Implement the `sdk:core` lane (AC-SDK-01 through AC-SDK-05) to create a functional developer SDK for the myosu game-solving platform.

## What Was Done

### Slice 1: SDK Crate Skeleton (AC-SDK-01)

Created the `myosu-sdk` crate from scratch:

**Files created:**
- `crates/myosu-sdk/Cargo.toml` — workspace member with `myosu-games` dependency, `tui` feature flag
- `crates/myosu-sdk/src/lib.rs` — re-exports traits, types, and helpers from `myosu-games`

**Key design decisions:**
- Crate is a meta-crate (no new game logic, just re-exports)
- `tui` feature gates `GameRenderer` for headless miners
- Added to workspace `Cargo.toml`

### Slice 2: Trait Compliance Test Harness (AC-SDK-03)

Created the test harness for validating `CfrGame` implementations:

**Files created:**
- `crates/myosu-sdk/src/testing/mod.rs` — module declarations
- `crates/myosu-sdk/src/testing/game_valid.rs` — `assert_game_valid<G>()` using `G::root()`
- `crates/myosu-sdk/src/testing/convergence.rs` — `assert_solver_converges<G, E>()` stub
- `crates/myosu-sdk/src/testing/tests.rs` — test cases using `RpsGame` from `rbp-mccfr`

**Key design decisions:**
- `assert_game_valid` uses `G::root()` instead of `G::default()` because `CfrGame` trait provides `root()` not `Default`
- Test harness uses `rbp-mccfr::RpsGame` as the reference implementation
- Compliance checks are stubbed with documentation for full implementation

### Slice 3: Scaffold Tool (AC-SDK-02)

Created the `ScaffoldGenerator` for generating new game crates:

**Files created:**
- `crates/myosu-sdk/src/scaffold/mod.rs` — `ScaffoldGenerator` struct with validation
- `crates/myosu-sdk/src/scaffold/templates.rs` — project template strings
- `crates/myosu-sdk/src/scaffold/tests.rs` — scaffold tests

**Key design decisions:**
- Game name validation (no spaces, valid Rust identifier)
- Generated crate compiles immediately with `todo!()` stubs
- Refuses to overwrite existing directories
- Template includes: `Cargo.toml`, `src/lib.rs`, `src/game.rs`, `src/encoder.rs`, `src/renderer.rs`, `src/tests.rs`, `README.md`

### Slice 4: Registration CLI (AC-SDK-04)

Created the `myosu register` command:

**Files created:**
- `crates/myosu-sdk/src/register/mod.rs` — `RegisterArgs` CLI struct and `register_game()` function
- `crates/myosu-sdk/src/register/tests.rs` — CLI parsing tests

**Key design decisions:**
- Uses `clap` for CLI argument parsing
- Connection timeout of 5 seconds enforced
- CLI side only (chain extrinsic not yet implemented)
- Default exploit unit is "exploit" with baseline 1.0

### Slice 5: Developer Documentation (AC-SDK-05)

Created the developer documentation:

**Files created:**
- `docs/sdk/quickstart.md` — 30-minute Kuhn Poker implementation guide
- `docs/sdk/trait-reference.md` — `CfrGame`, `Encoder`, `GameRenderer` trait documentation
- `docs/sdk/registration.md` — chain registration flow guide

## Files Changed

### Modified
- `Cargo.toml` — added `crates/myosu-sdk` to workspace members

### Created
- `crates/myosu-sdk/` — entire crate (new directory tree)
- `docs/sdk/` — documentation directory (new)

## Dependencies
- `myosu-games` — trusted game trait re-exports (kept leaf)
- `rbp-mccfr` — dev dependency for RPS reference implementation
- `tempfile` — dev dependency for scaffold tests
- `clap` — workspace dependency for CLI

## Blockers Resolved
- **Greenfield crate** — entire `myosu-sdk` created from scratch
- **RPS test target** — uses `rbp-mccfr::RpsGame` as reference implementation
- **`CfrGame` trait** — discovered `root()` method instead of `Default`

## Blockers Remaining
- **On-chain extrinsic** — `register_game_type` extrinsic not yet implemented in `chain:pallet`
- **Full compliance checks** — `assert_game_valid` is stubbed with documentation
- **Convergence testing** — `assert_solver_converges` is stubbed
