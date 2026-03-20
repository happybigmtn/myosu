# Slice 2 Verification

## Command Results

| Command | Exit | Result |
|---|---:|---|
| `CARGO_TARGET_DIR=/tmp/myosu-games-poker-build-clean cargo build -p myosu-games-poker` | 0 | PASS |
| `CARGO_TARGET_DIR=/tmp/myosu-games-poker-test-clean cargo test -p myosu-games-poker solver::tests::create_empty_solver -- --nocapture` | 0 | PASS |
| `CARGO_TARGET_DIR=/tmp/myosu-games-poker-test-clean cargo test -p myosu-games-poker solver::tests::strategy_is_valid_distribution -- --nocapture` | 0 | PASS |
| `CARGO_TARGET_DIR=/tmp/myosu-games-poker-test-clean cargo test -p myosu-games-poker solver::tests::checkpoint_roundtrip -- --nocapture` | 0 | PASS |
| `CARGO_TARGET_DIR=/tmp/myosu-games-poker-suite-clean cargo test -p myosu-games-poker -- --nocapture` | 0 | PASS (`5 passed`, `2 ignored`) |

## Proven in This Slice

- `myosu-games-poker` builds cleanly on the narrowed Slice 2 surface.
- `PokerSolver::new(encoder)` constructs an empty solver with `epochs() == 0`.
- `PokerSolver::strategy()` returns a valid probability distribution for an `NlheInfo`.
- `PokerSolver::save()` / `PokerSolver::load(path, encoder)` roundtrip the profile state under the `MYOS` framing.
- Invalid checkpoint magic and unsupported checkpoint versions are rejected with explicit errors.

## Full Suite Output

```text
running 7 tests
test solver::tests::exploitability_decreases ... ignored, requires a populated abstraction artifact / RF-02 to measure exploitability honestly
test solver::tests::create_empty_solver ... ok
test solver::tests::train_100_iterations ... ignored, requires a populated abstraction artifact / RF-02 to train honestly
test solver::tests::strategy_is_valid_distribution ... ok
test solver::tests::rejects_invalid_checkpoint_magic ... ok
test solver::tests::rejects_unsupported_checkpoint_version ... ok
test solver::tests::checkpoint_roundtrip ... ok

test result: ok. 5 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
```

## Remaining Proof Gap

- `solver::tests::train_100_iterations` is intentionally ignored.
- `solver::tests::exploitability_decreases` is intentionally ignored.
- Both remain blocked on the reviewed encoder/artifact prerequisite: a populated abstraction lookup (`RF-02` plus a usable artifact).

## Environment Note

- Verification used explicit `CARGO_TARGET_DIR=/tmp/...` overrides because the default shared Cargo target path in this sandbox is not writable.
