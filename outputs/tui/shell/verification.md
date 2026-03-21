# `tui:shell` Verification — Slice 1 Fixup

## Verification Context

The prior verify-stage failure was deterministic and happened before any `events` tests ran. The recorded command `cargo test events:: --no-ignore` is not valid Cargo CLI syntax.

All commands below were rerun from the workspace root. In this sandbox, Cargo needs `CARGO_TARGET_DIR=/tmp/myosu-target` because the default target path resolves outside the writable root.

## Automated Proof Commands

| Command | Exit Code | Result |
|---------|-----------|--------|
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui events::` | 0 | Focused `events` proof passed: 6 tests, 0 ignored |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui events:: -- --include-ignored` | 0 | Cargo-compatible Slice 1 proof gate passed: same 6 `events` tests, with no ignored tests remaining |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui` | 0 | Full crate regression passed: 86 unit tests and 1 doctest |

## Proof Outcome

Slice 1 is verified in its current form:

- tick delivery is proven headlessly
- key delivery is proven headlessly
- resize delivery is proven headlessly
- async update injection is proven headlessly
- the doctest for `EventLoop::new` compiles cleanly in headless verification

## Residual Risks

- This slice proves the shared event-loop logic without a live terminal harness; runtime use still depends on `crossterm::event::EventStream`.
- `shell.rs` remains reopened for the separate transition and render coverage gaps called out in review.
- `schema.rs` remains reopened for per-game schema coverage.
