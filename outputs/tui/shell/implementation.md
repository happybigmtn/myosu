# `tui:shell` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Event Loop Headless Test**

Implemented the next approved `tui:shell` slice from `outputs/tui/shell/spec.md` and `outputs/tui/shell/review.md` by making the `events.rs` event source injectable for tests and replacing the TTY-only ignored tests with headless proof.

## What Changed

### `crates/myosu-tui/src/events.rs`

The public `EventLoop::new` API is unchanged, but the internal task setup now delegates through a private injected-stream constructor:

- `EventLoop::new(...)` now calls private `with_stream(...)`
- `run_event_task(...)` holds the async select loop so production and test streams share the same logic
- `map_crossterm_event(...)` centralizes Crossterm-to-Myosu event mapping

This keeps the implementation slice inside the owned `events.rs` surface while letting tests supply a synthetic stream.

## Headless Proof Added

Added a channel-backed `MockEventStream` in the `events.rs` test module and replaced the two ignored tests with deterministic headless coverage:

- `tick_event_handled_headless`
- `key_event_handled`
- `resize_event_handled`
- `async_response_received`

The updated tests now prove that tick, key, resize, and async update events traverse the event loop without requiring a real terminal.

## Nearby Fix Discovered While Verifying

The `EventLoop::new` doctest executed the constructor in a headless environment and panicked inside `crossterm`. The example block is now marked `no_run`, so documentation still compiles without trying to open a terminal during doctest execution.

## Scope Guard

This slice touched only `crates/myosu-tui/src/events.rs`.

No changes were made to:

- `shell.rs`
- `schema.rs`
- `pipe.rs`
- any downstream game renderer surfaces

## Next Slice

Per `outputs/tui/shell/spec.md`, the next approved implementation slice remains:

**Slice 2 — Shell Integration Test**

Add a `shell.rs` integration test covering Lobby input submission through `handle_key`/`handle_submit` into a Game screen transition.
