# `play:tui` Verification Artifact

## Proof Commands and Outcomes

This verification used a writable local target directory because the shared default target path in this run environment is read-only.

### Proof 1: `myosu-play` builds with the training slice wired in

```bash
CARGO_TARGET_DIR=/tmp/myosu-target CARGO_NET_OFFLINE=true cargo build -p myosu-play
```

**Outcome**: exit 0. The build completed successfully with the new training dependencies (`rbp-cards`, `rbp-gameplay`, `rand`) and the `TrainingTable` entry wiring in place.

### Proof 2: all training-slice tests pass together

```bash
CARGO_TARGET_DIR=/tmp/myosu-target CARGO_NET_OFFLINE=true cargo test -p myosu-play training::tests
```

**Outcome**: exit 0.

Covered tests:

- `training::tests::hand_completes_fold`
- `training::tests::hand_completes_showdown`
- `training::tests::deal_command_sets_cards`
- `training::tests::bot_backend_fallback`
- `training::tests::practice_chips_update`
- `training::tests::alternating_button`

Result summary:

```text
running 6 tests
test training::tests::bot_backend_fallback ... ok
test training::tests::alternating_button ... ok
test training::tests::hand_completes_fold ... ok
test training::tests::deal_command_sets_cards ... ok
test training::tests::practice_chips_update ... ok
test training::tests::hand_completes_showdown ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Slice-3 Gate Coverage

| Gate | Command Coverage | Status |
|------|------------------|--------|
| Training hand can end on a fold | `training::tests::hand_completes_fold` | Passed |
| Training hand can reach showdown | `training::tests::hand_completes_showdown` | Passed |
| `/deal` applies to the next hand | `training::tests::deal_command_sets_cards` | Passed |
| Bot fallback state is surfaced | `training::tests::bot_backend_fallback` | Passed |
| Practice chips update after a hand | `training::tests::practice_chips_update` | Passed |
| Button alternates across hands | `training::tests::alternating_button` | Passed |
| Binary still builds with the new slice wired in | `cargo build -p myosu-play` | Passed |

## Notes From This Verification Run

- The spec examples for `/deal` and `/board` use Unicode suits. The automated proof now confirms that the command path accepts those examples by normalizing them before handing the cards to robopoker’s parser.
- Stack override commands were intentionally left behind a hard error path. That behavior was not promoted to an automated pass condition because the underlying robopoker `Game` type still lacks public stack setters.
- The shell loop itself was not the focus of this gate. This slice proves the training engine and the entry wiring that snapshots its current state into the existing renderer.
