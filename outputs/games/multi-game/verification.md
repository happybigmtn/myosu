# `games:multi-game` Verification — Slice 1

## Automated Proof Commands

The workspace default Cargo target path is read-only in this sandbox, so the
slice proof commands had to be rerun with
`CARGO_TARGET_DIR=/tmp/myosu-multi-game-target`.

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo build -p myosu-games-liars-dice` | 101 | Blocked by sandbox filesystem: Cargo could not open the default target-dir lockfile under `/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/`. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo build -p myosu-games-liars-dice` | 0 | Passed. New workspace member compiled successfully. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice` | 0 | Passed. 1 unit test passed; 0 doctests ran. |
| `cargo metadata --no-deps --format-version 1` | 0 | Passed. Workspace members in this checkout are `myosu-games`, `myosu-games-liars-dice`, `myosu-tui`, and `pallet-game-solver`. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-play` | 101 | Confirms the verify-stage blocker is outside slice 1: `myosu-play` is not a package in this checkout yet. |

## Observed Test Output

```text
running 1 test
test tests::public_api_stubs_exist ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Doc-tests myosu_games_liars_dice
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```text
error: failed to open: /home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/.cargo-lock

Caused by:
  Read-only file system (os error 30)
```

```text
error: package ID specification `myosu-play` did not match any packages
```

## Interpretation

Slice 1's own proof gate is green once Cargo writes to a sandbox-writable target
directory. The deterministic verify-stage failure came from the runner
continuing into later or absent package surfaces, not from the
`myosu-games-liars-dice` crate skeleton added in this slice.

## Risks Reduced

- **Greenfield crate blocker:** Reduced. `crates/myosu-games-liars-dice/` now exists and is wired into the workspace.
- **Dependency pin drift:** Reduced. The new crate uses the same robopoker revision as `myosu-games`, avoiding an accidental fork mismatch at slice start.
- **Lane boundary drift:** Reduced. The slice stayed within the approved `games:multi-game` surfaces and did not start later-slice work.
- **Proof reproducibility in this sandbox:** Reduced. The artifact now records the required `CARGO_TARGET_DIR` override instead of implying the default target path is usable here.

## Risks That Remain

- **No CFR implementation yet:** Unchanged. `LiarsDiceGame`, `LiarsDiceEdge`, `LiarsDiceTurn`, and `LiarsDiceInfo` are placeholders only.
- **`CfrGame: Copy` implementation risk:** Unchanged. Slice 2 still needs to prove the fixed-size bid history approach works.
- **Cross-game scoring and spectator work:** Unchanged. `ExploitMetric`, `SpectatorRelay`, and spectator TUI surfaces are still absent.
- **Later package surfaces absent from this checkout:** Unchanged. `myosu-play` and `myosu-games-poker` proof lines from the review artifact cannot run until those packages exist in the workspace.

## Next Approved Slice

**Slice 2 — Liar's Dice game engine**

Add `game.rs`, `edge.rs`, `turn.rs`, and `info.rs`, then satisfy the approved
proof commands:

```bash
cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node
cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase
cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game
cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum
cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied
```
