Read these files after verification succeeds:

- `outputs/games/multi-game/spec.md`
- `outputs/games/multi-game/review.md`
- `outputs/games/multi-game/implementation.md`

Inspect the code changes across the owned surfaces:
- `crates/myosu-games-liars-dice/`
- `crates/myosu-games/src/registry.rs`
- `crates/myosu-play/src/spectate.rs`
- `crates/myosu-tui/src/screens/spectate.rs`

Write or replace `outputs/games/multi-game/verification.md`.

The `verification.md` must say:

- which proof commands passed
- what the implemented slice now guarantees
- which portability or follow-on risks remain
- what the next implementation slice should be

Keep the artifact tied to actual files and proof output from this repository.