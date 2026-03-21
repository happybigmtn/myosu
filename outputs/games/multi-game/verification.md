# `games:multi-game` Verification — Slice 2

## Automated Proof Commands

The workspace default Cargo target directory is read-only in this sandbox, so
the slice-2 proof commands were run with
`CARGO_TARGET_DIR=/tmp/myosu-multi-game-target`.

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo build -p myosu-games-liars-dice` | 101 | Blocked by the sandbox default target directory: Cargo could not open the default lockfile under `/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/`. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo build -p myosu-games-liars-dice` | 0 | Passed. The slice-2 game-engine crate builds cleanly. |
| `cargo test -p myosu-games-liars-dice` | 101 | Blocked by the same read-only default target directory. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice` | 0 | Passed. All crate tests succeeded, including the new slice-2 game-engine proofs. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice game::tests::root_is_chance_node` | 0 | Passed. Confirms the root state is a chance node with roll branches. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice game::tests::legal_bids_increase` | 0 | Passed. Confirms bids advance monotonically and challenge unlocks only after the opening bid. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice game::tests::challenge_resolves_game` | 0 | Passed. Confirms a challenge transitions directly to terminal resolution. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice game::tests::payoff_is_zero_sum` | 0 | Passed. Confirms the terminal utility sum is zero. |
| `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice game::tests::all_trait_bounds_satisfied` | 0 | Passed. Confirms the new Liar's Dice types satisfy the robopoker trait bounds. |
| `cargo fmt --all` | 0 | Passed. Formatting completed after the code changes. |

## Observed Test Output

```text
error: failed to open: /home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/.cargo-lock

Caused by:
  Read-only file system (os error 30)
```

```text
running 6 tests
test game::tests::all_trait_bounds_satisfied ... ok
test game::tests::challenge_resolves_game ... ok
test game::tests::legal_bids_increase ... ok
test game::tests::payoff_is_zero_sum ... ok
test game::tests::root_is_chance_node ... ok
test tests::public_api_stubs_exist ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The active slice's first proof gate is green once Cargo writes to a
sandbox-writable target directory. `myosu-games-liars-dice` builds, the
fixed-size `Copy`-safe game state works, and all review-approved slice-2 checks
pass.

## Risks Reduced

- **`CfrGame: Copy` blocker:** Reduced. Bid history now uses a fixed-size
  sentinel-backed array instead of a heap allocation.
- **Chance-node wiring risk:** Reduced. The root state exposes all 36 dice-roll
  outcomes through the info-set choice surface.
- **Terminal scoring ambiguity:** Reduced. Challenge resolution and zero-sum
  payoff behavior are now covered by explicit tests.
- **Trait-compatibility risk:** Reduced. The game, edge, turn, and info types
  compile against the robopoker CFR trait bounds.

## Risks That Remain

- **No encoder/profile yet:** Unchanged. Slice 3 still needs to implement
  `LiarsDiceEncoder`, `LiarsDiceProfile`, and the Nash proof tests.
- **No cross-game metric registration yet:** Unchanged. `ExploitMetric` work is
  still slice 4.
- **No spectator surfaces yet:** Unchanged. `myosu-play` and the spectator TUI
  remain later slices.
- **Later proof-script lines remain outside this slice:** Unchanged. Commands
  that reference packages not yet present in this checkout, such as
  `myosu-play` or `myosu-games-poker`, are still blocked outside slice 2.

## Next Approved Slice

**Slice 3 — `encoder.rs` + `profile.rs`: solver and Nash verification**

Approved next proof commands:

```bash
cargo test -p myosu-games-liars-dice solver::tests::train_to_nash
cargo test -p myosu-games-liars-dice solver::tests::exploitability_near_zero
cargo test -p myosu-games-liars-dice solver::tests::strategy_is_nontrivial
cargo test -p myosu-games-liars-dice solver::tests::wire_serialization_works
```
