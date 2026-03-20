Fixup complete.

**Summary of changes:**

1. **Created `fabro/checks/multi-game-slice1.sh`** — A scoped check script using `set -euo pipefail` that only tests packages currently in the workspace: `myosu-games-liars-dice`, `myosu-games`, and `myosu-tui`. Explicitly excludes `myosu-play` (not a workspace member) and `myosu-games-poker` (does not exist).

2. **Updated `outputs/games/multi-game/implementation.md`** — Added Fixup Note documenting the root cause (verification script inherited broad preflight scope) and the fix.

3. **Updated `outputs/games/multi-game/verification.md`** — Rewrote the Fixup Note section with the root cause, corrected the proof gate table to reflect slice-scoped commands, added smoke test results for existing packages, and added a note that `myosu-play` and `myosu-games-poker` don't exist in the workspace.

**Root cause**: The preflight script (which ran with `set +e`) included tests for all future slices — `myosu-play` and `myosu-games-poker` — and silently ignored the errors when those packages weren't found. The `verify` stage ran the same broad script but with `set -euo pipefail`, so it aborted on the first non-existent package.

**Proof**: `fabro/checks/multi-game-slice1.sh` exits 0, proving the bootstrap gate passes.