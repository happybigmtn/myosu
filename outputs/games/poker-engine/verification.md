# `games:poker-engine` Verification — Slice 2

## Proof Commands That Passed

| Command | Exit Code | Result |
|---------|-----------|--------|
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-slice2 cargo build -p myosu-games-poker --offline` | 0 | Built the updated crate, including the new solver module and checkpoint dependencies |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-slice2 cargo test -p myosu-games-poker --offline` | 0 | Full crate suite passed: 10 unit tests green, 0 failed |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-slice2 cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip --offline` | 0 | Confirmed `MYOS` checkpoint write/read roundtrip works |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-slice2 cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution --offline` | 0 | Confirmed profile-driven strategy output remains a valid probability distribution |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-slice2 cargo test -p myosu-games-poker solver::tests::train_surfaces_missing_encoder_artifacts --offline` | 0 | Confirmed `train()` maps the encoder lookup panic into a typed error instead of crashing |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-slice2 cargo test -p myosu-games-poker solver::tests::exploitability_surfaces_missing_encoder_artifacts --offline` | 0 | Confirmed `exploitability()` maps the same encoder lookup panic into a typed error |

## Test Results

```text
running 10 tests
test solver::tests::create_empty_solver ... ok
test solver::tests::checkpoint_rejects_unknown_version ... ok
test solver::tests::checkpoint_rejects_bad_magic ... ok
test solver::tests::exploitability_surfaces_missing_encoder_artifacts ... ok
test tests::robopoker_nlhe_serde_surface_is_enabled ... ok
test tests::game_type_reexport_includes_nlhe_heads_up ... ok
test solver::tests::strategy_is_valid_distribution ... ok
test solver::tests::train_surfaces_missing_encoder_artifacts ... ok
test solver::tests::snapshot_profile_preserves_profile_data ... ok
test solver::tests::checkpoint_roundtrip ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Environment Notes

- A plain `cargo test -p myosu-games-poker` first attempted to refresh the crates.io index and failed because this sandbox has no DNS/network access.
- An offline Cargo run without overriding `CARGO_TARGET_DIR` hit the workspace default target path, which resolves to a read-only location in this environment.
- The passing proof path therefore used `--offline` plus `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-slice2`.

## Coverage Achieved By This Slice

- The new solver/checkpoint code compiles.
- Checkpoint format validation and profile roundtrip are covered.
- Strategy lookup over stored profile data is covered.
- Encoder-dependent solver paths now fail with typed errors instead of an uncaught panic when the encoder has no abstraction lookup data.

## Coverage Still Outside This Slice

The reviewed lane proof for a fully populated Slice 2 still depends on a non-DB encoder construction or artifact-ingestion path. Until that exists, this lane cannot honestly claim:

- successful `train(100)` over a populated NLHE abstraction map
- exploitability improvement after real training iterations

Those are the next proof targets once the encoder gap is resolved.
