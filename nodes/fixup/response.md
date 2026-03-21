Updated the current `games:multi-game` fixup without expanding beyond slice 1. I only changed [implementation.md](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/outputs/games/multi-game/implementation.md) and [verification.md](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/outputs/games/multi-game/verification.md) to make the slice boundary explicit and to document the real verification blocker honestly.

The artifacts now record that:
- slice 1 remains the `myosu-games-liars-dice` crate skeleton only
- this checkout’s workspace members are `myosu-games`, `myosu-games-liars-dice`, `myosu-tui`, and `pallet-game-solver`
- the deterministic verify failure came from later or absent package surfaces like `myosu-play`, not from the slice-1 crate wiring
- `quality.md` and `promotion.md` were intentionally not authored in this fixup

Verification I ran:
- `cargo build -p myosu-games-liars-dice` failed in this sandbox because the default Cargo target dir is read-only
- `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo build -p myosu-games-liars-dice` passed
- `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice` passed
- `cargo metadata --no-deps --format-version 1` confirmed the current workspace package inventory
- `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-play` failed with `package ID specification 'myosu-play' did not match any packages`, which matches the reported verify-stage blocker

No source code changed in this fixup.