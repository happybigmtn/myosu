# `games:multi-game` Verification — Slice 1

## Automated Proof Commands

The default shared cargo target path in this sandbox is read-only, so the proof
commands were executed with `CARGO_TARGET_DIR=/tmp/myosu-multi-game-target`.

| Command | Exit Code | Result |
|---------|-----------|--------|
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo build -p myosu-games-liars-dice` | 0 | Passed. New workspace member compiled successfully. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice` | 0 | Passed. 1 unit test passed; 0 doctests ran. |

## Observed Test Output

```text
running 1 test
test tests::public_api_stubs_exist ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Doc-tests myosu_games_liars_dice
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Risks Reduced

- **Greenfield crate blocker:** Reduced. `crates/myosu-games-liars-dice/` now exists and is wired into the workspace.
- **Dependency pin drift:** Reduced. The new crate uses the same robopoker revision as `myosu-games`, avoiding an accidental fork mismatch at slice start.
- **Lane boundary drift:** Reduced. The slice stayed within the approved `games:multi-game` surfaces and did not start later-slice work.

## Risks That Remain

- **No CFR implementation yet:** Unchanged. `LiarsDiceGame`, `LiarsDiceEdge`, `LiarsDiceTurn`, and `LiarsDiceInfo` are placeholders only.
- **`CfrGame: Copy` implementation risk:** Unchanged. Slice 2 still needs to prove the fixed-size bid history approach works.
- **Cross-game scoring and spectator work:** Unchanged. `ExploitMetric`, `SpectatorRelay`, and spectator TUI surfaces are still absent.

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
