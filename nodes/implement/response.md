Implemented the next approved `games:multi-game` slice: Slice 1, the `myosu-games-liars-dice` crate skeleton. I added the workspace member in [Cargo.toml](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/Cargo.toml), recorded the new package in [Cargo.lock](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/Cargo.lock), created the new crate manifest at [crates/myosu-games-liars-dice/Cargo.toml](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/crates/myosu-games-liars-dice/Cargo.toml), and added placeholder public API types in [crates/myosu-games-liars-dice/src/lib.rs](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/crates/myosu-games-liars-dice/src/lib.rs). No `myosu-games/src` or poker-engine surfaces were changed.

Curated artifacts are in [implementation.md](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/outputs/games/multi-game/implementation.md) and [verification.md](/home/r/.fabro/runs/20260320-01KM708EFRSGC8ZK0BFM57TK6A/worktree/outputs/games/multi-game/verification.md). I intentionally left `quality.md` and `promotion.md` untouched because the lane instructions reserve those for later stages.

Verification passed with:
- `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo build -p myosu-games-liars-dice`
- `env CARGO_TARGET_DIR=/tmp/myosu-multi-game-target cargo test -p myosu-games-liars-dice`

Both exited `0`; the new crate’s smoke test passed.