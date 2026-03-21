# `tui:shell` Integration — Slice 1 Fixup

## Integration Contract Preserved

This slice keeps the `tui:shell` integration boundary stable:

- `Shell::run()` still constructs the event loop through `EventLoop::new(...)`
- `EventLoop::next()` and `EventLoop::update_sender()` are unchanged
- downstream renderers and screen logic do not need any code changes

The only integration change in the original Slice 1 code was internal:
`events.rs` can now be exercised with a synthetic event stream in tests.

This fixup adds no new runtime integration behavior. It only clarifies which
proof command is live for the current implementation slice and records that the
legacy `cargo test events:: --no-ignore` failure comes from stale verifier
state rather than the current lane-owned runtime surfaces.

## Upstream and Downstream Impact

- **Upstream terminal driver:** unchanged in production. `crossterm::event::EventStream::new()` is still the live source used by `EventLoop::new()`.
- **Downstream shell consumer:** unchanged. `shell.rs` continues to react to `Event::Tick`, `Event::Key`, `Event::Resize`, and `Event::Update` the same way.
- **Background tasks:** unchanged. Cloned `update_sender()` handles are still the way solver/network tasks inject updates into the UI loop.
- **Curated proof consumers:** clarified. `outputs/tui/shell/spec.md` now marks
  only the active Slice 1 command as `**Proof gate**`; future slices stay
  planned until their code is approved to land.
- **Legacy verifier command:** isolated. The stale `--no-ignore` invocation is
  not part of the current shell/runtime integration contract and cannot be
  repaired from `events.rs`.

## Slice Boundary Check

This implementation stayed within the approved smallest slice:

- source edits from Slice 1: `crates/myosu-tui/src/events.rs`
- curated artifact fixup edits: `spec.md`, `implementation.md`, `verification.md`, `integration.md`
- intentionally not authored in this stage: `quality.md`, `promotion.md`

## Follow-On Integration Work

The next slice should validate the next open seam in the lane:

- shell input handling
- shell command submission
- `ScreenManager` routing from Lobby to Game

That follow-on work belongs in `crates/myosu-tui/src/shell.rs` per the reviewed lane contract.
