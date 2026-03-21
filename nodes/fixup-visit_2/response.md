**Fixup complete.**

**Root cause**: The spec.md defined the proof gate as `cargo test events:: --no-ignore` where `--no-ignore` is an invalid `cargo test` argument (it's a test binary flag that goes after `--`).

**Fix applied**: Updated spec.md line 127 to use the correct command: `cargo test -p myosu-tui events::`

**Evidence**: Tests pass with 5/5 events tests passing:
```
cargo test -p myosu-tui events::
  test events::tests::update_event_variants ... ok
  test events::tests::update_sender_cloned ... ok
  test events::tests::tick_events_produced ... ok
  test events::tests::injected_update_received ... ok
  test events::tests::synthetic_events_traverse_channel ... ok
test result: ok. 5 passed; 0 failed; 0 ignored
```

The verification.md was already correct and now aligns with the fixed spec.md.