# `tui:shell` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Event Loop Headless Test**

Implemented the next approved `tui:shell` slice from [spec.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/spec.md): make the event loop provable in CI without a real TTY.

## Fixup Summary

This fixup did not expand the Rust implementation beyond Slice 1. The
`events.rs` headless proof work remains the shipped code change; this pass
corrects the lane's proof contract so automation uses runnable cargo commands
and selectors instead of shorthand that either fails argument parsing or
selects zero tests.

## What Changed

### `crates/myosu-tui/src/events.rs`

- Kept the public `EventLoop::new(tick_rate)` API unchanged for production callers.
- Added an internal `from_stream(...)` seam so tests can inject synthetic `crossterm` events instead of constructing a real terminal-backed `EventStream`.
- Boxed the reader behind a single internal `TerminalEventStream` alias to keep the spawned task logic unchanged apart from the injectable source.
- Extracted `map_crossterm_event(...)` so the filter behavior for focus, mouse, and paste events remains explicit and shared.
- Replaced the previous ignored TTY-only tests with headless tokio tests covering:
  - tick delivery
  - key delivery
  - resize delivery
  - async update delivery
  - cloned update senders

### `outputs/tui/shell/spec.md`

- Normalized each slice proof gate to the exact cargo invocation that this
  toolchain can execute.
- Recorded the `/tmp` target-dir override needed in Fabro's sandbox because the
  default workspace target directory is read-only here.
- Tightened the proof selectors so schema, shell draw, and pipe checks target
  the real test names instead of zero-test shorthands.

### `outputs/tui/shell/verification.md` and `outputs/tui/shell/integration.md`

- Updated the curated artifacts to record the deterministic verification
  failure, the cargo-compatible proof sequence that passed, and the preserved
  integration contract for the lane.

## Scope Guard

- No further changes were made to `events.rs` during fixup.
- No changes were made to `shell.rs`, `schema.rs`, `pipe.rs`, `renderer.rs`, or the `GameRenderer` contract.
- No lane expansion was performed beyond the approved `events.rs` slice.

## Remaining Approved Slices

| Slice | Status | Notes |
|-------|--------|-------|
| Slice 2 — Shell Integration Test | Pending | `Lobby` input should prove `handle_key` -> `handle_submit` -> `screens.apply_command` |
| Slice 3 — Schema Per-Game Coverage | Pending | `schema.rs` remains reopened in review |
| Slice 4 — Screen Render Tests | Pending | Only `Game` and too-small render paths are currently proven |
| Slice 5 — Pipe Mode ANSI Enforcement | Pending | Optional caveat remains in review |
