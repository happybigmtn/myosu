# `tui:shell` Verification — Slice 1 Fixup

## Verification Context

The active run's verify-stage failure is deterministic, but it is caused by the
outer run graph rather than the `events.rs` slice. The recorded commands still
use invalid Cargo flags and unscoped workspace-root test invocations:

- `cargo test events:: --no-ignore`
- `cargo test shell:: --integration`

The recorded preflight stderr also shows that once the script continues past the
invalid flags, unscoped workspace-root test commands reach
`crates/myosu-chain/pallets/game-solver` and fail on unrelated compile errors.

All commands below were rerun from the workspace root against
`-p myosu-tui`. In this sandbox, Cargo needs
`CARGO_TARGET_DIR=/tmp/myosu-target` because the default target path resolves
outside the writable root.

## Automated Proof Commands

| Command | Exit Code | Result |
|---------|-----------|--------|
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui events::` | 0 | Focused `events` regression passed: 6 tests |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui events:: -- --include-ignored` | 0 | Slice 1 proof gate passed with Cargo-valid syntax: same 6 `events` tests and 0 ignored |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui shell::` | 0 | Existing shell-module regression passed: 11 tests; the review's missing transition/render gaps remain open |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui all_game_types_have_schema` | 0 | Existing schema coverage smoke test passed: 1 test; semantic per-game coverage remains open |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui shell_draw_` | 0 | Existing shell draw regression passed: 2 tests; non-Game screen coverage remains open |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui is_plain_text` | 0 | Existing pipe ANSI-detection regression passed: 1 test |
| `CARGO_TARGET_DIR=/tmp/myosu-target cargo test -p myosu-tui` | 0 | Full crate regression passed: 86 unit tests and 1 doctest |

## Proof Outcome

Slice 1 is verified in its current form:

- tick delivery is proven headlessly
- key delivery is proven headlessly
- resize delivery is proven headlessly
- async update injection is proven headlessly
- the doctest for `EventLoop::new` compiles cleanly in headless verification

The remaining lane gaps identified by review are still accurate:

- `shell.rs` still lacks the typed Lobby-to-Game transition proof called out in review
- `shell.rs` still lacks all-screen render coverage
- `schema.rs` still overclaims per-game proof relative to the tests
- `pipe.rs` still has only its existing ANSI-detection regression, not the Slice 5 property test

## Residual Risks

- This slice proves the shared event-loop logic without a live terminal harness; runtime use still depends on `crossterm::event::EventStream`.
- The checked-in lane artifacts now describe package-scoped proof commands, but the current run's external graph still needs to be rerendered before automation will stop replaying the stale commands.
- `shell.rs` remains reopened for the separate transition and render coverage gaps called out in review.
- `schema.rs` remains reopened for per-game schema coverage.
