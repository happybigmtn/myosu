# `tui:shell` Verification — Slice 1

## Automated Proof Commands

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo test events:: --no-ignore` | 1 | The command in `spec.md` is not valid Cargo syntax. Cargo rejects `--no-ignore` unless test args are passed after `--`. |
| `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events::` | 0 | Passed. 4 `events::` tests succeeded. |
| `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events:: -- --include-ignored` | 0 | Passed. 4 `events::` tests succeeded; no ignored tests remained. |

## Passing Test Summary

The verified `events::` suite now passes headlessly:

- `events::tests::headless_stream_delivers_tick_key_resize_and_update`
- `events::tests::async_response_received`
- `events::tests::update_sender_cloned`
- `events::tests::update_event_variants`

Observed result from the proof run:

```text
running 4 tests
test events::tests::update_event_variants ... ok
test events::tests::update_sender_cloned ... ok
test events::tests::async_response_received ... ok
test events::tests::headless_stream_delivers_tick_key_resize_and_update ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

## Verification Notes

- Cargo in this environment was configured to write build artifacts to a read-only path outside the workspace. Verification used `CARGO_TARGET_DIR=/tmp/myosu-cargo-target` to keep the proof runnable without mutating repository config.
- The spec gate string should be corrected in a future maintenance pass to a valid form such as `cargo test -p myosu-tui events:: -- --include-ignored`.

## Risks Reduced

- **TTY-only proof gap:** Reduced. The reopened `events.rs` proof now runs in headless automation.
- **Unproven async delivery path:** Reduced. Tick, key, resize, and async update delivery all traverse the same event-loop channel in CI.

## Risks Remaining

- **Production terminal integration:** Still partially unproven. This slice verifies the event-loop logic with a synthetic stream, not a real terminal session.
- **End-to-end shell integration:** Still open. The `shell.rs` input-routing chain remains the next approved proof gap.

## Next Approved Slice

**Slice 2 — Shell Integration Test**

Add the approved `shell.rs` integration test for Lobby input routing:

- start in `Screen::Lobby`
- type `"1"`
- submit with Enter
- verify transition to `Screen::Game`
