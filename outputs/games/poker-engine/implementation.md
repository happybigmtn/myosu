# `games:poker-engine` Implementation — Slice 2

## Slice Implemented

**Slice 2 — solver wrapper plus versioned checkpoint I/O**

This increment adds the `PokerSolver` surface for `myosu-games-poker` and lands the file checkpoint format defined by the lane spec: 4-byte `MYOS` magic, `u32` version, then bincode-encoded `NlheProfile`.

The reviewed robopoker pin still does not expose a non-DB path for building a populated `NlheEncoder`, so this slice focuses on the solver work that can be completed inside the owned `games:poker-engine` surfaces without mutating robopoker itself:

- wrap `rbp_nlhe::Flagship`
- expose profile-driven strategy lookup
- persist and restore profiles with version checks
- convert encoder-lookup panics into typed errors for `train()` and `exploitability()`

## What Changed

### Workspace root: `Cargo.toml`

- Added `rbp-core` to workspace dependencies at the same reviewed robopoker rev as the existing poker crates.
- Kept the robopoker rev pinned to `04716310143094ab41ec7172e6cea5a2a66744ef`.

### Lockfile: `Cargo.lock`

- Recorded `bincode v1.3.3` for checkpoint serialization.
- Recorded the added `rbp-core` and `thiserror` edges for `myosu-games-poker`.

### Crate manifest: `crates/myosu-games-poker/Cargo.toml`

- Added `bincode = "1"` for checkpoint payload encoding.
- Added `thiserror` for a typed error surface.
- Added a dev dependency on `rbp-core` so the unit tests can use `Arbitrary` with the same pinned rev as the rest of the poker stack.

### Crate root: `crates/myosu-games-poker/src/lib.rs`

- Exported the new solver module.
- Re-exported `PokerSolver`, `PokerSolverError`, `CHECKPOINT_MAGIC`, and `CHECKPOINT_VERSION`.
- Re-exported `NlheEncoder` and `NlheProfile` alongside the existing NLHE types so later slices can compose on the same public surface.

### New solver surface: `crates/myosu-games-poker/src/solver.rs`

Added a concrete `PokerSolver` wrapper around `rbp_nlhe::Flagship` with:

- `new_empty()`, `from_parts()`, `from_inner()`, and `into_inner()`
- `profile()`, `profile_mut()`, `encoder()`, and `epochs()`
- `strategy(&NlheInfo) -> StrategyResponse<NlheEdge>` using the profile’s averaged distribution
- `train(iterations)` and `exploitability()` with panic capture so missing encoder lookup data becomes `PokerSolverError::MissingEncoderArtifacts`
- `snapshot_profile()` via bincode roundtrip, which gives the lane a practical profile-copy path even though robopoker does not derive `Clone` on `NlheProfile`
- `save()`, `load()`, and `load_profile()` for the reviewed checkpoint format

### Solver Tests

Added unit coverage for:

- empty solver construction
- strategy distributions derived from the profile surface
- checkpoint roundtrip plus invalid magic and invalid version rejection
- snapshot/profile copying
- structured error mapping for encoder-dependent `train()` and `exploitability()` calls on an empty encoder

## Scope Boundary Kept

This slice does **not** claim successful encoder-backed training convergence. The reviewed robopoker pin still only hydrates `NlheEncoder` from the database path, so `NlheEncoder::default()` is enough for profile-only operations like `strategy()` and checkpoint I/O, but not for full tree traversal.

That boundary is now explicit in code rather than hidden behind a panic.

## What Remains After This Slice

- Wire serialization in `wire.rs`
- Query bridging in `query.rs`
- Remote/local exploitability helpers in a dedicated `exploit.rs`
- Batch/session orchestration in `training.rs`
- A vetted non-DB encoder construction path or encoder artifact ingestion path so `train(100)` and exploitability decrease proofs can run against a populated NLHE abstraction map
