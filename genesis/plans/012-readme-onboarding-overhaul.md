# 012 — README and Onboarding Overhaul

## Objective

Make the README honest and useful for a new developer encountering the repo for the first time. Reduce time-to-first-success from "unknown" to "under 5 minutes on warm cache."

## Context

The current README is accurate in its proof commands but:

1. Does not list prerequisites (Rust version, WASM target, protoc)
2. References `fabro run` commands that don't work (directory missing)
3. Does not document the fastest path to a meaningful result
4. Does not clearly separate "developer trying the repo" from "operator running infrastructure"

The operator guide (`docs/operator-guide/quickstart.md`) is comprehensive but lives too deep in the tree for first-contact.

## Acceptance Criteria

- README.md includes a "Prerequisites" section listing: Rust (edition 2024), WASM target, protoc, platform notes
- README.md includes a "Quick Verify" section with the fastest passing command: `cargo test -p myosu-games-kuhn --quiet` (~30s warm cache, exercises game traits + solver + wire codec)
- README.md removes or replaces all `fabro run` commands with working alternatives
- README.md has a clear separation between "developer" and "operator" paths
- The operator path points to `docs/operator-guide/quickstart.md` with a one-line description
- No command in README.md fails on a fresh checkout with prerequisites installed

## Verification

```bash
# Every command in README should succeed
# Extract commands and run them (manual review step)
grep '```bash' -A 5 README.md | grep -v '```' | head -20

# Specifically verify the quick-verify command works
cargo test -p myosu-games-kuhn --quiet
```

## Dependencies

- Plan 006 (Phase 1 gate) — README should describe the post-cleanup codebase, not the current one
