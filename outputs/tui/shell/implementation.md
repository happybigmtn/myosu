# `tui:shell` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Event Loop Headless Test**

Implemented the approved `events.rs` slice from [`outputs/tui/shell/spec.md`](./spec.md): the event loop can now be proven headlessly in CI without requiring a real TTY.

## Owned Surface

- `crates/myosu-tui/src/events.rs`

No other lane-owned source files were changed.

## What Changed

### Internal event-loop construction is now injectable for tests

`EventLoop::new(tick_rate)` remains the production entrypoint and still constructs a real `crossterm::event::EventStream`.

The event-task setup was moved behind an internal `from_stream(...)` constructor so tests can drive the exact same event-loop logic with a synthetic stream. This keeps the public API unchanged while making the async bridge testable.

### Added a headless mock event stream

The test module now defines `MockEventStream`, backed by a Tokio unbounded channel. It implements `futures::Stream<Item = io::Result<CrosstermEvent>>`, which lets tests inject synthetic:

- key events
- resize events
- async update events
- tick delivery alongside the injected events

### Replaced TTY-only proof with headless proof

The two ignored tests that depended on a real terminal were replaced with headless async tests:

- `headless_stream_delivers_tick_key_resize_and_update`
- `async_response_received`

`update_sender_cloned` was also updated to use the headless harness so the `events::` suite no longer depends on terminal availability.

## Behavioral Outcome

This slice proves that the `EventLoop` channel bridge delivers the critical event classes required by the shell:

- periodic ticks
- crossterm key input
- crossterm resize notifications
- injected async updates from background tasks

The production shell still calls `EventLoop::new()` exactly as before; this slice changes proofability, not the caller contract.

## Stage Ownership Note

Per lane instructions, this implementation stage does **not** hand-author:

- `outputs/tui/shell/quality.md`
- `outputs/tui/shell/promotion.md`

Those artifacts remain owned by the Quality Gate and Review stages.
