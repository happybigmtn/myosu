# `games:multi-game` Implementation — Slice 1: Crate Skeleton

## Slice Status

**COMPLETED** — Slice 1: Create `myosu-games-liars-dice` Crate Skeleton

## Fixup Note

**Issue**: The verification script inherited a broad test scope from the preflight, including
`cargo test -p myosu-play` and `cargo test -p myosu-games-poker` — packages that do not
exist in the workspace. Preflight passed (due to `set +e`) but `verify` failed.

**Fix**: Created `fabro/checks/multi-game-slice1.sh` scoped to only packages that exist in
the workspace: `myosu-games-liars-dice`, `myosu-games`, and `myosu-tui`. Updated
`verification.md` accordingly.

## Touched Files/Modules

### Created
- `crates/myosu-games-liars-dice/Cargo.toml` — package manifest with git dependency on `rbp-mccfr` at same rev as `myosu-games`
- `crates/myosu-games-liars-dice/README.md` — crate documentation
- `crates/myosu-games-liars-dice/src/lib.rs` — public re-exports for `LiarsDiceGame`, `LiarsDiceEdge`, `LiarsDiceTurn`, `LiarsDiceInfo`, `LiarsDiceEncoder`, `LiarsDiceProfile`
- `crates/myosu-games-liars-dice/src/game.rs` — `LiarsDiceGame` struct (stub) and `Bid` struct
- `crates/myosu-games-liars-dice/src/edge.rs` — `LiarsDiceEdge` enum (stub)
- `crates/myosu-games-liars-dice/src/turn.rs` — `LiarsDiceTurn` enum (stub)
- `crates/myosu-games-liars-dice/src/info.rs` — `LiarsDiceInfo` struct (stub)
- `crates/myosu-games-liars-dice/src/encoder.rs` — `LiarsDiceEncoder` struct (stub)
- `crates/myosu-games-liars-dice/src/profile.rs` — `LiarsDiceProfile` struct (stub)

### Modified
- `Cargo.toml` — added `crates/myosu-games-liars-dice` to workspace members

## Setup Steps Completed

1. **Workspace membership** — added `crates/myosu-games-liars-dice` to `[workspace].members`
2. **Git dependency** — `rbp-mccfr` and `rbp-core` from `happybigmtn/robopoker` at rev `04716310143094ab41ec7172e6cea5a2a66744ef`
3. **Dependency on `myosu-games`** — local path dependency for trait re-exports
4. **Module structure** — 6 stub modules (`game`, `edge`, `turn`, `info`, `encoder`, `profile`) with `todo!()` bodies
5. **Public re-exports** — `lib.rs` re-exports all 6 public types for downstream consumption
6. **Build verified** — `cargo build -p myosu-games-liars-dice` exits 0 with no warnings

## Deferred Items

| Item | Reason | Next Slice |
|------|--------|------------|
| `LiarsDiceGame::new()` implementation | Requires `rand` dice rolling | Slice 2 |
| `CfrGame` trait impl | Full game engine | Slice 2 |
| `CfrEdge`, `CfrTurn`, `CfrInfo` trait impls | Full game engine | Slice 2 |
| `LiarsDiceEncoder` encoding scheme | Direct enumeration logic | Slice 3 |
| `LiarsDiceProfile` training loop | MCCFR solver | Slice 3 |
| Nash convergence tests | Requires trained profile | Slice 3 |

## What This Slice Proved

1. The workspace correctly picks up the new crate (dependency resolution works)
2. The git dependency on `rbp-mccfr` at the correct rev resolves
3. The `myosu-games` local path dependency resolves
4. The module structure is sound (all types re-export correctly)
5. The crate compiles to a valid library with no warnings

## What This Enables

- **Slice 2** can now implement the actual game engine types without any setup blocking
- Downstream crates (like `myosu-play` for spectator relay) can depend on `myosu-games-liars-dice`

## Architectural Notes

- `crate-type = ["lib"]` — library only, no binary
- The `Bid` struct uses `u8` for quantity (1-12 max) and face (1-6)
- The bid history is specified as `Option<Bid>` array of size 12 (fixed-size for `CfrGame: Copy` constraint)
- All public types use `#[derive(Clone, Copy, Debug)]` to satisfy trait bounds in subsequent slices
