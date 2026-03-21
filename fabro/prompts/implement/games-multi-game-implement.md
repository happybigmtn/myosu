Read these inputs before you make changes:

- `outputs/games/multi-game/spec.md`
- `outputs/games/multi-game/review.md`
- `crates/myosu-games-liars-dice/`
- `crates/myosu-games/src/registry.rs`
- `crates/myosu-play/src/spectate.rs`
- `crates/myosu-tui/src/screens/spectate.rs`

Your job is to implement the smallest approved next slice for the
`games:multi-game` lane.

Rules:

1. Treat `spec.md` as the lane contract and `review.md` as the trust boundary.
2. Keep the change centered on the owned surfaces for `games:multi-game`:
   - `crates/myosu-games-liars-dice/` (Liar's Dice game engine)
   - `crates/myosu-games/src/registry.rs` (ExploitMetric registration)
   - `crates/myosu-play/src/spectate.rs` (spectator relay)
   - `crates/myosu-tui/src/screens/spectate.rs` (spectator TUI)
3. Do real code changes, not only analysis or planning.
4. Keep scope narrow. Do not try to implement every future game or feature in one run.
5. After making code changes, write or replace
   `outputs/games/multi-game/implementation.md`.

The `implementation.md` must say:

- what slice was implemented
- which files changed
- what proof commands are expected for this lane
- what remains for the next slice

Do not leave placeholders.