Created [foundation-plan.md](/home/r/.fabro/runs/20260321-01KM78X5SKEEPH0ZQ9H3B6CGXG/worktree/outputs/foundations/foundation-plan.md) and [review.md](/home/r/.fabro/runs/20260321-01KM78X5SKEEPH0ZQ9H3B6CGXG/worktree/outputs/foundations/review.md). The review keeps the frontier `REOPEN`: the doctrine and reviewed outputs are real, but `raspberry status`/`execute` still stall without rendering truthful operator output, so the `games:multi-game` false-submit is not honestly repaired yet.

I also hardened [games-traits.sh](/home/r/.fabro/runs/20260321-01KM78X5SKEEPH0ZQ9H3B6CGXG/worktree/fabro/checks/games-traits.sh) and [tui-shell.sh](/home/r/.fabro/runs/20260321-01KM78X5SKEEPH0ZQ9H3B6CGXG/worktree/fabro/checks/tui-shell.sh) to use the current repo’s `.raspberry/cargo-target` instead of inheriting the stale external `CARGO_TARGET_DIR`. That let the local proofs run cleanly here.

Verification:
- `./fabro/checks/games-traits.sh` passed
- `./fabro/checks/tui-shell.sh` passed
- `./fabro/checks/chain-runtime-reset.sh` passed
- `./fabro/checks/chain-pallet-reset.sh` passed
- `raspberry status --manifest fabro/programs/myosu-bootstrap.yaml` still timed out after mutating `.raspberry/myosu-bootstrap-state.json`
- `raspberry execute --manifest fabro/programs/myosu-platform.yaml --lane games:multi-game` still timed out with no truthful submission/failure output

The remaining repair is in `/home/r/coding/fabro`, which I could inspect but not edit from this sandbox.