# `games:traits` Lane Spec

## Lane Boundary

`games:traits` is the **trait abstraction and game-configuration surface** for the myosu game-solving chain. It owns:

- The public re-exports of robopoker's game-agnostic CFR traits (`CfrGame`, `CfrEdge`, `CfrTurn`, `CfrInfo`, `Profile`, `Encoder`)
- The `Probability` and `Utility` types from `rbp-core`
- The myosu-specific game configuration layer: `GameConfig`, `GameType`, `GameParams`
- The miner-validator communication types: `StrategyQuery<I>`, `StrategyResponse<E>`
- All doctests and unit tests for the above

`games:traits` does **not** own:
- Any robopoker implementation code (that lives in the `robopoker` sibling repo)
- Any on-chain pallet logic (that lives in `crates/myosu-chain/`)
- Any miner or validator binary (that lives in `crates/myosu-miner/`, `crates/myosu-validator/`)

## Currently Trusted Files

| File | Trust Signal |
|------|-------------|
| `crates/myosu-games/Cargo.toml` | Compiles; workspace member; `cargo test -p myosu-games` passes |
| `crates/myosu-games/src/lib.rs` | Trivially small; only re-exports `traits` module |
| `crates/myosu-games/src/traits.rs` | All 10 unit tests pass; all 4 doc tests pass |
| `crates/myosu-games/README.md` | Matches current code; doctests verify the examples |

## Current Proof Commands

### Bootstrap proof (trusted lane check)
```bash
cargo test -p myosu-games
```
Exit code 0 on current HEAD. All 10 unit tests + 4 doctests pass.

### Implementation proof (post-bootstrap verification)
```bash
cargo check -p myosu-games && cargo test -p myosu-games
```
Both commands must exit 0.

### Exact test inventory (verified 2026-03-19)
```
running 10 tests
  test traits::tests::game_type_from_bytes_custom    ... ok
  test traits::tests::game_config_nlhe_params        ... ok
  test traits::tests::game_type_from_bytes_known     ... ok
  test traits::tests::game_config_serializes          ... ok
  test traits::tests::game_type_to_bytes_roundtrip   ... ok
  test traits::tests::game_type_num_players          ... ok
  test traits::tests::reexports_compile               ... ok
  test traits::tests::strategy_response_probability_for  ... ok
  test traits::tests::strategy_response_validates     ... ok
  test traits::tests::strategy_query_response_roundtrip ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured

running 4 doctests
  traits::GameType::num_players (line 120)  ... ok
  README.md usage example (line 20)         ... ok
  traits::GameType::from_bytes (line 75)   ... ok
  traits::GameType::to_bytes (line 101)    ... ok

test result: ok. 4 passed
```

## Portability Blockers

### Blocker 1: Absolute Robopoker Path Dependencies
**File:** `crates/myosu-games/Cargo.toml` lines 16–17
```toml
rbp-core = { path = "/home/r/coding/robopoker/crates/util" }
rbp-mccfr = { path = "/home/r/coding/robopoker/crates/mccfr" }
```

These encode the caller's machine filesystem layout. The crate is **not portable** — any developer without `robopoker` checked out at exactly `/home/r/coding/robopoker/` cannot build it.

**Impact:** Blocks CI, blocks contributors, blocks any clean `cargo publish`.

**Must be resolved before:** this crate can be treated as a truly reusable published crate.

**Resolution options (not decided yet):**
- Replace with a git dependency once `happybigmtn/robopoker` is published
- Replace with a crates.io versioned dependency once published
- Keep as-is with explicit "developer setup requires robopoker sibling clone" documentation

### Blocker 2: `edition = "2024"` Workspace Pin
**File:** `Cargo.toml` workspace root (line 15)
```toml
edition = "2024"
```

Rust 2024 edition is **not yet stable** as of early 2026. This may cause build failures on stable Rust toolchains and limits contributor toolchain choices.

**Impact:** Forces contributors onto a nightly or beta Rust toolchain that supports 2024 edition.

**Must be resolved before:** this crate works with a stable Rust toolchain.

## Repo-Shape Blockers

### Issue 1: Robopoker Is a Separate Repo
`rbp-core` and `rbp-mccfr` are not part of the myosu monorepo — they live in `/home/r/coding/robopoker/`. This means:
- No atomic commits across robopoker + myosu
- No unified `cargo` workspace spanning both
- Version coupling is managed manually (the TODO comment in `Cargo.toml` acknowledges this)

### Issue 2: No `[lib]` Declaration in `Cargo.toml`
`crates/myosu-games/Cargo.toml` lacks an explicit `name = "myosu_games"` field (the `name` is derived from the directory name by cargo convention, which works but is implicit). More critically, it does not declare `crate-type = ["lib"]` explicitly, which is fine for normal library crates but worth noting for binary-only tooling that might depend on this.

## Smallest Approved Implementation Slices (in order)

### Slice 0 — Path Dependency Audit (do not change code)
Verify the robopoker re-exports are stable and that no robopoker-internal types leak through `myosu_games`'s public API. Produce a one-page audit note confirming `pub use rbp_core::...` and `pub use rbp_mccfr::...` surfaces are intentional and minimal.

### Slice 1 — Replace Absolute Paths with Git Rev (highest priority)
Replace:
```toml
rbp-core = { path = "/home/r/coding/robopoker/crates/util" }
rbp-mccfr = { path = "/home/r/coding/robopoker/crates/mccfr" }
```
With:
```toml
rbp-core = { git = "https://github.com/happybigmtn/robopoker", rev = "..." }
rbp-mccfr = { git = "https://github.com/happybigmtn/robopoker", rev = "..." }
```
**Proof:** `cargo fetch` succeeds without local robopoker; `cargo test -p myosu-games` passes.

### Slice 2 — Add Explicit `name` and Audit `crate-type`
Add explicit `name = "myosu_games"` to `Cargo.toml` and confirm `crate-type = ["lib"]` is appropriate (it is, since this is a library crate consumed by miner/validator binaries).

### Slice 3 — Resolve Edition 2024
Either downgrade to `edition = "2021"` or document the nightly requirement and add a `rust-toolchain.toml`.

### Slice 4 — Add `StrategyQuery` / `StrategyResponse` Integration Tests Against a Real Game Trait
Wire up a real `CfrGame` implementation (even a trivial mock) to `StrategyQuery`/`StrategyResponse` to prove the serialization roundtrips work across the miner-validator wire format.

## What the Implementation Lane May Change First

The implementation lane is **allowed to change first** (in priority order):

1. **Cargo.toml path-to-git migration** — this is the highest-leverage, lowest-risk change: it fixes portability without touching any logic
2. **Edition pin resolution** — either downgrade to 2021 or add `rust-toolchain.toml`
3. **`GameType`/`GameParams` extensions** — adding new game variants is safe and additive
4. **`StrategyQuery`/`StrategyResponse` serde traits** — the serialization layer is isolated and well-tested

The implementation lane must **not change first** (these require wider coordination):
- The robopoker re-export surface (changing what is re-exported requires robopoker compatibility guarantees)
- The `CfrGame`/`CfrEdge` trait signatures (these are owned by robopoker)
- The `Probability`/`Utility` type definitions (these are owned by `rbp-core`)
