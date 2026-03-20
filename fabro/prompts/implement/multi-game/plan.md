Read these inputs before you make changes:

- `outputs/games/multi-game/spec.md`
- `outputs/games/multi-game/review.md`
- `crates/myosu-games-liars-dice/`

Your job is to implement the smallest approved next slice for the
`games:multi-game` lane.

Rules:

1. Treat `spec.md` as the lane contract and `review.md` as the trust boundary.
2. Keep the change centered on `crates/myosu-games-liars-dice/` unless the chosen slice
   needs a tiny adjacent edit.
3. Do real code changes, not only analysis or planning.
4. Keep scope narrow. Do not try to implement multiple slices in one run.
5. After making code changes, write or replace
   `outputs/games/multi-game/implementation.md`.

The `implementation.md` must say:

- what slice was implemented
- which files changed
- what proof commands are expected for this lane
- what remains for the next slice

Do not leave placeholders.
