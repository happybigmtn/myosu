The previous verify step failed for the `games:multi-game` implementation lane.

Read:

- the verify output from context
- `outputs/games/multi-game/spec.md`
- `outputs/games/multi-game/review.md`
- `outputs/games/multi-game/implementation.md`

Your job is to fix the concrete proof failures without widening scope.

Rules:

1. Repair the current slice. Do not restart the lane from scratch.
2. Keep edits centered on `crates/myosu-games-liars-dice/` unless a tiny adjacent fix is
   clearly required.
3. Update `outputs/games/multi-game/implementation.md` if the implemented slice or
   changed files differ from what it currently says.
4. Leave the lane in a state where the verify command can pass cleanly.
