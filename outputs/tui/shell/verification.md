# `tui:shell` Verification — Slice 1

## Automated Proof Commands That Ran

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo test events:: --no-ignore` | 1 | Deterministic failure from the verify stage: `--no-ignore` is not a valid cargo argument without `--` |
| `cargo test shell:: --integration` | 1 | Deterministic failure waiting behind the first gate: `--integration` is not a valid cargo argument here |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-events cargo test -p myosu-tui schema::all_game_types` | 0 | Selected 0 tests; shorthand is not a real proof gate on this toolchain |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-events cargo test -p myosu-tui shell::shell_draw_` | 0 | Selected 0 tests; shorthand is not a real proof gate on this toolchain |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-events cargo test -p myosu-tui pipe::is_plain_text` | 0 | Selected 0 tests; shorthand is not a real proof gate on this toolchain |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui events:: -- --include-ignored` | 0 | 6 headless event-loop tests passed |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui shell::` | 0 | 11 shell tests passed unchanged |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui all_game_types_have_schema` | 0 | Existing schema smoke test passed |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui shell_draw_` | 0 | Existing shell draw tests passed |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell cargo test -p myosu-tui is_plain_text` | 0 | Existing ANSI-detection test passed |

## Command Notes

- The lane's prior proof-gate shorthands were human-readable but not
  automation-safe. Two failed argument parsing, and three returned success while
  exercising zero tests.
- [spec.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/spec.md)
  now records the exact cargo invocations/selectors that correspond to real
  tests in `myosu-tui`.
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target-tui-shell` was set because the
  default workspace target directory is not writable in this execution
  environment.

## Risks Reduced

- The `events.rs` CI blind spot is reduced: key, resize, tick, and update delivery are now proven headlessly.
- The lane no longer depends on a real TTY just to verify the event loop's core channel plumbing.
- The lane's proof contract is now executable in automation instead of relying
  on cargo shorthands that fail or silently do nothing.

## Risks Remaining

- `shell.rs` is still reopened in review because the shell input-to-screen transition chain remains unproven.
- `schema.rs` is still reopened in review because per-game semantic coverage remains incomplete.
- There is still no end-to-end `PipeMode::run_once()` async proof tied to the event loop.
- Slice 2's current proof surface still re-runs the existing `shell::` module;
  the dedicated Lobby-to-Game integration test has not been implemented yet.

## Next Approved Slice

**Slice 2 — Shell Integration Test**

Add a shell-level proof for `Lobby` input routing: simulate typing `1` and Enter, then assert the shell transitions into `Screen::Game`.
