Created [foundation-plan.md](/home/r/.fabro/runs/20260320-01KM784VTSAJRVJDZEYDM26R5K/worktree/outputs/foundations/foundation-plan.md) and [review.md](/home/r/.fabro/runs/20260320-01KM784VTSAJRVJDZEYDM26R5K/worktree/outputs/foundations/review.md) as the durable foundations artifacts. I also hardened the local proof scripts in [games-traits.sh](/home/r/.fabro/runs/20260320-01KM784VTSAJRVJDZEYDM26R5K/worktree/fabro/checks/games-traits.sh), [games-traits-implement.sh](/home/r/.fabro/runs/20260320-01KM784VTSAJRVJDZEYDM26R5K/worktree/fabro/checks/games-traits-implement.sh), and [tui-shell.sh](/home/r/.fabro/runs/20260320-01KM784VTSAJRVJDZEYDM26R5K/worktree/fabro/checks/tui-shell.sh) so they always use a repo-local `.raspberry/cargo-target`, and corrected the operator commands in [README.md](/home/r/.fabro/runs/20260320-01KM784VTSAJRVJDZEYDM26R5K/worktree/README.md) and [fabro/README.md](/home/r/.fabro/runs/20260320-01KM784VTSAJRVJDZEYDM26R5K/worktree/fabro/README.md) to the valid `cargo run --manifest-path ...` form with an absolute manifest path.

Verified locally:
- `./fabro/checks/games-traits.sh` passes
- `./fabro/checks/tui-shell.sh` passes
- `./fabro/checks/games-traits-implement.sh` passes
- `/home/r/coding/fabro/target-local/debug/raspberry status --manifest "$PWD/fabro/programs/myosu.yaml"` reports `complete=7 ready=0 running=0 blocked=0 failed=0`
- `watch --iterations 1` reports the same truth
- `execute` now fails truthfully with `Error: no ready lanes selected for execution`

Remaining gap: I did not re-exercise a fresh ready-lane detach run, so the historical `games:multi-game` false-submit remains documented as residual risk rather than newly disproven in this turn.