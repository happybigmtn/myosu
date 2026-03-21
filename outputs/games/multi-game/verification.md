# `games:multi-game` Verification — Slice 1

## Proof Commands That Passed

The default environment exported `CARGO_TARGET_DIR=/home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target`, which was read-only from this sandbox. To make the proof reproducible in this worktree, the commands below were run with a local writable target dir:

`CARGO_TARGET_DIR=/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/target/codex-multi-game`

| Command | Exit Code | Result |
|---------|-----------|--------|
| `CARGO_TARGET_DIR=/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/target/codex-multi-game cargo build -p myosu-games-liars-dice` | 0 | Passed; new crate compiled successfully |
| `CARGO_TARGET_DIR=/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/target/codex-multi-game cargo test -p myosu-games-liars-dice` | 0 | Passed; 2 unit tests passed, 0 doctests |
| `CARGO_TARGET_DIR=/home/r/.fabro/runs/20260320-01KM71XNRG6FY2B05MJEVXYBBS/worktree/target/codex-multi-game cargo test -p myosu-games` | 0 | Passed; 10 unit tests and 4 doctests still passed after workspace update |

## Automated Outcomes

- The new package is now recognized by Cargo and can be built by package name.
- The crate's initial public surface compiles and its registry hook matches `GameType::LiarsDice`.
- The existing `myosu-games` crate still passes its full current test suite after the workspace membership change.

## Environment Note

The first unmodified `cargo build -p myosu-games-liars-dice` attempt failed with:

```text
error: failed to open: /home/r/coding/myosu/.worktrees/autodev-live/.raspberry/cargo-target/debug/.cargo-lock

Caused by:
  Read-only file system (os error 30)
```

That failure was environmental, not code-related. Re-running with a writable local `CARGO_TARGET_DIR` resolved it.

## Risks Reduced

- **Critical blocker: missing `myosu-games-liars-dice` crate** — reduced. The crate now exists and is wired into the workspace.
- **Bootstrap proof gap** — reduced. The reviewed Slice 1 proof command (`cargo build -p myosu-games-liars-dice`) now succeeds when run against a writable target directory.
- **Public API churn risk** — reduced. The lane now has stable exported type names for later slices to fill in.

## Risks That Remain

- **MG-01 game engine work** is still pending. No `CfrGame: Copy` implementation exists yet.
- **MG-02 solver and Nash verification** are still pending. No encoder or profile logic exists yet.
- **CS-01 cross-game scoring** is still pending. `ExploitMetric` has not been added to `myosu-games`.
- **SP-01/SP-02 spectator work** is still pending. No `myosu-play` or `myosu-tui` spectator surfaces were touched in this slice.

## Next Slice

**Slice 2 — `game.rs` + `edge.rs` + `turn.rs` + `info.rs`**

That is the next approved implementation increment from `outputs/games/multi-game/spec.md` and `outputs/games/multi-game/review.md`.
