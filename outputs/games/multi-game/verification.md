# `games:multi-game` Verification — Slice 1 Fixup

## Verification Scope

This fixup revalidated the active Slice 1 proof gate and reproduced the first deterministic review-script blocker. It does not claim that Slice 2 through Slice 7 are complete.

Compilation commands were run with `CARGO_TARGET_DIR=/tmp/myosu-mg-fixup` because `cargo metadata` reports the default target directory as `/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target`, which is outside the writable sandbox for this run.

## Automated Commands Run

| Command | Exit Code | Outcome |
|---------|-----------|---------|
| `cargo metadata --no-deps --format-version 1` | 0 | Workspace members resolve to `myosu-games`, `myosu-games-liars-dice`, `myosu-tui`, and `pallet-game-solver` |
| `CARGO_TARGET_DIR=/tmp/myosu-mg-fixup cargo build -p myosu-games-liars-dice` | 0 | Passed; the Slice 1 crate skeleton compiles |
| `CARGO_TARGET_DIR=/tmp/myosu-mg-fixup cargo test -p myosu-games-liars-dice` | 0 | Passed; 2 unit tests passed and 0 doctests ran |
| `CARGO_TARGET_DIR=/tmp/myosu-mg-fixup cargo test -p myosu-games` | 0 | Passed; 10 unit tests and 4 doctests still pass after the workspace change |
| `cargo test -p myosu-play` | 101 | Failed immediately: `package ID specification 'myosu-play' did not match any packages` |
| `cargo test -p myosu-games-poker` | 101 | Failed immediately: `package ID specification 'myosu-games-poker' did not match any packages` |

## Active Proof Gate Status

The current slice's first proof gate is green:

- `myosu-games-liars-dice` builds by package name
- the crate's two Slice 1 smoke tests pass
- `myosu-games` still passes its current suite after the workspace membership change

That is the full approved proof surface for Slice 1.

## Deterministic Review-Script Blocker

The orchestrated `verify` stage ran the full review command list with `set -e`. In that run, commands through `cargo test -p myosu-games` completed successfully or returned clean filtered-test outputs. The first hard failure was:

```text
cargo test -p myosu-play
error: package ID specification `myosu-play` did not match any packages
```

Because the script stops on the first non-zero exit, later review commands were not reachable in that gate run.

Separate reproduction during this fixup confirms that `myosu-games-poker` is also absent from the current workspace snapshot and would fail for the same reason if reached.

## Current Workspace Constraint

The current workspace snapshot does not include:

- `myosu-play`
- `myosu-games-poker`

Those missing packages explain the deterministic review failure. This fixup does not create placeholder packages for them, because doing so would expand past the approved Slice 1 surface.

## Remaining Work Beyond Slice 1

- Slice 2: implement `game.rs`, `edge.rs`, `turn.rs`, and `info.rs`
- Slice 3: implement `encoder.rs`, `profile.rs`, and Nash verification tests
- Slice 4: add `ExploitMetric` registration to `myosu-games`
- Slice 5: add spectator relay surfaces for `myosu-play`
- Slice 6: add spectator TUI surfaces for `myosu-tui`
- Slice 7: run zero-change verification once the referenced packages exist in this workspace
