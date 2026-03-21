# `tui:shell` Implementation — Slice 1 Fixup

## Active Slice

**Slice 1 — Event Loop Headless Test** remains the active approved slice from `outputs/tui/shell/spec.md` and `outputs/tui/shell/review.md`.

The `events.rs` implementation for this slice was already present at the start of fixup and was not expanded further in this turn.

## Fix Applied

The failing verify stage did not uncover a new `events.rs` defect. It failed before running tests because the recorded Slice 1 proof gate used invalid Cargo syntax:

- before: `cargo test events:: --no-ignore`
- after: `cargo test events:: -- --include-ignored`

This preserves the original proof intent for Slice 1 while matching actual Cargo argument parsing.

## Implementation Status Preserved

The landed Slice 1 code in `crates/myosu-tui/src/events.rs` remains the same:

- `EventLoop::new(...)` keeps its public API
- private `with_stream(...)` allows tests to inject a synthetic event source
- `run_event_task(...)` continues to own the shared async select loop
- `map_crossterm_event(...)` continues to centralize Crossterm-to-Myosu mapping
- headless tests still cover tick, key, resize, and async update delivery

## Scope Guard

This fixup stayed within the current slice and its lane artifacts:

- updated `outputs/tui/shell/spec.md`
- refreshed `outputs/tui/shell/implementation.md`
- refreshed `outputs/tui/shell/verification.md`
- refreshed `outputs/tui/shell/integration.md`

No new runtime behavior was introduced, and no work was advanced into `shell.rs`, `schema.rs`, `pipe.rs`, or downstream renderer surfaces.
