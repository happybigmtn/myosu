# Slice 2: `PokerSolver` + Checkpoint Format

## Status

Slice 2 is now the only compiled `myosu-games-poker` surface. The solver wrapper and checkpoint framing are implemented, but the reviewed training/exploitability proof is still blocked by encoder availability (`RF-02` / abstraction artifact gap).

## Touched Surfaces

- `crates/myosu-games-poker/src/lib.rs`
- `crates/myosu-games-poker/src/solver.rs`

## What Changed

1. `lib.rs` now exports only the approved Slice 2 API:
   - `Flagship`
   - `PokerSolver`
   - `PokerSolverError`
   - `myosu-games` convenience re-exports

2. `solver.rs` now matches the reviewed Slice 2 direction instead of the prior placeholder/stub state:
   - `PokerSolver::new(encoder: NlheEncoder)` now requires an explicit encoder
   - `PokerSolver::from_parts(profile, encoder)` rebuilds a solver from state
   - `PokerSolver::save(path)` writes `MYOS` magic + version + bincode profile payload
   - `PokerSolver::load(path, encoder)` validates magic/version and reconstructs the solver with a caller-supplied encoder
   - `epochs()`, `strategy()`, `encoder()`, `profile()`, and `snapshot_profile()` are implemented
   - `exploitability()` now degrades to `INFINITY` on panic/non-finite output instead of propagating a hard failure from empty/misaligned solver state

3. Solver tests now provide real executable proof for the parts of Slice 2 that do not require a populated abstraction table:
   - `create_empty_solver`
   - `strategy_is_valid_distribution`
   - `checkpoint_roundtrip`
   - invalid checkpoint magic rejection
   - unsupported checkpoint version rejection

4. The two reviewed proof names that still require real abstractions are preserved but explicitly marked blocked:
   - `train_100_iterations`
   - `exploitability_decreases`

## Deliberate Scope Control

- Future slices (`query.rs`, `wire.rs`, `exploit.rs`, `training.rs`) remain on disk but are intentionally outside the compiled Slice 2 surface.
- This keeps the crate aligned with the “smallest next approved slice” instruction instead of shipping a mixed surface with future-slice stubs.

## Honest Blocker

- Real MCCFR training and full-tree exploitability require a populated `NlheEncoder`.
- The reviewed non-DB encoder loading path (`RF-02`: `from_map` / `from_file` / `from_dir`) is still missing from the pinned robopoker dependency in this workspace.
- No full abstraction artifact is present in-repo, so Slice 2 cannot yet prove the reviewed “train 100 iterations” and “exploitability decreases” outcomes honestly.
