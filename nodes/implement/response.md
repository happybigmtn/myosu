Implemented the approved `games:multi-game` Slice 1. The slice adds the new workspace member in [Cargo.toml](/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/Cargo.toml#L3), defines the skeleton package in [crates/myosu-games-liars-dice/Cargo.toml](/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/crates/myosu-games-liars-dice/Cargo.toml#L1) and [crates/myosu-games-liars-dice/src/lib.rs](/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/crates/myosu-games-liars-dice/src/lib.rs#L1), and records the new package in [Cargo.lock](/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/Cargo.lock#L1721). The public surface is intentionally stubbed for this slice: `LiarsDiceGame`, `LiarsDiceEdge`, `LiarsDiceTurn`, `LiarsDiceInfo`, `LiarsDiceEncoder`, `LiarsDiceProfile`, plus `GAME_TYPE = GameType::LiarsDice`.

I also wrote the owned curated artifacts [implementation.md](/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/outputs/games/multi-game/implementation.md#L1) and [verification.md](/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/outputs/games/multi-game/verification.md#L1). `quality.md` and `promotion.md` were intentionally left untouched per stage ownership.

Verification passed for:
- `cargo build -p myosu-games-liars-dice`
- `cargo test -p myosu-games-liars-dice`
- `cargo test -p myosu-games`

Those were run with `CARGO_TARGET_DIR=/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/target/codex-multi-game` because the environment’s default shared target dir was read-only. The next approved slice is the actual Liar’s Dice game engine work in `game.rs`, `edge.rs`, `turn.rs`, and `info.rs`.