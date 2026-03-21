# `tui:shell` Implementation — Slice 1

## Slice Implemented

**Slice 1 — Event Loop Headless Test**

Implemented the next approved `tui:shell` slice from [spec.md](/home/r/.fabro/runs/20260320-01KM755GFB9SS6HFPKZYFWVJDP/worktree/outputs/tui/shell/spec.md): make the event loop provable in CI without a real TTY.

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

## Scope Guard

- No changes were made to `shell.rs`, `schema.rs`, `pipe.rs`, `renderer.rs`, or the `GameRenderer` contract.
- No lane expansion was performed beyond the approved `events.rs` slice.

## Remaining Approved Slices

| Slice | Status | Notes |
|-------|--------|-------|
| Slice 2 — Shell Integration Test | Pending | `Lobby` input should prove `handle_key` -> `handle_submit` -> `screens.apply_command` |
| Slice 3 — Schema Per-Game Coverage | Pending | `schema.rs` remains reopened in review |
| Slice 4 — Screen Render Tests | Pending | Only `Game` and too-small render paths are currently proven |
| Slice 5 — Pipe Mode ANSI Enforcement | Pending | Optional caveat remains in review |
