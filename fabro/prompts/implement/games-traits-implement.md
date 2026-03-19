Read these inputs before you make changes:

- `outputs/games/traits/spec.md`
- `outputs/games/traits/review.md`
- `crates/myosu-games/`

Your job is to implement the smallest approved next slice for the
`games:traits` lane.

Rules:

1. Treat `spec.md` as the lane contract and `review.md` as the trust boundary.
2. Keep the change centered on `crates/myosu-games/` unless the chosen slice
   needs a tiny adjacent edit.
3. Do real code changes, not only analysis or planning.
4. Keep scope narrow. Do not try to solve every future trait or every future
   portability issue in one run.
5. After making code changes, write or replace
   `outputs/games/traits/implementation.md`.

The `implementation.md` must say:

- what slice was implemented
- which files changed
- what proof commands are expected for this lane
- what remains for the next slice

Do not leave placeholders.
