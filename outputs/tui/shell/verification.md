# `tui:shell` Verification — Slice 1

## Automated Proof Commands

| Command | Exit Code | Result |
|---------|-----------|--------|
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events:: -- --include-ignored` | 0 | 6 headless event-loop tests passed |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui shell::` | 0 | 11 shell tests passed unchanged |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui all_game_types_have_schema` | 0 | Existing schema smoke test passed |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui shell_draw_` | 0 | Existing shell draw tests passed |
| `CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui is_plain_text` | 0 | Existing ANSI-detection test passed |

## Command Notes

- The lane review/spec shorthand `cargo test events:: --no-ignore` is not accepted by this Cargo toolchain. The cargo-compatible equivalent is `cargo test ... -- --include-ignored`.
- The preflight shorthand `cargo test shell:: --integration` does not map to a real Cargo flag or a dedicated integration target in `myosu-tui`; the closest implemented proof surface is the `shell::` test module itself.
- `CARGO_TARGET_DIR=/tmp/myosu-cargo-target` was set because the default workspace target directory is not writable in this execution environment.

## Risks Reduced

- The `events.rs` CI blind spot is reduced: key, resize, tick, and update delivery are now proven headlessly.
- The lane no longer depends on a real TTY just to verify the event loop's core channel plumbing.

## Risks Remaining

- `shell.rs` is still reopened in review because the shell input-to-screen transition chain remains unproven.
- `schema.rs` is still reopened in review because per-game semantic coverage remains incomplete.
- There is still no end-to-end `PipeMode::run_once()` async proof tied to the event loop.

## Next Approved Slice

**Slice 2 — Shell Integration Test**

Add a shell-level proof for `Lobby` input routing: simulate typing `1` and Enter, then assert the shell transitions into `Screen::Game`.
