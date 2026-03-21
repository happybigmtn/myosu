# `tui:shell` Integration — Slice 1

## Integration Contract Preserved

This slice keeps the `tui:shell` integration boundary stable:

- `Shell::run()` still constructs the event loop through `EventLoop::new(...)`
- `EventLoop::next()` and `EventLoop::update_sender()` are unchanged
- downstream renderers and screen logic do not need any code changes

The only integration change is internal: `events.rs` can now be exercised with a synthetic event stream in tests.

## Upstream and Downstream Impact

- **Upstream terminal driver:** unchanged in production. `crossterm::event::EventStream::new()` is still the live source used by `EventLoop::new()`.
- **Downstream shell consumer:** unchanged. `shell.rs` continues to react to `Event::Tick`, `Event::Key`, `Event::Resize`, and `Event::Update` the same way.
- **Background tasks:** unchanged. Cloned `update_sender()` handles are still the way solver/network tasks inject updates into the UI loop.

## Slice Boundary Check

This implementation stayed within the approved smallest slice:

- source edits: `crates/myosu-tui/src/events.rs`
- curated artifacts added: `implementation.md`, `verification.md`, `integration.md`
- intentionally not authored in this stage: `quality.md`, `promotion.md`

## Follow-On Integration Work

The next slice should validate the next open seam in the lane:

- shell input handling
- shell command submission
- `ScreenManager` routing from Lobby to Game

That follow-on work belongs in `crates/myosu-tui/src/shell.rs` per the reviewed lane contract.
