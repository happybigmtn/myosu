# `tui:shell` Implementation — Slice 1 Fixup

## Slice Implemented

**Slice 1 — Event Loop Headless Test**

The approved `events.rs` slice from [`outputs/tui/shell/spec.md`](./spec.md)
was already present at the start of this fixup: the event loop can be proven
headlessly in CI without requiring a real TTY.

No additional Rust source changes were required in this turn. The fixup work
stays inside the current slice by refreshing the curated implementation records
so they describe the active proof gate truthfully and do not misattribute the
stale `cargo test events:: --no-ignore` failure to the current `spec.md`.

## Lane Surfaces Considered

- `crates/myosu-tui/src/events.rs`
- `outputs/tui/shell/spec.md`
- `outputs/tui/shell/implementation.md`
- `outputs/tui/shell/verification.md`
- `outputs/tui/shell/integration.md`

Only the curated implementation artifacts were updated in this fixup. The
lane-owned source surface in `events.rs` was revalidated but not edited.

## What Changed

### Slice code remains the same

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

### Proof activation remains slice-scoped

`outputs/tui/shell/spec.md` now exposes only one live `**Proof gate**`, for the
currently approved Slice 1. Later slices keep `**Planned proof gate**` labels
until they are explicitly selected.

The active command shape remains:

```bash
env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events::
```

The `CARGO_TARGET_DIR` override is required in this workspace because the
default Cargo target path points at a read-only location outside the writable
run sandbox.

The legacy `cargo test events:: --no-ignore` invocation is now treated as a
stale verifier command from earlier stage logs, not as the current live proof
gate for this slice.

## Behavioral Outcome

This slice proves that the `EventLoop` channel bridge delivers the critical event classes required by the shell:

- periodic ticks
- crossterm key input
- crossterm resize notifications
- injected async updates from background tasks

The production shell still calls `EventLoop::new()` exactly as before; this slice changes proofability, not the caller contract.

## Proof Command for This Fixup

```bash
env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events::
```

## What Remains for the Next Slice

**Slice 2 — Shell Integration Test**

The next approved code change remains in `crates/myosu-tui/src/shell.rs`:

- start in `Screen::Lobby`
- type `"1"`
- submit with Enter
- verify the shell transitions to `Screen::Game`

## Stage Ownership Note

Per lane instructions, this implementation stage does **not** hand-author:

- `outputs/tui/shell/quality.md`
- `outputs/tui/shell/promotion.md`

Those artifacts remain owned by the Quality Gate and Review stages.
