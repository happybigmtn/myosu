All three proof commands pass cleanly:
- `cargo build -p myosu-play` — exit 0
- `cargo test -p myosu-games` — 10 tests + 4 doc-tests pass
- `cargo test -p myosu-tui` — 82 tests + 1 doc-test pass (2 TTY-required tests correctly ignored)

The lane scripts are now aligned with the corrected `verification.md` artifact.

**Summary of fix:** The `graph.fabro` lane configuration had stale verification scripts that referenced `myosu-games-poker` (Slice 2) and training/blueprint/advisor tests (Slices 3–5) which don't exist in the workspace. Both `preflight` and `verify` script attributes were updated to only reference Slice 1–scoped commands matching the corrected `verification.md`.