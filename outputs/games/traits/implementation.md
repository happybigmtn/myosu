# `games:traits` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Replace Absolute Paths with Git Rev**

Replaced the two absolute `path` dependencies in `crates/myosu-games/Cargo.toml` with pinned `git` dependencies pointing to the `happybigmtn/robopoker` GitHub repository at a specific `rev`.

## What Changed

### `crates/myosu-games/Cargo.toml`

**Before (HEAD):**
```toml
# Robopoker dependencies - local path during fork development
# TODO: Switch to git dependency once happybigmtn/robopoker fork is published
rbp-core = { path = "/home/r/coding/robopoker/crates/util" }
rbp-mccfr = { path = "/home/r/coding/robopoker/crates/mccfr" }
```

**After (working tree):**
```toml
# Robopoker dependencies
rbp-core = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }
rbp-mccfr = { git = "https://github.com/happybigmtn/robopoker", rev = "04716310143094ab41ec7172e6cea5a2a66744ef" }
```

The `rev` pins to the current HEAD of the local robopoker clone, ensuring the dependency graph is frozen at a known commit. The `rbp-transport` crate (a transitive dependency) is also fetched via the same git source and locked compatibly.

Additionally, a `[lib]` section was added to make the library crate-type explicit:

```toml
[lib]
crate-type = ["lib"]
```

`name = "myosu-games"` was already explicit in `[package]`.

## Proof Commands for This Lane

```bash
# Must pass: compile check
cargo check -p myosu-games

# Must pass: all unit + doctests
cargo test -p myosu-games

# Must pass: fetch succeeds without local robopoker on filesystem
cargo fetch
```

All three commands exit 0 with this slice applied.

## Test Results

```
running 10 tests
  traits::tests::game_type_from_bytes_custom    ... ok
  traits::tests::game_config_nlhe_params        ... ok
  traits::tests::game_config_serializes         ... ok
  traits::tests::game_type_num_players          ... ok
  traits::tests::reexports_compile              ... ok
  traits::tests::game_type_from_bytes_known      ... ok
  traits::tests::game_type_to_bytes_roundtrip   ... ok
  traits::tests::strategy_response_probability_for ... ok
  traits::tests::strategy_response_validates    ... ok
  traits::tests::strategy_query_response_roundtrip ... ok
test result: ok. 10 passed

running 4 doctests
  traits::GameType::from_bytes (line 75)   ... ok
  traits::GameType::num_players (line 120) ... ok
  traits::GameType::to_bytes (line 101)    ... ok
  README.md usage example (line 20)         ... ok
test result: ok. 4 passed
```

## What Remains for Future Slices

| Slice | Description | Status |
|-------|-------------|--------|
| Slice 2 | Add explicit `name` and audit `crate-type` | **Done** — `name = "myosu-games"` was already explicit; `[lib]` section with `crate-type = ["lib"]` added in this session |
| Slice 3 | Resolve `edition = "2024"` (downgrade to 2021 or add `rust-toolchain.toml`) | Pending |
| Slice 4 | Add `StrategyQuery`/`StrategyResponse` integration tests against a real `CfrGame` | Pending |
