# `tui:shell` Integration — Slice 1

## Contract Preserved

This slice preserves the runtime contract consumed by the rest of `myosu-tui`:

- `EventLoop::new(Duration) -> EventLoop` is unchanged
- `EventLoop::next()` is unchanged
- `EventLoop::update_sender()` is unchanged

`shell.rs` continues to construct the event loop exactly as before, with no call-site changes required.

## Integration Effect

The only integration change is internal testability:

- production still uses `crossterm::event::EventStream`
- tests can now inject a mock stream through the same event-task logic
- the shell-facing event contract is now proven headlessly for tick, key, resize, and update events

## Surfaces Intentionally Not Touched

- `crates/myosu-tui/src/shell.rs`
- `crates/myosu-tui/src/screens.rs`
- `crates/myosu-tui/src/input.rs`
- `crates/myosu-tui/src/schema.rs`
- `crates/myosu-tui/src/pipe.rs`

## Verification Signal

The full `myosu-tui` crate test pass after this slice indicates the internal `events.rs` refactor did not break neighboring TUI surfaces or their current contracts.
