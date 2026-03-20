# `play:tui` Implementation Artifact

## Slice Coverage

This run implemented the next approved `play:tui` slice:

| Slice | Surface | Status | Files |
|-------|---------|--------|-------|
| Slice 4 | `BlueprintBackend` with graceful fallback | Done | `crates/myosu-play/src/blueprint.rs`, `crates/myosu-play/src/main.rs`, `crates/myosu-play/src/training.rs`, `crates/myosu-play/Cargo.toml`, `Cargo.lock` |

## What Landed

### `crates/myosu-play/src/blueprint.rs`

Added the Slice 4 blueprint loading surface:

- `BlueprintManifest` now defines the schema-checked metadata contract for training artifacts.
- `BlueprintBackend::load_default()` discovers artifacts in the approved order: `MYOSU_BLUEPRINT_DIR`, `MYOSU_DATA_DIR/.myosu/blueprints`, then `~/.myosu/blueprints`.
- The loader validates schema version, abstraction hash, and profile hash before any strategy lookup is exposed.
- `blueprint.keys.bin`, `blueprint.values.bin`, and `blueprint.isomorphism.bin` are opened as read-only mapped files so the runtime contract is backed by mapped bytes instead of eager file copies.
- `BlueprintBackend` implements `BotBackend` and converts a live recall into a legal action distribution by matching a street-and-pressure bucket from the mapped isomorphism index.

### Training Startup Wiring

- `resolve_training_backend()` in `training.rs` now tries `BlueprintBackend::load_default()` first.
- On success, training mode surfaces `bot strategy: blueprint · exploit X mbb/h`.
- On any load failure, training mode falls back to `HeuristicBackend` and exposes the loader error as the strategy status line.
- `myosu-play --train` now picks up that backend selection automatically before entering the existing Slice 3 training flow.

### Owned-Surface Cleanup

- Removed dead renderer helpers in `crates/myosu-games-poker/src/renderer.rs`.
- Removed two unused locals in `crates/myosu-games-poker/src/truth_stream.rs`.

These were small warning-only corrections inside the same owned `play:tui` surface so the slice proof can stay clean.

## Automated Coverage Added in This Slice

`blueprint.rs` now carries six focused unit tests:

- `blueprint::tests::load_valid_artifact`
- `blueprint::tests::missing_dir_returns_error`
- `blueprint::tests::schema_mismatch_returns_error`
- `blueprint::tests::hash_mismatch_returns_error`
- `blueprint::tests::lookup_returns_valid_distribution`
- `blueprint::tests::distribution_sums_to_one`

## Slice Notes

- This slice proves the blueprint discovery, manifest validation, hash validation, mapped-file loading, and legal distribution lookup path end-to-end with the canonical artifact filenames.
- Solver advisor, recorder output, and chain-backed play remain outside this slice.
