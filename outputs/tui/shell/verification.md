# `tui:shell` Verification — Slice 1 Fixup

## Automated Proof Commands

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo test events:: --no-ignore` | 1 | The command in `spec.md` is not valid Cargo syntax. Cargo rejects `--no-ignore` unless test args are passed after `--`. |
| `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui events::` | 0 | Passed. The live Slice 1 proof gate succeeded with 4 `events::` tests. |
| `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui schema::tests::all_game_types_have_schema` | 0 | Passed. The planned Slice 3 selector resolves to 1 real test once activated. |
| `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui shell::tests::shell_draw_` | 0 | Passed. The planned Slice 4 selector resolves to 2 existing render tests once activated. |
| `env CARGO_TARGET_DIR=/tmp/myosu-cargo-target cargo test -p myosu-tui pipe::tests::is_plain_text_detects_ansi` | 0 | Passed. The planned Slice 5 selector resolves to 1 real test once activated. |

Slice 2 still has no approved code change in this fixup, so its planned proof
gate remains dormant and was not run.

## Passing Test Summary

The active Slice 1 proof suite now passes headlessly:

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

- Cargo in this environment defaults to a read-only target directory outside
  the writable run sandbox. Verification used
  `CARGO_TARGET_DIR=/tmp/myosu-cargo-target` to keep proof execution runnable.
- `outputs/tui/shell/spec.md` now exposes the active Slice 1 command as the only
  live `**Proof gate**`. Later slices keep `**Planned proof gate**` labels until
  they are selected, which prevents the implementation verifier from running
  future-slice commands too early.
- The planned Slice 3/4/5 commands were spot-checked with fully qualified test
  selectors so those gates no longer rely on zero-match filters when they are
  activated later.

## Risks Reduced

- **TTY-only proof gap:** Reduced. The reopened `events.rs` proof now runs in
  headless automation.
- **Unproven async delivery path:** Reduced. Tick, key, resize, and async
  update delivery all traverse the same event-loop channel in CI.
- **Deterministic proof-script failure:** Reduced. The live proof gate now uses
  valid Cargo syntax and the sandbox-safe target directory override.

## Risks Remaining

- **Production terminal integration:** Still partially unproven. This slice
  verifies the event-loop logic with a synthetic stream, not a real terminal
  session.
- **End-to-end shell integration:** Still open. The `shell.rs` input-routing
  chain remains the next approved proof gap.
- **Schema/render/pipe claims:** Still open at the lane level. This fixup only
  repaired proof activation; it did not implement Slice 2-5.

## Next Approved Slice

**Slice 2 — Shell Integration Test**

Add the approved `shell.rs` integration test for Lobby input routing:

- start in `Screen::Lobby`
- type `"1"`
- submit with Enter
- verify transition to `Screen::Game`
