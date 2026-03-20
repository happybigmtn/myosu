# `games:multi-game` Verification — Slice 1: Crate Skeleton

## Fixup Note

**Root cause**: The verification script inherited the full preflight test suite, which
includes `cargo test -p myosu-play` and `cargo test -p myosu-games-poker`. Those
packages do not exist in the workspace — `myosu-play` is commented out in
`Cargo.toml` and `myosu-games-poker` has never existed. The preflight passed
because it used `set +e`; the `verify` stage failed because it uses `set -euo pipefail`.

**Fix**: Created `fabro/checks/multi-game-slice1.sh` scoped to only packages that
exist in the current workspace. Updated this artifact accordingly.

## Bootstrap Gate Proof (Slice 1 Scoped)

### `cargo build -p myosu-games-liars-dice`

**Result**: PASS

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.41s
```

Zero warnings on the final build.

### `cargo test -p myosu-games-liars-dice`

**Result**: PASS

```
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests myosu_games_liars_dice
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Note: No tests exist yet — test implementations belong to Slices 2 and 3.

### Smoke: existing workspace packages

**Result**: PASS

```
cargo test -p myosu-games       → 10 passed; 0 failed
cargo test -p myosu-tui         → 82 passed; 2 ignored (require real TTY)
```

The new crate does not break any existing tests.

## What the Bootstrap Gate Proved

1. **Crate resolution** — The workspace correctly resolves `myosu-games-liars-dice` as a member
2. **Dependency chain** — `rbp-mccfr` at rev `04716310143094ab41ec7172e6cea5a2a66744ef` resolves correctly
3. **Local dependency** — `myosu-games` path dependency resolves correctly
4. **Module exports** — All 6 public re-exports in `lib.rs` compile without errors
5. **Library shape** — Produces a valid `libmyosu_games_liars_dice.rlib` artifact
6. **No regression** — Existing `myosu-games` and `myosu-tui` tests still pass

## Proof Gate Completeness (Slice 1 Scoped)

| Command | Expected | Actual | Status |
|---------|----------|--------|--------|
| `cargo build -p myosu-games-liars-dice` | exit 0 | exit 0 | PASS |
| `cargo test -p myosu-games-liars-dice` | exit 0 | exit 0 | PASS |
| `cargo test -p myosu-games` | exit 0 | exit 0 | PASS |
| `cargo test -p myosu-tui` | exit 0 | exit 0 | PASS |

## Slice 1 Completeness Assessment

**The slice is complete enough to proceed to Slice 2.**

The bootstrap gate proves the crate skeleton is sound. Slice 2 (game engine) can now implement the actual `CfrGame`, `CfrEdge`, `CfrTurn`, `CfrInfo` trait implementations without any setup blocking.

## Remaining Proof Gates

The following commands are **blocked on later slices**:

| Blocked Command | Required By |
|----------------|-------------|
| `cargo test -p myosu-games-liars-dice game::tests::*` | Slice 2 (MG-01) |
| `cargo test -p myosu-games-liars-dice solver::tests::*` | Slice 3 (MG-02) |
| `cargo test -p myosu-games registry::tests::*` | Slice 4 (CS-01) |
| `cargo test -p myosu-play spectate::tests::*` | Slice 5 (SP-01) |
| `cargo test -p myosu-tui spectate::tests::*` | Slice 6 (SP-02) |
| `cargo test -p myosu-games-poker` | Slice 7 (MG-03) |

Note: `myosu-play` is not yet a workspace member. `myosu-games-poker` does not exist
in this repository.
