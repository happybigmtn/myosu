# `tui:shell` Integration — Slice 1 Fixup

## Runtime Contract

The runtime contract consumed by the rest of `myosu-tui` is unchanged:

- `EventLoop::new(Duration) -> EventLoop`
- `EventLoop::next()`
- `EventLoop::update_sender()`

`shell.rs` and the rest of the TUI continue to consume the event loop exactly as before.

## Integration Effect

This fixup does not introduce a new integration behavior. It makes the current Slice 1 proof durable:

- production still reads terminal input from `crossterm::event::EventStream`
- tests still prove the same event-task path through the injected headless stream
- the lane-local Slice 1 proof gate now uses package-scoped Cargo semantics, so the proof stays inside `myosu-tui` instead of spilling into unrelated chain crates

## Surfaces Intentionally Not Touched

- `crates/myosu-tui/src/shell.rs`
- `crates/myosu-tui/src/screens.rs`
- `crates/myosu-tui/src/input.rs`
- `crates/myosu-tui/src/schema.rs`
- `crates/myosu-tui/src/pipe.rs`

## Verification Signal

`CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui` passed after the artifact fixup, which indicates the existing `events.rs` implementation continues to integrate cleanly with neighboring TUI surfaces.

The active run's external verify graph still points at stale workspace-root
commands outside this worktree, so the remaining blocker is orchestration drift
rather than a `tui:shell` integration defect.
