# `tui:shell` Verification — Slice 1

## Automated Proof Commands

All commands were run from the workspace root. Because the default target directory resolves outside the writable sandbox in this run environment, Cargo commands were executed with `CARGO_TARGET_DIR=/tmp/myosu-target`.

| Command | Exit Code | Result |
|---------|-----------|--------|
| `cargo fmt --all` | 0 | Formatting passed |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui events::` | 0 | Focused `events` module proof passed: 6 tests |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui events:: -- --include-ignored` | 0 | Current Cargo-compatible equivalent of the spec gate; 6 tests passed and no ignored `events` tests remain |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui` | 0 | Full crate regression pass: 86 unit tests + 1 doctest passed |

## Proof Outcome

The reopened `events.rs` proof gap is reduced for Slice 1:

- headless tick delivery is proven
- headless key delivery is proven
- headless resize delivery is proven
- headless async update injection is proven

The original TTY-dependent ignored tests are no longer required for CI proof.

## Residual Risks

- The live terminal path still depends on `crossterm::event::EventStream` at runtime. This slice proves the shared event-loop logic headlessly, but it does not add an end-to-end live terminal harness.
- `shell.rs` remains reopened for its separate integration and render-proof gaps.
- `schema.rs` remains reopened for per-game coverage gaps.

## Next Approved Slice

**Slice 2 — Shell Integration Test**

Proof gate from the lane spec:

```bash
cargo test shell:: --integration
```
