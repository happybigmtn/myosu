Created [agent-adapter.md](/home/r/.fabro/runs/20260320-01KM74SBWBJ9PHFKVDJ932TEXA/worktree/outputs/agent-integration/agent-adapter.md) and [review.md](/home/r/.fabro/runs/20260320-01KM74SBWBJ9PHFKVDJ932TEXA/worktree/outputs/agent-integration/review.md).

They make the frontier call explicit: product does not need another upstream unblock first. It needs an implementation-family handoff next, starting with `play:tui` Slice 1 to create `myosu-play`, then `agent:experience` once that binary exists. The review also corrects a stale blocker from the earlier agent review: `myosu-games` is already on pinned robopoker git deps, so that portability issue is no longer the thing holding product back.

Verification I ran while writing this:
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-games cargo test -p myosu-games` passed: 10 unit tests, 4 doctests
- `CARGO_TARGET_DIR=/tmp/myosu-agent-integration-tui cargo test -p myosu-tui` passed: 82 tests, 2 ignored TTY-only tests

The next honest slice is to scaffold the `play:tui` implementation-family assets in `fabro/`.