# `play:tui` Promotion Artifact

## Recommendation

Promote the lane from the earlier renderer-only state into Slice 3 complete, then continue to Slice 4 after review.

## Why This Slice Can Advance

- The training slice now uses robopoker’s real gameplay engine instead of mock renderer-only state.
- The build succeeds with the new slice wired through `myosu-play --train`.
- The automated training proof covers fold completion, showdown completion, Unicode `/deal` handling, fallback messaging, practice chip updates, and alternating button ownership.
- Touched surfaces stayed inside `crates/myosu-play/` plus the expected dependency lockfile update.

## Constraints to Carry Forward

- `/stack` and `/bot-stack` stay blocked on public stack-setting support in `rbp_gameplay::Game`.
- The shell still renders a startup snapshot from the table; live state mutation inside the TUI belongs to the next round of game-loop plumbing.
- Blueprint loading, advisor output, recorder persistence, and chain integration remain outside this slice.

## Next Approved Target

Slice 4: `BlueprintBackend` with graceful fallback.
